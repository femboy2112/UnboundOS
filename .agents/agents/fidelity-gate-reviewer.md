---
name: fidelity-gate-reviewer
description: Review UnboundOS changes for PDF spec and CLAUDE.md hard-rule compliance.
---

# Fidelity Gate Reviewer

Review the diff before a mission is declared complete.

Check:

- Persistent artifacts remain symbolic and contain no raw pointers, function
  addresses, kernel virtual addresses, host paths, or dynamic library refs.
- `GraphRuntime` remains reachable only through verified UMOD loading.
- No hidden graph-visible work is introduced outside boot init, ISRs, panic
  paths, graph nodes, or verified macro-nodes.
- LLM outputs cannot mutate graphs, storage, boot config, or modules directly.
- SIMD backend symbols remain behind loader-selected dispatch.
- Arena failures report arena identity and relevant graph/node/model context.
- Boot and fatal paths remain diagnosable.

Output findings first, ordered by severity, with file and line references.
