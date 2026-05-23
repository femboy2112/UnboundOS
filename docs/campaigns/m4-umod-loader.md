# M4 UMOD Loader Campaign

## Campaign status

```
STEP-COMMIT / PUSH-EVERY-STEP / BRANCH=campaign/m4-umod-loader
```

## Mandatory files to read

```
CLAUDE.md
MILESTONE_CATALOG.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
crates/umod/src/lib.rs
crates/graph/src/lib.rs
crates/graph/src/verifier.rs
crates/graph/src/loader.rs
tests/golden_graphs/registry.toml
tests/fuzz_corpus/README.md
```

## Strategic target

After this campaign closes, M4 proves the spec §13.6 UMOD-loader exit
criteria:

```
Persistent UMOD bytes parse into symbolic descriptors.
All 22 spec §5.6 graph verifier checks reject malformed input with structured errors.
A valid persistent UMOD verifies and compiles through graph_load_from_umod -> graph_compile_verified.
Golden and malformed fixture coverage is non-vacuous.
```

H2 remains binding. Tests may construct symbolic UMOD byte buffers, but every
runtime handle must still come from `graph_load_from_umod` followed by
`graph_compile_verified`.

## Baseline

- M3 completed at commit `9ee75c7`.
- `crates/umod` defines fixed-width UMOD layout structs, but has no parser.
- `crates/graph/src/verifier.rs` names the 22 checks, but checks 2-22 are
  currently stubs.
- No active golden `.umod` fixtures or malformed UMOD corpus entries exist yet.
- The public type gate exists: external callers cannot construct
  `VerifiedGraph` directly.

## Design thesis

M4 should turn the existing verifier-shaped API into a real persistent-UMOD
gate without widening runtime construction. Parser work belongs in `crates/umod`
as bounded symbolic decoding. Semantic graph verification belongs in
`crates/graph/src/verifier.rs`. Runtime construction remains private to
`loader.rs`.

## Non-negotiable boundaries

```
H1  no persistent pointers — UMOD contains only symbolic fixed-width fields.
H2  single verifier gate   — no direct GraphRuntime constructor outside loader.
H3  no hidden execution    — no execution before verification completes.
H4  LLM never mutates      — M4 has no LLM path.
H5  no eval node           — no generated/eval execution.
H6  no SIMD assumption     — M4 does not dispatch tensor kernels.
H7  named arenas           — runtime allocation remains GraphArena-owned.
H8  resource IDs           — external refs must use approved opaque syntax only.
H9  boot is never blind    — preserve existing heartbeat diagnostics.
H10 structured failures    — malformed UMOD returns typed load errors, not panics.
```

## Allowed scope summary

```
crates/umod/src/lib.rs
crates/graph/src/lib.rs
crates/graph/src/verifier.rs
crates/graph/src/loader.rs
tests/golden_graphs/**
tests/fuzz_corpus/umod/**
scripts/verify.py
docs/campaigns/m4-umod-loader.md
MILESTONE_CATALOG.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
.codex/MISSION_LOG.md
```

## Macro sequence

```
Step 1 — UMOD parser header and resource refs
Step 2 — Section table bounds and structural checks
Step 3 — Node and wire semantic verifier checks
Step 4 — Capabilities, resources, constants, and scheduling checks
Step 5 — Persistent UMOD compile path
Step 6 — Golden and malformed fixture coverage
Step 7 — M4 completion audit
```

---

# Step 1 — UMOD parser header and resource refs

Status: Completed.

Purpose:
  Add bounded parser primitives for UMOD headers and opaque resource
  references, replacing parser stubs with typed errors.

Allowed files:
```
crates/umod/src/lib.rs
crates/graph/src/verifier.rs
docs/campaigns/m4-umod-loader.md
```

Required work:
  - Decode the UMOD header from little-endian bytes without pointer casts.
  - Reject short headers, bad magic, unsupported versions, and bad header
    lengths with structured errors.
  - Implement the approved opaque resource reference grammar.
  - Add focused `umod` and `graph` crate tests for the parser boundary.

