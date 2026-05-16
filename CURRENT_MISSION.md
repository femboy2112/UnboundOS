# CURRENT_MISSION.md — M0 Boot Heartbeat

## One-line trigger

When the operator types `go`, `/go`, `run mission`, or `do the current
task`, read this file, then `CURRENT_CAMPAIGN.md`, then run the next
incomplete eligible Step from the macro sequence. Stop at the first
mandatory checkpoint.

## Current milestone

`M0 — Boot heartbeat` (spec §3.2 kernel-entry contract, §1.6
boot-visible heartbeat, §3.9 boot-diagnostic-buffer fallback).

The campaign file is `docs/campaigns/m0-boot-heartbeat.md`. The
top-level `CURRENT_CAMPAIGN.md` is a working copy of that file.

## Branch / push / PR rule

```
Preferred campaign branch: campaign/m0-boot-heartbeat
Preferred final PR title:  M0: boot heartbeat (spec §1.6 / §3.2 / §3.9)
Rules:
  - Work on the campaign branch. Never commit to main.
  - Commit every step. Push every step.
  - Open the final PR only after M0 Step 8 (catalog flip) commits.
  - Never push --force. Never reset --hard.
```

If the current branch is `main` (or any non-campaign branch), `/go`
refuses to commit and tells the operator to checkout
`campaign/m0-boot-heartbeat` first.

## Required reads

```
CLAUDE.md
MILESTONE_CATALOG.md
CURRENT_CAMPAIGN.md
docs/campaigns/m0-boot-heartbeat.md
kernel/src/main.rs
kernel/src/boot.rs
kernel/src/heartbeat.rs
kernel/src/boot_diag.rs
kernel/src/serial.rs
scripts/qemu.sh
scripts/fidelity_check.sh
scripts/gates.sh
```

The spec PDF (`docs/UnboundOS_Tech_Spec_v2_1_1_Fidelity_Hardening.pdf`)
is referenced by section, not re-read in full. Use the
`milestone-explorer` agent for spec-section lookups.

## Baseline to verify

```
expected catalog status: IN-PROGRESS
expected branch:         campaign/m0-boot-heartbeat
expected serial heartbeat (already landed in 78f7ad5):
  UNBOUNDOS_BOOT_BEGIN
  UNBOUNDOS_CPU_PROFILE
  UNBOUNDOS_MEMMAP_OK
  UNBOUNDOS_IDT_OK         (Step 3 landing target)
  UNBOUNDOS_BOOT_OK        (Step 7 landing target)
make gates expected: fmt PASS, clippy PASS, address-scan PASS,
                     fidelity matrix PASS,
                     qemu-smoke may-fail until Step 7 lands.
```

If `make repo-state` disagrees with these facts, stop and ask the
operator to refresh the baseline (via `spec-refresher` or manual
edit).

## Architectural guardrails

This mission must not violate any of CLAUDE.md §2 Hard Rules
(H1–H10). Specifically for M0:

- **H9 (boot is never blind)** — every boot path emits a heartbeat
  string on the serial UART, or falls back to the
  boot-diagnostic-buffer (spec §3.9) when the UART probe fails.
- **H10 (fatal exceptions → SSOD)** — early IDT handlers route through
  the structured `DiagnosticContext` even at M0 stub level. No silent
  reboots, no swallowed faults.
- **H2 (single verifier gate)** — M0 does not exercise the graph
  loader; this is fine. But no shortcut to a `GraphRuntime` may land
  in this campaign.
- **H7 (named arenas)** — M0 sets up no graph arenas, but any boot-time
  allocation must already cite an arena phase (most M0 code is static
  / `.bss` and does not allocate).

## Command rule

Use `make` and `python3 scripts/...` over improvised shell pipelines
(CLAUDE.md §5). When Python module commands appear in this file as
`python -m ...`, run them as `python3 -m ...`.

## Final report format

```
/go report
==========
Steps executed:   <list of "Step N — title" or "(none)">
Created/updated:  <list of files, or "(none)">
Validation:       <one line per command + PASS/FAIL>
Git:              commit <sha>  branch <name>  push <ok|skipped|failed>
Next:             <next step title, or "(M0 complete)">
Stop reason:      <review-gate | gate-failure:<name> | hard-rule |
                   ambiguity | out-of-scope | done>
```
