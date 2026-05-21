---
name: parser-fuzz
description: Run the UMOD or UMDL parser fuzz harness against the seed corpus, or kick off a longer cargo-fuzz run. Use after touching parser code, after adding format fields, or to investigate a regression. Backed by spec §12.8 — confirms malformed inputs always produce structured errors, never panics or runaway allocations.
argument-hint: [umod|umdl|all] [--cargo-fuzz <seconds>]
allowed-tools: Read, Bash(cargo:*), Bash(rg:*), Bash(find:*)
---

# /parser-fuzz

Run parser fuzzing.

## Inputs

`$ARGUMENTS`:
- First token: `umod` | `umdl` | `all` (default: `all`).
- Optional `--cargo-fuzz <seconds>` to run a structured cargo-fuzz
  session for the requested target(s) for the given duration.
  Without this flag, the skill runs the seed corpus only (fast,
  deterministic, CI-grade).

## Procedure: seed corpus run (default)

1. Enumerate the corpus:
   ```bash
   find tests/fuzz_corpus -type f -name '*.bin' | wc -l
   ```

2. Run the harness:
   ```bash
   cargo test -p umod --test fuzz_corpus_runner
   cargo test -p umdl --test fuzz_corpus_runner
   ```

   Each runner enumerates files in its corpus and asserts:
   - parsing returns `Err`;
   - the variant matches the family declared in the sibling `.txt`
     (`unspecified` accepts any error variant);
   - parsing finishes within 50 ms per fixture;
   - allocation total stays under 1 MiB per fixture;
   - no `panic!` or `unwrap()` failure was hit.

3. Report.

## Procedure: cargo-fuzz session (longer)

```bash
cargo +nightly fuzz run umod_parse -- -max_total_time=$SECONDS
cargo +nightly fuzz run umdl_parse -- -max_total_time=$SECONDS
```

If a crash is found:

```bash
cp fuzz/artifacts/umod_parse/crash-* tests/fuzz_corpus/umod/regressions/
```

Add a sibling `.txt` describing the failure family (or `unspecified`
if not yet classified). Re-run the seed corpus to confirm the parser
now returns a typed error.

## Output

```
# Parser Fuzz — <scope>

## Seed corpus
- UMOD: 47 fixtures across 12 families
  - Pass: 47, Fail: 0
- UMDL: 31 fixtures across  9 families
  - Pass: 31, Fail: 0

## Coverage gaps
- umod/checksum_mismatch: 1 fixture (recommend ≥3)
- umdl/tokenizer_corruption: 0 fixtures (open gap)

## Verdict
- PASS / FAIL
```

If `--cargo-fuzz` was used:

```
## Cargo-fuzz session
- Target: umod_parse
- Duration: 300s
- Executions: 1.4M
- Coverage: 4128 branches
- New crashes: 1 → tests/fuzz_corpus/umod/regressions/crash-…bin
  - Family: unspecified (please classify)

## Action items
- Classify the new regression's failure family.
- Patch the parser site identified in the backtrace.
- Re-run the seed corpus.
```

## Hard rules

- A panic is a parser bug. Fix the parser, never widen the timeout.
- A passing parse on a malformed fixture is a parser bug. Fix the
  parser, never delete the fixture.
- The corpus only grows. Removing fixtures to make CI faster is a
  regression in coverage; parallelize the runner instead.
- A `.bin` fixture without a sibling `.txt` is incomplete; classify
  it or move it to `tests/fuzz_corpus/<parser>/unspecified/`.
