# `x86_64-unboundos.json` — sidecar rationale

JSON does not allow comments, so the per-field rationale that originally lived
inside the target spec is documented here.

## Why `"features": "-mmx,-sse,+soft-float"`

CPU is base x86_64. **SIMD is not statically enabled by the target spec.** The
runtime dispatch table in `crates/llm/src/dispatch.rs` selects a SIMD tier at
model-load time after CPUID + XCR0 probing in `kernel::cpu`. Adding
`+sse`, `+avx`, or `+avx2` here would bypass the loader-selected dispatch
and is forbidden by CLAUDE.md §2 H6 / §3 (forbidden patterns) / spec §2.3, §11.2.

`+soft-float` plus `-mmx,-sse` keeps the base ABI free of MMX and SSE register
use. The kernel then enables permitted SIMD via CR0 / CR4 / XCR0 manipulation
only after `kernel::cpu::detect_features` confirms support (spec §3.3, §3.4).

SIMD-using crates (notably `crates/llm` once kernels land) opt in per-function
via `#[target_feature]` on functions reachable only through the dispatch table.

## Why this file is at the repo root

`cargo build -p kernel --target x86_64-unboundos.json` resolves the path
relative to CWD. Keeping the JSON at the root makes `make kernel` and the
fidelity gate work without setting a `--target-dir` or env var.

## Required cargo flags

A custom JSON target requires `-Z build-std=core,alloc -Z json-target-spec`
on a nightly toolchain with `rust-src` installed. The pinned toolchain in
`rust-toolchain.toml` (`nightly-2026-04-15`) installs `rust-src` automatically.
