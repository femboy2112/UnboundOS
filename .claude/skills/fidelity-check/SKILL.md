---
name: fidelity-check
description: Run the spec §14.1 fidelity gate matrix and §14.3 review questions on the current branch. Use before any commit, merge, or release. Catches POSIX path leakage, hidden inference loops, verifier bypass, direct LLM mutation, arena leaks, SIMD assumption, persistent pointers, blind boot, eval creep, nondeterministic drift. Delegates to the fidelity-gate-reviewer subagent.
allowed-tools: Read, Glob, Grep, Bash, Task
---

# /fidelity-check

Full fidelity gate run.

## Procedure

1. Determine scope. Default: the diff of the current branch against `origin/main`.
   ```bash
   git fetch origin main 2>/dev/null || true
   git diff --name-only origin/main...HEAD || git diff --name-only HEAD~1
   ```
   If the operator passes a commit range, use it.

2. Invoke the `fidelity-gate-reviewer` subagent with that scope.

3. While that runs, run these complementary checks from the main thread:

   **POSIX path leakage** (spec §6.8, §7.2):
   ```bash
   rg -nP '"(?:/(?:etc|home|tmp|var|usr|opt|mnt|root)/|\./|\.\./|[A-Z]:\\\\|local://)' kernel/src
   ```
   Hits inside graph-visible code paths are FAIL. Storage adapter internals are PASS
   if they don't surface.

   **Hidden inference loop** (spec §1.8, §10.3):
   ```bash
   rg -nP 'thread::spawn|task::spawn|loop \{' kernel/src/llm
   ```
   Any non-orchestrator loop in the LLM subsystem is FAIL.

   **Verifier bypass** (spec §5.7):
   ```bash
   rg -nP 'GraphRuntime\s*\{|GraphRuntime::(new|from)' kernel/src
   ```
   Any match outside `kernel/src/graph/loader.rs` is FAIL.

   **Direct LLM mutation** (spec §10.18):
   ```bash
   rg -nP 'fn .*\(.*: .*Llm.*Output.*\).*-> Result<\s*Graph' kernel/src
   ```
   Any function that consumes an LLM output type and returns a Graph mutation is FAIL.

   **Persistent pointers** (spec §6.10, §14.1):
   ```bash
   python3 scripts/address_scan.py tests/golden_graphs tests/golden_models
   ```
   Any flagged byte sequence in a release-track fixture is FAIL.

   **Boot heartbeat** (spec §1.6):
   ```bash
   rg -n 'UNBOUNDOS_BOOT_BEGIN|UNBOUNDOS_BOOT_OK' kernel/src
   ```
   Both strings must appear in the boot path. Missing is FAIL.

   **AVX assumption** (spec §2.3, §11.2):
   ```bash
   rg -nP '_avx2\(|_avx512\(|_sse2\(' kernel/src | rg -v 'dispatch\.rs|tests/|fuzz/'
   ```
   Direct backend-symbol calls outside dispatch init are FAIL.

   **Eval creep** (spec §1.10):
   ```bash
   rg -nP '\b(eval|exec|run_code|load_generated)\s*\(' kernel/src
   ```
   Any LLM-output-to-execution path is FAIL.

   **Determinism explicit** (spec §5.11):
   ```bash
   rg -n 'rand::|RDRAND|read_tsc|rdtsc' kernel/src
   ```
   Any source of nondeterminism not gated behind a tagged source node is FAIL.

4. Combine the subagent report with the main-thread spot-check results into one
   verdict.

5. Output:

   ```
   # Fidelity Check — <branch> @ <commit>

   ## Subagent report
   <inline>

   ## Main-thread checks
   - POSIX path leakage: PASS|FAIL — <evidence>
   - Hidden inference loop: PASS|FAIL
   - Verifier bypass: PASS|FAIL
   - Direct LLM mutation: PASS|FAIL
   - Persistent pointers: PASS|FAIL
   - Boot heartbeat: PASS|FAIL
   - AVX assumption: PASS|FAIL
   - Eval creep: PASS|FAIL
   - Determinism explicit: PASS|FAIL

   ## §14.3 review questions
   1–8: <answers from subagent>

   ## Verdict
   READY | BLOCK | OPERATOR_DECISION
   ```

6. If `BLOCK`, list the precise fixes with file:line and the spec section that
   requires each.

This skill never commits, pushes, or modifies code. It reports.
