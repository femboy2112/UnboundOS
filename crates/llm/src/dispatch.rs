//! Tensor kernel dispatch table builder.
//!
//! **The only legal site in the workspace for references to
//! backend-specific symbols** (`matvec_q4_avx2`, `softmax_avx2`,
//! `rope_sse2`, …). The simd-dispatch-auditor enforces this at
//! review time:
//!
//! ```text
//! rg -n 'matvec_q4_(avx512|avx2|avx|sse2|scalar)' \
//!     --glob '!crates/llm/src/dispatch.rs' \
//!     --glob '!crates/llm/src/kernels/**'
//! ```
//!
//! Hits outside this file and `kernels/` are fidelity violations.

use crate::kernels;
use crate::TensorKernelTable;
use umdl::SimdTier;

/// Build a `TensorKernelTable` for the active CPU tier. The loader calls this
/// once per session after CPUID/XCR0 has resolved the runtime `SimdTier`. Graph
/// nodes then route through the returned table. Non-scalar backend entries are
/// intentionally not exposed until their runtime assertions exist, so every
/// current tier routes through deterministic scalar kernels.
#[must_use]
pub fn build_dispatch_table(active: SimdTier) -> TensorKernelTable {
    TensorKernelTable {
        project_i8_i8_i32: kernels::scalar::project_i8_i8_i32,
        matvec_q4: kernels::scalar::matvec_q4_i8_i32,
        matvec_q8: kernels::scalar::matvec_q8_i8_i32,
        vec_add_f32: kernels::scalar::vec_add_f32,
        vec_mul_f32: kernels::scalar::vec_mul_f32,
        rms_norm_f32: kernels::scalar::rms_norm_f32,
        layer_norm_f32: kernels::scalar::layer_norm_f32,
        rope_f32: kernels::scalar::rope_f32,
        attention_scores: kernels::scalar::attention_scores,
        softmax_f32: kernels::scalar::softmax_f32,
        embedding_lookup: kernels::scalar::embedding_lookup,
        final_proj_q4: kernels::scalar::matvec_q4_i8_i32,
        sample_top_k: kernels::scalar::sample_top_k,
        active_tier: active,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatch_table_routes_quantized_projection_through_scalar_kernel() {
        let table = build_dispatch_table(SimdTier::Avx2);
        let input = [2, -3, 4];
        let weights = [
            1, 2, 3, //
            -4, 5, -6,
        ];
        let bias = [7, -8];
        let mut output = [0i32; 2];

        let len =
            (table.project_i8_i8_i32)(&input, &weights, 2, 3, Some(&bias), &mut output).unwrap();

        assert_eq!(table.active_tier, SimdTier::Avx2);
        assert_eq!(len, 2);
        assert_eq!(output, [15, -55]);
    }

    #[test]
    fn dispatch_table_exercises_typed_scalar_entries() {
        let table = build_dispatch_table(SimdTier::Sse2);
        let input = [2, -3, 4];
        let weights = [
            1, 2, 3, //
            -4, 5, -6,
        ];
        let mut matvec = [0i32; 2];
        let len = (table.matvec_q8)(&input, &weights, 2, 3, &mut matvec).unwrap();
        assert_eq!(len, 2);
        assert_eq!(matvec, [8, -47]);

        let lhs = [1.0, 2.0, 3.0];
        let rhs = [4.0, 5.0, 6.0];
        let mut f32_output = [0.0; 3];
        assert_eq!((table.vec_add_f32)(&lhs, &rhs, &mut f32_output), Ok(3));
        assert_f32_slice_eq(&f32_output, &[5.0, 7.0, 9.0]);

        let logits = [3, 9, 1, 5];
        let mut top = [0u32; 2];
        assert_eq!((table.sample_top_k)(&logits, 2, &mut top), Ok(2));
        assert_eq!(top, [1, 3]);
        assert_eq!(table.active_tier, SimdTier::Sse2);
    }

    fn assert_f32_slice_eq(actual: &[f32], expected: &[f32]) {
        assert_eq!(actual.len(), expected.len());
        for (actual, expected) in actual.iter().zip(expected.iter()) {
            assert!((*actual - *expected).abs() < 0.000_001);
        }
    }
}
