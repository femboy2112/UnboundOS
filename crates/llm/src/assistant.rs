//! Local assistant data surfaces for M11.
//!
//! Assistant output is proposal data only. It cannot mutate graph state,
//! execute generated code, or bypass graph verification/operator approval.

use core::fmt::{self, Write};
use core::str;

pub const ACTION_PAYLOAD_WORDS: usize = 4;
pub const ACTION_TEXT_BYTES: usize = 32;
pub const ACTION_TEXT_BYTES_U32: u32 = 32;
pub const GRAPH_NODE_NONE: u32 = u32::MAX;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AssistantActionKind {
    ExplainOnly = 0,
    ProposeGraphPatch = 1,
    ProposeOperatorNote = 2,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AssistantActionError {
    UnsupportedKind { requested: u32 },
    OutputOverflow { required: u32, available: u32 },
    TextTooLong { required: u32, available: u32 },
}

/// Graph display facts accepted by the assistant explainer.
///
/// This is copied data only: it has no runtime graph handle, constructor, or
/// mutation authority.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct GraphExplanationInput {
    pub graph_id: u64,
    pub node_count: u32,
    pub wire_count: u32,
    pub active_node: u32,
    pub last_completed_node: u32,
}

impl GraphExplanationInput {
    #[must_use]
    pub const fn new(
        graph_id: u64,
        node_count: u32,
        wire_count: u32,
        active_node: Option<u32>,
        last_completed_node: Option<u32>,
    ) -> Self {
        Self {
            graph_id,
            node_count,
            wire_count,
            active_node: encode_optional_node(active_node),
            last_completed_node: encode_optional_node(last_completed_node),
        }
    }

    #[must_use]
    pub const fn active_node(&self) -> Option<u32> {
        decode_optional_node(self.active_node)
    }

    #[must_use]
    pub const fn last_completed_node(&self) -> Option<u32> {
        decode_optional_node(self.last_completed_node)
    }
}

/// Fixed-width proposal record. Payload words are symbolic data, not pointers.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct AssistantActionProposal {
    pub kind: u32,
    pub target_resource_id: u64,
    pub payload_words: [u32; ACTION_PAYLOAD_WORDS],
    pub text_len: u32,
    pub text: [u8; ACTION_TEXT_BYTES],
}

impl AssistantActionProposal {
    /// Build an explain-only proposal.
    ///
    /// # Errors
    ///
    /// Returns `AssistantActionError::TextTooLong` when `text` does not fit in
    /// the fixed proposal storage.
    pub fn explain_only(text: &str) -> Result<Self, AssistantActionError> {
        Self::new(
            AssistantActionKind::ExplainOnly,
            0,
            [0; ACTION_PAYLOAD_WORDS],
            text,
        )
    }

    /// Build a fixed-width action proposal record.
    ///
    /// # Errors
    ///
    /// Returns `AssistantActionError::TextTooLong` when `text` does not fit in
    /// the fixed proposal storage.
    pub fn new(
        kind: AssistantActionKind,
        target_resource_id: u64,
        payload_words: [u32; ACTION_PAYLOAD_WORDS],
        text: &str,
    ) -> Result<Self, AssistantActionError> {
        let bytes = text.as_bytes();
        if bytes.len() > ACTION_TEXT_BYTES {
            return Err(AssistantActionError::TextTooLong {
                required: len_to_u32(bytes.len()),
                available: ACTION_TEXT_BYTES_U32,
            });
        }
        let mut stored_text = [0u8; ACTION_TEXT_BYTES];
        stored_text[..bytes.len()].copy_from_slice(bytes);
        Ok(Self {
            kind: kind as u32,
            target_resource_id,
            payload_words,
            text_len: len_to_u32(bytes.len()),
            text: stored_text,
        })
    }

