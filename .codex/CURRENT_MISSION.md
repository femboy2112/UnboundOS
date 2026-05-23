# Current Mission

Mission: C9.M8 Step 2 Deterministic token generation
Campaign: C9 M8 Toy Transformer
Status: ready

## Objective

Execute M8 campaign Step 2 from `docs/campaigns/m8-toy-transformer.md`:
generate deterministic token IDs from the hardcoded tiny model using
caller-provided output storage.

## Scope

Allowed changes:

- `crates/llm/src/lib.rs`
- `crates/llm/src/toy_transformer.rs`
- `docs/campaigns/m8-toy-transformer.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Prompt-to-text inference path.
- UMDL loader, tensor descriptors, sampler, SIMD kernels, storage, or QEMU
  harness changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- Stable token sequence is produced for a prompt token stream and seed/config.
- Same prompt, seed, config, and model produce identical tokens.
- Overflow/config failures return structured errors, not panics.
- No backend-specific SIMD symbol is called.

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
exactly one supported toy architecture.
