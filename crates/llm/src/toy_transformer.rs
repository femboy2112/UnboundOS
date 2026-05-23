//! Hardcoded toy transformer contract for M8.
//!
//! This module intentionally starts with metadata/config validation only. The
//! generation path lands in the next mission and must use caller-provided
//! buffers rather than hidden allocation or hidden execution.

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
    OutputOverflow { required: u32, available: u32 },
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
}
