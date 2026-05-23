//! Boot sequence — spec §3.2 (kernel entry contract) and §1.6
//! (boot-visible heartbeat). Boot is never blind.
//!
//! The 14 source-level step assertions from §3.2 are kept in the body for
//! traceability. Steps not yet implemented carry `// TODO M<n>` markers per
//! CLAUDE.md §11 (no silent placeholders); the heartbeat lines themselves are
//! emitted in §1.6 order.

use crate::{arena, cpu, framebuffer, heartbeat, idt, multiboot2, operator_shell, storage};
use graph::{
    graph_compile_verified, graph_load_from_umod, GraphDisplayState, SOURCE_TRANSFORM_SINK_UMOD,
};

#[derive(Copy, Clone)]
pub struct BootHandoff {
    pub multiboot_magic: u32,
    pub multiboot_info_addr: u32,
}

unsafe extern "C" {
    static __kernel_end: u8;
}

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
pub unsafe fn run(handoff: BootHandoff) -> ! {
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
    // The current bootable image is GRUB/Multiboot2; Limine remains the
    // spec-primary target for the later bootloader handoff milestone.
    let memory_summary = read_boot_memory_summary(handoff);
    let framebuffer_info = memory_summary.framebuffer;
    heartbeat::emit_kv_hex("UNBOUNDOS_MEMMAP_OK", memory_summary.usable_bytes);
    let mut m2_arenas = emit_m2_memory_diagnostics(memory_summary);

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
    // The early M2 gate now constructs bounded arenas from the ingested memory
    // map and performs a small allocation from each. The later frame allocator
    // still needs to own page-frame reservation and free-list/bitmap policy.
    exercise_m2_arenas(&mut m2_arenas);

    // spec §3.2 step 10: enable permitted SIMD/FPU state.
    // SAFETY: cpu::enable_math_features is the sole CR0/CR4/XCR0
    // writer. detect_features ran in step 9 above; tier is honest.
    unsafe {
        cpu::enable_math_features(tier);
    }

    // spec §3.2 step 12: initialize permanent kernel structures.
    initialize_permanent_kernel_structures(&mut m2_arenas);

    run_m6_storage_smoke_from_env();

    // spec §3.2 step 13: load or embed initial graph.
    // The initial graph enters through the only legal verifier/compile gate.
    let graph_state = initialize_initial_graph();

    // spec §3.2 step 11: initialize framebuffer if available.
    render_framebuffer_if_available(framebuffer_info, graph_state);

    // spec §3.2 step 14: enter orchestrator or IDE shell.
    // The initial interactive surface is a polling serial operator shell. It
    // dynamically exercises graph, LLM, retrieval, and assistant surfaces until
    // the full framebuffer IDE/orchestrator replaces it.

    heartbeat::emit("UNBOUNDOS_BOOT_OK");
    qemu_exit_on_boot_ok_for_smoke();
    operator_shell::run(tier)
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

fn run_m6_storage_smoke_from_env() {
    if option_env!("UNBOUNDOS_STORAGE_SMOKE") != Some("1") {
        return;
    }
    heartbeat::emit("UNBOUNDOS_STORAGE_READ_BEGIN");
    let Ok(request) = storage::ReadSectorRequest::new(0) else {
        heartbeat::emit("UNBOUNDOS_STORAGE_REQUEST_INVALID");
        return;
    };
    let mut sector = [0u16; storage::SECTOR_WORDS];
    // SAFETY: this path is compile-time selected only by `make
    // qemu-storage-smoke`, which attaches a deterministic raw disk fixture as
    // the primary ATA device. Boot is single-threaded, so no other storage
    // command can race the legacy ATA PIO port range.
    match unsafe {
        storage::ata_pio_read_sector(
            request,
            storage::TimeoutBudget::new(storage::DEFAULT_ATA_TIMEOUT_POLLS),
            &mut sector,
        )
    } {
        Ok(_) if storage::sector_starts_with(&sector, storage::M6_STORAGE_MARKER) => {
            heartbeat::emit("UNBOUNDOS_STORAGE_MARKER_OK");
        }
        Ok(_) => heartbeat::emit("UNBOUNDOS_STORAGE_MARKER_MISMATCH"),
        Err(err) => {
            let diagnostic = err.diagnostic();
            heartbeat::emit("UNBOUNDOS_STORAGE_READ_ERROR");
            heartbeat::emit_kv_hex("storage_status", u64::from(diagnostic.status_register));
            heartbeat::emit_kv_hex("storage_timeout_count", u64::from(diagnostic.timeout_count));
        }
    }
}

fn initialize_initial_graph() -> Option<GraphDisplayState> {
    heartbeat::emit("UNBOUNDOS_GRAPH_LOAD_BEGIN");
    if let Ok(handle) = graph_load_from_umod(SOURCE_TRANSFORM_SINK_UMOD).and_then(|verified| {
        graph_compile_verified(verified).map_err(|_| graph::GraphLoadError::BadSectionTable)
    }) {
        let state = handle.display_state();
        heartbeat::emit("UNBOUNDOS_GRAPH_OK");
        heartbeat::emit_kv_hex("graph_id", state.graph_id());
        heartbeat::emit_kv_hex("graph_nodes", u64::from(state.node_count()));
        heartbeat::emit_kv_hex("graph_wires", u64::from(state.wire_count()));
        if let Some(last_completed) = state.last_completed_node() {
            heartbeat::emit_kv_hex("graph_last_completed", u64::from(last_completed));
        } else {
            heartbeat::emit_kv_str("graph_last_completed", "none");
        }
        Some(state)
    } else {
        heartbeat::emit("UNBOUNDOS_GRAPH_LOAD_ERROR");
        None
    }
}

fn render_framebuffer_if_available(
    info: Option<multiboot2::FramebufferInfo>,
    graph_state: Option<GraphDisplayState>,
) {
    let Some(info) = info else {
        heartbeat::emit_kv_str("UNBOUNDOS_FRAMEBUFFER", "unavailable");
        heartbeat::finalize_framebuffer_fallback(None);
        return;
    };
    if info.addr >= multiboot2::IDENTITY_MAPPED_LIMIT_4G {
        heartbeat::emit_kv_str("UNBOUNDOS_FRAMEBUFFER", "outside_identity_map");
        heartbeat::finalize_framebuffer_fallback(None);
        return;
    }
    let Some(pixel_count) = usize::try_from(info.pitch)
        .ok()
        .and_then(|pitch| pitch.checked_mul(usize::try_from(info.height).ok()?))
        .map(|bytes| bytes / core::mem::size_of::<u32>())
    else {
        heartbeat::emit_kv_str("UNBOUNDOS_FRAMEBUFFER", "invalid_geometry");
        heartbeat::finalize_framebuffer_fallback(None);
        return;
    };

    // SAFETY: Multiboot2 supplied a 32-bpp RGB framebuffer tag and the bootstrap
    // identity map covers the first 4 GiB. The surface is used only on the boot
    // CPU before interrupts/concurrency.
    let pixels =
        unsafe { core::slice::from_raw_parts_mut(info.addr as usize as *mut u32, pixel_count) };
    let stride_pixels = usize::try_from(info.pitch).unwrap_or(0) / core::mem::size_of::<u32>();
    let Ok(mut surface) = framebuffer::TextSurface::new(
        pixels,
        usize::try_from(info.width).unwrap_or(0),
        usize::try_from(info.height).unwrap_or(0),
        stride_pixels,
        0x00ff_ff00,
        0x0000_0000,
    ) else {
        heartbeat::emit_kv_str("UNBOUNDOS_FRAMEBUFFER", "surface_rejected");
        heartbeat::finalize_framebuffer_fallback(None);
        return;
    };

    surface.clear();
    surface.write_str("UNBOUNDOS_FRAMEBUFFER_RENDERED\n");
    heartbeat::finalize_framebuffer_fallback(Some(&mut surface));
    if let Some(state) = graph_state {
        surface.render_graph_state(
            state.graph_id(),
            state.node_count(),
            state.wire_count(),
            state.active_node(),
            state.last_completed_node(),
        );
    }

    heartbeat::emit("UNBOUNDOS_FRAMEBUFFER_OK");
    heartbeat::emit_kv_hex("framebuffer_addr", info.addr);
    heartbeat::emit_kv_hex("framebuffer_width", u64::from(info.width));
    heartbeat::emit_kv_hex("framebuffer_height", u64::from(info.height));
    heartbeat::emit("UNBOUNDOS_FRAMEBUFFER_RENDERED");
}

fn read_boot_memory_summary(handoff: BootHandoff) -> multiboot2::MemorySummary {
    let kernel_end = kernel_reserved_end();
    // SAFETY: `_mb2_start` preserves GRUB's Multiboot2 EAX/EBX handoff
    // registers and passes them to `_start`. The smoke image identity-maps the
    // first GiB, so the info block and selected early arenas are readable.
    unsafe {
        multiboot2::summarize_raw(
            handoff.multiboot_magic,
            handoff.multiboot_info_addr,
            kernel_end,
            multiboot2::IDENTITY_MAPPED_LIMIT,
        )
    }
}

fn kernel_reserved_end() -> u64 {
    // linker.ld defines __kernel_end at the aligned end of the loaded image.
    // Taking its address does not dereference memory.
    let raw = (&raw const __kernel_end) as u64;
    align_up(raw + 0x10000, 4096)
}

fn emit_m2_memory_diagnostics(summary: multiboot2::MemorySummary) -> Option<arena::M2ArenaSet> {
    heartbeat::emit("UNBOUNDOS_M2_MEMORY_DUMP_BEGIN");
    heartbeat::emit_kv_str(
        "m2_memmap_status",
        if summary.handoff_valid && summary.memmap_present {
            "available"
        } else {
            "unavailable"
        },
    );
    heartbeat::emit_kv_hex("m2_memmap_usable_bytes", summary.usable_bytes);
    heartbeat::emit_kv_hex("m2_memmap_region_count", u64::from(summary.usable_regions));
    heartbeat::emit_kv_hex("m2_multiboot_info_size", u64::from(summary.total_size));
    heartbeat::emit_kv_str("m2_arena_boot", arena::BOOT_ARENA.name);
    heartbeat::emit_kv_str("m2_arena_kernel", arena::KERNEL_ARENA.name);
    heartbeat::emit_kv_str("m2_arena_graph", arena::GRAPH_ARENA.name);
    heartbeat::emit_kv_str("m2_arena_scratch", arena::SCRATCH_ARENA.name);
    heartbeat::emit_kv_str("m2_arena_model_weight", arena::MODEL_WEIGHT_ARENA.name);
    heartbeat::emit_kv_str("m2_arena_inference", arena::INFERENCE_ARENA.name);
    heartbeat::emit_kv_str("m2_arena_kv_cache", arena::KV_CACHE_ARENA.name);
    heartbeat::emit_kv_str("m2_arena_tokenizer", arena::TOKENIZER_ARENA.name);

    let arenas = summary
        .arena_region
        .and_then(|region| m2_arena_regions_from(region.base).ok())
        .and_then(|regions| arena::M2ArenaSet::new(regions).ok());

    if let Some(ref arena_set) = arenas {
        emit_arena_initialized("m2_arena_boot", arena_set.boot());
        emit_arena_initialized("m2_arena_kernel", arena_set.kernel());
        emit_arena_initialized("m2_arena_graph", arena_set.graph());
        emit_arena_initialized("m2_arena_scratch", arena_set.scratch());
        emit_arena_initialized("m2_arena_model_weight", arena_set.model_weight());
        emit_arena_initialized("m2_arena_inference", arena_set.inference());
        emit_arena_initialized("m2_arena_kv_cache", arena_set.kv_cache());
        emit_arena_initialized("m2_arena_tokenizer", arena_set.tokenizer());
    } else {
        heartbeat::emit_kv_str("m2_arena_boot_status", "uninitialized");
        heartbeat::emit_kv_str("m2_arena_kernel_status", "uninitialized");
        heartbeat::emit_kv_str("m2_arena_graph_status", "uninitialized");
        heartbeat::emit_kv_str("m2_arena_scratch_status", "uninitialized");
        heartbeat::emit_kv_str("m2_arena_model_weight_status", "uninitialized");
        heartbeat::emit_kv_str("m2_arena_inference_status", "uninitialized");
        heartbeat::emit_kv_str("m2_arena_kv_cache_status", "uninitialized");
        heartbeat::emit_kv_str("m2_arena_tokenizer_status", "uninitialized");
    }
    heartbeat::emit("UNBOUNDOS_M2_MEMORY_DUMP_END");
    arenas
}

fn exercise_m2_arenas(arenas: &mut Option<arena::M2ArenaSet>) {
    let Some(arenas) = arenas else {
        heartbeat::emit_kv_str("m2_allocator_status", "unavailable");
        return;
    };

    let boot = arenas.with_boot_arena(|a| a.alloc_aligned(16, 16));
    let kernel = arenas.with_kernel_arena(|a| a.alloc_aligned(16, 16));
    let graph = arenas.with_graph_arena(|a| a.alloc_aligned(16, 16));
    let scratch = arenas.with_scratch_arena(|a| a.alloc_aligned(16, 16));
    let model = arenas.with_model_weight_arena(|a| a.alloc_aligned(16, 16));
    let inference = arenas.with_inference_arena(|a| a.alloc_aligned(16, 16));
    let kv_cache = arenas.with_kv_cache_arena(|a| a.alloc_aligned(16, 16));
    let tokenizer = arenas.with_tokenizer_arena(|a| a.alloc_aligned(16, 16));
    if boot.is_ok()
        && kernel.is_ok()
        && graph.is_ok()
        && scratch.is_ok()
        && model.is_ok()
        && inference.is_ok()
        && kv_cache.is_ok()
        && tokenizer.is_ok()
    {
        heartbeat::emit_kv_str("m2_allocator_status", "alloc_smoke_ok");
    } else {
        heartbeat::emit_kv_str("m2_allocator_status", "alloc_smoke_failed");
    }
}

fn initialize_permanent_kernel_structures(arenas: &mut Option<arena::M2ArenaSet>) {
    heartbeat::emit("UNBOUNDOS_KERNEL_STRUCTURES_BEGIN");
    let Some(arenas) = arenas else {
        heartbeat::emit_kv_str("kernel_structures_status", "unavailable");
        heartbeat::emit("UNBOUNDOS_KERNEL_STRUCTURES_END");
        return;
    };

    let idt_registry = arenas.with_kernel_arena(|a| a.alloc_aligned(64, 64));
    let driver_registry = arenas.with_kernel_arena(|a| a.alloc_aligned(64, 64));
    let graph_registry = arenas.with_kernel_arena(|a| a.alloc_aligned(64, 64));
    let model_catalog = arenas.with_model_weight_arena(|a| a.alloc_aligned(64, 64));
    let tokenizer_table = arenas.with_tokenizer_arena(|a| a.alloc_aligned(64, 64));
    let inference_session = arenas.with_inference_arena(|a| a.alloc_aligned(64, 64));
    let kv_session = arenas.with_kv_cache_arena(|a| a.alloc_aligned(64, 64));

    if let (
        Ok(idt_registry),
        Ok(driver_registry),
        Ok(graph_registry),
        Ok(model_catalog),
        Ok(tokenizer_table),
        Ok(inference_session),
        Ok(kv_session),
    ) = (
        idt_registry,
        driver_registry,
        graph_registry,
        model_catalog,
        tokenizer_table,
        inference_session,
        kv_session,
    ) {
        heartbeat::emit_kv_str("kernel_structures_status", "initialized");
        heartbeat::emit_kv_hex("kernel_registry_idt", idt_registry as u64);
        heartbeat::emit_kv_hex("kernel_registry_driver", driver_registry as u64);
        heartbeat::emit_kv_hex("kernel_registry_graph", graph_registry as u64);
        heartbeat::emit_kv_hex("model_catalog_base", model_catalog as u64);
        heartbeat::emit_kv_hex("tokenizer_table_base", tokenizer_table as u64);
        heartbeat::emit_kv_hex("inference_session_base", inference_session as u64);
        heartbeat::emit_kv_hex("kv_session_base", kv_session as u64);
        heartbeat::emit("UNBOUNDOS_KERNEL_STRUCTURES_OK");
    } else {
        heartbeat::emit_kv_str("kernel_structures_status", "alloc_failed");
    }
    heartbeat::emit("UNBOUNDOS_KERNEL_STRUCTURES_END");
}

fn m2_arena_regions_from(base: u64) -> Result<arena::M2ArenaRegions, ()> {
    let boot = range_at(base, multiboot2::M2_BOOT_ARENA_BYTES)?;
    let kernel_base = base
        .checked_add(multiboot2::M2_BOOT_ARENA_BYTES)
        .ok_or(())?;
    let kernel = range_at(kernel_base, multiboot2::M2_KERNEL_ARENA_BYTES)?;
    let graph_base = kernel_base
        .checked_add(multiboot2::M2_KERNEL_ARENA_BYTES)
        .ok_or(())?;
    let graph = range_at(graph_base, multiboot2::M2_GRAPH_ARENA_BYTES)?;
    let scratch_base = graph_base
        .checked_add(multiboot2::M2_GRAPH_ARENA_BYTES)
        .ok_or(())?;
    let scratch = range_at(scratch_base, multiboot2::M2_SCRATCH_ARENA_BYTES)?;
    let model_base = scratch_base
        .checked_add(multiboot2::M2_SCRATCH_ARENA_BYTES)
        .ok_or(())?;
    let model_weight = range_at(model_base, multiboot2::M2_MODEL_WEIGHT_ARENA_BYTES)?;
    let inference_base = model_base
        .checked_add(multiboot2::M2_MODEL_WEIGHT_ARENA_BYTES)
        .ok_or(())?;
    let inference = range_at(inference_base, multiboot2::M2_INFERENCE_ARENA_BYTES)?;
    let kv_cache_base = inference_base
        .checked_add(multiboot2::M2_INFERENCE_ARENA_BYTES)
        .ok_or(())?;
    let kv_cache = range_at(kv_cache_base, multiboot2::M2_KV_CACHE_ARENA_BYTES)?;
    let tokenizer_base = kv_cache_base
        .checked_add(multiboot2::M2_KV_CACHE_ARENA_BYTES)
        .ok_or(())?;
    let tokenizer = range_at(tokenizer_base, multiboot2::M2_TOKENIZER_ARENA_BYTES)?;

    Ok(arena::M2ArenaRegions {
        boot,
        kernel,
        graph,
        scratch,
        model_weight,
        inference,
        kv_cache,
        tokenizer,
    })
}

fn range_at(base: u64, size: u64) -> Result<arena::ArenaRange, ()> {
    let base = usize::try_from(base).map_err(|_| ())?;
    let size = usize::try_from(size).map_err(|_| ())?;
    Ok(arena::ArenaRange { base, size })
}

fn emit_arena_initialized(prefix: &str, arena: &arena::Arena) {
    let status_key = status_key(prefix);
    let base_key = base_key(prefix);
    let size_key = size_key(prefix);
    heartbeat::emit_kv_str(status_key, "initialized");
    heartbeat::emit_kv_hex(base_key, arena.base() as u64);
    heartbeat::emit_kv_hex(size_key, arena.remaining() as u64);
}

fn status_key(prefix: &str) -> &'static str {
    match prefix {
        "m2_arena_boot" => "m2_arena_boot_status",
        "m2_arena_kernel" => "m2_arena_kernel_status",
        "m2_arena_graph" => "m2_arena_graph_status",
        "m2_arena_scratch" => "m2_arena_scratch_status",
        "m2_arena_model_weight" => "m2_arena_model_weight_status",
        "m2_arena_inference" => "m2_arena_inference_status",
        "m2_arena_kv_cache" => "m2_arena_kv_cache_status",
        "m2_arena_tokenizer" => "m2_arena_tokenizer_status",
        _ => "m2_arena_unknown_status",
    }
}

