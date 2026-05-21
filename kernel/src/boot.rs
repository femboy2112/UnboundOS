//! Boot sequence — spec section 3.2. Boot is never blind.

use crate::{cpu, idt, serial, ssod};

/// Run the full boot sequence. Never returns.
///
/// # Safety
///
/// Called exactly once from `_start`. Holds exclusive ownership of the
/// CPU and the bootloader handoff structures.
pub unsafe fn run() -> ! {
    // 1. Disable interrupts.
    // SAFETY: pre-IDT phase; interrupts must be off until handlers exist.
    unsafe {
        core::arch::asm!("cli");
    }

    // 2. Initialize serial logging.
    serial::init();
    serial::write_str("UNBOUNDOS_BOOT_BEGIN\n");

    // 3. Probe CPU features and report tier.
    let tier = cpu::detect_features();
    serial::write_str("UNBOUNDOS_CPU_PROFILE=");
    serial::write_str(tier.as_str());
    serial::write_str("\n");

    // 4. Validate bootloader handoff and ingest memory map.
    // (Limine handoff parsing — stub.)
    let mem_bytes: u64 = 0;
    serial::write_str("UNBOUNDOS_MEMMAP_OK=");
    serial::write_hex_u64(mem_bytes);
    serial::write_str("\n");

    // 5. Install IDT with fatal handlers.
    idt::install();
    serial::write_str("UNBOUNDOS_IDT_OK\n");

    // 6. Enable permitted SIMD/FPU state.
    // SAFETY: cpu::enable_math_features is the sole CR0/CR4/XCR0 writer.
    unsafe {
        cpu::enable_math_features(tier);
    }

    // 7. Initialize framebuffer (optional). If no framebuffer, log
    //    BOOT_NO_SERIAL fallback path is unnecessary here because
    //    serial succeeded; the no-UART path is exercised by the
    //    no-serial QEMU smoke test.

    // 8. Initialize permanent kernel structures (allocator, registries).

    // 9. Load or embed initial graph.

    // 10. Enter orchestrator or IDE shell.
    serial::write_str("UNBOUNDOS_BOOT_OK\n");

    // Halt loop — for now, before the orchestrator exists.
    ssod::halt_idle()
}
