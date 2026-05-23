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
    IndexEmpty,
    DuplicateResourceRef { first: u32, duplicate: u32 },
    DocumentRefInvalid { index: u32 },
    UnsupportedQuery,
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

    #[must_use]
    pub fn is_valid(&self) -> bool {
        u32_to_usize(self.resource_ref_len) <= RETRIEVAL_RESOURCE_REF_BYTES
            && u32_to_usize(self.title_len) <= RETRIEVAL_TITLE_BYTES
            && u32_to_usize(self.snippet_len) <= RETRIEVAL_SNIPPET_BYTES
            && is_opaque_resource_ref(self.resource_ref_bytes())
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
        Self::from_resource_ref_bytes(document_index, score, resource_ref.as_bytes())
    }

    fn from_resource_ref_bytes(
        document_index: u32,
        score: u32,
        resource_ref_bytes: &[u8],
    ) -> Result<Self, RetrievalError> {
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

/// Rank local documents deterministically into caller-provided output.
///
/// # Errors
///
/// Returns `RetrievalError` when `query` has no searchable bytes or `output`
/// cannot hold the requested top-k result set.
pub fn retrieve_top_k(
    index: &RetrievalIndexSnapshot<'_>,
    query: &RetrievalQuery,
    top_k: usize,
    output: &mut RetrievalResultBuffer<'_>,
) -> Result<usize, RetrievalError> {
    if top_k == 0 || !query.text_bytes().iter().any(u8::is_ascii_alphanumeric) {
        return Err(RetrievalError::UnsupportedQuery);
    }

    let mut emitted = 0usize;
    while emitted < top_k {
        let Some((document_index, score)) = next_ranked_document(index, query, output.results())
        else {
            return Ok(emitted);
        };

        let document = index
            .get(document_index)
            .ok_or(RetrievalError::DocumentRefInvalid {
                index: len_to_u32(document_index),
            })?;
        output.push(RetrievalResult::from_resource_ref_bytes(
            len_to_u32(document_index),
            score,
            document.resource_ref_bytes(),
        )?)?;
        emitted += 1;
    }
    Ok(emitted)
}

/// Pack retrieved snippets into deterministic assistant context.
///
/// # Errors
///
/// Returns `RetrievalError` if a result does not match the index snapshot or
/// if `output` cannot hold the full context.
pub fn pack_retrieval_context(
    index: &RetrievalIndexSnapshot<'_>,
    results: &[RetrievalResult],
    output: &mut [u8],
) -> Result<usize, RetrievalError> {
    let mut writer = RetrievalContextWriter::new(output);
    for result in results {
        let document_index = u32_to_usize(result.document_index);
        let document = index
            .get(document_index)
            .ok_or(RetrievalError::DocumentRefInvalid {
                index: result.document_index,
            })?;
        if result.resource_ref_bytes() != document.resource_ref_bytes() {
            return Err(RetrievalError::DocumentRefInvalid {
                index: result.document_index,
            });
        }

        writer.write(b"doc=")?;
        writer.write(document.resource_ref_bytes())?;
        writer.write(b"\nsnippet=")?;
        writer.write(document.snippet_bytes())?;
        writer.write(b"\n---\n")?;
    }
    Ok(writer.len())
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

/// Read-only local document index over caller-owned document records.
#[derive(Debug)]
pub struct RetrievalIndexSnapshot<'a> {
    documents: &'a [RetrievalDocumentRef],
}

impl<'a> RetrievalIndexSnapshot<'a> {
    /// Build a read-only local document index snapshot.
    ///
    /// # Errors
    ///
    /// Returns `RetrievalError` when the index is empty, contains invalid
    /// document references, or repeats the same opaque document ID.
    pub fn new(documents: &'a [RetrievalDocumentRef]) -> Result<Self, RetrievalError> {
        if documents.is_empty() {
            return Err(RetrievalError::IndexEmpty);
        }

        for (index, document) in documents.iter().enumerate() {
            if !document.is_valid() {
                return Err(RetrievalError::DocumentRefInvalid {
                    index: len_to_u32(index),
                });
            }
        }

        for first in 0..documents.len() {
            for duplicate in (first + 1)..documents.len() {
                if documents[first].resource_ref_bytes()
                    == documents[duplicate].resource_ref_bytes()
                {
                    return Err(RetrievalError::DuplicateResourceRef {
                        first: len_to_u32(first),
                        duplicate: len_to_u32(duplicate),
                    });
                }
            }
        }

        Ok(Self { documents })
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    #[must_use]
    pub const fn documents(&self) -> &'a [RetrievalDocumentRef] {
        self.documents
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&'a RetrievalDocumentRef> {
        self.documents.get(index)
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

struct RetrievalContextWriter<'a> {
    output: &'a mut [u8],
    len: usize,
    required: usize,
}

impl<'a> RetrievalContextWriter<'a> {
    const fn new(output: &'a mut [u8]) -> Self {
        Self {
            output,
            len: 0,
            required: 0,
        }
    }

    const fn len(&self) -> usize {
        self.len
    }

    fn write(&mut self, bytes: &[u8]) -> Result<(), RetrievalError> {
        self.required = self.required.saturating_add(bytes.len());
        if self.required > self.output.len() {
            return Err(RetrievalError::OutputOverflow {
                required: len_to_u32(self.required),
                available: len_to_u32(self.output.len()),
            });
        }
        let end = self.len + bytes.len();
        self.output[self.len..end].copy_from_slice(bytes);
        self.len = end;
        Ok(())
    }
}

fn next_ranked_document(
    index: &RetrievalIndexSnapshot<'_>,
    query: &RetrievalQuery,
    emitted: &[RetrievalResult],
) -> Option<(usize, u32)> {
    let mut best: Option<(usize, u32)> = None;
    for (candidate_index, document) in index.documents().iter().enumerate() {
        if emitted
            .iter()
            .any(|result| result.document_index == len_to_u32(candidate_index))
        {
            continue;
        }

        let score = score_document(query, document);
        if score == 0 {
            continue;
        }
        if best.map_or(true, |(best_index, best_score)| {
            ranked_before(index, candidate_index, score, best_index, best_score)
        }) {
            best = Some((candidate_index, score));
        }
    }
    best
}

fn ranked_before(
    index: &RetrievalIndexSnapshot<'_>,
    candidate_index: usize,
    candidate_score: u32,
    best_index: usize,
    best_score: u32,
) -> bool {
    candidate_score > best_score
        || (candidate_score == best_score
            && resource_order(
                index.documents()[candidate_index].resource_ref_bytes(),
                index.documents()[best_index].resource_ref_bytes(),
            )
            .is_lt())
}

fn score_document(query: &RetrievalQuery, document: &RetrievalDocumentRef) -> u32 {
    let mut score = 0u32;
    for query_byte in query.text_bytes() {
        if !query_byte.is_ascii_alphanumeric() {
            continue;
        }
        if contains_ascii_case_insensitive(document.title_bytes(), *query_byte)
            || contains_ascii_case_insensitive(document.snippet_bytes(), *query_byte)
        {
            score = score.saturating_add(1);
        }
    }
    score
}

fn contains_ascii_case_insensitive(haystack: &[u8], needle: u8) -> bool {
    let needle = needle.to_ascii_lowercase();
    haystack
        .iter()
        .any(|byte| byte.to_ascii_lowercase() == needle)
}

fn resource_order(left: &[u8], right: &[u8]) -> core::cmp::Ordering {
    left.cmp(right)
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

    #[test]
    fn index_snapshot_is_read_only_view_over_caller_documents() {
        let docs = [
            RetrievalDocumentRef::new("index:spec-13.1", "Spec", "retrieval").unwrap(),
            RetrievalDocumentRef::new("blob:assistant-note", "Note", "context").unwrap(),
        ];

        let index = RetrievalIndexSnapshot::new(&docs).unwrap();

        assert_eq!(index.len(), 2);
        assert!(!index.is_empty());
        assert_eq!(index.documents(), &docs);
        assert_eq!(
            index.get(1).unwrap().resource_ref_bytes(),
            b"blob:assistant-note"
        );
        assert!(index.get(2).is_none());
    }

    #[test]
    fn index_snapshot_rejects_empty_duplicate_and_invalid_refs() {
        assert_eq!(
            RetrievalIndexSnapshot::new(&[]).unwrap_err(),
            RetrievalError::IndexEmpty
        );

        let duplicate_docs = [
            RetrievalDocumentRef::new("index:spec-13.1", "Spec", "one").unwrap(),
            RetrievalDocumentRef::new("index:spec-13.1", "Spec copy", "two").unwrap(),
        ];
        assert_eq!(
            RetrievalIndexSnapshot::new(&duplicate_docs).unwrap_err(),
            RetrievalError::DuplicateResourceRef {
                first: 0,
                duplicate: 1,
            }
        );

        let mut invalid = RetrievalDocumentRef::new("index:spec", "Spec", "snippet").unwrap();
        invalid.resource_ref[0] = b'/';
        let invalid_docs = [invalid];
        assert_eq!(
            RetrievalIndexSnapshot::new(&invalid_docs).unwrap_err(),
            RetrievalError::DocumentRefInvalid { index: 0 }
        );
    }

    #[test]
    fn retrieve_top_k_ranks_matches_deterministically() {
        let docs = [
            RetrievalDocumentRef::new("index:zeta", "Boot", "heartbeat").unwrap(),
            RetrievalDocumentRef::new("index:alpha", "Boot", "heartbeat").unwrap(),
            RetrievalDocumentRef::new("index:arena", "Arena", "memory").unwrap(),
        ];
        let index = RetrievalIndexSnapshot::new(&docs).unwrap();
        let query = RetrievalQuery::new("boot").unwrap();
        let mut storage = [empty_result(); 2];
        let mut results = RetrievalResultBuffer::new(&mut storage);

        assert_eq!(retrieve_top_k(&index, &query, 2, &mut results), Ok(2));

        assert_eq!(results.results()[0].resource_ref_bytes(), b"index:alpha");
        assert_eq!(results.results()[1].resource_ref_bytes(), b"index:zeta");
        assert_eq!(results.results()[0].score, results.results()[1].score);
    }

    #[test]
    fn retrieve_top_k_reports_overflow_and_unsupported_query() {
        let docs = [
            RetrievalDocumentRef::new("index:first", "Boot", "heartbeat").unwrap(),
            RetrievalDocumentRef::new("index:second", "Boot", "serial").unwrap(),
        ];
        let index = RetrievalIndexSnapshot::new(&docs).unwrap();
        let query = RetrievalQuery::new("boot").unwrap();
        let mut storage = [empty_result(); 1];
        let mut results = RetrievalResultBuffer::new(&mut storage);

        assert_eq!(
            retrieve_top_k(&index, &query, 2, &mut results),
            Err(RetrievalError::OutputOverflow {
                required: 2,
                available: 1,
            })
        );

        let empty_shape = RetrievalQuery::new("---").unwrap();
        let mut storage = [empty_result(); 1];
        let mut results = RetrievalResultBuffer::new(&mut storage);
        assert_eq!(
            retrieve_top_k(&index, &empty_shape, 1, &mut results),
            Err(RetrievalError::UnsupportedQuery)
        );
        assert_eq!(
            retrieve_top_k(&index, &query, 0, &mut results),
            Err(RetrievalError::UnsupportedQuery)
        );
    }

    #[test]
    fn pack_retrieval_context_preserves_ids_and_boundaries() {
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

        let written = pack_retrieval_context(&index, &results, &mut output).unwrap();

        assert_eq!(
            core::str::from_utf8(&output[..written]).unwrap(),
            "doc=index:spec-13.1\nsnippet=assistant searches local docs\n---\ndoc=blob:assistant-note\nsnippet=context pack\n---\n"
        );
    }

    #[test]
    fn pack_retrieval_context_rejects_overflow_and_mismatched_results() {
        let docs = [RetrievalDocumentRef::new("index:spec", "Spec", "retrieval").unwrap()];
        let index = RetrievalIndexSnapshot::new(&docs).unwrap();
        let results = [RetrievalResult::new(0, 90, "index:spec").unwrap()];
        let mut output = [0u8; 8];

        assert_eq!(
            pack_retrieval_context(&index, &results, &mut output),
            Err(RetrievalError::OutputOverflow {
                required: 14,
                available: 8,
            })
        );

        let mismatched = [RetrievalResult::new(0, 90, "index:other").unwrap()];
        let mut output = [0u8; 80];
        assert_eq!(
            pack_retrieval_context(&index, &mismatched, &mut output),
            Err(RetrievalError::DocumentRefInvalid { index: 0 })
        );
    }
}
