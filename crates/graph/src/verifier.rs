//! Graph verifier. Implements the 22 checks from spec §5.6 over
//! UMOD bytes and produces a `VerifiedGraph`. Single legal gate from
//! bytes to a verified graph.
//!
//! Each check is a discrete function returning a typed
//! `GraphLoadError` variant. The full pipeline runs them in
//! dependency order (structural before semantic; cycle detection
//! after node/wire resolution; etc.).

use crate::{GraphLoadError, VerifiedGraph};
use umod::{
    find_section, parse_header, parse_node_descriptor, parse_pin_type, parse_resource_ref,
    parse_structural, parse_wire_descriptor, section_payload, ParsedNodeDescriptor,
    ParsedUmodHeader, ResourceType, UmodParseError,
};

const NODE_TYPE_SOURCE: u64 = 1;
const NODE_TYPE_TRANSFORM: u64 = 2;
const NODE_TYPE_SINK: u64 = 3;
const NODE_TYPE_DELAY: u64 = 4;
const NONE_CONSTANT_REF: u32 = u32::MAX;
const MAX_WIRE_PAYLOAD_SIZE: u64 = 1 << 20;
const GRAPH_ARENA_BUDGET_BYTES: u64 = 1 << 20;
const NODE_RUNTIME_BUDGET_BYTES: u64 = 64;
const WIRE_RUNTIME_BUDGET_BYTES: u64 = 64;
const UI_COORD_LIMIT: i32 = 100_000;

/// Public entry point. Runs all 22 checks. Returns the verified
/// graph or the first failing check.
pub fn verify_umod(bytes: &[u8]) -> Result<VerifiedGraph<'_>, GraphLoadError> {
    check_magic(bytes)?;
    check_version(bytes)?;
    check_header_length(bytes)?;
    check_section_table(bytes)?;
    check_node_count(bytes)?;
    check_wire_count(bytes)?;
    check_node_indices(bytes)?;
    check_wire_endpoints(bytes)?;
    check_pin_indices(bytes)?;
    check_wire_types(bytes)?;
    check_node_types(bytes)?;
    check_capabilities(bytes)?;
    check_no_unbroken_cycles(bytes)?;
    check_payload_sizes(bytes)?;
    check_graph_arena_budget(bytes)?;
    check_model_refs(bytes)?;
    check_checksums(bytes)?;
    check_ui_layout(bytes)?;
    check_constant_blobs_exist(bytes)?;
    check_constant_blob_layouts(bytes)?;
    check_scheduling_section(bytes)?;
    check_opaque_resource_syntax(bytes)?;

    Ok(VerifiedGraph::new_internal(bytes))
}

// ───────────────────────────────────────────────────────────────────
// 22 checks. All stubs for now; each must be filled in with the
// concrete logic from spec §5.6 / §6.x.
// ───────────────────────────────────────────────────────────────────

fn check_magic(bytes: &[u8]) -> Result<(), GraphLoadError> {
    if bytes.len() < 4 || &bytes[0..4] != b"UMOD" {
        return Err(GraphLoadError::BadMagic);
    }
    Ok(())
}

fn check_version(bytes: &[u8]) -> Result<(), GraphLoadError> {
    parse_header(bytes).map_err(map_parse_error)?;
    Ok(())
}

fn check_header_length(bytes: &[u8]) -> Result<(), GraphLoadError> {
    parse_header(bytes).map_err(map_parse_error)?;
    Ok(())
}

fn check_section_table(bytes: &[u8]) -> Result<(), GraphLoadError> {
    parse_structural(bytes).map_err(map_parse_error)?;
    Ok(())
}

fn check_node_count(bytes: &[u8]) -> Result<(), GraphLoadError> {
    parse_structural(bytes).map_err(map_parse_error)?;
    Ok(())
}

fn check_wire_count(bytes: &[u8]) -> Result<(), GraphLoadError> {
    parse_structural(bytes).map_err(map_parse_error)?;
    Ok(())
}
fn check_node_indices(bytes: &[u8]) -> Result<(), GraphLoadError> {
    let header = parse_structural(bytes).map_err(map_parse_error)?;
    let mut index = 0;
    while index < header.node_count {
        let node = parse_node_descriptor(bytes, header, index).map_err(map_parse_error)?;
        if node.node_id == 0 || find_node_by_id(bytes, header, node.node_id, index + 1)?.is_some() {
            return Err(GraphLoadError::NodeIndexUnresolved {
                node_id: node.node_id,
            });
        }
        index = index.saturating_add(1);
    }
    Ok(())
}

fn check_wire_endpoints(bytes: &[u8]) -> Result<(), GraphLoadError> {
    let header = parse_structural(bytes).map_err(map_parse_error)?;
    let mut index = 0;
    while index < header.wire_count {
        let wire = parse_wire_descriptor(bytes, header, index).map_err(map_parse_error)?;
        if find_node_by_id(bytes, header, wire.src_node_id, 0)?.is_none()
            || find_node_by_id(bytes, header, wire.dst_node_id, 0)?.is_none()
        {
            return Err(GraphLoadError::WireEndpointUnresolved {
                wire_id: wire.wire_id,
            });
        }
        index = index.saturating_add(1);
    }
    Ok(())
}

