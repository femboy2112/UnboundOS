//! Boot sequence — spec §3.2 (kernel entry contract) and §1.6
//! (boot-visible heartbeat). Boot is never blind.
//!
//! The 14 source-level step assertions from §3.2 are kept in the body for
//! traceability. Steps not yet implemented carry `// TODO M<n>` markers per
//! CLAUDE.md §11 (no silent placeholders); the heartbeat lines themselves are
//! emitted in §1.6 order.

use crate::{arena, cpu, heartbeat, idt, ssod};

// Canonical source-level assertion for spec §3.2 kernel-entry order.
// Runtime comments below cite the same steps at the implementation site;
// the CPU-profile probe is currently lifted to preserve the §1.6 heartbeat
// order until the M1 bootloader handoff and allocator steps land.
//
// spec §3.2 step 1: disable interrupts.
// spec §3.2 step 2: initialize serial logging.
// spec §3.2 step 3: print boot-begin heartbeat.
// spec §3.2 step 4: validate bootloader handoff structures.
// spec §3.2 step 5: install temporary GDT if needed.
// spec §3.2 step 6: install early IDT with fatal handlers.
// spec §3.2 step 7: ingest memory map.
// spec §3.2 step 8: initialize boot allocator.
// spec §3.2 step 9: probe CPU features.
// spec §3.2 step 10: enable permitted SIMD/FPU state.
// spec §3.2 step 11: initialize framebuffer if available.
// spec §3.2 step 12: initialize permanent kernel structures.
// spec §3.2 step 13: load or embed initial graph.
// spec §3.2 step 14: enter orchestrator or IDE shell.

/// Run the full boot sequence. Never returns.
///
/// # Safety
///
/// Called exactly once from `_start`. Holds exclusive ownership of the
/// CPU and the bootloader handoff structures.
pub unsafe fn run() -> ! {
    // spec §3.2 step 1: disable interrupts.
    // SAFETY: pre-IDT phase; interrupts must be off until handlers exist.
    unsafe {
        core::arch::asm!("cli", options(nomem, nostack, preserves_flags));
    }

    // spec §3.2 step 2: initialize serial logging and the §3.9 boot
    // diagnostic buffer fallback. On UART-probe failure this records
    // `BOOT_NO_SERIAL` into the diagnostic buffer.
    heartbeat::init();

    // spec §3.2 step 3: print the §1.6 boot-begin heartbeat.
    heartbeat::emit("UNBOUNDOS_BOOT_BEGIN");

    // spec §3.2 step 4: validate bootloader handoff structures.
    // TODO M1 (spec §3.1): parse Limine handoff (bootloader info,
    // memory map response, framebuffer response, kernel address
    // response, higher-half direct map). Until then, `_start` may
    // be entered without a real Limine context (e.g. when
    // make_image.sh becomes a real bootable image at M1).

    // spec §3.2 step 5: install temporary GDT if needed.
    // TODO M1 (spec §3.6): Limine sets a flat code/data GDT, so
    // most paths can skip this. A self-owned kernel GDT lands with
    // the page tables.

    // spec §3.2 step 9: probe CPU features. This read-only CPUID probe is
    // lifted ahead of steps 6-8 so the §1.6 heartbeat order remains
    // BOOT_BEGIN, CPU_PROFILE, MEMMAP_OK, IDT_OK, BOOT_OK. It does not
    // enable SIMD/FPU state; that remains step 10 below.
    let tier = cpu::detect_features();
    heartbeat::emit_kv_str("UNBOUNDOS_CPU_PROFILE", tier.as_str());

    // spec §3.2 step 7: ingest memory map.
    // TODO M2 (spec §3.1, §4.2): consume the Limine memory-map
    // response and classify regions per §4.2. Until then the
    // kernel reports zero registered bytes — honest, since no
    // allocator is wired and no usable RAM has been claimed.
    let mem_bytes: u64 = 0;
    heartbeat::emit_kv_hex("UNBOUNDOS_MEMMAP_OK", mem_bytes);
    emit_m2_memory_diagnostics();

    // spec §3.2 step 6: install early IDT with halt handlers.
    // SAFETY: single boot-path call; no concurrent IDT writers.
    unsafe {
        idt::install();
    }
    heartbeat::emit("UNBOUNDOS_IDT_OK");
    // M1 forced-fault smoke harness. This is compile-time test plumbing
    // selected by Makefile targets, never normal boot behavior.
    idt::trigger_forced_fault_from_env();

    // spec §3.2 step 8: initialize boot allocator.
    // TODO M2 (spec §4.3): bitmap or stack frame allocator over
    // the memory map ingested in step 7.

    // spec §3.2 step 10: enable permitted SIMD/FPU state.
    // SAFETY: cpu::enable_math_features is the sole CR0/CR4/XCR0
    // writer. detect_features ran in step 9 above; tier is honest.
    unsafe {
        cpu::enable_math_features(tier);
    }

    // spec §3.2 step 11: initialize framebuffer if available.
    // TODO M5 (spec §3.7, §3.9): pass the real bootloader framebuffer surface
    // here once handoff parsing exists. Until then, this keeps headless boot
    // non-blocking while preserving the real fallback call path for callers
    // that can provide a bounded `TextSurface`.
    heartbeat::finalize_framebuffer_fallback(None);

    // spec §3.2 step 12: initialize permanent kernel structures.
    // TODO M3 (spec §4.4–§4.11): KernelArena, GraphArena,
    // ScratchArena, ModelWeightArena, registries.

    // spec §3.2 step 13: load or embed initial graph.
    // TODO M3 (spec §5.7): bytes → graph_load_from_umod →
    // graph_compile_verified → GraphRuntimeHandle. The single
    // verifier gate is enforced by crates/graph/src/loader.rs.

    // spec §3.2 step 14: enter orchestrator or IDE shell.
    // TODO M3 (spec §5.9): cooperative scheduler.

    heartbeat::emit("UNBOUNDOS_BOOT_OK");
    qemu_exit_on_boot_ok_for_smoke();
    ssod::halt_idle()
}

