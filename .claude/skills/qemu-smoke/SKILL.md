---
name: qemu-smoke
description: Build the kernel and boot it in QEMU, capturing serial output. Asserts the spec §1.6 heartbeat sequence and the §12.4 minimum smoke tests pass. Use after any change to boot, IDT, allocator, framebuffer, or scheduler code.
allowed-tools: Bash, Read
---

# /qemu-smoke

Boot the kernel in QEMU and walk the spec §12.4 smoke checklist.

## Procedure

1. Clean build for the custom target:
   ```bash
   cargo build -p kernel \
       --target x86_64-unboundos.json \
       -Z build-std=core,alloc
   ```

2. Boot in QEMU with serial captured to a log:
   ```bash
   make qemu-smoke 2>&1 | tee build/qemu-smoke.log
   ```
   If `make qemu-smoke` is not yet wired, fall back:
   ```bash
   qemu-system-x86_64 \
       -machine q35 \
       -m 256M \
       -no-reboot -no-shutdown \
       -serial stdio \
       -display none \
       -kernel build/kernel.elf \
       2>&1 | tee build/qemu-smoke.log
   ```

3. Parse `build/qemu-smoke.log` and assert each spec §12.4 line:

   | Test | Pass condition |
   |------|----------------|
   | Boot banner | `UNBOUNDOS_BOOT_BEGIN` present |
   | CPU profile | `UNBOUNDOS_CPU_PROFILE=` present, value non-empty |
   | Memory map | `UNBOUNDOS_MEMMAP_OK=` present, byte count > 0 |
   | IDT install | `UNBOUNDOS_IDT_OK` present |
   | Boot complete | `UNBOUNDOS_BOOT_OK` present |
   | Allocator | aligned alloc returns valid 32-byte boundary (test output) |
   | IDT divide | intentional divide-by-zero enters SSOD |
   | Page fault | bad pointer prints CR2 |
   | Graph load | embedded test graph verifies |
   | Scheduler | source → transform → sink graph executes once |
   | Fan-out | one producer reaches two consumers |
   | SIMD detection | active SIMD tier matches CPUID result |
   | .MOD reject | malformed graph is rejected, not executed |
   | .UMDL reject | malformed model package is rejected, not loaded |
   | Tokenizer | text → tokens → text round trip succeeds |
   | Tiny LLM | fixed toy model emits deterministic token sequence |

4. Report:

   ```
   # QEMU Smoke — <date> — <commit>

   ## Heartbeat
   - UNBOUNDOS_BOOT_BEGIN: <ts>
   - UNBOUNDOS_CPU_PROFILE=<value>
   - UNBOUNDOS_MEMMAP_OK=<bytes>
   - UNBOUNDOS_IDT_OK: <ts>
   - UNBOUNDOS_BOOT_OK: <ts>

   ## Smoke checklist
   <table with PASS|FAIL|SKIP per line; SKIP only allowed for milestones not yet
    reached, with the M-number cited from spec §13>

   ## Diagnostics observed
   <any SSOD records, with the structured fields decoded>

   ## Verdict
   PASS | FAIL | EARLY (Mn) — <reason>
   ```

5. If a heartbeat string is missing, this is a `BLOCK`. Boot must never be blind
   (spec §1.6, §3.9). Even on a broken UART the kernel must print
   `BOOT_NO_SERIAL` / `BOOT_HEARTBEAT_BUFFER_PRESENT` once the framebuffer is up.

6. On any SSOD record in the log, decode it:
   ```bash
   /ssod-decode build/qemu-smoke.log
   ```

This skill does not commit, push, or modify code. It boots and reports.