fn check_pin_indices(bytes: &[u8]) -> Result<(), GraphLoadError> {
    let header = parse_structural(bytes).map_err(map_parse_error)?;
    let mut index = 0;
    while index < header.wire_count {
        let wire = parse_wire_descriptor(bytes, header, index).map_err(map_parse_error)?;
        let src_node = find_node_by_id(bytes, header, wire.src_node_id, 0)?.ok_or(
            GraphLoadError::WireEndpointUnresolved {
                wire_id: wire.wire_id,
            },
        )?;
        let dst_node = find_node_by_id(bytes, header, wire.dst_node_id, 0)?.ok_or(
            GraphLoadError::WireEndpointUnresolved {
                wire_id: wire.wire_id,
            },
        )?;
        if u32::from(wire.src_pin_index) >= u32::from(src_node.output_pin_count) {
            return Err(GraphLoadError::PinIndexOutOfRange {
                node_id: src_node.node_id,
                pin_index: wire.src_pin_index,
            });
        }
        if u32::from(wire.dst_pin_index) >= u32::from(dst_node.input_pin_count) {
            return Err(GraphLoadError::PinIndexOutOfRange {
                node_id: dst_node.node_id,
                pin_index: wire.dst_pin_index,
            });
        }
        if pin_range_exceeds(
            header.pin_type_count,
            src_node.output_pin_base,
            src_node.output_pin_count,
        ) || pin_range_exceeds(
            header.pin_type_count,
            dst_node.input_pin_base,
            dst_node.input_pin_count,
        ) {
            return Err(GraphLoadError::PinIndexOutOfRange {
                node_id: src_node.node_id,
                pin_index: wire.src_pin_index,
            });
        }
        index = index.saturating_add(1);
    }
    Ok(())
}

fn check_wire_types(bytes: &[u8]) -> Result<(), GraphLoadError> {
    let header = parse_structural(bytes).map_err(map_parse_error)?;
    let mut index = 0;
    while index < header.wire_count {
        let wire = parse_wire_descriptor(bytes, header, index).map_err(map_parse_error)?;
        let src_node = find_node_by_id(bytes, header, wire.src_node_id, 0)?.ok_or(
            GraphLoadError::WireEndpointUnresolved {
                wire_id: wire.wire_id,
            },
        )?;
        let dst_node = find_node_by_id(bytes, header, wire.dst_node_id, 0)?.ok_or(
            GraphLoadError::WireEndpointUnresolved {
                wire_id: wire.wire_id,
            },
        )?;
        let src_type_index = src_node.output_pin_base + u32::from(wire.src_pin_index);
        let dst_type_index = dst_node.input_pin_base + u32::from(wire.dst_pin_index);
        let src_type = parse_pin_type(bytes, header, src_type_index).map_err(map_parse_error)?;
        let dst_type = parse_pin_type(bytes, header, dst_type_index).map_err(map_parse_error)?;
        if wire.type_id != src_type || wire.type_id != dst_type {
            return Err(GraphLoadError::WireTypeMismatch {
                wire_id: wire.wire_id,
            });
        }
        index = index.saturating_add(1);
    }
    Ok(())
}

fn check_node_types(bytes: &[u8]) -> Result<(), GraphLoadError> {
    let header = parse_structural(bytes).map_err(map_parse_error)?;
    let mut index = 0;
    while index < header.node_count {
        let node = parse_node_descriptor(bytes, header, index).map_err(map_parse_error)?;
        if !known_node_type(node.node_type_id) {
            return Err(GraphLoadError::UnknownNodeType {
                node_id: node.node_id,
                type_id: u32::try_from(node.node_type_id).unwrap_or(u32::MAX),
            });
        }
        index = index.saturating_add(1);
    }
    Ok(())
}

fn check_capabilities(bytes: &[u8]) -> Result<(), GraphLoadError> {
    let header = parse_structural(bytes).map_err(map_parse_error)?;
    let mut index = 0;
    while index < header.node_count {
        let node = parse_node_descriptor(bytes, header, index).map_err(map_parse_error)?;
        let end = node
            .capability_base
            .checked_add(u32::from(node.capability_count))
            .ok_or(GraphLoadError::UndeclaredCapability {
                capability_id: node.capability_base,
            })?;
        if end > header.capability_count {
            return Err(GraphLoadError::UndeclaredCapability {
                capability_id: node.capability_base,
            });
        }
        index = index.saturating_add(1);
    }
    Ok(())
}

/// Check 13. Reject any cycle that does not pass through a node
/// tagged with `NodeFlags::CYCLE_BREAK` (DelayOneTick, RegisterNode,
/// KVCacheNode, StateCellNode, FrameBufferNode, …; spec §5.10).
fn check_no_unbroken_cycles(bytes: &[u8]) -> Result<(), GraphLoadError> {
    let header = parse_structural(bytes).map_err(map_parse_error)?;
    let mut first_index = 0;
    while first_index < header.wire_count {
        let first = parse_wire_descriptor(bytes, header, first_index).map_err(map_parse_error)?;
        if first.src_node_id == first.dst_node_id
            && !node_breaks_cycle(bytes, header, first.src_node_id)?
        {
            return Err(GraphLoadError::UnbrokenCycle {
                sample_node_id: first.src_node_id,
            });
        }

        let mut second_index = first_index.saturating_add(1);
        while second_index < header.wire_count {
            let second =
                parse_wire_descriptor(bytes, header, second_index).map_err(map_parse_error)?;
            if first.src_node_id == second.dst_node_id
                && first.dst_node_id == second.src_node_id
                && !node_breaks_cycle(bytes, header, first.src_node_id)?
                && !node_breaks_cycle(bytes, header, first.dst_node_id)?
            {
                return Err(GraphLoadError::UnbrokenCycle {
                    sample_node_id: first.src_node_id,
                });
            }
            second_index = second_index.saturating_add(1);
        }
        first_index = first_index.saturating_add(1);
    }
    Ok(())
}

