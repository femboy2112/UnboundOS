//! Minimal Multiboot2 memory-map parser for the GRUB smoke boot path.
//!
//! The spec-primary bootloader handoff is Limine, but the current executable
//! image is a GRUB Multiboot2 ISO. This parser keeps the QEMU gate honest until
//! the Limine path lands: boot memory diagnostics must be derived from the
//! bootloader-provided map, not hard-coded.

use core::cmp::{max, min};

pub const BOOTLOADER_MAGIC: u32 = 0x36d7_6289;
const TAG_TYPE_END: u32 = 0;
const TAG_TYPE_MMAP: u32 = 6;
const MIN_INFO_SIZE: usize = 16;
const MAX_INFO_SIZE: usize = 1024 * 1024;
const INFO_HEADER_SIZE: usize = 8;
const TAG_HEADER_SIZE: usize = 8;
const MMAP_TAG_HEADER_SIZE: usize = 16;
const MIN_MMAP_ENTRY_SIZE: u32 = 24;

pub const IDENTITY_MAPPED_LIMIT: u64 = 1024 * 1024 * 1024;
pub const M2_BOOT_ARENA_BYTES: u64 = 64 * 1024;
pub const M2_KERNEL_ARENA_BYTES: u64 = 256 * 1024;
pub const M2_GRAPH_ARENA_BYTES: u64 = 256 * 1024;
pub const M2_SCRATCH_ARENA_BYTES: u64 = 128 * 1024;
pub const M2_ARENA_BYTES: u64 =
    M2_BOOT_ARENA_BYTES + M2_KERNEL_ARENA_BYTES + M2_GRAPH_ARENA_BYTES + M2_SCRATCH_ARENA_BYTES;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct UsableRegion {
    pub base: u64,
    pub length: u64,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct MemorySummary {
    pub handoff_valid: bool,
    pub memmap_present: bool,
    pub total_size: u32,
    pub usable_bytes: u64,
    pub usable_regions: u32,
    pub arena_region: Option<UsableRegion>,
}

impl MemorySummary {
    pub const fn invalid() -> Self {
        Self {
            handoff_valid: false,
            memmap_present: false,
            total_size: 0,
            usable_bytes: 0,
            usable_regions: 0,
            arena_region: None,
        }
    }
}

/// Summarize a raw Multiboot2 information block.
///
/// # Safety
///
/// `info_addr` must point to the bootloader-provided Multiboot2 information
/// block for this boot. The block must remain readable while this function
/// executes. The parser bounds reads by the header `total_size` and rejects
/// implausibly large blocks before building a slice.
pub unsafe fn summarize_raw(
    magic: u32,
    info_addr: u32,
    min_arena_base: u64,
    max_arena_limit: u64,
) -> MemorySummary {
    if magic != BOOTLOADER_MAGIC || info_addr == 0 {
        return MemorySummary::invalid();
    }

    let info_ptr = info_addr as usize as *const u8;
    // SAFETY: caller guarantees `info_addr` is a readable Multiboot2 info
    // block. An unaligned read is used because the parser should not assume
    // more than byte readability before it validates the header.
    let total_size = unsafe { read_u32(info_ptr) };
    if total_size as usize > MAX_INFO_SIZE || (total_size as usize) < MIN_INFO_SIZE {
        return MemorySummary::invalid();
    }

    // SAFETY: total_size has been bounded above and below, and the caller's
    // boot-handoff contract makes the whole info block readable.
    let bytes = unsafe { core::slice::from_raw_parts(info_ptr, total_size as usize) };
    summarize_bytes(magic, bytes, min_arena_base, max_arena_limit)
}

pub fn summarize_bytes(
    magic: u32,
    bytes: &[u8],
    min_arena_base: u64,
    max_arena_limit: u64,
) -> MemorySummary {
    if magic != BOOTLOADER_MAGIC || bytes.len() < MIN_INFO_SIZE {
        return MemorySummary::invalid();
    }

    let total_size = read_u32_from(bytes, 0);
    let Some(total_size_usize) = usize::try_from(total_size).ok() else {
        return MemorySummary::invalid();
    };
    if total_size_usize < MIN_INFO_SIZE
        || total_size_usize > bytes.len()
        || total_size_usize > MAX_INFO_SIZE
    {
        return MemorySummary::invalid();
    }

    let mut summary = MemorySummary {
        handoff_valid: true,
        memmap_present: false,
        total_size,
        usable_bytes: 0,
        usable_regions: 0,
        arena_region: None,
    };

    let mut offset = INFO_HEADER_SIZE;
    while offset + TAG_HEADER_SIZE <= total_size_usize {
        let typ = read_u32_from(bytes, offset);
        let size_u32 = read_u32_from(bytes, offset + 4);
        let Some(size) = usize::try_from(size_u32).ok() else {
            return MemorySummary::invalid();
        };
        if size < TAG_HEADER_SIZE || offset + size > total_size_usize {
            return MemorySummary::invalid();
        }

        if typ == TAG_TYPE_END {
            return summary;
        }
        if typ == TAG_TYPE_MMAP {
            summary.memmap_present = true;
            parse_mmap_tag(
                &mut summary,
                bytes,
                offset,
                size,
                min_arena_base,
                max_arena_limit,
            );
        }

        let Some(next) = align_up_usize(offset + size, 8) else {
            return MemorySummary::invalid();
        };
        offset = next;
    }

    MemorySummary::invalid()
}

fn parse_mmap_tag(
    summary: &mut MemorySummary,
    bytes: &[u8],
    tag_offset: usize,
    tag_size: usize,
    min_arena_base: u64,
    max_arena_limit: u64,
) {
    if tag_size < MMAP_TAG_HEADER_SIZE {
        return;
    }

    let entry_size = read_u32_from(bytes, tag_offset + 8);
    if entry_size < MIN_MMAP_ENTRY_SIZE {
        return;
    }
    let Some(entry_size_usize) = usize::try_from(entry_size).ok() else {
        return;
    };

    let tag_end = tag_offset + tag_size;
    let mut entry_offset = tag_offset + MMAP_TAG_HEADER_SIZE;
    while entry_offset + entry_size_usize <= tag_end {
        let base = read_u64_from(bytes, entry_offset);
        let length = read_u64_from(bytes, entry_offset + 8);
        let entry_type = read_u32_from(bytes, entry_offset + 16);
        if entry_type == 1 && length > 0 {
            summary.usable_regions = summary.usable_regions.saturating_add(1);
            summary.usable_bytes = summary.usable_bytes.saturating_add(length);
            select_arena_region(summary, base, length, min_arena_base, max_arena_limit);
        }
        entry_offset += entry_size_usize;
    }
}

fn select_arena_region(
    summary: &mut MemorySummary,
    base: u64,
    length: u64,
    min_arena_base: u64,
    max_arena_limit: u64,
) {
    let Some(region_end) = base.checked_add(length) else {
        return;
    };
    let start = max(base, min_arena_base);
    let end = min(region_end, max_arena_limit);
    let Some(aligned_start) = align_up_u64(start, 4096) else {
        return;
    };
    let aligned_end = end & !0xfff;
    let Some(required_end) = aligned_start.checked_add(M2_ARENA_BYTES) else {
        return;
    };
    if required_end > aligned_end {
        return;
    }

    let candidate = UsableRegion {
        base: aligned_start,
        length: aligned_end - aligned_start,
    };
    if match summary.arena_region {
        Some(current) => candidate.base < current.base,
        None => true,
    } {
        summary.arena_region = Some(candidate);
    }
}

fn align_up_usize(value: usize, alignment: usize) -> Option<usize> {
    let mask = alignment - 1;
    value.checked_add(mask).map(|v| v & !mask)
}

fn align_up_u64(value: u64, alignment: u64) -> Option<u64> {
    let mask = alignment - 1;
    value.checked_add(mask).map(|v| v & !mask)
}

unsafe fn read_u32(ptr: *const u8) -> u32 {
    // SAFETY: caller establishes that four bytes at `ptr` are readable.
    unsafe { core::ptr::read_unaligned(ptr.cast::<u32>()) }
}

fn read_u32_from(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("u32 bounds checked"),
    )
}

