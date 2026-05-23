# Current Mission

Mission: C2.M1 Step 2 Divide-by-zero SSOD proof
Campaign: C2 M1 Diagnostics Core
Status: ready

## Objective

Execute M1 campaign Step 2 from `docs/campaigns/m1-diagnostics-core.md`: prove
the #DE path routes through SSOD and includes reason and RIP in serial output.

## Scope

Allowed changes:

- `kernel/src/idt.rs`
- `kernel/src/ssod.rs`
- `scripts/qemu.sh`
- `scripts/gates.sh`
- `Makefile`
- `docs/campaigns/m1-diagnostics-core.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Allocator, memory-map, framebuffer, graph, storage, or LLM behavior.
- Persistent artifact format changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- `make qemu-fault-de` passes.
- Serial output includes `UNBOUNDOS_SSOD_BEGIN`, `reason=divide_error`, a
  non-empty `rip=...`, and `UNBOUNDOS_SSOD_END`.
- Normal `make qemu-headless` still reaches `UNBOUNDOS_BOOT_OK`.

## Baseline to verify

```
branch: campaign/m1-diagnostics-core
status: IN-PROGRESS
```

## Verification Commands

```bash
python3 scripts/status.py
python3 scripts/mission.py validate
make fmt
make clippy
make kernel
make qemu-headless
make qemu-fault-de
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m1-diagnostics-core`. Step 1 already installed the
explicit forced-fault selector and QEMU SSOD assertion plumbing.
