---
name: unboundos-go
description: Execute exactly one UnboundOS mission from the repo-local Codex control files. Use when the operator says "go" in this repo or asks to advance the current UnboundOS mission.
---

# UnboundOS Go

Use this skill only from the UnboundOS repo root.

## Required Load Order

1. Read `CLAUDE.md`.
2. Read `.codex/CURRENT_CAMPAIGN.md`.
3. Read `.codex/CURRENT_MISSION.md`.
4. Read `.codex/PROJECT_PLAN.md`.
5. Run `python3 scripts/status.py`.

If any file is missing, stop and restore the control surface before touching
implementation files.

## Execution Rule

Execute exactly one active mission. Do not continue into the next mission in the
same turn. If the active mission is ambiguous, repair the mission file first
instead of guessing.

## Scope Rule

Only edit files listed in `.codex/CURRENT_MISSION.md` unless the mission itself
explicitly expands scope. Never stage unrelated user changes.

## Verification Rule

Before completion, run:

```bash
python3 scripts/mission.py validate
python3 scripts/verify.py --mission current
```

Run additional mission-specific commands listed in `.codex/CURRENT_MISSION.md`.
For control-plane or documentation-only missions, the mission may explicitly use
`python3 scripts/verify.py --mission current --allow-missing-rust`. For
implementation missions, missing `cargo` or `rustup` is a blocker and Rust
verification must not be claimed.

## Review Rule

Use the matching role file under `.agents/agents/` for touched subsystems:

- graph load/runtime: `graph-verifier-auditor.md`
- arenas/allocation: `arena-auditor.md`
- SIMD/tensor dispatch: `simd-dispatch-auditor.md`
- UMOD format: `umod-format-engineer.md`
- UMDL/LLM: `umdl-llm-engineer.md`
- IDT/SSOD/boot diagnostics: `ssod-diagnostics-engineer.md`
- fuzz fixtures: `parser-fuzz-runner.md`
- otherwise: `fidelity-gate-reviewer.md`

## Publish Rule

When the mission is complete:

1. Update `.codex/MISSION_LOG.md`.
2. Advance `.codex/CURRENT_CAMPAIGN.md` and `.codex/CURRENT_MISSION.md` to the
   next mission, or mark blocked if no safe next mission exists.
3. Run `git status --short`.
4. Stage only mission-owned files.
5. Commit with message `mission: <mission id> <short title>`.
6. Push the current branch.
7. Stop.

## Hard Stops

Stop immediately if a change would violate any `CLAUDE.md` hard rule, bypass
graph verification, add POSIX path leakage above storage adapters, introduce
hidden execution, let LLM output mutate state directly, or assume SIMD features
without CPUID/XCR0 gating.
