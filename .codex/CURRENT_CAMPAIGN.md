# Current Campaign

Campaign: C11 M10 Quantized Inference
Active mission: C11.M10 Step 4 Streaming token surface
Status: ready
Stop rule: stop after one complete mission unless the operator explicitly
approves a bundled run; bundled runs stop at the next review gate, blocker, or
failed verification.
Bundle policy: sequential only; run validation, commit, push, and reload
mission state after each completed mission.
Publish policy: commit and push the campaign branch after each completed
mission.
Main policy: never merge to main, never push main, or force-push.
Campaign branch: campaign/m10-quantized-inference

## Campaign Objective

Close M10 by proving the spec §13.12 local-LLM criterion: a small quantized
model streams tokens.

## Active Mission

See `.codex/CURRENT_MISSION.md`.

## Macro Sequence

1. C11.M10 Step 1 Scalar quantized kernel contracts. Completed.
2. C11.M10 Step 2 Dispatch-selected scalar kernel table. Completed.
3. C11.M10 Step 3 Deterministic quantized token step. Completed.
4. C11.M10 Step 4 Streaming token surface. Active.
5. C11.M10 Step 5 Quantized inference smoke evidence and gates. Pending.
6. C11.M10 Step 6 M10 completion audit. Pending.

## Required Preflight For `go`

1. Read `CLAUDE.md`.
2. Read `docs/campaigns/m10-quantized-inference.md`.
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

The detailed M10 step contract is `docs/campaigns/m10-quantized-inference.md`.
Step allowed-file blocks are binding for implementation files; `.codex/*`
files may be edited only for required mission-state closeout.
