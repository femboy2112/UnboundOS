# Current Mission

Mission: C11.M10 Step 4 Streaming token surface
Campaign: C11 M10 Quantized Inference
Status: ready

## Objective

Execute M10 campaign Step 4 from `docs/campaigns/m10-quantized-inference.md`:
stream deterministic tokens through an explicit callable surface.

## Scope

Allowed changes:

- `crates/llm/src/**`
- `docs/campaigns/m10-quantized-inference.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Smoke target, graph mutation, storage, or QEMU harness changes.
- LLM sampler, SIMD kernels, storage, QEMU harness, or graph mutation changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- Streaming state/config object uses explicit caller-owned buffers.
- Stable token sequences are produced for a tiny prompt/model/config.
- Generated tokens remain output data only with no graph mutation authority.

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
dispatch. Step 2 routed the first quantized projection through the dispatch
table with scalar implementations selected for all current tiers. Step 3 added
a deterministic next-token step from validated model metadata and
caller-provided logits/output buffers.
