# Current Mission

Mission: C5.M4 Step 1 UMOD parser header and resource refs
Campaign: C5 M4 UMOD Loader
Status: ready

## Objective

Execute M4 campaign Step 1 from `docs/campaigns/m4-umod-loader.md`: add
bounded parser primitives for UMOD headers and opaque resource references,
replacing parser stubs with typed errors.

## Scope

Allowed changes:

- `crates/umod/src/lib.rs`
- `crates/graph/src/verifier.rs`
- `docs/campaigns/m4-umod-loader.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Runtime graph construction changes.
- Fixture, fuzz corpus, UI, storage, LLM, or boot changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- UMOD header decode uses fixed-width little-endian reads, not pointer casts.
- Bad magic, unsupported version, short header, and bad header length return
  structured parser or graph-load errors.
- `parse_resource_ref` accepts only approved opaque resource syntax and rejects
  path-shaped references.
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

Campaign branch: `campaign/m4-umod-loader`. M4 must not add a developer-mode
or test-only graph runtime constructor.
