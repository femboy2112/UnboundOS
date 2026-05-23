# M6 Storage Stage 1 Campaign

## Campaign status

```
STEP-COMMIT / PUSH-EVERY-STEP / BRANCH=campaign/m6-storage-stage-1
```

## Mandatory files to read

```
CLAUDE.md
MILESTONE_CATALOG.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
kernel/src/main.rs
kernel/src/boot.rs
kernel/src/serial.rs
kernel/src/ssod.rs
scripts/qemu.sh
```

## Strategic target

Close M6 by proving the spec §13.8 Storage stage 1 exit criterion:

```
Raw sector read works with timeout.
```

Spec §7.1 says storage is staged and FAT32 should not be the first storage
milestone. Spec §7.2 keeps graph-visible storage references as opaque IDs, not
paths. Spec §7.3 permits ATA PIO as the first QEMU/legacy backend and requires
timeout handling; infinite polling is invalid.

## Baseline

- M5 completed at commit `84ddde6`.
- QEMU image launch can attach a raw drive through `scripts/qemu.sh`.
- No storage module or raw-sector read API exists yet.
- Graph-visible persistent resources are already checked for path leakage by
  the existing verifier and source checks.

## Non-negotiable boundaries

```
H1  symbolic artifacts      — persistent files never store raw pointers.
H2  single verifier gate    — storage never constructs GraphRuntime directly.
H3  no hidden execution     — storage smoke work is boot init or graph-visible.
H8  resource IDs            — no FAT32/POSIX path crosses into graph state.
H9  boot is never blind     — storage diagnostics emit heartbeat/SSOD context.
H10 structured failures     — read errors report backend, LBA, op, status, timeout.
```

Memory-unsafe Rust is allowed and expected for ATA PIO port I/O and future DMA
boundaries. The constraint is not "avoid unsafe"; the constraint is that unsafe
storage access remains bounded, inspectable, deterministic, and not undefined
by design. Infinite polling, unchecked buffer writes, and path leakage are the
actual bugs for this milestone.

## Macro sequence

```
Step 1 — Storage contracts, diagnostics, and timeout model
Step 2 — ATA PIO sector-read primitive
Step 3 — QEMU raw-sector smoke fixture
Step 4 — Resource namespace guard evidence
Step 5 — M6 completion audit
```

---

# Step 1 — Storage contracts, diagnostics, and timeout model

Status: Completed.

Purpose:
  Add the kernel storage surface for finite-poll raw-sector reads before
  touching real port I/O.

Allowed files:
```
kernel/src/main.rs
kernel/src/storage.rs
docs/campaigns/m6-storage-stage-1.md
```

Required work:
  - Add a storage module with fixed-width LBA/count/status fields and a
    structured diagnostic error carrying backend, LBA, operation, status, and
    timeout count.
  - Add a timeout-budget poll model that can be unit-tested without hardware.
  - Keep writes unimplemented and unavailable by default.
  - Do not introduce path-like graph resource identifiers.

Validation:
```
make fmt
make clippy
rustc --test kernel/src/storage.rs
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 2 — ATA PIO sector-read primitive

Status: Completed.

Purpose:
  Implement the spec §7.3 ATA PIO read sequence behind an explicit unsafe port
  boundary.

Allowed files:
```
kernel/src/storage.rs
kernel/src/boot.rs
docs/campaigns/m6-storage-stage-1.md
```

Required work:
  - Select drive/head and LBA high bits, write sector count and LBA bytes,
    issue command `0x20`, poll until DRQ or error/timeout, and read exactly
    256 16-bit words into a caller-provided sector buffer.
  - Document each unsafe port-I/O boundary.
  - Return structured timeout/error diagnostics rather than panicking.

Validation:
```
make fmt
make clippy
rustc --test kernel/src/storage.rs
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 3 — QEMU raw-sector smoke fixture

Purpose:
  Prove raw-sector read under QEMU with a deterministic disk image and finite
  timeout behavior.

Allowed files:
```
Makefile
scripts/**
kernel/src/boot.rs
kernel/src/storage.rs
docs/campaigns/m6-storage-stage-1.md
```

Required work:
  - Add a deterministic raw disk fixture or generator with a recognizable
    first-sector marker.
  - Add a QEMU smoke target that boots with the fixture attached and asserts a
    storage heartbeat proving the sector marker was read.
  - Preserve existing headless heartbeat gates.

Validation:
```
make fmt
make clippy
make qemu-storage-smoke
make gates
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 4 — Resource namespace guard evidence

Purpose:
  Prove storage bring-up did not leak POSIX/FAT32 paths into graph-visible
  resource state.

Allowed files:
```
scripts/**
crates/umod/src/lib.rs
crates/graph/src/verifier.rs
tests/fuzz_corpus/umod/**
docs/campaigns/m6-storage-stage-1.md
```

Required work:
  - Add or extend checks that reject path-like storage references above the
    storage adapter boundary.
  - Keep accepted examples to opaque `type:id` resource references.
  - Ensure the aggregate verifier runs the guard evidence.

Validation:
```
make fmt
make clippy
cargo test -p umod
cargo test -p graph
python3 scripts/verify.py --mission current
make gates
```

Commit and push.

---

# Step 5 — M6 completion audit

Purpose:
  Close M6 after raw-sector read, timeout behavior, QEMU smoke evidence, and
  resource-boundary checks are reproducibly verified.

Allowed files:
```
MILESTONE_CATALOG.md
docs/campaigns/m6-storage-stage-1.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
.codex/MISSION_LOG.md
```

Required work:
  - Change row M6 `Status` from `IN-PROGRESS` to `DONE`.
  - Bump the catalog version banner.
  - Add a change-log entry under `## Change log` summarizing M6.
  - In this campaign file, append a `## Closeout` section noting the commit
    SHAs of Steps 1-4.

Validation:
```
make gates
make repo-state
python3 scripts/verify.py --mission current
```

Commit and push.
