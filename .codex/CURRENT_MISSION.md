# Current Mission

Mission: C7.M6 Step 2 ATA PIO sector-read primitive
Campaign: C7 M6 Storage Stage 1
Status: ready

## Objective

Execute M6 campaign Step 2 from `docs/campaigns/m6-storage-stage-1.md`:
implement the spec §7.3 ATA PIO read sequence behind an explicit unsafe port
boundary and the Step 1 timeout/error contract.

## Scope

Allowed changes:

- `kernel/src/storage.rs`
- `kernel/src/boot.rs`
- `docs/campaigns/m6-storage-stage-1.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- QEMU disk fixture or harness changes.
- FAT32, append-only graph store, or write support.
- Merging to or pushing `main`.

## Acceptance Criteria

- ATA PIO read-sector code follows the spec §7.3 command sequence.
- Every port-I/O unsafe block has a local safety comment and routes through the
  finite timeout/error model.
- The read path fills exactly one caller-provided 512-byte sector buffer.
- Device errors and timeouts return structured diagnostics rather than
  panicking or looping forever.

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

Campaign branch: `campaign/m6-storage-stage-1`. Step 1 added the fixed-width
storage diagnostic surface and host-tested finite polling. Step 2 is allowed to
add unsafe ATA PIO port I/O, but only behind that bounded, inspectable, and
deterministic contract.
