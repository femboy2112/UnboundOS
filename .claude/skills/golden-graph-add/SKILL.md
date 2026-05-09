---
name: golden-graph-add
description: Add a new golden graph fixture under fixtures/golden/. Constructs a symbolic UMOD buffer, runs it through the verifier, and registers it in the golden-graph test suite. Never builds a runtime graph directly. Use when adding a new test case to the spec §12.7 minimum suite or extending coverage.
argument-hint: <name> [description]
allowed-tools: Read, Glob, Grep, Edit, Write, Bash
---

# /golden-graph-add

Add a new golden graph fixture. The minimum suite (spec §12.7):

- single pulse graph
- transform graph
- fan-out graph
- legal delay cycle graph
- invalid unbroken cycle graph
- graph with unknown node type
- graph with type mismatch
- LLM streaming toy graph
- LLM mutation proposal graph that stops at approval

## Procedure

1. Resolve `<name>` from `$ARGUMENTS`. Sanitize to `[a-z0-9_-]`. Path:
   ```
   fixtures/golden/<name>/
       graph.umod          # the binary fixture
       README.md           # what the fixture exercises
       expected.json       # expected verifier verdict + node firing trace
   ```

2. Ask the operator (via the main thread) for the graph topology if not obvious from
   `<name>`. Do not invent topology silently.

3. Build the symbolic UMOD buffer using the host-side builder:
   ```bash
   cargo run -p unbound-tools --bin umod-build -- \
       --spec fixtures/golden/<name>/spec.toml \
       --out fixtures/golden/<name>/graph.umod
   ```
   If the builder is not yet implemented, write a small Python or Rust script under
   `fixtures/golden/<name>/build.py` that emits the bytes per the format in
   CLAUDE.md (UMOD layout). Commit the script alongside the fixture.

4. Run the fixture through the verifier:
   ```
   /verify-graph fixtures/golden/<name>/graph.umod
   ```
   The expected verdict is one of:
   - `READY` for valid fixtures
   - `REJECT` with a specific error variant for invalid fixtures (the case is the
     point of the fixture)

   Record the expected verdict in `expected.json`.

5. Register the fixture in the test harness:
   - Append to `kernel/tests/golden_graphs.rs` (or wherever golden tests live).
   - The test loads the fixture, runs the verifier, and asserts the expected
     verdict matches.

6. Run the full golden-graph test suite:
   ```bash
   cargo test --test golden_graphs
   ```

7. Run the address-scan to confirm the new fixture contains no persistent pointers:
   ```bash
   python3 tools/address-scan/scan.py fixtures/golden/<name>/
   ```

8. Output:

   ```
   # Golden Graph Added — <name>

   ## Fixture
   - path: fixtures/golden/<name>/graph.umod
   - size: <bytes>
   - graph stable ID: 0x<hex>

   ## Topology
   <ASCII diagram or short description>

   ## Verifier verdict
   <READY | REJECT(<variant>)> — matches expected.json

   ## Address-scan
   clean | flagged: <details>

   ## Test registration
   - kernel/tests/golden_graphs.rs: line <n>
   ```

## Rules

- The fixture is symbolic UMOD bytes. Never construct a `GraphRuntime` directly to
  produce one (spec §5.7).
- Negative fixtures (graphs that should be rejected) are first-class. They prove
  the verifier's structured-error behavior.
- The README must cite the spec section being exercised.
- Every fixture goes through `address-scan` before it is committed.
