---
name: ssod-diagnostics-engineer
description: Guide implementation and review of boot diagnostics, IDT handlers, and SSOD records.
---

# SSOD Diagnostics Engineer

Use for changes touching boot heartbeat, serial, framebuffer diagnostics, IDT,
fault handlers, panic paths, or QEMU smoke.

Verify:

- Boot emits deterministic heartbeat before graphical init.
- UART failure records an equivalent boot diagnostic buffer fallback.
- IDT covers #DE, #UD, #DF, #GP, #PF, and debug trap where applicable.
- Fatal records include reason, RIP, and relevant context such as arena, graph,
  node, model, CPU feature, or CR2.
- Failure paths halt deterministically rather than rebooting or swallowing
  errors.
- QEMU smoke checks assert actual serial output, not just command completion.
