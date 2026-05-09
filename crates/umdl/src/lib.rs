//! UMDL — bare-metal model package. Spec §10.5.
//!
//! Persistent format. Header + tokenizer section + tensor descriptor
//! table + weight blob + checksum section. No pointers, no host
//! paths, no Linux/CUDA assumptions. Loaded into ModelWeightArena;
//! becomes read-only after load on profiles that support paging.

#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

pub const UMDL_MAGIC: [u8; 4] = *b"UMDL";

pub const UMDL_FORMAT_MAJOR: u16 = 1;
pub const UMDL_FORMAT_MINOR: u16 = 0;

/// Header at file offset 0. Spec §10.5.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UmdlHeader {
    pub magic: [u8; 4],
    pub format_major: u16,
    pub format_minor: u16,
    pub header_length: u32,
    pub architecture_id: u32,
    pub quantization_scheme_id: u32,
    pub tensor_count: u32,
    pub tokenizer_section_offset: u64,
    pub tokenizer_section_length: u64,
    pub tensor_section_offset: u64,
    pub tensor_section_length: u64,
    pub weight_blob_offset: u64,
    pub weight_blob_length: u64,
    pub checksum_section_offset: u64,
    pub checksum_section_length: u64,
    pub required_memory_bytes: u64,
    pub required_scratch_bytes: u64,
    pub required_kv_cache_bytes_per_token: u64,
    pub max_context_tokens: u32,
    pub vocabulary_size: u32,
    pub layer_count: u32,
    pub hidden_size: u32,
    pub attention_head_count: u32,
    pub minimum_simd_tier: u32,
    pub model_stable_id: u64,
    pub header_checksum: u64,
}

/// Scalar element type for tensor data.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ScalarType {
    F32 = 0,
    F16 = 1,
    BF16 = 2,
    I8 = 3,
    U8 = 4,
}

/// Quantization registry (spec §10.6). Stable numeric IDs.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum QuantType {
    QNoneF32 = 0,
    QNoneF16 = 1,
    Q4Block32 = 10,
    Q8Block32 = 11,
}

/// Tokenizer registry (spec §10.7).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum TokenizerType {
    ByteFallbackBpe = 1,
    SentencepieceUnigram = 2,
    RawByteToToken = 3,
}

/// Tensor descriptor — spec §10.12.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TensorDesc {
    pub tensor_id: u32,
    pub scalar_type: u8,
    pub quant_type: u8,
    pub rank: u8,
    pub _reserved: u8,
    pub dims: [u32; 4],
    pub byte_offset: u64,
    pub byte_length: u64,
    pub alignment: u32,
    pub flags: u32,
}

/// SIMD tier the runtime advertises and the loader compares against
/// the model's `minimum_simd_tier`. Numerical ordering is meaningful:
/// higher = more capable. Mirrors `cpu::SimdTier` in the kernel.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u32)]
pub enum SimdTier {
    Scalar = 0,
    Sse2 = 1,
    Avx = 2,
    Avx2 = 3,
    Avx512 = 4,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UmdlLoadError {
    BadMagic,
    UnsupportedVersion { major: u16, minor: u16 },
    HeaderTooShort,
    HeaderChecksumMismatch,
    TokenizerSectionChecksumMismatch,
    TensorSectionChecksumMismatch,
    WeightBlobChecksumMismatch,
    TensorOutOfBounds { tensor_id: u32 },
    UnknownQuantizationId(u32),
    UnknownTokenizerType(u32),
    UnknownArchitectureId(u32),
    SimdRequirementUnmet { required: u32, available: u32 },
    ProfileRamBudgetExceeded { required: u64, available: u64 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magic_bytes_are_stable() {
        assert_eq!(UMDL_MAGIC, [b'U', b'M', b'D', b'L']);
    }

    #[test]
    fn simd_tier_ordering() {
        assert!(SimdTier::Scalar < SimdTier::Sse2);
        assert!(SimdTier::Sse2 < SimdTier::Avx2);
        assert!(SimdTier::Avx2 < SimdTier::Avx512);
    }
}
