# Mission Log

Append one entry per completed mission. Keep entries concise and factual.

## Pending

- C7.M6 Step 2 ATA PIO sector-read primitive: ready.

## 2026-05-23T07:17:28Z - C7.M6 Step 1 Storage contracts and timeout model

- Status: completed
- Summary: Added `kernel/src/storage.rs` with fixed-width storage diagnostics,
  a read-sector request surface, finite ATA status polling, LBA28 validation,
  no default write support, and host tests for ready, device-error, timeout,
  zero-budget, and out-of-range behavior.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make fmt`, `make clippy`, `rustc --test kernel/src/storage.rs`,
  and `python3 scripts/verify.py --mission current`.
- Memory-unsafety audit: Step 1 introduces no new unsafe block; this is
  intentional because real ATA PIO port I/O is Step 2. The contract permits
  unsafe storage access when it remains bounded, inspectable, deterministic,
  and not undefined by design.
- Blockers: none.

## 2026-05-23T07:13:10Z - C7.M6 campaign activation

- Status: completed
- Summary: Opened `campaign/m6-storage-stage-1`, marked M6 `IN-PROGRESS`,
  created the M6 campaign plan, and rotated `.codex` state to Step 1 while
  preserving the project rule that memory-unsafe Rust is allowed at bounded,
  inspectable hardware boundaries.
- Verification: pending below for the active Step 1 mission state.
- Blockers: none.

## 2026-05-23T07:12:42Z - C6.M5 Step 5 M5 completion audit

- Status: completed
- Summary: Marked M5 `DONE`, bumped the milestone catalog to `v0.12`,
  recorded the Step 1-4 checkpoint commits in the campaign closeout, and
  preserved the campaign constraint that memory-unsafe Rust is allowed at
  hardware boundaries when bounded, inspectable, deterministic, and not
  undefined by design.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make gates`, `make repo-state`, and `python3 scripts/verify.py
  --mission current`.
- Repo-state: expected STOP because no milestone is `IN-PROGRESS` after M5
  closeout.
- Blockers: none for M5; next action is final M5 PR or M6 rotation.

## 2026-05-23T07:10:27Z - C6.M5 Step 4 UI smoke evidence and gates

- Status: completed
- Summary: Added `make ui-smoke` and `scripts/check_ui_smoke.py` to prove the
  framebuffer graph-state renderer, read-only graph display snapshot, and
  no-serial smoke assertion remain source-reachable without requiring graphical
  CI. Wired the UI smoke into `python3 scripts/verify.py --mission current`.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make fmt`, `make clippy`, `make ui-smoke`, `make gates`, and
  `python3 scripts/verify.py --mission current`.
- Blockers: none.

## 2026-05-23T07:08:52Z - C6.M5 Step 3 Minimal graph-state display model

- Status: completed
- Summary: Added a copied read-only `GraphDisplayState` snapshot on
  `GraphRuntimeHandle`, kept its constructor crate-private, preserved runtime
  construction inside `loader.rs`, and added framebuffer text rendering for
  graph id, node count, wire count, active node, and last completed node.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make fmt`, `make clippy`, `cargo test -p graph`, and `python3
  scripts/verify.py --mission current`.
- Review: `graph-verifier-auditor` checks passed; no public runtime
  constructor, verifier bypass, or graph mutation surface was added.
- Blockers: none.

## 2026-05-23T07:06:21Z - C6.M5 Step 2 Boot diagnostic framebuffer fallback

- Status: completed
- Summary: Replaced the TODO-only framebuffer fallback with a real
  `TextSurface` call path, preserved headless boot by passing `None` until real
  framebuffer handoff exists, and repaired `make qemu-no-serial` to verify boot
  completion via QEMU debug-exit instead of a disabled serial log.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make fmt`, `make clippy`, `make qemu-headless`, `make
  qemu-no-serial`, and `python3 scripts/verify.py --mission current`.
- Memory-unsafety audit: unsafe is allowed by project identity; the new unsafe
  boundaries are explicit and bounded (`boot_diag::snapshot` under boot-phase
  exclusivity and test-only port `0xF4` debug-exit for no-serial QEMU smoke).
- Blockers: none.

## 2026-05-23T06:59:40Z - C6.M5 Step 1 Framebuffer text surface primitives

- Status: completed
- Summary: Added `kernel/src/framebuffer.rs` with boot-passive text-cell
  rendering over caller-provided linear pixel memory and registered the module
  in `kernel/src/main.rs`. The surface uses checked buffer sizing, checked
  pixel indexing, explicit clipping, and no global framebuffer pointer.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make fmt`, `make clippy`, `make kernel`, and `python3
  scripts/verify.py --mission current`.
