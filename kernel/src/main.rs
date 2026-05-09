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
// TODO M0/M1 (spec §13): drop this allow once the boot path, IDT,
// and arena allocator stop being stubs and the SimdTier variants
// are constructed by `cpu::detect_features`. The scaffolding types
// (ArenaId, AllocError, the non-Scalar SimdTier variants) are
// declared ahead of their first use.
#![allow(dead_code)]

use core::panic::PanicInfo;

mod arena;
mod boot;
mod cpu;
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
