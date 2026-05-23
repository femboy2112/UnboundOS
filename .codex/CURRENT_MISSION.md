# Current Mission

Mission: C4.M3 Step 2 Private hardcoded graph runtime
Campaign: C4 M3 Embedded Graph
Status: ready

## Objective

Execute M3 campaign Step 2 from `docs/campaigns/m3-embedded-graph.md`:
implement a built-in source -> transform -> sink graph shape behind the
verified compile path.

## Scope

Allowed changes:

- `crates/graph/src/lib.rs`
- `crates/graph/src/loader.rs`
- `crates/graph/src/verifier.rs`
- `docs/campaigns/m3-embedded-graph.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Direct public `GraphRuntime` construction.
- Boot hook, storage, UI, LLM, or persistent artifact format expansion beyond
  the minimal symbolic built-in fixture.
- Merging to or pushing `main`.

## Acceptance Criteria

- A symbolic built-in graph payload passes `graph_load_from_umod`.
- `graph_compile_verified` builds private runtime structures for that verified
  graph.
- A graph-crate test executes source -> transform -> sink once.
- No public bypass around `graph_load_from_umod -> graph_compile_verified` is
  added.

## Baseline to verify

```
branch: campaign/m3-embedded-graph
status: IN-PROGRESS
```

## Verification Commands

```bash
python3 scripts/status.py
python3 scripts/mission.py validate
make fmt
make clippy
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m3-embedded-graph`. Step 1 added private epoch
readiness primitives and tests.
