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

pub mod assistant;
pub mod dispatch;
pub mod kernels;
pub mod quantized;
pub mod retrieval;
pub mod tokenizer;
pub mod toy_transformer;

use umdl::SimdTier;

pub type ProjectI8Kernel = fn(
    input: &[i8],
    weights: &[i8],
    rows: usize,
    cols: usize,
    bias: Option<&[i32]>,
    output: &mut [i32],
) -> Result<usize, kernels::scalar::ScalarKernelError>;

pub type MatVecI8Kernel = fn(
    input: &[i8],
    weights: &[i8],
    rows: usize,
    cols: usize,
    output: &mut [i32],
) -> Result<usize, kernels::scalar::ScalarKernelError>;

pub type F32BinaryKernel = fn(
    lhs: &[f32],
    rhs: &[f32],
    output: &mut [f32],
) -> Result<usize, kernels::scalar::ScalarKernelError>;

pub type F32UnaryKernel =
    fn(input: &[f32], output: &mut [f32]) -> Result<usize, kernels::scalar::ScalarKernelError>;

pub type EmbeddingLookupKernel = fn(
    token_ids: &[u32],
    embedding_table: &[f32],
    embedding_width: usize,
    output: &mut [f32],
) -> Result<usize, kernels::scalar::ScalarKernelError>;

pub type SampleTopKKernel = fn(
    logits: &[i32],
    top_k: usize,
    output: &mut [u32],
) -> Result<usize, kernels::scalar::ScalarKernelError>;

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
    pub project_i8_i8_i32: ProjectI8Kernel,
    pub matvec_q4: MatVecI8Kernel,
    pub matvec_q8: MatVecI8Kernel,
    pub vec_add_f32: F32BinaryKernel,
    pub vec_mul_f32: F32BinaryKernel,
    pub rms_norm_f32: F32UnaryKernel,
    pub layer_norm_f32: F32UnaryKernel,
    pub rope_f32: F32UnaryKernel,
    pub attention_scores: F32BinaryKernel,
    pub softmax_f32: F32UnaryKernel,
    pub embedding_lookup: EmbeddingLookupKernel,
    pub final_proj_q4: MatVecI8Kernel,
    pub sample_top_k: SampleTopKKernel,
    pub active_tier: SimdTier,
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
