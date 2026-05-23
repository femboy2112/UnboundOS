# M12 Local Retrieval Campaign

## Campaign status

```
STEP-COMMIT / PUSH-EVERY-STEP / BRANCH=campaign/m12-local-retrieval
```

## Mandatory files to read

```
CLAUDE.md
MILESTONE_CATALOG.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
.codex/PROJECT_PLAN.md
crates/llm/src/lib.rs
crates/llm/src/assistant.rs
```

## Strategic target

Close M12 by proving the spec §13.1 retrieval criterion:

```
Assistant searches local docs.
```

M12 is local retrieval and context packaging, not host filesystem browsing or
autonomous mutation. Document references must be fixed-width data or opaque
resource IDs, and assistant-visible results remain explanatory context until
operator-approved graph verification paths consume them.

## Baseline

- M11 completed at commit `8f91be3`.
- `crates/llm` has assistant graph/SSOD explanation surfaces, caller-owned
  action proposal buffers, and `make assistant-smoke`.
- No M12 retrieval campaign file existed before this activation.

## Non-negotiable boundaries

```
H3  no hidden execution or assistant worker loop.
H4  assistant output cannot mutate state directly.
H5  generated code remains text, never executable code.
H8  assistant-visible document/storage references stay opaque IDs.
H10 retrieval context must not weaken SSOD diagnostics.
```

Memory-unsafe Rust remains allowed by project identity, but M12 should not need
new unsafe code. Retrieval/context-packing work must be safe, deterministic,
bounded, and non-executing.

## Macro sequence

```
Step 1 — Retrieval data contracts
Step 2 — Local document index snapshot
Step 3 — Deterministic retrieval ranking
Step 4 — Context packing
Step 5 — Assistant retrieval surface
Step 6 — Retrieval smoke evidence and gates
Step 7 — M12 completion audit
```

---

# Step 1 — Retrieval data contracts

Status: Completed.

Purpose:
  Add fixed-width retrieval query, document reference, and result records.

Allowed files:
```
crates/llm/src/lib.rs
crates/llm/src/retrieval.rs
docs/campaigns/m12-local-retrieval.md
```

Required work:
  - Add a `retrieval` module under `crates/llm`.
  - Define fixed-width query, document reference, and result records using
    caller-owned buffers and bounded text/resource fields.
  - Reject host paths, `local://`, and oversized text deterministically.
  - Do not add unsafe code, filesystem access, threads, queues, eval, graph
    mutation, or execution hooks.

Validation:
```
make fmt
make clippy
cargo test -p llm
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 2 — Local document index snapshot

Status: Active.

Purpose:
  Represent a read-only local document index snapshot from fixed document
  records.

Allowed files:
```
crates/llm/src/retrieval.rs
docs/campaigns/m12-local-retrieval.md
```

Required work:
  - Add caller-owned document slice/index snapshot APIs.
  - Preserve opaque document/resource IDs above storage adapters.
  - Add tests for empty index, duplicate IDs, and invalid references.

Validation:
```
make fmt
make clippy
cargo test -p llm
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 3 — Deterministic retrieval ranking

Purpose:
  Return deterministic top-k local document matches into caller-provided output.

Allowed files:
```
crates/llm/src/retrieval.rs
docs/campaigns/m12-local-retrieval.md
```

Required work:
  - Implement deterministic query matching and stable tie-breaking.
  - Write ranked results only into caller-provided output.
  - Report output overflow and unsupported query shapes with structured errors.

Validation:
```
make fmt
make clippy
cargo test -p llm
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 4 — Context packing

Purpose:
  Pack retrieved document snippets into bounded assistant context.

Allowed files:
```
crates/llm/src/retrieval.rs
docs/campaigns/m12-local-retrieval.md
```

Required work:
  - Add deterministic context packing into caller-provided byte output.
  - Preserve document IDs and snippet boundaries in the packed context.
  - Reject overflow without truncating silently.

Validation:
```
make fmt
make clippy
cargo test -p llm
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 5 — Assistant retrieval surface

Purpose:
  Connect local retrieval results to the assistant data surface.

Allowed files:
```
crates/llm/src/assistant.rs
crates/llm/src/retrieval.rs
docs/campaigns/m12-local-retrieval.md
```

Required work:
  - Add an explicit assistant retrieval request/response surface.
  - Keep retrieval output explanatory context, not graph mutation.
  - Route optional proposed actions only through `StructuredActionBuffer`.

Validation:
```
make fmt
make clippy
cargo test -p llm
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 6 — Retrieval smoke evidence and gates

Purpose:
  Make local retrieval evidence reproducible from checkout.

Allowed files:
```
Makefile
scripts/**
crates/llm/src/**
docs/campaigns/m12-local-retrieval.md
```

Required work:
  - Add `make retrieval-smoke` or equivalent source-level check.
  - Prove retrieval contracts, ranking, context packing, assistant retrieval
    routing, and no host-path/no-direct-mutation evidence are source-reachable.
  - Wire retrieval smoke into aggregate mission verification.
  - Keep aggregate gates green.

Validation:
```
make fmt
make clippy
make retrieval-smoke
make gates
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 7 — M12 completion audit

Purpose:
  Close M12 after retrieval contracts, document snapshot, deterministic
  ranking, context packing, assistant retrieval surface, and smoke evidence are
  reproducibly verified.

Allowed files:
```
MILESTONE_CATALOG.md
docs/campaigns/m12-local-retrieval.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
.codex/MISSION_LOG.md
```

Required work:
  - Change row M12 `Status` from `IN-PROGRESS` to `DONE`.
  - Bump the catalog version banner.
  - Add a change-log entry under `## Change log` summarizing M12.
  - In this campaign file, append a `## Closeout` section noting the commit
    SHAs of Steps 1-6.

Validation:
```
make gates
make repo-state
python3 scripts/verify.py --mission current
```

Commit and push.
