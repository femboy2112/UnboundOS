# Current Campaign

Campaign: C0 Control Plane
Active mission: C0.M2 Mission state handoff validation
Status: ready
Stop rule: stop after one complete mission, even when the next mission is obvious.
Publish policy: commit and push after each completed mission.

## Campaign Objective

Install the Codex-native control surface required to run UnboundOS as a
mission-by-mission project. The control surface must be repo-local,
spec-bound, and deterministic enough that an operator can say `go` and Codex
can select, execute, verify, publish, and stop after the active mission.

## Active Mission

See `.codex/CURRENT_MISSION.md`.

## Next Mission Queue

1. C1.M0 Boot heartbeat and real QEMU smoke
2. C2.M1 Diagnostics core
3. C3.M2 Arena memory
4. C4.M3 Embedded graph execution
5. C5.M4 UMOD loader and verifier
6. C6.M5-M6 UI and storage
7. C7.M7-M10 local LLM core
8. C8.M11-M12 assistant and retrieval

## Required Preflight For `go`

1. Read `CLAUDE.md`.
2. Read `docs/UnboundOS_Tech_Spec_v2_1_1_Fidelity_Hardening.pdf` or the
   extracted requirements in `.codex/PROJECT_PLAN.md`.
3. Read `.codex/CURRENT_CAMPAIGN.md`.
4. Read `.codex/CURRENT_MISSION.md`.
5. Run `python3 scripts/status.py`.
6. Confirm the worktree state and avoid staging unrelated files.

## Completion Rule

A mission is complete only when all mission acceptance criteria pass or a
blocking dependency is recorded explicitly in `.codex/CURRENT_MISSION.md` and
`.codex/MISSION_LOG.md`. Passing a vacuous or skipped check is not sufficient
unless the mission explicitly permits that skip.
