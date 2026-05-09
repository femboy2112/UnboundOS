#!/usr/bin/env python3
"""
address_scan.py — UnboundOS persistent-pointer detector.

Scans .umod / .umdl / .mod files for byte sequences that resemble
runtime addresses, kernel virtual addresses, or known sentinel
patterns. Persistent UMOD/UMDL files contain only fixed-width
integers, offsets, lengths, type IDs, checksums, and constant blobs.
Pointer-like values mean a host-side serializer leaked runtime
state, which is forbidden by spec §6.10 and CLAUDE.md §3.

Usage:
    python3 scripts/address_scan.py [path ...]

If no path is given, scans tests/golden_graphs and tests/golden_models.

Exits 0 if no hits, 1 if any hit was found.
"""

from __future__ import annotations

import os
import struct
import sys
from pathlib import Path

# Canonical x86_64 kernel virtual address range (high half).
KERNEL_HALF_LO = 0xFFFFFFFF80000000
KERNEL_HALF_HI = 0xFFFFFFFFFFFFFFFF

# x86_64 user-space upper boundary; values above this are suspicious
# in a persistent file because they cannot be legitimate offsets,
# lengths, or counts.
USER_UPPER = 0x0000_7FFF_FFFF_FFFF

# Known "looks like a pointer dump" sentinels.
SENTINELS = {
    0xDEADBEEFDEADBEEF: "DEADBEEF sentinel",
    0xCAFEBABECAFEBABE: "CAFEBABE sentinel",
    0xCDCDCDCDCDCDCDCD: "uninitialized memory poison (0xCD)",
    0xABABABABABABABAB: "uninitialized memory poison (0xAB)",
    0xFEEEFEEEFEEEFEEE: "freed memory poison (0xFEEE)",
}


def classify(value: int) -> str | None:
    if value in SENTINELS:
        return SENTINELS[value]
    if KERNEL_HALF_LO <= value <= KERNEL_HALF_HI:
        return "kernel virtual address range"
    # User-space upper-half pointers: values 0x0000_4000_0000_0000 and
    # above almost never appear as legitimate offsets in our format.
    if 0x0000_4000_0000_0000 <= value <= USER_UPPER:
        return "user-space pointer range"
    return None


def scan_file(path: Path) -> list[tuple[int, int, str]]:
    """Return list of (offset, value, classification) hits."""
    data = path.read_bytes()
    hits: list[tuple[int, int, str]] = []
    # Slide an 8-byte LE window across the whole file at every byte
    # offset. We do not exempt header regions because the legitimate
    # values there fall well below KERNEL_HALF_LO.
    for off in range(0, len(data) - 8 + 1):
        (val,) = struct.unpack_from("<Q", data, off)
        cls = classify(val)
        if cls is not None:
            hits.append((off, val, cls))
    return hits


def collect_targets(args: list[str]) -> list[Path]:
    if not args:
        args = ["tests/golden_graphs", "tests/golden_models"]
    targets: list[Path] = []
    exts = {".umod", ".umdl", ".mod"}
    for a in args:
        p = Path(a)
        if not p.exists():
            print(f"[address-scan] skip: {p} does not exist", file=sys.stderr)
            continue
        if p.is_file():
            if p.suffix in exts:
                targets.append(p)
        else:
            for f in p.rglob("*"):
                if f.is_file() and f.suffix in exts:
                    targets.append(f)
    return sorted(targets)


def main(argv: list[str]) -> int:
    targets = collect_targets(argv[1:])
    if not targets:
        print("[address-scan] no .umod/.umdl/.mod files found in scope.")
        return 0

    print(f"[address-scan] scanning {len(targets)} file(s)...")
    total_hits = 0
    for path in targets:
        hits = scan_file(path)
        if not hits:
            continue
        total_hits += len(hits)
        print(f"\n{path}")
        for off, val, cls in hits:
            print(f"  offset 0x{off:08X}  value 0x{val:016X}  → {cls}")

    if total_hits == 0:
        print(f"[address-scan] PASS — no pointer-like values found in {len(targets)} file(s).")
        return 0

    print(
        f"\n[address-scan] FAIL — {total_hits} pointer-like value(s) found "
        f"across {len(targets)} file(s).\n"
        f"Most likely cause: a host-side serializer derived "
        f"`Serialize`/`Deserialize` on a runtime type "
        f"(e.g. NodeRuntime, GraphRuntime). Confirm builders write\n"
        f"`Umod*Descriptor` / `Umdl*Descriptor` types only.",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
