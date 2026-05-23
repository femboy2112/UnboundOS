# Current Mission

Mission: C9.M8 Step 1 Toy model architecture contract
Campaign: C9 M8 Toy Transformer
Status: ready

## Objective

Execute M8 campaign Step 1 from `docs/campaigns/m8-toy-transformer.md`:
define the hardcoded toy model metadata, deterministic generation config, and
caller-provided buffer contracts.

## Scope

Allowed changes:

- `crates/llm/src/lib.rs`
- `crates/llm/src/toy_transformer.rs`
- `docs/campaigns/m8-toy-transformer.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Token generation implementation beyond metadata/config validation.
- UMDL loader, tensor descriptors, sampler, SIMD kernels, storage, or QEMU
  harness changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- Toy model module defines fixed-width model/config metadata.
- M8 exposes exactly one supported toy architecture.
- Structured errors exist for output buffer overflow and unsupported config.
- No hidden allocation or hidden execution path is introduced.

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
code. If a later model loader or SIMD kernel requires unsafe access, it must be
bounded, inspectable, deterministic, and not undefined by design.