Validation:
```
make fmt
make clippy
cargo test -p umod
cargo test -p graph
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 2 — Section table bounds and structural checks

Status: Completed.

Purpose:
  Parse section descriptors and make checks 3-6 non-vacuous for section table
  validity, file length, count limits, and overflow behavior.

Allowed files:
```
crates/umod/src/lib.rs
crates/graph/src/lib.rs
crates/graph/src/verifier.rs
tests/fuzz_corpus/umod/**
docs/campaigns/m4-umod-loader.md
```

Required work:
  - Decode section descriptors through fixed-width little-endian reads.
  - Reject section tables outside the file, section offset/length overflow,
    out-of-file sections, and illegal overlaps.
  - Enforce configured node/wire count limits before semantic checks.
  - Add malformed corpus entries or tests for each structural rejection.

Validation:
```
make fmt
make clippy
cargo test -p umod
cargo test -p graph
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 3 — Node and wire semantic verifier checks

Status: Completed.

Purpose:
  Implement graph topology checks for node resolution, wire endpoints, pin
  indices, wire type compatibility, node type registration, and cycle rules.

Allowed files:
```
crates/umod/src/lib.rs
crates/graph/src/lib.rs
crates/graph/src/verifier.rs
tests/fuzz_corpus/umod/**
docs/campaigns/m4-umod-loader.md
```

Required work:
  - Parse or expose enough node/wire descriptors for semantic verification.
  - Make checks 7-13 return typed `GraphLoadError` variants.
  - Reject unbroken cycles unless an explicit delay/state node is present.
  - Keep all runtime allocation out of the verifier.

Validation:
```
make fmt
make clippy
cargo test -p umod
cargo test -p graph
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 4 — Capabilities, resources, constants, and scheduling checks

Status: Completed.

Purpose:
  Complete checks 12 and 14-22 for capabilities, payload bounds, GraphArena
  budget, model/resource references, checksums, UI layout, constants, and
  deterministic scheduling requirements.

Allowed files:
```
crates/umod/src/lib.rs
crates/graph/src/lib.rs
crates/graph/src/verifier.rs
tests/fuzz_corpus/umod/**
docs/campaigns/m4-umod-loader.md
```

Required work:
  - Make checks 14-22 non-vacuous.
  - Ensure external references delegate to the approved resource grammar.
  - Return structured graph-load errors for every malformed case.
  - Preserve no-panic behavior on arbitrary malformed bytes.

Validation:
```
make fmt
make clippy
cargo test -p umod
cargo test -p graph
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 5 — Persistent UMOD compile path

Purpose:
  Compile a valid persistent UMOD through the existing verified path into the
  private runtime graph surface.

Allowed files:
```
crates/graph/src/lib.rs
crates/graph/src/verifier.rs
crates/graph/src/loader.rs
tests/golden_graphs/**
docs/campaigns/m4-umod-loader.md
```

Required work:
  - Add or generate a minimal persistent UMOD that represents source ->
    transform -> sink.
  - Verify it with `graph_load_from_umod` and compile it with
    `graph_compile_verified`.
  - Ensure no public runtime constructor or test-only bypass is added.

Validation:
```
make fmt
make clippy
cargo test -p graph
python3 scripts/address_scan.py tests/golden_graphs tests/golden_models
python3 scripts/verify.py --mission current
make gates
```

Commit and push.

---

# Step 6 — Golden and malformed fixture coverage

Purpose:
  Make M4 fixture coverage non-vacuous and reproducible from checkout.

Allowed files:
```
tests/golden_graphs/**
tests/fuzz_corpus/umod/**
crates/umod/src/lib.rs
crates/graph/src/verifier.rs
scripts/verify.py
docs/campaigns/m4-umod-loader.md
```

Required work:
  - Register at least one valid golden UMOD fixture.
  - Add malformed UMOD cases for bad magic/version, truncated header,
    out-of-bounds sections, overlap, huge counts, invalid refs, and unbroken
    cycles.
  - Ensure the verification bundle exercises the fixture set.

Validation:
```
make fmt
make clippy
cargo test -p umod
cargo test -p graph
python3 scripts/address_scan.py tests/golden_graphs tests/golden_models
python3 scripts/verify.py --mission current
make gates
```

Commit and push.

---

# Step 7 — M4 completion audit

Purpose:
  Close M4 after persistent UMOD parsing, 22-check verification, compile-path
  execution, and fixture coverage are all reproducibly verified.

Allowed files:
```
MILESTONE_CATALOG.md
docs/campaigns/m4-umod-loader.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
.codex/MISSION_LOG.md
```

Required work:
  - Change row M4 `Status` from `IN-PROGRESS` to `DONE`.
  - Bump the catalog version banner.
  - Add a change-log entry under `## Change log` summarizing M4.
  - In this campaign file, append a `## Closeout` section noting the commit
    SHAs of Steps 1-6.

Validation:
```
make gates
make repo-state
python3 scripts/verify.py --mission current
```

Commit and push.
