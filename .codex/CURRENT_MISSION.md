# Current Mission

Mission: C7.M6 Step 4 Resource namespace guard evidence
Campaign: C7 M6 Storage Stage 1
Status: ready

## Objective

Execute M6 campaign Step 4 from `docs/campaigns/m6-storage-stage-1.md`: prove
storage bring-up did not leak POSIX/FAT32 paths into graph-visible resource
state.

## Scope

Allowed changes:

- `scripts/**`
- `crates/umod/src/lib.rs`
- `crates/graph/src/verifier.rs`
- `tests/fuzz_corpus/umod/**`
- `docs/campaigns/m6-storage-stage-1.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Storage driver, boot path, QEMU harness, FAT32, append-only graph store, or
  write support.
- Merging to or pushing `main`.

## Acceptance Criteria

- Path-like storage references are rejected above the storage adapter boundary.
- Accepted examples remain opaque `type:id` resource references.
- Aggregate mission verification exercises the guard evidence.
- No graph runtime construction or verifier bypass is introduced.

## Baseline to verify

```
branch: campaign/m6-storage-stage-1
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
python3 scripts/verify.py --mission current
make gates
```

## Notes

Campaign branch: `campaign/m6-storage-stage-1`. Step 3 added the deterministic
raw-sector fixture generator, QEMU storage smoke target, and serial assertion
for `UNBOUNDOS_STORAGE_MARKER_OK`. Step 4 should stay out of the hardware path
and strengthen only the graph-visible resource namespace guard evidence.
