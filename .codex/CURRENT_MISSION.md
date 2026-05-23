# Current Mission

Mission: C4.M3 Step 1 Runtime epoch readiness primitives
Campaign: C4 M3 Embedded Graph
Status: ready

## Objective

Execute M3 campaign Step 1 from `docs/campaigns/m3-embedded-graph.md`: add
private runtime wire/consumer epoch observation primitives with tests that prove
readiness is `wire_epoch > last_observed_epoch`.

## Scope

Allowed changes:

- `crates/graph/src/lib.rs`
- `crates/graph/src/loader.rs`
- `scripts/verify.py`
- `docs/campaigns/m3-embedded-graph.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Direct public `GraphRuntime` construction.
- Storage, UI, LLM, or persistent artifact format changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- Runtime epoch observation primitives are private to the graph crate/loader
  surface.
- Tests prove readiness before observation, not ready after observation, and
  ready again after producer epoch increment.
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

Campaign branch: `campaign/m3-embedded-graph`. M3 must not add a developer-mode
or test-only graph runtime constructor.
