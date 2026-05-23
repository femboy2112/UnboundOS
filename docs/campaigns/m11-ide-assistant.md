# M11 IDE Assistant Campaign

## Campaign status

```
STEP-COMMIT / PUSH-EVERY-STEP / BRANCH=campaign/m11-ide-assistant
```

## Mandatory files to read

```
CLAUDE.md
MILESTONE_CATALOG.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
.codex/PROJECT_PLAN.md
crates/llm/src/lib.rs
crates/graph/src/lib.rs
crates/graph/src/loader.rs
kernel/src/ssod.rs
```

## Strategic target

Close M11 by proving the spec §13.1 assistant criterion:

```
Local assistant explains graph and SSOD.
```

M11 is explanation and proposal infrastructure, not autonomous mutation. It
must keep assistant output as structured data until validation, graph
verification, operator approval, and reload.

## Baseline

- M10 completed at commit `3c5ee2d`.
- `crates/llm` has local tokenizer, toy transformer, UMDL loader integration,
  scalar quantized inference, and a placeholder `StructuredActionBuffer`.
- Graph display snapshots and SSOD diagnostic records already exist in earlier
  milestones.

## Non-negotiable boundaries

```
H2  graph runtime path stays graph_load_from_umod -> verifier -> compile.
H3  no hidden execution or assistant worker loop.
H4  assistant output cannot mutate state directly.
H5  generated code remains text, never executable code.
H8  assistant-visible resources stay opaque IDs.
H10 SSOD explanation must preserve structured fatal diagnostics.
```

Memory-unsafe Rust remains allowed by project identity, but M11 should not need
new unsafe code. Explanation/action-buffer work must be safe, deterministic,
bounded, and non-executing.

## Macro sequence

```
Step 1 — Structured action buffer contract
Step 2 — Graph explanation snapshot
Step 3 — SSOD explanation snapshot
Step 4 — Assistant explanation surface
Step 5 — Assistant smoke evidence and gates
Step 6 — M11 completion audit
```

---

# Step 1 — Structured action buffer contract

Status: Completed.

Purpose:
  Replace the placeholder assistant action surface with a bounded data-only
  proposal buffer.

Allowed files:
```
crates/llm/src/lib.rs
crates/llm/src/assistant.rs
docs/campaigns/m11-ide-assistant.md
```

Required work:
  - Add fixed-width action proposal records and buffer metadata.
  - Enforce caller-provided storage and deterministic overflow errors.
  - Preserve the rule that proposals are data, not graph mutations.
  - Do not add unsafe code, threads, queues, eval, or execution hooks.

Validation:
```
make fmt
make clippy
cargo test -p llm
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 2 — Graph explanation snapshot

Status: Completed.

Purpose:
  Produce deterministic text/data explanations from verified graph display
  state.

Allowed files:
```
crates/graph/src/lib.rs
crates/llm/src/assistant.rs
docs/campaigns/m11-ide-assistant.md
```

Required work:
  - Expose a read-only graph explanation input shape from existing graph display
    snapshot data.
  - Format graph identity, node/wire counts, active node, and last completed
    node into caller-provided output.
  - Do not construct or mutate `GraphRuntime`.

Validation:
```
make fmt
make clippy
cargo test -p graph
cargo test -p llm
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 3 — SSOD explanation snapshot

Status: Completed.

Purpose:
  Produce deterministic explanations from structured SSOD diagnostic records.

Allowed files:
```
kernel/src/ssod.rs
crates/llm/src/assistant.rs
docs/campaigns/m11-ide-assistant.md
```

Required work:
  - Add or reuse fixed-width SSOD diagnostic snapshot fields.
  - Format reason/RIP/fault-family style information into caller-provided
    output.
  - Preserve H10: explanation must not swallow or weaken fatal diagnostics.

Validation:
```
make fmt
make clippy
cargo test -p llm
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 4 — Assistant explanation surface

Status: Active.

Purpose:
  Provide a single local assistant explain surface for graph and SSOD states.

Allowed files:
```
crates/llm/src/assistant.rs
crates/llm/src/lib.rs
docs/campaigns/m11-ide-assistant.md
```

Required work:
  - Add an explicit assistant request/response surface for graph and SSOD
    explanation.
  - Route proposed actions only into `StructuredActionBuffer`.
  - Reject unsupported request kinds with structured errors.

Validation:
```
make fmt
make clippy
cargo test -p llm
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 5 — Assistant smoke evidence and gates

Purpose:
  Make assistant explanation and action-buffer evidence reproducible from
  checkout.

Allowed files:
```
Makefile
scripts/**
crates/llm/src/**
crates/graph/src/**
kernel/src/ssod.rs
docs/campaigns/m11-ide-assistant.md
```

Required work:
  - Add `make assistant-smoke` or equivalent source-level check.
  - Prove graph explanation, SSOD explanation, action-buffer, and no-direct
    mutation evidence are source-reachable.
  - Wire assistant smoke into aggregate mission verification.
  - Keep QEMU and graph gates green.

Validation:
```
make fmt
make clippy
make assistant-smoke
make gates
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 6 — M11 completion audit

Purpose:
  Close M11 after assistant action-buffer, graph explanation, SSOD explanation,
  unified explain surface, and smoke evidence are reproducibly verified.

Allowed files:
```
MILESTONE_CATALOG.md
docs/campaigns/m11-ide-assistant.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
.codex/MISSION_LOG.md
```

Required work:
  - Change row M11 `Status` from `IN-PROGRESS` to `DONE`.
  - Bump the catalog version banner.
  - Add a change-log entry under `## Change log` summarizing M11.
  - In this campaign file, append a `## Closeout` section noting the commit
    SHAs of Steps 1-5.

Validation:
```
make gates
make repo-state
python3 scripts/verify.py --mission current
```

Commit and push.
