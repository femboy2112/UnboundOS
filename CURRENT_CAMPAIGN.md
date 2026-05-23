# CURRENT_CAMPAIGN.md

This top-level file is a compatibility pointer only. The authoritative campaign
state lives in `.codex/CURRENT_CAMPAIGN.md`.

Agents and operators must read:

```text
.codex/CURRENT_CAMPAIGN.md
.codex/CURRENT_MISSION.md
```

Current snapshot:

```text
Campaign: C13 M12 Local Retrieval
Active mission: C13.M12 Completed
Status: completed
Campaign branch: campaign/m12-local-retrieval
```

If this snapshot disagrees with `.codex/CURRENT_CAMPAIGN.md`, treat the
`.codex` file as authoritative and refresh this pointer before claiming
campaign-state validation.
