# Current Mission

Mission: C12.M11 Step 5 Assistant smoke evidence and gates
Campaign: C12 M11 IDE Assistant
Status: ready

## Objective

Execute M11 campaign Step 5 from `docs/campaigns/m11-ide-assistant.md`:
make assistant explanation and action-buffer evidence reproducible from
checkout.

## Scope

Allowed changes:

- `Makefile`
- `scripts/**`
- `crates/llm/src/assistant.rs`
- `crates/llm/src/**`
- `crates/graph/src/**`
- `kernel/src/ssod.rs`
- `docs/campaigns/m11-ide-assistant.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Graph mutation, storage behavior changes, QEMU harness changes beyond
  existing aggregate gates, thread/queue, eval, or execution-hook changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- `make assistant-smoke` or equivalent source-level check is available.
- Smoke proves graph explanation, SSOD explanation, action-buffer, unified
  request routing, and no-direct mutation evidence are source-reachable.
- Assistant smoke is wired into aggregate mission verification.
- Graph/QEMU aggregate gates remain green.

## Baseline to verify

```
branch: campaign/m11-ide-assistant
status: IN-PROGRESS
```

## Verification Commands

```bash
python3 scripts/status.py
python3 scripts/mission.py validate
make fmt
make clippy
make assistant-smoke
make gates
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m11-ide-assistant`. Step 4 added a single explicit
assistant explain request/response surface for graph and SSOD explanation, with
optional proposals routed only through `StructuredActionBuffer`. Memory-unsafe
Rust remains allowed by project identity, but M11 assistant explanation/action
buffer work should be safe, deterministic, bounded, and non-executing.
