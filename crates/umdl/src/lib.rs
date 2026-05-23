//! UMDL — bare-metal model package. Spec §10.5.
//!
//! Persistent format. Header + tokenizer section + tensor descriptor
//! table + weight blob + checksum section. No pointers, no host
//! paths, no Linux/CUDA assumptions. Loaded into `ModelWeightArena`;
//! becomes read-only after load on profiles that support paging.

#![no_std]
#![forbid(unsafe_op_in_unsafe_fn)]

use core::ops::Range;

pub const UMDL_MAGIC: [u8; 4] = *b"UMDL";

pub const UMDL_FORMAT_MAJOR: u16 = 1;
pub const UMDL_FORMAT_MINOR: u16 = 0;
pub const UMDL_HEADER_LENGTH: u32 = 152;
pub const UMDL_CHECKSUM_SEED: u64 = 0xcbf2_9ce4_8422_2325;
pub const UMDL_CHECKSUM_PRIME: u64 = 0x0000_0100_0000_01b3;
pub const TOKENIZER_METADATA_LENGTH: u64 = 72;
pub const TENSOR_DESC_LENGTH: u64 = 48;
pub const TENSOR_DESC_LENGTH_USIZE: usize = 48;
pub const M9_SUPPORTED_ARCHITECTURE_ID: u32 = 1;

