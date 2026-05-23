# Current Mission

Mission: C12.M11 Step 1 Structured action buffer contract
Campaign: C12 M11 IDE Assistant
Status: ready

## Objective

Execute M11 campaign Step 1 from `docs/campaigns/m11-ide-assistant.md`:
replace the placeholder assistant action surface with a bounded data-only
proposal buffer.

## Scope

Allowed changes:

- `crates/llm/src/lib.rs`
- `crates/llm/src/assistant.rs`
- `docs/campaigns/m11-ide-assistant.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Graph explanation, SSOD explanation, smoke target, graph mutation, storage,
  QEMU harness, thread/queue, eval, or execution-hook changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- Fixed-width action proposal records and buffer metadata exist.
- Caller-provided storage and deterministic overflow errors are enforced.
- Proposals are data only and cannot mutate graph state.
- No unsafe code, threads, queues, eval, or execution hooks are introduced.

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

Campaign branch: `campaign/m11-ide-assistant`. Memory-unsafe Rust remains
allowed by project identity, but M11 assistant explanation/action-buffer work
should be safe, deterministic, bounded, and non-executing.
