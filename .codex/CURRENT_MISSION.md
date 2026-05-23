# Current Mission

Mission: C7.M6 Step 3 QEMU raw-sector smoke fixture
Campaign: C7 M6 Storage Stage 1
Status: ready

## Objective

Execute M6 campaign Step 3 from `docs/campaigns/m6-storage-stage-1.md`: prove
raw-sector read under QEMU with a deterministic disk image and finite timeout
behavior.

## Scope

Allowed changes:

- `Makefile`
- `scripts/**`
- `kernel/src/storage.rs`
- `kernel/src/boot.rs`
- `docs/campaigns/m6-storage-stage-1.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- FAT32, append-only graph store, or write support.
- Merging to or pushing `main`.

## Acceptance Criteria

- A deterministic raw disk fixture or generator provides a recognizable
  first-sector marker.
- QEMU boots with that fixture attached and asserts a storage heartbeat proving
  the marker was read.
- Existing headless heartbeat gates remain green.
- The smoke path preserves finite timeout behavior and structured failure
  diagnostics.

## Baseline to verify

```
branch: campaign/m6-storage-stage-1
status: IN-PROGRESS
```

## Verification Commands

```bash
python3 scripts/status.py
python3 scripts/mission.py validate
make fmt
make clippy
make qemu-storage-smoke
make gates
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m6-storage-stage-1`. Step 2 added the ATA PIO
read-sector primitive with explicit unsafe port-I/O boundaries and host-tested
command sequencing. Step 3 may add Makefile/script/QEMU smoke plumbing.
