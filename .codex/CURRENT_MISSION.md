# Current Mission

Mission: C8.M7 Step 3 Raw-byte detokenizer round trip
Campaign: C8 M7 Tokenizer
Status: ready

## Objective

Execute M7 campaign Step 3 from `docs/campaigns/m7-tokenizer.md`: implement
token-to-UTF-8 decoding and round-trip tests.

## Scope

Allowed changes:

- `crates/llm/src/lib.rs`
- `crates/llm/src/tokenizer.rs`
- `docs/campaigns/m7-tokenizer.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Additional tokenizer families.
- UMDL loader, tensor descriptors, model execution, sampler, storage, or QEMU
  harness changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- Raw-byte token IDs decode into caller-provided byte output.
- Representative UTF-8 prompts round trip through encode then decode.
- Invalid token IDs and output overflow return structured errors.
- No hidden allocation or hidden execution path is introduced.

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
cargo test -p llm
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m7-tokenizer`. Step 2 added no-alloc byte-to-token
encoding over caller-provided buffers with structured overflow and metadata
errors. Step 3 adds decode and round-trip coverage.
