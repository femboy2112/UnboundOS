# Spec Completion Audit

Date: 2026-05-23
Branch: `campaign/m12-local-retrieval`
Baseline: current tree after full arena initialization hardening

This audit is intentionally stricter than the milestone catalog. It records
where the current repo is dynamically verified and where `DONE` milestone rows
still represent thin prototype paths instead of fully fleshed-out OS behavior.

## Verified Now

- `make gates` passes 27/27, including live QEMU boots, the interactive serial
  shell, no-serial fallback, three SSOD fault vectors, M2 memory diagnostics,
  boot-time graph load, framebuffer memory inspection, storage, and the repeated
  QEMU stress and CPU/RAM matrix sweeps.
- `python3 scripts/verify.py --mission current` passes and runs the same QEMU
  stress sweep.
- `make qemu-stress` repeats 10 runtime paths twice by default:
  heartbeat, no-serial fallback, divide-error SSOD, invalid-opcode SSOD,
  page-fault SSOD, M2 memory dump, graph boot, framebuffer render, interactive
  graph/LLM/retrieval/assistant shell, and storage marker read.
- `make qemu-matrix` runs heartbeat, M2 arena/memory, graph boot, framebuffer,
  and interactive shell gates under low-RAM, baseline, larger-RAM, and `max`
  CPU QEMU profiles.
- The boot path now initializes and dynamically proves all spec §4.4 named
  early/model/inference arenas: BootArena, KernelArena, GraphArena,
  ScratchArena, ModelWeightArena, InferenceArena, KVCacheArena, and
  TokenizerArena. `make qemu-m2-dump` rejects missing arena names, missing
  initialized status markers, missing nonzero bases, missing allocation smoke,
  or missing `UNBOUNDOS_KERNEL_STRUCTURES_OK`.
- The memory-unsafety constraint is being treated correctly: unsafe Rust remains
  allowed at bounded hardware and boot-memory boundaries, with
  `#![forbid(unsafe_op_in_unsafe_fn)]` preserving explicit unsafe blocks. The
  current gates should not impose a safe-Rust-only policy.

## Remaining Spec Gaps

1. Limine handoff is not implemented.

   Spec §3.1 names Limine as the initial bootloader target and requires memory
   map, kernel file location, stack/handoff, framebuffer, bootloader revision,
   and optional HHDM information. The current bootable image is still a
   GRUB/Multiboot2 ISO assembled by `scripts/make_image.sh`, and the boot path
   still has `TODO M1` comments for Limine handoff and GDT ownership. This is
   the largest mismatch between `M1 DONE` and the spec.

2. The kernel entry contract is still partly smoke-path shaped.

   Spec §3.2 requires the entry path to validate bootloader handoff structures,
   install GDT if needed, initialize permanent kernel structures, initialize
   framebuffer before loading the graph, then enter the orchestrator or IDE
   shell. The current QEMU path validates Multiboot2 enough for memory and
   framebuffer, but permanent kernel structures are still marked TODO and the
   graph is loaded before framebuffer rendering so the framebuffer can display
   graph state. That is useful, but it is not a clean implementation of the
   written order.

3. The arena model still lacks paging-backed hardening.

   The repo now carves and allocation-smokes BootArena, KernelArena,
   GraphArena, ScratchArena, ModelWeightArena, InferenceArena, KVCacheArena,
   and TokenizerArena from the QEMU boot memory map. What remains incomplete is
   paging/read-only policy for model weights, guard pages or poisoning for
   debug reset discipline, and richer runtime diagnostics for phase violations.

4. The UI is framebuffer text plus serial shell, not a full framebuffer IDE.

   The framebuffer path is now real under QEMU and writes visible memory, but
   the usable operator interface remains the polling serial shell. The spec
   milestone language says the framebuffer IDE displays graph state; the current
   framebuffer displays diagnostics and graph state but does not yet provide a
   real IDE interaction surface.

5. The LLM path is dynamically exercised but still prototype-scale.

   The interactive QEMU shell exercises tokenizer, toy transformer, quantized
   kernels, retrieval, and assistant explanation. It does not yet prove a
   full graph-native model package lifecycle with read-only model weights or
   hardware-selected SIMD backends beyond the current scalar/SSE2 profile work.

6. Stress coverage is still short-run.

   `make qemu-stress` now catches single-shot flakiness by repeating all live
   runtime paths twice, and `make qemu-matrix` covers several CPU/RAM profiles.
   This is still not a long soak test, randomized parser fuzz run,
   storage-error matrix, framebuffer-mode matrix, or broad host-hardware matrix.

7. Stale closed-milestone TODOs and comments remain.

   Source still contains closed-milestone TODOs or stale comments in
   `kernel/src/boot.rs`, `kernel/src/main.rs`, `crates/graph/src/lib.rs`, and
   `crates/graph/src/verifier.rs`. Some are stale comments over implemented
   behavior; others point at real gaps. They should be either implemented,
   reworded as explicit hardening backlog, or used to reopen ownership.

## Next Hardening Order

1. Implement or explicitly gate the Limine boot path.
2. Add a real permanent kernel-structure initialization phase and make the boot
   order match spec §3.2 or document a spec-compatible reason for any reorder.
3. Expand arena ownership to ModelWeight, Inference, KVCache, and Tokenizer
   arenas with phase guards and diagnostics.
4. Turn the framebuffer path into an interactive IDE surface rather than only a
   rendered diagnostic surface.
5. Add remaining matrix stress gates for framebuffer mode, storage errors, and
   malformed persistent artifacts.
