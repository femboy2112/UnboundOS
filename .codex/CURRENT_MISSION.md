# Current Mission

Mission: C10.M9 Step 4 Model load view and arena reservation contract
Campaign: C10 M9 UMDL Loader
Status: ready

## Objective

Execute M9 campaign Step 4 from `docs/campaigns/m9-umdl-loader.md`: expose a
read-only loaded model view and explicit arena requirements without allocating
hidden storage.

## Scope

Allowed changes:

- `crates/umdl/src/lib.rs`
- `crates/llm/src/lib.rs`
- `docs/campaigns/m9-umdl-loader.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Smoke target, fixture, sampler, tensor kernel, or graph mutation work.
- LLM sampler, SIMD kernels, storage, QEMU harness, or graph mutation changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- A loaded-model view carries validated header, tokenizer, tensor-count, and
  byte-range metadata without copying hidden storage.
- Required model, scratch, and KV-cache reservation bytes are explicit.
- Minimum SIMD tier validates against an available tier argument.
- No tensor kernels are called and no graph mutation surface is introduced.
- No new unsafe code, allocation, host paths, or pointer fields are introduced.

## Baseline to verify

```
branch: campaign/m9-umdl-loader
status: IN-PROGRESS
```

## Verification Commands

```bash
python3 scripts/status.py
python3 scripts/mission.py validate
make fmt
make clippy
cargo test -p umdl
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m9-umdl-loader`. Memory-unsafe Rust remains allowed
by project identity, but UMDL persistent-format parsing should be safe,
fixed-width, deterministic, and free of host paths or raw pointers. Step 1
added little-endian header parsing and malformed-header tests. Step 2 added
overflow-safe section bounds, non-overlap checks, and deterministic checksum
validation. Step 3 added tokenizer metadata and tensor descriptor parsing and
validation.