- Memory-unsafety audit: current Step 1 code introduces no new unsafe block;
  this is intentional because the real hardware/MMIO boundary is not in Step 1.
  Future unsafe framebuffer access remains allowed when bounded, inspectable,
  deterministic, and not undefined by design.
- Blockers: none.

## 2026-05-23T06:55:00Z - C6.M5 campaign activation

- Status: completed
- Summary: Opened `campaign/m5-minimal-ui`, marked M5 `IN-PROGRESS`, created
  the M5 campaign plan, and rotated `.codex` state to Step 1.
- Verification: pending below for the active Step 1 mission state.
- Blockers: none.

## 2026-05-23T06:52:36Z - C5.M4 Step 7 M4 completion audit

- Status: completed
- Summary: Marked M4 `DONE` in `MILESTONE_CATALOG.md`, bumped the catalog to
  `v0.10`, recorded M4 closeout in the catalog change log, and appended the
  Step 1-6 checkpoint SHAs to `docs/campaigns/m4-umod-loader.md`.
- Verification: `make gates`, `make repo-state`, and `python3
  scripts/verify.py --mission current`.
- Repo-state: expected STOP because no milestone is `IN-PROGRESS` after M4
  closeout.
- Blockers: none for M4; next action is final M4 PR or M5 rotation.

## 2026-05-23T06:51:16Z - C5.M4 Step 6 Golden and malformed fixture coverage

- Status: completed
- Summary: Registered the source-transform-sink golden fixture, added malformed
  UMOD corpus cases for bad magic/version, truncated header, out-of-bounds
  sections, overlap, huge counts, invalid refs, and unbroken cycles, and added
  graph verifier tests that include every fixture directly from checkout.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make fmt`, `make clippy`, `cargo test -p umod`, `cargo test -p
  graph`, `python3 scripts/address_scan.py tests/golden_graphs
  tests/golden_models`, `python3 scripts/verify.py --mission current`, and
  `make gates`.
- Blockers: none.

## 2026-05-23T06:48:28Z - C5.M4 Step 5 Persistent UMOD compile path

- Status: completed
- Summary: Added persistent source -> transform -> sink UMOD bytes under
  `tests/golden_graphs`, removed the old verifier sentinel bypass, and routed
  the fixture through `graph_load_from_umod` and `graph_compile_verified` into
  the private loader runtime path.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make fmt`, `make clippy`, `cargo test -p umod`, `cargo test -p
  graph`, `python3 scripts/address_scan.py tests/golden_graphs
  tests/golden_models`, `python3 scripts/verify.py --mission current`, and
  `make gates`.
- Review: `graph-verifier-auditor` checks passed; `VerifiedGraph` construction
  remains verifier-only, runtime construction remains private to `loader.rs`,
  and no test-only runtime constructor or verifier bypass was added.
- Blockers: none.

## 2026-05-23T02:40:33Z - C5.M4 Step 4 Capabilities, resources, constants, and scheduling checks

- Status: completed
- Summary: Completed non-vacuous verifier checks for payload bounds, GraphArena
  budget, model refs, section checksums, UI layout, constant blobs,
  deterministic scheduling, and opaque external resource syntax.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make fmt`, `make clippy`, `cargo test -p umod`, `cargo test -p
  graph`, and `python3 scripts/verify.py --mission current`.
- Blockers: none.

## 2026-05-23T02:29:49Z - C5.M4 Step 3 Node and wire semantic verifier checks

- Status: completed
- Summary: Added fixed-width node, wire, and pin-type decoding plus verifier
  checks for duplicate/unresolved node indices, unresolved wire endpoints,
  pin bounds, wire type compatibility, known node types, declared capability
  ranges, and simple unbroken cycles.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make fmt`, `make clippy`, `cargo test -p umod`, `cargo test -p
  graph`, and `python3 scripts/verify.py --mission current`.
