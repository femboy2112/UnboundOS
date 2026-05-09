---
name: audit-arenas
description: Walk the kernel for every allocation site, classify by arena, and verify lifetime phase, alignment contract, scratch reset discipline, and diagnostic identity per spec §4. Use after any change to allocator code, arena setup, or memory-touching subsystems. Delegates to the arena-auditor subagent for the heavy read.
allowed-tools: Read, Glob, Grep, Bash, Task
---

# /audit-arenas

Run a full arena audit on the kernel.

## Procedure

1. Invoke the `arena-auditor` subagent with the full kernel as scope. The subagent
   has the spec §4 checklist baked in.

2. While the subagent runs, do these in parallel from the main thread:
   - `rg -n 'alloc_aligned\(' kernel/src` — every allocation site
   - `rg -n 'AllocError::OutOfArenaMemory' kernel/src` — exhaustion sites
   - `rg -n 'ScratchArena|ScratchTensorArena' kernel/src` — scratch users
   - `rg -n 'with_(boot|kernel|graph|scratch|model_weight|inference|kv_cache|tokenizer)_arena' kernel/src` — guard-fn users

3. Cross-check: every match in step 2's allocation site list must appear inside one
   of the guard-fn callsites. Any allocation outside a guard fn is a finding.

4. Produce a combined report from the subagent's findings plus the main-thread
   spot checks:

   ```
   # Arena Audit — <date> — <branch>

   ## Coverage
   Allocation sites: <n>. Through guard fn: <n>. Direct: <n>.

   ## Findings (subagent)
   <inline subagent report>

   ## Spot-check deltas
   - Guard-fn coverage: <n>/<n>
   - Exhaustion sites with full identity: <n>/<n>

   ## Verdict
   PASS | FAIL

   ## Required fixes
   - <bullets, file:line, spec section>
   ```

5. If the audit is FAIL, list the precise fixes needed. Do not apply them
   automatically — the main thread or operator decides.

## Notes

- This skill never modifies code. It reports.
- Cite spec §4 sections inline.
- If the kernel does not yet implement the arena module, mark the audit
  `EARLY: arena module not yet implemented; checklist will run when present` and
  list the missing pieces against the spec.
