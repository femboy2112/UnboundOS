//! UMOD — symbolic graph format. Spec §6.
//!
//! Persistent format only: fixed-width integers, offsets, lengths,
//! type IDs, checksums, capability declarations. **No pointers, no
//! function pointers, no host paths, no live runtime addresses.**
//! Layout is authoritative; the verifier (in `graph` crate) validates
//! semantics.

#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

/// Magic bytes at file offset 0: ASCII `UMOD` = `[0x55, 0x4D, 0x4F,
/// 0x44]` on disk; reads as `0x444F4D55` when interpreted as a u32 LE.
pub const UMOD_MAGIC: [u8; 4] = *b"UMOD";

pub const UMOD_FORMAT_MAJOR: u16 = 1;
pub const UMOD_FORMAT_MINOR: u16 = 0;

/// Header at file offset 0. Spec §6.3, exact byte offsets.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UmodHeader {
    pub magic: [u8; 4],            // 0x00
    pub format_major: u16,         // 0x04
    pub format_minor: u16,         // 0x06
    pub header_length: u32,        // 0x08
    pub section_count: u32,        // 0x0C
    pub node_count: u32,           // 0x10
    pub wire_count: u32,           // 0x14
    pub pin_type_count: u32,       // 0x18
    pub capability_count: u32,     // 0x1C
    pub section_table_offset: u64, // 0x20
    pub file_length_bytes: u64,    // 0x28
    pub graph_stable_id: u64,      // 0x30
    pub header_checksum: u64,      // 0x38
}

const _: () = assert!(core::mem::size_of::<UmodHeader>() == 0x40);

/// Section descriptor — 24 bytes per spec §6.4.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SectionDescriptor {
    pub kind: u32,
    pub flags: u32,
    pub offset: u64,
    pub length: u64,
    pub checksum: u64,
}

const _: () = assert!(core::mem::size_of::<SectionDescriptor>() == 0x20);

/// Node descriptor — spec §6.5.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct NodeDescriptor {
    pub node_id: u32,
    pub node_type_id: u32,
    pub input_pin_base: u32,
    pub input_pin_count: u16,
    pub output_pin_base: u32,
    pub output_pin_count: u16,
    pub capability_base: u32,
    pub capability_count: u16,
    pub reserved0: u16,
    pub ui_x: i32,
    pub ui_y: i32,
    pub label_offset: u32,
    pub constant_ref: u32,
}

/// Wire descriptor — spec §6.6.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct WireDescriptor {
    pub wire_id: u32,
    pub src_node_id: u32,
    pub src_pin_index: u16,
    pub dst_pin_index: u16,
    pub dst_node_id: u32,
    pub type_id: u32,
    pub payload_size: u32,
    pub alignment: u32,
    pub flags: u32,
}

/// Approved opaque-resource grammar (spec §6.8):
/// `resource_ref = resource_type ":" opaque_id`
/// where `resource_type ∈ {model, graph, index, blob, font, profile}`
/// and `opaque_id` is 1–64 chars from `[A-Za-z0-9_.-]`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ResourceType {
    Model,
    Graph,
    Index,
    Blob,
    Font,
    Profile,
}

#[derive(Copy, Clone, Debug)]
pub struct ResourceRef<'a> {
    pub kind: ResourceType,
    pub opaque_id: &'a [u8],
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ResourceRefError {
    Empty,
    MissingColon,
    UnknownType,
    OpaqueIdEmpty,
    OpaqueIdTooLong,
    OpaqueIdInvalidChar,
    LooksLikeAPath,
    NonAsciiPrintable,
}

/// Parse and validate a resource reference. Rejects POSIX-style
/// paths, `local://`, `..`, `~`, and any non-ASCII-printable byte.
///
/// Stub. Real implementation: locate `:`, match prefix to
/// `ResourceType`, validate `opaque_id` charset/length, reject
/// path-shaped inputs.
///
/// # Errors
///
/// Returns [`ResourceRefError`] when the bytes do not match the approved
/// opaque-resource grammar.
pub fn parse_resource_ref(_bytes: &[u8]) -> Result<ResourceRef<'_>, ResourceRefError> {
    Err(ResourceRefError::Empty)
}

/// Parser-stage errors (structural). The verifier in `graph` crate
/// produces a richer error covering semantic checks (spec §5.6).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UmodParseError {
    BadMagic,
    UnsupportedVersion { major: u16, minor: u16 },
    HeaderTooShort,
    SectionTableOutOfBounds,
    SectionOutOfBounds { index: u32 },
    NodeCountOverflow,
    WireCountOverflow,
    HeaderChecksumMismatch,
    SectionChecksumMismatch { index: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_size_matches_spec() {
        assert_eq!(core::mem::size_of::<UmodHeader>(), 0x40);
    }

    #[test]
    fn section_descriptor_size_matches_spec() {
        assert_eq!(core::mem::size_of::<SectionDescriptor>(), 0x20);
    }

    #[test]
    fn magic_bytes_are_stable() {
        assert_eq!(UMOD_MAGIC, [b'U', b'M', b'O', b'D']);
    }
}
