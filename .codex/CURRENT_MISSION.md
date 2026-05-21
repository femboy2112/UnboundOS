# Current Mission

Mission: C0.M2 Mission state handoff validation
Campaign: C0 Control Plane
Status: ready

## Objective

Validate that the Codex-native `go` workflow can read the newly installed
control surface, identify this mission as the active stopped boundary, and
prepare a safe handoff into C1.M0 without touching implementation code.

## Scope

Allowed changes:

- `.codex/CURRENT_MISSION.md`
- `.codex/PROJECT_PLAN.md`
- `.codex/MISSION_LOG.md`
- `.codex/CURRENT_CAMPAIGN.md`

Out of scope:

- Kernel, graph, UMOD, UMDL, or LLM implementation changes.
- Bootloader/image/QEMU behavior changes.
- Adding or redesigning skills, agents, or scripts unless the handoff files are
  invalid.

## Acceptance Criteria

- `python3 scripts/status.py` reports repo state and missing tool blockers.
- `python3 scripts/mission.py validate` validates the active mission files.
- `python3 scripts/verify.py --mission current --dry-run` prints the planned
  command set without mutating tracked files.
- The next implementation mission is C1.M0 Boot heartbeat and real QEMU smoke.
- No implementation files are changed.

## Verification Commands

```bash
python3 scripts/status.py
python3 scripts/mission.py validate
python3 scripts/verify.py --mission current --dry-run
python3 scripts/verify.py --mission current
```

## Notes

The current environment has `qemu-system-x86_64`, `pdftotext`, and the pinned
Rust toolchain installed through rustup under `/home/leah/.cargo/bin`.
