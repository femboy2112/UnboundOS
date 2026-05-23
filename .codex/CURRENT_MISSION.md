# Current Mission

Mission: C2.M1 Step 1 Forced-fault smoke harness
Campaign: C2 M1 Diagnostics Core
Status: ready

## Objective

Execute M1 campaign Step 1 from `docs/campaigns/m1-diagnostics-core.md`: add
an explicit QEMU-only forced-fault selection harness without changing normal
M0 heartbeat boot behavior.

## Scope

Allowed changes:

- `kernel/src/boot.rs`
- `kernel/src/idt.rs`
- `scripts/qemu.sh`
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

- Normal `make qemu-headless` still reaches `UNBOUNDOS_BOOT_OK`.
- Forced-fault mode can request `divide_error`, `invalid_opcode`, and
  `page_fault` after `UNBOUNDOS_IDT_OK`.
- Forced-fault assertions check `UNBOUNDOS_SSOD_BEGIN`, `reason=<fault>`,
  `rip=...`, and `UNBOUNDOS_SSOD_END`.
- The forced-fault path is explicit test plumbing and cannot trigger during
  normal boot.

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
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m1-diagnostics-core`. M1 proves Diagnostics Core
per spec §13.3. Limine handoff, memory-map ingestion, and allocator completion
remain later milestone work.
