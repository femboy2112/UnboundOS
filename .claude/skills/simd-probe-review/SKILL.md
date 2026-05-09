---
name: simd-probe-review
description: Review the CPUID and XCR0 enable code against spec §3.3 and §3.4. Verifies gating order, FPU/SIMD enable sequence, dispatch table integrity, and profile policy alignment. Delegates to the simd-dispatch-auditor subagent. Use after any change to CPU feature detection, FPU init, or tensor backend dispatch.
allowed-tools: Read, Grep, Bash, Task
---

# /simd-probe-review

Review the SIMD probe and dispatch path.

## Procedure

1. Invoke the `simd-dispatch-auditor` subagent on the kernel.

2. While that runs, gather the context:
   ```bash
   rg -n 'cpuid|CPUID' kernel/src
   rg -n 'XCR0|xgetbv|xsetbv|OSXSAVE|OSFXSR|OSXMMEXCPT' kernel/src
   rg -n 'enable_cpu_math_features|SimdTier|TensorKernelTable' kernel/src
   ```

3. Read the boot path and confirm the order:
   ```
   _start
       → disable interrupts
       → init serial
       → boot heartbeat
       → bootloader handoff
       → GDT
       → early IDT
       → memory map
       → boot allocator
       → CPU feature probe (CPUID)              ← step 9
       → enable_cpu_math_features               ← step 10
       → framebuffer
       → permanent kernel structures
       → graph load
       → orchestrator
   ```
   Any reordering that uses SIMD before CPUID/enable is a finding.

4. Read `enable_cpu_math_features` (or its named equivalent) and confirm:
   - clears `CR0.EM`, sets `CR0.MP`
   - sets `CR4.OSFXSR`, `CR4.OSXMMEXCPT`
   - sets `CR4.OSXSAVE` only when CPUID reports XSAVE
   - calls XGETBV/XSETBV only when OSXSAVE is set
   - enables only XCR0 bits supported AND required
   - separates AVX-512 / ZMM state from AVX / YMM

5. Read the dispatch table init and confirm:
   - selected from `CpuFeatureSet ∩ ModelRequirements`
   - selected tier logged to serial
   - debug-build assertion that active CPU feature flags match the selected table
     before executing a backend-specific primitive

6. Verify model rejection: a `.UMDL` declaring `requires_simd_avx2` MUST be rejected
   on a profile that supports only SSE2. Run the corresponding test or fixture if it
   exists.

7. Combine into one report from the subagent's output plus the spot checks. Final
   verdict: `PASS` or `FAIL` with required fixes citing spec sections.

## What to flag

- Any `*_avx2(`, `*_avx512(`, `*_sse2(` call outside dispatch init or test fixtures.
- Any path that sets a CR4 bit without first checking the corresponding CPUID bit.
- Any host-CPU-driven dispatch in cross-compile paths.
- Any silent fallback when a model's required tier is unmet (must be a structured
  rejection, not a tier downgrade).

This skill does not modify code. It reports.
