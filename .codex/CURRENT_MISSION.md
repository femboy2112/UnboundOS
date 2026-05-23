# Current Mission

Mission: C4.M3 Step 4 Active node diagnostics
Campaign: C4 M3 Embedded Graph
Status: ready

## Objective

Execute M3 campaign Step 4 from `docs/campaigns/m3-embedded-graph.md`: track
active node identity during graph execution and clear it after each node fires.

## Scope

Allowed changes:

- `crates/graph/src/lib.rs`
- `crates/graph/src/loader.rs`
- `kernel/src/ssod.rs`
- `docs/campaigns/m3-embedded-graph.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Direct public `GraphRuntime` construction.
- Boot hook, storage, UI, LLM, or persistent artifact format changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- Active node is set before each graph node fires and cleared afterward.
- Graph tests prove the last completed node and active-node clearing behavior.
- No external code can mutate graph state directly.

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
make gates
```

## Notes

Campaign branch: `campaign/m3-embedded-graph`. Step 3 proved fan-out epoch
readiness.