fn check_payload_sizes(bytes: &[u8]) -> Result<(), GraphLoadError> {
    let header = parse_structural(bytes).map_err(map_parse_error)?;
    let mut index = 0;
    while index < header.wire_count {
        let wire = parse_wire_descriptor(bytes, header, index).map_err(map_parse_error)?;
        if wire.payload_size == 0
            || wire.payload_size > MAX_WIRE_PAYLOAD_SIZE
            || wire.alignment == 0
            || !wire.alignment.is_power_of_two()
        {
            return Err(GraphLoadError::PayloadSizeUnbounded {
                wire_id: wire.wire_id,
            });
        }
        index = index.saturating_add(1);
    }
    Ok(())
}

fn check_graph_arena_budget(bytes: &[u8]) -> Result<(), GraphLoadError> {
    let header = parse_structural(bytes).map_err(map_parse_error)?;
    let mut required = u64::from(header.node_count)
        .checked_mul(NODE_RUNTIME_BUDGET_BYTES)
        .and_then(|nodes| {
            u64::from(header.wire_count)
                .checked_mul(WIRE_RUNTIME_BUDGET_BYTES)
                .and_then(|wires| nodes.checked_add(wires))
        })
        .ok_or(GraphLoadError::GraphArenaBudgetExceeded { required: u64::MAX })?;

    let mut index = 0;
    while index < header.wire_count {
        let wire = parse_wire_descriptor(bytes, header, index).map_err(map_parse_error)?;
        required = required
            .checked_add(wire.payload_size)
            .ok_or(GraphLoadError::GraphArenaBudgetExceeded { required: u64::MAX })?;
        index = index.saturating_add(1);
    }

    if required > GRAPH_ARENA_BUDGET_BYTES {
        return Err(GraphLoadError::GraphArenaBudgetExceeded { required });
    }
    Ok(())
}

fn check_model_refs(bytes: &[u8]) -> Result<(), GraphLoadError> {
    let header = parse_structural(bytes).map_err(map_parse_error)?;
    for_each_external_ref(bytes, header, |ref_index, resource| {
        if resource.kind == ResourceType::Model {
            return Err(GraphLoadError::ModelRefUnresolved { ref_index });
        }
        Ok(())
    })
}

fn check_checksums(bytes: &[u8]) -> Result<(), GraphLoadError> {
    let header = parse_structural(bytes).map_err(map_parse_error)?;
    let mut index = 0;
    while index < header.section_count {
        let section =
            umod::parse_section_descriptor(bytes, header, index).map_err(map_parse_error)?;
        if section.checksum != 0 {
            let payload = section_payload(bytes, section).map_err(map_parse_error)?;
            if byte_sum(payload) != section.checksum {
                return Err(GraphLoadError::ChecksumMismatch {
                    section_index: index,
                });
            }
        }
        index = index.saturating_add(1);
    }
    Ok(())
}

fn check_ui_layout(bytes: &[u8]) -> Result<(), GraphLoadError> {
    let header = parse_structural(bytes).map_err(map_parse_error)?;
    let mut index = 0;
    while index < header.node_count {
        let node = parse_node_descriptor(bytes, header, index).map_err(map_parse_error)?;
        if node.ui_x.unsigned_abs() > UI_COORD_LIMIT.unsigned_abs()
            || node.ui_y.unsigned_abs() > UI_COORD_LIMIT.unsigned_abs()
            || u64::from(node.label_offset) > header.file_length_bytes
        {
            return Err(GraphLoadError::UiLayoutInvalid);
        }
        index = index.saturating_add(1);
    }
    Ok(())
}

fn check_constant_blobs_exist(bytes: &[u8]) -> Result<(), GraphLoadError> {
    let header = parse_structural(bytes).map_err(map_parse_error)?;
    let constant_section =
        find_section(bytes, header, umod::SECTION_KIND_CONSTANT_BLOBS).map_err(map_parse_error)?;
    let mut index = 0;
    while index < header.node_count {
        let node = parse_node_descriptor(bytes, header, index).map_err(map_parse_error)?;
        if node.constant_ref != NONE_CONSTANT_REF && constant_section.is_none() {
            return Err(GraphLoadError::MissingConstantBlob {
                node_id: node.node_id,
            });
        }
        index = index.saturating_add(1);
    }
    Ok(())
}

fn check_constant_blob_layouts(bytes: &[u8]) -> Result<(), GraphLoadError> {
    let header = parse_structural(bytes).map_err(map_parse_error)?;
    let constant_section =
        find_section(bytes, header, umod::SECTION_KIND_CONSTANT_BLOBS).map_err(map_parse_error)?;
    let mut index = 0;
    while index < header.node_count {
        let node = parse_node_descriptor(bytes, header, index).map_err(map_parse_error)?;
        if node.constant_ref != NONE_CONSTANT_REF {
            let Some(section) = constant_section else {
                return Err(GraphLoadError::MissingConstantBlob {
                    node_id: node.node_id,
                });
            };
            if u64::from(node.constant_ref) >= section.length {
                return Err(GraphLoadError::ConstantBlobLayoutBad {
                    node_id: node.node_id,
                });
            }
        }
        index = index.saturating_add(1);
    }
    Ok(())
}

fn check_scheduling_section(bytes: &[u8]) -> Result<(), GraphLoadError> {
    let header = parse_structural(bytes).map_err(map_parse_error)?;
    if deterministic_schedule_required(bytes, header)?
        && find_section(bytes, header, umod::SECTION_KIND_SCHEDULING)
            .map_err(map_parse_error)?
            .is_none()
    {
        return Err(GraphLoadError::SchedulingSectionMissing);
    }
    Ok(())
}

