# Current Mission

Mission: C10.M9 Step 3 Tokenizer and tensor descriptor validation
Campaign: C10 M9 UMDL Loader
Status: ready

## Objective

Execute M9 campaign Step 3 from `docs/campaigns/m9-umdl-loader.md`: validate
tokenizer metadata and tensor descriptor tables without loading executable code
or backend-specific kernels.

## Scope

Allowed changes:

- `crates/umdl/src/lib.rs`
- `docs/campaigns/m9-umdl-loader.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Arena reservation, smoke target, fixture, sampler, or kernel work.
- LLM sampler, SIMD kernels, storage, QEMU harness, or graph mutation changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- `TokenizerMetadata` and `TensorDesc` entries parse from UMDL sections using
  fixed-width little-endian fields.
- Supported tokenizer metadata validates through the existing raw-byte
  contract.
- Tensor scalar/quant IDs, rank/dim shape, alignment, and weight-blob bounds
  return structured errors.
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
validation.
