//! Hardcoded toy transformer contract for M8.
//!
//! Generation uses caller-provided buffers rather than hidden allocation or
//! hidden execution. The arithmetic is deliberately scalar and deterministic;
//! SIMD kernels remain behind dispatch and are not touched by M8.

use crate::tokenizer::{self, TokenizerError};
use umdl::TokenizerMetadata;
use umdl::TokenizerType;

pub const M8_SUPPORTED_ARCHITECTURE_ID: u32 = 1;
pub const M8_TOY_MODEL_STABLE_ID: u64 = 0x0000_0000_0008_0001;
pub const M8_TOY_VOCAB_SIZE: u32 = 256;
pub const M8_TOY_CONTEXT_TOKENS: u32 = 32;
pub const M8_TOY_HIDDEN_SIZE: u32 = 8;
pub const M8_TOY_LAYER_COUNT: u32 = 1;
pub const M8_TOY_ATTENTION_HEADS: u32 = 1;
pub const M8_TOY_MAX_NEW_TOKENS: u32 = 16;

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ToyModelMetadata {
    pub architecture_id: u32,
    pub model_stable_id: u64,
    pub tokenizer_type: u32,
    pub vocabulary_size: u32,
    pub max_context_tokens: u32,
    pub hidden_size: u32,
    pub layer_count: u32,
    pub attention_head_count: u32,
}

