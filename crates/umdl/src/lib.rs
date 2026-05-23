//! UMDL — bare-metal model package. Spec §10.5.
//!
//! Persistent format. Header + tokenizer section + tensor descriptor
//! table + weight blob + checksum section. No pointers, no host
//! paths, no Linux/CUDA assumptions. Loaded into `ModelWeightArena`;
//! becomes read-only after load on profiles that support paging.

#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

pub const UMDL_MAGIC: [u8; 4] = *b"UMDL";

pub const UMDL_FORMAT_MAJOR: u16 = 1;
pub const UMDL_FORMAT_MINOR: u16 = 0;
pub const UMDL_HEADER_LENGTH: u32 = 152;

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

impl UmdlHeader {
    /// Parse a fixed-width UMDL header from caller-provided bytes.
    ///
    /// # Errors
    ///
    /// Returns `UmdlLoadError` when the input is shorter than the fixed header
    /// shape, the magic bytes are wrong, the format version is unsupported, or
    /// the declared header length cannot cover the fixed-width header.
    pub fn parse(bytes: &[u8]) -> Result<Self, UmdlLoadError> {
        if bytes.len() < UMDL_HEADER_LENGTH as usize {
            return Err(UmdlLoadError::HeaderTooShort);
        }

        let header = Self {
            magic: read_magic(bytes),
            format_major: read_u16(bytes, 4),
            format_minor: read_u16(bytes, 6),
            header_length: read_u32(bytes, 8),
            architecture_id: read_u32(bytes, 12),
            quantization_scheme_id: read_u32(bytes, 16),
            tensor_count: read_u32(bytes, 20),
            tokenizer_section_offset: read_u64(bytes, 24),
            tokenizer_section_length: read_u64(bytes, 32),
            tensor_section_offset: read_u64(bytes, 40),
            tensor_section_length: read_u64(bytes, 48),
            weight_blob_offset: read_u64(bytes, 56),
            weight_blob_length: read_u64(bytes, 64),
            checksum_section_offset: read_u64(bytes, 72),
            checksum_section_length: read_u64(bytes, 80),
            required_memory_bytes: read_u64(bytes, 88),
            required_scratch_bytes: read_u64(bytes, 96),
            required_kv_cache_bytes_per_token: read_u64(bytes, 104),
            max_context_tokens: read_u32(bytes, 112),
            vocabulary_size: read_u32(bytes, 116),
            layer_count: read_u32(bytes, 120),
            hidden_size: read_u32(bytes, 124),
            attention_head_count: read_u32(bytes, 128),
            minimum_simd_tier: read_u32(bytes, 132),
            model_stable_id: read_u64(bytes, 136),
            header_checksum: read_u64(bytes, 144),
        };
        header.validate_header_prefix(bytes.len())?;
        Ok(header)
    }

    fn validate_header_prefix(self, input_len: usize) -> Result<(), UmdlLoadError> {
        if self.magic != UMDL_MAGIC {
            return Err(UmdlLoadError::BadMagic);
        }
        if self.format_major != UMDL_FORMAT_MAJOR || self.format_minor > UMDL_FORMAT_MINOR {
            return Err(UmdlLoadError::UnsupportedVersion {
                major: self.format_major,
                minor: self.format_minor,
            });
        }
        if self.header_length < UMDL_HEADER_LENGTH || self.header_length as usize > input_len {
            return Err(UmdlLoadError::HeaderTooShort);
        }
        Ok(())
    }
}

fn read_magic(bytes: &[u8]) -> [u8; 4] {
    [bytes[0], bytes[1], bytes[2], bytes[3]]
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
        bytes[offset + 4],
        bytes[offset + 5],
        bytes[offset + 6],
        bytes[offset + 7],
    ])
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

