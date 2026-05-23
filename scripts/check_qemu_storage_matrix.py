#!/usr/bin/env python3
"""Boot QEMU storage smoke paths for success and failure cases."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
GOOD_FIXTURE = Path("/tmp/unboundos-storage-sector0.bin")
BAD_FIXTURE = Path("/tmp/unboundos-storage-sector0-bad.bin")
STORAGE_IMAGE = Path("/tmp/unboundos-storage-matrix.img")
SECTOR_BYTES = 512


def write_bad_fixture() -> None:
    sector = bytearray(SECTOR_BYTES)
    sector[: len(b"UNBOUNDOS_BAD_SECTOR")] = b"UNBOUNDOS_BAD_SECTOR"
    BAD_FIXTURE.write_bytes(sector)
    print(f"[storage-matrix] wrote {BAD_FIXTURE} marker=UNBOUNDOS_BAD_SECTOR", flush=True)


def run_case(name: str, args: list[str]) -> None:
    print(f"[storage-matrix] RUN {name}: {' '.join(args)}", flush=True)
    subprocess.run(args, cwd=ROOT, check=True)
    print(f"[storage-matrix] PASS {name}", flush=True)


def main() -> int:
    subprocess.run(
        ["python3", "scripts/make_storage_fixture.py", str(GOOD_FIXTURE)],
        cwd=ROOT,
        check=True,
    )
    write_bad_fixture()

    env = {"UNBOUNDOS_STORAGE_SMOKE": "1"}
    subprocess.run(
        ["make", "-s", "image", f"IMAGE={STORAGE_IMAGE}"],
        cwd=ROOT,
        env={**os.environ, **env},
        check=True,
    )

    try:
        run_case(
            "marker-ok",
            [
                "./scripts/qemu.sh",
                "--headless",
                "--image",
                str(STORAGE_IMAGE),
                "--storage-image",
                str(GOOD_FIXTURE),
                "--assert-storage-marker",
            ],
        )
        run_case(
            "marker-mismatch",
            [
                "./scripts/qemu.sh",
                "--headless",
                "--image",
                str(STORAGE_IMAGE),
                "--storage-image",
                str(BAD_FIXTURE),
                "--assert-storage-mismatch",
            ],
        )
        run_case(
            "read-error-no-primary-disk",
            [
                "./scripts/qemu.sh",
                "--headless",
                "--image",
                str(STORAGE_IMAGE),
                "--assert-storage-error",
            ],
        )
    except subprocess.CalledProcessError as exc:
        print(
            f"[storage-matrix] FAIL: {' '.join(exc.cmd)} exited {exc.returncode}",
            file=sys.stderr,
        )
        return exc.returncode

    print("[storage-matrix] PASS: marker, mismatch, and read-error cases", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
