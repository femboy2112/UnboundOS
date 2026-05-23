# Current Campaign

Campaign: C12 M11 IDE Assistant
Active mission: C12.M11 Step 5 Assistant smoke evidence and gates
Status: ready
Stop rule: stop after one complete mission unless the operator explicitly
approves a bundled run; bundled runs stop at the next review gate, blocker, or
failed verification.
Bundle policy: sequential only; run validation, commit, push, and reload
mission state after each completed mission.
Publish policy: commit and push the campaign branch after each completed
mission.
Main policy: never merge to main, never push main, or force-push.
Campaign branch: campaign/m11-ide-assistant

## Campaign Objective

Close M11 by proving the spec §13.1 assistant criterion: a local assistant
explains graph and SSOD.

## Active Mission

See `.codex/CURRENT_MISSION.md`.

## Macro Sequence

1. C12.M11 Step 1 Structured action buffer contract. Completed.
2. C12.M11 Step 2 Graph explanation snapshot. Completed.
3. C12.M11 Step 3 SSOD explanation snapshot. Completed.
4. C12.M11 Step 4 Assistant explanation surface. Completed.
5. C12.M11 Step 5 Assistant smoke evidence and gates. Active.
6. C12.M11 Step 6 M11 completion audit. Pending.

## Required Preflight For `go`

1. Read `CLAUDE.md`.
2. Read `docs/campaigns/m11-ide-assistant.md`.
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

The detailed M11 step contract is `docs/campaigns/m11-ide-assistant.md`.
Step allowed-file blocks are binding for implementation files; `.codex/*`
files may be edited only for required mission-state closeout.
