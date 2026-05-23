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
pub const UMOD_HEADER_LEN: usize = 0x40;
pub const UMOD_HEADER_LEN_U32: u32 = 0x40;

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

/// Decoded UMOD header. This is parsed from little-endian bytes without
/// casting the input buffer to an artifact struct.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ParsedUmodHeader {
    pub magic: [u8; 4],
    pub format_major: u16,
    pub format_minor: u16,
    pub header_length: u32,
    pub section_count: u32,
    pub node_count: u32,
    pub wire_count: u32,
    pub pin_type_count: u32,
    pub capability_count: u32,
    pub section_table_offset: u64,
    pub file_length_bytes: u64,
    pub graph_stable_id: u64,
    pub header_checksum: u64,
}

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

/// Node descriptor — spec §6.5. `node_type_id` is `u64` to match the
/// spec table and Appendix A's `NodeTypeId = u64`. The `pad_*` fields
/// make the implicit `#[repr(C)]` alignment padding explicit so the
/// on-disk layout is auditable.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct NodeDescriptor {
    pub node_id: u32,
    pub pad_after_node_id: u32,
    pub node_type_id: u64,
    pub input_pin_base: u32,
    pub input_pin_count: u16,
    pub pad_after_input_pin_count: u16,
    pub output_pin_base: u32,
    pub output_pin_count: u16,
    pub pad_after_output_pin_count: u16,
    pub capability_base: u32,
    pub capability_count: u16,
    pub pad_after_capability_count: u16,
    pub ui_x: i32,
    pub ui_y: i32,
    pub label_offset: u32,
    pub constant_ref: u32,
}

const _: () = assert!(core::mem::size_of::<NodeDescriptor>() == 0x38);

/// Wire descriptor — spec §6.6. Fields are in spec order:
/// `wire_id, src_node_id, src_pin_index, dst_node_id, dst_pin_index,
/// type_id, payload_size, alignment, flags`. `type_id` and
/// `payload_size` are `u64` per the spec table and Appendix A
/// (`TypeId = u64`).
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct WireDescriptor {
    pub wire_id: u32,
    pub src_node_id: u32,
    pub src_pin_index: u16,
    pub pad_after_src_pin_index: u16,
    pub dst_node_id: u32,
    pub dst_pin_index: u16,
    pub pad_after_dst_pin_index: u16,
    pub pad_before_type_id: u32,
    pub type_id: u64,
    pub payload_size: u64,
    pub alignment: u32,
    pub flags: u32,
}

const _: () = assert!(core::mem::size_of::<WireDescriptor>() == 0x30);

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
/// # Errors
///
/// Returns a `ResourceRefError` variant if the input is empty,
/// missing the `:` separator, has an unknown `resource_type`
/// prefix, has an empty/too-long/invalid-charset `opaque_id`,
/// looks like a path (`/`, `..`, `local://`, `\`), or contains
/// any non-ASCII-printable byte.
pub fn parse_resource_ref(bytes: &[u8]) -> Result<ResourceRef<'_>, ResourceRefError> {
    if bytes.is_empty() {
        return Err(ResourceRefError::Empty);
    }
    if bytes
        .iter()
        .any(|byte| !byte.is_ascii() || byte.is_ascii_control())
    {
        return Err(ResourceRefError::NonAsciiPrintable);
    }
    if looks_like_path(bytes) {
        return Err(ResourceRefError::LooksLikeAPath);
    }

    let Some(colon) = bytes.iter().position(|byte| *byte == b':') else {
        return Err(ResourceRefError::MissingColon);
    };
    let (kind_bytes, opaque_with_colon) = bytes.split_at(colon);
    let opaque_id = &opaque_with_colon[1..];
    let kind = match kind_bytes {
        b"model" => ResourceType::Model,
        b"graph" => ResourceType::Graph,
        b"index" => ResourceType::Index,
        b"blob" => ResourceType::Blob,
        b"font" => ResourceType::Font,
        b"profile" => ResourceType::Profile,
        _ => return Err(ResourceRefError::UnknownType),
    };

    if opaque_id.is_empty() {
        return Err(ResourceRefError::OpaqueIdEmpty);
    }
    if opaque_id.len() > 64 {
        return Err(ResourceRefError::OpaqueIdTooLong);
    }
    if opaque_id.iter().any(|byte| !opaque_id_char(*byte)) {
        return Err(ResourceRefError::OpaqueIdInvalidChar);
    }

    Ok(ResourceRef { kind, opaque_id })
}

