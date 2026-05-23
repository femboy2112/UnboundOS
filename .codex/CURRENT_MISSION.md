# Current Mission

Mission: C6.M5 Step 3 Minimal graph-state display model
Campaign: C6 M5 Minimal UI
Status: ready

## Objective

Execute M5 campaign Step 3 from `docs/campaigns/m5-minimal-ui.md`: provide a
read-only UI model that can display verified graph state without constructing,
mutating, or bypassing graph runtime handles.

## Scope

Allowed changes:

- `crates/graph/src/lib.rs`
- `crates/graph/src/loader.rs`
- `kernel/src/framebuffer.rs`
- `docs/campaigns/m5-minimal-ui.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Public runtime graph construction changes.
- Storage, LLM, or SIMD changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- Only read-only, symbolic graph display facts needed by the minimal IDE
  surface are exposed.
- Framebuffer rendering can display graph id, node count, wire count, and last
  active/completed-node diagnostics where available.
- Private runtime construction remains inside `loader.rs`.

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
cargo test -p graph
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m5-minimal-ui`. Step 2 wired the framebuffer
fallback call path, preserved normal serial heartbeat boot, and made
`make qemu-no-serial` prove boot completion without depending on serial output.
