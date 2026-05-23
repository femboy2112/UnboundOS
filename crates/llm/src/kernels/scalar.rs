//! Safe scalar quantized kernels for M10.

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ScalarKernelError {
    ShapeMismatch,
    OutputOverflow { required: u32, available: u32 },
    InvalidQuantizedValue,
}

/// Project one quantized input vector through an i8 weight matrix.
///
/// Weights are row-major with shape `rows x cols`. Accumulators and optional
/// bias use i32 so deterministic integer arithmetic is independent of backend
/// vector width.
///
/// # Errors
///
/// Returns `ScalarKernelError` when input/weight/bias shapes do not match or
/// the caller-provided output buffer cannot hold `rows` accumulators.
pub fn project_i8_i8_i32(
    input: &[i8],
    weights: &[i8],
    rows: usize,
    cols: usize,
    bias: Option<&[i32]>,
    output: &mut [i32],
) -> Result<usize, ScalarKernelError> {
    if input.len() != cols || weights.len() != rows.saturating_mul(cols) {
        return Err(ScalarKernelError::ShapeMismatch);
    }
    if let Some(bias) = bias {
        if bias.len() != rows {
            return Err(ScalarKernelError::ShapeMismatch);
        }
    }
    if output.len() < rows {
        return Err(ScalarKernelError::OutputOverflow {
            required: len_to_u32(rows),
            available: len_to_u32(output.len()),
        });
    }

    for row in 0..rows {
        let mut acc = bias.map_or(0, |values| values[row]);
        let row_start = row * cols;
        for col in 0..cols {
            acc += i32::from(input[col]) * i32::from(weights[row_start + col]);
        }
        output[row] = acc;
    }
    Ok(rows)
}

/// Project one vector through an unpacked q8 row-major matrix.
///
/// # Errors
///
/// Returns `ScalarKernelError` when shapes mismatch or `output` is too small.
pub fn matvec_q8_i8_i32(
    input: &[i8],
    weights: &[i8],
    rows: usize,
    cols: usize,
    output: &mut [i32],
) -> Result<usize, ScalarKernelError> {
    project_i8_i8_i32(input, weights, rows, cols, None, output)
}

/// Project one vector through an unpacked signed q4 row-major matrix.
///
/// # Errors
///
/// Returns `ScalarKernelError` when weights are outside `-8..=7`, shapes
/// mismatch, or `output` is too small.
pub fn matvec_q4_i8_i32(
    input: &[i8],
    weights: &[i8],
    rows: usize,
    cols: usize,
    output: &mut [i32],
) -> Result<usize, ScalarKernelError> {
    if weights.iter().any(|value| !(-8..=7).contains(value)) {
        return Err(ScalarKernelError::InvalidQuantizedValue);
    }
    project_i8_i8_i32(input, weights, rows, cols, None, output)
}

/// Add two f32 vectors into caller-owned output.
///
/// # Errors
///
/// Returns `ScalarKernelError` when shapes mismatch or `output` is too small.
pub fn vec_add_f32(
    lhs: &[f32],
    rhs: &[f32],
    output: &mut [f32],
) -> Result<usize, ScalarKernelError> {
    binary_f32(lhs, rhs, output, |left, right| left + right)
}

/// Multiply two f32 vectors into caller-owned output.
///
/// # Errors
///
/// Returns `ScalarKernelError` when shapes mismatch or `output` is too small.
pub fn vec_mul_f32(
    lhs: &[f32],
    rhs: &[f32],
    output: &mut [f32],
) -> Result<usize, ScalarKernelError> {
    binary_f32(lhs, rhs, output, |left, right| left * right)
}

/// Compute elementwise attention scores for one scalar fallback lane.
///
/// # Errors
///
/// Returns `ScalarKernelError` when shapes mismatch or `output` is too small.
pub fn attention_scores(
    query: &[f32],
    key: &[f32],
    output: &mut [f32],
) -> Result<usize, ScalarKernelError> {
    binary_f32(query, key, output, |left, right| left * right)
}

