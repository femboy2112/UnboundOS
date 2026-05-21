# Current Mission

Mission: C1.M0 Step 1 Boot-order assertion vs spec §3.2
Campaign: C1 M0 Boot Heartbeat
Status: ready

## Objective

Execute M0 campaign Step 1 from `docs/campaigns/m0-boot-heartbeat.md`: add or
verify source-level assertions that the kernel entry path follows the spec
§3.2 early boot order without implementing later-milestone behavior.

## Scope

Allowed changes:

- `kernel/src/main.rs`
- `kernel/src/boot.rs`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Implementing M1+ bootloader, GDT, frame allocator, framebuffer, graph, UMOD,
  UMDL, LLM, or storage behavior.
- Constructing or bypassing any `GraphRuntime`.
- Editing scripts, campaign archives, catalog rows, or implementation files
  outside the allowed list.
- Adding hidden execution or weakening boot diagnostics.

## Acceptance Criteria

- `_start` or its outermost dispatcher has numbered comments for the spec §3.2
  kernel-entry order where that order is represented in source.
- Any M1+ or later boot steps that remain deferred keep explicit TODO or
  `unimplemented!()` markers with milestone/spec citations.
- No M1+ behavior is implemented as part of this mission.
- The `/boot-heartbeat-check` procedure in
  `.claude/skills/boot-heartbeat-check/SKILL.md` has no Step 1 ordering
  findings that require changes outside this mission.
- No implementation files outside `kernel/src/main.rs` and `kernel/src/boot.rs`
  are changed.

## Verification Commands

```bash
python3 scripts/status.py
python3 scripts/mission.py validate
make fmt
make clippy
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m0-boot-heartbeat`. If execution begins on another
branch, switch to or create that branch before editing implementation files.
The current environment has `qemu-system-x86_64`, `pdftotext`, and the pinned
Rust toolchain installed through rustup under `/home/leah/.cargo/bin`.
