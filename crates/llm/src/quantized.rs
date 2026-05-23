//! Deterministic quantized inference step for M10.

use crate::kernels::scalar::ScalarKernelError;
use crate::TensorKernelTable;
use umdl::LoadedUmdlModel;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct QuantizedStepConfig {
    pub candidate_token_base: u32,
    pub candidate_count: u32,
}

pub struct QuantizedStepBuffers<'a> {
    pub projection_input: &'a [i8],
    pub projection_weights: &'a [i8],
    pub projection_bias: Option<&'a [i32]>,
    pub logits: &'a mut [i32],
    pub output_tokens: &'a mut [u32],
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum QuantizedInferenceError {
    InvalidConfig,
    LogitOverflow { required: u32, available: u32 },
    OutputOverflow { required: u32, available: u32 },
    TokenOutOfVocabulary { token_id: u32, vocabulary_size: u32 },
    Kernel(ScalarKernelError),
}

/// Produce one deterministic next token through the selected kernel table.
///
/// The model view must come from UMDL validation. The function writes logits
/// and the selected token into caller-provided buffers and has no graph mutation
/// authority.
///
/// # Errors
///
/// Returns `QuantizedInferenceError` when config/buffers are invalid, a kernel
/// call fails, or the selected candidate is outside the model vocabulary.
pub fn next_token_step(
    model: LoadedUmdlModel,
    kernels: &TensorKernelTable,
    config: QuantizedStepConfig,
    buffers: &mut QuantizedStepBuffers<'_>,
) -> Result<u32, QuantizedInferenceError> {
    if config.candidate_count == 0 || buffers.projection_input.is_empty() || model.tensor_count == 0
    {
        return Err(QuantizedInferenceError::InvalidConfig);
    }
    let rows = config.candidate_count as usize;
    if buffers.logits.len() < rows {
        return Err(QuantizedInferenceError::LogitOverflow {
            required: config.candidate_count,
            available: len_to_u32(buffers.logits.len()),
        });
    }
    if buffers.output_tokens.is_empty() {
        return Err(QuantizedInferenceError::OutputOverflow {
            required: 1,
            available: 0,
        });
    }

    let cols = buffers.projection_input.len();
    (kernels.project_i8_i8_i32)(
        buffers.projection_input,
        buffers.projection_weights,
        rows,
        cols,
        buffers.projection_bias,
        buffers.logits,
    )
    .map_err(QuantizedInferenceError::Kernel)?;

    let mut best_index = 0usize;
    let mut best_logit = buffers.logits[0];
    for (index, logit) in buffers.logits[..rows].iter().copied().enumerate().skip(1) {
        if logit > best_logit {
            best_index = index;
            best_logit = logit;
        }
    }
    let token_id = config.candidate_token_base + len_to_u32(best_index);
    if token_id >= model.header.vocabulary_size {
        return Err(QuantizedInferenceError::TokenOutOfVocabulary {
            token_id,
            vocabulary_size: model.header.vocabulary_size,
        });
    }
    buffers.output_tokens[0] = token_id;
    Ok(token_id)
}

