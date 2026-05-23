# Current Mission

Mission: C11.M10 Step 2 Dispatch-selected scalar kernel table
Campaign: C11 M10 Quantized Inference
Status: ready

## Objective

Execute M10 campaign Step 2 from `docs/campaigns/m10-quantized-inference.md`:
route graph-facing tensor calls through the loader-selected dispatch table,
initially selecting scalar kernels only.

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

- Deterministic token-step, streaming, smoke target, graph mutation, storage,
  or QEMU harness changes.
- LLM sampler, SIMD kernels, storage, QEMU harness, or graph mutation changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- Dispatch table exposes the first quantized projection entry through a table
  function pointer.
- Scalar implementations are selected for every available SIMD tier until real
  backend tiers exist.
- Tests prove callers can invoke the scalar projection through the table.
- Backend-specific symbols remain confined to `dispatch.rs` and `kernels/**`.

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
remains allowed by project identity. Step 1 added safe scalar quantized
projection contracts with caller-owned output buffers. Any later SIMD unsafe
must be isolated under `crates/llm/src/kernels` and selected only through
dispatch.
