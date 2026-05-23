# CURRENT_MISSION.md

This top-level file is a compatibility pointer only. The authoritative mission
state lives in `.codex/CURRENT_MISSION.md`.

Agents and operators must read:

```text
.codex/CURRENT_MISSION.md
.codex/CURRENT_CAMPAIGN.md
```

Current snapshot:

```text
Mission: C13.M12 Completed
Campaign: C13 M12 Local Retrieval
Status: completed
Campaign branch: campaign/m12-local-retrieval
```

If this snapshot disagrees with `.codex/CURRENT_MISSION.md`, treat the
`.codex` file as authoritative and refresh this pointer before claiming
mission-state validation.