/// Normalize a vector by root-mean-square using deterministic scalar math.
///
/// # Errors
///
/// Returns `ScalarKernelError` when `output` is too small.
pub fn rms_norm_f32(input: &[f32], output: &mut [f32]) -> Result<usize, ScalarKernelError> {
    if output.len() < input.len() {
        return Err(ScalarKernelError::OutputOverflow {
            required: len_to_u32(input.len()),
            available: len_to_u32(output.len()),
        });
    }
    if input.is_empty() {
        return Ok(0);
    }

    let sum_squares = input.iter().map(|value| value * value).sum::<f32>();
    let rms = sqrt_approx(sum_squares / len_to_f32(input.len()) + 0.000_001);
    for (out, value) in output.iter_mut().zip(input.iter().copied()) {
        *out = value / rms;
    }
    Ok(input.len())
}

/// Layer-normalize a vector using deterministic scalar math.
///
/// # Errors
///
/// Returns `ScalarKernelError` when `output` is too small.
pub fn layer_norm_f32(input: &[f32], output: &mut [f32]) -> Result<usize, ScalarKernelError> {
    if output.len() < input.len() {
        return Err(ScalarKernelError::OutputOverflow {
            required: len_to_u32(input.len()),
            available: len_to_u32(output.len()),
        });
    }
    if input.is_empty() {
        return Ok(0);
    }

    let len = len_to_f32(input.len());
    let mean = input.iter().sum::<f32>() / len;
    let variance = input
        .iter()
        .map(|value| {
            let centered = *value - mean;
            centered * centered
        })
        .sum::<f32>()
        / len;
    let denom = sqrt_approx(variance + 0.000_001);
    for (out, value) in output.iter_mut().zip(input.iter().copied()) {
        *out = (value - mean) / denom;
    }
    Ok(input.len())
}

/// Rotate adjacent f32 pairs for the scalar `RoPE` fallback.
///
/// # Errors
///
/// Returns `ScalarKernelError` when `input` has odd length or `output` is too
/// small.
pub fn rope_f32(input: &[f32], output: &mut [f32]) -> Result<usize, ScalarKernelError> {
    if input.len() % 2 != 0 {
        return Err(ScalarKernelError::ShapeMismatch);
    }
    if output.len() < input.len() {
        return Err(ScalarKernelError::OutputOverflow {
            required: len_to_u32(input.len()),
            available: len_to_u32(output.len()),
        });
    }

    for (index, pair) in input.chunks_exact(2).enumerate() {
        let out = index * 2;
        output[out] = -pair[1];
        output[out + 1] = pair[0];
    }
    Ok(input.len())
}

/// Compute a bounded scalar softmax approximation.
///
/// # Errors
///
/// Returns `ScalarKernelError` when `output` is too small.
pub fn softmax_f32(input: &[f32], output: &mut [f32]) -> Result<usize, ScalarKernelError> {
    if output.len() < input.len() {
        return Err(ScalarKernelError::OutputOverflow {
            required: len_to_u32(input.len()),
            available: len_to_u32(output.len()),
        });
    }
    if input.is_empty() {
        return Ok(0);
    }

    let max = input.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut sum = 0.0;
    for (out, value) in output.iter_mut().zip(input.iter().copied()) {
        let exp = exp_approx(value - max);
        *out = exp;
        sum += exp;
    }
    for out in output.iter_mut().take(input.len()) {
        *out /= sum;
    }
    Ok(input.len())
}

