#!/usr/bin/env python3
"""
hook_post_address_scan.py — PostToolUse warning for the §6.10 /
§14.1 persistent-pointer rule.

After an Edit / Write / MultiEdit that touches one of the watched
prefixes, run scripts/address_scan.py against the release-track
fixture roots and surface its output as a warning when anything
flags. Never blocks — it is a reminder. The /fidelity-check skill
runs the same scanner authoritatively.

Watched prefixes (per README-claude.md):
  crates/umod/      — UMOD format crate; emit code may grow leakage
  crates/umdl/      — UMDL format crate; same
  tests/golden_graphs/   — release fixtures
  tests/golden_models/   — release fixtures
  tests/fuzz_corpus/     — fuzz fixtures (some are intentionally suspicious)
"""

from __future__ import annotations

import json
import os
import subprocess
import sys

WATCHED = (
    "crates/umod/",
    "crates/umdl/",
    "tests/golden_graphs/",
    "tests/golden_models/",
    "tests/fuzz_corpus/",
)


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except json.JSONDecodeError:
        return 0

    tool_name = payload.get("tool_name") or ""
    if tool_name not in ("Edit", "Write", "MultiEdit"):
        return 0

    inp = payload.get("tool_input") or {}
    file_path = str(inp.get("file_path") or "")
    cwd = payload.get("cwd") or os.getcwd()
    try:
        rel = os.path.relpath(file_path, cwd) if file_path else ""
    except ValueError:
        rel = file_path
    rel = rel.replace(os.sep, "/")

    if not any(rel.startswith(p) for p in WATCHED):
        return 0

    scan = subprocess.run(
        [
            "python3",
            "scripts/address_scan.py",
            "tests/golden_graphs",
            "tests/golden_models",
        ],
        cwd=cwd,
        capture_output=True,
        text=True,
        check=False,
    )
    if scan.returncode == 0:
        return 0

    sys.stderr.write(
        "[address-scan] WARNING after edit to "
        f"{rel}\n"
        "Persistent-pointer scan flagged at least one byte sequence "
        "in tests/golden_graphs or tests/golden_models. Spec §6.10 "
        "and §14.1 forbid raw runtime addresses in persistent files.\n"
        "----- scanner output -----\n"
    )
    sys.stderr.write(scan.stdout)
    sys.stderr.write(scan.stderr)
    sys.stderr.write("--------------------------\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
