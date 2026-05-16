Read `CLAUDE.md`, then execute the repo-local current mission exactly as
if the operator said `go`.

Required sources of truth (read in this exact order):

1. `CURRENT_MISSION.md`
2. Every file named in `CURRENT_MISSION.md` `## Required reads`
3. `CURRENT_CAMPAIGN.md`
4. The active milestone row in `MILESTONE_CATALOG.md`

Delegate to the `current-mission` agent
(`Task` with `subagent_type=current-mission`). That agent owns the
parallel preflight burst, step selection, allowed-files enforcement,
validation, commit, push, loop, and stop logic.

Do as many sequential Steps as the gates allow in one invocation. Stop
cleanly at the first hard stop:

- Explicit `# Step N — Review gate` header.
- Any `make gates` sub-gate failure not declared as the step's
  purpose.
- `fidelity-gate-reviewer` BLOCK verdict.
- Stale baseline (`CURRENT_MISSION.md` disagrees with `make repo-state`).
- Out-of-scope edit required.
- Campaign complete.

Never bypass H1–H10 (CLAUDE.md §2). Never push to `main`. Never use
`--force`. The campaign branch is named in
`CURRENT_MISSION.md ## Branch / push / PR rule`.

Use `python3 -m ...` for any Python module commands. Prefer the
existing `Makefile` targets (`make gates`, `make fidelity`,
`make qemu-headless`, `make address-scan`) over improvised shell
pipelines (CLAUDE.md §5).
