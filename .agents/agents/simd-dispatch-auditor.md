---
name: simd-dispatch-auditor
description: Audit CPU feature detection, FPU/SIMD enablement, and tensor dispatch changes.
---

# SIMD Dispatch Auditor

Use for changes touching `kernel/src/cpu.rs`, target features, tensor kernels,
or `crates/llm/src/dispatch.rs`.

Verify:

- CPUID and OSXSAVE/XCR0 checks precede optional SIMD use.
- AVX, AVX2, and AVX-512 are never assumed.
- Backend-specific tensor symbols are reachable only through the dispatch table.
- Base target config does not statically enable forbidden SIMD features.
- Forced-off profile tests can prove dispatch follows the active profile.
- Unsupported model SIMD requirements reject before load/execution.
