# Current Mission

Mission: C13.M12 Step 7 M12 completion audit
Campaign: C13 M12 Local Retrieval
Status: ready

## Objective

Execute M12 campaign Step 7 from `docs/campaigns/m12-local-retrieval.md`:
close M12 after retrieval contracts, document snapshot, deterministic ranking,
context packing, assistant retrieval surface, and smoke evidence are
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

- M12 status changes from `IN-PROGRESS` to `DONE` in `MILESTONE_CATALOG.md`.
- Catalog version banner is bumped.
- Catalog change log records M12 local retrieval completion.
- Campaign closeout records commit SHAs for Steps 1-6.

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

Campaign branch: `campaign/m12-local-retrieval`. Step 6 added
`make retrieval-smoke`, wired retrieval smoke into aggregate gates and mission
verification, and kept the gates green. Memory-unsafe Rust remains allowed by
project identity; M12 local retrieval stayed deterministic, bounded,
non-executing, and graph-mutation-free.
