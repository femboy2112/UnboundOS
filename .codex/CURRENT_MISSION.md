# Current Mission

Mission: C4.M3 Step 3 Fan-out execution proof
Campaign: C4 M3 Embedded Graph
Status: ready

## Objective

Execute M3 campaign Step 3 from `docs/campaigns/m3-embedded-graph.md`: prove
one producer output can be observed by multiple consumers without either
consumer erasing readiness for the other.

## Scope

Allowed changes:

- `crates/graph/src/lib.rs`
- `crates/graph/src/loader.rs`
- `docs/campaigns/m3-embedded-graph.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Direct public `GraphRuntime` construction.
- Boot hook, storage, UI, LLM, or persistent artifact format changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- A graph-crate test proves two consumers can observe the same produced wire
  epoch independently.
- Neither consumer observation clears readiness for the other.
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

Campaign branch: `campaign/m3-embedded-graph`. Step 2 added the private
source -> transform -> sink runtime test through the verified graph pipeline.