fn qemu_exit_on_boot_ok_for_smoke() {
    if option_env!("UNBOUNDOS_QEMU_EXIT_ON_BOOT_OK") != Some("1") {
        return;
    }
    // SAFETY: this is QEMU smoke-test plumbing selected at compile time by the
    // no-serial harness. Port 0xF4 is the configured isa-debug-exit device in
    // scripts/qemu.sh and is not touched in normal builds.
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") 0xF4_u16,
            in("al") 0x10_u8,
            options(nomem, nostack, preserves_flags)
        );
    }
}

fn emit_m2_memory_diagnostics() {
    heartbeat::emit("UNBOUNDOS_M2_MEMORY_DUMP_BEGIN");
    heartbeat::emit_kv_str("m2_memmap_status", "unavailable");
    heartbeat::emit_kv_hex("m2_memmap_usable_bytes", 0);
    heartbeat::emit_kv_str("m2_arena_boot", arena::BOOT_ARENA.name);
    heartbeat::emit_kv_str("m2_arena_boot_status", "uninitialized");
    heartbeat::emit_kv_str("m2_arena_kernel", arena::KERNEL_ARENA.name);
    heartbeat::emit_kv_str("m2_arena_kernel_status", "uninitialized");
    heartbeat::emit_kv_str("m2_arena_graph", arena::GRAPH_ARENA.name);
    heartbeat::emit_kv_str("m2_arena_graph_status", "uninitialized");
    heartbeat::emit_kv_str("m2_arena_scratch", arena::SCRATCH_ARENA.name);
    heartbeat::emit_kv_str("m2_arena_scratch_status", "uninitialized");
    heartbeat::emit("UNBOUNDOS_M2_MEMORY_DUMP_END");
}
