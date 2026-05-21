#!/usr/bin/env python3
"""Run the verification bundle for the current UnboundOS mission."""

from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CARGO_BIN = Path.home() / ".cargo" / "bin"


@dataclass(frozen=True)
class Command:
    name: str
    args: list[str]
    requires: tuple[str, ...] = ()
    optional: bool = False


STATIC_COMMANDS = [
    Command("mission state", ["python3", "scripts/mission.py", "validate"]),
    Command("no_std hygiene", ["bash", "scripts/check_no_std.sh"]),
    Command("no eval node", ["bash", "scripts/check_no_eval_node.sh"]),
    Command("no pointer serde derive", ["bash", "scripts/check_no_pointer_derive.sh"]),
    Command("no POSIX path leakage", ["bash", "scripts/check_no_posix_paths.sh"]),
    Command("dispatch boundary", ["bash", "scripts/check_dispatch_boundary.sh"]),
    Command(
        "address scan",
        ["python3", "scripts/address_scan.py", "tests/golden_graphs", "tests/golden_models"],
    ),
]

RUST_COMMANDS = [
    Command("cargo fmt", ["cargo", "fmt", "--check"], requires=("cargo",), optional=True),
    Command(
        "cargo clippy",
        ["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"],
        requires=("cargo",),
        optional=True,
    ),
    Command(
        "host tests",
        ["cargo", "test", "--workspace", "--exclude", "kernel"],
        requires=("cargo",),
        optional=True,
    ),
]


def available(command: Command) -> tuple[bool, str]:
    for tool in command.requires:
        if resolve_tool(tool) is None:
            return False, f"{tool} missing"
    if resolve_tool(command.args[0]) is None:
        return False, f"{command.args[0]} missing"
    return True, ""


def resolve_tool(name: str) -> str | None:
    found = shutil.which(name)
    if found:
        return found
    fallback = CARGO_BIN / name
    if fallback.exists():
        return str(fallback)
    return None


def run(command: Command) -> int:
    args = command.args.copy()
    resolved = resolve_tool(args[0])
    if resolved is not None:
        args[0] = resolved
    print(f"[verify] RUN {command.name}: {' '.join(command.args)}")
    proc = subprocess.run(args, cwd=ROOT, text=True, check=False)
    if proc.returncode != 0:
        print(f"[verify] FAIL {command.name}", file=sys.stderr)
    else:
        print(f"[verify] PASS {command.name}")
    return proc.returncode


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mission", default="current", choices=("current",))
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument(
        "--strict-rust",
        action="store_true",
        help="alias for the default behavior: Rust checks must run and pass",
    )
    parser.add_argument(
        "--allow-missing-rust",
        action="store_true",
        help=(
            "permit missing Rust tooling for control-plane/doc-only missions; "
            "prints a partial verdict and never claims Rust verification"
        ),
    )
    args = parser.parse_args(argv[1:])

    commands = STATIC_COMMANDS + RUST_COMMANDS
    failed = 0
    skipped: list[str] = []

    for command in commands:
        ok, reason = available(command)
        if not ok:
            line = f"[verify] SKIP {command.name}: {reason}"
            if command.optional and args.allow_missing_rust:
                print(line)
                skipped.append(f"{command.name}: {reason}")
                continue
            print(line, file=sys.stderr)
            failed = 1
            continue

        if args.dry_run:
            print(f"[verify] DRY {command.name}: {' '.join(command.args)}")
            continue

        code = run(command)
        if code != 0:
            if command.optional and args.allow_missing_rust:
                skipped.append(f"{command.name}: failed with {code}")
            else:
                failed = 1

    if skipped:
        print("[verify] optional blockers:")
        for item in skipped:
            print(f"  {item}")
        print("[verify] Rust-dependent verification was not claimed.")
        if not args.allow_missing_rust:
            failed = 1

    if failed:
        print("[verify] verdict: BLOCK", file=sys.stderr)
        return 1
    if skipped:
        print("[verify] verdict: PROCEED_STATIC_ONLY")
        return 0
    print("[verify] verdict: PROCEED")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
