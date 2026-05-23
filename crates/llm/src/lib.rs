//! Bare-metal local LLM subsystem. Spec §10.
//!
//! Inference is exposed as graph nodes — never as a hidden thread,
//! hidden queue, or side loop. Tokenization, embedding, transformer
//! blocks, attention, MLP, normalization, logits, sampling,
//! detokenization, KV cache, context packing — all are graph nodes
//! or verified macro-nodes.
//!
//! Backend kernels (scalar / SSE2 / AVX / AVX2 / AVX-512) are
//! reachable only through the loader-built `TensorKernelTable` in
//! `dispatch.rs`. Graph nodes call `kernels.matvec_q4(...)`, never
//! the bare backend symbol. The simd-dispatch-auditor subagent
//! enforces this at review time.

#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

pub mod dispatch;
pub mod tokenizer;
pub mod toy_transformer;

use umdl::SimdTier;

/// Tensor kernel function pointers. The loader builds one of these
/// per session, picking entries appropriate to the active SIMD tier
/// and the model's declared minimum backend (§11.2).
///
/// **This is the only legal entry point from graph nodes to backend
/// kernels.** Graph code calls `kernels.matvec_q4(...)`. Direct
/// reference to `matvec_q4_avx2` from anywhere outside `dispatch.rs`
/// or `kernels/<tier>/...` is a fidelity violation.
#[repr(C)]
pub struct TensorKernelTable {
    pub matvec_q4: fn(/* args TBD */),
    pub matvec_q8: fn(/* args TBD */),
    pub vec_add_f32: fn(/* args TBD */),
    pub vec_mul_f32: fn(/* args TBD */),
    pub rms_norm_f32: fn(/* args TBD */),
    pub layer_norm_f32: fn(/* args TBD */),
    pub rope_f32: fn(/* args TBD */),
    pub attention_scores: fn(/* args TBD */),
    pub softmax_f32: fn(/* args TBD */),
    pub embedding_lookup: fn(/* args TBD */),
    pub final_proj_q4: fn(/* args TBD */),
    pub sample_top_k: fn(/* args TBD */),
    pub active_tier: SimdTier,
}

/// Authority gate output. The LLM's tool-planning output flows here
/// only — never directly into graph mutation. From here:
/// `structured_action_buffer → action schema validator → graph
/// verifier on a temp UMOD patch → operator approval → handle swap`.
/// Spec §10.18.
#[derive(Debug)]
pub struct StructuredActionBuffer {
    /// Pending action records. Not exposed; consumers only read the
    /// validated stream.
    _phantom: core::marker::PhantomData<()>,
}

/// Sampler config (spec §10.15). Explicit, deterministic in
/// deterministic mode (same model + prompt + config + seed →
/// identical token sequence).
#[derive(Copy, Clone, Debug)]
pub struct SamplerConfig {
    pub max_new_tokens: u32,
    pub temperature_q15: i32, // fixed-point ×2^15 to keep deterministic math
    pub top_k: u32,
    pub top_p_q15: i32,
    pub repetition_penalty_q15: i32,
    pub random_seed: u64,
    pub deterministic: bool,
}

/// Inference modes (spec §10.14). Each mode is a graph topology
/// rather than a hidden runtime path.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum InferenceMode {
    OneShot,
    Streaming,
    Embedding,
    ToolPlanning,
}
