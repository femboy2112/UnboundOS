Run `make gates` and report each sub-gate's PASS/FAIL verdict.
Read-only — never commit, never push.

For each failing gate, pair it with:

- The Hard Rule it enforces (H1–H10 from `CLAUDE.md §2`).
- The spec § it cites.
- The diagnostic skill that drills into the failure:
  - cargo fmt / clippy → `/audit-arenas` is unrelated; just run
    `cargo fmt` / `cargo clippy --fix` proposals.
  - address-scan → `/address-scan` skill.
  - fidelity matrix → `/fidelity-check` skill +
    `fidelity-gate-reviewer` agent.
  - qemu-smoke → `/qemu-smoke` skill + `/boot-heartbeat-check`.
  - verifier failure → `/verify-graph` skill +
    `graph-verifier-auditor` agent.

Surface output verbatim. Do not propose edits. Do not run repair
commands. The operator decides next steps.
