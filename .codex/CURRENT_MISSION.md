# Current Mission

Mission: C13.M12 Step 1 Retrieval data contracts
Campaign: C13 M12 Local Retrieval
Status: ready

## Objective

Execute M12 campaign Step 1 from `docs/campaigns/m12-local-retrieval.md`:
add fixed-width retrieval query, document reference, and result records.

## Scope

Allowed changes:

- `crates/llm/src/lib.rs`
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

- `crates/llm` exposes a retrieval module.
- Retrieval query, document reference, and result records use fixed-width
  bounded fields and caller-owned buffers.
- Host paths, `local://`, and oversized text are rejected deterministically.
- No unsafe code, filesystem access, thread/queue, eval, execution hook, or
  graph mutation surface is introduced.

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

Campaign branch: `campaign/m12-local-retrieval`. M11 completed at `8f91be3`.
Memory-unsafe Rust remains allowed by project identity, but M12 retrieval
contracts should be safe, deterministic, bounded, and non-executing.
