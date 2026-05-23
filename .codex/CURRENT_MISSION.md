# Current Mission

Mission: C12.M11 Step 2 Graph explanation snapshot
Campaign: C12 M11 IDE Assistant
Status: ready

## Objective

Execute M11 campaign Step 2 from `docs/campaigns/m11-ide-assistant.md`:
produce deterministic text/data explanations from verified graph display state.

## Scope

Allowed changes:

- `crates/graph/src/lib.rs`
- `crates/llm/src/assistant.rs`
- `docs/campaigns/m11-ide-assistant.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- SSOD explanation, unified assistant surface, smoke target, graph mutation,
  storage, QEMU harness, thread/queue, eval, or execution-hook changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- Graph explanation input is read-only and derived from existing display
  snapshot data.
- Graph identity, node/wire counts, active node, and last completed node format
  into caller-provided output.
- No `GraphRuntime` constructor or graph mutation surface is introduced.

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
cargo test -p graph
cargo test -p llm
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m11-ide-assistant`. Memory-unsafe Rust remains
allowed by project identity, but M11 assistant explanation/action-buffer work
should be safe, deterministic, bounded, and non-executing. Step 1 added
fixed-width action proposal records and caller-owned `StructuredActionBuffer`
storage.
