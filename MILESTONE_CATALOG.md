# UnboundOS Milestone Catalog

> **Catalog version:** v0.4
> **Spec rev:** `docs/UnboundOS_Tech_Spec_v2_1_1_Fidelity_Hardening.pdf`
> **Active milestone:** none (M1 complete; next milestone opens by operator rotation)

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
| M0  | Boot heartbeat | §3.2, §1.6, §3.9 | DONE | `make qemu-headless` serial log contains `UNBOUNDOS_BOOT_BEGIN`, `UNBOUNDOS_CPU_PROFILE`, `UNBOUNDOS_MEMMAP_OK`, `UNBOUNDOS_IDT_OK`, `UNBOUNDOS_BOOT_OK` in that order; `make gates` PROCEED | docs/campaigns/m0-boot-heartbeat.md |
| M1  | Diagnostics core | §3.5, §9, §13.3 | DONE | IDT installed; divide-by-zero, page fault, and invalid opcode forced faults route through SSOD; serial SSOD output includes RIP and reason; `make gates` PROCEED | docs/campaigns/m1-diagnostics-core.md |
| M2  | Arena memory | §4.2–§4.11, §13.4 | TODO | BootArena, KernelArena, GraphArena, and ScratchArena exist; alignment tests pass; arena exhaustion is deterministic; memory map dump is available | docs/campaigns/m2-arena-memory.md *(not yet written)* |
| M3  | Embedded graph | §5.7, §5.9, §13.5 | TODO | Hardcoded graph source -> transform -> sink executes; epoch readiness works; fan-out test passes; active node diagnostics work | docs/campaigns/m3-embedded-graph.md *(not yet written)* |
| M4  | UMOD loader | §6, §13.6 | TODO | Persistent graph verifies and executes | docs/campaigns/m4-umod-loader.md *(not yet written)* |
| M5  | Minimal UI | §3.7, §8, §13.7 | TODO | Framebuffer IDE displays graph | docs/campaigns/m5-minimal-ui.md *(not yet written)* |
| M6  | Storage stage 1 | §7, §13.8 | TODO | Raw sector read works with timeout | docs/campaigns/m6-storage-stage-1.md *(not yet written)* |
| M7  | Tokenizer | §10.6, §13.9 | TODO | Bare-metal tokenizer runs | docs/campaigns/m7-tokenizer.md *(not yet written)* |
| M8  | Toy transformer | §10, §13.10 | TODO | Hardcoded tiny model generates text | docs/campaigns/m8-toy-transformer.md *(not yet written)* |
| M9  | UMDL loader | §10, §13.11 | TODO | Model package validates and loads | docs/campaigns/m9-umdl-loader.md *(not yet written)* |
| M10 | Quantized inference | §10, §11, §13.12 | TODO | Small quantized model streams tokens | docs/campaigns/m10-quantized-inference.md *(not yet written)* |
| M11 | IDE assistant | §10, §13.1 | TODO | Local assistant explains graph and SSOD | docs/campaigns/m11-ide-assistant.md *(not yet written)* |
| M12 | Local retrieval | §10, §13.1 | TODO | Assistant searches local docs | docs/campaigns/m12-local-retrieval.md *(not yet written)* |

## Deferred reasons

*(none yet)*

## Change log

- **v0.4** — M1 completed on `campaign/m1-diagnostics-core`: QEMU forced
  faults now prove #DE, #UD, and #PF route through SSOD, and the serial SSOD
  assertions verify reason, RIP, end marker, and page-fault error code while
  normal heartbeat boot remains green.
- **v0.3** — Opened M1 as the spec-defined Diagnostics Core milestone.
  Corrected the stale M1/M2/M3 catalog drift from the M0 planning notes and
  aligned the remaining milestone names with spec §13. M1 now owns forced
  exception coverage and SSOD serial evidence on branch
  `campaign/m1-diagnostics-core`.
- **v0.2** — M0 completed on `campaign/m0-boot-heartbeat`: source-level
  boot-order assertions, serial heartbeat, boot-diagnostic-buffer fallback,
  early IDT/SSOD routing, M0 SSOD records, and QEMU heartbeat smoke now pass
  `make gates`. The M0 smoke image uses a GRUB Multiboot2 bridge only for
  heartbeat verification; Limine handoff and real memory-map parsing remain M1.
- **v0.1** — Initial catalog. M0 seeded with the boot heartbeat work
  already partly landed (commit `78f7ad5`). M1–M3 row outlines pulled
  from existing `// TODO M<N>` markers in `kernel/src/boot.rs`,
  `kernel/src/heartbeat.rs`, `kernel/src/boot_diag.rs`. M4–M12 are
  placeholders the operator fills from spec §13.
