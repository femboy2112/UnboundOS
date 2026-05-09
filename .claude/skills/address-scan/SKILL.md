---
name: address-scan
description: Run the persistent-pointer leakage scanner (scripts/address_scan.py) over .MOD/.UMDL fixtures. Spec §6.10 and §14.1 require persistent files to contain no raw runtime addresses. Use before committing any new fixture, after any UMOD/UMDL emit code change, and as part of /fidelity-check. Reports flagged byte sequences resembling x86_64 kernel virtual addresses.
argument-hint: "[path]"
allowed-tools: Read, Bash
---

# /address-scan

Scan persistent artifacts for raw-pointer leakage.

## Procedure

1. Resolve `$ARGUMENTS`:
   - empty → scan `tests/golden_graphs tests/golden_models`
   - directory → scan recursively
   - file → scan that file

2. Run the scanner:
   ```bash
   python3 scripts/address_scan.py "$path"
   ```

3. The scanner flags 8-byte little-endian values matching canonical x86_64 patterns:
   - higher-half kernel: `0xFFFF8000_00000000` … `0xFFFFFFFF_FFFFFFFF`
   - higher-half direct map: `0xFFFF8800_00000000` …
   - userspace stack-like: `0x00007FFF_00000000` …
   - PIE userspace heap-like: `0x00005555_00000000` …

4. Output a report listing each finding with file, byte offset, hex value, and
   category. End with a verdict line:

   ```
   ## Verdict
   CLEAN | FLAGGED (<n> hits in <m> files)
   ```

5. If `FLAGGED` and the affected file is a release-track fixture (under
   `tests/golden_graphs/` or `tests/golden_models/`), this is a `BLOCK` for spec
   §6.10 / §14.1.

   If the affected file is a fuzz fixture (under
   `tests/fuzz_corpus/suspicious-pointers/`), this is `EXPECTED` — those fixtures
   are intentionally suspicious.

## Notes

- Some flags are false positives. A `u64` holding a tensor byte offset that
  coincidentally matches a kernel-VA pattern is not actually a pointer. Prefer
  reshaping the field over carrying the ambiguity forward.
- The scanner is advisory but its output is treated as authoritative until the
  operator overrides it explicitly.
