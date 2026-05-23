# Current Campaign

Campaign: C7 M6 Storage Stage 1
Active mission: C7.M6 Step 3 QEMU raw-sector smoke fixture
Status: ready
Stop rule: stop after one complete mission unless the operator explicitly
approves a bundled run; bundled runs stop at the next review gate, blocker, or
failed verification.
Bundle policy: sequential only; run validation, commit, push, and reload
mission state after each completed mission.
Publish policy: commit and push the campaign branch after each completed
mission.
Main policy: never merge to main, never push main, or force-push.
Campaign branch: campaign/m6-storage-stage-1

## Campaign Objective

Close M6 by proving the spec §13.8 Storage stage 1 exit criterion: raw sector
read works with timeout while graph-visible storage references remain opaque.

## Active Mission

See `.codex/CURRENT_MISSION.md`.

## Macro Sequence

1. C7.M6 Step 1 Storage contracts and timeout model. Completed.
2. C7.M6 Step 2 ATA PIO sector-read primitive. Completed.
3. C7.M6 Step 3 QEMU raw-sector smoke fixture. Active.
4. C7.M6 Step 4 Resource namespace guard evidence. Pending.
5. C7.M6 Step 5 M6 completion audit. Pending.

## Required Preflight For `go`

1. Read `CLAUDE.md`.
2. Read `docs/campaigns/m6-storage-stage-1.md`.
3. Read `docs/UnboundOS_Tech_Spec_v2_1_1_Fidelity_Hardening.pdf` or the
   extracted requirements in `.codex/PROJECT_PLAN.md`.
4. Read `.codex/CURRENT_CAMPAIGN.md`.
5. Read `.codex/CURRENT_MISSION.md`.
6. Run `python3 scripts/status.py`.
7. Confirm the worktree state and avoid staging unrelated files.

## Completion Rule

A mission is complete only when all mission acceptance criteria pass or a
blocking dependency is recorded explicitly in `.codex/CURRENT_MISSION.md` and
`.codex/MISSION_LOG.md`. Passing a vacuous or skipped check is not sufficient
unless the mission explicitly permits that skip.

## Campaign Source

The detailed M6 step contract is `docs/campaigns/m6-storage-stage-1.md`.
Step allowed-file blocks are binding for implementation files; `.codex/*`
files may be edited only for required mission-state closeout.
