# M10 Quantized Inference Campaign

## Campaign status

```
STEP-COMMIT / PUSH-EVERY-STEP / BRANCH=campaign/m10-quantized-inference
```

## Mandatory files to read

```
CLAUDE.md
MILESTONE_CATALOG.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
.codex/PROJECT_PLAN.md
crates/llm/src/lib.rs
crates/llm/src/dispatch.rs
crates/llm/src/toy_transformer.rs
crates/umdl/src/lib.rs
docs/campaigns/m9-umdl-loader.md
```

## Strategic target

Close M10 by proving the spec §13.12 local-LLM criterion:

```
Small quantized model streams tokens.
```

M10 builds the first deterministic CPU quantized inference path on top of M9's
validated UMDL model view. SIMD-specific kernels remain future work unless
introduced behind the dispatch boundary; the first path is scalar and
deterministic.

## Baseline

- M9 completed at commit `26c801c`.
- UMDL packages validate and expose read-only model views plus arena
  reservations.
- `crates/llm/src/dispatch.rs` has a dispatch-table stub but no real scalar
  quantized kernels.

## Non-negotiable boundaries

```
H3  no hidden execution; token streaming is an explicit callable surface.
H4  generated tokens cannot mutate graph state directly.
H5  model bytes are data, never executable code.
H6  backend-specific SIMD symbols are reachable only through dispatch/kernels.
H7  caller-provided buffers and explicit arena requirements only.
H8  model references stay opaque resource IDs; no host paths in graph state.
```

Memory-unsafe Rust remains allowed by project identity. M10 starts with scalar
safe Rust; any later SIMD unsafe must be isolated under `crates/llm/src/kernels`,
selected only through dispatch, and remain bounded, inspectable,
deterministic, and not undefined by design.

## Macro sequence

```
Step 1 — Scalar quantized kernel contracts
Step 2 — Dispatch-selected scalar kernel table
Step 3 — Deterministic quantized token step
Step 4 — Streaming token surface
Step 5 — Quantized inference smoke evidence and gates
Step 6 — M10 completion audit
```

---

# Step 1 — Scalar quantized kernel contracts

Status: Completed.

Purpose:
  Add scalar quantized kernel contracts and deterministic tests without
  touching SIMD-specific backends.

Allowed files:
```
crates/llm/src/lib.rs
crates/llm/src/dispatch.rs
crates/llm/src/kernels/**
docs/campaigns/m10-quantized-inference.md
```

Required work:
  - Add a scalar kernel module for small fixed-slice quantized math.
  - Define caller-provided input/output buffer contracts.
  - Add deterministic scalar tests for a tiny quantized projection.
  - Do not add unsafe code or backend-specific SIMD symbols.

Validation:
```
make fmt
make clippy
cargo test -p llm
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 2 — Dispatch-selected scalar kernel table

Status: Completed.

Purpose:
  Route graph-facing tensor calls through the loader-selected dispatch table,
  initially selecting scalar kernels only.

Allowed files:
```
crates/llm/src/lib.rs
crates/llm/src/dispatch.rs
crates/llm/src/kernels/**
docs/campaigns/m10-quantized-inference.md
```

Required work:
  - Replace placeholder dispatch entries for the first quantized kernel with
    scalar implementations.
  - Preserve `dispatch.rs` and `kernels/**` as the only legal backend-symbol
    reference locations.
  - Add tests proving graph-facing calls use table entries.

Validation:
```
make fmt
make clippy
cargo test -p llm
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 3 — Deterministic quantized token step

Status: Completed.

Purpose:
  Produce one deterministic next-token step from a validated model view and
  caller-provided buffers.

Allowed files:
```
crates/llm/src/**
crates/umdl/src/lib.rs
docs/campaigns/m10-quantized-inference.md
```

Required work:
  - Add a small quantized inference step that consumes validated model metadata.
  - Use caller-provided prompt/logit/output buffers.
  - Return structured overflow/config errors instead of panics.
  - Do not mutate graph state or call backend-specific symbols directly.

Validation:
```
make fmt
make clippy
cargo test -p llm
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 4 — Streaming token surface

Status: Completed.

Purpose:
  Stream deterministic tokens through an explicit callable surface.

Allowed files:
```
crates/llm/src/**
docs/campaigns/m10-quantized-inference.md
```

Required work:
  - Add a streaming state/config object with explicit caller-owned buffers.
  - Produce stable token sequences for a tiny prompt/model/config.
  - Keep generated tokens as output data only; no graph mutation authority.

Validation:
```
make fmt
make clippy
cargo test -p llm
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 5 — Quantized inference smoke evidence and gates

Status: Completed.

Purpose:
  Make quantized inference evidence reproducible from checkout.

Allowed files:
```
Makefile
scripts/**
crates/llm/src/**
docs/campaigns/m10-quantized-inference.md
```

Required work:
  - Add `make quantized-smoke` or equivalent source-level check.
  - Prove scalar kernels, dispatch routing, deterministic token step, and
    streaming tests are source-reachable.
  - Wire the smoke into aggregate mission verification.
  - Keep QEMU and graph gates green.

Validation:
```
make fmt
make clippy
make quantized-smoke
make gates
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 6 — M10 completion audit

Status: Completed.

Purpose:
  Close M10 after scalar quantized kernels, dispatch routing, deterministic
  token stepping, streaming, and smoke evidence are reproducibly verified.

Allowed files:
```
MILESTONE_CATALOG.md
docs/campaigns/m10-quantized-inference.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
.codex/MISSION_LOG.md
```

Required work:
  - Change row M10 `Status` from `IN-PROGRESS` to `DONE`.
  - Bump the catalog version banner.
  - Add a change-log entry under `## Change log` summarizing M10.
  - In this campaign file, append a `## Closeout` section noting the commit
    SHAs of Steps 1-5.

Validation:
```
make gates
make repo-state
python3 scripts/verify.py --mission current
```

Commit and push.

## Closeout

M10 is complete. Checkpoint commits:

- Step 1 Scalar quantized kernel contracts: `ec536f9`
- Step 2 Dispatch-selected scalar kernel table: `d2b494c`
- Step 3 Deterministic quantized token step: `008b7e8`
- Step 4 Streaming token surface: `1140303`
- Step 5 Quantized inference smoke evidence and gates: `0130ca8`

No new unsafe blocks or functions were required for M10. The quantized
inference path uses safe scalar kernels, dispatch-table routing,
caller-provided buffers, deterministic token stepping, explicit streaming
state, and source-level smoke evidence. Memory-unsafe Rust remains allowed by
project identity, but SIMD-specific unsafe work is still future work and must
remain isolated under `crates/llm/src/kernels/**` and selected only through
dispatch.

`/go` must stop here until the operator opens the final M10 PR or rotates the
control files to M11.
