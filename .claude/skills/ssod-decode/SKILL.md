---
name: ssod-decode
description: Parse a Snark Screen of Death record from a serial log or framebuffer dump and explain the structured fields per spec §9.7. Use when triaging a crash, reviewing a QEMU log, or debugging an exception path. Cross-references the snark matrix (§9.6) and fault code families (Appendix B).
argument-hint: <path-to-log-or-record>
allowed-tools: Read, Bash, Grep
---

# /ssod-decode

Decode an SSOD record from `$ARGUMENTS`.

## Procedure

1. Resolve `$ARGUMENTS` to a file (serial log, framebuffer dump text, paste). If a
   directory, search recent log files.

2. Locate SSOD blocks. They begin with a fault marker like:
   ```
   ===== UNBOUNDOS SSOD =====
   ```
   and contain the structured record per spec §9.7.

3. For each block, parse the spec §9.7 fields and present them:

   ```
   # SSOD Decode — <source>

   ## Block <n>
   kernel version      : <semver>
   build profile       : <qemu-dev | legacy-bios | modern-x86_64 | t500-class>
   CPU feature profile : <Scalar | Sse2 | Avx | Avx2 | Avx512>
   fault type          : <PanicReason variant>     # e.g. CPU_PAGE_FAULT
   fault family        : <BOOT_*|MEM_*|GRAPH_*|NODE_*|STORAGE_*|IDE_*|LLM_*|CPU_*>
   instruction pointer : 0x<hex>
   stack pointer       : 0x<hex>
   error code          : 0x<hex> | absent
   active graph ID     : <u64> | none
   active node ID      : <u32> | none
   active model ID     : <ModelId> | none
   active arena        : <BootArena|...|InferenceArena|...> | none
   last serial chkpt   : <string>                  # e.g. UNBOUNDOS_IDT_OK
   recommended next    : <text>

   ## Snark
   "<snark line as printed>" — matches §9.6 entry: <fault row>

   ## Interpretation
   <one paragraph of analysis, citing the spec rule that triggered the fault and
    pointing at the most likely faulting subsystem given the active graph/node/model
    and the last serial checkpoint>

   ## Suggested next step
   <e.g., "rerun with /qemu-smoke after reviewing the page-fault handler's CR2 path,
    spec §9.4. Then run /audit-arenas if MEM_*; /verify-graph if GRAPH_*; etc.">
   ```

4. If the record is missing any §9.7 field, that itself is a finding (the SSOD
   render is non-conforming). List missing fields in a `## Conformance` section
   and reference the `ssod-diagnostics-engineer` subagent for a fix.

5. If the record uses a snark line that is not in the §9.6 matrix, note it. The
   snark text is style and may evolve, but the structured fields are not optional.

6. If multiple SSOD blocks exist (a fault during fault), the second block's
   `prior fault if known` field SHOULD reference the first. Verify the chain.

## Important

- Do not propose code changes from this skill. Identify the fault, point at the
  subsystem, and let the operator route to the appropriate subagent.
- The boot-heartbeat is not an SSOD block; if the log shows only `UNBOUNDOS_BOOT_*`
  lines without an SSOD, run `/boot-heartbeat-check` instead.
