# Current Mission

Mission: C10.M9 Step 2 Section bounds and checksum validation
Campaign: C10 M9 UMDL Loader
Status: ready

## Objective

Execute M9 campaign Step 2 from `docs/campaigns/m9-umdl-loader.md`: prove UMDL
section ranges are finite, non-overlapping where required, and covered by
deterministic checksums.

## Scope

Allowed changes:

- `crates/umdl/src/lib.rs`
- `docs/campaigns/m9-umdl-loader.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Tensor descriptor parsing, tokenizer metadata parsing, arena reservation,
  smoke target, or fixture work.
- LLM sampler, SIMD kernels, storage, QEMU harness, or graph mutation changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- Tokenizer, tensor, weight, and checksum section ranges are checked against
  input byte length with overflow-safe arithmetic.
- Header and section checksum mismatches return structured errors.
- Section validation tests cover out-of-bounds and checksum failure cases.
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
added little-endian header parsing and malformed-header tests.
