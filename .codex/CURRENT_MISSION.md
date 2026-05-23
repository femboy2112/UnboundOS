# Current Mission

Mission: C13.M12 Step 2 Local document index snapshot
Campaign: C13 M12 Local Retrieval
Status: ready

## Objective

Execute M12 campaign Step 2 from `docs/campaigns/m12-local-retrieval.md`:
represent a read-only local document index snapshot from fixed document
records.

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

- Document index snapshots are read-only views over caller-owned document
  records.
- Empty indexes, duplicate document IDs, and invalid document references return
  structured errors.
- Opaque document/resource IDs remain enforced above storage adapters.

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

Campaign branch: `campaign/m12-local-retrieval`. Step 1 added fixed-width
retrieval query, document reference, result, and caller-owned result buffer
contracts. Memory-unsafe Rust remains allowed by project identity, but M12
retrieval contracts should be safe, deterministic, bounded, and non-executing.
