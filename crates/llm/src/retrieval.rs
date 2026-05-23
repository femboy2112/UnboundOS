//! Fixed-width local retrieval data contracts for M12.
//!
//! Retrieval records are data only. They do not read host files, mutate graph
//! state, spawn workers, or execute assistant output.

pub const RETRIEVAL_QUERY_BYTES: usize = 64;
pub const RETRIEVAL_QUERY_BYTES_U32: u32 = 64;
pub const RETRIEVAL_RESOURCE_REF_BYTES: usize = 72;
pub const RETRIEVAL_RESOURCE_REF_BYTES_U32: u32 = 72;
pub const RETRIEVAL_TITLE_BYTES: usize = 48;
pub const RETRIEVAL_TITLE_BYTES_U32: u32 = 48;
pub const RETRIEVAL_SNIPPET_BYTES: usize = 128;
pub const RETRIEVAL_SNIPPET_BYTES_U32: u32 = 128;
const LOCAL_SCHEME_BYTES: [u8; 8] = [108, 111, 99, 97, 108, 58, 47, 47];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RetrievalError {
    QueryTooLong { required: u32, available: u32 },
    QueryEmpty,
    ResourceRefTooLong { required: u32, available: u32 },
    ResourceRefInvalid,
    TextTooLong { required: u32, available: u32 },
    OutputOverflow { required: u32, available: u32 },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct RetrievalQuery {
    pub text_len: u32,
    pub text: [u8; RETRIEVAL_QUERY_BYTES],
}

impl RetrievalQuery {
    /// Build a fixed-width retrieval query.
    ///
    /// # Errors
    ///
    /// Returns `RetrievalError` when `text` is empty or exceeds bounded query
    /// storage.
    pub fn new(text: &str) -> Result<Self, RetrievalError> {
        let bytes = text.as_bytes();
        if bytes.is_empty() {
            return Err(RetrievalError::QueryEmpty);
        }
        if bytes.len() > RETRIEVAL_QUERY_BYTES {
            return Err(RetrievalError::QueryTooLong {
                required: len_to_u32(bytes.len()),
                available: RETRIEVAL_QUERY_BYTES_U32,
            });
        }

        let mut stored_text = [0u8; RETRIEVAL_QUERY_BYTES];
        stored_text[..bytes.len()].copy_from_slice(bytes);
        Ok(Self {
            text_len: len_to_u32(bytes.len()),
            text: stored_text,
        })
    }

    #[must_use]
    pub fn text_bytes(&self) -> &[u8] {
        let len = u32_to_usize(self.text_len).min(RETRIEVAL_QUERY_BYTES);
        &self.text[..len]
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct RetrievalDocumentRef {
    pub resource_ref_len: u32,
    pub resource_ref: [u8; RETRIEVAL_RESOURCE_REF_BYTES],
    pub title_len: u32,
    pub title: [u8; RETRIEVAL_TITLE_BYTES],
    pub snippet_len: u32,
    pub snippet: [u8; RETRIEVAL_SNIPPET_BYTES],
}

impl RetrievalDocumentRef {
    /// Build a fixed-width read-only document reference.
    ///
    /// # Errors
    ///
    /// Returns `RetrievalError` when `resource_ref` is not an opaque resource
    /// ID or any bounded text field is too large.
    pub fn new(resource_ref: &str, title: &str, snippet: &str) -> Result<Self, RetrievalError> {
        let resource_ref_bytes = resource_ref.as_bytes();
        if resource_ref_bytes.len() > RETRIEVAL_RESOURCE_REF_BYTES {
            return Err(RetrievalError::ResourceRefTooLong {
                required: len_to_u32(resource_ref_bytes.len()),
                available: RETRIEVAL_RESOURCE_REF_BYTES_U32,
            });
        }
        if !is_opaque_resource_ref(resource_ref_bytes) {
            return Err(RetrievalError::ResourceRefInvalid);
        }

        let title_bytes = checked_text_bytes(title, RETRIEVAL_TITLE_BYTES)?;
        let snippet_bytes = checked_text_bytes(snippet, RETRIEVAL_SNIPPET_BYTES)?;
        let mut stored_resource_ref = [0u8; RETRIEVAL_RESOURCE_REF_BYTES];
        let mut stored_title = [0u8; RETRIEVAL_TITLE_BYTES];
        let mut stored_snippet = [0u8; RETRIEVAL_SNIPPET_BYTES];

        stored_resource_ref[..resource_ref_bytes.len()].copy_from_slice(resource_ref_bytes);
        stored_title[..title_bytes.len()].copy_from_slice(title_bytes);
        stored_snippet[..snippet_bytes.len()].copy_from_slice(snippet_bytes);
        Ok(Self {
            resource_ref_len: len_to_u32(resource_ref_bytes.len()),
            resource_ref: stored_resource_ref,
            title_len: len_to_u32(title_bytes.len()),
            title: stored_title,
            snippet_len: len_to_u32(snippet_bytes.len()),
            snippet: stored_snippet,
        })
    }

    #[must_use]
    pub fn resource_ref_bytes(&self) -> &[u8] {
        let len = u32_to_usize(self.resource_ref_len).min(RETRIEVAL_RESOURCE_REF_BYTES);
        &self.resource_ref[..len]
    }

    #[must_use]
    pub fn title_bytes(&self) -> &[u8] {
        let len = u32_to_usize(self.title_len).min(RETRIEVAL_TITLE_BYTES);
        &self.title[..len]
    }

    #[must_use]
    pub fn snippet_bytes(&self) -> &[u8] {
        let len = u32_to_usize(self.snippet_len).min(RETRIEVAL_SNIPPET_BYTES);
        &self.snippet[..len]
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct RetrievalResult {
    pub document_index: u32,
    pub score: u32,
    pub resource_ref_len: u32,
    pub resource_ref: [u8; RETRIEVAL_RESOURCE_REF_BYTES],
}

impl RetrievalResult {
    /// Build a fixed-width retrieval result.
    ///
    /// # Errors
    ///
    /// Returns `RetrievalError` when the result resource reference is not an
    /// opaque resource ID or does not fit fixed storage.
    pub fn new(
        document_index: u32,
        score: u32,
        resource_ref: &str,
    ) -> Result<Self, RetrievalError> {
        let resource_ref_bytes = resource_ref.as_bytes();
        if resource_ref_bytes.len() > RETRIEVAL_RESOURCE_REF_BYTES {
            return Err(RetrievalError::ResourceRefTooLong {
                required: len_to_u32(resource_ref_bytes.len()),
                available: RETRIEVAL_RESOURCE_REF_BYTES_U32,
            });
        }
        if !is_opaque_resource_ref(resource_ref_bytes) {
            return Err(RetrievalError::ResourceRefInvalid);
        }

        let mut stored_resource_ref = [0u8; RETRIEVAL_RESOURCE_REF_BYTES];
        stored_resource_ref[..resource_ref_bytes.len()].copy_from_slice(resource_ref_bytes);
        Ok(Self {
            document_index,
            score,
            resource_ref_len: len_to_u32(resource_ref_bytes.len()),
            resource_ref: stored_resource_ref,
        })
    }

    #[must_use]
    pub fn resource_ref_bytes(&self) -> &[u8] {
        let len = u32_to_usize(self.resource_ref_len).min(RETRIEVAL_RESOURCE_REF_BYTES);
        &self.resource_ref[..len]
    }
}

pub struct RetrievalResultBuffer<'a> {
    storage: &'a mut [RetrievalResult],
    len: usize,
}

impl<'a> RetrievalResultBuffer<'a> {
    #[must_use]
    pub fn new(storage: &'a mut [RetrievalResult]) -> Self {
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

    /// Append retrieval result data to caller-provided storage.
    ///
    /// # Errors
    ///
    /// Returns `RetrievalError::OutputOverflow` when storage is full.
    pub fn push(&mut self, result: RetrievalResult) -> Result<(), RetrievalError> {
        if self.len == self.storage.len() {
            return Err(RetrievalError::OutputOverflow {
                required: len_to_u32(self.len + 1),
                available: len_to_u32(self.storage.len()),
            });
        }
        self.storage[self.len] = result;
        self.len += 1;
        Ok(())
    }

    #[must_use]
    pub fn results(&self) -> &[RetrievalResult] {
        &self.storage[..self.len]
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }
}

fn checked_text_bytes(text: &str, capacity: usize) -> Result<&[u8], RetrievalError> {
    let bytes = text.as_bytes();
    if bytes.len() > capacity {
        return Err(RetrievalError::TextTooLong {
            required: len_to_u32(bytes.len()),
            available: len_to_u32(capacity),
        });
    }
    Ok(bytes)
}

fn is_opaque_resource_ref(bytes: &[u8]) -> bool {
    if bytes.is_empty() || looks_like_path(bytes) {
        return false;
    }
    let Some(colon) = bytes.iter().position(|byte| *byte == b':') else {
        return false;
    };
    let (kind, opaque_with_colon) = bytes.split_at(colon);
    let opaque_id = &opaque_with_colon[1..];
    matches!(kind, b"index" | b"blob" | b"profile")
        && !opaque_id.is_empty()
        && opaque_id.len() <= 64
        && opaque_id.iter().all(|byte| opaque_id_char(*byte))
}

fn looks_like_path(bytes: &[u8]) -> bool {
    bytes.starts_with(&LOCAL_SCHEME_BYTES)
        || bytes.contains(&b'/')
        || bytes.contains(&b'\\')
        || bytes.windows(2).any(|window| window == b"..")
        || bytes.starts_with(b"~")
}

const fn opaque_id_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-')
}

fn len_to_u32(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

fn u32_to_usize(value: u32) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_result() -> RetrievalResult {
        RetrievalResult::new(0, 0, "index:empty").expect("empty retrieval result")
    }

    #[test]
    fn retrieval_query_is_fixed_width_data() {
        let query = RetrievalQuery::new("arena diagnostics").unwrap();

        assert_eq!(query.text_len, 17);
        assert_eq!(query.text_bytes(), b"arena diagnostics");
        assert_eq!(
            RetrievalQuery::new("").unwrap_err(),
            RetrievalError::QueryEmpty
        );
        assert_eq!(
            RetrievalQuery::new("x".repeat(RETRIEVAL_QUERY_BYTES + 1).as_str()).unwrap_err(),
            RetrievalError::QueryTooLong {
                required: 65,
                available: RETRIEVAL_QUERY_BYTES_U32,
            }
        );
    }

    #[test]
    fn document_ref_accepts_opaque_ids_and_rejects_path_shapes() {
        let doc = RetrievalDocumentRef::new(
            "index:spec-13.1",
            "Spec section",
            "assistant searches local docs",
        )
        .unwrap();

        assert_eq!(doc.resource_ref_bytes(), b"index:spec-13.1");
        assert_eq!(doc.title_bytes(), b"Spec section");
        assert_eq!(doc.snippet_bytes(), b"assistant searches local docs");

        let local_url = [
            b'l', b'o', b'c', b'a', b'l', b':', b'/', b'/', b'd', b'o', b'c',
        ];
        let rooted_path = [b'/', b'd', b'o', b'c'];
        for rejected in [&local_url[..], &rooted_path[..], b"index:bad/id".as_slice()] {
            assert_eq!(
                RetrievalDocumentRef::new(core::str::from_utf8(rejected).unwrap(), "t", "s")
                    .unwrap_err(),
                RetrievalError::ResourceRefInvalid
            );
        }
    }

    #[test]
    fn document_ref_rejects_oversized_text() {
        assert_eq!(
            RetrievalDocumentRef::new(
                "index:spec",
                "x".repeat(RETRIEVAL_TITLE_BYTES + 1).as_str(),
                "snippet",
            )
            .unwrap_err(),
            RetrievalError::TextTooLong {
                required: 49,
                available: RETRIEVAL_TITLE_BYTES_U32,
            }
        );
    }

    #[test]
    fn retrieval_result_buffer_uses_caller_storage() {
        let mut storage = [empty_result(); 2];
        let mut buffer = RetrievalResultBuffer::new(&mut storage);
        let first = RetrievalResult::new(2, 90, "index:spec-13.1").unwrap();
        let second = RetrievalResult::new(3, 80, "blob:assistant-note").unwrap();

        buffer.push(first).unwrap();
        buffer.push(second).unwrap();

        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.capacity(), 2);
        assert_eq!(buffer.results(), &[first, second]);
        assert_eq!(
            buffer.push(RetrievalResult::new(4, 70, "index:overflow").unwrap()),
            Err(RetrievalError::OutputOverflow {
                required: 3,
                available: 2,
            })
        );
        buffer.clear();
        assert!(buffer.is_empty());
    }
}
