# Current Campaign

Campaign: C2 M1 Diagnostics Core
Active mission: C2.M1 Step 5 M1 completion audit
Status: ready
Stop rule: stop after one complete mission unless the operator explicitly
approves a bundled run; bundled runs stop at the next review gate, blocker, or
failed verification.
Bundle policy: sequential only; run validation, commit, push, and reload
mission state after each completed mission.
Publish policy: commit and push the campaign branch after each completed
mission.
Main policy: never merge to main, never push main, or force-push.
Campaign branch: campaign/m1-diagnostics-core

## Campaign Objective

Close M1 by proving the spec §13.3 diagnostics exit criteria in QEMU:
IDT installed, divide-by-zero handled, page fault handled, invalid opcode
handled, and SSOD serial output includes RIP and reason.

## Active Mission

See `.codex/CURRENT_MISSION.md`.

## Macro Sequence

1. C2.M1 Step 1 Forced-fault smoke harness. Completed.
2. C2.M1 Step 2 Divide-by-zero SSOD proof. Completed.
3. C2.M1 Step 3 Invalid-opcode SSOD proof. Completed.
4. C2.M1 Step 4 Page-fault SSOD proof. Completed.
5. C2.M1 Step 5 M1 completion audit. Active.

## Required Preflight For `go`

1. Read `CLAUDE.md`.
2. Read `docs/campaigns/m1-diagnostics-core.md`.
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

The detailed M1 step contract is `docs/campaigns/m1-diagnostics-core.md`.
Step allowed-file blocks are binding for implementation files; `.codex/*`
files may be edited only for required mission-state closeout.
