//! Local assistant data surfaces for M11.
//!
//! Assistant output is proposal data only. It cannot mutate graph state,
//! execute generated code, or bypass graph verification/operator approval.

pub const ACTION_PAYLOAD_WORDS: usize = 4;
pub const ACTION_TEXT_BYTES: usize = 32;
pub const ACTION_TEXT_BYTES_U32: u32 = 32;

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
    pub fn text_bytes(self) -> &'static [u8] {
        &[]
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

fn len_to_u32(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
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

        assert_eq!(
            AssistantActionProposal::explain_only("x".repeat(ACTION_TEXT_BYTES + 1).as_str())
                .unwrap_err(),
            AssistantActionError::TextTooLong {
                required: 33,
                available: ACTION_TEXT_BYTES_U32,
            }
        );
    }
}
