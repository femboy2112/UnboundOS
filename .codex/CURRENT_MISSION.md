# Current Mission

Mission: C5.M4 Step 4 Capabilities, resources, constants, and scheduling checks
Campaign: C5 M4 UMOD Loader
Status: ready

## Objective

Execute M4 campaign Step 4 from `docs/campaigns/m4-umod-loader.md`: complete
checks 12 and 14-22 for capabilities, payload bounds, GraphArena budget,
model/resource references, checksums, UI layout, constants, and deterministic
scheduling requirements.

## Scope

Allowed changes:

- `crates/umod/src/lib.rs`
- `crates/graph/src/lib.rs`
- `crates/graph/src/verifier.rs`
- `tests/fuzz_corpus/umod/**`
- `docs/campaigns/m4-umod-loader.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Runtime graph construction or allocation changes.
- UI, storage, LLM, or boot changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- Checks 14-22 are non-vacuous and return typed `GraphLoadError` variants.
- External references delegate to the approved opaque resource grammar.
- Malformed payload sizes, resource refs, constants, UI layout, checksums, and
  deterministic scheduling requirements do not panic.
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
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m4-umod-loader`. Step 3 added node/wire descriptor
parsing plus topology, pin, type, node-type, capability, and cycle checks.
