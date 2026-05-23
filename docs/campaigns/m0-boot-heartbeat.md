# M0 Boot Heartbeat Campaign

## Campaign status

```
STEP-COMMIT / PUSH-EVERY-STEP / BRANCH=campaign/m0-boot-heartbeat
```

## Mandatory files to read

```
CLAUDE.md
MILESTONE_CATALOG.md
CURRENT_MISSION.md
kernel/src/main.rs
kernel/src/boot.rs
kernel/src/heartbeat.rs
kernel/src/boot_diag.rs
kernel/src/serial.rs
scripts/qemu.sh
scripts/fidelity_check.sh
```

## Strategic target

After this campaign closes, `make qemu-headless` from a clean checkout
prints, on the serial line, in this exact order:

```
UNBOUNDOS_BOOT_BEGIN
UNBOUNDOS_CPU_PROFILE
UNBOUNDOS_MEMMAP_OK
UNBOUNDOS_IDT_OK
UNBOUNDOS_BOOT_OK
```

…and the kernel halts cleanly. Boot is no longer blind (H9 / spec §1.6).

## Baseline

- Commit `78f7ad5` ("boot heartbeat: serial UART, boot-diag fallback,
  real IDT (M0 §1.6/§3.2)") has already landed on `main` and is the
  starting point.
- `kernel/src/serial.rs` already implements the UART 16550 driver
  with internal loopback probe.
- `kernel/src/boot_diag.rs` already exposes a recorded-bytes view used
  by the M2 framebuffer fallback (M0 only needs the recording path).
- `make qemu-headless` may currently fail the heartbeat-order assertion
  until Step 7 lands — that's expected.

## Design thesis

Boot heartbeat is the foundation of every subsequent milestone's
diagnostics. Until M0 closes, every later failure mode (page fault,
verifier reject, allocator exhaustion) lacks a working channel to
surface. We therefore prioritize: (1) a probed UART, (2) a fallback
recording buffer, (3) an early IDT with structured handlers, (4) a
heartbeat-order test runnable in QEMU.

## Non-negotiable boundaries

```
H1  no persistent pointers — M0 emits no persistent files; irrelevant.
H2  single verifier gate   — M0 must not construct a GraphRuntime.
H3  no hidden execution    — heartbeat emission is the only "work".
H4  LLM never mutates      — M0 has no LLM path.
H5  no eval node           — M0 has no graph; trivially holds.
H6  no SIMD assumption     — M0 does no SIMD; trivially holds.
H7  named arenas           — M0 uses static memory only.
H8  resource IDs           — M0 touches no storage.
H9  boot is never blind    — THE central rule of this campaign.
H10 SSOD for fatal         — IDT handlers must route through SSOD stub.
```

## Allowed scope summary

```
kernel/src/main.rs
kernel/src/boot.rs
kernel/src/heartbeat.rs
kernel/src/boot_diag.rs
kernel/src/serial.rs
kernel/src/arch/idt.rs
kernel/src/arch/exceptions.rs
kernel/src/panic.rs
scripts/qemu.sh
scripts/fidelity_check.sh
docs/campaigns/m0-boot-heartbeat.md   (this file, for status updates)
MILESTONE_CATALOG.md                  (only at Step 8 for the catalog flip)
```

## Suggested artifact schema

M0 introduces no persistent artifacts. Heartbeat strings are
`&'static str` constants in `kernel/src/heartbeat.rs`.

## Macro sequence

```
Step 1 — Boot-order assertion vs spec §3.2 14-step contract
Step 2 — Serial UART probe + heartbeat string emission
Step 3 — IDT install + UNBOUNDOS_IDT_OK
Step 4 — Boot-diagnostic-buffer fallback (spec §3.9) on UART probe fail
Step 5 — Review gate
Step 6 — Panic path routed through SSOD (M0-scope stub of spec §9)
Step 7 — qemu-smoke headless assertion (UNBOUNDOS_BOOT_OK observed)
Step 8 — M0 completion audit: flip MILESTONE_CATALOG.md row M0 → DONE
```

---

# Step 1 — Boot-order assertion vs spec §3.2 14-step contract

Purpose:
  Add a source-level assertion that `_start` performs the 14 ordered
  steps of the spec §3.2 kernel-entry contract. The `/boot-heartbeat-check`
  skill walks the assertion and reports any drift.

Allowed files:
```
kernel/src/main.rs
kernel/src/boot.rs
```

Required work:
  - Annotate `_start` (or its outermost dispatcher) with numbered spec
    citations: `// spec §3.2 step <N>: <one-line>` per step.
  - Where a step is still M1+ work, leave the existing
    `unimplemented!()` or `TODO M<n>` marker but ensure the comment
    cites the correct spec section.
  - Do **not** implement M1+ steps in this campaign.

Validation:
```
make fmt
make clippy
/boot-heartbeat-check
```

Commit and push.

---

# Step 2 — Serial UART probe + heartbeat string emission

Purpose:
  Confirm `kernel/src/serial.rs` probes COM1 with internal loopback,
  then emits `UNBOUNDOS_BOOT_BEGIN`, `UNBOUNDOS_CPU_PROFILE`, and
  `UNBOUNDOS_MEMMAP_OK` in order. If `78f7ad5` already covers this,
  this step is a verification-only commit (touch nothing if already
  green; otherwise patch).

Allowed files:
```
kernel/src/serial.rs
kernel/src/heartbeat.rs
kernel/src/boot.rs
```

Required work:
  - Run the validation block. If everything passes already, skip
    edits; commit "verify-only" with `git commit --allow-empty` is
    not permitted — instead, move directly to Step 3 without a commit.
  - If validation fails, patch `serial.rs` / `heartbeat.rs` /
    `boot.rs` to fix the emission order before committing.

Validation:
```
make fmt
make clippy
make kernel
make qemu-headless | grep -E '^UNBOUNDOS_(BOOT_BEGIN|CPU_PROFILE|MEMMAP_OK)$'
```

Commit and push (only if files changed; otherwise skip).

---

# Step 3 — IDT install + UNBOUNDOS_IDT_OK

Purpose:
  Install the early IDT with fatal handler stubs (#DE, #UD, #GP, #PF,
  #DF), routed through `DiagnosticContext`. Emit `UNBOUNDOS_IDT_OK` on
  the serial line after install succeeds. Comes before any allocator
  init (spec §3.2 step 6).

Allowed files:
```
kernel/src/main.rs
kernel/src/boot.rs
kernel/src/heartbeat.rs
kernel/src/arch/idt.rs
kernel/src/arch/exceptions.rs
```

Required work:
  - Define a 256-entry IDT static.
  - Wire fatal handler stubs that fill a `DiagnosticContext` and call
    `kernel_panic(reason, ctx)` (the existing panic surface).
  - In `_start`, install the IDT BEFORE the memory-map ingest step.
  - Emit `UNBOUNDOS_IDT_OK` immediately after `lidt` returns.
  - On install failure (any), fall through to the boot-diagnostic-buffer
    path (Step 4 implements the fallback recording; this step just
    has to not panic blind).

Validation:
```
make fmt
make clippy
make kernel
/boot-heartbeat-check
make qemu-headless | grep -E '^UNBOUNDOS_(BOOT_BEGIN|CPU_PROFILE|MEMMAP_OK|IDT_OK)$'
```

Commit and push.

---

# Step 4 — Boot-diagnostic-buffer fallback (spec §3.9)

Purpose:
  When the COM1 loopback probe fails, every subsequent heartbeat
  string must still be recorded into the boot-diagnostic-buffer
  (already exposed by `kernel/src/boot_diag.rs`). The next milestone
  (M2) dumps it to the framebuffer; at M0 we only need the recording
  path to work.

Allowed files:
```
kernel/src/heartbeat.rs
kernel/src/boot_diag.rs
kernel/src/serial.rs
```

Required work:
  - In the heartbeat emission helper, branch: if `serial::init()`
    returned the no-UART variant, route emissions through
    `boot_diag::record()`.
  - Add a runtime-readable flag (or symbol) so an M0 smoke variant
    `make qemu-no-serial` can exercise this path.
  - Document the fallback in `kernel/src/heartbeat.rs` doc-comment
    with `// spec §3.9` citation.

Validation:
```
make fmt
make clippy
make kernel
make qemu-no-serial
# Expect: boot-diagnostic-buffer contains BOOT_BEGIN..IDT_OK strings.
```

Commit and push.

---

# Step 5 — Review gate

Stop. The operator must:

1. Run `make qemu-headless` and observe the four heartbeat strings
   emit on the serial line in the right order.
2. Run `make qemu-no-serial` and confirm the boot-diagnostic-buffer
   records them when UART is absent.
3. Spot-check the IDT install path in `kernel/src/arch/idt.rs` for
   spec §3.2 ordering.

Only after explicit operator approval may Steps 6+ run.

`/go` emits `Stop reason: review-gate` and ends.

---

# Step 6 — Panic path routed through SSOD (M0-scope stub of spec §9)

Purpose:
  Wire the existing `kernel_panic` surface to emit a minimal SSOD
  record on the serial line (and into the boot-diagnostic-buffer)
  before halting. Full SSOD (snark matrix, fault-code families,
  framebuffer rendering) lands in later milestones; M0 only needs
  the structured-record skeleton.

Allowed files:
```
kernel/src/ssod.rs
kernel/src/heartbeat.rs
kernel/src/boot_diag.rs
kernel/src/idt.rs
```

Required work:
  - In `ssod.rs`, emit `UNBOUNDOS_SSOD_BEGIN` then a structured
    record `{ reason, ctx.arena_id?, ctx.graph_id?, ctx.node_id? }`
    serialized as plain text key=value lines, then
    `UNBOUNDOS_SSOD_END`, then halt (`hlt`).
  - Ensure the IDT exception stubs feed `DiagnosticContext` correctly
    so the panic gets non-empty fields.

Validation:
```
make fmt
make clippy
make kernel
# Inject a #UD via a debug-only int3 / ud2 path is out of scope;
# verify by reading the panic source path and confirming the record
# format matches /ssod-decode expectations.
```

Spawn `Task subagent_type=ssod-diagnostics-engineer` with prompt
"review M0-scope panic record format in kernel/src/ssod.rs for spec
§9.7 structured field compliance".

Commit and push.

---

# Step 7 — qemu-smoke headless assertion (UNBOUNDOS_BOOT_OK observed)

Purpose:
  Emit `UNBOUNDOS_BOOT_OK` as the last heartbeat after all M0 init
  succeeds, then `hlt`. Add an assertion in `scripts/qemu.sh
  --assert-heartbeat` (or `gates.sh` step 6) that the serial log
  contains the five canonical strings in order.

Allowed files:
```
kernel/src/main.rs
kernel/src/heartbeat.rs
scripts/qemu.sh
scripts/gates.sh
```

Required work:
  - After all M0 init returns, emit `UNBOUNDOS_BOOT_OK` then `hlt`.
  - In `scripts/qemu.sh`, implement `--assert-heartbeat`: greps the
    serial log for the five strings in order, exits non-zero on miss.
  - Make sure `make qemu-headless` continues to work without
    `--assert-heartbeat` for interactive debugging.

Validation:
```
make fmt
make clippy
make kernel
make gates
/qemu-smoke
```

Commit and push.

---

# Step 8 — M0 completion audit: flip MILESTONE_CATALOG.md row M0 → DONE

Purpose:
  Catalog the completion. Bump the catalog version. Stage the rotation
  to M1 (do not flip M1 to IN-PROGRESS in this step — that's the
  operator's call via `spec-refresher`).

Allowed files:
```
MILESTONE_CATALOG.md
docs/campaigns/m0-boot-heartbeat.md
```

Required work:
  - Change row M0 `Status` from `IN-PROGRESS` to `DONE`.
  - Bump the catalog version banner at top of
    `MILESTONE_CATALOG.md` (v0.1 → v0.2).
  - Add a change-log entry under `## Change log` summarizing M0.
  - In this campaign file, append a `## Closeout` section noting the
    commit shas of Steps 1–7.

Validation:
```
make repo-state
# Expect verdict STOP: campaign complete — refresh CURRENT_MISSION.md.
```

Commit and push.

After this commit, `/go` will refuse to advance further until the
operator either opens the final PR for M0 or rotates the mission to
M1 via the `spec-refresher` agent.
