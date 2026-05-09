# UnboundOS — Claude Code Setup

This directory contains the Claude Code configuration for the UnboundOS
repo: a project constitution, eight specialist subagents, eleven
slash-command skills, hooks, permissions, and a strict output style.
The setup is built around the v2.1.1 fidelity-hardening spec
(`docs/UnboundOS_Tech_Spec_v2_1_1_Fidelity_Hardening.pdf`) and treats
the design contract as non-negotiable.

## Quick start

```bash
# From repo root (after cloning):
cat CLAUDE.md                      # read the constitution end-to-end
ls .claude/agents                  # specialist subagents
ls .claude/skills                  # slash-command-style workflows
cat .claude/settings.json          # permissions, hooks, env

# In Claude Code:
/fidelity-check                    # full local gate sweep
/qemu-smoke                        # build + boot + heartbeat assert
/verify-graph tests/golden_graphs/single-pulse.umod
/audit-arenas
/simd-probe-review
```

Subagents are dispatched via the Task tool with
`subagent_type=<name>`, e.g. `fidelity-gate-reviewer`,
`graph-verifier-auditor`, `arena-auditor`, etc. Skills are invoked as
slash commands.

## File map

```
CLAUDE.md                                   # project constitution
README-claude.md                            # this file
.claude/
├── settings.json                            # permissions, hooks, env, output style
├── agents/
│   ├── fidelity-gate-reviewer.md
│   ├── graph-verifier-auditor.md
│   ├── arena-auditor.md
│   ├── simd-dispatch-auditor.md
│   ├── umod-format-engineer.md
│   ├── umdl-llm-engineer.md
│   ├── ssod-diagnostics-engineer.md
│   └── parser-fuzz-runner.md
├── skills/
│   ├── verify-graph/SKILL.md
│   ├── audit-arenas/SKILL.md
│   ├── fidelity-check/SKILL.md
│   ├── qemu-smoke/SKILL.md
│   ├── golden-graph-add/SKILL.md
│   ├── simd-probe-review/SKILL.md
│   ├── umdl-inspect/SKILL.md
│   ├── ssod-decode/SKILL.md
│   ├── boot-heartbeat-check/SKILL.md
│   ├── address-scan/SKILL.md
│   └── parser-fuzz/SKILL.md
└── output-styles/
    └── fidelity-strict.md
```

## Workflow guide

### Every change

1. Read or recall the relevant CLAUDE.md section.
2. Plan with a subagent if the change is large; implement directly if
   small.
3. Run `/fidelity-check` before declaring done.
4. Run `/qemu-smoke` if the change touches kernel/, allocator, IDT, or
   framebuffer.
5. Pass through the `fidelity-gate-reviewer` subagent before opening a
   PR.

### Format / spec changes

Use `umod-format-engineer` or `umdl-llm-engineer` for any change to
`crates/umod` or `crates/umdl`. After the change:

1. `/golden-graph-add <name>` for any new graph-level concept.
2. Add fuzz fixtures for new failure modes.
3. `/parser-fuzz` to confirm the parser handles the new failure modes
   structurally.
4. `/address-scan` to confirm no pointer-like values leaked into the
   new fixtures.
5. `/verify-graph <fixture>` for each new fixture.
6. `/fidelity-check`.

### LLM subsystem work

Use `umdl-llm-engineer`. After the change:

1. `/audit-arenas` — confirms LLM arena phase boundaries hold.
2. `/simd-probe-review` — confirms backend dispatch is honored.
3. `/umdl-inspect <model>` — confirms package well-formedness.
4. Reference test sweep across all available SIMD tiers
   (`UNBOUNDOS_FORCE_SIMD=scalar|sse2|avx|avx2|avx512`).
5. `/fidelity-check`.

### Crash investigation

When QEMU smoke or real hardware reports a fault:

1. `/ssod-decode <log>` — decode the structured record.
2. Identify the fault family and the file:line of the reported RIP.
3. If diagnostic identity is incomplete (missing arena/graph/node
   IDs), open an issue against the diagnostic emitter. Incomplete
   SSOD is itself a bug.
4. Fix the underlying cause; do not "improve" the SSOD by hiding the
   field.
5. Add a fault-injection test under `tests/fault_injection/`.

## Permission posture

The `settings.json` is set to **loose**: build/test/inspection
commands auto-allow; mutating ops on the host or outside-repo writes
require approval. Specifically:

- Auto-allow: `cargo`, `qemu-system-x86_64`, `make`, `git` read-only,
  shell inspection (`ls`, `find`, `rg`, `grep`, `cat`, `xxd`, etc.),
  `python3 scripts/`, `./scripts/`.
- Ask: `git push`, `git commit`, `git reset`, `git rebase`,
  `git checkout`, `git merge`, `cargo publish`, `rustup default`,
  `make install`, `sudo`.
- Deny: `rm -rf /`, `dd if=`, `mkfs`, `curl | sh`, writes to
  `/etc`, `/usr`, `/boot`, `~/.ssh`, `~/.gnupg`.

If you need a tighter posture, edit `.claude/settings.json` and move
items from `allow` to `ask`.

## Hooks

Two `PostToolUse` hooks fire on `Edit` and `Write`:

1. After edits to `crates/umod`, `crates/umdl`, or `tests/golden_*` /
   `tests/fuzz_corpus/`, the address scan runs. Any pointer-like value
   prints a warning.
2. After edits to `*.rs` anywhere in the repo, `cargo fmt --check`
   runs. Format drift prints a reminder.

A `PreToolUse` hook **blocks** edits that introduce direct
`GraphRuntime { ... }` construction outside `crates/graph/src/loader.rs`.
The single-verifier-gate invariant is mechanically enforced.

## Environment

```
RUST_BACKTRACE=1
CARGO_TERM_COLOR=always
UNBOUNDOS_PROFILE=qemu-dev
```

Override `UNBOUNDOS_PROFILE` when working against a different hardware
profile (`qemu-dev`, `legacy-bios`, `modern-x86_64`, `t500-class`).

## What this setup does **not** do

- It does not push to GitHub, deploy, or run on real hardware. All
  Bash commands run locally inside the Claude Code sandbox.
- It does not bypass any Section-2 rule. There is no
  `--skip-fidelity` flag and no plan to add one.
- It does not write `.UMDL` model packages from scratch. Use the
  host-side conversion tools (out of scope for this repo's bare-metal
  runtime) to produce `.UMDL` packages, then validate them with
  `/umdl-inspect` here.

## Extending

- **New skill:** `mkdir .claude/skills/<name>` and add `SKILL.md`
  with frontmatter `name`, `description`, `argument-hint`,
  `allowed-tools`. Skills become available as `/<name>`.
- **New subagent:** add `.claude/agents/<name>.md` with frontmatter
  `name`, `description`, `tools`. Dispatched via the Task tool.
- **New hook:** edit `.claude/settings.json` `hooks` block. Match
  patterns are `PreToolUse`, `PostToolUse`, plus the matcher regex.

When extending, keep the same pattern: name, description tied to a
CLAUDE.md section, hard rules at the bottom, output schema explicit.
The fidelity-strict style is the default; outputs that don't conform
will feel out of place.
