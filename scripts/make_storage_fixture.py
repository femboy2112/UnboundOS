#!/usr/bin/env python3
"""Create the deterministic M6 raw-sector smoke fixture."""

from __future__ import annotations

import sys
from pathlib import Path


MARKER = b"UNBOUNDOS_M6_SECTOR0"
SECTOR_BYTES = 512


def main() -> int:
    if len(sys.argv) != 2:
        print("Usage: make_storage_fixture.py <output-raw-sector>", file=sys.stderr)
        return 2

    out = Path(sys.argv[1])
    sector = bytearray(SECTOR_BYTES)
    sector[: len(MARKER)] = MARKER
    out.write_bytes(sector)
    print(f"[storage-fixture] wrote {out} marker={MARKER.decode('ascii')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
