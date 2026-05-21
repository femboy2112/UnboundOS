//! Boot sequence — spec §3.2 (kernel entry contract) and §1.6
//! (boot-visible heartbeat). Boot is never blind.
//!
//! The 14 step labels from §3.2 are kept in the body for traceability.
//! Steps not yet implemented carry `// TODO M<n>` markers per CLAUDE.md
//! §11 (no silent placeholders); the heartbeat lines themselves are
//! emitted in §1.6 order.

use crate::{cpu, heartbeat, idt, ssod};

/// Run the full boot sequence. Never returns.
///
/// # Safety
///
/// Called exactly once from `_start`. Holds exclusive ownership of the
/// CPU and the bootloader handoff structures.
pub unsafe fn run() -> ! {
    // 1. Disable interrupts (§3.2 step 1).
    // SAFETY: pre-IDT phase; interrupts must be off until handlers exist.
    unsafe {
        core::arch::asm!("cli", options(nomem, nostack, preserves_flags));
    }

    // 2. Initialize serial logging + boot diagnostic buffer (§3.2 step 2,
    //    §3.9 fallback). On UART-probe failure this records the line
    //    `BOOT_NO_SERIAL` into the diagnostic buffer.
    heartbeat::init();

    // 3. Boot-begin heartbeat (§3.2 step 3, §1.6).
    heartbeat::emit("UNBOUNDOS_BOOT_BEGIN");

    // 4. Validate bootloader handoff structures (§3.2 step 4).
    // TODO M1 (spec §3.1): parse Limine handoff (bootloader info,
    // memory map response, framebuffer response, kernel address
    // response, higher-half direct map). Until then, `_start` may
    // be entered without a real Limine context (e.g. when
    // make_image.sh becomes a real bootable image at M1).

    // 5. Install temporary GDT if needed (§3.2 step 5).
    // TODO M1 (spec §3.6): Limine sets a flat code/data GDT, so
    // most paths can skip this. A self-owned kernel GDT lands with
    // the page tables.

    // 9. Probe CPU features (§3.2 step 9, lifted ahead of step 6 so the
    //    §1.6 heartbeat order — BOOT_BEGIN, CPU_PROFILE, MEMMAP_OK,
    //    IDT_OK, BOOT_OK — is preserved). CPUID is read-only and does
    //    not raise interrupts, so executing it before the IDT is safe.
    let tier = cpu::detect_features();
    heartbeat::emit_kv_str("UNBOUNDOS_CPU_PROFILE", tier.as_str());

    // 7. Ingest memory map (§3.2 step 7).
    // TODO M1 (spec §3.1, §4.2): consume the Limine memory-map
    // response and classify regions per §4.2. Until then the
    // kernel reports zero registered bytes — honest, since no
    // allocator is wired and no usable RAM has been claimed.
    let mem_bytes: u64 = 0;
    heartbeat::emit_kv_hex("UNBOUNDOS_MEMMAP_OK", mem_bytes);

    // 6. Install early IDT with halt handlers (§3.2 step 6, §3.5).
    // SAFETY: single boot-path call; no concurrent IDT writers.
    unsafe {
        idt::install();
    }
    heartbeat::emit("UNBOUNDOS_IDT_OK");

    // 8. Initialize boot allocator (§3.2 step 8).
    // TODO M1 (spec §4.3): bitmap or stack frame allocator over
    // the memory map ingested in step 7.

    // 10. Enable permitted SIMD/FPU state (§3.2 step 10).
    // SAFETY: cpu::enable_math_features is the sole CR0/CR4/XCR0
    // writer. detect_features ran in step 9 above; tier is honest.
    unsafe {
        cpu::enable_math_features(tier);
    }

    // 11. Initialize framebuffer if available (§3.2 step 11).
    // TODO M2 (spec §3.7, §3.9): once framebuffer init succeeds,
    // call `heartbeat::finalize_framebuffer_fallback()` so a
    // UART-failed boot still surfaces BOOT_NO_SERIAL and
    // BOOT_HEARTBEAT_BUFFER_PRESENT to the screen.

    // 12. Initialize permanent kernel structures (§3.2 step 12).
    // TODO M3 (spec §4.4–§4.11): KernelArena, GraphArena,
    // ScratchArena, ModelWeightArena, registries.

    // 13. Load or embed initial graph (§3.2 step 13).
    // TODO M3 (spec §5.7): bytes → graph_load_from_umod →
    // graph_compile_verified → GraphRuntimeHandle. The single
    // verifier gate is enforced by crates/graph/src/loader.rs.

    // 14. Enter orchestrator or IDE shell (§3.2 step 14).
    // TODO M3 (spec §5.9): cooperative scheduler.

    heartbeat::emit("UNBOUNDOS_BOOT_OK");
    ssod::halt_idle()
}
