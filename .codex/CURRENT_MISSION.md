# Current Mission

Mission: C11.M10 Step 5 Quantized inference smoke evidence and gates
Campaign: C11 M10 Quantized Inference
Status: ready

## Objective

Execute M10 campaign Step 5 from `docs/campaigns/m10-quantized-inference.md`:
make quantized inference evidence reproducible from checkout.

## Scope

Allowed changes:

- `Makefile`
- `scripts/**`
- `crates/llm/src/**`
- `docs/campaigns/m10-quantized-inference.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Graph mutation, storage, or QEMU harness changes.
- LLM sampler, SIMD kernels, storage, QEMU harness, or graph mutation changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- `make quantized-smoke` or equivalent source-level check exists.
- Smoke proves scalar kernels, dispatch routing, deterministic token step, and
  streaming tests are source-reachable.
- Aggregate mission verification runs quantized smoke and `make gates` remains
  green.

## Baseline to verify

```
branch: campaign/m10-quantized-inference
status: IN-PROGRESS
```

## Verification Commands

```bash
python3 scripts/status.py
python3 scripts/mission.py validate
make quantized-smoke
make gates
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m10-quantized-inference`. Memory-unsafe Rust
remains allowed by project identity. Step 1 added safe scalar quantized
projection contracts with caller-owned output buffers. Any later SIMD unsafe
must be isolated under `crates/llm/src/kernels` and selected only through
dispatch. Step 2 routed the first quantized projection through the dispatch
table with scalar implementations selected for all current tiers. Step 3 added
a deterministic next-token step from validated model metadata and
caller-provided logits/output buffers. Step 4 added explicit streaming state,
stream buffers, and stable token sequence tests without graph mutation
authority.
