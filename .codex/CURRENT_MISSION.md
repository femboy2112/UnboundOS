# Current Mission

Mission: C1.M0 Step 4 Boot-diagnostic-buffer fallback
Campaign: C1 M0 Boot Heartbeat
Status: ready

## Objective

Execute M0 campaign Step 4 from `docs/campaigns/m0-boot-heartbeat.md`: verify
or implement the no-UART boot-diagnostic-buffer fallback for heartbeat lines.

## Scope

Allowed changes:

- `kernel/src/heartbeat.rs`
- `kernel/src/boot_diag.rs`
- `kernel/src/serial.rs`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Implementing framebuffer rendering, full SSOD formatting, Step 7 QEMU
  heartbeat assertions, or any M1+ bootloader/allocator behavior.
- Constructing or bypassing any `GraphRuntime`.
- Editing scripts, campaign archives, catalog rows, or implementation files
  outside the allowed list.
- Adding hidden execution or weakening boot diagnostics.

## Acceptance Criteria

- A failed COM1 probe records `BOOT_NO_SERIAL` into the boot diagnostic buffer.
- Every heartbeat emission records into `boot_diag` even when UART output is
  unavailable.
- Source exposes the `BOOT_HEARTBEAT_BUFFER_PRESENT` marker for the future M2
  framebuffer dump path.
- No framebuffer implementation or Step 7 QEMU assertion work is added in this
  mission.

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

Campaign branch: `campaign/m0-boot-heartbeat`. `make qemu-no-serial`,
`make qemu-headless`, and `make gates` remain non-authoritative until the later
QEMU assertion path lands.
