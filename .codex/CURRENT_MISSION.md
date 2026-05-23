# Current Mission

Mission: C9.M8 Step 5 M8 completion audit
Campaign: C9 M8 Toy Transformer
Status: ready

## Objective

Execute M8 campaign Step 5 from `docs/campaigns/m8-toy-transformer.md`:
close M8 after toy-model metadata, deterministic generation, prompt-to-text
inference, and smoke evidence are reproducibly verified.

## Scope

Allowed changes:

- `MILESTONE_CATALOG.md`
- `docs/campaigns/m8-toy-transformer.md`
- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Implementation changes outside M8 closeout metadata.
- UMDL loader, tensor descriptors, sampler, SIMD kernels, storage, or QEMU
  harness changes.
- Merging to or pushing `main`.

## Acceptance Criteria

- Row M8 `Status` changes from `IN-PROGRESS` to `DONE`.
- Catalog version banner is bumped.
- M8 change-log and campaign closeout record Step 1-4 checkpoint commits.

## Baseline to verify

```
branch: campaign/m8-toy-transformer
status: IN-PROGRESS
```

## Verification Commands

```bash
python3 scripts/status.py
python3 scripts/mission.py validate
make gates
make repo-state
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m8-toy-transformer`. M8 should not need new unsafe
code. Step 1 added fixed-width toy model/config metadata and validation for
exactly one supported toy architecture. Step 2 added deterministic token
generation. Step 3 added prompt-to-text inference through tokenizer
encode/decode and the toy generator with caller-provided buffers. Step 4 added
`make toy-transformer-smoke`, wired it into aggregate verification, and passed
`make gates`.