/// Parse the fixed-width UMOD header from bytes.
///
/// # Errors
///
/// Returns a `UmodParseError` when the buffer is too short, the magic/version
/// is unsupported, or the declared header length does not match the UMOD v1
/// header size.
pub fn parse_header(bytes: &[u8]) -> Result<ParsedUmodHeader, UmodParseError> {
    if bytes.len() < UMOD_HEADER_LEN {
        return Err(UmodParseError::HeaderTooShort);
    }

    let magic = read_array::<4>(bytes, 0).ok_or(UmodParseError::HeaderTooShort)?;
    if magic != UMOD_MAGIC {
        return Err(UmodParseError::BadMagic);
    }

    let format_major = read_u16_le(bytes, 0x04).ok_or(UmodParseError::HeaderTooShort)?;
    let format_minor = read_u16_le(bytes, 0x06).ok_or(UmodParseError::HeaderTooShort)?;
    if format_major != UMOD_FORMAT_MAJOR || format_minor != UMOD_FORMAT_MINOR {
        return Err(UmodParseError::UnsupportedVersion {
            major: format_major,
            minor: format_minor,
        });
    }

    let header_length = read_u32_le(bytes, 0x08).ok_or(UmodParseError::HeaderTooShort)?;
    if header_length as usize != UMOD_HEADER_LEN {
        return Err(UmodParseError::BadHeaderLength {
            declared: header_length,
        });
    }

    Ok(ParsedUmodHeader {
        magic,
        format_major,
        format_minor,
        header_length,
        section_count: read_u32_le(bytes, 0x0C).ok_or(UmodParseError::HeaderTooShort)?,
        node_count: read_u32_le(bytes, 0x10).ok_or(UmodParseError::HeaderTooShort)?,
        wire_count: read_u32_le(bytes, 0x14).ok_or(UmodParseError::HeaderTooShort)?,
        pin_type_count: read_u32_le(bytes, 0x18).ok_or(UmodParseError::HeaderTooShort)?,
        capability_count: read_u32_le(bytes, 0x1C).ok_or(UmodParseError::HeaderTooShort)?,
        section_table_offset: read_u64_le(bytes, 0x20).ok_or(UmodParseError::HeaderTooShort)?,
        file_length_bytes: read_u64_le(bytes, 0x28).ok_or(UmodParseError::HeaderTooShort)?,
        graph_stable_id: read_u64_le(bytes, 0x30).ok_or(UmodParseError::HeaderTooShort)?,
        header_checksum: read_u64_le(bytes, 0x38).ok_or(UmodParseError::HeaderTooShort)?,
    })
}

fn read_array<const N: usize>(bytes: &[u8], offset: usize) -> Option<[u8; N]> {
    bytes.get(offset..offset.checked_add(N)?)?.try_into().ok()
}

fn read_u16_le(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(read_array(bytes, offset)?))
}

fn read_u32_le(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(read_array(bytes, offset)?))
}

fn read_u64_le(bytes: &[u8], offset: usize) -> Option<u64> {
    Some(u64::from_le_bytes(read_array(bytes, offset)?))
}

fn looks_like_path(bytes: &[u8]) -> bool {
    bytes.starts_with(b"local://")
        || bytes.starts_with(b"~")
        || bytes.windows(2).any(|window| window == b"..")
        || bytes.iter().any(|byte| matches!(*byte, b'/' | b'\\'))
}

fn opaque_id_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-')
}

