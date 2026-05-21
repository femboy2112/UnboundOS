# Current Campaign

Campaign: C1 M0 Boot Heartbeat
Active mission: C1.M0 Step 1 Boot-order assertion vs spec §3.2
Status: ready
Stop rule: stop after one complete mission, even when the next mission is obvious.
Publish policy: commit and push after each completed mission.
Campaign branch: campaign/m0-boot-heartbeat

## Campaign Objective

Close M0 by making boot visible and reproducible. A clean checkout must be able
to run `make qemu-headless` and observe the required heartbeat sequence:
`UNBOUNDOS_BOOT_BEGIN`, `UNBOUNDOS_CPU_PROFILE`, `UNBOUNDOS_MEMMAP_OK`,
`UNBOUNDOS_IDT_OK`, and `UNBOUNDOS_BOOT_OK`.

## Active Mission

See `.codex/CURRENT_MISSION.md`.

## Macro Sequence

1. C1.M0 Step 1 Boot-order assertion vs spec §3.2.
2. C1.M0 Step 2 Serial UART probe and heartbeat string emission.
3. C1.M0 Step 3 IDT install and `UNBOUNDOS_IDT_OK`.
4. C1.M0 Step 4 Boot-diagnostic-buffer fallback.
5. C1.M0 Step 5 Review gate.
6. C1.M0 Step 6 Panic path routed through SSOD.
7. C1.M0 Step 7 QEMU smoke headless assertion.
8. C1.M0 Step 8 M0 completion audit.

## Required Preflight For `go`

1. Read `CLAUDE.md`.
2. Read `docs/campaigns/m0-boot-heartbeat.md`.
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

The detailed M0 step contract is `docs/campaigns/m0-boot-heartbeat.md`. Step
allowed-file blocks are binding for implementation files; `.codex/*` files may
be edited only for required mission-state closeout.
