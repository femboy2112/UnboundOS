# Current Mission

Mission: C10.M9 Step 6 M9 completion audit
Campaign: C10 M9 UMDL Loader
Status: ready

## Objective

Execute M9 campaign Step 6 from `docs/campaigns/m9-umdl-loader.md`: close M9
after UMDL parsing, validation, load-view, arena reservation, and smoke
evidence are reproducibly verified.

## Scope

Allowed changes:

- `MILESTONE_CATALOG.md`
- `docs/campaigns/m9-umdl-loader.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/MISSION_LOG.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Implementation changes outside M9 closeout metadata.
- Sampler, tensor kernel, graph mutation, storage, or QEMU harness changes.
- LLM sampler, SIMD kernels, storage, QEMU harness, or graph mutation changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- Row M9 `Status` changes from `IN-PROGRESS` to `DONE`.
- Catalog version banner is bumped.
- M9 change-log and campaign closeout record Step 1-5 checkpoint commits.

## Baseline to verify

```
branch: campaign/m9-umdl-loader
status: IN-PROGRESS
```

## Verification Commands

```bash
python3 scripts/status.py
python3 scripts/mission.py validate
make gates
make repo-state
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m9-umdl-loader`. Memory-unsafe Rust remains allowed
by project identity, but UMDL persistent-format parsing should be safe,
fixed-width, deterministic, and free of host paths or raw pointers. Step 1
added little-endian header parsing and malformed-header tests. Step 2 added
overflow-safe section bounds, non-overlap checks, and deterministic checksum
validation. Step 3 added tokenizer metadata and tensor descriptor parsing and
validation. Step 4 added a read-only loaded model view, explicit arena
reservation accounting, and SIMD/profile budget validation. Step 5 added
`make umdl-smoke`, a deterministic fixture generator, malformed corpus entry,
and aggregate verification wiring.
