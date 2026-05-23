# M5 Minimal UI Campaign

## Campaign status

```
STEP-COMMIT / PUSH-EVERY-STEP / BRANCH=campaign/m5-minimal-ui
```

## Mandatory files to read

```
CLAUDE.md
MILESTONE_CATALOG.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
kernel/src/main.rs
kernel/src/boot.rs
kernel/src/heartbeat.rs
kernel/src/boot_diag.rs
crates/graph/src/lib.rs
crates/graph/src/loader.rs
```

## Strategic target

After this campaign closes, M5 proves the spec §13.7 minimal-UI exit
criterion:

```
Framebuffer text output exists, boot diagnostics can surface without UART, and
the minimal IDE display can show verified graph state.
```

H2, H9, and H10 remain binding. UI display may read graph diagnostics but must
not construct or mutate runtime graphs directly.

## Baseline

- M4 completed at commit `65d8ab3`.
- Boot heartbeat and SSOD diagnostics are serial-first and recorded into
  `boot_diag`.
- `heartbeat::finalize_framebuffer_fallback` still has a TODO stub.
- No `kernel/src/framebuffer.rs` module exists yet.

## Non-negotiable boundaries

```
H2  single verifier gate   — UI never constructs GraphRuntime directly.
H3  no hidden execution    — UI work is boot display / IDE shell only.
H8  resource IDs           — no path-like storage identifiers in UI state.
H9  boot is never blind    — framebuffer augments heartbeat, never replaces it.
H10 structured failures    — UI failures must not swallow SSOD diagnostics.
```

Memory-unsafe Rust is allowed and expected when the hardware boundary requires
it. The constraint is not "avoid unsafe"; the constraint is that unsafe memory
access remains bounded, inspectable, deterministic, and not undefined by
design. Safe wrappers over caller-provided memory are acceptable until a real
MMIO or bootloader-handoff boundary requires an explicit unsafe block.

## Macro sequence

```
Step 1 — Framebuffer text surface primitives
Step 2 — Boot diagnostic framebuffer fallback
Step 3 — Minimal graph-state display model
Step 4 — UI smoke evidence and gates
Step 5 — M5 completion audit
```

---

# Step 1 — Framebuffer text surface primitives

Status: Completed.

Purpose:
  Add a small framebuffer text surface with deterministic glyph-cell writes
  that can be built and tested without requiring bootloader framebuffer handoff.

Allowed files:
```
kernel/src/main.rs
kernel/src/framebuffer.rs
docs/campaigns/m5-minimal-ui.md
```

Required work:
  - Add `kernel/src/framebuffer.rs` with fixed-size text-cell rendering
    primitives over a caller-provided linear pixel buffer.
  - Keep the module boot-passive: no global framebuffer assumptions and no
    writes before explicit initialization.
  - Add focused tests or build-time assertions for cell placement, newline, and
    bounds clipping.

Validation:
```
make fmt
make clippy
make kernel
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 2 — Boot diagnostic framebuffer fallback

Status: Completed.

Purpose:
  Wire the heartbeat fallback hook so a framebuffer surface can display
  `BOOT_NO_SERIAL`, `BOOT_HEARTBEAT_BUFFER_PRESENT`, and the recorded boot
  diagnostic buffer when UART is unavailable.

Allowed files:
```
kernel/src/boot.rs
kernel/src/heartbeat.rs
kernel/src/boot_diag.rs
kernel/src/framebuffer.rs
Makefile
scripts/qemu.sh
docs/campaigns/m5-minimal-ui.md
```

Required work:
  - Replace the TODO-only fallback with a real call path through framebuffer
    text output.
  - Preserve serial heartbeat order and normal boot behavior.
  - Do not require a framebuffer for successful headless boot.
  - Ensure the no-serial QEMU harness verifies boot completion without relying
    on serial-log heartbeats.

Validation:
```
make fmt
make clippy
make qemu-headless
make qemu-no-serial
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 3 — Minimal graph-state display model

Status: Completed.

Purpose:
  Provide a read-only UI model that can display verified graph state without
  constructing, mutating, or bypassing graph runtime handles.

Allowed files:
```
crates/graph/src/lib.rs
crates/graph/src/loader.rs
kernel/src/framebuffer.rs
docs/campaigns/m5-minimal-ui.md
```

Required work:
  - Expose only read-only, symbolic graph display facts needed by the minimal
    IDE surface.
  - Render graph id, node count, wire count, and last active/completed-node
    diagnostics where available.
  - Preserve the private runtime construction boundary in `loader.rs`.

Validation:
```
make fmt
make clippy
cargo test -p graph
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 4 — UI smoke evidence and gates

Status: Completed.

Purpose:
  Make the minimal UI evidence reproducible from checkout.

Allowed files:
```
Makefile
scripts/**
kernel/src/**
docs/campaigns/m5-minimal-ui.md
```

Required work:
  - Add a smoke target or source-level check that proves framebuffer text and
    graph-state rendering are reachable.
  - Keep QEMU headless gates green.
  - Do not add graphical-only CI requirements.

Validation:
```
make fmt
make clippy
make ui-smoke
make gates
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 5 — M5 completion audit

Purpose:
  Close M5 after framebuffer text output, boot-diagnostic fallback, graph-state
  display, and smoke evidence are reproducibly verified.

Allowed files:
```
MILESTONE_CATALOG.md
docs/campaigns/m5-minimal-ui.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
.codex/MISSION_LOG.md
```

Required work:
  - Change row M5 `Status` from `IN-PROGRESS` to `DONE`.
  - Bump the catalog version banner.
  - Add a change-log entry under `## Change log` summarizing M5.
  - In this campaign file, append a `## Closeout` section noting the commit
    SHAs of Steps 1-4.

Validation:
```
make gates
make repo-state
python3 scripts/verify.py --mission current
```

Commit and push.
