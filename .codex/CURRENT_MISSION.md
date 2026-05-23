# Current Mission

Mission: C10.M9 Step 5 UMDL smoke fixtures and gates
Campaign: C10 M9 UMDL Loader
Status: ready

## Objective

Execute M9 campaign Step 5 from `docs/campaigns/m9-umdl-loader.md`: make UMDL
loader evidence reproducible from checkout.

## Scope

Allowed changes:

- `Makefile`
- `scripts/**`
- `crates/umdl/src/lib.rs`
- `crates/umdl/src/**`
- `tests/golden_models/**`
- `tests/fuzz_corpus/umdl/**`
- `docs/campaigns/m9-umdl-loader.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Sampler, tensor kernel, graph mutation, storage, or QEMU harness changes.
- LLM sampler, SIMD kernels, storage, QEMU harness, or graph mutation changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- Deterministic golden `.UMDL` fixture or fixture generator exists.
- Malformed corpus entry exists for a rejected UMDL package.
- `make umdl-smoke` proves loader source and fixture evidence are reachable.
- Aggregate mission verification runs UMDL smoke and `make gates` remains
  green.

## Baseline to verify

```
branch: campaign/m9-umdl-loader
status: IN-PROGRESS
```

## Verification Commands

```bash
python3 scripts/status.py
python3 scripts/mission.py validate
make umdl-smoke
make gates
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m9-umdl-loader`. Memory-unsafe Rust remains allowed
by project identity, but UMDL persistent-format parsing should be safe,
fixed-width, deterministic, and free of host paths or raw pointers. Step 1
added little-endian header parsing and malformed-header tests. Step 2 added
overflow-safe section bounds, non-overlap checks, and deterministic checksum
validation. Step 3 added tokenizer metadata and tensor descriptor parsing and
validation. Step 4 added a read-only loaded model view, explicit arena
reservation accounting, and SIMD/profile budget validation.
