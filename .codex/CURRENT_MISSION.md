# Current Mission

Mission: C9.M8 Step 4 Toy transformer smoke evidence and gates
Campaign: C9 M8 Toy Transformer
Status: ready

## Objective

Execute M8 campaign Step 4 from `docs/campaigns/m8-toy-transformer.md`:
make toy-model deterministic output evidence reproducible from checkout.

## Scope

Allowed changes:

- `Makefile`
- `scripts/**`
- `crates/llm/src/**`
- `docs/campaigns/m8-toy-transformer.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- UMDL loader, tensor descriptors, sampler, SIMD kernels, storage, or QEMU
  harness changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- Smoke target/source check proves toy-model deterministic output and
  prompt-to-text tests exist.
- Aggregate mission verification runs toy transformer smoke.
- QEMU and graph gates remain green.

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
make toy-transformer-smoke
make gates
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m8-toy-transformer`. M8 should not need new unsafe
code. Step 1 added fixed-width toy model/config metadata and validation for
exactly one supported toy architecture. Step 2 added deterministic token
generation. Step 3 added prompt-to-text inference through tokenizer
encode/decode and the toy generator with caller-provided buffers.
