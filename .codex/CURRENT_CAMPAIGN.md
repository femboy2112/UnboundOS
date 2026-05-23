# Current Campaign

Campaign: C8 M7 Tokenizer
Active mission: C8.M7 Step 3 Raw-byte detokenizer round trip
Status: ready
Stop rule: stop after one complete mission unless the operator explicitly
approves a bundled run; bundled runs stop at the next review gate, blocker, or
failed verification.
Bundle policy: sequential only; run validation, commit, push, and reload
mission state after each completed mission.
Publish policy: commit and push the campaign branch after each completed
mission.
Main policy: never merge to main, never push main, or force-push.
Campaign branch: campaign/m7-tokenizer

## Campaign Objective

Close M7 by proving the spec §13.7 tokenizer exit criterion: tokenizer
round-trip works for the initially supported tokenizer family.

## Active Mission

See `.codex/CURRENT_MISSION.md`.

## Macro Sequence

1. C8.M7 Step 1 Tokenizer registry and metadata contract. Completed.
2. C8.M7 Step 2 Raw-byte tokenizer encode path. Completed.
3. C8.M7 Step 3 Raw-byte detokenizer round trip. Active.
4. C8.M7 Step 4 Tokenizer smoke evidence and gates. Pending.
5. C8.M7 Step 5 M7 completion audit. Pending.

## Required Preflight For `go`

1. Read `CLAUDE.md`.
2. Read `docs/campaigns/m7-tokenizer.md`.
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

The detailed M7 step contract is `docs/campaigns/m7-tokenizer.md`.
Step allowed-file blocks are binding for implementation files; `.codex/*`
files may be edited only for required mission-state closeout.
