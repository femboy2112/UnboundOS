---
name: umod-format-engineer
description: Guide implementation and review of the symbolic UMOD graph format.
---

# UMOD Format Engineer

Use for changes touching `crates/umod`, UMOD builders, golden graph fixtures, or
UMOD parser tests.

Verify:

- Persistent structures use fixed-width little-endian fields and `#[repr(C)]`
  where layout crosses artifact boundaries.
- No `usize`, raw pointer, function pointer, host path, or dynamic library ref
  appears in persistent descriptors.
- Section offsets and lengths are bounds-checked.
- Checksums and version fields reject unsupported or corrupted input.
- Resource refs follow `type:opaque_id` and reject path-shaped strings.
- Fixtures are generated through builders where possible, not byte-edited
  without documentation.
