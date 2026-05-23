//! Tokenizer graph-node surface for M7.
//!
//! M7 supports exactly one tiny tokenizer family, `RawByteToToken`. Encode and
//! decode land in later steps; this module owns the support boundary and
//! metadata validation used by those paths.

use umdl::{TokenizerMetadata, TokenizerMetadataError, TokenizerType, M7_SUPPORTED_TOKENIZER};

pub const SUPPORTED_TOKENIZER: TokenizerType = M7_SUPPORTED_TOKENIZER;

/// Validate tokenizer metadata against the M7 support boundary.
///
/// # Errors
///
/// Returns `TokenizerMetadataError` when metadata requests an unsupported
/// tokenizer family or an invalid raw-byte metadata shape.
pub fn validate_metadata(metadata: TokenizerMetadata) -> Result<(), TokenizerMetadataError> {
    metadata.validate_m7()
}

#[must_use]
pub fn is_supported(kind: TokenizerType) -> bool {
    kind == SUPPORTED_TOKENIZER
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_raw_byte_tokenizer_is_supported_in_m7() {
        assert!(is_supported(TokenizerType::RawByteToToken));
        assert!(!is_supported(TokenizerType::ByteFallbackBpe));
        assert!(!is_supported(TokenizerType::SentencepieceUnigram));
    }

    #[test]
    fn validates_raw_byte_metadata() {
        validate_metadata(TokenizerMetadata::raw_byte_to_token())
            .expect("raw-byte metadata should validate");
    }

    #[test]
    fn rejects_bpe_metadata_for_now() {
        let mut metadata = TokenizerMetadata::raw_byte_to_token();
        metadata.tokenizer_type = TokenizerType::ByteFallbackBpe as u32;

        assert_eq!(
            validate_metadata(metadata).unwrap_err(),
            TokenizerMetadataError::UnsupportedTokenizerType {
                requested: TokenizerType::ByteFallbackBpe as u32,
                supported: TokenizerType::RawByteToToken as u32,
            }
        );
    }
}
