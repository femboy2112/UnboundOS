# Current Mission

Mission: C13.M12 Step 5 Assistant retrieval surface
Campaign: C13 M12 Local Retrieval
Status: ready

## Objective

Execute M12 campaign Step 5 from `docs/campaigns/m12-local-retrieval.md`:
connect local retrieval results to the assistant data surface.

## Scope

Allowed changes:

- `crates/llm/src/assistant.rs`
- `crates/llm/src/retrieval.rs`
- `docs/campaigns/m12-local-retrieval.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Filesystem access, host paths above storage adapters, graph mutation,
  storage behavior changes, QEMU harness changes, thread/queue, eval, or
  execution-hook changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- Assistant exposes an explicit retrieval request/response surface.
- Retrieval output remains explanatory context, not graph mutation.
- Optional proposed actions route only through `StructuredActionBuffer`.

## Baseline to verify

```
branch: campaign/m12-local-retrieval
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

Campaign branch: `campaign/m12-local-retrieval`. Step 4 added deterministic
context packing into caller-provided output, preserving document refs and
snippet boundaries while rejecting overflow without truncation. Memory-unsafe
Rust remains allowed by project identity, but M12 retrieval contracts should
be safe, deterministic, bounded, and non-executing.