- Blockers: none.

## 2026-05-23T02:21:56Z - C5.M4 Step 2 Section table bounds and structural checks

- Status: completed
- Summary: Added fixed-width section descriptor decoding, structural UMOD
  validation for declared file length, section table bounds, section
  out-of-file errors, illegal overlaps, and configured node/wire count limits.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make fmt`, `make clippy`, `cargo test -p umod`, `cargo test -p
  graph`, and `python3 scripts/verify.py --mission current`.
- Blockers: none.

## 2026-05-23T02:15:38Z - C5.M4 Step 1 UMOD parser header and resource refs

- Status: completed
- Summary: Added fixed-width little-endian UMOD header parsing, structured
  parser errors for bad magic/version/short header/bad length, real opaque
  resource reference validation, and graph verifier mapping for those parser
  failures.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make fmt`, `make clippy`, `cargo test -p umod`, `cargo test -p
  graph`, and `python3 scripts/verify.py --mission current`.
- Blockers: none.

## 2026-05-23T02:06:17Z - C5.M4 campaign activation

- Status: completed
- Summary: Opened `campaign/m4-umod-loader`, marked M4 `IN-PROGRESS`,
  created the M4 campaign plan, and rotated `.codex` state to Step 1 while
  preserving the single verifier gate.
- Verification: pending below for the active Step 1 mission state.
- Blockers: none.

## 2026-05-23T01:57:15Z - C4.M3 Step 5 M3 completion audit

- Status: completed
- Summary: Marked M3 `DONE` in `MILESTONE_CATALOG.md`, bumped the catalog to
  `v0.8`, recorded the Step 1-4 checkpoint commits in the campaign closeout,
  and left M4 for operator rotation.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make gates`, `make repo-state`, and `python3 scripts/verify.py
  --mission current`.
- Repo-state: STOP because no milestone is `IN-PROGRESS`, which is the
  expected closed-M3 state.
- Blockers: none for M3; next action is M4 rotation.

## 2026-05-23T01:54:40Z - C4.M3 Step 4 Active node diagnostics

- Status: completed
- Summary: Added private active-node tracking to the built-in graph runtime,
  cleared it after each node fired, and added graph tests proving the active
  node is clear after execution while the last completed node records the sink.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make fmt`, `make clippy`, `python3 scripts/verify.py --mission
  current`, and `make gates`.
- Blockers: none.

## 2026-05-23T01:49:41Z - C4.M3 Step 3 Fan-out execution proof

- Status: completed
- Summary: Added a graph-crate fan-out test proving two consumers can observe
  the same produced wire epoch independently and one consumer observation does
  not clear readiness for the other.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make fmt`, `make clippy`, `python3 scripts/verify.py --mission
  current`, and `make gates`.
- Blockers: none.

## 2026-05-23T01:47:06Z - C4.M3 Step 2 Private hardcoded graph runtime

- Status: completed
- Summary: Added a symbolic built-in source/transform/sink payload that passes
  `graph_load_from_umod`, compiles through `graph_compile_verified`, and
  executes once through private runtime structures in `loader.rs`.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make fmt`, `cargo test -p graph`, `python3 scripts/verify.py
  --mission current`, and `make gates`.
- Blockers: none.

## 2026-05-23T01:42:42Z - C4.M3 Step 1 Runtime epoch readiness primitives

