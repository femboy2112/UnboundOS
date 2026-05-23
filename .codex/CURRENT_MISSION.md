# Current Mission

Mission: C6.M5 Step 4 UI smoke evidence and gates
Campaign: C6 M5 Minimal UI
Status: ready

## Objective

Execute M5 campaign Step 4 from `docs/campaigns/m5-minimal-ui.md`: make the
minimal UI evidence reproducible from checkout.

## Scope

Allowed changes:

- `Makefile`
- `scripts/**`
- `kernel/src/**`
- `docs/campaigns/m5-minimal-ui.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Graph runtime or verifier changes.
- Storage, LLM, or SIMD changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- A smoke target or source-level check proves framebuffer text and graph-state
  rendering are reachable.
- QEMU headless gates remain green.
- No graphical-only CI requirement is added.

## Baseline to verify

```
branch: campaign/m5-minimal-ui
status: IN-PROGRESS
```

## Verification Commands

```bash
python3 scripts/status.py
python3 scripts/mission.py validate
make fmt
make clippy
make gates
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m5-minimal-ui`. Step 3 exposed a copied,
read-only graph display snapshot from compiled graph handles and added
framebuffer text rendering for graph id, node count, wire count, active-node,
and last-completed-node facts.
