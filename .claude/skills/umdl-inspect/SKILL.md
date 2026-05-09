---
name: umdl-inspect
description: Dump the header, tensor descriptor table, tokenizer metadata, and checksums of a .UMDL model package. Validates the spec §10.4–§10.7 layout without loading weights into runtime arenas. Use to inspect a new model package, debug a load rejection, or verify a host-side conversion.
argument-hint: <path-to-.umdl>
allowed-tools: Read, Bash, Grep
---

# /umdl-inspect

Dump and validate a `.UMDL` model package symbolically.

## Procedure

1. Resolve `$ARGUMENTS` to a `.UMDL` file path. Reject if missing or not regular.

2. Use the host-side inspection tool if available:
   ```bash
   cargo run -p unbound-tools --bin umdl-inspect -- "$path"
   ```
   If not yet implemented, fall back to direct binary inspection per the spec
   §10.5 header layout (CLAUDE.md mirrors it).

3. Print the header (all fields little-endian):

   ```
   # UMDL Inspect — <path>

   ## Header (96 bytes)
   magic                : "UMDL" (0x444d4055)
   format major         : <u16>
   format minor         : <u16>
   header length        : <u32>
   architecture ID      : <u32>
   quantization scheme  : <id> (<name from §10.6 registry, or UNKNOWN>)
   tensor count         : <u32>
   tokenizer offset     : 0x<hex>
   tensor desc offset   : 0x<hex>
   weight blob offset   : 0x<hex>
   checksum offset      : 0x<hex>
   required memory      : <bytes> (<MiB>)
   required scratch     : <bytes>
   KV-cache per token   : <bytes>
   max context tokens   : <u32>
   vocabulary size      : <u32>
   layer count          : <u32>
   hidden size          : <u32>
   attention head count : <u32>
   ```

4. Print the tokenizer metadata:
   ```
   ## Tokenizer
   type id              : <id> (<name from §10.7 registry>)
   vocab size           : <u32>
   token table          : offset 0x<hex> len <u32>
   merge table          : offset 0x<hex> len <u32> (or N/A)
   special tokens       : BOS=<id> EOS=<id> PAD=<id> UNK=<id>
   max token byte len   : <u32>
   utf-8 policy         : <enum>
   table checksum       : 0x<hex> — match | mismatch
   ```

5. Print the tensor descriptor table summary:
   ```
   ## Tensors (<n> total)
   id   | name                | scalar | quant     | rank | dims               | offset     | length     | align
   0001 | embed.token         | F32    | Q4_BLOCK32 |    2 | [vocab, hidden]    | 0x00001000 | 0x...      | 32
   0002 | block[0].attn.q     | F32    | Q4_BLOCK32 |    2 | [hidden, hidden]   | ...        | ...        | 32
   ...
   ```

   For each tensor, verify the `[byte_offset, byte_offset+byte_length)` range falls
   inside the weight blob section. Flag any tensor that escapes.

6. Print checksum verification:
   ```
   ## Checksums
   header              : 0x<hex> — match | mismatch
   tokenizer tables    : 0x<hex> — match | mismatch
   tensor descriptors  : 0x<hex> — match | mismatch
   weight blob         : 0x<hex> — match | mismatch
   ```

7. Run the address-scan on the `.UMDL`:
   ```bash
   python3 tools/address-scan/scan.py "$path"
   ```
   Persistent files MUST NOT contain raw runtime addresses (spec §10.4, §6.10).
   Flagged values are a fail.

8. Final verdict:
   ```
   ## Verdict
   READY | REJECT — <reason citing spec section>

   ## Notes
   - <e.g., "model declares requires_simd_avx2; not loadable on legacy-bios profile">
   - <e.g., "vocabulary size mismatch between header and tokenizer table">
   ```

## Rules

- Inspection is symbolic. Do not allocate `ModelWeightArena` or run any tensor
  primitive (spec §5.7 single verifier gate analog applies to model load too).
- If the magic is not `"UMDL"`, stop after the magic check.
- If the header length is implausible, stop after that check.
- Cite spec sections in the rejection reason.
