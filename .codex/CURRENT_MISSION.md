# Current Mission

Mission: C3.M2 Step 2 Named M2 arena set
Campaign: C3 M2 Arena Memory
Status: ready

## Objective

Execute M2 campaign Step 2 from `docs/campaigns/m2-arena-memory.md`:
materialize BootArena, KernelArena, GraphArena, and ScratchArena as named
bounded arenas with declared lifetime/phase comments.

## Scope

Allowed changes:

- `kernel/src/arena.rs`
- `kernel/src/boot.rs`
- `docs/campaigns/m2-arena-memory.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Graph runtime construction, storage, UI, or LLM behavior.
- Persistent artifact format changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- Boot, Kernel, Graph, and Scratch arena descriptors exist.
- Direct allocation remains behind named arena APIs or guard helpers.
- Arena lifetime/phase rules are documented in code.
- Normal `make qemu-headless` still reaches `UNBOUNDOS_BOOT_OK`.

## Baseline to verify

```
branch: campaign/m2-arena-memory
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

Campaign branch: `campaign/m2-arena-memory`. Step 1 established the bounded
arena cursor contract and host-tested alignment, overflow, exhaustion, and
reset behavior.
