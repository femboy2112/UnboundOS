# Current Mission

Mission: C3.M2 Step 1 Bounded arena core and alignment checks
Campaign: C3 M2 Arena Memory
Status: ready

## Objective

Execute M2 campaign Step 1 from `docs/campaigns/m2-arena-memory.md`: implement
the reusable bounded arena cursor contract with explicit alignment rejection
and overflow/exhaustion errors.

## Scope

Allowed changes:

- `kernel/src/arena.rs`
- `scripts/verify.py`
- `docs/campaigns/m2-arena-memory.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Boot integration, memory-map ingestion, graph, storage, UI, or LLM behavior.
- Persistent artifact format changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- `kernel/src/arena.rs` defines a bounded `Arena` cursor contract.
- `alloc_aligned(size, alignment)` rejects non-power-of-two alignments.
- Overflow and exhaustion return deterministic `AllocError` values with arena
  identity and request context.
- Verification covers alignment success, alignment rejection, overflow, and
  exhaustion.

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

Campaign branch: `campaign/m2-arena-memory`. Step 1 is an arena contract step
only; named arena boot integration starts in Step 2.
