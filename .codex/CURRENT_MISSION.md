# Current Mission

Mission: C5.M4 Step 7 M4 completion audit
Campaign: C5 M4 UMOD Loader
Status: ready

## Objective

Execute M4 campaign Step 7 from `docs/campaigns/m4-umod-loader.md`: close M4
after persistent UMOD parsing, 22-check verification, compile-path execution,
and fixture coverage are all reproducibly verified.

## Scope

Allowed changes:

- `MILESTONE_CATALOG.md`
- `docs/campaigns/m4-umod-loader.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Implementation changes.
- Fixture or verifier changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- M4 row in `MILESTONE_CATALOG.md` changes from `IN-PROGRESS` to `DONE`.
- Catalog version banner is bumped.
- `MILESTONE_CATALOG.md` change log records the M4 closeout.
- `docs/campaigns/m4-umod-loader.md` has a `## Closeout` section naming the
  Step 1-6 checkpoint commits.
- `make gates`, `make repo-state`, and `python3 scripts/verify.py --mission
  current` prove the closeout state.

## Baseline to verify

```
branch: campaign/m4-umod-loader
status: IN-PROGRESS
```

## Verification Commands

```bash
python3 scripts/status.py
python3 scripts/mission.py validate
make gates
make repo-state
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m4-umod-loader`. Step 6 registered the valid
source -> transform -> sink golden fixture, added malformed UMOD corpus cases
for the required failure families, and exercised the fixture set from graph
crate tests in the verification bundle.
