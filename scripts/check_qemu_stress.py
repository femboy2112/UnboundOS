#!/usr/bin/env python3
"""Repeated QEMU smoke sweep for milestone runtime paths."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_ITERS = 2

TARGETS = (
    ("M0 heartbeat", ("make", "-s", "qemu-headless")),
    ("M0 no-serial fallback", ("make", "-s", "qemu-no-serial")),
    ("M1 SSOD divide_error", ("make", "-s", "qemu-fault-de")),
    ("M1 SSOD invalid_opcode", ("make", "-s", "qemu-fault-ud")),
    ("M1 SSOD page_fault", ("make", "-s", "qemu-fault-pf")),
    ("M2 arena/memory dump", ("make", "-s", "qemu-m2-dump")),
    ("M3/M4 graph boot", ("make", "-s", "qemu-graph-boot")),
    ("M5 framebuffer render", ("make", "-s", "qemu-framebuffer-smoke")),
    ("M6-M12 interactive shell", ("make", "-s", "qemu-interactive-smoke")),
    ("M6 storage marker", ("make", "-s", "qemu-storage-smoke")),
)


def stress_iters() -> int:
    raw = os.environ.get("UNBOUNDOS_STRESS_ITERS", str(DEFAULT_ITERS))
    try:
        value = int(raw)
    except ValueError as exc:
        raise ValueError(f"UNBOUNDOS_STRESS_ITERS must be an integer, got {raw!r}") from exc
    if value < 1:
        raise ValueError("UNBOUNDOS_STRESS_ITERS must be >= 1")
    return value


def main() -> int:
    try:
        iterations = stress_iters()
    except ValueError as exc:
        print(f"[qemu-stress] FAIL: {exc}", file=sys.stderr)
        return 2

    for iteration in range(1, iterations + 1):
        print(f"[qemu-stress] iteration {iteration}/{iterations}")
        for label, command in TARGETS:
            print(f"[qemu-stress] RUN {label}: {' '.join(command)}")
            subprocess.run(command, cwd=ROOT, check=True)
            print(f"[qemu-stress] PASS {label}")

    print(f"[qemu-stress] PASS: {iterations} iterations across {len(TARGETS)} runtime paths")
    return 0


if __name__ == "__main__":
    sys.exit(main())
