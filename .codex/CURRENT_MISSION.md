# Current Mission

Mission: C13.M12 Step 6 Retrieval smoke evidence and gates
Campaign: C13 M12 Local Retrieval
Status: ready

## Objective

Execute M12 campaign Step 6 from `docs/campaigns/m12-local-retrieval.md`:
make local retrieval evidence reproducible from checkout.

## Scope

Allowed changes:

- `crates/llm/src/assistant.rs`
- `crates/llm/src/retrieval.rs`
- `crates/llm/src/lib.rs`
- `Makefile`
- `scripts/**`
- `docs/campaigns/m12-local-retrieval.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Filesystem access beyond source-level smoke inspection, host paths above
  storage adapters, graph mutation, storage behavior changes, QEMU harness
  changes, thread/queue, eval, or execution-hook changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- `make retrieval-smoke` is available.
- Smoke evidence proves retrieval contracts, ranking, context packing,
  assistant retrieval routing, and no host-path/no-direct-mutation boundaries
  are source-reachable.
- Retrieval smoke is wired into aggregate verification.
- Aggregate gates are green.

## Baseline to verify

```
branch: campaign/m12-local-retrieval
status: IN-PROGRESS
```

## Verification Commands

```bash
make fmt
make clippy
make retrieval-smoke
make gates
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m12-local-retrieval`. Step 5 added
`AssistantRetrievalRequest`, `AssistantRetrievalResponse`, and
`assistant_retrieve_context`, keeping retrieval output as bounded explanatory
context while routing optional actions through `StructuredActionBuffer`.
Memory-unsafe Rust remains allowed by project identity, but M12 retrieval
contracts should be deterministic, bounded, and non-executing.
