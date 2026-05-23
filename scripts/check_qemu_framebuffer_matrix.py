#!/usr/bin/env python3
"""Boot QEMU framebuffer rendering across emulated VGA adapters."""

from __future__ import annotations

import os
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


@dataclass(frozen=True)
class Profile:
    name: str
    video: str
    expected: str


PROFILES = (
    Profile("std-vga-render", "std", "rendered"),
    Profile("cirrus-vga-fallback", "cirrus", "unavailable"),
    Profile("virtio-vga-render", "virtio", "rendered"),
)


def run_profile(profile: Profile) -> None:
    env = {
        **os.environ,
        "QEMU_VIDEO": profile.video,
        "QEMU_EXPECT_FRAMEBUFFER": profile.expected,
    }
    command = ["make", "-s", "qemu-framebuffer-smoke"]
    print(
        f"[qemu-framebuffer-matrix] RUN {profile.name}: "
        f"QEMU_VIDEO={profile.video} QEMU_EXPECT_FRAMEBUFFER={profile.expected} "
        f"{' '.join(command)}",
        flush=True,
    )
    subprocess.run(command, cwd=ROOT, env=env, check=True)
    print(f"[qemu-framebuffer-matrix] PASS {profile.name}", flush=True)


def main() -> int:
    try:
        for profile in PROFILES:
            run_profile(profile)
    except subprocess.CalledProcessError as exc:
        print(
            f"[qemu-framebuffer-matrix] FAIL: {' '.join(exc.cmd)} exited {exc.returncode}",
            file=sys.stderr,
        )
        return exc.returncode

    print(f"[qemu-framebuffer-matrix] PASS: {len(PROFILES)} VGA profiles", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
