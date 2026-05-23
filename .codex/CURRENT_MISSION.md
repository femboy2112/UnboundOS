# Current Mission

Mission: C10.M9 Step 1 UMDL header parse and fixed-width contract
Campaign: C10 M9 UMDL Loader
Status: ready

## Objective

Execute M9 campaign Step 1 from `docs/campaigns/m9-umdl-loader.md`: parse a
UMDL header from bytes without allocation, pointers, host paths, or unsafe
code.

## Scope

Allowed changes:

- `crates/umdl/src/lib.rs`
- `docs/campaigns/m9-umdl-loader.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Section, tensor, checksum, arena reservation, smoke target, or fixture work.
- LLM sampler, SIMD kernels, storage, QEMU harness, or graph mutation changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- UMDL header parses from a caller-provided byte slice using little-endian
  fixed-width fields.
- Magic, supported format version, and minimum header length are validated.
- Malformed-header tests cover bad magic, short input/header, and unsupported
  version.
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
fixed-width, deterministic, and free of host paths or raw pointers.
