# Current Mission

Mission: C7.M6 Step 5 M6 completion audit
Campaign: C7 M6 Storage Stage 1
Status: ready

## Objective

Execute M6 campaign Step 5 from `docs/campaigns/m6-storage-stage-1.md`: close
M6 after raw-sector read, timeout behavior, QEMU smoke evidence, and
resource-boundary checks are reproducibly verified.

## Scope

Allowed changes:

- `MILESTONE_CATALOG.md`
- `docs/campaigns/m6-storage-stage-1.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Implementation changes.
- Script, fixture, verifier, parser, storage driver, boot path, QEMU harness,
  FAT32, append-only graph store, or write support.
- Merging to or pushing `main`.

## Acceptance Criteria

- M6 row in `MILESTONE_CATALOG.md` changes from `IN-PROGRESS` to `DONE`.
- Catalog version banner is bumped.
- `MILESTONE_CATALOG.md` change log records the M6 closeout.
- `docs/campaigns/m6-storage-stage-1.md` has a `## Closeout` section naming
  the Step 1-4 checkpoint commits.
- `make gates`, `make repo-state`, and `python3 scripts/verify.py --mission
  current` prove the closeout state.

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
make gates
make repo-state
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m6-storage-stage-1`. Step 4 added an aggregate
storage namespace guard and broadened UMOD resource tests so path-shaped
storage refs remain rejected above the storage adapter boundary.
