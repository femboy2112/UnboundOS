# Current Mission

Mission: C6.M5 Step 1 Framebuffer text surface primitives
Campaign: C6 M5 Minimal UI
Status: ready

## Objective

Execute M5 campaign Step 1 from `docs/campaigns/m5-minimal-ui.md`: add a small
framebuffer text surface with deterministic glyph-cell writes that can be built
and tested without requiring bootloader framebuffer handoff.

## Scope

Allowed changes:

- `kernel/src/main.rs`
- `kernel/src/framebuffer.rs`
- `docs/campaigns/m5-minimal-ui.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Boot framebuffer initialization.
- Graph runtime or verifier changes.
- Storage, LLM, or SIMD changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- `kernel/src/framebuffer.rs` provides fixed-size text-cell rendering
  primitives over a caller-provided linear pixel buffer.
- The module is boot-passive: no global framebuffer assumptions and no writes
  before explicit initialization.
- Cell placement, newline, and bounds clipping are covered by tests or
  build-time assertions.

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
make kernel
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m5-minimal-ui`. M4 completed at `65d8ab3`; this
mission starts the framebuffer UI surface without bootloader handoff
assumptions.
