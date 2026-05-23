# Current Mission

Mission: C9.M8 Step 3 Prompt-to-text toy inference path
Campaign: C9 M8 Toy Transformer
Status: ready

## Objective

Execute M8 campaign Step 3 from `docs/campaigns/m8-toy-transformer.md`:
connect M7 tokenizer encode/decode with the M8 deterministic toy generator.

## Scope

Allowed changes:

- `crates/llm/src/tokenizer.rs`
- `crates/llm/src/lib.rs`
- `crates/llm/src/toy_transformer.rs`
- `docs/campaigns/m8-toy-transformer.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Smoke target or script wiring.
- UMDL loader, tensor descriptors, sampler, SIMD kernels, storage, or QEMU
  harness changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- UTF-8 prompts tokenize through `RawByteToToken`, generate new tokens, and
  decode to UTF-8 text using caller-provided buffers.
- Representative prompts produce deterministic text output.
- All state stays in explicit caller-provided buffers.

## Baseline to verify

```
branch: campaign/m8-toy-transformer
status: IN-PROGRESS
```

## Verification Commands

```bash
python3 scripts/status.py
python3 scripts/mission.py validate
make fmt
make clippy
cargo test -p llm
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m8-toy-transformer`. M8 should not need new unsafe
code. Step 1 added fixed-width toy model/config metadata and validation for
exactly one supported toy architecture. Step 2 added deterministic token
generation.
