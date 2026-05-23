# Current Mission

Mission: C13.M12 Step 3 Deterministic retrieval ranking
Campaign: C13 M12 Local Retrieval
Status: ready

## Objective

Execute M12 campaign Step 3 from `docs/campaigns/m12-local-retrieval.md`:
return deterministic top-k local document matches into caller-provided output.

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

- Query matching and top-k ranking are deterministic with stable tie-breaking.
- Ranked results are written only into caller-provided output.
- Output overflow and unsupported query shapes return structured errors.

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

Campaign branch: `campaign/m12-local-retrieval`. Step 2 added read-only local
document index snapshots over caller-owned records, with validation for empty
indexes, duplicate refs, and invalid refs. Memory-unsafe Rust remains allowed
by project identity, but M12 retrieval contracts should be safe,
deterministic, bounded, and non-executing.