- Status: completed
- Summary: Added private graph runtime wire/consumer epoch observation
  primitives inside the loader module and tests proving readiness follows
  `wire_epoch > last_observed_epoch`.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make fmt`, `cargo test -p graph`, `python3 scripts/verify.py
  --mission current`, and `make gates`.
- Blockers: none.

## 2026-05-23T01:36:00Z - C4.M3 campaign activation

- Status: completed
- Summary: Opened `campaign/m3-embedded-graph`, marked M3 `IN-PROGRESS`,
  created the M3 campaign plan, and rotated `.codex` state to Step 1 while
  preserving the H2 verifier-gate boundary.
- Verification: pending below for the active Step 1 mission state.
- Blockers: none.

## 2026-05-23T01:32:37Z - C3.M2 Step 5 M2 completion audit

- Status: completed
- Summary: Marked M2 `DONE` in `MILESTONE_CATALOG.md`, bumped the catalog to
  `v0.6`, recorded the M2 closeout and Step 1-4 commit SHAs in the campaign
  file, and left M3 as future milestone work.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make qemu-m2-dump`, `make gates`, `make repo-state`, and
  `python3 scripts/verify.py --mission current`.
- Repo-state: STOP because no milestone is `IN-PROGRESS`, which is the expected
  closed-M2 state.
- Blockers: none for M2; next action is M3 rotation.

## 2026-05-23T01:30:50Z - C3.M2 Step 4 Memory-map and arena dump

- Status: completed
- Summary: Added an honest serial M2 memory/arena dump that reports the smoke
  profile memory map as unavailable while listing the required named arena
  descriptors, plus a QEMU assertion target for that dump.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `bash -n scripts/qemu.sh`, `make qemu-m2-dump`, `make
  qemu-headless`, `python3 scripts/verify.py --mission current`, and
  `make gates`.
- Blockers: none.

## 2026-05-23T01:26:10Z - C3.M2 Step 3 Deterministic exhaustion diagnostics

- Status: completed
- Summary: Added arena fault context extraction for exhaustion errors and
  taught SSOD to serialize arena identity, requested size, alignment, base,
  cursor, and limit while keeping graph/node/model context explicitly absent.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make fmt`, `make clippy`, `python3 scripts/verify.py --mission
  current`, and `make gates`.
- Blockers: none.

## 2026-05-23T01:22:57Z - C3.M2 Step 2 Named M2 arena set

- Status: completed
- Summary: Added BootArena, KernelArena, GraphArena, and ScratchArena
  descriptors with declared phases, plus an `M2ArenaSet` whose allocation
  surface goes through named guard-style methods.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make fmt`, `make clippy`, `make kernel`, `make qemu-headless`,
  `python3 scripts/verify.py --mission current`, and `make gates`.
- Blockers: none.

## 2026-05-23T01:18:28Z - C3.M2 Step 1 Bounded arena core and alignment checks

- Status: completed
- Summary: Implemented the bounded `Arena` cursor contract with explicit
  alignment rejection, checked overflow handling, deterministic exhaustion
  context, reset support, and verifier-run host tests for the arena module.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make fmt`, `make clippy`, `python3 scripts/verify.py --mission
  current`, and `make gates`.
- Blockers: none.

## 2026-05-23T01:06:00Z - C3.M2 campaign activation

- Status: completed
- Summary: Opened `campaign/m2-arena-memory`, marked M2 `IN-PROGRESS`,
  created the M2 campaign plan, and rotated `.codex` state to Step 1.
- Verification: pending below for the active Step 1 mission state.
- Blockers: none.

## 2026-05-23T01:03:14Z - C2.M1 Step 5 M1 completion audit

- Status: completed
- Summary: Marked M1 `DONE` in `MILESTONE_CATALOG.md`, bumped the catalog to
  `v0.4`, recorded the M1 closeout and Step 1-4 commit SHAs in the campaign
  file, and left M2 as future milestone work.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make qemu-fault-de`, `make qemu-fault-ud`, `make
  qemu-fault-pf`, `make gates`, `make repo-state`, and `python3
  scripts/verify.py --mission current`.
- Repo-state: STOP because no milestone is `IN-PROGRESS`, which is the expected
  closed-M1 state.
- Blockers: none for M1; next action is M2 rotation.

## 2026-05-23T01:01:52Z - C2.M1 Step 4 Page-fault SSOD proof

