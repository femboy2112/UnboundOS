# Current Mission

Mission: C12.M11 Step 3 SSOD explanation snapshot
Campaign: C12 M11 IDE Assistant
Status: ready

## Objective

Execute M11 campaign Step 3 from `docs/campaigns/m11-ide-assistant.md`:
produce deterministic explanations from structured SSOD diagnostic records.

## Scope

Allowed changes:

- `kernel/src/ssod.rs`
- `crates/llm/src/assistant.rs`
- `docs/campaigns/m11-ide-assistant.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Unified assistant surface, smoke target, graph mutation, storage, QEMU
  harness, thread/queue, eval, or execution-hook changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- SSOD explanation input is read-only and derived from structured SSOD
  diagnostic fields.
- Reason/RIP/fault-family style information formats into caller-provided
  output.
- H10 remains intact: fatal diagnostics are not swallowed, weakened, or routed
  around the SSOD record.

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
cargo test -p llm
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m11-ide-assistant`. Step 2 added read-only graph
explanation snapshots and deterministic caller-buffer graph explanation text.
Memory-unsafe Rust remains allowed by project identity, but M11 assistant
explanation/action-buffer work should be safe, deterministic, bounded, and
non-executing.
