# M3 Embedded Graph Campaign

## Campaign status

```
STEP-COMMIT / PUSH-EVERY-STEP / BRANCH=campaign/m3-embedded-graph
```

## Mandatory files to read

```
CLAUDE.md
MILESTONE_CATALOG.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
crates/graph/src/lib.rs
crates/graph/src/loader.rs
crates/graph/src/verifier.rs
kernel/src/boot.rs
```

## Strategic target

After this campaign closes, M3 proves the spec §13.5 embedded-graph exit
criteria:

```
Hardcoded graph source -> transform -> sink executes.
Epoch readiness works.
Fan-out test passes.
Active node diagnostics work.
```

H2 remains binding. The hardcoded graph is a built-in symbolic graph payload or
builder-owned verified input; runtime graph construction still occurs only in
the graph loader after verifier success.

## Baseline

- M2 completed at commit `f10a0ba`.
- `crates/graph` exposes `graph_load_from_umod -> graph_compile_verified` as
  the only legal verified/compiled path.
- `loader.rs` currently returns an opaque stub handle.
- Runtime node/wire, epoch observation, fan-out, and active-node diagnostics do
  not exist yet.

## Design thesis

M3 should introduce the smallest executable runtime graph surface while
preserving H2. The graph crate can own private runtime structures and tests for
epoch/fan-out behavior, but public callers must keep using the verified graph
pipeline and opaque handles.

## Non-negotiable boundaries

```
H1  no persistent pointers — embedded graph input is symbolic.
H2  single verifier gate   — no direct GraphRuntime constructor outside loader.
H3  no hidden execution    — work happens through graph nodes/orchestrator.
H4  LLM never mutates      — M3 has no LLM path.
H5  no eval node           — no generated/eval execution.
H6  no SIMD assumption     — graph execution is scalar/trivial.
H7  named arenas           — runtime allocation must target GraphArena once wired.
H8  resource IDs           — no POSIX or local path graph refs.
H9  boot is never blind    — preserve existing heartbeat diagnostics.
H10 SSOD for fatal         — preserve M1/M2 fatal diagnostics.
```

## Allowed scope summary

```
crates/graph/src/lib.rs
crates/graph/src/loader.rs
crates/graph/src/verifier.rs
kernel/src/boot.rs
scripts/verify.py
docs/campaigns/m3-embedded-graph.md
MILESTONE_CATALOG.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
.codex/MISSION_LOG.md
```

## Macro sequence

```
Step 1 — Runtime epoch readiness primitives
Step 2 — Private hardcoded graph runtime
Step 3 — Fan-out execution proof
Step 4 — Active node diagnostics
Step 5 — M3 completion audit
```

---

# Step 1 — Runtime epoch readiness primitives

Status: Completed.

Purpose:
  Add private runtime wire/consumer epoch observation primitives with tests that
  prove readiness is `wire_epoch > last_observed_epoch`.

Allowed files:
```
crates/graph/src/lib.rs
crates/graph/src/loader.rs
scripts/verify.py
docs/campaigns/m3-embedded-graph.md
```

Required work:
  - Add private runtime structures inside the graph crate.
  - Keep runtime construction private to `loader.rs` or its private child
    modules.
  - Test epoch readiness and observation transitions.

Validation:
```
make fmt
make clippy
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 2 — Private hardcoded graph runtime

Status: Completed.

Purpose:
  Implement a built-in source -> transform -> sink graph shape behind the
  verified compile path.

Allowed files:
```
crates/graph/src/lib.rs
crates/graph/src/loader.rs
crates/graph/src/verifier.rs
docs/campaigns/m3-embedded-graph.md
```

Required work:
  - Provide a symbolic built-in graph payload that passes
    `graph_load_from_umod`.
  - Compile it into private runtime structures through
    `graph_compile_verified`.
  - Execute source -> transform -> sink once in a graph-crate test.

Validation:
```
make fmt
make clippy
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 3 — Fan-out execution proof

Status: Completed.

Purpose:
  Prove one producer output can be observed by multiple consumers without
  either consumer erasing readiness for the other.

Allowed files:
```
crates/graph/src/lib.rs
crates/graph/src/loader.rs
docs/campaigns/m3-embedded-graph.md
```

Required work:
  - Add a fan-out graph-crate test.
  - Assert both consumers observe the same produced epoch.

Validation:
```
make fmt
make clippy
python3 scripts/verify.py --mission current
```

Commit and push.

---

# Step 4 — Active node diagnostics

Status: Completed.

Purpose:
  Track active node identity during graph execution and clear it after each
  node fires.

Allowed files:
```
crates/graph/src/lib.rs
crates/graph/src/loader.rs
kernel/src/ssod.rs
docs/campaigns/m3-embedded-graph.md
```

Required work:
  - Add active-node tracking inside graph runtime execution.
  - Surface the current active node in diagnostics shape without letting LLM or
    external code mutate graph state.

Validation:
```
make fmt
make clippy
python3 scripts/verify.py --mission current
make gates
```

Commit and push.

---

# Step 5 — M3 completion audit

Status: Completed.

Purpose:
  Close M3 after source -> transform -> sink, epoch readiness, fan-out, and
  active-node diagnostics are all verified.

Allowed files:
```
MILESTONE_CATALOG.md
docs/campaigns/m3-embedded-graph.md
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
.codex/MISSION_LOG.md
```

Required work:
  - Change row M3 `Status` from `IN-PROGRESS` to `DONE`.
  - Bump the catalog version banner.
  - Add a change-log entry under `## Change log` summarizing M3.
  - In this campaign file, append a `## Closeout` section noting the commit
    SHAs of Steps 1-4.

Validation:
```
make gates
make repo-state
python3 scripts/verify.py --mission current
```

Commit and push.

## Closeout

M3 completed on branch `campaign/m3-embedded-graph`.

Step commits:

- Step 1 — Runtime epoch readiness primitives: `46d415c`
- Step 2 — Private hardcoded graph runtime: `f2c150f`
- Step 3 — Fan-out execution proof: `9c5cb77`
- Step 4 — Active node diagnostics: `e74bd2f`

Final gates:

- `python3 scripts/verify.py --mission current`: ran graph crate tests covering
  built-in graph verification, verified compile path execution, source ->
  transform -> sink execution, epoch readiness, fan-out independence, and
  active-node clearing.
- `make gates`: PROCEED.
- `make repo-state`: STOP because no milestone remains `IN-PROGRESS`, which is
  the expected closed-M3 state.

Boundary note: M3 proves the embedded built-in graph runtime path while keeping
runtime graph internals private to the loader. It does not claim the M4 UMOD
parser, malformed UMOD structured errors, storage, UI, LLM, or persistent graph
fixture coverage.
