# Current Mission

Mission: C1.M0 Step 5 Review gate
Campaign: C1 M0 Boot Heartbeat
Status: blocked

## Objective

Stop at the mandatory M0 campaign Step 5 review gate from
`docs/campaigns/m0-boot-heartbeat.md`. Do not implement Step 6 until the
operator explicitly approves continuing past this gate.

## Scope

Allowed changes:

- `.codex/CURRENT_MISSION.md`
- `.codex/CURRENT_CAMPAIGN.md`
- `.codex/MISSION_LOG.md`

Out of scope:

- Any implementation file edits.
- Step 6 panic/SSOD work.
- Step 7 QEMU heartbeat assertion work.
- M1+ bootloader, allocator, framebuffer, graph, storage, or LLM behavior.

## Acceptance Criteria

- Step 4 is recorded as completed in `.codex/MISSION_LOG.md`.
- `.codex/CURRENT_CAMPAIGN.md` names Step 5 as the active review gate.
- This mission file states the review-gate stop explicitly.
- No implementation files are edited for Step 5.

## Verification Commands

```bash
python3 scripts/status.py
python3 scripts/mission.py validate
python3 scripts/verify.py --mission current
```

## Notes

Campaign branch: `campaign/m0-boot-heartbeat`. Stop reason: review-gate.

The operator review items are listed under Step 5 in
`docs/campaigns/m0-boot-heartbeat.md`. Step 6+ remains blocked until the
operator explicitly approves continuing past this gate.
