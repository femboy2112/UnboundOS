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

/// Build a `TensorKernelTable` for the active CPU tier. The loader
/// calls this once per session after CPUID/XCR0 has resolved the
/// runtime `SimdTier`. Graph nodes then route through the returned
/// table.
///
/// In debug builds, callers should additionally assert that the
/// active CPU feature flags match the selected backend before the
/// first kernel call; that assertion lives in
/// `kernels/runtime_assert.rs` (TBD).
#[must_use]
pub fn build_dispatch_table(active: SimdTier) -> TensorKernelTable {
    // Real implementation: match on `active` and return a table
    // whose function pointers reference the matching backend
    // (`kernels::scalar::matvec_q4`, `kernels::sse2::matvec_q4`,
    // `kernels::avx2::matvec_q4`, …).
    //
    // Stub: zero-sized placeholder. Will be replaced when the
    // kernel implementations land.
    fn nop() {}
    TensorKernelTable {
        project_i8_i8_i32: kernels::scalar::project_i8_i8_i32,
        matvec_q4: nop,
        matvec_q8: nop,
        vec_add_f32: nop,
        vec_mul_f32: nop,
        rms_norm_f32: nop,
        layer_norm_f32: nop,
        rope_f32: nop,
        attention_scores: nop,
        softmax_f32: nop,
        embedding_lookup: nop,
        final_proj_q4: nop,
        sample_top_k: nop,
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
}