impl ToyModelMetadata {
    #[must_use]
    pub const fn m8_toy() -> Self {
        Self {
            architecture_id: M8_SUPPORTED_ARCHITECTURE_ID,
            model_stable_id: M8_TOY_MODEL_STABLE_ID,
            tokenizer_type: TokenizerType::RawByteToToken as u32,
            vocabulary_size: M8_TOY_VOCAB_SIZE,
            max_context_tokens: M8_TOY_CONTEXT_TOKENS,
            hidden_size: M8_TOY_HIDDEN_SIZE,
            layer_count: M8_TOY_LAYER_COUNT,
            attention_head_count: M8_TOY_ATTENTION_HEADS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ToyGenerationConfig {
    pub max_new_tokens: u32,
    pub seed: u64,
    pub deterministic: u32,
}

impl ToyGenerationConfig {
    #[must_use]
    pub const fn deterministic(max_new_tokens: u32, seed: u64) -> Self {
        Self {
            max_new_tokens,
            seed,
            deterministic: 1,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ToyTransformerError {
    UnsupportedArchitecture { requested: u32, supported: u32 },
    UnsupportedTokenizer { requested: u32, supported: u32 },
    InvalidVocabularySize { expected: u32, actual: u32 },
    InvalidContextLength { max_context_tokens: u32 },
    InvalidHiddenSize { hidden_size: u32 },
    InvalidLayerCount { layer_count: u32 },
    InvalidAttentionHeads { attention_head_count: u32 },
    UnsupportedConfig,
    PromptTooLong { provided: u32, max: u32 },
    OutputOverflow { required: u32, available: u32 },
    Tokenizer(TokenizerError),
}

/// Validate the single toy model shape supported by M8.
///
/// # Errors
///
/// Returns `ToyTransformerError` when metadata does not match the hardcoded M8
/// toy architecture contract.
pub fn validate_model(metadata: ToyModelMetadata) -> Result<(), ToyTransformerError> {
    if metadata.architecture_id != M8_SUPPORTED_ARCHITECTURE_ID {
        return Err(ToyTransformerError::UnsupportedArchitecture {
            requested: metadata.architecture_id,
            supported: M8_SUPPORTED_ARCHITECTURE_ID,
        });
    }
    if metadata.tokenizer_type != TokenizerType::RawByteToToken as u32 {
        return Err(ToyTransformerError::UnsupportedTokenizer {
            requested: metadata.tokenizer_type,
            supported: TokenizerType::RawByteToToken as u32,
        });
    }
    if metadata.vocabulary_size != M8_TOY_VOCAB_SIZE {
        return Err(ToyTransformerError::InvalidVocabularySize {
            expected: M8_TOY_VOCAB_SIZE,
            actual: metadata.vocabulary_size,
        });
    }
    if metadata.max_context_tokens == 0 || metadata.max_context_tokens > M8_TOY_CONTEXT_TOKENS {
        return Err(ToyTransformerError::InvalidContextLength {
            max_context_tokens: metadata.max_context_tokens,
        });
    }
    if metadata.hidden_size != M8_TOY_HIDDEN_SIZE {
        return Err(ToyTransformerError::InvalidHiddenSize {
            hidden_size: metadata.hidden_size,
        });
    }
    if metadata.layer_count != M8_TOY_LAYER_COUNT {
        return Err(ToyTransformerError::InvalidLayerCount {
            layer_count: metadata.layer_count,
        });
    }
    if metadata.attention_head_count != M8_TOY_ATTENTION_HEADS {
        return Err(ToyTransformerError::InvalidAttentionHeads {
            attention_head_count: metadata.attention_head_count,
        });
    }
    Ok(())
}

/// Validate deterministic generation config supported by M8.
///
/// # Errors
///
/// Returns `ToyTransformerError::UnsupportedConfig` when config requests hidden
/// nondeterministic behavior or an unsupported generation length.
pub fn validate_config(config: ToyGenerationConfig) -> Result<(), ToyTransformerError> {
    if config.deterministic != 1
        || config.max_new_tokens == 0
        || config.max_new_tokens > M8_TOY_MAX_NEW_TOKENS
    {
        return Err(ToyTransformerError::UnsupportedConfig);
    }
    Ok(())
}

/// Generate deterministic raw-byte token IDs from the hardcoded M8 toy model.
///
/// # Errors
///
/// Returns `ToyTransformerError` when model/config validation fails, the prompt
/// exceeds the toy context length, or the caller-provided output buffer cannot
/// hold `config.max_new_tokens`.
pub fn generate_tokens(
    metadata: ToyModelMetadata,
    config: ToyGenerationConfig,
    prompt_tokens: &[u32],
    output: &mut [u32],
) -> Result<usize, ToyTransformerError> {
    validate_model(metadata)?;
    validate_config(config)?;
    if prompt_tokens.len() > metadata.max_context_tokens as usize {
        return Err(ToyTransformerError::PromptTooLong {
            provided: len_to_u32(prompt_tokens.len()),
            max: metadata.max_context_tokens,
        });
    }
    if output.len() < config.max_new_tokens as usize {
        return Err(ToyTransformerError::OutputOverflow {
            required: config.max_new_tokens,
            available: len_to_u32(output.len()),
        });
    }

    let mut state = initial_state(metadata, config, prompt_tokens);
    let count = config.max_new_tokens as usize;
    for slot in &mut output[..count] {
        state = step_state(state);
        *slot = 32 + ((state >> 24) % 95) as u32;
    }
    Ok(count)
}

/// Run the M8 prompt-to-text toy path using explicit caller buffers.
///
/// # Errors
///
/// Returns `ToyTransformerError` when tokenizer metadata is invalid, prompt
/// tokenization or output decoding fails, toy model/config validation fails,
/// or any caller-provided buffer is too small.
pub fn generate_text<'a>(
    metadata: ToyModelMetadata,
    config: ToyGenerationConfig,
    tokenizer_metadata: TokenizerMetadata,
    prompt: &str,
    prompt_tokens: &mut [u32],
    generated_tokens: &mut [u32],
    output_bytes: &'a mut [u8],
) -> Result<&'a str, ToyTransformerError> {
    let prompt_len = tokenizer::encode_raw_bytes(tokenizer_metadata, prompt, prompt_tokens)
        .map_err(ToyTransformerError::Tokenizer)?;
    let generated_len = generate_tokens(
        metadata,
        config,
        &prompt_tokens[..prompt_len],
        generated_tokens,
    )?;
    tokenizer::decode_raw_bytes(
        tokenizer_metadata,
        &generated_tokens[..generated_len],
        output_bytes,
    )
    .map_err(ToyTransformerError::Tokenizer)
}

fn initial_state(
    metadata: ToyModelMetadata,
    config: ToyGenerationConfig,
    prompt_tokens: &[u32],
) -> u64 {
    let mut state =
        metadata.model_stable_id ^ config.seed ^ u64::from(len_to_u32(prompt_tokens.len()));
    for (idx, token) in prompt_tokens.iter().copied().enumerate() {
        state ^= u64::from(token & 0xFF) << ((idx % 8) * 8);
        state = step_state(state);
    }
    state
}

fn step_state(state: u64) -> u64 {
    state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407)
}

fn len_to_u32(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn toy_model_metadata_layout_is_fixed_width() {
        assert_eq!(size_of::<ToyModelMetadata>(), 40);
        assert_eq!(size_of::<ToyGenerationConfig>(), 24);
    }

    #[test]
    fn m8_exposes_exactly_one_supported_architecture() {
        let metadata = ToyModelMetadata::m8_toy();

        assert_eq!(metadata.architecture_id, M8_SUPPORTED_ARCHITECTURE_ID);
        assert_eq!(metadata.model_stable_id, M8_TOY_MODEL_STABLE_ID);
        validate_model(metadata).expect("M8 toy metadata");
    }

    #[test]
    fn rejects_unsupported_architecture_and_tokenizer() {
        let mut metadata = ToyModelMetadata::m8_toy();
        metadata.architecture_id = 99;
        assert_eq!(
            validate_model(metadata).unwrap_err(),
            ToyTransformerError::UnsupportedArchitecture {
                requested: 99,
                supported: M8_SUPPORTED_ARCHITECTURE_ID,
            }
        );

        let mut metadata = ToyModelMetadata::m8_toy();
        metadata.tokenizer_type = TokenizerType::ByteFallbackBpe as u32;
        assert_eq!(
            validate_model(metadata).unwrap_err(),
            ToyTransformerError::UnsupportedTokenizer {
                requested: TokenizerType::ByteFallbackBpe as u32,
                supported: TokenizerType::RawByteToToken as u32,
            }
        );
    }

    #[test]
    fn rejects_invalid_model_shape() {
        let mut metadata = ToyModelMetadata::m8_toy();
        metadata.vocabulary_size = 255;
        assert_eq!(
            validate_model(metadata).unwrap_err(),
            ToyTransformerError::InvalidVocabularySize {
                expected: M8_TOY_VOCAB_SIZE,
                actual: 255,
            }
        );

        let mut metadata = ToyModelMetadata::m8_toy();
        metadata.hidden_size = 4;
        assert_eq!(
            validate_model(metadata).unwrap_err(),
            ToyTransformerError::InvalidHiddenSize { hidden_size: 4 }
        );
    }

    #[test]
    fn validates_only_deterministic_config() {
        validate_config(ToyGenerationConfig::deterministic(4, 7)).expect("config");

        assert_eq!(
            validate_config(ToyGenerationConfig::deterministic(0, 7)).unwrap_err(),
            ToyTransformerError::UnsupportedConfig
        );

        let mut config = ToyGenerationConfig::deterministic(4, 7);
        config.deterministic = 0;
        assert_eq!(
            validate_config(config).unwrap_err(),
            ToyTransformerError::UnsupportedConfig
        );
    }

    #[test]
    fn generation_is_deterministic_for_same_prompt_seed_config_and_model() {
        let metadata = ToyModelMetadata::m8_toy();
        let config = ToyGenerationConfig::deterministic(6, 123);
        let prompt = [u32::from(b'O'), u32::from(b'S')];
        let mut first = [0u32; 8];
        let mut second = [0u32; 8];

        let first_len = generate_tokens(metadata, config, &prompt, &mut first).unwrap();
        let second_len = generate_tokens(metadata, config, &prompt, &mut second).unwrap();

        assert_eq!(first_len, 6);
        assert_eq!(second_len, 6);
        assert_eq!(&first[..first_len], &second[..second_len]);
        assert_eq!(&first[..first_len], &[108, 32, 85, 110, 107, 65]);
    }

    #[test]
    fn generation_changes_with_seed_or_prompt() {
        let metadata = ToyModelMetadata::m8_toy();
        let prompt = [u32::from(b'O'), u32::from(b'S')];
        let mut seed_a = [0u32; 4];
        let mut seed_b = [0u32; 4];
        let mut prompt_b = [0u32; 4];

        generate_tokens(
            metadata,
            ToyGenerationConfig::deterministic(4, 1),
            &prompt,
            &mut seed_a,
        )
        .unwrap();
        generate_tokens(
            metadata,
            ToyGenerationConfig::deterministic(4, 2),
            &prompt,
            &mut seed_b,
        )
        .unwrap();
        generate_tokens(
            metadata,
            ToyGenerationConfig::deterministic(4, 1),
            &[u32::from(b'X')],
            &mut prompt_b,
        )
        .unwrap();

        assert_ne!(seed_a, seed_b);
        assert_ne!(seed_a, prompt_b);
    }

    #[test]
    fn generation_reports_prompt_and_output_bounds() {
        let metadata = ToyModelMetadata::m8_toy();
        let config = ToyGenerationConfig::deterministic(4, 1);
        let long_prompt = [0u32; M8_TOY_CONTEXT_TOKENS as usize + 1];
        let mut output = [0u32; 4];

        assert_eq!(
            generate_tokens(metadata, config, &long_prompt, &mut output).unwrap_err(),
            ToyTransformerError::PromptTooLong {
                provided: M8_TOY_CONTEXT_TOKENS + 1,
                max: M8_TOY_CONTEXT_TOKENS,
            }
        );

        let mut short_output = [0u32; 3];
        assert_eq!(
            generate_tokens(metadata, config, &[], &mut short_output).unwrap_err(),
            ToyTransformerError::OutputOverflow {
                required: 4,
                available: 3,
            }
        );
    }

    #[test]
    fn prompt_to_text_generation_is_deterministic() {
        let metadata = ToyModelMetadata::m8_toy();
        let config = ToyGenerationConfig::deterministic(6, 123);
        let tokenizer_metadata = TokenizerMetadata::raw_byte_to_token();
        let mut prompt_tokens = [0u32; 32];
        let mut generated_tokens = [0u32; 8];
        let mut output_bytes = [0u8; 8];
        let text = generate_text(
            metadata,
            config,
            tokenizer_metadata,
            "OS",
            &mut prompt_tokens,
            &mut generated_tokens,
            &mut output_bytes,
        )
        .unwrap();

        assert_eq!(text, "l UnkA");
    }

    #[test]
    fn prompt_to_text_uses_caller_provided_buffers() {
        let metadata = ToyModelMetadata::m8_toy();
        let config = ToyGenerationConfig::deterministic(4, 1);
        let tokenizer_metadata = TokenizerMetadata::raw_byte_to_token();
        let mut short_prompt_tokens = [0u32; 1];
        let mut generated_tokens = [0u32; 4];
        let mut output_bytes = [0u8; 4];

        assert_eq!(
            generate_text(
                metadata,
                config,
                tokenizer_metadata,
                "too long",
                &mut short_prompt_tokens,
                &mut generated_tokens,
                &mut output_bytes,
            )
            .unwrap_err(),
            ToyTransformerError::Tokenizer(TokenizerError::OutputOverflow {
                required: 8,
                available: 1,
            })
        );

        let mut prompt_tokens = [0u32; 8];
        let mut short_generated_tokens = [0u32; 3];
        assert_eq!(
            generate_text(
                metadata,
                config,
                tokenizer_metadata,
                "ok",
                &mut prompt_tokens,
                &mut short_generated_tokens,
                &mut output_bytes,
            )
            .unwrap_err(),
            ToyTransformerError::OutputOverflow {
                required: 4,
                available: 3,
            }
        );
    }
}
