//! CPU feature detection and SIMD enable. Spec §3.3 (CPU feature
//! detection) and §3.4 (FPU/SIMD initialization).
//!
//! Detection (§3.3) is read-only: CPUID leaves 0x1, 0x7, 0xD, and
//! 0x80000001 are walked to populate `CpuFeatures`, and CPUID 0xD
//! sub-leaf 0 EAX:EDX gives the XSAVE-supported XCR0 mask without
//! needing CR4.OSXSAVE. Enable (§3.4) is the sole writer of CR0,
//! CR4, and XCR0; it consumes a `CpuFeatures` and returns the
//! achieved `SimdTier`. AVX is never assumed (CLAUDE.md H6).

use core::arch::x86_64::__cpuid_count;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
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

/// XCR0 component bits (Intel SDM Vol. 1 §13.1, Vol. 3 §2.6).
mod xcr0 {
    pub const X87: u64 = 1 << 0;
    pub const SSE: u64 = 1 << 1;
    pub const YMM: u64 = 1 << 2;
    pub const OPMASK: u64 = 1 << 5;
    pub const ZMM_HI256: u64 = 1 << 6;
    pub const HI16_ZMM: u64 = 1 << 7;

    pub const AVX: u64 = X87 | SSE | YMM;
    pub const AVX512: u64 = AVX | OPMASK | ZMM_HI256 | HI16_ZMM;
}

use xcr0::{AVX as XCR0_AVX, AVX512 as XCR0_AVX512};

/// Raw CPU feature bits read from CPUID. Construction is the only
/// legal entry point to `enable_math_features`; it carries the
/// XSAVE-supported XCR0 mask alongside the boolean flags so the
/// enable path does not re-issue CPUID.
//
// `clippy::struct_excessive_bools`: the struct is a CPUID feature
// bitset by design; each bool maps to a distinct architecturally
// defined CPUID bit. A bitflags type would erase the names that
// make the §3.3 / §3.4 enable code auditable.
#[allow(clippy::struct_excessive_bools)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CpuFeatures {
    pub long_mode: bool,
    pub sse: bool,
    pub sse2: bool,
    pub xsave: bool,
    pub osxsave: bool,
    pub avx: bool,
    pub avx2: bool,
    pub avx512f: bool,
    /// XCR0 bitmap of state components reported as supported by
    /// CPUID 0xD sub-leaf 0 EAX:EDX. Zero when XSAVE is unsupported.
    pub xsave_supported_mask: u64,
}

impl CpuFeatures {
    /// Tier the kernel intends to enable based on feature bits and
    /// the XSAVE-supported XCR0 mask. Mirrors the spec §3.4
    /// reference: AVX requires `X87|SSE|YMM` in the supported mask;
    /// AVX-512 additionally requires `OPMASK|ZMM_HI256|HI16_ZMM`.
    pub fn intended_tier(&self) -> SimdTier {
        if !(self.sse && self.sse2) {
            return SimdTier::Scalar;
        }
        if !(self.xsave && self.avx) {
            return SimdTier::Sse2;
        }
        if self.xsave_supported_mask & XCR0_AVX != XCR0_AVX {
            return SimdTier::Sse2;
        }
        if self.avx512f && (self.xsave_supported_mask & XCR0_AVX512 == XCR0_AVX512) {
            return SimdTier::Avx512;
        }
        if self.avx2 {
            return SimdTier::Avx2;
        }
        SimdTier::Avx
    }
}

/// Probe CPUID and return the populated feature set. Spec §3.3.
///
/// CPUID is a base `x86_64` instruction with no SIMD-feature
/// gating, so this is callable before CR4.OSFXSR / CR4.OSXSAVE are
/// set. Reads only; never writes a CR or XCR.
pub fn detect_features() -> CpuFeatures {
    // Leaf 0x80000001 EDX bit 29 = LM (long mode). Reaching this
    // function in 64-bit code already proves long mode, but the
    // CPUID bit is recorded for completeness.
    let ext1 = __cpuid_count(0x8000_0001, 0);
    let long_mode = (ext1.edx & (1 << 29)) != 0;

    // Leaf 0x1 EDX/ECX: SSE/SSE2/XSAVE/OSXSAVE/AVX.
    let leaf1 = __cpuid_count(0x1, 0);
    let sse = (leaf1.edx & (1 << 25)) != 0;
    let sse2 = (leaf1.edx & (1 << 26)) != 0;
    let xsave = (leaf1.ecx & (1 << 26)) != 0;
    let osxsave = (leaf1.ecx & (1 << 27)) != 0;
    let avx = (leaf1.ecx & (1 << 28)) != 0;

    // Leaf 0x7 sub-leaf 0 EBX: AVX2 (bit 5), AVX-512F (bit 16).
    let leaf7 = __cpuid_count(0x7, 0);
    let avx2 = (leaf7.ebx & (1 << 5)) != 0;
    let avx512f = (leaf7.ebx & (1 << 16)) != 0;

    // Leaf 0xD sub-leaf 0 EAX:EDX: XSAVE-supported XCR0 mask.
    // Only valid when XSAVE is reported by CPUID 0x1; reading it
    // otherwise yields garbage and is skipped.
    let xsave_supported_mask = if xsave {
        let leaf_d = __cpuid_count(0xD, 0);
        (u64::from(leaf_d.edx) << 32) | u64::from(leaf_d.eax)
    } else {
        0
    };

    CpuFeatures {
        long_mode,
        sse,
        sse2,
        xsave,
        osxsave,
        avx,
        avx2,
        avx512f,
        xsave_supported_mask,
    }
}

