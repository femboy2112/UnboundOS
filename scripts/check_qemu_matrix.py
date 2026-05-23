#!/usr/bin/env python3
"""Exercise live QEMU gates across CPU and RAM profiles."""

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
    cpu: str
    ram: str


PROFILES = (
    Profile("low-ram-qemu64", "qemu64", "256M"),
    Profile("baseline-qemu64", "qemu64", "512M"),
    Profile("large-ram-qemu64", "qemu64", "768M"),
    Profile("max-cpu", "max", "512M"),
)

TARGETS = (
    ("heartbeat", ("make", "-s", "qemu-headless")),
    ("arena/memory", ("make", "-s", "qemu-m2-dump")),
    ("graph boot", ("make", "-s", "qemu-graph-boot")),
    ("framebuffer", ("make", "-s", "qemu-framebuffer-smoke")),
    ("interactive shell", ("make", "-s", "qemu-interactive-smoke")),
)


def run_profile(profile: Profile) -> None:
    env = os.environ.copy()
    env["QEMU_CPU"] = profile.cpu
    env["QEMU_RAM"] = profile.ram
    print(f"[qemu-matrix] profile={profile.name} cpu={profile.cpu} ram={profile.ram}")
    for label, command in TARGETS:
        print(f"[qemu-matrix] RUN {profile.name} {label}: {' '.join(command)}")
        subprocess.run(command, cwd=ROOT, env=env, check=True)
        print(f"[qemu-matrix] PASS {profile.name} {label}")


def main() -> int:
    try:
        for profile in PROFILES:
            run_profile(profile)
    except subprocess.CalledProcessError as exc:
        print(
            f"[qemu-matrix] FAIL: {' '.join(exc.cmd)} exited {exc.returncode}",
            file=sys.stderr,
        )
        return exc.returncode

    print(f"[qemu-matrix] PASS: {len(PROFILES)} profiles x {len(TARGETS)} live gates")
    return 0


if __name__ == "__main__":
    sys.exit(main())
