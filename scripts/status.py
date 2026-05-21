#!/usr/bin/env python3
"""Report UnboundOS mission, git, and toolchain status."""

from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CARGO_BIN = Path.home() / ".cargo" / "bin"


def run(args: list[str]) -> tuple[int, str]:
    proc = subprocess.run(
        args,
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    return proc.returncode, proc.stdout.strip()


def first_value(path: Path, key: str) -> str:
    if not path.exists():
        return "missing"
    prefix = f"{key}:"
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith(prefix):
            return line[len(prefix) :].strip()
    return "unknown"


def tool_status(name: str) -> str:
    found = shutil.which(name)
    if found:
        return found
    fallback = CARGO_BIN / name
    return str(fallback) if fallback.exists() else "missing"


def main() -> int:
    campaign = ROOT / ".codex" / "CURRENT_CAMPAIGN.md"
    mission = ROOT / ".codex" / "CURRENT_MISSION.md"

    print("=== UnboundOS Status ===")
    print(f"root: {ROOT}")
    print(f"campaign: {first_value(campaign, 'Campaign')}")
    print(f"active_mission: {first_value(campaign, 'Active mission')}")
    print(f"mission: {first_value(mission, 'Mission')}")
    print(f"mission_status: {first_value(mission, 'Status')}")

    code, branch = run(["git", "branch", "--show-current"])
    print(f"branch: {branch if code == 0 and branch else 'unknown'}")

    code, status = run(["git", "status", "--short"])
    print("git_status:")
    if status:
        for line in status.splitlines():
            print(f"  {line}")
    else:
        print("  clean")

    print("tools:")
    for name in ("python3", "git", "cargo", "rustup", "qemu-system-x86_64", "pdftotext"):
        print(f"  {name}: {tool_status(name)}")

    blockers: list[str] = []
    for required in (
        ROOT / "CLAUDE.md",
        ROOT / "docs" / "UnboundOS_Tech_Spec_v2_1_1_Fidelity_Hardening.pdf",
        ROOT / ".codex" / "CURRENT_CAMPAIGN.md",
        ROOT / ".codex" / "CURRENT_MISSION.md",
        ROOT / ".codex" / "PROJECT_PLAN.md",
    ):
        if not required.exists():
            blockers.append(f"missing {required.relative_to(ROOT)}")

    if tool_status("cargo") == "missing":
        blockers.append("cargo missing: Rust fmt/clippy/tests/kernel build cannot run")
    if tool_status("rustup") == "missing":
        blockers.append("rustup missing: pinned toolchain component checks cannot run")

    print("blockers:")
    if blockers:
        for item in blockers:
            print(f"  {item}")
    else:
        print("  none")

    return 0


if __name__ == "__main__":
    sys.exit(main())