impl TokenizerType {
    #[must_use]
    pub const fn from_id(id: u32) -> Option<Self> {
        match id {
            1 => Some(Self::ByteFallbackBpe),
            2 => Some(Self::SentencepieceUnigram),
            3 => Some(Self::RawByteToToken),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum TokenizerUtf8Policy {
    Strict = 1,
}

impl TokenizerUtf8Policy {
    #[must_use]
    pub const fn from_id(id: u32) -> Option<Self> {
        match id {
            1 => Some(Self::Strict),
            _ => None,
        }
    }
}

pub const TOKENIZER_TOKEN_ID_NONE: u32 = u32::MAX;
pub const M7_SUPPORTED_TOKENIZER: TokenizerType = TokenizerType::RawByteToToken;
pub const RAW_BYTE_TO_TOKEN_VOCAB_SIZE: u32 = 256;
pub const RAW_BYTE_TO_TOKEN_MAX_TOKEN_BYTES: u32 = 1;

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TokenizerSpecialTokens {
    pub bos: u32,
    pub eos: u32,
    pub pad: u32,
    pub unk: u32,
}

impl TokenizerSpecialTokens {
    pub const NONE: Self = Self {
        bos: TOKENIZER_TOKEN_ID_NONE,
        eos: TOKENIZER_TOKEN_ID_NONE,
        pad: TOKENIZER_TOKEN_ID_NONE,
        unk: TOKENIZER_TOKEN_ID_NONE,
    };
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TokenizerMetadata {
    pub tokenizer_type: u32,
    pub vocabulary_size: u32,
    pub token_table_offset: u64,
    pub token_table_length: u64,
    pub merge_table_offset: u64,
    pub merge_table_length: u64,
    pub special_tokens: TokenizerSpecialTokens,
    pub utf8_policy: u32,
    pub max_token_byte_length: u32,
    pub tokenizer_tables_checksum: u64,
}

impl TokenizerMetadata {
    #[must_use]
    pub const fn raw_byte_to_token() -> Self {
        Self {
            tokenizer_type: TokenizerType::RawByteToToken as u32,
            vocabulary_size: RAW_BYTE_TO_TOKEN_VOCAB_SIZE,
            token_table_offset: 0,
            token_table_length: 0,
            merge_table_offset: 0,
            merge_table_length: 0,
            special_tokens: TokenizerSpecialTokens::NONE,
            utf8_policy: TokenizerUtf8Policy::Strict as u32,
            max_token_byte_length: RAW_BYTE_TO_TOKEN_MAX_TOKEN_BYTES,
            tokenizer_tables_checksum: 0,
        }
    }

    /// Validate the M7-supported tokenizer metadata shape.
    ///
    /// # Errors
    ///
    /// Returns `TokenizerMetadataError` when the tokenizer family is unknown or
    /// unsupported for M7, or when raw-byte metadata fields do not match the
    /// fixed-width, no-table shape required by `RawByteToToken`.
    pub fn validate_m7(self) -> Result<(), TokenizerMetadataError> {
        let Some(kind) = TokenizerType::from_id(self.tokenizer_type) else {
            return Err(TokenizerMetadataError::UnknownTokenizerType(
                self.tokenizer_type,
            ));
        };
        if kind != M7_SUPPORTED_TOKENIZER {
            return Err(TokenizerMetadataError::UnsupportedTokenizerType {
                requested: self.tokenizer_type,
                supported: M7_SUPPORTED_TOKENIZER as u32,
            });
        }
        if self.vocabulary_size != RAW_BYTE_TO_TOKEN_VOCAB_SIZE {
            return Err(TokenizerMetadataError::InvalidVocabularySize {
                expected: RAW_BYTE_TO_TOKEN_VOCAB_SIZE,
                actual: self.vocabulary_size,
            });
        }
        if TokenizerUtf8Policy::from_id(self.utf8_policy) != Some(TokenizerUtf8Policy::Strict) {
            return Err(TokenizerMetadataError::UnsupportedUtf8Policy(
                self.utf8_policy,
            ));
        }
        if self.max_token_byte_length != RAW_BYTE_TO_TOKEN_MAX_TOKEN_BYTES {
            return Err(TokenizerMetadataError::InvalidMaxTokenByteLength {
                expected: RAW_BYTE_TO_TOKEN_MAX_TOKEN_BYTES,
                actual: self.max_token_byte_length,
            });
        }
        if self.token_table_length != 0 || self.merge_table_length != 0 {
            return Err(TokenizerMetadataError::UnexpectedTokenizerTables);
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TokenizerMetadataError {
    UnknownTokenizerType(u32),
    UnsupportedTokenizerType { requested: u32, supported: u32 },
    InvalidVocabularySize { expected: u32, actual: u32 },
    UnsupportedUtf8Policy(u32),
    InvalidMaxTokenByteLength { expected: u32, actual: u32 },
    UnexpectedTokenizerTables,
}

/// Tensor descriptor — spec §10.12. The `pad_after_rank` field
/// makes the implicit `#[repr(C)]` alignment padding explicit so
/// the on-disk layout is auditable.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TensorDesc {
    pub tensor_id: u32,
    pub scalar_type: u8,
    pub quant_type: u8,
    pub rank: u8,
    pub pad_after_rank: u8,
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

    fn minimal_header_bytes() -> [u8; UMDL_HEADER_LENGTH as usize] {
        let mut bytes = [0u8; UMDL_HEADER_LENGTH as usize];
        bytes[0..4].copy_from_slice(&UMDL_MAGIC);
        bytes[4..6].copy_from_slice(&UMDL_FORMAT_MAJOR.to_le_bytes());
        bytes[6..8].copy_from_slice(&UMDL_FORMAT_MINOR.to_le_bytes());
        bytes[8..12].copy_from_slice(&UMDL_HEADER_LENGTH.to_le_bytes());
        bytes[12..16].copy_from_slice(&1u32.to_le_bytes());
        bytes[16..20].copy_from_slice(&(QuantType::QNoneF32 as u32).to_le_bytes());
        bytes[20..24].copy_from_slice(&0u32.to_le_bytes());
        bytes[112..116].copy_from_slice(&32u32.to_le_bytes());
        bytes[116..120].copy_from_slice(&RAW_BYTE_TO_TOKEN_VOCAB_SIZE.to_le_bytes());
        bytes[120..124].copy_from_slice(&1u32.to_le_bytes());
        bytes[124..128].copy_from_slice(&8u32.to_le_bytes());
        bytes[128..132].copy_from_slice(&1u32.to_le_bytes());
        bytes[132..136].copy_from_slice(&(SimdTier::Scalar as u32).to_le_bytes());
        bytes[136..144].copy_from_slice(&0x0000_0000_0009_0001u64.to_le_bytes());
        bytes
    }

    #[test]
    fn magic_bytes_are_stable() {
        assert_eq!(UMDL_MAGIC, [b'U', b'M', b'D', b'L']);
    }

    #[test]
    fn header_layout_is_fixed_width() {
        assert_eq!(
            core::mem::size_of::<UmdlHeader>(),
            UMDL_HEADER_LENGTH as usize
        );
    }

    #[test]
    fn parses_umdl_header_from_little_endian_bytes() {
        let header = UmdlHeader::parse(&minimal_header_bytes()).expect("UMDL header");

        assert_eq!(header.magic, UMDL_MAGIC);
        assert_eq!(header.format_major, UMDL_FORMAT_MAJOR);
        assert_eq!(header.format_minor, UMDL_FORMAT_MINOR);
        assert_eq!(header.header_length, UMDL_HEADER_LENGTH);
        assert_eq!(header.architecture_id, 1);
        assert_eq!(header.quantization_scheme_id, QuantType::QNoneF32 as u32);
        assert_eq!(header.minimum_simd_tier, SimdTier::Scalar as u32);
        assert_eq!(header.model_stable_id, 0x0000_0000_0009_0001);
    }

    #[test]
    fn rejects_malformed_umdl_header_prefix() {
        assert_eq!(
            UmdlHeader::parse(&minimal_header_bytes()[..UMDL_HEADER_LENGTH as usize - 1])
                .unwrap_err(),
            UmdlLoadError::HeaderTooShort
        );

        let mut bytes = minimal_header_bytes();
        bytes[0] = b'X';
        assert_eq!(
            UmdlHeader::parse(&bytes).unwrap_err(),
            UmdlLoadError::BadMagic
        );

        let mut bytes = minimal_header_bytes();
        bytes[4..6].copy_from_slice(&2u16.to_le_bytes());
        assert_eq!(
            UmdlHeader::parse(&bytes).unwrap_err(),
            UmdlLoadError::UnsupportedVersion { major: 2, minor: 0 }
        );

        let mut bytes = minimal_header_bytes();
        bytes[8..12].copy_from_slice(&(UMDL_HEADER_LENGTH - 1).to_le_bytes());
        assert_eq!(
            UmdlHeader::parse(&bytes).unwrap_err(),
            UmdlLoadError::HeaderTooShort
        );
    }

    #[test]
    fn simd_tier_ordering() {
        assert!(SimdTier::Scalar < SimdTier::Sse2);
        assert!(SimdTier::Sse2 < SimdTier::Avx2);
        assert!(SimdTier::Avx2 < SimdTier::Avx512);
    }

    #[test]
    fn tokenizer_metadata_layout_is_fixed_width() {
        assert_eq!(core::mem::size_of::<TokenizerSpecialTokens>(), 16);
        assert_eq!(core::mem::size_of::<TokenizerMetadata>(), 72);
    }

    #[test]
    fn raw_byte_tokenizer_metadata_validates_for_m7() {
        assert_eq!(M7_SUPPORTED_TOKENIZER, TokenizerType::RawByteToToken);
        TokenizerMetadata::raw_byte_to_token()
            .validate_m7()
            .expect("raw-byte tokenizer metadata");
    }

    #[test]
    fn m7_rejects_unsupported_tokenizer_families() {
        for tokenizer_type in [
            TokenizerType::ByteFallbackBpe as u32,
            TokenizerType::SentencepieceUnigram as u32,
        ] {
            let mut metadata = TokenizerMetadata::raw_byte_to_token();
            metadata.tokenizer_type = tokenizer_type;
            assert_eq!(
                metadata.validate_m7().unwrap_err(),
                TokenizerMetadataError::UnsupportedTokenizerType {
                    requested: tokenizer_type,
                    supported: TokenizerType::RawByteToToken as u32,
                }
            );
        }
    }

    #[test]
    fn raw_byte_tokenizer_metadata_rejects_bad_shape() {
        let mut metadata = TokenizerMetadata::raw_byte_to_token();
        metadata.vocabulary_size = 255;
        assert_eq!(
            metadata.validate_m7().unwrap_err(),
            TokenizerMetadataError::InvalidVocabularySize {
                expected: RAW_BYTE_TO_TOKEN_VOCAB_SIZE,
                actual: 255,
            }
        );

        let mut metadata = TokenizerMetadata::raw_byte_to_token();
        metadata.merge_table_length = 1;
        assert_eq!(
            metadata.validate_m7().unwrap_err(),
            TokenizerMetadataError::UnexpectedTokenizerTables
        );
    }
}
