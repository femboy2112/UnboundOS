# Current Mission

Mission: C5.M4 Step 5 Persistent UMOD compile path
Campaign: C5 M4 UMOD Loader
Status: ready

## Objective

Execute M4 campaign Step 5 from `docs/campaigns/m4-umod-loader.md`: compile a
valid persistent UMOD through the existing verified path into the private
runtime graph surface.

## Scope

Allowed changes:

- `crates/graph/src/lib.rs`
- `crates/graph/src/verifier.rs`
- `crates/graph/src/loader.rs`
- `tests/golden_graphs/**`
- `tests/fuzz_corpus/umod/**`
- `docs/campaigns/m4-umod-loader.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Public runtime graph construction changes.
- UI, storage, LLM, or boot changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- A minimal persistent UMOD represents source -> transform -> sink.
- The persistent UMOD verifies with `graph_load_from_umod` and compiles with
  `graph_compile_verified`.
- No public runtime constructor or test-only verifier bypass is added.
- Existing `graph_load_from_umod -> graph_compile_verified` gate remains intact.

## Baseline to verify

```
branch: campaign/m4-umod-loader
status: IN-PROGRESS
```

## Verification Commands

```bash
python3 scripts/status.py
python3 scripts/mission.py validate
cargo test -p umod
cargo test -p graph
python3 scripts/address_scan.py tests/golden_graphs tests/golden_models
python3 scripts/verify.py --mission current
make gates
```

## Notes

Campaign branch: `campaign/m4-umod-loader`. Step 4 completed checks 14-22 for
payloads, arena budget, model/resource refs, checksums, UI layout, constants,
scheduling, and opaque resource syntax.
