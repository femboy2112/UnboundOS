# Current Mission

Mission: C13.M12 Completed
Campaign: C13 M12 Local Retrieval
Status: completed

## Objective

M12 is closed after retrieval contracts, document snapshot, deterministic
ranking, context packing, assistant retrieval surface, and smoke evidence were
reproducibly verified.

## Scope

Allowed changes:

- `MILESTONE_CATALOG.md`
- `docs/campaigns/m12-local-retrieval.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Runtime, retrieval, graph, storage, QEMU harness, thread/queue, eval, or
  execution-hook changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- M12 status is `DONE` in `MILESTONE_CATALOG.md`.
- Catalog version banner is `v0.26`.
- Catalog change log records M12 local retrieval completion.
- Campaign closeout records commit SHAs for Steps 1-6.
- No milestone remains in progress.

## Baseline to verify

```
branch: campaign/m12-local-retrieval
status: IN-PROGRESS
```

## Verification Commands

```bash
make gates
make repo-state
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m12-local-retrieval`. M12 added local retrieval
contracts, deterministic ranking, context packing, assistant retrieval routing,
and retrieval smoke evidence. Memory-unsafe Rust remains allowed by project
identity; M12 local retrieval stayed deterministic, bounded, non-executing,
and graph-mutation-free.
