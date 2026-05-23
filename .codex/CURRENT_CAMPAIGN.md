# Current Campaign

Campaign: C13 M12 Local Retrieval
Active mission: C13.M12 Step 2 Local document index snapshot
Status: ready
Stop rule: stop after one complete mission unless the operator explicitly
approves a bundled run; bundled runs stop at the next review gate, blocker, or
failed verification.
Bundle policy: sequential only; run validation, commit, push, and reload
mission state after each completed mission.
Publish policy: commit and push the campaign branch after each completed
mission.
Main policy: never merge to main, never push main, or force-push.
Campaign branch: campaign/m12-local-retrieval

## Campaign Objective

Close M12 by proving the spec §13.1 retrieval criterion: a local assistant
searches local docs.

## Active Mission

See `.codex/CURRENT_MISSION.md`.

## Macro Sequence

1. C13.M12 Step 1 Retrieval data contracts. Completed.
2. C13.M12 Step 2 Local document index snapshot. Active.
3. C13.M12 Step 3 Deterministic retrieval ranking. Pending.
4. C13.M12 Step 4 Context packing. Pending.
5. C13.M12 Step 5 Assistant retrieval surface. Pending.
6. C13.M12 Step 6 Retrieval smoke evidence and gates. Pending.
7. C13.M12 Step 7 M12 completion audit. Pending.

## Required Preflight For `go`

1. Read `CLAUDE.md`.
2. Read `docs/campaigns/m12-local-retrieval.md`.
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

The detailed M12 step contract is `docs/campaigns/m12-local-retrieval.md`.
Step allowed-file blocks are binding for implementation files; `.codex/*`
files may be edited only for required mission-state closeout.