/// Copy embedding rows for token ids into caller-owned output.
///
/// # Errors
///
/// Returns `ScalarKernelError` when the table shape is invalid, a token is out
/// of range, or `output` is too small.
pub fn embedding_lookup(
    token_ids: &[u32],
    embedding_table: &[f32],
    embedding_width: usize,
    output: &mut [f32],
) -> Result<usize, ScalarKernelError> {
    if embedding_width == 0 || embedding_table.len() % embedding_width != 0 {
        return Err(ScalarKernelError::ShapeMismatch);
    }
    let required = token_ids.len().saturating_mul(embedding_width);
    if output.len() < required {
        return Err(ScalarKernelError::OutputOverflow {
            required: len_to_u32(required),
            available: len_to_u32(output.len()),
        });
    }

    let rows = embedding_table.len() / embedding_width;
    for (token_index, token_id) in token_ids.iter().copied().enumerate() {
        let row = usize::try_from(token_id).map_err(|_| ScalarKernelError::ShapeMismatch)?;
        if row >= rows {
            return Err(ScalarKernelError::ShapeMismatch);
        }
        let source_start = row * embedding_width;
        let dest_start = token_index * embedding_width;
        output[dest_start..dest_start + embedding_width]
            .copy_from_slice(&embedding_table[source_start..source_start + embedding_width]);
    }
    Ok(required)
}

/// Select deterministic top-k logit indices in descending score order.
///
/// # Errors
///
/// Returns `ScalarKernelError` when `top_k` is invalid or `output` is too
/// small.
pub fn sample_top_k(
    logits: &[i32],
    top_k: usize,
    output: &mut [u32],
) -> Result<usize, ScalarKernelError> {
    if top_k == 0 || top_k > logits.len() {
        return Err(ScalarKernelError::ShapeMismatch);
    }
    if output.len() < top_k {
        return Err(ScalarKernelError::OutputOverflow {
            required: len_to_u32(top_k),
            available: len_to_u32(output.len()),
        });
    }

    let mut selected = 0usize;
    while selected < top_k {
        let mut best_index: Option<usize> = None;
        let mut best_value = i32::MIN;
        for (index, value) in logits.iter().copied().enumerate() {
            if output[..selected]
                .iter()
                .any(|existing| usize::try_from(*existing) == Ok(index))
            {
                continue;
            }
            if best_index.is_none() || value > best_value {
                best_index = Some(index);
                best_value = value;
            }
        }
        let Some(best_index) = best_index else {
            return Err(ScalarKernelError::ShapeMismatch);
        };
        output[selected] = len_to_u32(best_index);
        selected += 1;
    }
    Ok(top_k)
}

fn binary_f32(
    lhs: &[f32],
    rhs: &[f32],
    output: &mut [f32],
    op: impl Fn(f32, f32) -> f32,
) -> Result<usize, ScalarKernelError> {
    if lhs.len() != rhs.len() {
        return Err(ScalarKernelError::ShapeMismatch);
    }
    if output.len() < lhs.len() {
        return Err(ScalarKernelError::OutputOverflow {
            required: len_to_u32(lhs.len()),
            available: len_to_u32(output.len()),
        });
    }
    for ((out, left), right) in output
        .iter_mut()
        .zip(lhs.iter().copied())
        .zip(rhs.iter().copied())
    {
        *out = op(left, right);
    }
    Ok(lhs.len())
}

fn exp_approx(value: f32) -> f32 {
    let clamped = value.clamp(-8.0, 8.0);
    let x2 = clamped * clamped;
    let x3 = x2 * clamped;
    let x4 = x2 * x2;
    (1.0 + clamped + x2 * 0.5 + x3 / 6.0 + x4 / 24.0).max(0.000_001)
}

fn sqrt_approx(value: f32) -> f32 {
    if value <= 0.0 {
        return 0.0;
    }
    let mut estimate = if value >= 1.0 { value } else { 1.0 };
    for _ in 0..8 {
        estimate = 0.5 * (estimate + value / estimate);
    }
    estimate
}

