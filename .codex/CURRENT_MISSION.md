# Current Mission

Mission: C11.M10 Step 1 Scalar quantized kernel contracts
Campaign: C11 M10 Quantized Inference
Status: ready

## Objective

Execute M10 campaign Step 1 from `docs/campaigns/m10-quantized-inference.md`:
add scalar quantized kernel contracts and deterministic tests without touching
SIMD-specific backends.

## Scope

Allowed changes:

- `crates/llm/src/lib.rs`
- `crates/llm/src/dispatch.rs`
- `crates/llm/src/kernels/**`
- `docs/campaigns/m10-quantized-inference.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Dispatch routing changes beyond module exposure.
- SIMD-specific backend symbols, graph mutation, storage, or QEMU harness
  changes.
- LLM sampler, SIMD kernels, storage, QEMU harness, or graph mutation changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- Scalar quantized kernel module exists with caller-provided buffer contracts.
- Deterministic tiny quantized projection tests pass.
- No unsafe code or backend-specific SIMD symbols are introduced.

## Baseline to verify

```
branch: campaign/m10-quantized-inference
status: IN-PROGRESS
```

## Verification Commands

```bash
python3 scripts/status.py
python3 scripts/mission.py validate
make fmt
make clippy
cargo test -p llm
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m10-quantized-inference`. Memory-unsafe Rust
remains allowed by project identity, but Step 1 starts with safe scalar kernel
contracts. Any later SIMD unsafe must be isolated under `crates/llm/src/kernels`
and selected only through dispatch.
