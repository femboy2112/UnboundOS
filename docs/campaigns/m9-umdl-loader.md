# M9 UMDL Loader Campaign

## Campaign status

```
STEP-COMMIT / PUSH-EVERY-STEP / BRANCH=campaign/m9-umdl-loader
```

## Mandatory files to read

```
CLAUDE.md
MILESTONE_CATALOG.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
.codex/PROJECT_PLAN.md
crates/umdl/src/lib.rs
crates/llm/src/lib.rs
crates/llm/src/toy_transformer.rs
tests/golden_models/README.md
```

## Strategic target

Close M9 by proving the spec §13.11 local-LLM criterion:

```
Model package validates and loads.
```

Spec §10 requires `.UMDL` to be persistent symbolic data: header, tokenizer
metadata, tensor descriptors, weight blob, checksums, and declared memory
requirements. M9 validates and loads a package into a read-only model view
contract. M10 owns quantized kernel execution.

## Baseline

- M8 completed at commit `91650da`.
- `crates/umdl` has fixed-width UMDL header, tokenizer metadata, tensor
  descriptor, quantization, SIMD tier, and error enum scaffolding.
- `tests/golden_models` exists but has no concrete `.UMDL` fixture yet.

## Non-negotiable boundaries

```
H1  no raw pointers/addresses/paths in persistent UMDL data.
H3  no hidden execution; loading validates data, it does not run inference.
H5  no generated-code exec; model data remains data.
H6  SIMD dispatch is declared only; no backend-specific symbol calls in M9.
H7  named arenas; loader reports required model/scratch/KV budgets explicitly.
H8  resource IDs only; no host paths become graph-visible model references.
```

Memory-unsafe Rust remains allowed by project identity, but M9 parsing and
validation should not need new unsafe code. If later arena-copy or SIMD work
requires unsafe access, it must be bounded, inspectable, deterministic, and not
undefined by design.

## Macro sequence

```
Step 1 — UMDL header parse and fixed-width contract
Step 2 — Section bounds and checksum validation
Step 3 — Tokenizer and tensor descriptor validation
Step 4 — Model load view and arena reservation contract
Step 5 — UMDL smoke fixtures and gates
Step 6 — M9 completion audit
```

---

# Step 1 — UMDL header parse and fixed-width contract

Status: Completed.

Purpose:
  Parse a UMDL header from bytes without allocation, pointers, host paths, or
  unsafe code.

Allowed files:
```
crates/umdl/src/lib.rs
docs/campaigns/m9-umdl-loader.md
```

Required work:
  - Add little-endian parsing for `UmdlHeader` from a caller-provided byte
    slice.
  - Validate magic, format version, and minimum header length.
  - Preserve fixed-width layout tests and add malformed-header tests.
  - Do not validate sections, tensors, or checksums yet.

Validation:
```
make fmt
make clippy
cargo test -p umdl
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 2 — Section bounds and checksum validation

Status: Completed.

Purpose:
  Prove UMDL section ranges are finite, non-overlapping where required, and
  covered by deterministic checksums.

Allowed files:
```
crates/umdl/src/lib.rs
docs/campaigns/m9-umdl-loader.md
```

Required work:
  - Validate tokenizer, tensor, weight, and checksum section offsets/lengths
    against the input byte length.
  - Add deterministic checksum helpers and header/section checksum checks.
  - Return structured `UmdlLoadError` variants, not panics.

Validation:
```
make fmt
make clippy
cargo test -p umdl
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 3 — Tokenizer and tensor descriptor validation

Status: Completed.

Purpose:
  Validate tokenizer metadata and tensor descriptor tables without loading
  executable code or backend-specific kernels.

Allowed files:
```
crates/umdl/src/lib.rs
docs/campaigns/m9-umdl-loader.md
```

Required work:
  - Parse `TokenizerMetadata` and `TensorDesc` entries from UMDL sections.
  - Validate supported tokenizer metadata through the existing raw-byte
    contract.
  - Validate tensor scalar/quant IDs, rank/dim shape, alignment, and
    weight-blob bounds.

Validation:
```
make fmt
make clippy
cargo test -p umdl
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 4 — Model load view and arena reservation contract

Status: Completed.

Purpose:
  Expose a read-only loaded model view and explicit arena requirements without
  allocating hidden storage.

Allowed files:
```
crates/umdl/src/lib.rs
crates/llm/src/lib.rs
docs/campaigns/m9-umdl-loader.md
```

Required work:
  - Add a loaded-model view carrying validated header/tokenizer/tensor counts
    and byte ranges.
  - Report required model, scratch, and KV-cache reservation bytes explicitly.
  - Validate minimum SIMD tier against an available tier argument.
  - Do not call tensor kernels or graph mutation surfaces.

Validation:
```
make fmt
make clippy
cargo test -p umdl
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 5 — UMDL smoke fixtures and gates

Purpose:
  Make UMDL loader evidence reproducible from checkout.

Allowed files:
```
Makefile
scripts/**
crates/umdl/src/**
tests/golden_models/**
tests/fuzz_corpus/umdl/**
docs/campaigns/m9-umdl-loader.md
```

Required work:
  - Add a deterministic golden `.UMDL` fixture or fixture generator and a
    malformed corpus entry.
  - Add `make umdl-smoke` or equivalent source/fixture check.
  - Wire UMDL smoke into aggregate mission verification.
  - Keep QEMU and graph gates green.

Validation:
```
make fmt
make clippy
make umdl-smoke
make gates
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 6 — M9 completion audit

Purpose:
  Close M9 after UMDL parsing, validation, load-view, arena reservation, and
  smoke evidence are reproducibly verified.

Allowed files:
```
MILESTONE_CATALOG.md
docs/campaigns/m9-umdl-loader.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
.codex/MISSION_LOG.md
```

Required work:
  - Change row M9 `Status` from `IN-PROGRESS` to `DONE`.
  - Bump the catalog version banner.
  - Add a change-log entry under `## Change log` summarizing M9.
  - In this campaign file, append a `## Closeout` section noting the commit
    SHAs of Steps 1-5.

Validation:
```
make gates
make repo-state
python3 scripts/verify.py --mission current
```

Commit and push.