/// Header at file offset 0. Spec §10.5.
#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

    /// Validate declared section ranges and deterministic checksums.
    ///
    /// # Errors
    ///
    /// Returns `UmdlLoadError` when a section range is out of bounds,
    /// overlapping, or its computed checksum does not match the expected
    /// checksum supplied by the checksum-section parser.
    pub fn validate_sections(
        self,
        bytes: &[u8],
        checksums: UmdlSectionChecksums,
    ) -> Result<UmdlSectionRanges, UmdlLoadError> {
        self.validate_header_prefix(bytes.len())?;
        if checksum_header(bytes, self.header_length) != self.header_checksum {
            return Err(UmdlLoadError::HeaderChecksumMismatch);
        }

        let ranges = UmdlSectionRanges {
            tokenizer: UmdlSectionRange::new(
                self.tokenizer_section_offset,
                self.tokenizer_section_length,
                bytes.len(),
            )?,
            tensor: UmdlSectionRange::new(
                self.tensor_section_offset,
                self.tensor_section_length,
                bytes.len(),
            )?,
            weight_blob: UmdlSectionRange::new(
                self.weight_blob_offset,
                self.weight_blob_length,
                bytes.len(),
            )?,
            checksum: UmdlSectionRange::new(
                self.checksum_section_offset,
                self.checksum_section_length,
                bytes.len(),
            )?,
        };
        ranges.validate_non_overlapping()?;

        if checksum64(ranges.tokenizer.slice(bytes)) != checksums.tokenizer {
            return Err(UmdlLoadError::TokenizerSectionChecksumMismatch);
        }
        if checksum64(ranges.tensor.slice(bytes)) != checksums.tensor {
            return Err(UmdlLoadError::TensorSectionChecksumMismatch);
        }
        if checksum64(ranges.weight_blob.slice(bytes)) != checksums.weight_blob {
            return Err(UmdlLoadError::WeightBlobChecksumMismatch);
        }
        Ok(ranges)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct UmdlSectionChecksums {
    pub tokenizer: u64,
    pub tensor: u64,
    pub weight_blob: u64,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct UmdlSectionRanges {
    pub tokenizer: UmdlSectionRange,
    pub tensor: UmdlSectionRange,
    pub weight_blob: UmdlSectionRange,
    pub checksum: UmdlSectionRange,
}

impl UmdlSectionRanges {
    fn validate_non_overlapping(self) -> Result<(), UmdlLoadError> {
        let ranges = [self.tokenizer, self.tensor, self.weight_blob, self.checksum];
        for left_index in 0..ranges.len() {
            for right in &ranges[left_index + 1..] {
                if ranges[left_index].overlaps(*right) {
                    return Err(UmdlLoadError::SectionOverlap);
                }
            }
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct UmdlSectionRange {
    pub offset: u64,
    pub length: u64,
}

impl UmdlSectionRange {
    fn new(offset: u64, length: u64, input_len: usize) -> Result<Self, UmdlLoadError> {
        let Some(end) = offset.checked_add(length) else {
            return Err(UmdlLoadError::SectionOutOfBounds {
                offset,
                length,
                input_len: len_to_u64(input_len),
            });
        };
        if end > len_to_u64(input_len) {
            return Err(UmdlLoadError::SectionOutOfBounds {
                offset,
                length,
                input_len: len_to_u64(input_len),
            });
        }
        Ok(Self { offset, length })
    }

    fn as_range(self) -> Range<usize> {
        let start = usize::try_from(self.offset).expect("validated section offset");
        let length = usize::try_from(self.length).expect("validated section length");
        start..start + length
    }

    fn slice(self, bytes: &[u8]) -> &[u8] {
        &bytes[self.as_range()]
    }

    fn overlaps(self, other: Self) -> bool {
        if self.length == 0 || other.length == 0 {
            return false;
        }
        let self_end = self.offset + self.length;
        let other_end = other.offset + other.length;
        self.offset < other_end && other.offset < self_end
    }
}

#[must_use]
pub fn checksum64(bytes: &[u8]) -> u64 {
    let mut hash = UMDL_CHECKSUM_SEED;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(UMDL_CHECKSUM_PRIME);
    }
    hash
}

#[must_use]
pub fn checksum_header(bytes: &[u8], header_length: u32) -> u64 {
    let header_len = core::cmp::min(header_length as usize, bytes.len());
    let mut hash = UMDL_CHECKSUM_SEED;
    for (offset, byte) in bytes[..header_len].iter().copied().enumerate() {
        let value = if (144..152).contains(&offset) {
            0
        } else {
            byte
        };
        hash ^= u64::from(value);
        hash = hash.wrapping_mul(UMDL_CHECKSUM_PRIME);
    }
    hash
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

impl ScalarType {
    #[must_use]
    pub const fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Self::F32),
            1 => Some(Self::F16),
            2 => Some(Self::BF16),
            3 => Some(Self::I8),
            4 => Some(Self::U8),
            _ => None,
        }
    }
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

impl QuantType {
    #[must_use]
    pub const fn from_id(id: u32) -> Option<Self> {
        match id {
            0 => Some(Self::QNoneF32),
            1 => Some(Self::QNoneF16),
            10 => Some(Self::Q4Block32),
            11 => Some(Self::Q8Block32),
            _ => None,
        }
    }
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

    /// Parse fixed-width tokenizer metadata from a validated tokenizer section.
    ///
    /// # Errors
    ///
    /// Returns `UmdlLoadError` when the section is not exactly one tokenizer
    /// metadata record or when the metadata violates the M7 raw-byte tokenizer
    /// contract.
    pub fn parse_umdl(bytes: &[u8], range: UmdlSectionRange) -> Result<Self, UmdlLoadError> {
        if range.length != TOKENIZER_METADATA_LENGTH {
            return Err(UmdlLoadError::InvalidTokenizerSectionLength {
                expected: TOKENIZER_METADATA_LENGTH,
                actual: range.length,
            });
        }
        let section = range.slice(bytes);
        let metadata = Self {
            tokenizer_type: read_u32(section, 0),
            vocabulary_size: read_u32(section, 4),
            token_table_offset: read_u64(section, 8),
            token_table_length: read_u64(section, 16),
            merge_table_offset: read_u64(section, 24),
            merge_table_length: read_u64(section, 32),
            special_tokens: TokenizerSpecialTokens {
                bos: read_u32(section, 40),
                eos: read_u32(section, 44),
                pad: read_u32(section, 48),
                unk: read_u32(section, 52),
            },
            utf8_policy: read_u32(section, 56),
            max_token_byte_length: read_u32(section, 60),
            tokenizer_tables_checksum: read_u64(section, 64),
        };
        metadata.validate_m7().map_err(|error| match error {
            TokenizerMetadataError::UnknownTokenizerType(kind) => {
                UmdlLoadError::UnknownTokenizerType(kind)
            }
            _ => UmdlLoadError::InvalidTokenizerMetadata,
        })?;
        Ok(metadata)
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

impl TensorDesc {
    /// Parse one fixed-width tensor descriptor from a tensor section.
    ///
    /// # Errors
    ///
    /// Returns `UmdlLoadError::TensorOutOfBounds` when `index` is outside the
    /// descriptor section.
    pub fn parse_umdl(
        bytes: &[u8],
        tensor_section: UmdlSectionRange,
        index: u32,
    ) -> Result<Self, UmdlLoadError> {
        let byte_offset = u64::from(index)
            .checked_mul(TENSOR_DESC_LENGTH)
            .ok_or(UmdlLoadError::TensorOutOfBounds { tensor_id: index })?;
        let Some(byte_end) = byte_offset.checked_add(TENSOR_DESC_LENGTH) else {
            return Err(UmdlLoadError::TensorOutOfBounds { tensor_id: index });
        };
        if byte_end > tensor_section.length {
            return Err(UmdlLoadError::TensorOutOfBounds { tensor_id: index });
        }
        let section = tensor_section.slice(bytes);
        let start = usize::try_from(byte_offset)
            .map_err(|_| UmdlLoadError::TensorOutOfBounds { tensor_id: index })?;
        let desc = &section[start..start + TENSOR_DESC_LENGTH_USIZE];
        Ok(Self {
            tensor_id: read_u32(desc, 0),
            scalar_type: desc[4],
            quant_type: desc[5],
            rank: desc[6],
            pad_after_rank: desc[7],
            dims: [
                read_u32(desc, 8),
                read_u32(desc, 12),
                read_u32(desc, 16),
                read_u32(desc, 20),
            ],
            byte_offset: read_u64(desc, 24),
            byte_length: read_u64(desc, 32),
            alignment: read_u32(desc, 40),
            flags: read_u32(desc, 44),
        })
    }

    /// Validate descriptor IDs, shape, alignment, and weight-blob bounds.
    ///
    /// # Errors
    ///
    /// Returns `UmdlLoadError` when descriptor fields are unsupported or point
    /// outside the declared weight blob section.
    pub fn validate_umdl(self, weight_blob: UmdlSectionRange) -> Result<(), UmdlLoadError> {
        if ScalarType::from_id(self.scalar_type).is_none() {
            return Err(UmdlLoadError::UnknownScalarType(self.scalar_type));
        }
        if QuantType::from_id(u32::from(self.quant_type)).is_none() {
            return Err(UmdlLoadError::UnknownQuantizationId(u32::from(
                self.quant_type,
            )));
        }
        if self.rank == 0 || self.rank > 4 {
            return Err(UmdlLoadError::InvalidTensorRank {
                tensor_id: self.tensor_id,
                rank: self.rank,
            });
        }
        for index in 0..4 {
            let dim = self.dims[index];
            if (index < usize::from(self.rank) && dim == 0)
                || (index >= usize::from(self.rank) && dim != 0)
            {
                return Err(UmdlLoadError::InvalidTensorShape {
                    tensor_id: self.tensor_id,
                });
            }
        }
        if self.alignment == 0 || !self.alignment.is_power_of_two() {
            return Err(UmdlLoadError::InvalidTensorAlignment {
                tensor_id: self.tensor_id,
                alignment: self.alignment,
            });
        }
        let Some(end) = self.byte_offset.checked_add(self.byte_length) else {
            return Err(UmdlLoadError::TensorOutOfBounds {
                tensor_id: self.tensor_id,
            });
        };
        if end > weight_blob.length {
            return Err(UmdlLoadError::TensorOutOfBounds {
                tensor_id: self.tensor_id,
            });
        }
        Ok(())
    }
}

/// Validate every tensor descriptor in a tensor section.
///
/// # Errors
///
/// Returns `UmdlLoadError` when the section length does not match
/// `tensor_count` or any descriptor is invalid.
pub fn validate_tensor_descriptors(
    bytes: &[u8],
    tensor_section: UmdlSectionRange,
    weight_blob: UmdlSectionRange,
    tensor_count: u32,
) -> Result<(), UmdlLoadError> {
    let expected = u64::from(tensor_count)
        .checked_mul(TENSOR_DESC_LENGTH)
        .ok_or(UmdlLoadError::InvalidTensorSectionLength {
            expected: u64::MAX,
            actual: tensor_section.length,
        })?;
    if tensor_section.length != expected {
        return Err(UmdlLoadError::InvalidTensorSectionLength {
            expected,
            actual: tensor_section.length,
        });
    }
    let mut index = 0;
    while index < tensor_count {
        TensorDesc::parse_umdl(bytes, tensor_section, index)?.validate_umdl(weight_blob)?;
        index += 1;
    }
    Ok(())
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct UmdlArenaReservations {
    pub model_weight_bytes: u64,
    pub scratch_bytes: u64,
    pub kv_cache_bytes_per_token: u64,
    pub max_context_tokens: u32,
}

impl UmdlArenaReservations {
    fn validate_against_profile(self, profile_ram_bytes: u64) -> Result<(), UmdlLoadError> {
        let Some(kv_total) = self
            .kv_cache_bytes_per_token
            .checked_mul(u64::from(self.max_context_tokens))
        else {
            return Err(UmdlLoadError::ProfileRamBudgetExceeded {
                required: u64::MAX,
                available: profile_ram_bytes,
            });
        };
        let Some(required) = self
            .model_weight_bytes
            .checked_add(self.scratch_bytes)
            .and_then(|value| value.checked_add(kv_total))
        else {
            return Err(UmdlLoadError::ProfileRamBudgetExceeded {
                required: u64::MAX,
                available: profile_ram_bytes,
            });
        };
        if required > profile_ram_bytes {
            return Err(UmdlLoadError::ProfileRamBudgetExceeded {
                required,
                available: profile_ram_bytes,
            });
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LoadedUmdlModel {
    pub header: UmdlHeader,
    pub tokenizer: TokenizerMetadata,
    pub tensor_count: u32,
    pub ranges: UmdlSectionRanges,
    pub reservations: UmdlArenaReservations,
    pub active_simd_tier: SimdTier,
}

/// Validate a UMDL byte package and return a read-only loaded model view.
///
/// # Errors
///
/// Returns `UmdlLoadError` when header, section, tokenizer, tensor, SIMD, or
/// arena-reservation validation fails.
pub fn load_model_view(
    bytes: &[u8],
    checksums: UmdlSectionChecksums,
    available_simd_tier: SimdTier,
    profile_ram_bytes: u64,
) -> Result<LoadedUmdlModel, UmdlLoadError> {
    let header = UmdlHeader::parse(bytes)?;
    if header.architecture_id != M9_SUPPORTED_ARCHITECTURE_ID {
        return Err(UmdlLoadError::UnknownArchitectureId(header.architecture_id));
    }
    let required_simd =
        SimdTier::from_id(header.minimum_simd_tier).ok_or(UmdlLoadError::SimdRequirementUnmet {
            required: header.minimum_simd_tier,
            available: available_simd_tier as u32,
        })?;
    if required_simd > available_simd_tier {
        return Err(UmdlLoadError::SimdRequirementUnmet {
            required: header.minimum_simd_tier,
            available: available_simd_tier as u32,
        });
    }
    let ranges = header.validate_sections(bytes, checksums)?;
    let tokenizer = TokenizerMetadata::parse_umdl(bytes, ranges.tokenizer)?;
    validate_tensor_descriptors(
        bytes,
        ranges.tensor,
        ranges.weight_blob,
        header.tensor_count,
    )?;
    let reservations = UmdlArenaReservations {
        model_weight_bytes: header.required_memory_bytes,
        scratch_bytes: header.required_scratch_bytes,
        kv_cache_bytes_per_token: header.required_kv_cache_bytes_per_token,
        max_context_tokens: header.max_context_tokens,
    };
    reservations.validate_against_profile(profile_ram_bytes)?;
    Ok(LoadedUmdlModel {
        header,
        tokenizer,
        tensor_count: header.tensor_count,
        ranges,
        reservations,
        active_simd_tier: available_simd_tier,
    })
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

impl SimdTier {
    #[must_use]
    pub const fn from_id(id: u32) -> Option<Self> {
        match id {
            0 => Some(Self::Scalar),
            1 => Some(Self::Sse2),
            2 => Some(Self::Avx),
            3 => Some(Self::Avx2),
            4 => Some(Self::Avx512),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UmdlLoadError {
    BadMagic,
    UnsupportedVersion {
        major: u16,
        minor: u16,
    },
    HeaderTooShort,
    HeaderChecksumMismatch,
    TokenizerSectionChecksumMismatch,
    TensorSectionChecksumMismatch,
    WeightBlobChecksumMismatch,
    SectionOutOfBounds {
        offset: u64,
        length: u64,
        input_len: u64,
    },
    SectionOverlap,
    TensorOutOfBounds {
        tensor_id: u32,
    },
    InvalidTokenizerSectionLength {
        expected: u64,
        actual: u64,
    },
    InvalidTokenizerMetadata,
    InvalidTensorSectionLength {
        expected: u64,
        actual: u64,
    },
    UnknownScalarType(u8),
    UnknownQuantizationId(u32),
    InvalidTensorRank {
        tensor_id: u32,
        rank: u8,
    },
    InvalidTensorShape {
        tensor_id: u32,
    },
    InvalidTensorAlignment {
        tensor_id: u32,
        alignment: u32,
    },
    UnknownTokenizerType(u32),
    UnknownArchitectureId(u32),
    SimdRequirementUnmet {
        required: u32,
        available: u32,
    },
    ProfileRamBudgetExceeded {
        required: u64,
        available: u64,
    },
}

fn len_to_u64(len: usize) -> u64 {
    u64::try_from(len).unwrap_or(u64::MAX)
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

    fn write_u64(bytes: &mut [u8], offset: usize, value: u64) {
        bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }

    fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn sectioned_umdl_bytes() -> ([u8; 256], UmdlSectionChecksums) {
        let mut bytes = [0u8; 256];
        bytes[..UMDL_HEADER_LENGTH as usize].copy_from_slice(&minimal_header_bytes());
        bytes[160..164].copy_from_slice(b"tokn");
        bytes[176..184].copy_from_slice(b"tensor!!");
        bytes[192..200].copy_from_slice(b"weights!");

        write_u64(&mut bytes, 24, 160);
        write_u64(&mut bytes, 32, 4);
        write_u64(&mut bytes, 40, 176);
        write_u64(&mut bytes, 48, 8);
        write_u64(&mut bytes, 56, 192);
        write_u64(&mut bytes, 64, 8);
        write_u64(&mut bytes, 72, 224);
        write_u64(&mut bytes, 80, 24);

        let checksums = UmdlSectionChecksums {
            tokenizer: checksum64(&bytes[160..164]),
            tensor: checksum64(&bytes[176..184]),
            weight_blob: checksum64(&bytes[192..200]),
        };
        let header_checksum = checksum_header(&bytes, UMDL_HEADER_LENGTH);
        write_u64(&mut bytes, 144, header_checksum);
        (bytes, checksums)
    }

    fn refresh_header_checksum(bytes: &mut [u8]) {
        write_u64(bytes, 144, 0);
        write_u64(bytes, 144, checksum_header(bytes, UMDL_HEADER_LENGTH));
    }

    fn checksums_for_described(bytes: &[u8; 512]) -> UmdlSectionChecksums {
        UmdlSectionChecksums {
            tokenizer: checksum64(&bytes[160..232]),
            tensor: checksum64(&bytes[240..288]),
            weight_blob: checksum64(&bytes[320..336]),
        }
    }

    fn described_umdl_bytes() -> ([u8; 512], UmdlSectionChecksums) {
        let mut bytes = [0u8; 512];
        bytes[..UMDL_HEADER_LENGTH as usize].copy_from_slice(&minimal_header_bytes());
        write_u32(&mut bytes, 20, 1);

        write_u64(&mut bytes, 24, 160);
        write_u64(&mut bytes, 32, TOKENIZER_METADATA_LENGTH);
        write_u64(&mut bytes, 40, 240);
        write_u64(&mut bytes, 48, TENSOR_DESC_LENGTH);
        write_u64(&mut bytes, 56, 320);
        write_u64(&mut bytes, 64, 16);
        write_u64(&mut bytes, 72, 400);
        write_u64(&mut bytes, 80, 24);
        write_u64(&mut bytes, 88, 16);
        write_u64(&mut bytes, 96, 8);
        write_u64(&mut bytes, 104, 2);

        write_u32(&mut bytes, 160, TokenizerType::RawByteToToken as u32);
        write_u32(&mut bytes, 164, RAW_BYTE_TO_TOKEN_VOCAB_SIZE);
        write_u32(&mut bytes, 200, TOKENIZER_TOKEN_ID_NONE);
        write_u32(&mut bytes, 204, TOKENIZER_TOKEN_ID_NONE);
        write_u32(&mut bytes, 208, TOKENIZER_TOKEN_ID_NONE);
        write_u32(&mut bytes, 212, TOKENIZER_TOKEN_ID_NONE);
        write_u32(&mut bytes, 216, TokenizerUtf8Policy::Strict as u32);
        write_u32(&mut bytes, 220, RAW_BYTE_TO_TOKEN_MAX_TOKEN_BYTES);

        write_u32(&mut bytes, 240, 7);
        bytes[244] = ScalarType::F32 as u8;
        bytes[245] = QuantType::QNoneF32 as u8;
        bytes[246] = 2;
        write_u32(&mut bytes, 248, 2);
        write_u32(&mut bytes, 252, 4);
        write_u64(&mut bytes, 264, 0);
        write_u64(&mut bytes, 272, 16);
        write_u32(&mut bytes, 280, 16);

        bytes[320..336].copy_from_slice(b"0123456789abcdef");

        refresh_header_checksum(&mut bytes);
        let checksums = checksums_for_described(&bytes);
        (bytes, checksums)
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
    fn malformed_corpus_fixtures_reject_with_declared_errors() {
        let bad_magic = include_bytes!("../../../tests/fuzz_corpus/umdl/bad-magic.umdl");

        assert_eq!(
            UmdlHeader::parse(bad_magic).unwrap_err(),
            UmdlLoadError::BadMagic
        );
        assert_eq!(
            load_model_view(
                bad_magic,
                UmdlSectionChecksums {
                    tokenizer: 0,
                    tensor: 0,
                    weight_blob: 0,
                },
                SimdTier::Scalar,
                u64::MAX,
            )
            .unwrap_err(),
            UmdlLoadError::BadMagic
        );
    }

    #[test]
    fn validates_section_ranges_and_checksums() {
        let (bytes, checksums) = sectioned_umdl_bytes();
        let header = UmdlHeader::parse(&bytes).expect("header");
        let ranges = header
            .validate_sections(&bytes, checksums)
            .expect("valid sections");

        assert_eq!(
            ranges.tokenizer,
            UmdlSectionRange {
                offset: 160,
                length: 4
            }
        );
        assert_eq!(
            ranges.tensor,
            UmdlSectionRange {
                offset: 176,
                length: 8
            }
        );
        assert_eq!(
            ranges.weight_blob,
            UmdlSectionRange {
                offset: 192,
                length: 8
            }
        );
        assert_eq!(
            ranges.checksum,
            UmdlSectionRange {
                offset: 224,
                length: 24
            }
        );
    }

    #[test]
    fn rejects_section_bounds_overlap_and_checksum_failures() {
        let (mut bytes, checksums) = sectioned_umdl_bytes();
        write_u64(&mut bytes, 56, 250);
        write_u64(&mut bytes, 64, 16);
        refresh_header_checksum(&mut bytes);
        let header = UmdlHeader::parse(&bytes).expect("header");
        assert_eq!(
            header.validate_sections(&bytes, checksums).unwrap_err(),
            UmdlLoadError::SectionOutOfBounds {
                offset: 250,
                length: 16,
                input_len: 256,
            }
        );

        let (mut bytes, checksums) = sectioned_umdl_bytes();
        write_u64(&mut bytes, 40, 162);
        refresh_header_checksum(&mut bytes);
        let header = UmdlHeader::parse(&bytes).expect("header");
        assert_eq!(
            header.validate_sections(&bytes, checksums).unwrap_err(),
            UmdlLoadError::SectionOverlap
        );

        let (mut bytes, checksums) = sectioned_umdl_bytes();
        bytes[12] ^= 1;
        let header = UmdlHeader::parse(&bytes).expect("header");
        assert_eq!(
            header.validate_sections(&bytes, checksums).unwrap_err(),
            UmdlLoadError::HeaderChecksumMismatch
        );

        let (bytes, mut checksums) = sectioned_umdl_bytes();
        checksums.tokenizer ^= 1;
        let header = UmdlHeader::parse(&bytes).expect("header");
        assert_eq!(
            header.validate_sections(&bytes, checksums).unwrap_err(),
            UmdlLoadError::TokenizerSectionChecksumMismatch
        );
    }

    #[test]
    fn parses_tokenizer_metadata_and_tensor_descriptors() {
        let (bytes, checksums) = described_umdl_bytes();
        let header = UmdlHeader::parse(&bytes).expect("header");
        let ranges = header
            .validate_sections(&bytes, checksums)
            .expect("valid sections");
        let tokenizer =
            TokenizerMetadata::parse_umdl(&bytes, ranges.tokenizer).expect("tokenizer metadata");
        let tensor = TensorDesc::parse_umdl(&bytes, ranges.tensor, 0).expect("tensor desc");

        assert_eq!(tokenizer, TokenizerMetadata::raw_byte_to_token());
        assert_eq!(tensor.tensor_id, 7);
        assert_eq!(tensor.scalar_type, ScalarType::F32 as u8);
        assert_eq!(tensor.quant_type, QuantType::QNoneF32 as u8);
        assert_eq!(tensor.rank, 2);
        assert_eq!(tensor.dims, [2, 4, 0, 0]);
        validate_tensor_descriptors(
            &bytes,
            ranges.tensor,
            ranges.weight_blob,
            header.tensor_count,
        )
        .expect("tensor descriptors");
    }

    #[test]
    fn rejects_bad_tokenizer_and_tensor_metadata() {
        let (mut bytes, _) = described_umdl_bytes();
        write_u32(&mut bytes, 160, 99);
        refresh_header_checksum(&mut bytes);
        let checksums = checksums_for_described(&bytes);
        let header = UmdlHeader::parse(&bytes).expect("header");
        let ranges = header
            .validate_sections(&bytes, checksums)
            .expect("valid sections");
        assert_eq!(
            TokenizerMetadata::parse_umdl(&bytes, ranges.tokenizer).unwrap_err(),
            UmdlLoadError::UnknownTokenizerType(99)
        );

        let (mut bytes, _) = described_umdl_bytes();
        bytes[245] = 99;
        refresh_header_checksum(&mut bytes);
        let checksums = checksums_for_described(&bytes);
        let header = UmdlHeader::parse(&bytes).expect("header");
        let ranges = header
            .validate_sections(&bytes, checksums)
            .expect("valid sections");
        assert_eq!(
            validate_tensor_descriptors(&bytes, ranges.tensor, ranges.weight_blob, 1).unwrap_err(),
            UmdlLoadError::UnknownQuantizationId(99)
        );

        let (mut bytes, _) = described_umdl_bytes();
        write_u32(&mut bytes, 252, 0);
        refresh_header_checksum(&mut bytes);
        let checksums = checksums_for_described(&bytes);
        let header = UmdlHeader::parse(&bytes).expect("header");
        let ranges = header
            .validate_sections(&bytes, checksums)
            .expect("valid sections");
        assert_eq!(
            validate_tensor_descriptors(&bytes, ranges.tensor, ranges.weight_blob, 1).unwrap_err(),
            UmdlLoadError::InvalidTensorShape { tensor_id: 7 }
        );

        let (mut bytes, _) = described_umdl_bytes();
        write_u64(&mut bytes, 264, 8);
        write_u64(&mut bytes, 272, 16);
        refresh_header_checksum(&mut bytes);
        let checksums = checksums_for_described(&bytes);
        let header = UmdlHeader::parse(&bytes).expect("header");
        let ranges = header
            .validate_sections(&bytes, checksums)
            .expect("valid sections");
        assert_eq!(
            validate_tensor_descriptors(&bytes, ranges.tensor, ranges.weight_blob, 1).unwrap_err(),
            UmdlLoadError::TensorOutOfBounds { tensor_id: 7 }
        );
    }

    #[test]
    fn loads_read_only_model_view_and_arena_reservations() {
        let (bytes, checksums) = described_umdl_bytes();
        let view =
            load_model_view(&bytes, checksums, SimdTier::Scalar, 128).expect("loaded model view");

        assert_eq!(view.header.architecture_id, M9_SUPPORTED_ARCHITECTURE_ID);
        assert_eq!(view.tokenizer, TokenizerMetadata::raw_byte_to_token());
        assert_eq!(view.tensor_count, 1);
        assert_eq!(view.active_simd_tier, SimdTier::Scalar);
        assert_eq!(
            view.reservations,
            UmdlArenaReservations {
                model_weight_bytes: 16,
                scratch_bytes: 8,
                kv_cache_bytes_per_token: 2,
                max_context_tokens: 32,
            }
        );
    }

    #[test]
    fn load_model_view_rejects_simd_and_profile_budget_mismatch() {
        let (mut bytes, _) = described_umdl_bytes();
        write_u32(&mut bytes, 132, SimdTier::Avx2 as u32);
        refresh_header_checksum(&mut bytes);
        let checksums = checksums_for_described(&bytes);
        assert_eq!(
            load_model_view(&bytes, checksums, SimdTier::Sse2, 128).unwrap_err(),
            UmdlLoadError::SimdRequirementUnmet {
                required: SimdTier::Avx2 as u32,
                available: SimdTier::Sse2 as u32,
            }
        );

        let (bytes, checksums) = described_umdl_bytes();
        assert_eq!(
            load_model_view(&bytes, checksums, SimdTier::Scalar, 87).unwrap_err(),
            UmdlLoadError::ProfileRamBudgetExceeded {
                required: 88,
                available: 87,
            }
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