/// Check 22. Every external reference must use the approved
/// opaque-resource grammar (spec §6.8). Delegates to the umod crate's
/// `parse_resource_ref`, which rejects POSIX paths.
fn check_opaque_resource_syntax(bytes: &[u8]) -> Result<(), GraphLoadError> {
    let header = parse_structural(bytes).map_err(map_parse_error)?;
    for_each_external_ref(bytes, header, |_ref_index, _resource| Ok(()))
}

fn find_node_by_id(
    bytes: &[u8],
    header: ParsedUmodHeader,
    node_id: u32,
    start_index: u32,
) -> Result<Option<ParsedNodeDescriptor>, GraphLoadError> {
    let mut index = start_index;
    while index < header.node_count {
        let node = parse_node_descriptor(bytes, header, index).map_err(map_parse_error)?;
        if node.node_id == node_id {
            return Ok(Some(node));
        }
        index = index.saturating_add(1);
    }
    Ok(None)
}

fn pin_range_exceeds(pin_type_count: u32, base: u32, count: u16) -> bool {
    base.checked_add(u32::from(count))
        .map_or(true, |end| end > pin_type_count)
}

const fn known_node_type(node_type_id: u64) -> bool {
    matches!(
        node_type_id,
        NODE_TYPE_SOURCE | NODE_TYPE_TRANSFORM | NODE_TYPE_SINK | NODE_TYPE_DELAY
    )
}

fn node_breaks_cycle(
    bytes: &[u8],
    header: ParsedUmodHeader,
    node_id: u32,
) -> Result<bool, GraphLoadError> {
    let node = find_node_by_id(bytes, header, node_id, 0)?
        .ok_or(GraphLoadError::NodeIndexUnresolved { node_id })?;
    Ok(node.node_type_id == NODE_TYPE_DELAY)
}

