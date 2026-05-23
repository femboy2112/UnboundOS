# Current Mission

Mission: C3.M2 Step 4 Memory-map and arena dump
Campaign: C3 M2 Arena Memory
Status: ready

## Objective

Execute M2 campaign Step 4 from `docs/campaigns/m2-arena-memory.md`: make the
M2 diagnostic dump available on serial without claiming a full Limine handoff
if the M0 smoke boot path is still active.

## Scope

Allowed changes:

- `kernel/src/arena.rs`
- `kernel/src/boot.rs`
- `scripts/qemu.sh`
- `Makefile`
- `docs/campaigns/m2-arena-memory.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Graph runtime construction, storage, UI, or LLM behavior.
- Persistent artifact format changes.
- Claiming real Limine memory-map parsing unless it is actually implemented.
- Merging to or pushing `main`.

## Acceptance Criteria

- Serial boot output contains a stable M2 memory-map/arena diagnostic dump.
- If real bootloader memory-map ingestion is unavailable, the dump explicitly
  reports `unavailable` rather than pretending usable ranges exist.
- Normal `make qemu-headless` still reaches `UNBOUNDOS_BOOT_OK`.

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
make kernel
make qemu-headless
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m2-arena-memory`. The M0 smoke profile still lacks
real Limine memory-map ingestion, so honest unavailable-state diagnostics are
acceptable for Step 4.
