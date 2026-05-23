# Current Mission

Mission: C5.M4 Step 6 Golden and malformed fixture coverage
Campaign: C5 M4 UMOD Loader
Status: ready

## Objective

Execute M4 campaign Step 6 from `docs/campaigns/m4-umod-loader.md`: make M4
golden and malformed UMOD fixture coverage non-vacuous and reproducible from
checkout.

## Scope

Allowed changes:

- `tests/golden_graphs/**`
- `tests/fuzz_corpus/umod/**`
- `crates/umod/src/lib.rs`
- `crates/graph/src/verifier.rs`
- `scripts/verify.py`
- `docs/campaigns/m4-umod-loader.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Public runtime graph construction changes.
- UI, storage, LLM, or boot changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- At least one valid golden UMOD fixture is registered.
- Malformed UMOD cases cover bad magic/version, truncated header,
  out-of-bounds sections, overlap, huge counts, invalid refs, and unbroken
  cycles.
- The verification bundle exercises the fixture set.
- Golden and malformed fixture coverage is reproducible from checkout.

## Baseline to verify

```
branch: campaign/m4-umod-loader
status: IN-PROGRESS
```

## Verification Commands

```bash
python3 scripts/status.py
python3 scripts/mission.py validate
make fmt
make clippy
cargo test -p umod
cargo test -p graph
python3 scripts/address_scan.py tests/golden_graphs tests/golden_models
python3 scripts/verify.py --mission current
make gates
```

## Notes

Campaign branch: `campaign/m4-umod-loader`. Step 5 added persistent
source -> transform -> sink UMOD bytes and proved they verify through
`graph_load_from_umod` and compile through `graph_compile_verified` without a
public runtime constructor or verifier bypass.
