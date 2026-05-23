# M7 Tokenizer Campaign

## Campaign status

```
STEP-COMMIT / PUSH-EVERY-STEP / BRANCH=campaign/m7-tokenizer
```

## Mandatory files to read

```
CLAUDE.md
MILESTONE_CATALOG.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
crates/llm/src/lib.rs
crates/umdl/src/lib.rs
```

## Strategic target

Close M7 by proving the spec §13.7 LLM/tokenizer exit criterion:

```
Tokenizer round trip works.
```

Spec §10.7 requires a declared tokenizer type, metadata for tokenizer tables,
special token IDs, UTF-8 policy, maximum token byte length, checksum, and a
round-trip test for every supported tokenizer type.

## Baseline

- M6 completed at commit `c9033d7`.
- `umdl::TokenizerType` already declares the registry IDs.
- `crates/llm` has no tokenizer module or encode/decode API yet.

## Non-negotiable boundaries

```
H1  symbolic artifacts      — tokenizer metadata is fixed-width, no pointers.
H3  no hidden execution     — tokenization is a callable graph-node surface only.
H4  no direct mutation      — decoded/generated text cannot mutate state.
H5  no generated-code exec  — tokenizer data is data, not executable code.
H7  named arenas            — future table allocation belongs to TokenizerArena.
H8  resource IDs            — tokenizer/model refs remain opaque IDs.
```

Memory-unsafe Rust remains allowed by project identity, but M7 should not need
new unsafe code. If a later tokenizer table loader needs unsafe access, it must
be bounded, inspectable, deterministic, and not undefined by design.

## Supported tokenizer scope

M7 supports exactly one initial tokenizer family: `RawByteToToken`. It is the
tiny toy-model target from the spec registry and is enough to prove bare-metal
tokenizer round trips before M8 introduces a toy transformer. BPE and
SentencePiece remain unsupported until explicitly opened.

## Macro sequence

```
Step 1 — Tokenizer registry and metadata contract
Step 2 — Raw-byte tokenizer encode path
Step 3 — Raw-byte detokenizer round trip
Step 4 — Tokenizer smoke evidence and gates
Step 5 — M7 completion audit
```

---

# Step 1 — Tokenizer registry and metadata contract

Status: Completed.

Purpose:
  Define the supported tokenizer family and fixed-width metadata contract.

Allowed files:
```
crates/umdl/src/lib.rs
crates/llm/src/lib.rs
crates/llm/src/tokenizer.rs
docs/campaigns/m7-tokenizer.md
```

Required work:
  - Add tokenizer metadata types covering tokenizer type, vocabulary size,
    special token IDs, UTF-8 policy, maximum token byte length, and checksum.
  - Accept only `RawByteToToken` as supported in M7.
  - Reject BPE and SentencePiece as unsupported for now.
  - Keep persistent metadata fixed-width and pointer-free.

Validation:
```
make fmt
make clippy
cargo test -p umdl
cargo test -p llm
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 2 — Raw-byte tokenizer encode path

Status: Completed.

Purpose:
  Implement no-alloc UTF-8 byte-to-token encoding for caller-provided output
  storage.

Allowed files:
```
crates/llm/src/lib.rs
crates/llm/src/tokenizer.rs
docs/campaigns/m7-tokenizer.md
```

Required work:
  - Encode UTF-8 input bytes into stable token IDs.
  - Return structured overflow/invalid-metadata errors.
  - Use caller-provided token buffers; do not allocate hidden storage.

Validation:
```
make fmt
make clippy
cargo test -p llm
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 3 — Raw-byte detokenizer round trip

Status: Completed.

Purpose:
  Implement token-to-UTF-8 decoding and round-trip tests.

Allowed files:
```
crates/llm/src/lib.rs
crates/llm/src/tokenizer.rs
docs/campaigns/m7-tokenizer.md
```

Required work:
  - Decode stable raw-byte token IDs into caller-provided byte output.
  - Preserve valid UTF-8 round trips for representative prompts.
  - Return structured errors for invalid token IDs and output overflow.

Validation:
```
make fmt
make clippy
cargo test -p llm
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 4 — Tokenizer smoke evidence and gates

Purpose:
  Make tokenizer evidence reproducible from checkout.

Allowed files:
```
Makefile
scripts/**
crates/llm/src/**
docs/campaigns/m7-tokenizer.md
```

Required work:
  - Add a smoke target or source-level check proving exactly one tokenizer
    family is supported and round-trip tests exist.
  - Wire the smoke into aggregate mission verification.
  - Keep QEMU and graph gates green.

Validation:
```
make fmt
make clippy
make tokenizer-smoke
make gates
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 5 — M7 completion audit

Purpose:
  Close M7 after tokenizer metadata, encode/decode, round-trip tests, and smoke
  evidence are reproducibly verified.

Allowed files:
```
MILESTONE_CATALOG.md
docs/campaigns/m7-tokenizer.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
.codex/MISSION_LOG.md
```

Required work:
  - Change row M7 `Status` from `IN-PROGRESS` to `DONE`.
  - Bump the catalog version banner.
  - Add a change-log entry under `## Change log` summarizing M7.
  - In this campaign file, append a `## Closeout` section noting the commit
    SHAs of Steps 1-4.

Validation:
```
make gates
make repo-state
python3 scripts/verify.py --mission current
```

Commit and push.
