# Current Mission

Mission: C7.M6 Step 1 Storage contracts and timeout model
Campaign: C7 M6 Storage Stage 1
Status: ready

## Objective

Execute M6 campaign Step 1 from `docs/campaigns/m6-storage-stage-1.md`: add
the storage contracts, diagnostics, and timeout model needed before real ATA
PIO port I/O.

## Scope

Allowed changes:

- `kernel/src/main.rs`
- `kernel/src/storage.rs`
- `docs/campaigns/m6-storage-stage-1.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- ATA PIO port I/O.
- QEMU disk fixture or harness changes.
- FAT32, append-only graph store, or write support.
- Merging to or pushing `main`.

## Acceptance Criteria

- `kernel/src/storage.rs` defines fixed-width storage diagnostics with backend,
  LBA, operation, status, and timeout-count evidence.
- Timeout behavior is deterministic and unit-tested without hardware.
- No write API is exposed by default.
- No graph-visible path-like storage identifier is introduced.

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
rustc --test kernel/src/storage.rs
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m6-storage-stage-1`. Memory-unsafe Rust remains
allowed at real storage hardware boundaries; Step 1 intentionally builds the
bounded timeout/error contract first so later unsafe ATA PIO access has a
deterministic failure surface.
