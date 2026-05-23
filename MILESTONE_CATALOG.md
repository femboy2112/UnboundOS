# UnboundOS Milestone Catalog

> **Catalog version:** v0.25
> **Spec rev:** `docs/UnboundOS_Tech_Spec_v2_1_1_Fidelity_Hardening.pdf`
> **Active milestone:** M12 Local retrieval

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
| M2  | Arena memory | §4.2–§4.11, §13.4 | DONE | BootArena, KernelArena, GraphArena, and ScratchArena exist; alignment tests pass; arena exhaustion is deterministic; memory map dump is available; `make gates` PROCEED | docs/campaigns/m2-arena-memory.md |
| M3  | Embedded graph | §5.7, §5.9, §13.5 | DONE | Hardcoded graph source -> transform -> sink executes through the verified graph path; epoch readiness works; fan-out test passes; active node diagnostics work; `make gates` PROCEED | docs/campaigns/m3-embedded-graph.md |
| M4  | UMOD loader | §6, §13.6 | DONE | Persistent graph verifies and executes through `graph_load_from_umod -> graph_compile_verified`; malformed UMODs return structured errors; `make gates` PROCEED | docs/campaigns/m4-umod-loader.md |
| M5  | Minimal UI | §3.7, §8, §13.7 | DONE | Framebuffer text primitives render boot diagnostics and graph state; `make gates` PROCEED | docs/campaigns/m5-minimal-ui.md |
| M6  | Storage stage 1 | §7, §13.8 | DONE | Raw sector read works with timeout; graph-visible storage refs remain opaque; `make gates` PROCEED | docs/campaigns/m6-storage-stage-1.md |
| M7  | Tokenizer | §10.7, §13.7 | DONE | Bare-metal tokenizer round trip works for the initially supported tokenizer family; `make gates` PROCEED | docs/campaigns/m7-tokenizer.md |
| M8  | Toy transformer | §10.8, §13.7 | DONE | Hardcoded tiny model generates deterministic token output; `make gates` PROCEED | docs/campaigns/m8-toy-transformer.md |
| M9  | UMDL loader | §10, §13.11 | DONE | Model package validates and loads; `make gates` PROCEED | docs/campaigns/m9-umdl-loader.md |
| M10 | Quantized inference | §10, §11, §13.12 | DONE | Small quantized model streams tokens; `make gates` PROCEED | docs/campaigns/m10-quantized-inference.md |
| M11 | IDE assistant | §10, §13.1 | DONE | Local assistant explains graph and SSOD; `make gates` PROCEED | docs/campaigns/m11-ide-assistant.md |
| M12 | Local retrieval | §10, §13.1 | IN-PROGRESS | Assistant searches local docs; `make gates` PROCEED | docs/campaigns/m12-local-retrieval.md |

## Deferred reasons

*(none yet)*

## Change log

- **v0.25** — Opened M12 Local retrieval on
  `campaign/m12-local-retrieval`. The campaign owns fixed-width local document
  retrieval inputs, deterministic ranking, context packing, assistant retrieval
  surface integration, and smoke evidence without exposing host paths or
  granting assistant mutation authority.
- **v0.24** — M11 completed on `campaign/m11-ide-assistant`: fixed-width
  assistant action proposals, caller-owned action buffers, read-only graph and
  SSOD explanation snapshots, deterministic caller-buffer explanation text, a
  unified `assistant_explain` request surface, and `make assistant-smoke` now
  prove the local assistant explains graph/SSOD state without direct mutation
  authority. M11 added no unsafe blocks or functions; memory-unsafe Rust remains
  allowed by project identity at bounded OS/model-kernel boundaries.
- **v0.23** — Opened M11 IDE assistant on `campaign/m11-ide-assistant`.
  The campaign owns graph/SSOD explanation surfaces and a structured action
  buffer that keeps assistant output as data until schema validation, graph
  verification, and operator approval.
- **v0.22** — M10 completed on `campaign/m10-quantized-inference`: safe scalar
  quantized kernels, dispatch-table routing, deterministic next-token stepping,
  explicit streaming state/buffers, and `make quantized-smoke` now prove a
  small quantized model path streams stable tokens. M10 added no unsafe blocks
  or functions; future SIMD work remains constrained to `kernels/**` and
  dispatch selection.
- **v0.21** — Opened M10 Quantized inference on
  `campaign/m10-quantized-inference`. The campaign owns scalar quantized kernel
  contracts, dispatch-table routing, deterministic token streaming from a
  validated model view, and smoke evidence before any SIMD-specific backend is
  introduced.
- **v0.20** — M9 completed on `campaign/m9-umdl-loader`: UMDL headers,
  sections, checksums, tokenizer metadata, tensor descriptors, loaded-model
  views, arena reservations, SIMD requirements, and profile RAM budgets now
  validate through fixed-width deterministic parsing. `make umdl-smoke` keeps
  fixture-generation and malformed-corpus evidence in aggregate verification.
  M9 added no unsafe blocks or functions.
