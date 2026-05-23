# M1 Diagnostics Core Campaign

## Campaign status

```
STEP-COMMIT / PUSH-EVERY-STEP / BRANCH=campaign/m1-diagnostics-core
```

## Mandatory files to read

```
CLAUDE.md
MILESTONE_CATALOG.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
kernel/src/idt.rs
kernel/src/ssod.rs
kernel/src/main.rs
kernel/src/boot.rs
scripts/qemu.sh
scripts/gates.sh
```

## Strategic target

After this campaign closes, M1 proves the spec §13.3 diagnostics exit criteria
with real forced faults:

```
IDT installed.
Divide-by-zero handled.
Page fault handled.
Invalid opcode handled.
SSOD serial output includes RIP and reason.
```

M1 builds on the M0 heartbeat path. It does not claim allocator, memory-map,
framebuffer, graph, storage, or LLM milestones.

## Baseline

- M0 is complete at commit `67d2bd2`.
- `kernel/src/idt.rs` already installs entries for #DE, #UD, #DF, #GP, and
  #PF and routes them through `ssod::kernel_panic`.
- `kernel/src/ssod.rs` already emits a structured serial record with reason,
  vector, RIP, segment, flags, stack, and optional error code.
- No real QEMU forced-fault smoke exists yet, so the M1 exit criteria are not
  reproducibly proven.

## Design thesis

M0 made boot visible. M1 makes fatal boot failures inspectable and testable.
Every intentional fault used for M1 validation must be explicit, gated by the
QEMU smoke tooling, and impossible to enter accidentally in normal boot.

## Non-negotiable boundaries

```
H1  no persistent pointers — M1 emits diagnostics only.
H2  single verifier gate   — M1 must not construct a GraphRuntime.
H3  no hidden execution    — forced faults are boot-test entry points only.
H4  LLM never mutates      — M1 has no LLM path.
H5  no eval node           — M1 has no graph path.
H6  no SIMD assumption     — M1 does not assume SIMD features.
H7  named arenas           — allocator work remains M2.
H8  resource IDs           — M1 touches no storage.
H9  boot is never blind    — M1 preserves the M0 heartbeat channel.
H10 SSOD for fatal         — central rule of this campaign.
```

## Allowed scope summary

```
kernel/src/idt.rs
kernel/src/ssod.rs
kernel/src/boot.rs
kernel/src/main.rs
scripts/qemu.sh
scripts/gates.sh
Makefile
docs/campaigns/m1-diagnostics-core.md
MILESTONE_CATALOG.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
.codex/MISSION_LOG.md
```

## Macro sequence

```
Step 1 — Forced-fault smoke harness
Step 2 — Divide-by-zero SSOD proof
Step 3 — Invalid-opcode SSOD proof
Step 4 — Page-fault SSOD proof
Step 5 — M1 completion audit
```

---

# Step 1 — Forced-fault smoke harness

Status: Completed.

Purpose:
  Add a QEMU-only forced-fault selection path that can intentionally trigger
  one diagnostic vector after the M0 heartbeat is live. Normal boot must remain
  unchanged and reach `UNBOUNDOS_BOOT_OK`.

Allowed files:
```
kernel/src/boot.rs
kernel/src/idt.rs
scripts/qemu.sh
Makefile
docs/campaigns/m1-diagnostics-core.md
```

Required work:
  - Add an explicit boot-test selector for `divide_error`, `invalid_opcode`,
    and `page_fault`.
  - Ensure forced-fault mode still emits the M0 heartbeat through
    `UNBOUNDOS_IDT_OK` before triggering the selected vector.
  - Add QEMU script support to request a forced-fault mode and assert
    `UNBOUNDOS_SSOD_BEGIN`, `reason=<fault>`, `rip=...`, and
    `UNBOUNDOS_SSOD_END`.
  - Keep normal `make qemu-headless` behavior unchanged.

Validation:
```
make fmt
make clippy
make kernel
make qemu-headless
make qemu-fault-de
make qemu-fault-ud
make qemu-fault-pf
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 2 — Divide-by-zero SSOD proof

Purpose:
  Prove the #DE path routes through SSOD and includes reason and RIP in serial
  output.

Allowed files:
```
kernel/src/idt.rs
kernel/src/ssod.rs
scripts/qemu.sh
scripts/gates.sh
Makefile
docs/campaigns/m1-diagnostics-core.md
```

Required work:
  - Add a `make qemu-fault-de` target or equivalent gate entry.
  - Assert serial output includes `reason=divide_error` and a non-empty `rip`.

Validation:
```
make fmt
make clippy
make kernel
make qemu-fault-de
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 3 — Invalid-opcode SSOD proof

Purpose:
  Prove the #UD path routes through SSOD and includes reason and RIP in serial
  output.

Allowed files:
```
kernel/src/idt.rs
kernel/src/ssod.rs
scripts/qemu.sh
scripts/gates.sh
Makefile
docs/campaigns/m1-diagnostics-core.md
```

Required work:
  - Add a `make qemu-fault-ud` target or equivalent gate entry.
  - Assert serial output includes `reason=invalid_opcode` and a non-empty
    `rip`.

Validation:
```
make fmt
make clippy
make kernel
make qemu-fault-ud
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 4 — Page-fault SSOD proof

Purpose:
  Prove the #PF path routes through SSOD and includes reason, RIP, and an error
  code in serial output.

Allowed files:
```
kernel/src/idt.rs
kernel/src/ssod.rs
scripts/qemu.sh
scripts/gates.sh
Makefile
docs/campaigns/m1-diagnostics-core.md
```

Required work:
  - Add a `make qemu-fault-pf` target or equivalent gate entry.
  - Assert serial output includes `reason=page_fault`, a non-empty `rip`, and
    `error_code=...`.

Validation:
```
make fmt
make clippy
make kernel
make qemu-fault-pf
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 5 — M1 completion audit

Purpose:
  Close M1 only after the forced-fault gates prove all spec §13.3 exit
  criteria.

Allowed files:
```
MILESTONE_CATALOG.md
docs/campaigns/m1-diagnostics-core.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
.codex/MISSION_LOG.md
```

Required work:
  - Change row M1 `Status` from `IN-PROGRESS` to `DONE`.
  - Bump the catalog version banner.
  - Add a change-log entry under `## Change log` summarizing M1.
  - In this campaign file, append a `## Closeout` section noting the commit
    SHAs of Steps 1-4.

Validation:
```
make gates
make repo-state
python3 scripts/verify.py --mission current
```

Commit and push.
