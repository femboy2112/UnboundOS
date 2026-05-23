# Current Mission

Mission: C13.M12 Step 4 Context packing
Campaign: C13 M12 Local Retrieval
Status: ready

## Objective

Execute M12 campaign Step 4 from `docs/campaigns/m12-local-retrieval.md`:
pack retrieved document snippets into bounded assistant context.

## Scope

Allowed changes:

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

- Retrieved document snippets pack deterministically into caller-provided byte
  output.
- Packed context preserves document IDs and snippet boundaries.
- Output overflow rejects without silent truncation.

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

Campaign branch: `campaign/m12-local-retrieval`. Step 3 added deterministic
top-k retrieval ranking with stable tie-breaking, caller-owned result output,
and structured overflow/unsupported-query errors. Memory-unsafe Rust remains
allowed by project identity, but M12 retrieval contracts should be safe,
deterministic, bounded, and non-executing.
