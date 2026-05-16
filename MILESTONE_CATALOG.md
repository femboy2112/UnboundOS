# UnboundOS Milestone Catalog

> **Catalog version:** v0.1
> **Spec rev:** `docs/UnboundOS_Tech_Spec_v2_1_1_Fidelity_Hardening.pdf`
> **Active milestone:** M0 (one at a time — see Rules below)

Every milestone owns exactly one campaign file under
`docs/campaigns/`. The top-level `CURRENT_CAMPAIGN.md` is a working
copy of the active milestone's campaign file. Completed campaigns are
archived under `docs/campaigns/` and never edited again.

## Rules

1. Only one milestone may be `IN-PROGRESS` at a time. `/go` refuses
   to advance if this rule is violated.
2. `DONE` requires the row's `Gate criteria` to be reproducible from a
   clean checkout via `make gates`.
3. `DEFERRED` requires a comment row below the table explaining the
   reason and the conditions under which the milestone unfreezes.
4. Status grammar: `TODO` | `IN-PROGRESS` | `DONE` | `DEFERRED`.
5. The `Spec §` column is authoritative — when the spec PDF revs, the
   `spec-refresher` agent walks every row and re-validates citations.

## Catalog

| ID  | Title | Spec § | Status | Gate criteria (operator-verifiable) | Owning campaign file |
|-----|-------|--------|--------|--------------------------------------|----------------------|
| M0  | Boot heartbeat | §3.2, §1.6, §3.9 | IN-PROGRESS | `make qemu-headless` serial log contains `UNBOUNDOS_BOOT_BEGIN`, `UNBOUNDOS_CPU_PROFILE`, `UNBOUNDOS_MEMMAP_OK`, `UNBOUNDOS_IDT_OK`, `UNBOUNDOS_BOOT_OK` in that order; `make gates` PROCEED | docs/campaigns/m0-boot-heartbeat.md |
| M1  | Limine handoff + GDT + memory map + frame allocator *(operator: fill from spec §13.1; code TODO M1 markers exist in kernel/src/boot.rs)* | §3.1, §3.6, §4.2, §4.3 | TODO | *(operator: fill — likely Limine info parsed, GDT installed, frame allocator returns a frame, `cargo test -p kernel-mem`)* | docs/campaigns/m1-limine-handoff.md *(not yet written)* |
| M2  | Framebuffer + boot-diagnostic-buffer fallback *(operator: fill from spec §13.2; code TODO M2 markers in kernel/src/heartbeat.rs, kernel/src/boot_diag.rs)* | §3.7, §3.9 | TODO | *(operator: fill — likely `make qemu` displays heartbeat on framebuffer; no-serial fallback dumps boot-diagnostic-buffer)* | docs/campaigns/m2-framebuffer.md *(not yet written)* |
| M3  | Arenas + graph_load + scheduler *(operator: fill from spec §13.3; code TODO M3 markers in kernel/src/boot.rs)* | §4.4–§4.11, §5.7, §5.9 | TODO | *(operator: fill — likely `cargo test -p graph` green for first golden graph; `/verify-graph` PASS; cooperative scheduler runs one tick)* | docs/campaigns/m3-arenas-and-scheduler.md *(not yet written)* |
| M4  | *(operator: fill from spec §13.4)* | §13.4 | TODO | *(tbd)* | *(tbd)* |
| M5  | *(operator: fill from spec §13.5)* | §13.5 | TODO | *(tbd)* | *(tbd)* |
| M6  | *(operator: fill from spec §13.6)* | §13.6 | TODO | *(tbd)* | *(tbd)* |
| M7  | *(operator: fill from spec §13.7)* | §13.7 | TODO | *(tbd)* | *(tbd)* |
| M8  | *(operator: fill from spec §13.8)* | §13.8 | TODO | *(tbd)* | *(tbd)* |
| M9  | *(operator: fill from spec §13.9)* | §13.9 | TODO | *(tbd)* | *(tbd)* |
| M10 | *(operator: fill from spec §13.10)* | §13.10 | TODO | *(tbd)* | *(tbd)* |
| M11 | *(operator: fill from spec §13.11)* | §13.11 | TODO | *(tbd)* | *(tbd)* |
| M12 | *(operator: fill from spec §13.12)* | §13.12 | TODO | *(tbd)* | *(tbd)* |

## Deferred reasons

*(none yet)*

## Change log

- **v0.1** — Initial catalog. M0 seeded with the boot heartbeat work
  already partly landed (commit `78f7ad5`). M1–M3 row outlines pulled
  from existing `// TODO M<N>` markers in `kernel/src/boot.rs`,
  `kernel/src/heartbeat.rs`, `kernel/src/boot_diag.rs`. M4–M12 are
  placeholders the operator fills from spec §13.
