---
name: verify-graph
description: Run the spec §5.6 verifier checklist on a .MOD file and report each of the 22 checks as PASS/FAIL/N/A. Use when the operator wants to validate a graph artifact, or after any change to the verifier or the UMOD format. Never compiles a runtime graph; this is symbolic verification only.
argument-hint: <path-to-.mod-file>
allowed-tools: Read, Bash, Grep
---

# /verify-graph

Run the full spec §5.6 verifier checklist on `$ARGUMENTS`. The path may be a single
`.MOD` file or a directory; if a directory, run all `*.mod` recursively.

## Procedure

1. Resolve `$ARGUMENTS` to one or more file paths.
2. For each file, invoke the project's verifier in dry-run mode:
   ```bash
   cargo run -p unbound-tools --bin umod-verify -- --dry-run "$path"
   ```
   If `unbound-tools` is not yet implemented, fall back to inspecting the binary
   directly with `xxd`, `hexdump -C`, and the format spec (CLAUDE.md §UMOD layout)
   to walk the header, section table, and section descriptors by hand.
3. For each file, produce a per-check verdict from the spec §5.6 list:

   ```
   ## <relative path>

   01. Magic valid                 PASS|FAIL — <evidence>
   02. Version supported           PASS|FAIL — <evidence>
   03. Header length valid         PASS|FAIL — <evidence>
   04. Section table valid         PASS|FAIL — <evidence>
   05. Node count within limit     PASS|FAIL — <evidence>
   06. Wire count within limit     PASS|FAIL — <evidence>
   07. Every node index resolves   PASS|FAIL — <evidence>
   08. Every wire endpoint resolves PASS|FAIL — <evidence>
   09. Every pin index exists      PASS|FAIL — <evidence>
   10. Wire types match pins       PASS|FAIL — <evidence>
   11. Node type IDs registered    PASS|FAIL — <evidence>
   12. No undeclared capability    PASS|FAIL — <evidence>
   13. No unbroken cycle           PASS|FAIL — <evidence>
   14. Payload sizes bounded       PASS|FAIL — <evidence>
   15. Total memory fits GraphArena PASS|FAIL — <evidence>
   16. Model refs resolve/fail-grace PASS|FAIL — <evidence>
   17. Checksums match             PASS|FAIL — <evidence>
   18. UI layout refs nodes that exist PASS|FAIL — <evidence>
   19. Constant blob refs exist    PASS|FAIL — <evidence>
   20. Constant blobs match declared length/alignment PASS|FAIL — <evidence>
   21. Scheduling section if deterministic mode PASS|FAIL — <evidence>
   22. External refs use opaque-resource syntax PASS|FAIL — <evidence>
   ```

4. After per-file verdicts, also run the address-scan to catch persistent-pointer
   leakage (spec §6.10):
   ```bash
   python3 scripts/address_scan.py <path>
   ```
   Include its output as a separate section.

5. Final verdict per file: `READY` if all 22 PASS and address-scan clean,
   `REJECT` otherwise.

## Important

- This skill is symbolic verification only. It MUST NOT compile a runtime graph or
  call `graph_compile_verified`. The verifier gate (spec §5.7) is enforced even by
  diagnostics tooling.
- If a check is genuinely impossible to evaluate from the artifact alone (e.g.,
  graph-arena capacity depends on the active profile), mark it `N/A: <reason>`
  rather than guessing.
- Cite spec sections in evidence strings.

## Output

Markdown report. No interactive prompts. The operator reads the report and decides.