/// Parser-stage errors (structural). The verifier in `graph` crate
/// produces a richer error covering semantic checks (spec §5.6).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UmodParseError {
    BadMagic,
    UnsupportedVersion { major: u16, minor: u16 },
    HeaderTooShort,
    BadHeaderLength { declared: u32 },
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

    fn minimal_header() -> [u8; UMOD_HEADER_LEN] {
        let mut bytes = [0; UMOD_HEADER_LEN];
        bytes[0..4].copy_from_slice(&UMOD_MAGIC);
        bytes[0x04..0x06].copy_from_slice(&UMOD_FORMAT_MAJOR.to_le_bytes());
        bytes[0x06..0x08].copy_from_slice(&UMOD_FORMAT_MINOR.to_le_bytes());
        bytes[0x08..0x0C].copy_from_slice(&UMOD_HEADER_LEN_U32.to_le_bytes());
        bytes[0x20..0x28].copy_from_slice(&(UMOD_HEADER_LEN as u64).to_le_bytes());
        bytes[0x28..0x30].copy_from_slice(&(UMOD_HEADER_LEN as u64).to_le_bytes());
        bytes[0x30..0x38].copy_from_slice(&0xAABB_CCDD_EEFF_0011_u64.to_le_bytes());
        bytes
    }

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

    #[test]
    fn parse_header_decodes_little_endian_fields() {
        let mut bytes = minimal_header();
        bytes[0x0C..0x10].copy_from_slice(&3_u32.to_le_bytes());
        bytes[0x10..0x14].copy_from_slice(&4_u32.to_le_bytes());
        bytes[0x14..0x18].copy_from_slice(&5_u32.to_le_bytes());
        bytes[0x1C..0x20].copy_from_slice(&6_u32.to_le_bytes());

        let header = parse_header(&bytes).expect("valid header");

        assert_eq!(header.magic, UMOD_MAGIC);
        assert_eq!(header.header_length, UMOD_HEADER_LEN_U32);
        assert_eq!(header.section_count, 3);
        assert_eq!(header.node_count, 4);
        assert_eq!(header.wire_count, 5);
        assert_eq!(header.capability_count, 6);
        assert_eq!(header.graph_stable_id, 0xAABB_CCDD_EEFF_0011);
    }

    #[test]
    fn parse_header_rejects_short_header() {
        assert_eq!(
            parse_header(b"UMOD").unwrap_err(),
            UmodParseError::HeaderTooShort
        );
    }

    #[test]
    fn parse_header_rejects_bad_magic() {
        let mut bytes = minimal_header();
        bytes[0] = b'X';

        assert_eq!(parse_header(&bytes).unwrap_err(), UmodParseError::BadMagic);
    }

    #[test]
    fn parse_header_rejects_unsupported_version() {
        let mut bytes = minimal_header();
        bytes[0x04..0x06].copy_from_slice(&2_u16.to_le_bytes());

        assert_eq!(
            parse_header(&bytes).unwrap_err(),
            UmodParseError::UnsupportedVersion { major: 2, minor: 0 }
        );
    }

    #[test]
    fn parse_header_rejects_bad_header_length() {
        let mut bytes = minimal_header();
        bytes[0x08..0x0C].copy_from_slice(&0x30_u32.to_le_bytes());

        assert_eq!(
            parse_header(&bytes).unwrap_err(),
            UmodParseError::BadHeaderLength { declared: 0x30 }
        );
    }

    #[test]
    fn parse_resource_ref_accepts_approved_opaque_refs() {
        let parsed = parse_resource_ref(b"model:tiny_transformer-01").expect("resource ref");

        assert_eq!(parsed.kind, ResourceType::Model);
        assert_eq!(parsed.opaque_id, b"tiny_transformer-01");
    }

    #[test]
    fn parse_resource_ref_rejects_path_shapes() {
        assert_eq!(
            parse_resource_ref(b"local://models/tiny.umdl").unwrap_err(),
            ResourceRefError::LooksLikeAPath
        );
        assert_eq!(
            parse_resource_ref(b"model:../tiny").unwrap_err(),
            ResourceRefError::LooksLikeAPath
        );
        assert_eq!(
            parse_resource_ref(br"model:dir\tiny").unwrap_err(),
            ResourceRefError::LooksLikeAPath
        );
    }

    #[test]
    fn parse_resource_ref_rejects_bad_opaque_ids() {
        assert_eq!(
            parse_resource_ref(b"model:").unwrap_err(),
            ResourceRefError::OpaqueIdEmpty
        );
        assert_eq!(
            parse_resource_ref(b"model:bad*id").unwrap_err(),
            ResourceRefError::OpaqueIdInvalidChar
        );
        assert_eq!(
            parse_resource_ref(
                b"model:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            )
            .unwrap_err(),
            ResourceRefError::OpaqueIdTooLong
        );
    }
}