// CR0 / CR4 bits used by §3.4 (Intel SDM Vol. 3 §2.5).
const CR0_MP: u64 = 1 << 1;
const CR0_EM: u64 = 1 << 2;
const CR4_OSFXSR: u64 = 1 << 9;
const CR4_OSXMMEXCPT: u64 = 1 << 10;
const CR4_OSXSAVE: u64 = 1 << 18;

/// # Safety
///
/// Single CR0/CR4/XCR0 writer in the kernel. Must be called once
/// during boot, after `detect_features` and before any SIMD-using
/// code path. Caller asserts exclusive ownership of the CPU and
/// that no IRQ or graph code observes the partial enable sequence.
///
/// Returns the `SimdTier` actually enabled. On a conformant CPU
/// the return value equals `features.intended_tier()`; the boot
/// path debug-asserts that equality.
pub unsafe fn enable_math_features(features: CpuFeatures) -> SimdTier {
    let intended = features.intended_tier();

    if intended == SimdTier::Scalar {
        // No SSE/SSE2 → leave CR0/CR4 untouched; running soft-float.
        return SimdTier::Scalar;
    }

    // Step 1-2: CR0.EM = 0 (use FPU, not emulation), CR0.MP = 1
    // (track FPU state on context switch). Spec §3.4.
    // SAFETY: Ring-0 single writer; caller-asserted exclusivity.
    unsafe {
        let mut cr0 = read_cr0();
        cr0 &= !CR0_EM;
        cr0 |= CR0_MP;
        write_cr0(cr0);
    }

    // Step 3-4: CR4.OSFXSR + CR4.OSXMMEXCPT — enable FXSAVE/FXRSTOR
    // and SIMD FP exceptions. Spec §3.4.
    // SAFETY: same as above.
    let mut cr4 = unsafe { read_cr4() };
    cr4 |= CR4_OSFXSR | CR4_OSXMMEXCPT;

    if !(features.xsave && intended >= SimdTier::Avx) {
        // SSE2-only path: write CR4 and we're done. AVX needs XCR0.
        // SAFETY: Ring-0 single writer.
        unsafe { write_cr4(cr4) };
        return SimdTier::Sse2;
    }

    // Step 5: CR4.OSXSAVE only when CPUID reports XSAVE. Spec §3.4.
    cr4 |= CR4_OSXSAVE;
    // SAFETY: Ring-0 single writer.
    unsafe { write_cr4(cr4) };

    // Step 6-7: XSETBV with the intersection of intended XCR0 bits
    // and features.xsave_supported_mask. Spec §3.4 — never set an
    // XCR0 bit the CPU does not advertise as supported.
    let xcr0_target = match intended {
        SimdTier::Avx512 => XCR0_AVX512,
        SimdTier::Avx2 | SimdTier::Avx => XCR0_AVX,
        SimdTier::Sse2 | SimdTier::Scalar => unreachable!(),
    };
    let xcr0_value = xcr0_target & features.xsave_supported_mask;

    // SAFETY: OSXSAVE was just set; XSETBV is now legal. The mask
    // is a strict subset of supported bits, so XSETBV cannot #GP
    // for setting unsupported bits. Reserved bits remain zero.
    unsafe { xsetbv(0, xcr0_value) };

    // Step 8: derive achieved tier from what we actually enabled.
    // ZMM_HI256/HI16_ZMM/OPMASK presence in xcr0_value gates AVX-512;
    // YMM presence gates AVX/AVX2.
    if (xcr0_value & XCR0_AVX512) == XCR0_AVX512 && features.avx512f {
        SimdTier::Avx512
    } else if (xcr0_value & XCR0_AVX) == XCR0_AVX {
        if features.avx2 {
            SimdTier::Avx2
        } else {
            SimdTier::Avx
        }
    } else {
        // Supported mask did not actually contain X87|SSE|YMM after
        // all — fall back to SSE2 (CR4.OSFXSR is already set).
        SimdTier::Sse2
    }
}

#[inline]
unsafe fn read_cr0() -> u64 {
    let v: u64;
    // SAFETY: caller asserts Ring-0 exclusivity.
    unsafe {
        core::arch::asm!("mov {}, cr0", out(reg) v, options(nomem, nostack, preserves_flags));
    }
    v
}

#[inline]
unsafe fn write_cr0(v: u64) {
    // SAFETY: caller asserts Ring-0 exclusivity and a valid CR0 value.
    unsafe {
        core::arch::asm!("mov cr0, {}", in(reg) v, options(nomem, nostack, preserves_flags));
    }
}

#[inline]
unsafe fn read_cr4() -> u64 {
    let v: u64;
    // SAFETY: caller asserts Ring-0 exclusivity.
    unsafe {
        core::arch::asm!("mov {}, cr4", out(reg) v, options(nomem, nostack, preserves_flags));
    }
    v
}

#[inline]
unsafe fn write_cr4(v: u64) {
    // SAFETY: caller asserts Ring-0 exclusivity and a valid CR4 value.
    unsafe {
        core::arch::asm!("mov cr4, {}", in(reg) v, options(nomem, nostack, preserves_flags));
    }
}

#[inline]
unsafe fn xsetbv(xcr: u32, value: u64) {
    let lo = value as u32;
    let hi = (value >> 32) as u32;
    // SAFETY: caller asserts CR4.OSXSAVE is set and that `value`
    // contains only XCR0 bits the CPU advertises as supported.
    unsafe {
        core::arch::asm!(
            "xsetbv",
            in("ecx") xcr,
            in("eax") lo,
            in("edx") hi,
            options(nomem, nostack, preserves_flags),
        );
    }
}
