# Current Mission

Mission: C12.M11 Step 4 Assistant explanation surface
Campaign: C12 M11 IDE Assistant
Status: ready

## Objective

Execute M11 campaign Step 4 from `docs/campaigns/m11-ide-assistant.md`:
provide a single local assistant explain surface for graph and SSOD states.

## Scope

Allowed changes:

- `crates/llm/src/assistant.rs`
- `crates/llm/src/lib.rs`
- `docs/campaigns/m11-ide-assistant.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Smoke target, graph mutation, storage, QEMU harness, thread/queue, eval, or
  execution-hook changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- A single explicit assistant request/response surface routes graph and SSOD
  explanation requests.
- Proposed actions, if requested, can only land in `StructuredActionBuffer`.
- Unsupported request kinds return structured errors.

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

Campaign branch: `campaign/m11-ide-assistant`. Step 3 added fixed-width SSOD
explanation snapshots and deterministic caller-buffer SSOD explanation text.
Memory-unsafe Rust remains allowed by project identity, but M11 assistant
explanation/action-buffer work should be safe, deterministic, bounded, and
non-executing.