fn len_to_u32(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatch::build_dispatch_table;
    use umdl::{
        LoadedUmdlModel, SimdTier, TokenizerMetadata, UmdlArenaReservations, UmdlHeader,
        UmdlSectionRange, UmdlSectionRanges, M9_SUPPORTED_ARCHITECTURE_ID, UMDL_FORMAT_MAJOR,
        UMDL_FORMAT_MINOR, UMDL_HEADER_LENGTH,
    };

    fn model_view() -> LoadedUmdlModel {
        let header = UmdlHeader {
            magic: *b"UMDL",
            format_major: UMDL_FORMAT_MAJOR,
            format_minor: UMDL_FORMAT_MINOR,
            header_length: UMDL_HEADER_LENGTH,
            architecture_id: M9_SUPPORTED_ARCHITECTURE_ID,
            quantization_scheme_id: 0,
            tensor_count: 1,
            tokenizer_section_offset: 160,
            tokenizer_section_length: 72,
            tensor_section_offset: 240,
            tensor_section_length: 48,
            weight_blob_offset: 320,
            weight_blob_length: 16,
            checksum_section_offset: 400,
            checksum_section_length: 24,
            required_memory_bytes: 16,
            required_scratch_bytes: 8,
            required_kv_cache_bytes_per_token: 2,
            max_context_tokens: 32,
            vocabulary_size: 256,
            layer_count: 1,
            hidden_size: 8,
            attention_head_count: 1,
            minimum_simd_tier: SimdTier::Scalar as u32,
            model_stable_id: 0x0000_0000_000a_0001,
            header_checksum: 0,
        };
        LoadedUmdlModel {
            header,
            tokenizer: TokenizerMetadata::raw_byte_to_token(),
            tensor_count: 1,
            ranges: UmdlSectionRanges {
                tokenizer: UmdlSectionRange {
                    offset: 160,
                    length: 72,
                },
                tensor: UmdlSectionRange {
                    offset: 240,
                    length: 48,
                },
                weight_blob: UmdlSectionRange {
                    offset: 320,
                    length: 16,
                },
                checksum: UmdlSectionRange {
                    offset: 400,
                    length: 24,
                },
            },
            reservations: UmdlArenaReservations {
                model_weight_bytes: 16,
                scratch_bytes: 8,
                kv_cache_bytes_per_token: 2,
                max_context_tokens: 32,
            },
            active_simd_tier: SimdTier::Scalar,
        }
    }

    #[test]
    fn quantized_next_token_step_is_deterministic() {
        let kernels = build_dispatch_table(SimdTier::Scalar);
        let input = [2, -3, 4];
        let weights = [
            1, 2, 3, //
            -4, 5, -6, //
            3, -2, 1,
        ];
        let bias = [7, -8, 3];
        let mut first_logits = [0i32; 3];
        let mut second_logits = [0i32; 3];
        let mut first_token = [0u32; 1];
        let mut second_token = [0u32; 1];
        let config = QuantizedStepConfig {
            candidate_token_base: 65,
            candidate_count: 3,
        };

        let first = next_token_step(
            model_view(),
            &kernels,
            config,
            &mut QuantizedStepBuffers {
                projection_input: &input,
                projection_weights: &weights,
                projection_bias: Some(&bias),
                logits: &mut first_logits,
                output_tokens: &mut first_token,
            },
        )
        .unwrap();
        let second = next_token_step(
            model_view(),
            &kernels,
            config,
            &mut QuantizedStepBuffers {
                projection_input: &input,
                projection_weights: &weights,
                projection_bias: Some(&bias),
                logits: &mut second_logits,
                output_tokens: &mut second_token,
            },
        )
        .unwrap();

        assert_eq!(first_logits, [15, -55, 19]);
        assert_eq!(first, 67);
        assert_eq!(first_token, [67]);
        assert_eq!(first, second);
        assert_eq!(first_logits, second_logits);
        assert_eq!(first_token, second_token);
    }

    #[test]
    fn quantized_next_token_step_reports_buffer_and_vocab_errors() {
        let kernels = build_dispatch_table(SimdTier::Scalar);
        let input = [1, 2];
        let weights = [1, 1, 2, 2];
        let mut logits = [0i32; 1];
        let mut token = [0u32; 1];

        assert_eq!(
            next_token_step(
                model_view(),
                &kernels,
                QuantizedStepConfig {
                    candidate_token_base: 0,
                    candidate_count: 2,
                },
                &mut QuantizedStepBuffers {
                    projection_input: &input,
                    projection_weights: &weights,
                    projection_bias: None,
                    logits: &mut logits,
                    output_tokens: &mut token,
                },
            )
            .unwrap_err(),
            QuantizedInferenceError::LogitOverflow {
                required: 2,
                available: 1,
            }
        );

        let mut logits = [0i32; 2];
        let mut empty_output = [0u32; 0];
        assert_eq!(
            next_token_step(
                model_view(),
                &kernels,
                QuantizedStepConfig {
                    candidate_token_base: 0,
                    candidate_count: 2,
                },
                &mut QuantizedStepBuffers {
                    projection_input: &input,
                    projection_weights: &weights,
                    projection_bias: None,
                    logits: &mut logits,
                    output_tokens: &mut empty_output,
                },
            )
            .unwrap_err(),
            QuantizedInferenceError::OutputOverflow {
                required: 1,
                available: 0,
            }
        );

        let mut token = [0u32; 1];
        assert_eq!(
            next_token_step(
                model_view(),
                &kernels,
                QuantizedStepConfig {
                    candidate_token_base: 255,
                    candidate_count: 2,
                },
                &mut QuantizedStepBuffers {
                    projection_input: &input,
                    projection_weights: &weights,
                    projection_bias: None,
                    logits: &mut logits,
                    output_tokens: &mut token,
                },
            )
            .unwrap_err(),
            QuantizedInferenceError::TokenOutOfVocabulary {
                token_id: 256,
                vocabulary_size: 256,
            }
        );
    }
}
