//! Local assistant data surfaces for M11/M12.
//!
//! Assistant output is proposal data only. It cannot mutate graph state,
//! execute generated code, or bypass graph verification/operator approval.

use core::fmt::{self, Write};
use core::str;

use crate::retrieval::{
    pack_retrieval_context, RetrievalError, RetrievalIndexSnapshot, RetrievalResult,
};

pub const ACTION_PAYLOAD_WORDS: usize = 4;
pub const ACTION_TEXT_BYTES: usize = 32;
pub const ACTION_TEXT_BYTES_U32: u32 = 32;
pub const GRAPH_NODE_NONE: u32 = u32::MAX;
pub const SSOD_REASON_BYTES: usize = 32;
pub const SSOD_REASON_BYTES_U32: u32 = 32;

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
    UnsupportedRequest { requested: u32 },
    ActionBufferRequired,
    OutputOverflow { required: u32, available: u32 },
    TextTooLong { required: u32, available: u32 },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AssistantRetrievalError {
    RetrievalError(RetrievalError),
    Action(AssistantActionError),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AssistantRequestKind {
    ExplainGraph = 0,
    ExplainSsod = 1,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum SsodFaultFamily {
    CpuException = 1,
    RustPanic = 2,
    Arena = 3,
    Unknown = 255,
}

/// Explicit local assistant request surface for M11.
///
/// Requests are explain-only. Optional proposed actions stay as fixed-width
/// data and must be written through `StructuredActionBuffer`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AssistantExplainRequest<'a> {
    Graph {
        input: &'a GraphExplanationInput,
        proposed_action: Option<AssistantActionProposal>,
    },
    Ssod {
        input: &'a SsodExplanationInput,
        proposed_action: Option<AssistantActionProposal>,
    },
    Unsupported {
        requested: u32,
    },
}

impl AssistantExplainRequest<'_> {
    #[must_use]
    pub const fn kind(&self) -> Option<AssistantRequestKind> {
        match self {
            Self::Graph { .. } => Some(AssistantRequestKind::ExplainGraph),
            Self::Ssod { .. } => Some(AssistantRequestKind::ExplainSsod),
            Self::Unsupported { .. } => None,
        }
    }

    const fn proposed_action(&self) -> Option<AssistantActionProposal> {
        match self {
            Self::Graph {
                proposed_action, ..
            }
            | Self::Ssod {
                proposed_action, ..
            } => *proposed_action,
            Self::Unsupported { .. } => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct AssistantExplainResponse {
    pub request_kind: u32,
    pub explanation_len: u32,
    pub action_count: u32,
}

/// Explicit assistant retrieval request surface for M12.
///
/// Retrieval output is explanatory context only. Optional proposed actions stay
/// as data and must be written through `StructuredActionBuffer`.
#[derive(Copy, Clone, Debug)]
pub struct AssistantRetrievalRequest<'a> {
    pub index: &'a RetrievalIndexSnapshot<'a>,
    pub results: &'a [RetrievalResult],
    pub proposed_action: Option<AssistantActionProposal>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct AssistantRetrievalResponse {
    pub context_len: u32,
    pub action_count: u32,
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

/// SSOD diagnostic facts accepted by the assistant explainer.
///
/// This is copied diagnostic data only; it does not replace or route around the
/// kernel SSOD fatal record.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct SsodExplanationInput {
    pub vector: u8,
    pub fault_family: u32,
    pub reason_len: u32,
    pub reason: [u8; SSOD_REASON_BYTES],
    pub instruction_pointer: u64,
    pub has_error_code: u8,
    pub error_code: u64,
}

impl SsodExplanationInput {
    /// Build a fixed-width SSOD explanation input.
    ///
    /// # Errors
    ///
    /// Returns `AssistantActionError::TextTooLong` when `reason` does not fit
    /// in the fixed diagnostic storage.
    pub fn new(
        fault_family: SsodFaultFamily,
        vector: u8,
        reason: &str,
        instruction_pointer: u64,
        error_code: Option<u64>,
    ) -> Result<Self, AssistantActionError> {
        let reason_bytes = reason.as_bytes();
        if reason_bytes.len() > SSOD_REASON_BYTES {
            return Err(AssistantActionError::TextTooLong {
                required: len_to_u32(reason_bytes.len()),
                available: SSOD_REASON_BYTES_U32,
            });
        }

        let mut stored_reason = [0u8; SSOD_REASON_BYTES];
        stored_reason[..reason_bytes.len()].copy_from_slice(reason_bytes);
        Ok(Self {
            vector,
            fault_family: fault_family as u32,
            reason_len: len_to_u32(reason_bytes.len()),
            reason: stored_reason,
            instruction_pointer,
            has_error_code: u8::from(error_code.is_some()),
            error_code: error_code.unwrap_or(0),
        })
    }

    #[must_use]
    pub fn fault_family(self) -> SsodFaultFamily {
        match self.fault_family {
            1 => SsodFaultFamily::CpuException,
            2 => SsodFaultFamily::RustPanic,
            3 => SsodFaultFamily::Arena,
            _ => SsodFaultFamily::Unknown,
        }
    }

    #[must_use]
    pub const fn error_code(self) -> Option<u64> {
        if self.has_error_code == 0 {
            None
        } else {
            Some(self.error_code)
        }
    }

    #[must_use]
    pub fn reason_bytes(&self) -> &[u8] {
        let len = usize::try_from(self.reason_len)
            .unwrap_or(SSOD_REASON_BYTES)
            .min(SSOD_REASON_BYTES);
        &self.reason[..len]
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

/// Format an SSOD explanation into caller-provided storage.
///
/// # Errors
///
/// Returns `AssistantActionError::OutputOverflow` when `output` is too small
/// for the deterministic explanation text.
pub fn explain_ssod<'a>(
    input: &SsodExplanationInput,
    output: &'a mut [u8],
) -> Result<&'a str, AssistantActionError> {
    let mut writer = BoundedTextWriter::new(output);
    writer
        .write_str("ssod reason=")
        .map_err(|_| writer.overflow())?;
    write_reason(&mut writer, input)?;
    write!(
        writer,
        " rip=0x{:016x} fault_family=",
        input.instruction_pointer
    )
    .map_err(|_| writer.overflow())?;
    writer
        .write_str(ssod_fault_family_name(input.fault_family()))
        .map_err(|_| writer.overflow())?;
    write!(writer, " vector=0x{:02x} error_code=", input.vector).map_err(|_| writer.overflow())?;
    write_optional_error_code(&mut writer, input.error_code())?;
    writer.finish()
}

/// Route one local assistant explanation request.
///
/// # Errors
///
/// Returns structured errors for unsupported request kinds, missing action
/// buffers, proposal validation failures, or output overflow.
pub fn assistant_explain(
    request: AssistantExplainRequest<'_>,
    output: &mut [u8],
    actions: Option<&mut StructuredActionBuffer<'_>>,
) -> Result<AssistantExplainResponse, AssistantActionError> {
    let (request_kind, explanation_len) = match request {
        AssistantExplainRequest::Graph { input, .. } => {
            let explanation = explain_graph(input, output)?;
            (
                AssistantRequestKind::ExplainGraph as u32,
                len_to_u32(explanation.len()),
            )
        }
        AssistantExplainRequest::Ssod { input, .. } => {
            let explanation = explain_ssod(input, output)?;
            (
                AssistantRequestKind::ExplainSsod as u32,
                len_to_u32(explanation.len()),
            )
        }
        AssistantExplainRequest::Unsupported { requested } => {
            return Err(AssistantActionError::UnsupportedRequest { requested });
        }
    };

    let action_count = match request.proposed_action() {
        Some(proposal) => {
            let buffer = actions.ok_or(AssistantActionError::ActionBufferRequired)?;
            buffer.push(proposal)?;
            len_to_u32(buffer.len())
        }
        None => 0,
    };

    Ok(AssistantExplainResponse {
        request_kind,
        explanation_len,
        action_count,
    })
}

/// Pack local retrieval results into assistant context.
///
/// # Errors
///
/// Returns structured retrieval or action-buffer errors. This function never
/// mutates graph state and never consumes host paths.
pub fn assistant_retrieve_context(
    request: AssistantRetrievalRequest<'_>,
    output: &mut [u8],
    actions: Option<&mut StructuredActionBuffer<'_>>,
) -> Result<AssistantRetrievalResponse, AssistantRetrievalError> {
    let context_len = pack_retrieval_context(request.index, request.results, output)
        .map_err(AssistantRetrievalError::RetrievalError)?;

    let action_count = match request.proposed_action {
        Some(proposal) => {
            let buffer = actions.ok_or(AssistantRetrievalError::Action(
                AssistantActionError::ActionBufferRequired,
            ))?;
            buffer
                .push(proposal)
                .map_err(AssistantRetrievalError::Action)?;
            len_to_u32(buffer.len())
        }
        None => 0,
    };

    Ok(AssistantRetrievalResponse {
        context_len: len_to_u32(context_len),
        action_count,
    })
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

fn write_reason(
    writer: &mut BoundedTextWriter<'_>,
    input: &SsodExplanationInput,
) -> Result<(), AssistantActionError> {
    let reason = str::from_utf8(input.reason_bytes()).unwrap_or("<invalid-reason>");
    writer.write_str(reason).map_err(|_| writer.overflow())
}

fn write_optional_error_code(
    writer: &mut BoundedTextWriter<'_>,
    error_code: Option<u64>,
) -> Result<(), AssistantActionError> {
    match error_code {
        Some(code) => write!(writer, "0x{code:016x}").map_err(|_| writer.overflow()),
        None => writer.write_str("none").map_err(|_| writer.overflow()),
    }
}

const fn ssod_fault_family_name(fault_family: SsodFaultFamily) -> &'static str {
    match fault_family {
        SsodFaultFamily::CpuException => "cpu_exception",
        SsodFaultFamily::RustPanic => "rust_panic",
        SsodFaultFamily::Arena => "arena",
        SsodFaultFamily::Unknown => "unknown",
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

    use crate::retrieval::RetrievalDocumentRef;

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

    #[test]
    fn ssod_explanation_formats_structured_diagnostic_fields() {
        let input = SsodExplanationInput::new(
            SsodFaultFamily::CpuException,
            14,
            "page_fault",
            0xFFFF_8000_0000_1234,
            Some(0x2),
        )
        .unwrap();
        let mut output = [0u8; 128];

        let explanation = explain_ssod(&input, &mut output).unwrap();

        assert_eq!(
            explanation,
            "ssod reason=page_fault rip=0xffff800000001234 fault_family=cpu_exception vector=0x0e error_code=0x0000000000000002"
        );
    }

    #[test]
    fn ssod_explanation_formats_absent_error_code_and_rejects_long_reason() {
        let input =
            SsodExplanationInput::new(SsodFaultFamily::RustPanic, 0xFF, "rust_panic", 0, None)
                .unwrap();
        let mut output = [0u8; 128];

        let explanation = explain_ssod(&input, &mut output).unwrap();

        assert_eq!(
            explanation,
            "ssod reason=rust_panic rip=0x0000000000000000 fault_family=rust_panic vector=0xff error_code=none"
        );
        assert_eq!(
            SsodExplanationInput::new(
                SsodFaultFamily::Unknown,
                0xFF,
                "x".repeat(SSOD_REASON_BYTES + 1).as_str(),
                0,
                None,
            )
            .unwrap_err(),
            AssistantActionError::TextTooLong {
                required: 33,
                available: SSOD_REASON_BYTES_U32,
            }
        );
    }

    #[test]
    fn ssod_explanation_reports_caller_output_overflow() {
        let input =
            SsodExplanationInput::new(SsodFaultFamily::Arena, 0xFF, "arena_alloc_error", 0, None)
                .unwrap();
        let mut output = [0u8; 8];

        assert_eq!(
            explain_ssod(&input, &mut output).unwrap_err(),
            AssistantActionError::OutputOverflow {
                required: 12,
                available: 8,
            }
        );
    }

    #[test]
    fn assistant_explain_routes_graph_requests_without_actions() {
        let input = GraphExplanationInput::new(0x0053_5453, 3, 2, None, Some(3));
        let mut output = [0u8; 96];

        let response = assistant_explain(
            AssistantExplainRequest::Graph {
                input: &input,
                proposed_action: None,
            },
            &mut output,
            None,
        )
        .unwrap();

        assert_eq!(
            response,
            AssistantExplainResponse {
                request_kind: AssistantRequestKind::ExplainGraph as u32,
                explanation_len: 79,
                action_count: 0,
            }
        );
        assert_eq!(
            str::from_utf8(&output[..usize::try_from(response.explanation_len).unwrap()]).unwrap(),
            "graph=0x0000000000535453 nodes=3 wires=2 active_node=none last_completed_node=3"
        );
    }

    #[test]
    fn assistant_explain_routes_ssod_requests_and_buffers_actions() {
        let input =
            SsodExplanationInput::new(SsodFaultFamily::RustPanic, 0xFF, "rust_panic", 0, None)
                .unwrap();
        let proposal = AssistantActionProposal::new(
            AssistantActionKind::ProposeOperatorNote,
            9,
            [0; ACTION_PAYLOAD_WORDS],
            "inspect ssod",
        )
        .unwrap();
        let mut storage = [empty_proposal(); 1];
        let mut actions = StructuredActionBuffer::new(&mut storage);
        let mut output = [0u8; 128];

        let response = assistant_explain(
            AssistantExplainRequest::Ssod {
                input: &input,
                proposed_action: Some(proposal),
            },
            &mut output,
            Some(&mut actions),
        )
        .unwrap();

        assert_eq!(
            response.request_kind,
            AssistantRequestKind::ExplainSsod as u32
        );
        assert_eq!(response.action_count, 1);
        assert_eq!(actions.proposals(), &[proposal]);
    }

    #[test]
    fn assistant_explain_requires_buffer_for_proposals() {
        let input = GraphExplanationInput::new(0x0053_5453, 3, 2, None, Some(3));
        let proposal = AssistantActionProposal::explain_only("note").unwrap();
        let mut output = [0u8; 96];

        assert_eq!(
            assistant_explain(
                AssistantExplainRequest::Graph {
                    input: &input,
                    proposed_action: Some(proposal),
                },
                &mut output,
                None,
            )
            .unwrap_err(),
            AssistantActionError::ActionBufferRequired
        );
    }

    #[test]
    fn assistant_explain_rejects_unsupported_requests() {
        let mut output = [0u8; 96];

        assert_eq!(
            assistant_explain(
                AssistantExplainRequest::Unsupported { requested: 99 },
                &mut output,
                None,
            )
            .unwrap_err(),
            AssistantActionError::UnsupportedRequest { requested: 99 }
        );
    }

    #[test]
    fn assistant_retrieve_context_packs_explanatory_context() {
        let docs = [
            RetrievalDocumentRef::new("index:spec-13.1", "Spec", "assistant searches local docs")
                .unwrap(),
            RetrievalDocumentRef::new("blob:assistant-note", "Note", "context pack").unwrap(),
        ];
        let index = RetrievalIndexSnapshot::new(&docs).unwrap();
        let results = [
            RetrievalResult::new(0, 90, "index:spec-13.1").unwrap(),
            RetrievalResult::new(1, 70, "blob:assistant-note").unwrap(),
        ];
        let mut output = [0u8; 160];

        let response = assistant_retrieve_context(
            AssistantRetrievalRequest {
                index: &index,
                results: &results,
                proposed_action: None,
            },
            &mut output,
            None,
        )
        .unwrap();

        assert_eq!(response.action_count, 0);
        assert_eq!(
            str::from_utf8(&output[..usize::try_from(response.context_len).unwrap()]).unwrap(),
            "doc=index:spec-13.1\nsnippet=assistant searches local docs\n---\ndoc=blob:assistant-note\nsnippet=context pack\n---\n"
        );
    }

    #[test]
    fn assistant_retrieve_context_buffers_optional_actions() {
        let docs = [RetrievalDocumentRef::new("index:spec", "Spec", "assistant context").unwrap()];
        let index = RetrievalIndexSnapshot::new(&docs).unwrap();
        let results = [RetrievalResult::new(0, 90, "index:spec").unwrap()];
        let proposal = AssistantActionProposal::new(
            AssistantActionKind::ProposeOperatorNote,
            11,
            [0; ACTION_PAYLOAD_WORDS],
            "review context",
        )
        .unwrap();
        let mut storage = [empty_proposal(); 1];
        let mut actions = StructuredActionBuffer::new(&mut storage);
        let mut output = [0u8; 80];

        let response = assistant_retrieve_context(
            AssistantRetrievalRequest {
                index: &index,
                results: &results,
                proposed_action: Some(proposal),
            },
            &mut output,
            Some(&mut actions),
        )
        .unwrap();

        assert_eq!(response.action_count, 1);
        assert_eq!(actions.proposals(), &[proposal]);
    }

    #[test]
    fn assistant_retrieve_context_requires_action_buffer_and_maps_retrieval_errors() {
        let docs = [RetrievalDocumentRef::new("index:spec", "Spec", "retrieval").unwrap()];
        let index = RetrievalIndexSnapshot::new(&docs).unwrap();
        let results = [RetrievalResult::new(0, 90, "index:spec").unwrap()];
        let proposal = AssistantActionProposal::explain_only("note").unwrap();
        let mut output = [0u8; 80];

        assert_eq!(
            assistant_retrieve_context(
                AssistantRetrievalRequest {
                    index: &index,
                    results: &results,
                    proposed_action: Some(proposal),
                },
                &mut output,
                None,
            )
            .unwrap_err(),
            AssistantRetrievalError::Action(AssistantActionError::ActionBufferRequired)
        );

        let mismatched = [RetrievalResult::new(0, 90, "index:other").unwrap()];
        assert_eq!(
            assistant_retrieve_context(
                AssistantRetrievalRequest {
                    index: &index,
                    results: &mismatched,
                    proposed_action: None,
                },
                &mut output,
                None,
            )
            .unwrap_err(),
            AssistantRetrievalError::RetrievalError(RetrievalError::DocumentRefInvalid {
                index: 0
            })
        );
    }
}