- Status: completed
- Summary: Verified the #PF forced-fault path through the Step 1 harness and
  tightened the QEMU SSOD assertion so `page_fault` records must include a hex
  `error_code`.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `bash -n scripts/qemu.sh`, `make qemu-fault-pf`, `make
  qemu-headless`, and `python3 scripts/verify.py --mission current`.
- Blockers: none.

## 2026-05-23T00:59:23Z - C2.M1 Step 3 Invalid-opcode SSOD proof

- Status: completed
- Summary: Verified the #UD forced-fault path through the Step 1 harness.
  `make qemu-fault-ud` asserted the SSOD begin marker,
  `reason=invalid_opcode`, RIP field, and SSOD end marker while normal boot
  still reached `UNBOUNDOS_BOOT_OK`.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make qemu-fault-ud`, `make qemu-headless`, and `python3
  scripts/verify.py --mission current`.
- Blockers: none.

## 2026-05-23T00:57:47Z - C2.M1 Step 2 Divide-by-zero SSOD proof

- Status: completed
- Summary: Verified the #DE forced-fault path through the Step 1 harness.
  `make qemu-fault-de` asserted the SSOD begin marker, `reason=divide_error`,
  RIP field, and SSOD end marker while normal boot still reached
  `UNBOUNDOS_BOOT_OK`.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make qemu-fault-de`, `make qemu-headless`, and `python3
  scripts/verify.py --mission current`.
- Blockers: none.

## 2026-05-23T00:55:45Z - C2.M1 Step 1 Forced-fault smoke harness

- Status: completed
- Summary: Added explicit compile-time forced-fault selectors for
  `divide_error`, `invalid_opcode`, and `page_fault`, wired QEMU SSOD
  assertions for reason/RIP/end markers, and added dedicated Makefile targets
  while preserving normal heartbeat boot.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `bash -n scripts/qemu.sh`, `make fmt`, `make clippy`,
  `make kernel`, `make qemu-headless`, `make qemu-fault-de`,
  `make qemu-fault-ud`, `make qemu-fault-pf`, `make repo-state`,
  `python3 scripts/verify.py --mission current`, and `make gates`.
- Blockers: none.

## 2026-05-23T00:49:00Z - C2.M1 campaign activation

- Status: completed
- Summary: Opened `campaign/m1-diagnostics-core`, corrected the stale catalog
  drift so M1 matches spec §13.3 Diagnostics Core, created the M1 campaign
  plan, and rotated `.codex` state to Step 1.
- Verification: pending below for the active Step 1 mission state.
- Blockers: none.

## 2026-05-23T00:41:41Z - C1.M0 Step 8 M0 completion audit

- Status: completed
- Summary: Marked M0 `DONE` in `MILESTONE_CATALOG.md`, bumped the catalog to
  `v0.2`, recorded the M0 closeout and Step 1-7 commit SHAs in the campaign
  file, and left M1 as operator-rotated future work.
- Verification: `python3 scripts/status.py`, `python3 scripts/mission.py
  validate`, `make gates`, `make repo-state`, and `python3
  scripts/verify.py --mission current`.
- Repo-state: STOP because no milestone is `IN-PROGRESS`, which is the expected
  closed-M0 state.
- Blockers: none for M0; next action is operator PR/mission rotation.

## 2026-05-23T00:34:26Z - C1.M0 Step 7 QEMU smoke headless assertion

- Status: completed
- Summary: Replaced the placeholder image with an M0-only GRUB Multiboot2 ISO
  smoke path, added a small 32-bit bootstrap that enters long mode before
  calling the existing Rust `_start`, taught QEMU to boot ISO images and stop
  headless runs after `UNBOUNDOS_BOOT_OK`, and made the gates pipeline rebuild
  the image before asserting heartbeat order.
- Verification: `python3 scripts/status.py`,
  `python3 scripts/mission.py validate`, `bash -n scripts/qemu.sh
  scripts/gates.sh scripts/make_image.sh`, `make fmt`, `make clippy`,
  `make kernel`, `make qemu-headless`, and `make gates`.