    #[must_use]
    pub fn kind(self) -> Option<AssistantActionKind> {
        match self.kind {
            0 => Some(AssistantActionKind::ExplainOnly),
            1 => Some(AssistantActionKind::ProposeGraphPatch),
            2 => Some(AssistantActionKind::ProposeOperatorNote),
            _ => None,
        }
    }

    #[must_use]
    pub fn text_bytes(&self) -> &[u8] {
        let len = usize::try_from(self.text_len)
            .unwrap_or(ACTION_TEXT_BYTES)
            .min(ACTION_TEXT_BYTES);
        &self.text[..len]
    }
}

/// Caller-owned proposal buffer. This is not a queue or execution hook.
pub struct StructuredActionBuffer<'a> {
    storage: &'a mut [AssistantActionProposal],
    len: usize,
}

impl<'a> StructuredActionBuffer<'a> {
    #[must_use]
    pub fn new(storage: &'a mut [AssistantActionProposal]) -> Self {
        Self { storage, len: 0 }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub fn capacity(&self) -> usize {
        self.storage.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Append proposal data to caller-provided storage.
    ///
    /// # Errors
    ///
    /// Returns `AssistantActionError` for unsupported proposal kinds or when
    /// storage is full.
    pub fn push(&mut self, proposal: AssistantActionProposal) -> Result<(), AssistantActionError> {
        if proposal.kind().is_none() {
            return Err(AssistantActionError::UnsupportedKind {
                requested: proposal.kind,
            });
        }
        if self.len == self.storage.len() {
            return Err(AssistantActionError::OutputOverflow {
                required: len_to_u32(self.len + 1),
                available: len_to_u32(self.storage.len()),
            });
        }
        self.storage[self.len] = proposal;
        self.len += 1;
        Ok(())
    }

    #[must_use]
    pub fn proposals(&self) -> &[AssistantActionProposal] {
        &self.storage[..self.len]
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }
}

/// Format a graph explanation into caller-provided storage.
///
/// # Errors
///
/// Returns `AssistantActionError::OutputOverflow` when `output` is too small
/// for the deterministic explanation text.
pub fn explain_graph<'a>(
    input: &GraphExplanationInput,
    output: &'a mut [u8],
) -> Result<&'a str, AssistantActionError> {
    let mut writer = BoundedTextWriter::new(output);
    write!(
        writer,
        "graph=0x{:016x} nodes={} wires={} active_node=",
        input.graph_id, input.node_count, input.wire_count
    )
    .map_err(|_| writer.overflow())?;
    write_optional_node(&mut writer, input.active_node())?;
    writer
        .write_str(" last_completed_node=")
        .map_err(|_| writer.overflow())?;
    write_optional_node(&mut writer, input.last_completed_node())?;
    writer.finish()
}

const fn encode_optional_node(node: Option<u32>) -> u32 {
    match node {
        Some(id) => id,
        None => GRAPH_NODE_NONE,
    }
}

const fn decode_optional_node(node: u32) -> Option<u32> {
    if node == GRAPH_NODE_NONE {
        None
    } else {
        Some(node)
    }
}

fn len_to_u32(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

fn write_optional_node(
    writer: &mut BoundedTextWriter<'_>,
    node: Option<u32>,
) -> Result<(), AssistantActionError> {
    match node {
        Some(id) => write!(writer, "{id}").map_err(|_| writer.overflow()),
        None => writer.write_str("none").map_err(|_| writer.overflow()),
    }
}

struct BoundedTextWriter<'a> {
    output: &'a mut [u8],
    len: usize,
    required: usize,
}

impl<'a> BoundedTextWriter<'a> {
    const fn new(output: &'a mut [u8]) -> Self {
        Self {
            output,
            len: 0,
            required: 0,
        }
    }

    fn overflow(&self) -> AssistantActionError {
        AssistantActionError::OutputOverflow {
            required: len_to_u32(self.required),
            available: len_to_u32(self.output.len()),
        }
    }

