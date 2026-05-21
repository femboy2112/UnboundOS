---
name: boot-heartbeat-check
description: Verify the boot order matches spec §3.2 (kernel entry contract) and §1.6 (boot-visible heartbeat rule). Walks _start, asserts the 14-step early order, confirms heartbeat strings are emitted before graphical init, and confirms the boot-diagnostic-buffer fallback exists (§3.9). Use after any change to early init, the serial driver, the UART probe, or the framebuffer init order.
allowed-tools: Read, Glob, Grep, Bash
---

# /boot-heartbeat-check

Verify boot is never blind.

## Required early order (spec §3.2)

1. disable interrupts
2. initialize serial logging
3. print boot begin heartbeat
4. validate bootloader handoff structures
5. install temporary GDT if needed
6. install early IDT with fatal handlers
7. ingest memory map
8. initialize boot allocator
9. probe CPU features
10. enable permitted SIMD/FPU state
11. initialize framebuffer if available
12. initialize permanent kernel structures
13. load or embed initial graph
14. enter orchestrator or IDE shell

## Required heartbeat strings (spec §1.6)

Before graphical init:
```
UNBOUNDOS_BOOT_BEGIN
UNBOUNDOS_CPU_PROFILE=<profile>
UNBOUNDOS_MEMMAP_OK=<available_bytes>
UNBOUNDOS_IDT_OK
UNBOUNDOS_BOOT_OK
```

UART-failure fallback (spec §3.9):
```
BOOT_NO_SERIAL
BOOT_HEARTBEAT_BUFFER_PRESENT
```

## Procedure

1. Read `_start` and the early init module:
   ```bash
   rg -n 'fn _start|extern "C" fn _start|pub unsafe extern' kernel/src
   ```

2. For each of the 14 steps, find its line in source. Produce a table of step
   number, step description, file:line, and PASS/FAIL/MISSING.

3. Verify each heartbeat string is emitted in order:
   ```bash
   rg -n 'UNBOUNDOS_BOOT_BEGIN|UNBOUNDOS_CPU_PROFILE|UNBOUNDOS_MEMMAP_OK|UNBOUNDOS_IDT_OK|UNBOUNDOS_BOOT_OK' kernel/src
   ```
   Confirm:
   - All five appear.
   - They appear in the order listed in §1.6.
   - All five appear before any framebuffer init (`init_framebuffer`, `clear_screen`,
     etc.).

4. Verify the UART-failure fallback exists:
   ```bash
   rg -n 'BOOT_NO_SERIAL|BOOT_HEARTBEAT_BUFFER_PRESENT' kernel/src
   ```
   Both must appear, with the second printed once the framebuffer is available.

5. (Optional) Boot in QEMU and confirm the strings appear in captured serial:
   ```
   /qemu-smoke
   ```

## Output

```
# Boot Heartbeat Check — <branch>

## Boot order (§3.2)
| # | Step                              | file:line                | verdict |
|---|-----------------------------------|--------------------------|---------|
| 1 | disable interrupts                | <path>                   | PASS    |
| ... | ...                             | ...                      | ...     |

## Heartbeat strings (§1.6)
- UNBOUNDOS_BOOT_BEGIN: <file:line>
- UNBOUNDOS_CPU_PROFILE: <file:line>
- UNBOUNDOS_MEMMAP_OK: <file:line>
- UNBOUNDOS_IDT_OK: <file:line>
- UNBOUNDOS_BOOT_OK: <file:line>
- Order before framebuffer init: yes | no

## UART fallback (§3.9)
- BOOT_NO_SERIAL: <file:line | MISSING>
- BOOT_HEARTBEAT_BUFFER_PRESENT: <file:line | MISSING>

## QEMU run (optional)
<PASS | FAIL with which strings missing>

## Verdict
PASS | FAIL — boot must never be blind

## Required fixes
- <bullets, file:line, spec section>
```

This skill never modifies code. It reports.
