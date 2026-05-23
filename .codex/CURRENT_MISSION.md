# Current Mission

Mission: C3.M2 Step 3 Deterministic exhaustion diagnostics
Campaign: C3 M2 Arena Memory
Status: ready

## Objective

Execute M2 campaign Step 3 from `docs/campaigns/m2-arena-memory.md`: ensure
arena exhaustion returns structured context and fatal boot/kernel exhaustion
can route through SSOD with arena identity.

## Scope

Allowed changes:

- `kernel/src/arena.rs`
- `kernel/src/ssod.rs`
- `docs/campaigns/m2-arena-memory.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Graph runtime construction, storage, UI, or LLM behavior.
- Persistent artifact format changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- Exhaustion diagnostics include arena id, requested size, alignment, base,
  cursor, and limit.
- Fatal arena diagnostics can be serialized through SSOD with explicit absent
  graph/model/node context while those systems do not exist.
- Existing M0 heartbeat and M1 forced-fault behavior remain intact.

## Baseline to verify

```
branch: campaign/m2-arena-memory
status: IN-PROGRESS
```

## Verification Commands

```bash
python3 scripts/status.py
python3 scripts/mission.py validate
make fmt
make clippy
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m2-arena-memory`. Step 2 materialized the M2 named
arena set and guard-style allocation methods.