- **v0.19** — Opened M9 UMDL loader on `campaign/m9-umdl-loader`.
  The campaign owns fixed-width UMDL parsing, section/tensor/checksum
  validation, tokenizer/model metadata extraction, and explicit arena
  reservation without exposing host paths, raw pointers, or hidden execution.
- **v0.18** — M8 completed on `campaign/m8-toy-transformer`: fixed-width toy
  model metadata validates exactly one hardcoded architecture, deterministic
  generation emits stable raw-byte token IDs into caller-provided buffers,
  prompt-to-text inference routes through the M7 tokenizer, and
  `make toy-transformer-smoke` is part of aggregate mission verification. M8
  added no unsafe code; memory-unsafe Rust remains allowed only where bounded,
  inspectable OS/model-kernel boundaries require it.
- **v0.17** — Opened M8 Toy transformer on `campaign/m8-toy-transformer`.
  The campaign owns a single tiny deterministic decoder-only model path,
  caller-provided inference buffers, and smoke evidence for deterministic token
  output without adding hidden inference loops or SIMD assumptions.
- **v0.16** — M7 completed on `campaign/m7-tokenizer`: fixed-width
  tokenizer metadata validates exactly the initial `RawByteToToken` family,
  no-alloc encode/decode paths round trip representative UTF-8 prompts through
  caller-provided buffers, and tokenizer smoke evidence is part of aggregate
  mission verification.
- **v0.15** — Opened M7 Tokenizer on `campaign/m7-tokenizer`.
  The campaign owns one bare-metal tokenizer family, tokenizer metadata
  validation, round-trip evidence, and graph-visible node contracts without
  adding hidden inference loops or direct mutation authority.
- **v0.14** — M6 completed on `campaign/m6-storage-stage-1`: raw-sector
  read works through an ATA PIO backend with finite timeout diagnostics, QEMU
  proves sector 0 marker reads from a deterministic primary-disk fixture, and
  aggregate verification preserves the opaque storage-resource namespace above
  the adapter boundary. The milestone intentionally uses bounded unsafe port
  I/O where hardware access requires it.
- **v0.13** — Opened M6 Storage stage 1 on
  `campaign/m6-storage-stage-1`. The campaign owns raw-sector read bring-up
  with finite polling, storage diagnostics, and the graph/resource namespace
  boundary before any FAT32 compatibility work.
- **v0.12** — M5 completed on `campaign/m5-minimal-ui`: framebuffer text
  primitives render over caller-provided memory, boot diagnostics have a
  no-UART framebuffer fallback path, verified graph display state is exposed as
  a read-only snapshot, and `make ui-smoke` keeps the minimal UI evidence in
  the aggregate verification bundle. Memory-unsafe Rust remains allowed at
  hardware boundaries, with M5 preserving bounded, inspectable, deterministic
  access rather than imposing a safe-Rust-only constraint.
- **v0.11** — Opened M5 Minimal UI on `campaign/m5-minimal-ui`.
  The campaign owns framebuffer text primitives, boot-diagnostic framebuffer
  fallback, and a minimal graph-state display without weakening the existing
  heartbeat, SSOD, or verified graph-load boundaries.
- **v0.10** — M4 completed on `campaign/m4-umod-loader`: persistent UMOD
  bytes parse into symbolic descriptors, all 22 verifier checks are
  non-vacuous, the source -> transform -> sink fixture compiles only through
  `graph_load_from_umod -> graph_compile_verified`, and golden/malformed
  fixture coverage is exercised by the verification bundle.
- **v0.9** — Opened M4 UMOD Loader on `campaign/m4-umod-loader`.
  The campaign owns persistent UMOD parsing, all 22 graph verifier checks,
  non-vacuous golden/malformed coverage, and execution through the existing
  single verifier gate.
- **v0.8** — M3 completed on `campaign/m3-embedded-graph`: the built-in
  symbolic graph verifies through `graph_load_from_umod`, compiles only through
  the loader, executes source -> transform -> sink, and has tests for epoch
  readiness, fan-out independence, and active-node diagnostic clearing.
- **v0.7** — Opened M3 Embedded Graph on
  `campaign/m3-embedded-graph`. The campaign explicitly preserves H2: even the
  hardcoded graph path must enter through symbolic bytes and the verifier /
  compile pipeline, with runtime graph internals private to the loader.
- **v0.6** — M2 completed on `campaign/m2-arena-memory`: bounded arena
  allocation now has real host-test coverage, the required named arenas exist
  behind guard-style APIs, exhaustion context routes into SSOD fields, and QEMU
  asserts the honest M2 memory/arena dump while normal boot remains green.
- **v0.5** — Opened M2 Arena Memory on `campaign/m2-arena-memory`.
  The campaign owns named bounded arena construction, aligned allocation,
  deterministic exhaustion, and a serial memory-map/arena diagnostic dump.
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
