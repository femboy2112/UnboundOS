# Current Mission

Mission: C1.M0 Step 2 Serial UART probe and heartbeat string emission
Campaign: C1 M0 Boot Heartbeat
Status: ready

## Objective

Execute M0 campaign Step 2 from `docs/campaigns/m0-boot-heartbeat.md`: confirm
the COM1 UART loopback probe and the first three heartbeat emissions are present
and ordered, patching only if validation shows drift.

## Scope

Allowed changes:

- `kernel/src/serial.rs`
- `kernel/src/heartbeat.rs`
- `kernel/src/boot.rs`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Implementing Step 3 IDT work, Step 4 framebuffer fallback behavior, Step 7
  QEMU heartbeat assertions, or any M1+ bootloader/allocator behavior.
- Constructing or bypassing any `GraphRuntime`.
- Editing scripts, campaign archives, catalog rows, or implementation files
  outside the allowed list.
- Adding hidden execution or weakening boot diagnostics.

## Acceptance Criteria

- `kernel/src/serial.rs` initializes COM1 and probes internal loopback before
  marking the UART available.
- `kernel/src/heartbeat.rs` records heartbeat lines into the boot diagnostic
  buffer even when UART output is unavailable.
- `kernel/src/boot.rs` emits `UNBOUNDOS_BOOT_BEGIN`,
  `UNBOUNDOS_CPU_PROFILE`, and `UNBOUNDOS_MEMMAP_OK` in that order.
- If validation is already green, make no implementation changes and advance
  only by the mission closeout rules on the next `go`.

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

Campaign branch: `campaign/m0-boot-heartbeat`. `make qemu-headless` may still
fail before Step 7 lands; Step 2 only owns the UART probe and first three
heartbeat emissions.
