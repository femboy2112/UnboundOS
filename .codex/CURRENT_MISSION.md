# Current Mission

Mission: C8.M7 Step 4 Tokenizer smoke evidence and gates
Campaign: C8 M7 Tokenizer
Status: ready

## Objective

Execute M7 campaign Step 4 from `docs/campaigns/m7-tokenizer.md`: make
tokenizer evidence reproducible from checkout.

## Scope

Allowed changes:

- `Makefile`
- `scripts/**`
- `crates/llm/src/lib.rs`
- `crates/llm/src/tokenizer.rs`
- `docs/campaigns/m7-tokenizer.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- UMDL loader, tensor descriptors, model execution, sampler, storage, or QEMU
  harness changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- A smoke target or source-level check proves exactly one tokenizer family is
  supported and round-trip tests exist.
- Aggregate mission verification runs the tokenizer smoke.
- QEMU and graph gates remain green.

## Baseline to verify

```
branch: campaign/m7-tokenizer
status: IN-PROGRESS
```

## Verification Commands

```bash
python3 scripts/status.py
python3 scripts/mission.py validate
make fmt
make clippy
make tokenizer-smoke
make gates
cargo test -p llm
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m7-tokenizer`. Step 3 added decode, UTF-8
validation, invalid-token handling, and representative encode/decode round-trip
tests. Step 4 should add only smoke/check wiring.
