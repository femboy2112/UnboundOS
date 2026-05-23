# Current Mission

Mission: C8.M7 Step 1 Tokenizer registry and metadata contract
Campaign: C8 M7 Tokenizer
Status: ready

## Objective

Execute M7 campaign Step 1 from `docs/campaigns/m7-tokenizer.md`: define the
supported tokenizer family and fixed-width metadata contract.

## Scope

Allowed changes:

- `crates/umdl/src/lib.rs`
- `crates/llm/src/lib.rs`
- `crates/llm/src/tokenizer.rs`
- `docs/campaigns/m7-tokenizer.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Encode/decode implementation beyond metadata validation.
- UMDL loader, tensor descriptors, model execution, sampler, storage, or QEMU
  harness changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- Tokenizer metadata covers tokenizer type, vocabulary size, special token IDs,
  UTF-8 policy, maximum token byte length, and checksum.
- M7 supports exactly `RawByteToToken`; BPE and SentencePiece return structured
  unsupported-family errors.
- Metadata structs are fixed-width and pointer-free.

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
cargo test -p umdl
cargo test -p llm
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m7-tokenizer`. M7 should not need new unsafe code;
if a later tokenizer table loader does, it must follow the project rule:
bounded, inspectable, deterministic, and not undefined by design.
