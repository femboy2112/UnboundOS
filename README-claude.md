# UnboundOS — Codex Mission Setup

This repo uses a Codex-native control surface for mission-by-mission
implementation. The setup is built around the v2.1.1 fidelity-hardening spec
(`docs/UnboundOS_Tech_Spec_v2_1_1_Fidelity_Hardening.pdf`) and treats the
design contract as non-negotiable.

## Quick start

```bash
# From repo root:
cat CLAUDE.md
cat .codex/CURRENT_CAMPAIGN.md
cat .codex/CURRENT_MISSION.md
python3 scripts/status.py
python3 scripts/verify.py --mission current --dry-run
```

When the operator says `go`, use `.agents/skills/unboundos-go/SKILL.md`.
That workflow executes exactly one active mission, verifies it, commits,
pushes, and stops.

## File map

```
CLAUDE.md                                   # project constitution
README-claude.md                            # this compatibility guide
.codex/
├── CURRENT_CAMPAIGN.md                      # active campaign and stop rule
├── CURRENT_MISSION.md                       # current mission contract
├── PROJECT_PLAN.md                          # campaign roadmap
└── MISSION_LOG.md                           # completed mission ledger
.agents/
├── agents/
│   ├── fidelity-gate-reviewer.md
│   ├── graph-verifier-auditor.md
│   ├── arena-auditor.md
│   ├── simd-dispatch-auditor.md
│   ├── umod-format-engineer.md
│   ├── umdl-llm-engineer.md
│   ├── ssod-diagnostics-engineer.md
│   └── parser-fuzz-runner.md
└── skills/
    └── unboundos-go/SKILL.md
scripts/
├── status.py
├── verify.py
└── mission.py
```

## Workflow guide

### Every change

1. Read or recall the relevant CLAUDE.md section.
2. Plan with a subagent if the change is large; implement directly if
   small.
3. Run `python3 scripts/verify.py --mission current` before declaring done.
   Control-plane or documentation-only missions may explicitly use
   `--allow-missing-rust`; implementation missions must treat missing Rust
   tooling as a blocker.
4. Run `make qemu-headless` once the active mission reaches real boot image
   support and touches kernel, allocator, IDT, or framebuffer behavior.
5. Use `fidelity-gate-reviewer` before opening a PR or publishing a mission
   with implementation changes.

### Format / spec changes

Use `umod-format-engineer` or `umdl-llm-engineer` for any change to
`crates/umod` or `crates/umdl`. After the change:

1. Add a golden graph fixture for any new graph-level concept.
2. Add fuzz fixtures for new failure modes.
3. Run the parser fuzz mission checks once the corpus runner exists.
4. Run `python3 scripts/address_scan.py tests/golden_graphs tests/golden_models`.
5. Verify each new fixture through the normal graph loader.
6. Run `python3 scripts/verify.py --mission current`.

### LLM subsystem work

Use `umdl-llm-engineer`. After the change:

1. Use `arena-auditor` to confirm LLM arena phase boundaries hold.
2. Use `simd-dispatch-auditor` to confirm backend dispatch is honored.
3. Inspect the model package with the UMDL tooling once implemented.
4. Reference test sweep across all available SIMD tiers
   (`UNBOUNDOS_FORCE_SIMD=scalar|sse2|avx|avx2|avx512`).
5. Run `python3 scripts/verify.py --mission current`.

### Crash investigation

When QEMU smoke or real hardware reports a fault:

1. Decode the structured record with the SSOD tooling once implemented.
2. Identify the fault family and the file:line of the reported RIP.
3. If diagnostic identity is incomplete (missing arena/graph/node
   IDs), open an issue against the diagnostic emitter. Incomplete
   SSOD is itself a bug.
4. Fix the underlying cause; do not "improve" the SSOD by hiding the
   field.
5. Add a fault-injection test under `tests/fault_injection/`.

## Permission posture

There is no committed `.claude/settings.json` in this Codex-first surface.
Rely on the active Codex sandbox and the mission scope in
`.codex/CURRENT_MISSION.md`. Any future hook restoration should preserve the
same invariants: address scan after artifact changes, format checks after Rust
changes, and a hard block on direct `GraphRuntime { ... }` construction outside
`crates/graph/src/loader.rs`.

## Environment

```
RUST_BACKTRACE=1
CARGO_TERM_COLOR=always
UNBOUNDOS_PROFILE=qemu-dev
```

Override `UNBOUNDOS_PROFILE` when working against a different hardware
profile (`qemu-dev`, `legacy-bios`, `modern-x86_64`, `t500-class`).

## What this setup does **not** do

- It does not deploy or run on real hardware. The mission workflow does commit
  and push after completed missions because `.codex/CURRENT_CAMPAIGN.md`
  declares that publication policy.
- It does not bypass any Section-2 rule. There is no
  `--skip-fidelity` flag and no plan to add one.
- It does not write `.UMDL` model packages from scratch. Use the
  host-side conversion tools (out of scope for this repo's bare-metal
  runtime) to produce `.UMDL` packages, then validate them with repo-local UMDL
  tooling once that mission lands.

## Extending

- **New skill:** add `.agents/skills/<name>/SKILL.md` with frontmatter
  `name` and `description`.
- **New agent role:** add `.agents/agents/<name>.md` with frontmatter
  `name` and `description`.
- **New mission:** update `.codex/CURRENT_CAMPAIGN.md`,
  `.codex/CURRENT_MISSION.md`, and `.codex/PROJECT_PLAN.md`.

When extending, keep the same pattern: name, description tied to a `CLAUDE.md`
section, hard rules at the bottom, and explicit verification commands.