fn len_to_u32(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

fn len_to_f32(len: usize) -> f32 {
    f32::from(u16::try_from(len).unwrap_or(u16::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_i8_projection_is_deterministic() {
        let input = [2, -3, 4];
        let weights = [
            1, 2, 3, //
            -4, 5, -6,
        ];
        let bias = [7, -8];
        let mut first = [0i32; 2];
        let mut second = [0i32; 2];

        let first_len = project_i8_i8_i32(&input, &weights, 2, 3, Some(&bias), &mut first).unwrap();
        let second_len =
            project_i8_i8_i32(&input, &weights, 2, 3, Some(&bias), &mut second).unwrap();

        assert_eq!(first_len, 2);
        assert_eq!(second_len, 2);
        assert_eq!(first, [15, -55]);
        assert_eq!(first, second);
    }

    #[test]
    fn scalar_i8_projection_uses_caller_output_buffer() {
        let input = [1, 2];
        let weights = [1, 1, 2, 2];
        let mut output = [0i32; 1];

        assert_eq!(
            project_i8_i8_i32(&input, &weights, 2, 2, None, &mut output).unwrap_err(),
            ScalarKernelError::OutputOverflow {
                required: 2,
                available: 1,
            }
        );
    }

    #[test]
    fn scalar_i8_projection_rejects_bad_shapes() {
        let input = [1, 2];
        let weights = [1, 2, 3];
        let mut output = [0i32; 2];

        assert_eq!(
            project_i8_i8_i32(&input, &weights, 2, 2, None, &mut output).unwrap_err(),
            ScalarKernelError::ShapeMismatch
        );
    }

    #[test]
    fn scalar_matvec_q4_rejects_out_of_range_weights() {
        let input = [1, 2];
        let weights = [1, 9];
        let mut output = [0i32; 1];

        assert_eq!(
            matvec_q4_i8_i32(&input, &weights, 1, 2, &mut output).unwrap_err(),
            ScalarKernelError::InvalidQuantizedValue
        );
    }

    #[test]
    fn scalar_f32_kernels_use_caller_buffers() {
        let lhs = [1.0, 2.0, 3.0];
        let rhs = [4.0, 5.0, 6.0];
        let mut output = [0.0; 3];

        assert_eq!(vec_add_f32(&lhs, &rhs, &mut output), Ok(3));
        assert_f32_slice_eq(&output, &[5.0, 7.0, 9.0]);
        assert_eq!(vec_mul_f32(&lhs, &rhs, &mut output), Ok(3));
        assert_f32_slice_eq(&output, &[4.0, 10.0, 18.0]);
        assert_eq!(
            vec_add_f32(&lhs, &rhs[..2], &mut output).unwrap_err(),
            ScalarKernelError::ShapeMismatch
        );
    }

    #[test]
    fn scalar_embedding_and_top_k_are_deterministic() {
        let token_ids = [2, 0];
        let table = [1.0, 1.5, 2.0, 2.5, 3.0, 3.5];
        let mut embeddings = [0.0; 4];
        assert_eq!(
            embedding_lookup(&token_ids, &table, 2, &mut embeddings),
            Ok(4)
        );
        assert_f32_slice_eq(&embeddings, &[3.0, 3.5, 1.0, 1.5]);

        let logits = [3, 9, 1, 5];
        let mut top = [0u32; 3];
        assert_eq!(sample_top_k(&logits, 3, &mut top), Ok(3));
        assert_eq!(top, [1, 3, 0]);
    }

    #[test]
    fn scalar_norm_and_rope_are_bounded() {
        let input = [1.0, 2.0, 3.0, 4.0];
        let mut output = [0.0; 4];
        assert_eq!(rms_norm_f32(&input, &mut output), Ok(4));
        assert!(output[0] > 0.0);
        assert_eq!(rope_f32(&input, &mut output), Ok(4));
        assert_f32_slice_eq(&output, &[-2.0, 1.0, -4.0, 3.0]);
        assert_eq!(softmax_f32(&input, &mut output), Ok(4));
        let sum = output.iter().sum::<f32>();
        assert!((sum - 1.0).abs() < 0.000_01);
    }

    fn assert_f32_slice_eq(actual: &[f32], expected: &[f32]) {
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(expected.iter()) {
            assert!((*actual - *expected).abs() < 0.000_001);
        }
    }
}
