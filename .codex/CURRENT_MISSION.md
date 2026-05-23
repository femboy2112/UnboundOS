# Current Mission

Mission: C6.M5 Step 5 M5 completion audit
Campaign: C6 M5 Minimal UI
Status: completed

## Objective

Execute M5 campaign Step 5 from `docs/campaigns/m5-minimal-ui.md`: close M5
after framebuffer text output, boot-diagnostic fallback, graph-state display,
and smoke evidence are reproducibly verified.

## Scope

Allowed changes:

- `MILESTONE_CATALOG.md`
- `docs/campaigns/m5-minimal-ui.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Implementation changes.
- Script or Makefile changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- M5 row in `MILESTONE_CATALOG.md` changes from `IN-PROGRESS` to `DONE`.
- Catalog version banner is bumped.
- `MILESTONE_CATALOG.md` change log records the M5 closeout.
- `docs/campaigns/m5-minimal-ui.md` has a `## Closeout` section naming the
  Step 1-4 checkpoint commits.
- `make gates`, `make repo-state`, and `python3 scripts/verify.py --mission
  current` prove the closeout state.

## Baseline to verify

```
branch: campaign/m5-minimal-ui
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

Campaign branch: `campaign/m5-minimal-ui`. Step 4 added `make ui-smoke` and
`scripts/check_ui_smoke.py`, then wired the UI smoke into the aggregate mission
verifier without requiring graphical CI.

Stop reason: M5 campaign complete. Await operator action to open the final M5
PR or rotate mission state to M6.
