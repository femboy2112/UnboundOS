# UnboundOS Project Plan

Authoritative spec: `docs/UnboundOS_Tech_Spec_v2_1_1_Fidelity_Hardening.pdf`.

This plan maps the PDF milestone sequence to executable Codex campaigns. Each
mission must preserve the hard rules in `CLAUDE.md`: symbolic persistent
artifacts, verifier-gated graph runtime construction, no hidden execution, no
direct LLM mutation, loader-selected SIMD dispatch, named arenas, opaque
resource IDs, visible boot diagnostics, and structured SSOD failures.

## Operating Contract

When the operator says `go`, Codex must complete exactly one active mission by
default:

1. Load `CLAUDE.md`, this plan, `.codex/CURRENT_CAMPAIGN.md`, and
   `.codex/CURRENT_MISSION.md`.
2. Run `python3 scripts/status.py`.
3. Execute only the active mission scope.
4. Run `python3 scripts/verify.py --mission current`.
5. Use the matching `.agents/agents/*` review role for touched subsystems.
6. Update mission state and `.codex/MISSION_LOG.md`.
7. Stage only mission-owned files, commit, push, and stop.

When the operator explicitly approves a bundled run, Codex may complete
multiple adjacent missions in campaign order before stopping. Bundled runs must
stay off `main`, never merge to `main`, never push `main`, never force-push,
preserve every `CLAUDE.md` hard rule, keep per-mission log evidence, run each
mission's validation commands before completion, commit and push after each
completed mission, reload mission state after each checkpoint, and stop at the
next review gate, failed verification, blocker, or ambiguous scope.

## Campaigns

### C0 Control Plane

Install and validate the Codex-native workflow.

- C0.M1: completed mission/campaign docs, agents, skill, scripts, doc path
  alignment.
- C0.M2: completed `go` state transition by advancing to C1.M0 without touching
  implementation files.

### C1 M0 Boot Heartbeat

Active campaign: `docs/campaigns/m0-boot-heartbeat.md` on branch
`campaign/m0-boot-heartbeat`.

Exit criterion: QEMU boots the kernel, serial prints the required heartbeat,
and the kernel intentionally reaches halt/idle.

- Implement real COM1 serial output.
- Implement a real Limine image path or explicitly document and implement the
  spec-compatible boot image alternative.
- Add QEMU smoke assertion for `UNBOUNDOS_BOOT_OK`.
- Add no-UART diagnostic buffer fallback coverage.

### C2 M1 Diagnostics Core

Exit criterion: IDT and SSOD handle forced faults with structured serial output.

- Implement CPUID/XCR0 probing and guarded SIMD/FPU enablement.
- Install IDT handlers for #DE, #UD, #DF, #GP, #PF, and debug trap.
- Emit SSOD records with RIP, reason, and fault-family identity.

### C3 M2 Arena Memory

Exit criterion: named bounded arenas allocate, align, reset, and fail with
deterministic context.

- Implement Boot, Kernel, Graph, and Scratch arenas first.
- Add guard helpers and forbid direct allocation outside declared phases.
- Add exhaustion and alignment tests.

### C4 M3 Embedded Graph

Exit criterion: hardcoded graph executes once with epoch readiness and fan-out.

- Define minimal runtime node/wire types private to the graph runtime.
- Implement source -> transform -> sink execution.
- Add active node diagnostics.

### C5 M4 UMOD Loader

Exit criterion: valid symbolic UMOD verifies and executes; malformed UMODs
return structured errors.

- Implement UMOD byte parsing and section bounds checks.
- Implement all 22 verifier checks.
- Keep `graph_load_from_umod -> verifier -> graph_compile_verified` as the only
  runtime graph path.
- Add non-vacuous golden graph and fuzz corpus coverage.

### C6 M5-M6 UI And Storage

Exit criterion: minimal framebuffer IDE displays graph state, and raw storage
read works with timeout while graph-visible refs remain opaque.

- Implement framebuffer text primitives sufficient for diagnostics and graph
  display.
- Add minimal operator shell/IDE display path.
- Add raw sector read with timeout and no write-by-default behavior.

### C7 M7-M10 Local LLM Core

Exit criterion: local graph-native inference path streams deterministic tokens
from a validated model package.

- Implement tokenizer round trip.
- Implement hardcoded tiny transformer.
- Implement UMDL header/tensor/checksum validation and arena reservation.
- Implement scalar CPU kernels, then dispatch-selected SIMD tiers.

### C8 M11-M12 Assistant And Retrieval

Exit criterion: assistant explains graph/SSOD state and searches local docs
without direct mutation authority.

- Implement graph and SSOD explainer nodes.
- Route tool-planning through structured action buffer, schema validation,
  temporary UMOD patch, verifier, operator approval, and reload.
- Implement local docs retrieval and context packing.

## Verification Ladder

Use the narrowest reliable checks first, then widen:

1. Mission script checks: `python3 scripts/status.py`,
   `python3 scripts/mission.py validate`, `python3 scripts/verify.py`.
2. Static fidelity scripts in `scripts/check_*.sh` and `scripts/address_scan.py`.
3. Rust checks once toolchain is available: fmt, clippy, host tests, kernel
   build.
4. QEMU smoke and profile-specific tests once image generation is real.
5. Golden fixtures and fuzz corpora once parsers/builders exist.
