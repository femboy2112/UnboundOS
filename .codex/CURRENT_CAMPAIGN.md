# Current Campaign

Campaign: C6 M5 Minimal UI
Active mission: C6.M5 Step 2 Boot diagnostic framebuffer fallback
Status: ready
Stop rule: stop after one complete mission unless the operator explicitly
approves a bundled run; bundled runs stop at the next review gate, blocker, or
failed verification.
Bundle policy: sequential only; run validation, commit, push, and reload
mission state after each completed mission.
Publish policy: commit and push the campaign branch after each completed
mission.
Main policy: never merge to main, never push main, or force-push.
Campaign branch: campaign/m5-minimal-ui

## Campaign Objective

Close M5 by proving the spec §13.7 Minimal UI exit criterion: framebuffer text
output exists, boot diagnostics can surface without UART, and a minimal IDE
display can show verified graph state.

## Active Mission

See `.codex/CURRENT_MISSION.md`.

## Macro Sequence

1. C6.M5 Step 1 Framebuffer text surface primitives. Completed.
2. C6.M5 Step 2 Boot diagnostic framebuffer fallback. Active.
3. C6.M5 Step 3 Minimal graph-state display model. Pending.
4. C6.M5 Step 4 UI smoke evidence and gates. Pending.
5. C6.M5 Step 5 M5 completion audit. Pending.

## Required Preflight For `go`

1. Read `CLAUDE.md`.
2. Read `docs/campaigns/m5-minimal-ui.md`.
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

The detailed M5 step contract is `docs/campaigns/m5-minimal-ui.md`.
Step allowed-file blocks are binding for implementation files; `.codex/*`
files may be edited only for required mission-state closeout.
