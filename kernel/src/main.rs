// UnboundOS kernel entry point.
//
// This is the bare-metal entry. It MUST follow spec section 3.2 strictly:
// disable interrupts, init serial, emit boot heartbeat, validate
// bootloader handoff, install GDT/IDT, ingest memory map, init boot
// allocator, probe CPU features, enable permitted SIMD, init framebuffer
// (optional), init permanent kernel structures, load initial graph,
// enter orchestrator.
//
// Boot is never blind. Every path either reaches `UNBOUNDOS_BOOT_OK` on
// serial, or — when UART is unavailable — records into the boot
// diagnostic buffer and prints `BOOT_NO_SERIAL` /
// `BOOT_HEARTBEAT_BUFFER_PRESENT` once the framebuffer is up.

#![no_std]
#![no_main]
#![forbid(unsafe_op_in_unsafe_fn)]
#![feature(abi_x86_interrupt)]
// TODO M0/M1 (spec §13): drop this allow once the remaining
// scaffolding types start being constructed. The CPU-feature path
// is now real (§3.3/§3.4), so all `SimdTier` variants are reached
// through `CpuFeatures::intended_tier` and `enable_math_features`.
// The remaining unused declarations are `ArenaId`, `AllocError`,
// the `boot_diag::snapshot` reader, `idt::is_installed`, etc., all
// staged ahead of their first use.
#![allow(dead_code)]
// Kernel idiom: bit-level descriptor pack/unpack relies on
// intentional truncating casts and lossy field widths. Pedantic
// floor stays denied for the rest of the workspace; this kernel
// crate carves out the cast lints.
#![allow(clippy::cast_possible_truncation, clippy::cast_lossless)]

use core::panic::PanicInfo;

mod arena;
mod boot;
mod boot_diag;
mod cpu;
mod heartbeat;
mod idt;
mod serial;
mod ssod;

/// Kernel entry point. Called by Limine after the CPU is in 64-bit long
/// mode. The handoff contract is defined in spec section 3.1.
///
/// # Safety
///
/// The bootloader has set up the CPU state per the Limine contract.
/// The function takes ownership of the CPU and never returns.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // SAFETY: this is the unique entry point. We hold exclusive access
    // to the CPU. Each substep is responsible for its own invariants;
    // see boot::run for the full sequence.
    unsafe { boot::run() }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // The panic path funnels into kernel_panic with a structured
    // diagnostic context. SSOD is the only legal exit from a panic.
    ssod::from_rust_panic(info)
}
