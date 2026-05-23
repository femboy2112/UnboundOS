# Current Mission

Mission: C5.M4 Step 2 Section table bounds and structural checks
Campaign: C5 M4 UMOD Loader
Status: ready

## Objective

Execute M4 campaign Step 2 from `docs/campaigns/m4-umod-loader.md`: parse
section descriptors and make section-table, file-length, count-limit, and
overflow checks non-vacuous.

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

- Runtime graph construction changes.
- UI, storage, LLM, or boot changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- Section descriptors decode through fixed-width little-endian reads, not
  pointer casts.
- Section table offsets, lengths, overflows, out-of-file sections, and illegal
  overlaps return structured errors.
- Configured node and wire count limits are enforced before semantic checks.
- Existing `graph_load_from_umod -> graph_compile_verified` gate remains
  intact.

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

Campaign branch: `campaign/m4-umod-loader`. Step 1 added fixed-width UMOD
header parsing and opaque resource reference validation.
