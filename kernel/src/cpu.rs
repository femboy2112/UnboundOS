//! CPU feature detection and SIMD enable. Spec sections 3.3, 3.4.

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SimdTier {
    Scalar,
    Sse2,
    Avx,
    Avx2,
    Avx512,
}

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
    let leaf1 = core::arch::x86_64::__cpuid(1);
    if leaf1.edx & (1 << 26) != 0 {
        SimdTier::Sse2
    } else {
        SimdTier::Scalar
    }
}

/// # Safety
///
/// Single CR0/CR4/XCR0 writer. Must be called once during boot, after
/// `detect_features` and before any SIMD-using code path.
pub unsafe fn enable_math_features(tier: SimdTier) {
    let mut cr0: u64;
    // SAFETY: this function is the single boot-time CR0/CR4 math-state writer.
    // It runs before interrupts and before any SIMD-using code path.
    unsafe {
        core::arch::asm!("mov {}, cr0", out(reg) cr0, options(nomem, nostack, preserves_flags));
    }
    cr0 &= !(1 << 2); // clear EM: do not trap FPU/SIMD instructions
    cr0 |= 1 << 1; // set MP: monitor WAIT/FWAIT with TS
                   // SAFETY: see function safety contract above.
    unsafe {
        core::arch::asm!("mov cr0, {}", in(reg) cr0, options(nomem, nostack, preserves_flags));
    }

    if matches!(
        tier,
        SimdTier::Sse2 | SimdTier::Avx | SimdTier::Avx2 | SimdTier::Avx512
    ) {
        let mut cr4: u64;
        // SAFETY: see function safety contract above.
        unsafe {
            core::arch::asm!("mov {}, cr4", out(reg) cr4, options(nomem, nostack, preserves_flags));
        }
        cr4 |= (1 << 9) | (1 << 10); // OSFXSR | OSXMMEXCPT
                                     // SAFETY: see function safety contract above.
        unsafe {
            core::arch::asm!("mov cr4, {}", in(reg) cr4, options(nomem, nostack, preserves_flags));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{detect_features, SimdTier};

    #[test]
    fn simd_tier_names_are_boot_profile_strings() {
        assert_eq!(SimdTier::Scalar.as_str(), "Scalar");
        assert_eq!(SimdTier::Sse2.as_str(), "Sse2");
        assert_eq!(SimdTier::Avx.as_str(), "Avx");
        assert_eq!(SimdTier::Avx2.as_str(), "Avx2");
        assert_eq!(SimdTier::Avx512.as_str(), "Avx512");
    }

    #[test]
    fn x86_64_host_detects_sse2_or_better() {
        assert_ne!(detect_features(), SimdTier::Scalar);
    }
}
