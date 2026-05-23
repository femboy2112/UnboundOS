# Current Mission

Mission: C6.M5 Step 2 Boot diagnostic framebuffer fallback
Campaign: C6 M5 Minimal UI
Status: ready

## Objective

Execute M5 campaign Step 2 from `docs/campaigns/m5-minimal-ui.md`: wire the
heartbeat fallback hook so a framebuffer surface can display `BOOT_NO_SERIAL`,
`BOOT_HEARTBEAT_BUFFER_PRESENT`, and the recorded boot diagnostic buffer when
UART is unavailable.

## Scope

Allowed changes:

- `kernel/src/boot.rs`
- `kernel/src/heartbeat.rs`
- `kernel/src/boot_diag.rs`
- `kernel/src/framebuffer.rs`
- `docs/campaigns/m5-minimal-ui.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Graph runtime or verifier changes.
- Storage, LLM, or SIMD changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- The TODO-only framebuffer fallback is replaced with a real call path through
  framebuffer text output.
- Serial heartbeat order and normal boot behavior remain unchanged.
- Headless boot does not require a framebuffer.

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
make qemu-headless
make qemu-no-serial
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m5-minimal-ui`. Step 1 added boot-passive
framebuffer text primitives over caller-provided pixel memory. Unsafe memory
access remains allowed for real hardware/MMIO boundaries, but must be bounded,
inspectable, deterministic, and not undefined by design.
