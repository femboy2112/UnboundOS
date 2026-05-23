# Current Mission

Mission: C5.M4 Step 3 Node and wire semantic verifier checks
Campaign: C5 M4 UMOD Loader
Status: ready

## Objective

Execute M4 campaign Step 3 from `docs/campaigns/m4-umod-loader.md`: implement
graph topology checks for node resolution, wire endpoints, pin indices, wire
type compatibility, node type registration, and cycle rules.

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

- Node and wire descriptors needed for topology verification decode through
  fixed-width little-endian reads.
- Checks 7-13 return typed `GraphLoadError` variants for unresolved nodes,
  unresolved endpoints, out-of-range pins, type mismatch, unknown node type,
  undeclared capability, and unbroken cycles.
- The verifier performs no runtime allocation before all checks pass.
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

Campaign branch: `campaign/m4-umod-loader`. Step 2 added section descriptor
parsing, section-table bounds checks, overlap rejection, and node/wire count
limits.
