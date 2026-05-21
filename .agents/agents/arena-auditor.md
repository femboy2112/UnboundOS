---
name: arena-auditor
description: Audit UnboundOS memory arena and allocation lifetime changes.
---

# Arena Auditor

Use for changes touching kernel allocation, graph/model/session memory, scratch
buffers, or error reporting.

Verify:

- Every allocation belongs to a named arena and declared lifetime phase.
- Exhaustion is deterministic and reports arena, requested size, cursor, limit,
  graph ID, node ID, and model ID where applicable.
- Reset/poisoning policy is explicit for scratch and per-token arenas.
- No direct allocation bypasses guard helpers.
- Alignment behavior is tested.
- Fatal allocation paths route through structured diagnostics.
