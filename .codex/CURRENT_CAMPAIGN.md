# Current Campaign

Campaign: C10 M9 UMDL Loader
Active mission: C10.M9 Step 2 Section bounds and checksum validation
Status: ready
Stop rule: stop after one complete mission unless the operator explicitly
approves a bundled run; bundled runs stop at the next review gate, blocker, or
failed verification.
Bundle policy: sequential only; run validation, commit, push, and reload
mission state after each completed mission.
Publish policy: commit and push the campaign branch after each completed
mission.
Main policy: never merge to main, never push main, or force-push.
Campaign branch: campaign/m9-umdl-loader

## Campaign Objective

Close M9 by proving the spec §13.11 local-LLM criterion: a model package
validates and loads.

## Active Mission

See `.codex/CURRENT_MISSION.md`.

## Macro Sequence

1. C10.M9 Step 1 UMDL header parse and fixed-width contract. Completed.
2. C10.M9 Step 2 Section bounds and checksum validation. Active.
3. C10.M9 Step 3 Tokenizer and tensor descriptor validation. Pending.
4. C10.M9 Step 4 Model load view and arena reservation contract. Pending.
5. C10.M9 Step 5 UMDL smoke fixtures and gates. Pending.
6. C10.M9 Step 6 M9 completion audit. Pending.

## Required Preflight For `go`

1. Read `CLAUDE.md`.
2. Read `docs/campaigns/m9-umdl-loader.md`.
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

The detailed M9 step contract is `docs/campaigns/m9-umdl-loader.md`.
Step allowed-file blocks are binding for implementation files; `.codex/*`
files may be edited only for required mission-state closeout.
