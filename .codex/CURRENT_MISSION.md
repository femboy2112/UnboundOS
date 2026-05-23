# Current Mission

Mission: C8.M7 Step 2 Raw-byte tokenizer encode path
Campaign: C8 M7 Tokenizer
Status: ready

## Objective

Execute M7 campaign Step 2 from `docs/campaigns/m7-tokenizer.md`: implement
no-alloc UTF-8 byte-to-token encoding for caller-provided output storage.

## Scope

Allowed changes:

- `crates/llm/src/lib.rs`
- `crates/llm/src/tokenizer.rs`
- `docs/campaigns/m7-tokenizer.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Decode implementation.
- UMDL loader, tensor descriptors, model execution, sampler, storage, or QEMU
  harness changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- UTF-8 input bytes encode into stable raw-byte token IDs.
- Encoding writes only into caller-provided token buffers.
- Output overflow and invalid metadata return structured errors.
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

Campaign branch: `campaign/m7-tokenizer`. Step 1 added fixed-width tokenizer
metadata and support validation for exactly `RawByteToToken`. Step 2 should
only add the encode path; decode lands in Step 3.
