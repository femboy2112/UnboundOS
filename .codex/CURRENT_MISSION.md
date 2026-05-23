# Current Mission

Mission: C1.M0 Step 7 QEMU smoke headless assertion
Campaign: C1 M0 Boot Heartbeat
Status: blocked

## Objective

Execute M0 campaign Step 7 from `docs/campaigns/m0-boot-heartbeat.md`: make
the headless QEMU smoke path assert the canonical boot heartbeat sequence.

## Scope

Allowed changes:

- `kernel/src/main.rs`
- `kernel/src/heartbeat.rs`
- `scripts/qemu.sh`
- `scripts/gates.sh`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- SSOD record changes beyond the Step 6 checkpoint.
- M1+ allocator, graph, storage, or LLM behavior.
- M0 completion audit/catalog closeout.
- Merging to or pushing `main`.

## Acceptance Criteria

- The boot path emits `UNBOUNDOS_BOOT_OK` after all M0 init.
- `scripts/qemu.sh --assert-heartbeat` verifies the canonical heartbeat order
  in the serial log, allowing key/value suffixes for CPU profile and memory map
  lines.
- The assertion path clears stale serial logs before boot and fails if the
  heartbeat is missing or out of order.
- `make qemu-headless` remains available without assertion mode.
- If real image generation is still unavailable, the mission records a blocker
  rather than claiming QEMU smoke success.

## Verification Commands

```bash
python3 scripts/status.py
python3 scripts/mission.py validate
make fmt
make clippy
make kernel
python3 scripts/verify.py --mission current
make qemu-headless
make gates
```

## Notes

Campaign branch: `campaign/m0-boot-heartbeat`. Stop reason: qemu-image-blocker.

`scripts/qemu.sh --assert-heartbeat` now implements ordered heartbeat checking,
but runtime QEMU smoke is blocked because `scripts/make_image.sh` still writes
the deliberate placeholder image instead of a bootable image. Step 8 must not
run until a real boot image path lets `make gates` pass.