fn read_u64_from(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(
        bytes[offset..offset + 8]
            .try_into()
            .expect("u64 bounds checked"),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        summarize_bytes, BOOTLOADER_MAGIC, IDENTITY_MAPPED_LIMIT, M2_ARENA_BYTES,
        M2_BOOT_ARENA_BYTES, M2_GRAPH_ARENA_BYTES, M2_KERNEL_ARENA_BYTES, M2_SCRATCH_ARENA_BYTES,
    };

    #[test]
    fn parses_usable_memory_and_selects_aligned_arena_region() {
        let bytes = info_with_mmap(&[
            (0x0000_0000, 0x0009_fc00, 1),
            (0x0010_0000, 0x1ff0_0000, 1),
            (0x2000_0000, 0x0100_0000, 2),
        ]);

        let summary = summarize_bytes(BOOTLOADER_MAGIC, &bytes, 0x0031_2345, IDENTITY_MAPPED_LIMIT);

        assert!(summary.handoff_valid);
        assert!(summary.memmap_present);
        assert_eq!(summary.usable_regions, 2);
        assert_eq!(summary.usable_bytes, 0x0009_fc00 + 0x1ff0_0000);
        let arena_region = summary.arena_region.unwrap();
        assert_eq!(arena_region.base, 0x0031_3000);
        assert!(arena_region.length >= M2_ARENA_BYTES);
    }

    #[test]
    fn rejects_bad_magic() {
        let bytes = info_with_mmap(&[(0x0010_0000, 0x0100_0000, 1)]);

        let summary = summarize_bytes(0, &bytes, 0, IDENTITY_MAPPED_LIMIT);

        assert!(!summary.handoff_valid);
        assert!(!summary.memmap_present);
        assert_eq!(summary.usable_bytes, 0);
    }

    #[test]
    fn requires_enough_identity_mapped_space_for_m2_arenas() {
        let bytes = info_with_mmap(&[(0x3ff8_0000, 0x0008_0000, 1)]);

        let summary = summarize_bytes(BOOTLOADER_MAGIC, &bytes, 0, IDENTITY_MAPPED_LIMIT);

        assert!(summary.handoff_valid);
        assert!(summary.memmap_present);
        assert_eq!(summary.arena_region, None);
    }

    #[test]
    fn arena_size_constants_cover_four_boot_arenas() {
        assert_eq!(
            M2_ARENA_BYTES,
            M2_BOOT_ARENA_BYTES
                + M2_KERNEL_ARENA_BYTES
                + M2_GRAPH_ARENA_BYTES
                + M2_SCRATCH_ARENA_BYTES
        );
    }

    fn info_with_mmap(entries: &[(u64, u64, u32)]) -> Vec<u8> {
        let mmap_tag_size = 16 + entries.len() * 24;
        let aligned_mmap_tag_size = align_up(mmap_tag_size, 8);
        let total_size = 8 + aligned_mmap_tag_size + 8;
        let mut bytes = vec![0_u8; total_size];
        write_u32(&mut bytes, 0, total_size as u32);

        let tag_offset = 8;
        write_u32(&mut bytes, tag_offset, 6);
        write_u32(&mut bytes, tag_offset + 4, mmap_tag_size as u32);
        write_u32(&mut bytes, tag_offset + 8, 24);
        write_u32(&mut bytes, tag_offset + 12, 0);
        for (idx, (base, length, typ)) in entries.iter().copied().enumerate() {
            let entry_offset = tag_offset + 16 + idx * 24;
            write_u64(&mut bytes, entry_offset, base);
            write_u64(&mut bytes, entry_offset + 8, length);
            write_u32(&mut bytes, entry_offset + 16, typ);
        }

        let end_offset = 8 + aligned_mmap_tag_size;
        write_u32(&mut bytes, end_offset, 0);
        write_u32(&mut bytes, end_offset + 4, 8);
        bytes
    }

    fn align_up(value: usize, alignment: usize) -> usize {
        (value + alignment - 1) & !(alignment - 1)
    }

    fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn write_u64(bytes: &mut [u8], offset: usize, value: u64) {
        bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }
}
