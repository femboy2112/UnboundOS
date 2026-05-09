//! CPU feature detection and SIMD enable. Spec sections 3.3, 3.4.

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SimdTier { Scalar, Sse2, Avx, Avx2, Avx512 }

impl SimdTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Scalar => "Scalar",
            Self::Sse2 => "Sse2",
            Self::Avx => "Avx",
            Self::Avx2 => "Avx2",
            Self::Avx512 => "Avx512",
        }
    }
}

pub fn detect_features() -> SimdTier {
    // CPUID + XCR0 + OSXSAVE checks per spec 3.3. Stub returns Scalar.
    SimdTier::Scalar
}

/// # Safety
///
/// Single CR0/CR4/XCR0 writer. Must be called once during boot, after
/// `detect_features` and before any SIMD-using code path.
pub unsafe fn enable_math_features(_tier: SimdTier) {
    // CR0.EM clear, CR0.MP set, CR4.OSFXSR/OSXMMEXCPT set, optionally
    // CR4.OSXSAVE + XSETBV. Stub.
}
