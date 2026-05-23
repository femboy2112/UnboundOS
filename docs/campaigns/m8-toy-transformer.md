# M8 Toy Transformer Campaign

## Campaign status

```
STEP-COMMIT / PUSH-EVERY-STEP / BRANCH=campaign/m8-toy-transformer
```

## Mandatory files to read

```
CLAUDE.md
MILESTONE_CATALOG.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
crates/llm/src/lib.rs
crates/llm/src/tokenizer.rs
crates/llm/src/dispatch.rs
crates/umdl/src/lib.rs
```

## Strategic target

Close M8 by proving the spec §13.7 local-LLM criterion:

```
Tiny model generates deterministic output.
```

Spec §10.8 says the first model architecture target should be one small
decoder-only transformer path with token embeddings, repeated blocks, attention
or a staged tiny equivalent, normalization, feed-forward/projection, and a
sampling loop. M8 intentionally uses a hardcoded toy model before M9 introduces
the `.UMDL` loader.

## Baseline

- M7 completed at commit `16f09a3`.
- Raw-byte tokenizer encode/decode round trips exist.
- `crates/llm` has dispatch scaffolding but no toy model or generation API.

## Non-negotiable boundaries

```
H3  no hidden execution     — generation is a callable graph-node/macro-node surface.
H4  no direct mutation      — output tokens/text cannot mutate state.
H5  no generated-code exec  — toy model data is data, not executable code.
H6  SIMD dispatch           — M8 must not call backend-specific SIMD symbols.
H7  named arenas            — generation uses caller-provided buffers only.
H8  resource IDs            — no model/storage paths enter graph-visible state.
```

Memory-unsafe Rust remains allowed by project identity, but M8 should not need
new unsafe code. If a later model loader or SIMD kernel requires unsafe access,
it must be bounded, inspectable, deterministic, and not undefined by design.

## Macro sequence

```
Step 1 — Toy model architecture contract
Step 2 — Deterministic token generation
Step 3 — Prompt-to-text toy inference path
Step 4 — Toy transformer smoke evidence and gates
Step 5 — M8 completion audit
```

---

# Step 1 — Toy model architecture contract

Status: Completed.

Purpose:
  Define the hardcoded toy model metadata, deterministic generation config, and
  caller-provided buffer contracts.

Allowed files:
```
crates/llm/src/lib.rs
crates/llm/src/toy_transformer.rs
docs/campaigns/m8-toy-transformer.md
```

Required work:
  - Add a toy model module with fixed-width model/config metadata.
  - Expose only one supported toy architecture for M8.
  - Define structured errors for buffer overflow and unsupported config.
  - Do not allocate hidden storage or introduce hidden execution.

Validation:
```
make fmt
make clippy
cargo test -p llm
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 2 — Deterministic token generation

Status: Completed.

Purpose:
  Generate deterministic token IDs from the hardcoded tiny model using
  caller-provided output storage.

Allowed files:
```
crates/llm/src/lib.rs
crates/llm/src/toy_transformer.rs
docs/campaigns/m8-toy-transformer.md
```

Required work:
  - Produce a stable token sequence for a prompt token stream and seed/config.
  - Same prompt, seed, config, and model must produce identical tokens.
  - Return structured overflow/config errors, not panics.
  - Do not call SIMD backend-specific symbols.

Validation:
```
make fmt
make clippy
cargo test -p llm
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 3 — Prompt-to-text toy inference path

Status: Completed.

Purpose:
  Connect M7 tokenizer encode/decode with the M8 deterministic toy generator.

Allowed files:
```
crates/llm/src/lib.rs
crates/llm/src/tokenizer.rs
crates/llm/src/toy_transformer.rs
docs/campaigns/m8-toy-transformer.md
```

Required work:
  - Accept a UTF-8 prompt, tokenize through `RawByteToToken`, generate new
    tokens, and decode to UTF-8 text using caller-provided buffers.
  - Preserve deterministic output for representative prompts.
  - Keep all state in explicit caller-provided buffers.

Validation:
```
make fmt
make clippy
cargo test -p llm
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 4 — Toy transformer smoke evidence and gates

Purpose:
  Make toy-model deterministic output evidence reproducible from checkout.

Allowed files:
```
Makefile
scripts/**
crates/llm/src/**
docs/campaigns/m8-toy-transformer.md
```

Required work:
  - Add a smoke target or source-level check proving toy-model deterministic
    output and prompt-to-text tests exist.
  - Wire the smoke into aggregate mission verification.
  - Keep QEMU and graph gates green.

Validation:
```
make fmt
make clippy
make toy-transformer-smoke
make gates
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 5 — M8 completion audit

Purpose:
  Close M8 after toy-model metadata, deterministic generation, prompt-to-text
  inference, and smoke evidence are reproducibly verified.

Allowed files:
```
MILESTONE_CATALOG.md
docs/campaigns/m8-toy-transformer.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
.codex/MISSION_LOG.md
```

Required work:
  - Change row M8 `Status` from `IN-PROGRESS` to `DONE`.
  - Bump the catalog version banner.
  - Add a change-log entry under `## Change log` summarizing M8.
  - In this campaign file, append a `## Closeout` section noting the commit
    SHAs of Steps 1-4.

Validation:
```
make gates
make repo-state
python3 scripts/verify.py --mission current
```

Commit and push.
