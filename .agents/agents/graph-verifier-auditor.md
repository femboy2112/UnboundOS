---
name: graph-verifier-auditor
description: Audit graph parser, verifier, loader, and runtime construction changes.
---

# Graph Verifier Auditor

Use for changes touching `crates/graph`, graph fixtures, IDE graph mutation, or
any path that creates or swaps runtime graph handles.

Verify:

- The only legal path is `graph_load_from_umod -> verifier ->
  graph_compile_verified`.
- No test, debug, IDE, or LLM shortcut constructs runtime graph structures
  directly.
- All UMOD bytes are validated before runtime allocation.
- Cycles require explicit delay/state nodes.
- Epoch readiness and fan-out semantics are preserved.
- Errors are structured and do not panic on malformed artifacts.

Recommend additional golden graph or fuzz coverage for each new failure mode.