fn base_key(prefix: &str) -> &'static str {
    match prefix {
        "m2_arena_boot" => "m2_arena_boot_base",
        "m2_arena_kernel" => "m2_arena_kernel_base",
        "m2_arena_graph" => "m2_arena_graph_base",
        "m2_arena_scratch" => "m2_arena_scratch_base",
        "m2_arena_model_weight" => "m2_arena_model_weight_base",
        "m2_arena_inference" => "m2_arena_inference_base",
        "m2_arena_kv_cache" => "m2_arena_kv_cache_base",
        "m2_arena_tokenizer" => "m2_arena_tokenizer_base",
        _ => "m2_arena_unknown_base",
    }
}

fn size_key(prefix: &str) -> &'static str {
    match prefix {
        "m2_arena_boot" => "m2_arena_boot_size",
        "m2_arena_kernel" => "m2_arena_kernel_size",
        "m2_arena_graph" => "m2_arena_graph_size",
        "m2_arena_scratch" => "m2_arena_scratch_size",
        "m2_arena_model_weight" => "m2_arena_model_weight_size",
        "m2_arena_inference" => "m2_arena_inference_size",
        "m2_arena_kv_cache" => "m2_arena_kv_cache_size",
        "m2_arena_tokenizer" => "m2_arena_tokenizer_size",
        _ => "m2_arena_unknown_size",
    }
}

fn align_up(value: u64, alignment: u64) -> u64 {
    let mask = alignment - 1;
    (value + mask) & !mask
}