- Notes: this is an M0 smoke boot path only. Limine handoff, bootloader
  information parsing, real memory-map ingestion, and allocator setup remain
  later milestones.
- Blockers: none.

## 2026-05-23T00:20:05Z - C1.M0 Step 7 QEMU smoke headless assertion

- Status: blocked
- Summary: Added `scripts/qemu.sh --assert-heartbeat` with stale-log clearing,
  ordered serial-log matching for the five canonical heartbeat markers, and
  early QEMU termination once `UNBOUNDOS_BOOT_OK` is observed. Normal
  `make qemu-headless` remains non-asserting.
- Verification: `python3 scripts/status.py`,
  `python3 scripts/mission.py validate`, `bash -n scripts/qemu.sh`,
  `make fmt`, `make clippy`, `make kernel`, and
  `python3 scripts/verify.py --mission current`.
- Runtime gates: `make qemu-headless` timed out through the deliberate
  placeholder image path. `make gates` passed 5/6 gates and failed only at
  `qemu-smoke heartbeat` because no serial heartbeat was observed from the
  placeholder image.
- Resolution: replaced the placeholder with the M0 Multiboot2 smoke ISO path
  in the completed Step 7 entry above.
- Blockers: none after the Step 7 completion pass.

## 2026-05-23T00:13:59Z - C1.M0 Step 6 Panic path routed through SSOD

- Status: completed
- Summary: Documented the operator-approved bundled-run workflow with
  per-mission validation/commit/push checkpoints and no-main guards, aligned
  the Step 6 campaign paths to the live `ssod.rs` / `idt.rs` code, hardened
  mission validation to enforce the campaign branch and main policy, and
  changed the M0 panic path to emit `UNBOUNDOS_SSOD_BEGIN` /
  `UNBOUNDOS_SSOD_END` with key=value fields to both serial and `boot_diag`.
- Verification: `python3 scripts/status.py`,
  `python3 scripts/mission.py validate`, `make fmt`, `make clippy`,
  `make kernel`, SSOD source review, and
  `python3 scripts/verify.py --mission current`.
- Notes: the SSOD record intentionally uses explicit `none` context for
  arena/graph/node/model IDs because those subsystems are later milestones.
- Blockers: none.

## 2026-05-23T00:03:42Z - C1.M0 Step 5 Review gate

- Status: blocked
- Summary: Re-validated the active review gate after Step 4 and recorded the
  required stop without advancing to Step 6 or touching implementation files.
- Verification: `python3 scripts/status.py`,
  `python3 scripts/mission.py validate`, review of
  `docs/campaigns/m0-boot-heartbeat.md` Step 5, and
  `python3 scripts/verify.py --mission current`.
- Stop reason: review-gate. The campaign requires explicit operator approval
  before Steps 6+ run.
- Resolution: operator later approved continuing past the review gate in an
  explicit bundled run while preserving spec adherence and editing working
  code.
- Blockers: none after operator approval.

## 2026-05-22T23:56:36Z - C1.M0 Step 4 Boot-diagnostic-buffer fallback

- Status: completed
- Summary: Promoted the boot-diagnostic-buffer fallback markers into
  `boot_diag` source-visible symbols, routed failed UART probes through the
  `BOOT_NO_SERIAL` marker, and kept every heartbeat emission recording into
  the diagnostic buffer while leaving framebuffer and Step 7 QEMU assertion
  work out of scope.
- Verification: `python3 scripts/status.py`,
  `python3 scripts/mission.py validate`, source audit of
  `kernel/src/serial.rs`, `kernel/src/heartbeat.rs`, `kernel/src/boot_diag.rs`,
  and `kernel/src/boot.rs`, `make fmt`, `make clippy`, `make kernel`, and
  `python3 scripts/verify.py --mission current`.
- Notes: `make qemu-no-serial` was run and timed out after the current
  placeholder image path; this remains non-authoritative for Step 4 per
  `.codex/CURRENT_MISSION.md` and is still owned by the later QEMU assertion
  path.
- Blockers: none.

## 2026-05-21T04:38:06Z - C1.M0 Step 3 IDT install and `UNBOUNDOS_IDT_OK`

