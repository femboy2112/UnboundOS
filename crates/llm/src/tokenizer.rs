//! Tokenizer graph-node surface for M7.
//!
//! M7 supports exactly one tiny tokenizer family, `RawByteToToken`. Encode and
//! decode land in later steps; this module owns the support boundary and
//! metadata validation used by those paths.

use umdl::{TokenizerMetadata, TokenizerMetadataError, TokenizerType, M7_SUPPORTED_TOKENIZER};

pub const SUPPORTED_TOKENIZER: TokenizerType = M7_SUPPORTED_TOKENIZER;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TokenizerError {
    Metadata(TokenizerMetadataError),
    OutputOverflow { required: u32, available: u32 },
    InvalidTokenId { token_id: u32 },
    InvalidUtf8,
}

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

/// Encode UTF-8 input bytes into raw-byte token IDs.
///
/// # Errors
///
/// Returns `TokenizerError::Metadata` when the tokenizer metadata is invalid
/// for M7, or `TokenizerError::OutputOverflow` when the caller-provided token
/// buffer cannot hold every input byte.
pub fn encode_raw_bytes(
    metadata: TokenizerMetadata,
    input: &str,
    output: &mut [u32],
) -> Result<usize, TokenizerError> {
    validate_metadata(metadata).map_err(TokenizerError::Metadata)?;
    let bytes = input.as_bytes();
    if output.len() < bytes.len() {
        return Err(TokenizerError::OutputOverflow {
            required: len_to_u32(bytes.len()),
            available: len_to_u32(output.len()),
        });
    }
    for (slot, byte) in output.iter_mut().zip(bytes.iter().copied()) {
        *slot = u32::from(byte);
    }
    Ok(bytes.len())
}

/// Decode raw-byte token IDs into caller-provided UTF-8 output storage.
///
/// # Errors
///
/// Returns `TokenizerError::Metadata` when metadata is invalid,
/// `TokenizerError::OutputOverflow` when output is too small,
/// `TokenizerError::InvalidTokenId` when any token is outside the byte range,
/// or `TokenizerError::InvalidUtf8` when the resulting bytes are not valid
/// UTF-8.
pub fn decode_raw_bytes<'a>(
    metadata: TokenizerMetadata,
    tokens: &[u32],
    output: &'a mut [u8],
) -> Result<&'a str, TokenizerError> {
    validate_metadata(metadata).map_err(TokenizerError::Metadata)?;
    if output.len() < tokens.len() {
        return Err(TokenizerError::OutputOverflow {
            required: len_to_u32(tokens.len()),
            available: len_to_u32(output.len()),
        });
    }
    for (slot, token) in output.iter_mut().zip(tokens.iter().copied()) {
        let byte =
            u8::try_from(token).map_err(|_| TokenizerError::InvalidTokenId { token_id: token })?;
        *slot = byte;
    }
    core::str::from_utf8(&output[..tokens.len()]).map_err(|_| TokenizerError::InvalidUtf8)
}

/// Encode and immediately decode through the raw-byte tokenizer.
///
/// # Errors
///
/// Returns the same `TokenizerError` variants as `encode_raw_bytes` and
/// `decode_raw_bytes`.
pub fn round_trip_raw_bytes<'a>(
    metadata: TokenizerMetadata,
    input: &str,
    token_buffer: &mut [u32],
    byte_buffer: &'a mut [u8],
) -> Result<&'a str, TokenizerError> {
    let token_count = encode_raw_bytes(metadata, input, token_buffer)?;
    decode_raw_bytes(metadata, &token_buffer[..token_count], byte_buffer)
}

fn len_to_u32(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
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

    #[test]
    fn encodes_ascii_bytes_into_stable_token_ids() {
        let mut tokens = [0u32; 8];
        let written =
            encode_raw_bytes(TokenizerMetadata::raw_byte_to_token(), "Hi!", &mut tokens).unwrap();

        assert_eq!(written, 3);
        assert_eq!(&tokens[..written], &[72, 105, 33]);
        assert_eq!(tokens[3], 0);
    }

    #[test]
    fn encodes_utf8_as_bytes_not_scalar_values() {
        let mut tokens = [0u32; 8];
        let written =
            encode_raw_bytes(TokenizerMetadata::raw_byte_to_token(), "µ", &mut tokens).unwrap();

        assert_eq!(written, 2);
        assert_eq!(&tokens[..written], &[0xC2, 0xB5]);
    }

    #[test]
    fn encode_reports_output_overflow() {
        let mut tokens = [0u32; 2];
        let err = encode_raw_bytes(TokenizerMetadata::raw_byte_to_token(), "abc", &mut tokens)
            .unwrap_err();

        assert_eq!(
            err,
            TokenizerError::OutputOverflow {
                required: 3,
                available: 2,
            }
        );
    }

    #[test]
    fn encode_reports_invalid_metadata() {
        let mut metadata = TokenizerMetadata::raw_byte_to_token();
        metadata.vocabulary_size = 255;
        let mut tokens = [0u32; 4];

        assert_eq!(
            encode_raw_bytes(metadata, "abc", &mut tokens).unwrap_err(),
            TokenizerError::Metadata(TokenizerMetadataError::InvalidVocabularySize {
                expected: umdl::RAW_BYTE_TO_TOKEN_VOCAB_SIZE,
                actual: 255,
            })
        );
    }

    #[test]
    fn decodes_token_ids_into_utf8_text() {
        let tokens = [72, 105, 33];
        let mut bytes = [0u8; 8];
        let text =
            decode_raw_bytes(TokenizerMetadata::raw_byte_to_token(), &tokens, &mut bytes).unwrap();

        assert_eq!(text, "Hi!");
    }

    #[test]
    fn decode_reports_output_overflow() {
        let tokens = [72, 105, 33];
        let mut bytes = [0u8; 2];
        let err = decode_raw_bytes(TokenizerMetadata::raw_byte_to_token(), &tokens, &mut bytes)
            .unwrap_err();

        assert_eq!(
            err,
            TokenizerError::OutputOverflow {
                required: 3,
                available: 2,
            }
        );
    }

    #[test]
    fn decode_reports_invalid_token_id() {
        let tokens = [256];
        let mut bytes = [0u8; 1];
        let err = decode_raw_bytes(TokenizerMetadata::raw_byte_to_token(), &tokens, &mut bytes)
            .unwrap_err();

        assert_eq!(err, TokenizerError::InvalidTokenId { token_id: 256 });
    }

    #[test]
    fn decode_rejects_invalid_utf8_bytes() {
        let tokens = [0xFF];
        let mut bytes = [0u8; 1];
        let err = decode_raw_bytes(TokenizerMetadata::raw_byte_to_token(), &tokens, &mut bytes)
            .unwrap_err();

        assert_eq!(err, TokenizerError::InvalidUtf8);
    }

    #[test]
    fn representative_prompts_round_trip() {
        for prompt in ["", "hello", "snark OS", "µ-kernel", "line\nbreak"] {
            let mut tokens = [0u32; 64];
            let mut bytes = [0u8; 64];
            let text = round_trip_raw_bytes(
                TokenizerMetadata::raw_byte_to_token(),
                prompt,
                &mut tokens,
                &mut bytes,
            )
            .unwrap();

            assert_eq!(text, prompt);
        }
    }
}
