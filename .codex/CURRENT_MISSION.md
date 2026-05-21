# Current Mission

Mission: C1.M0 Step 3 IDT install and `UNBOUNDOS_IDT_OK`
Campaign: C1 M0 Boot Heartbeat
Status: ready

## Objective

Execute M0 campaign Step 3 from `docs/campaigns/m0-boot-heartbeat.md`: verify
or implement early IDT installation with fatal handler stubs, then emit
`UNBOUNDOS_IDT_OK` after `lidt` succeeds.

## Scope

Allowed changes:

- `kernel/src/main.rs`
- `kernel/src/boot.rs`
- `kernel/src/heartbeat.rs`
- `kernel/src/idt.rs`
- `kernel/src/ssod.rs`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Implementing Step 4 framebuffer fallback behavior, Step 6 full SSOD record
  formatting, Step 7 QEMU heartbeat assertions, or any M1+ bootloader/allocator
  behavior.
- Constructing or bypassing any `GraphRuntime`.
- Editing scripts, campaign archives, catalog rows, or implementation files
  outside the allowed list.
- Adding hidden execution or weakening boot diagnostics.

## Acceptance Criteria

- The early boot path installs an IDT before allocator or graph work.
- Fatal handler stubs for M0-scope exceptions route through the structured
  diagnostic surface rather than silently returning or rebooting.
- `kernel/src/boot.rs` emits `UNBOUNDOS_IDT_OK` immediately after IDT install
  succeeds.
- No M1+ behavior is implemented as part of this mission.

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

Campaign branch: `campaign/m0-boot-heartbeat`. `make qemu-headless` and
`make gates` remain non-authoritative until Step 7 owns the QEMU heartbeat
assertion path.