- Status: completed
- Summary: Wired the M0-required fatal IDT vectors (#DE, #UD, #DF, #GP, #PF)
  through `ssod::kernel_panic` with `DiagnosticContext`, added a minimal
  serial/boot-diagnostic-buffer SSOD stub record, and preserved the existing
  `UNBOUNDOS_IDT_OK` emission immediately after `idt::install()`.
- Verification: `python3 scripts/status.py`,
  `python3 scripts/mission.py validate`, source audit of `kernel/src/boot.rs`,
  `kernel/src/idt.rs`, and `kernel/src/ssod.rs`, `make fmt`, `make clippy`,
  `make kernel`, and `python3 scripts/verify.py --mission current`.
- Notes: read-only subagent audits flagged the campaign tension between
  installing IDT before real memory-map ingest and preserving the documented
  heartbeat order (`MEMMAP_OK` before `IDT_OK`). Current M0 still uses a
  zero-byte placeholder rather than real memory-map traversal; Step 7 remains
  the QEMU heartbeat assertion owner.
- Blockers: none.

## 2026-05-21T04:32:10Z - C1.M0 Step 2 Serial UART probe and heartbeat string emission

- Status: completed
- Summary: Verified the existing Step 2 implementation without code changes:
  COM1 initializes through an internal loopback probe, failed UART probes leave
  writes disabled while heartbeat records to `boot_diag`, and
  `UNBOUNDOS_BOOT_BEGIN`, `UNBOUNDOS_CPU_PROFILE`, and `UNBOUNDOS_MEMMAP_OK`
  are emitted in source order.
- Verification: `python3 scripts/status.py`,
  `python3 scripts/mission.py validate`, source audit of
  `kernel/src/serial.rs`, `kernel/src/heartbeat.rs`, and `kernel/src/boot.rs`,
  `make fmt`, `make clippy`, `make kernel`, and
  `python3 scripts/verify.py --mission current`.
- Notes: read-only subagent audits found no H2/H3/H6/H9/H10 implementation
  violation in the Step 2 surface. They flagged the archived exact-line QEMU
  grep and `--assert-heartbeat` gate as Step 7/tooling concerns; those remain
  out of Step 2 implementation scope.
- Blockers: none.

## 2026-05-21T04:27:24Z - C1.M0 Step 1 Boot-order assertion vs spec §3.2

- Status: completed
- Summary: Added explicit source-level `spec §3.2 step <N>` assertions for the
  full 14-step kernel-entry contract in `kernel/src/boot.rs` while preserving
  existing boot behavior and later-milestone TODO boundaries.
- Verification: `python3 scripts/mission.py validate`, `make fmt`,
  `make clippy`, `/boot-heartbeat-check` source walk, and
  `python3 scripts/verify.py --mission current`.
- Blockers: none.

## 2026-05-21T04:20:07Z - C0.M2 Mission state handoff validation

- Status: completed
- Summary: Validated the Codex-native `go` workflow against the installed
  control surface, confirmed status and verification commands pass, and
  advanced the active mission to C1.M0 Step 1 without touching implementation
  files.
- Verification: `python3 scripts/status.py`,
  `python3 scripts/mission.py validate`,
  `python3 scripts/verify.py --mission current --dry-run`, and
  `python3 scripts/verify.py --mission current`.
- Blockers: none.

## 2026-05-21T03:38:23Z - C0.M1 Codex mission harness

- Status: completed
- Summary: Installed Codex-native mission/campaign state, project plan, local
  review roles, `unboundos-go` skill, status/mission/verify scripts, and
  documentation path reconciliation. Installed the pinned Rust toolchain,
  repaired user-local tool discovery, and cleared mechanical fmt/clippy/custom
  target blockers so full fidelity can run.
- Verification: `python3 scripts/status.py`,
  `python3 scripts/mission.py validate`,
  `python3 scripts/verify.py --mission current --dry-run`,
  `python3 scripts/verify.py --mission current`, and
  `env PATH=/home/leah/.cargo/bin:$PATH make fidelity`.
- Blockers: none.
