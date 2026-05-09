# UnboundOS Parser Fuzz Corpus

Seed corpus for the UMOD and UMDL parsers. The `parser-fuzz-runner`
subagent and the `/parser-fuzz` skill drive this corpus. Spec §12.8
defines the policy.

## Layout

```
tests/fuzz_corpus/
├── umod/
│   ├── overlapping_sections/
│   ├── offsets_outside_bounds/
│   ├── undersized_headers/
│   ├── huge_counts_tiny_files/
│   ├── invalid_utf8_labels/
│   ├── unknown_resource_syntax/
│   ├── checksum_mismatch/
│   ├── pointer_shaped_values/
│   ├── unknown_node_type/
│   ├── type_mismatched_wires/
│   ├── unbroken_cycles/
│   ├── missing_constant_blobs/
│   ├── regressions/                 # crashes lifted from cargo-fuzz
│   └── unspecified/                 # uncategorized; classify when possible
└── umdl/
    ├── overlapping_sections/
    ├── offsets_outside_bounds/
    ├── undersized_headers/
    ├── huge_counts_tiny_files/
    ├── invalid_utf8_labels/
    ├── unsupported_quantization_id/
    ├── tensor_oob_weight_blob/
    ├── checksum_mismatch/
    ├── pointer_shaped_values/
    ├── regressions/
    └── unspecified/
```

## Fixture format

Each fixture is two files:

- `<name>.bin` — the malformed bytes.
- `<name>.txt` — one-line description: declared failure family, the
  expected `*LoadError` variant (or `unspecified`), and a short
  rationale.

Example `tests/fuzz_corpus/umod/unbroken_cycles/two-node-loop.txt`:

```
family:   unbroken_cycles
expected: GraphLoadError::UnbrokenCycle
note:     Two-node combinational loop A→B→A with no CYCLE_BREAK node.
          Reproduces spec §5.6 check 13.
```

## Adding a new fixture

The preferred path is the host-side UMOD/UMDL builder followed by a
deliberate corruption step. Hand-crafted bytes are accepted but must
include a precise `.txt` describing the corruption.

1. Build a valid file with the host builder.
2. Apply the targeted corruption (pad-byte flip, truncate, splice).
3. Confirm the parser returns `Err` with the declared variant.
4. Place under the matching family directory.
5. Add the `.txt` description.
6. Run `/parser-fuzz` to confirm the corpus runner picks it up.

## What you do not do

- Do not delete fixtures to make CI faster; parallelize the runner
  instead.
- Do not commit a `.bin` without a sibling `.txt`.
- Do not commit a fixture under a family it does not actually
  exercise.
