# Current Campaign

Campaign: C4 M3 Embedded Graph
Active mission: C4.M3 Step 5 M3 completion audit
Status: completed
Stop rule: stop after one complete mission unless the operator explicitly
approves a bundled run; bundled runs stop at the next review gate, blocker, or
failed verification.
Bundle policy: sequential only; run validation, commit, push, and reload
mission state after each completed mission.
Publish policy: commit and push the campaign branch after each completed
mission.
Main policy: never merge to main, never push main, or force-push.
Campaign branch: campaign/m3-embedded-graph

## Campaign Objective

Close M3 by proving the spec §13.5 embedded-graph exit criteria while
preserving H2: source -> transform -> sink executes, epoch readiness works,
fan-out works, and active node diagnostics work.

## Active Mission

See `.codex/CURRENT_MISSION.md`.

## Macro Sequence

1. C4.M3 Step 1 Runtime epoch readiness primitives. Completed.
2. C4.M3 Step 2 Private hardcoded graph runtime. Completed.
3. C4.M3 Step 3 Fan-out execution proof. Completed.
4. C4.M3 Step 4 Active node diagnostics. Completed.
5. C4.M3 Step 5 M3 completion audit. Completed.

## Closeout

M3 is complete. `/go` must stop here until the operator opens the final M3 PR
or rotates the control files to M4.

## Required Preflight For `go`

1. Read `CLAUDE.md`.
2. Read `docs/campaigns/m3-embedded-graph.md`.
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

The detailed M3 step contract is `docs/campaigns/m3-embedded-graph.md`.
Step allowed-file blocks are binding for implementation files; `.codex/*`
files may be edited only for required mission-state closeout.
