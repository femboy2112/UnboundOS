# M2 Arena Memory Campaign

## Campaign status

```
STEP-COMMIT / PUSH-EVERY-STEP / BRANCH=campaign/m2-arena-memory
```

## Mandatory files to read

```
CLAUDE.md
MILESTONE_CATALOG.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
kernel/src/arena.rs
kernel/src/boot.rs
kernel/src/ssod.rs
scripts/gates.sh
scripts/verify.py
```

## Strategic target

After this campaign closes, M2 proves the spec §13.4 arena-memory exit
criteria:

```
BootArena, KernelArena, GraphArena, and ScratchArena exist.
Alignment tests pass.
Arena exhaustion is deterministic.
Memory map dump is available.
```

M2 does not claim graph execution, UMOD loading, storage, UI, or LLM behavior.

## Baseline

- M1 completed at commit `cda1c9a`.
- `kernel/src/arena.rs` currently defines only `ArenaId` and `AllocError`.
- `kernel/src/boot.rs` still marks allocator and memory-map work as later
  TODOs.
- No arena alignment/exhaustion tests exist yet.

## Design thesis

UnboundOS uses bounded named arenas instead of a global heap. M2 must make the
first arena contract concrete enough that later graph, model, and scratch
systems cannot allocate without named ownership, declared phase, alignment, and
deterministic failure context.

## Non-negotiable boundaries

```
H1  no persistent pointers — arenas are runtime-only.
H2  single verifier gate   — M2 must not construct a GraphRuntime.
H3  no hidden execution    — arena diagnostics are boot/test paths only.
H4  LLM never mutates      — M2 has no LLM path.
H5  no eval node           — M2 has no generated execution path.
H6  no SIMD assumption     — alignment support must not imply SIMD use.
H7  named arenas           — central rule of this campaign.
H8  resource IDs           — M2 touches no graph-visible storage refs.
H9  boot is never blind    — preserve M0 heartbeat and M1 SSOD.
H10 SSOD for fatal         — fatal arena exhaustion must be diagnosable.
```

## Allowed scope summary

```
kernel/src/arena.rs
kernel/src/boot.rs
kernel/src/ssod.rs
scripts/gates.sh
scripts/verify.py
Makefile
docs/campaigns/m2-arena-memory.md
MILESTONE_CATALOG.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
.codex/MISSION_LOG.md
```

## Macro sequence

```
Step 1 — Bounded arena core and alignment checks
Step 2 — Named M2 arena set
Step 3 — Deterministic exhaustion diagnostics
Step 4 — Memory-map and arena dump
Step 5 — M2 completion audit
```

---

# Step 1 — Bounded arena core and alignment checks

Status: Completed.

Purpose:
  Implement the reusable bounded arena cursor contract with explicit alignment
  rejection and overflow/exhaustion errors.

Allowed files:
```
kernel/src/arena.rs
scripts/verify.py
docs/campaigns/m2-arena-memory.md
```

Required work:
  - Define an `Arena` structure with id, base, cursor, and limit.
  - Implement `alloc_aligned(size, alignment)` with spec §4.8 semantics.
  - Reject non-power-of-two alignments deterministically.
  - Add verification coverage for alignment, overflow, and exhaustion.

Validation:
```
make fmt
make clippy
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 2 — Named M2 arena set

Status: Completed.

Purpose:
  Materialize BootArena, KernelArena, GraphArena, and ScratchArena as named
  bounded arenas with declared lifetime/phase comments.

Allowed files:
```
kernel/src/arena.rs
kernel/src/boot.rs
docs/campaigns/m2-arena-memory.md
```

Required work:
  - Provide constructors or statics for Boot, Kernel, Graph, and Scratch arenas.
  - Keep direct allocation behind named arena methods or guard helpers.
  - Preserve normal M0/M1 boot behavior.

Validation:
```
make fmt
make clippy
make kernel
make qemu-headless
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 3 — Deterministic exhaustion diagnostics

Status: Completed.

Purpose:
  Ensure arena exhaustion returns structured context and fatal boot/kernel
  exhaustion can route through SSOD with arena identity.

Allowed files:
```
kernel/src/arena.rs
kernel/src/ssod.rs
docs/campaigns/m2-arena-memory.md
```

Required work:
  - Include arena id, requested size, alignment, base, cursor, and limit in
    exhaustion diagnostics.
  - Keep graph/model/node ids absent or explicit `none` until those systems
    exist.

Validation:
```
make fmt
make clippy
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 4 — Memory-map and arena dump

Purpose:
  Make the M2 diagnostic dump available on serial without claiming a full
  Limine handoff if the M0 smoke boot path is still active.

Allowed files:
```
kernel/src/arena.rs
kernel/src/boot.rs
scripts/qemu.sh
Makefile
docs/campaigns/m2-arena-memory.md
```

Required work:
  - Emit a stable serial dump for current memory-map/arena state.
  - If real memory-map ingestion is unavailable in the smoke profile, dump an
    explicit `unavailable` state rather than pretending usable ranges exist.
  - Add a QEMU or source-level assertion that the dump is present.

Validation:
```
make fmt
make clippy
make kernel
make qemu-headless
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 5 — M2 completion audit

Purpose:
  Close M2 only after the arena contract, named arenas, exhaustion behavior,
  and memory-map dump are all reproducibly verified.

Allowed files:
```
MILESTONE_CATALOG.md
docs/campaigns/m2-arena-memory.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
.codex/MISSION_LOG.md
```

Required work:
  - Change row M2 `Status` from `IN-PROGRESS` to `DONE`.
  - Bump the catalog version banner.
  - Add a change-log entry under `## Change log` summarizing M2.
  - In this campaign file, append a `## Closeout` section noting the commit
    SHAs of Steps 1-4.

Validation:
```
make gates
make repo-state
python3 scripts/verify.py --mission current
```

Commit and push.
