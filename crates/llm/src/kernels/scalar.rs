//! Safe scalar quantized kernels for M10.

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ScalarKernelError {
    ShapeMismatch,
    OutputOverflow { required: u32, available: u32 },
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

fn len_to_u32(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
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
}
