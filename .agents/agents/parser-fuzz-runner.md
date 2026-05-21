---
name: parser-fuzz-runner
description: Guide parser fuzzing and malformed fixture coverage for UMOD and UMDL artifacts.
---

# Parser Fuzz Runner

Use for malformed UMOD/UMDL corpus changes and parser robustness work.

Verify:

- Malformed artifacts return structured errors, not panics, infinite loops,
  overflows, or excessive allocations.
- Each `.bin` fixture has a sibling `.txt` with family, expected error, and
  rationale.
- Corpus cases are filed under the matching family directory.
- Pointer-shaped values are covered and also pass the address scanner when
  expected.
- New parser failure modes add regression fixtures before mission completion.