fn for_each_external_ref(
    bytes: &[u8],
    header: ParsedUmodHeader,
    mut visit: impl FnMut(u32, umod::ResourceRef<'_>) -> Result<(), GraphLoadError>,
) -> Result<(), GraphLoadError> {
    let Some(section) =
        find_section(bytes, header, umod::SECTION_KIND_EXTERNAL_REFS).map_err(map_parse_error)?
    else {
        return Ok(());
    };
    let payload = section_payload(bytes, section).map_err(map_parse_error)?;
    let mut start = 0;
    let mut ref_index = 0;
    let mut cursor = 0;
    while cursor <= payload.len() {
        if cursor == payload.len() || payload[cursor] == 0 {
            if cursor > start {
                let resource = parse_resource_ref(&payload[start..cursor])
                    .map_err(|_| GraphLoadError::OpaqueResourceSyntaxBad { ref_index })?;
                visit(ref_index, resource)?;
                ref_index = ref_index.saturating_add(1);
            }
            start = cursor.saturating_add(1);
        }
        cursor = cursor.saturating_add(1);
    }
    Ok(())
}

fn deterministic_schedule_required(
    bytes: &[u8],
    header: ParsedUmodHeader,
) -> Result<bool, GraphLoadError> {
    let mut index = 0;
    while index < header.section_count {
        let section =
            umod::parse_section_descriptor(bytes, header, index).map_err(map_parse_error)?;
        if section.flags & umod::SECTION_FLAG_REQUIRES_DETERMINISTIC_SCHEDULE != 0 {
            return Ok(true);
        }
        index = index.saturating_add(1);
    }

    index = 0;
    while index < header.wire_count {
        let wire = parse_wire_descriptor(bytes, header, index).map_err(map_parse_error)?;
        if wire.flags & umod::SECTION_FLAG_REQUIRES_DETERMINISTIC_SCHEDULE != 0 {
            return Ok(true);
        }
        index = index.saturating_add(1);
    }
    Ok(false)
}

fn byte_sum(bytes: &[u8]) -> u64 {
    bytes
        .iter()
        .fold(0_u64, |sum, byte| sum.wrapping_add(u64::from(*byte)))
}

const fn map_parse_error(error: UmodParseError) -> GraphLoadError {
    match error {
        UmodParseError::BadMagic => GraphLoadError::BadMagic,
        UmodParseError::UnsupportedVersion { .. } => GraphLoadError::UnsupportedVersion,
        UmodParseError::HeaderTooShort | UmodParseError::FileLengthOutOfBounds { .. } => {
            GraphLoadError::ParseTruncated
        }
        UmodParseError::BadHeaderLength { .. } => GraphLoadError::BadHeaderLength,
        UmodParseError::SectionTableOutOfBounds
        | UmodParseError::SectionOutOfBounds { .. }
        | UmodParseError::SectionOverlap { .. } => GraphLoadError::BadSectionTable,
        UmodParseError::NodeCountOverflow => GraphLoadError::NodeCountExceedsLimit,
        UmodParseError::WireCountOverflow => GraphLoadError::WireCountExceedsLimit,
        UmodParseError::HeaderChecksumMismatch => GraphLoadError::ParseHeaderChecksumMismatch,
        UmodParseError::SectionChecksumMismatch { index } => GraphLoadError::ChecksumMismatch {
            section_index: index,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use umod::{
        SECTION_KIND_NODE_DESCRIPTORS, SECTION_KIND_PIN_TYPES, SECTION_KIND_WIRE_DESCRIPTORS,
        UMOD_FORMAT_MAJOR, UMOD_FORMAT_MINOR, UMOD_HEADER_LEN, UMOD_HEADER_LEN_U32,
        UMOD_NODE_DESCRIPTOR_LEN, UMOD_NODE_DESCRIPTOR_LEN_U64, UMOD_PIN_TYPE_LEN_U64,
        UMOD_WIRE_DESCRIPTOR_LEN, UMOD_WIRE_DESCRIPTOR_LEN_U64,
    };

    const NODE_SECTION_OFFSET: usize = 0xC0;
    const WIRE_SECTION_OFFSET: usize = 0x130;
    const PIN_SECTION_OFFSET: usize = 0x160;
    const BASE_UMOD_LEN: usize = 0x170;

    fn minimal_header() -> [u8; UMOD_HEADER_LEN] {
        let mut bytes = [0; UMOD_HEADER_LEN];
        bytes[0..4].copy_from_slice(b"UMOD");
        bytes[0x04..0x06].copy_from_slice(&UMOD_FORMAT_MAJOR.to_le_bytes());
        bytes[0x06..0x08].copy_from_slice(&UMOD_FORMAT_MINOR.to_le_bytes());
        bytes[0x08..0x0C].copy_from_slice(&UMOD_HEADER_LEN_U32.to_le_bytes());
        bytes[0x20..0x28].copy_from_slice(&(UMOD_HEADER_LEN as u64).to_le_bytes());
        bytes[0x28..0x30].copy_from_slice(&(UMOD_HEADER_LEN as u64).to_le_bytes());
        bytes
    }

    fn write_section(bytes: &mut [u8], offset: usize, kind: u32, section_offset: u64, length: u64) {
        write_section_with_flags(bytes, offset, kind, 0, section_offset, length, 0);
    }

    fn write_section_with_flags(
        bytes: &mut [u8],
        offset: usize,
        kind: u32,
        flags: u32,
        section_offset: u64,
        length: u64,
        checksum: u64,
    ) {
        bytes[offset..offset + 0x04].copy_from_slice(&kind.to_le_bytes());
        bytes[offset + 0x04..offset + 0x08].copy_from_slice(&flags.to_le_bytes());
        bytes[offset + 0x08..offset + 0x10].copy_from_slice(&section_offset.to_le_bytes());
        bytes[offset + 0x10..offset + 0x18].copy_from_slice(&length.to_le_bytes());
        bytes[offset + 0x18..offset + 0x20].copy_from_slice(&checksum.to_le_bytes());
    }

    fn semantic_umod() -> [u8; BASE_UMOD_LEN] {
        let mut bytes = [0; BASE_UMOD_LEN];
        bytes[0..UMOD_HEADER_LEN].copy_from_slice(&minimal_header());
        bytes[0x0C..0x10].copy_from_slice(&3_u32.to_le_bytes());
        bytes[0x10..0x14].copy_from_slice(&2_u32.to_le_bytes());
        bytes[0x14..0x18].copy_from_slice(&1_u32.to_le_bytes());
        bytes[0x18..0x1C].copy_from_slice(&2_u32.to_le_bytes());
        bytes[0x28..0x30].copy_from_slice(&(BASE_UMOD_LEN as u64).to_le_bytes());

        write_section(
            &mut bytes,
            0x40,
            SECTION_KIND_NODE_DESCRIPTORS,
            NODE_SECTION_OFFSET as u64,
            2 * UMOD_NODE_DESCRIPTOR_LEN_U64,
        );
        write_section(
            &mut bytes,
            0x60,
            SECTION_KIND_WIRE_DESCRIPTORS,
            WIRE_SECTION_OFFSET as u64,
            UMOD_WIRE_DESCRIPTOR_LEN_U64,
        );
        write_section(
            &mut bytes,
            0x80,
            SECTION_KIND_PIN_TYPES,
            PIN_SECTION_OFFSET as u64,
            2 * UMOD_PIN_TYPE_LEN_U64,
        );

        write_node(
            &mut bytes,
            NODE_SECTION_OFFSET,
            1,
            NODE_TYPE_SOURCE,
            0,
            0,
            0,
            1,
            0,
            0,
        );
        write_node(
            &mut bytes,
            NODE_SECTION_OFFSET + UMOD_NODE_DESCRIPTOR_LEN,
            2,
            NODE_TYPE_SINK,
            1,
            1,
            0,
            0,
            0,
            0,
        );
        write_wire(&mut bytes, WIRE_SECTION_OFFSET, (1, 1, 2), 0, 0, 7);
        write_wire_with_payload(&mut bytes, WIRE_SECTION_OFFSET, 8, 8, 0);
        write_pin_type(&mut bytes, PIN_SECTION_OFFSET, 7);
        write_pin_type(&mut bytes, PIN_SECTION_OFFSET + 8, 7);
        bytes
    }

    fn semantic_umod_with_extra_section(kind: u32, payload: &[u8]) -> [u8; 0x1B0] {
        let mut bytes = [0; 0x1B0];
        bytes[0..BASE_UMOD_LEN].copy_from_slice(&semantic_umod());
        bytes[0x0C..0x10].copy_from_slice(&4_u32.to_le_bytes());
        bytes[0x28..0x30].copy_from_slice(&0x1B0_u64.to_le_bytes());
        write_section(
            &mut bytes,
            0x80,
            SECTION_KIND_PIN_TYPES,
            PIN_SECTION_OFFSET as u64,
            0x10,
        );
        write_section(
            &mut bytes,
            0xA0,
            kind,
            BASE_UMOD_LEN as u64,
            payload.len() as u64,
        );
        bytes[BASE_UMOD_LEN..BASE_UMOD_LEN + payload.len()].copy_from_slice(payload);
        bytes
    }

    #[allow(clippy::too_many_arguments)]
    fn write_node(
        bytes: &mut [u8],
        offset: usize,
        node_id: u32,
        node_type_id: u64,
        input_pin_base: u32,
        input_pin_count: u16,
        output_pin_base: u32,
        output_pin_count: u16,
        capability_base: u32,
        capability_count: u16,
    ) {
        bytes[offset..offset + 0x04].copy_from_slice(&node_id.to_le_bytes());
        bytes[offset + 0x08..offset + 0x10].copy_from_slice(&node_type_id.to_le_bytes());
        bytes[offset + 0x10..offset + 0x14].copy_from_slice(&input_pin_base.to_le_bytes());
        bytes[offset + 0x14..offset + 0x16].copy_from_slice(&input_pin_count.to_le_bytes());
        bytes[offset + 0x18..offset + 0x1C].copy_from_slice(&output_pin_base.to_le_bytes());
        bytes[offset + 0x1C..offset + 0x1E].copy_from_slice(&output_pin_count.to_le_bytes());
        bytes[offset + 0x20..offset + 0x24].copy_from_slice(&capability_base.to_le_bytes());
        bytes[offset + 0x24..offset + 0x26].copy_from_slice(&capability_count.to_le_bytes());
        bytes[offset + 0x34..offset + 0x38].copy_from_slice(&NONE_CONSTANT_REF.to_le_bytes());
    }

    fn write_wire(
        bytes: &mut [u8],
        offset: usize,
        endpoints: (u32, u32, u32),
        src_pin_index: u16,
        dst_pin_index: u16,
        type_id: u64,
    ) {
        let (wire_id, src_node_id, dst_node_id) = endpoints;
        bytes[offset..offset + 0x04].copy_from_slice(&wire_id.to_le_bytes());
        bytes[offset + 0x04..offset + 0x08].copy_from_slice(&src_node_id.to_le_bytes());
        bytes[offset + 0x08..offset + 0x0A].copy_from_slice(&src_pin_index.to_le_bytes());
        bytes[offset + 0x0C..offset + 0x10].copy_from_slice(&dst_node_id.to_le_bytes());
        bytes[offset + 0x10..offset + 0x12].copy_from_slice(&dst_pin_index.to_le_bytes());
        bytes[offset + 0x18..offset + 0x20].copy_from_slice(&type_id.to_le_bytes());
    }

    fn write_wire_with_payload(
        bytes: &mut [u8],
        offset: usize,
        payload_size: u64,
        alignment: u32,
        flags: u32,
    ) {
        bytes[offset + 0x20..offset + 0x28].copy_from_slice(&payload_size.to_le_bytes());
        bytes[offset + 0x28..offset + 0x2C].copy_from_slice(&alignment.to_le_bytes());
        bytes[offset + 0x2C..offset + 0x30].copy_from_slice(&flags.to_le_bytes());
    }

    fn write_pin_type(bytes: &mut [u8], offset: usize, type_id: u64) {
        bytes[offset..offset + 0x08].copy_from_slice(&type_id.to_le_bytes());
    }

    #[test]
    fn empty_bytes_fail_magic() {
        let r = verify_umod(&[]);
        assert_eq!(r.unwrap_err(), GraphLoadError::BadMagic);
    }

    #[test]
    fn wrong_magic_fails() {
        let r = verify_umod(b"NOPEXXXX");
        assert_eq!(r.unwrap_err(), GraphLoadError::BadMagic);
    }

    #[test]
    fn persistent_source_transform_sink_payload_verifies() {
        let r = verify_umod(crate::SOURCE_TRANSFORM_SINK_UMOD);
        assert!(r.is_ok());
    }

    #[test]
    fn golden_registry_fixture_is_exercised() {
        let registry = include_str!("../../../tests/golden_graphs/registry.toml");

        assert!(registry.contains("source-transform-sink.bin"));
        assert!(verify_umod(include_bytes!(
            "../../../tests/golden_graphs/source-transform-sink.bin"
        ))
        .is_ok());
    }

    #[test]
    fn malformed_corpus_fixtures_reject_with_declared_errors() {
        assert_eq!(
            verify_umod(include_bytes!(
                "../../../tests/fuzz_corpus/umod/bad_magic/bad-magic.bin"
            ))
            .unwrap_err(),
            GraphLoadError::BadMagic
        );
        assert_eq!(
            verify_umod(include_bytes!(
                "../../../tests/fuzz_corpus/umod/unsupported_versions/unsupported-version.bin"
            ))
            .unwrap_err(),
            GraphLoadError::UnsupportedVersion
        );
        assert_eq!(
            verify_umod(include_bytes!(
                "../../../tests/fuzz_corpus/umod/undersized_headers/truncated-header.bin"
            ))
            .unwrap_err(),
            GraphLoadError::ParseTruncated
        );
        assert_eq!(
            verify_umod(include_bytes!(
                "../../../tests/fuzz_corpus/umod/offsets_outside_bounds/section-out-of-bounds.bin"
            ))
            .unwrap_err(),
            GraphLoadError::BadSectionTable
        );
        assert_eq!(
            verify_umod(include_bytes!(
                "../../../tests/fuzz_corpus/umod/overlapping_sections/section-overlap.bin"
            ))
            .unwrap_err(),
            GraphLoadError::BadSectionTable
        );
        assert_eq!(
            verify_umod(include_bytes!(
                "../../../tests/fuzz_corpus/umod/huge_counts_tiny_files/node-count-over-limit.bin"
            ))
            .unwrap_err(),
            GraphLoadError::NodeCountExceedsLimit
        );
        assert_eq!(
            verify_umod(include_bytes!(
                "../../../tests/fuzz_corpus/umod/unknown_resource_syntax/path-shaped-resource-ref.bin"
            ))
            .unwrap_err(),
            GraphLoadError::OpaqueResourceSyntaxBad { ref_index: 0 }
        );
        assert_eq!(
            verify_umod(include_bytes!(
                "../../../tests/fuzz_corpus/umod/unbroken_cycles/two-node-loop.bin"
            ))
            .unwrap_err(),
            GraphLoadError::UnbrokenCycle { sample_node_id: 1 }
        );
    }

    #[test]
    fn short_umod_header_fails_structurally() {
        let r = verify_umod(b"UMOD");
        assert_eq!(r.unwrap_err(), GraphLoadError::ParseTruncated);
    }

    #[test]
    fn unsupported_umod_version_fails_structurally() {
        let mut bytes = minimal_header();
        bytes[0x04..0x06].copy_from_slice(&2_u16.to_le_bytes());

        let r = verify_umod(&bytes);

        assert_eq!(r.unwrap_err(), GraphLoadError::UnsupportedVersion);
    }

    #[test]
    fn bad_umod_header_length_fails_structurally() {
        let mut bytes = minimal_header();
        bytes[0x08..0x0C].copy_from_slice(&0x30_u32.to_le_bytes());

        let r = verify_umod(&bytes);

        assert_eq!(r.unwrap_err(), GraphLoadError::BadHeaderLength);
    }

    #[test]
    fn section_table_out_of_bounds_fails_structurally() {
        let mut bytes = minimal_header();
        bytes[0x0C..0x10].copy_from_slice(&1_u32.to_le_bytes());

        let r = verify_umod(&bytes);

        assert_eq!(r.unwrap_err(), GraphLoadError::BadSectionTable);
    }

    #[test]
    fn section_overlap_fails_structurally() {
        let mut bytes = [0; 0x90];
        bytes[0..UMOD_HEADER_LEN].copy_from_slice(&minimal_header());
        bytes[0x0C..0x10].copy_from_slice(&2_u32.to_le_bytes());
        bytes[0x28..0x30].copy_from_slice(&0x90_u64.to_le_bytes());
        write_section(&mut bytes, 0x40, 7, 0x80, 0x10);
        write_section(&mut bytes, 0x60, 8, 0x88, 0x08);

        let r = verify_umod(&bytes);

        assert_eq!(r.unwrap_err(), GraphLoadError::BadSectionTable);
    }

    #[test]
    fn node_and_wire_count_limits_fail_structurally() {
        let mut bytes = minimal_header();
        bytes[0x10..0x14].copy_from_slice(&(umod::UMOD_MAX_NODE_COUNT + 1).to_le_bytes());
        assert_eq!(
            verify_umod(&bytes).unwrap_err(),
            GraphLoadError::NodeCountExceedsLimit
        );

        let mut bytes = minimal_header();
        bytes[0x14..0x18].copy_from_slice(&(umod::UMOD_MAX_WIRE_COUNT + 1).to_le_bytes());
        assert_eq!(
            verify_umod(&bytes).unwrap_err(),
            GraphLoadError::WireCountExceedsLimit
        );
    }

    #[test]
    fn semantic_umod_verifies_topology() {
        let bytes = semantic_umod();

        assert!(verify_umod(&bytes).is_ok());
    }

    #[test]
    fn unresolved_wire_endpoint_fails_semantically() {
        let mut bytes = semantic_umod();
        write_wire(&mut bytes, WIRE_SECTION_OFFSET, (1, 1, 99), 0, 0, 7);

        assert_eq!(
            verify_umod(&bytes).unwrap_err(),
            GraphLoadError::WireEndpointUnresolved { wire_id: 1 }
        );
    }

    #[test]
    fn out_of_range_pin_index_fails_semantically() {
        let mut bytes = semantic_umod();
        write_wire(&mut bytes, WIRE_SECTION_OFFSET, (1, 1, 2), 1, 0, 7);

        assert_eq!(
            verify_umod(&bytes).unwrap_err(),
            GraphLoadError::PinIndexOutOfRange {
                node_id: 1,
                pin_index: 1
            }
        );
    }

    #[test]
    fn wire_type_mismatch_fails_semantically() {
        let mut bytes = semantic_umod();
        write_pin_type(&mut bytes, PIN_SECTION_OFFSET + 8, 8);

        assert_eq!(
            verify_umod(&bytes).unwrap_err(),
            GraphLoadError::WireTypeMismatch { wire_id: 1 }
        );
    }

    #[test]
    fn unknown_node_type_fails_semantically() {
        let mut bytes = semantic_umod();
        write_node(&mut bytes, NODE_SECTION_OFFSET, 1, 99, 0, 0, 0, 1, 0, 0);

        assert_eq!(
            verify_umod(&bytes).unwrap_err(),
            GraphLoadError::UnknownNodeType {
                node_id: 1,
                type_id: 99
            }
        );
    }

    #[test]
    fn undeclared_capability_fails_semantically() {
        let mut bytes = semantic_umod();
        write_node(
            &mut bytes,
            NODE_SECTION_OFFSET,
            1,
            NODE_TYPE_SOURCE,
            0,
            0,
            0,
            1,
            1,
            1,
        );

        assert_eq!(
            verify_umod(&bytes).unwrap_err(),
            GraphLoadError::UndeclaredCapability { capability_id: 1 }
        );
    }

    #[test]
    fn unbroken_two_node_cycle_fails_semantically() {
        let mut bytes = [0; 0x1A0];
        bytes[0..BASE_UMOD_LEN].copy_from_slice(&semantic_umod());
        bytes[0x14..0x18].copy_from_slice(&2_u32.to_le_bytes());
        bytes[0x28..0x30].copy_from_slice(&0x1A0_u64.to_le_bytes());
        write_section(
            &mut bytes,
            0x60,
            SECTION_KIND_WIRE_DESCRIPTORS,
            WIRE_SECTION_OFFSET as u64,
            2 * UMOD_WIRE_DESCRIPTOR_LEN_U64,
        );
        write_section(&mut bytes, 0x80, SECTION_KIND_PIN_TYPES, 0x190, 0x10);
        write_wire(&mut bytes, WIRE_SECTION_OFFSET, (1, 1, 2), 0, 0, 7);
        write_wire(
            &mut bytes,
            WIRE_SECTION_OFFSET + UMOD_WIRE_DESCRIPTOR_LEN,
            (2, 2, 1),
            0,
            0,
            7,
        );
        write_pin_type(&mut bytes, 0x190, 7);
        write_pin_type(&mut bytes, 0x198, 7);
        write_node(
            &mut bytes,
            NODE_SECTION_OFFSET,
            1,
            NODE_TYPE_SOURCE,
            1,
            1,
            0,
            1,
            0,
            0,
        );
        write_node(
            &mut bytes,
            NODE_SECTION_OFFSET + UMOD_NODE_DESCRIPTOR_LEN,
            2,
            NODE_TYPE_TRANSFORM,
            1,
            1,
            0,
            1,
            0,
            0,
        );

        assert_eq!(
            verify_umod(&bytes).unwrap_err(),
            GraphLoadError::UnbrokenCycle { sample_node_id: 1 }
        );
    }

    #[test]
    fn unbounded_payload_fails_semantically() {
        let mut bytes = semantic_umod();
        write_wire_with_payload(&mut bytes, WIRE_SECTION_OFFSET, 0, 8, 0);

        assert_eq!(
            verify_umod(&bytes).unwrap_err(),
            GraphLoadError::PayloadSizeUnbounded { wire_id: 1 }
        );
    }

    #[test]
    fn graph_arena_budget_exceeded_fails_semantically() {
        let mut bytes = semantic_umod();
        write_wire_with_payload(&mut bytes, WIRE_SECTION_OFFSET, MAX_WIRE_PAYLOAD_SIZE, 8, 0);

        assert_eq!(
            verify_umod(&bytes).unwrap_err(),
            GraphLoadError::GraphArenaBudgetExceeded {
                required: MAX_WIRE_PAYLOAD_SIZE
                    + 2 * NODE_RUNTIME_BUDGET_BYTES
                    + WIRE_RUNTIME_BUDGET_BYTES
            }
        );
    }

    #[test]
    fn model_ref_fails_gracefully() {
        let bytes =
            semantic_umod_with_extra_section(umod::SECTION_KIND_EXTERNAL_REFS, b"model:tiny\0");

        assert_eq!(
            verify_umod(&bytes).unwrap_err(),
            GraphLoadError::ModelRefUnresolved { ref_index: 0 }
        );
    }

    #[test]
    fn checksum_mismatch_fails_semantically() {
        let mut bytes = semantic_umod();
        write_section_with_flags(
            &mut bytes,
            0x80,
            SECTION_KIND_PIN_TYPES,
            0,
            PIN_SECTION_OFFSET as u64,
            0x10,
            1,
        );

        assert_eq!(
            verify_umod(&bytes).unwrap_err(),
            GraphLoadError::ChecksumMismatch { section_index: 2 }
        );
    }

    #[test]
    fn invalid_ui_layout_fails_semantically() {
        let mut bytes = semantic_umod();
        bytes[NODE_SECTION_OFFSET + 0x28..NODE_SECTION_OFFSET + 0x2C]
            .copy_from_slice(&(UI_COORD_LIMIT + 1).to_le_bytes());

        assert_eq!(
            verify_umod(&bytes).unwrap_err(),
            GraphLoadError::UiLayoutInvalid
        );
    }

    #[test]
    fn missing_constant_blob_fails_semantically() {
        let mut bytes = semantic_umod();
        bytes[NODE_SECTION_OFFSET + 0x34..NODE_SECTION_OFFSET + 0x38]
            .copy_from_slice(&0_u32.to_le_bytes());

        assert_eq!(
            verify_umod(&bytes).unwrap_err(),
            GraphLoadError::MissingConstantBlob { node_id: 1 }
        );
    }

    #[test]
    fn bad_constant_blob_layout_fails_semantically() {
        let mut bytes = semantic_umod_with_extra_section(umod::SECTION_KIND_CONSTANT_BLOBS, b"abc");
        bytes[NODE_SECTION_OFFSET + 0x34..NODE_SECTION_OFFSET + 0x38]
            .copy_from_slice(&3_u32.to_le_bytes());

        assert_eq!(
            verify_umod(&bytes).unwrap_err(),
            GraphLoadError::ConstantBlobLayoutBad { node_id: 1 }
        );
    }

    #[test]
    fn missing_scheduling_section_fails_semantically() {
        let mut bytes = semantic_umod();
        write_wire_with_payload(
            &mut bytes,
            WIRE_SECTION_OFFSET,
            8,
            8,
            umod::SECTION_FLAG_REQUIRES_DETERMINISTIC_SCHEDULE,
        );

        assert_eq!(
            verify_umod(&bytes).unwrap_err(),
            GraphLoadError::SchedulingSectionMissing
        );
    }

    #[test]
    fn bad_external_ref_syntax_fails_semantically() {
        let bytes =
            semantic_umod_with_extra_section(umod::SECTION_KIND_EXTERNAL_REFS, b"blob:../bad\0");

        assert_eq!(
            verify_umod(&bytes).unwrap_err(),
            GraphLoadError::OpaqueResourceSyntaxBad { ref_index: 0 }
        );
    }
}