    fn finish(self) -> Result<&'a str, AssistantActionError> {
        str::from_utf8(&self.output[..self.len]).map_err(|_| self.overflow())
    }
}

impl Write for BoundedTextWriter<'_> {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let bytes = value.as_bytes();
        self.required = self.required.saturating_add(bytes.len());
        if self.required > self.output.len() {
            return Err(fmt::Error);
        }
        let end = self.len + bytes.len();
        self.output[self.len..end].copy_from_slice(bytes);
        self.len = end;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_proposal() -> AssistantActionProposal {
        AssistantActionProposal::explain_only("").expect("empty proposal")
    }

    #[test]
    fn action_buffer_uses_caller_provided_storage() {
        let mut storage = [empty_proposal(); 2];
        let mut buffer = StructuredActionBuffer::new(&mut storage);
        let first = AssistantActionProposal::explain_only("graph ok").unwrap();
        let second = AssistantActionProposal::new(
            AssistantActionKind::ProposeOperatorNote,
            7,
            [1, 2, 3, 4],
            "review",
        )
        .unwrap();

        buffer.push(first).unwrap();
        buffer.push(second).unwrap();

        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.capacity(), 2);
        assert_eq!(
            buffer.proposals()[0].kind(),
            Some(AssistantActionKind::ExplainOnly)
        );
        assert_eq!(
            buffer.proposals()[1].kind(),
            Some(AssistantActionKind::ProposeOperatorNote)
        );
        assert_eq!(buffer.proposals()[1].target_resource_id, 7);
        assert_eq!(buffer.proposals()[1].payload_words, [1, 2, 3, 4]);
    }

    #[test]
    fn action_buffer_reports_overflow_and_rejects_unknown_kind() {
        let mut storage = [empty_proposal(); 1];
        let mut buffer = StructuredActionBuffer::new(&mut storage);
        buffer
            .push(AssistantActionProposal::explain_only("ok").unwrap())
            .unwrap();

        assert_eq!(
            buffer
                .push(AssistantActionProposal::explain_only("full").unwrap())
                .unwrap_err(),
            AssistantActionError::OutputOverflow {
                required: 2,
                available: 1,
            }
        );

        let mut invalid = AssistantActionProposal::explain_only("bad").unwrap();
        invalid.kind = 99;
        buffer.clear();
        assert_eq!(
            buffer.push(invalid).unwrap_err(),
            AssistantActionError::UnsupportedKind { requested: 99 }
        );
    }

    #[test]
    fn proposal_text_is_fixed_width_data() {
        let proposal = AssistantActionProposal::explain_only("explain").unwrap();
        assert_eq!(proposal.text_len, 7);
        assert_eq!(&proposal.text[..7], b"explain");
        assert_eq!(proposal.text_bytes(), b"explain");

        assert_eq!(
            AssistantActionProposal::explain_only("x".repeat(ACTION_TEXT_BYTES + 1).as_str())
                .unwrap_err(),
            AssistantActionError::TextTooLong {
                required: 33,
                available: ACTION_TEXT_BYTES_U32,
            }
        );
    }

    #[test]
    fn graph_explanation_formats_display_snapshot_fields() {
        let input = GraphExplanationInput::new(0x0053_5453, 3, 2, None, Some(3));
        let mut output = [0u8; 96];

        let explanation = explain_graph(&input, &mut output).unwrap();

        assert_eq!(
            explanation,
            "graph=0x0000000000535453 nodes=3 wires=2 active_node=none last_completed_node=3"
        );
    }

    #[test]
    fn graph_explanation_reports_caller_output_overflow() {
        let input = GraphExplanationInput::new(0x0053_5453, 3, 2, Some(1), Some(0));
        let mut output = [0u8; 16];

        assert_eq!(
            explain_graph(&input, &mut output).unwrap_err(),
            AssistantActionError::OutputOverflow {
                required: 17,
                available: 16,
            }
        );
    }
}
