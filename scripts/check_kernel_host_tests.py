#!/usr/bin/env python3
"""Run host-side unit tests for standalone kernel modules."""

from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CARGO_BIN = Path.home() / ".cargo" / "bin"
TESTS: tuple[tuple[str, Path], ...] = (
    ("arena", ROOT / "kernel" / "src" / "arena.rs"),
    ("cpu", ROOT / "kernel" / "src" / "cpu.rs"),
    ("multiboot2", ROOT / "kernel" / "src" / "multiboot2.rs"),
)


def resolve_tool(name: str) -> str | None:
    found = shutil.which(name)
    if found:
        return found
    fallback = CARGO_BIN / name
    if fallback.exists():
        return str(fallback)
    return None


def run_one(rustc: str, name: str, source: Path) -> int:
    wrapper = Path(f"/tmp/unboundos_{name}_host_tests.rs")
    binary = Path(f"/tmp/unboundos_{name}_host_tests")
    wrapper.write_text(
        f'#![allow(dead_code)]\n#[path = "{source}"]\nmod {name};\n',
        encoding="utf-8",
    )

    print(f"[kernel-host-tests] RUN {name}: rustc --test {source.relative_to(ROOT)}")
    compile_proc = subprocess.run(
        [
            rustc,
            "--edition",
            "2021",
            "-D",
            "warnings",
            "--test",
            str(wrapper),
            "-o",
            str(binary),
        ],
        cwd=ROOT,
        text=True,
        check=False,
    )
    if compile_proc.returncode != 0:
        print(f"[kernel-host-tests] FAIL {name} compile", file=sys.stderr)
        return compile_proc.returncode

    run_proc = subprocess.run([str(binary)], cwd=ROOT, text=True, check=False)
    if run_proc.returncode != 0:
        print(f"[kernel-host-tests] FAIL {name}", file=sys.stderr)
        return run_proc.returncode
    print(f"[kernel-host-tests] PASS {name}")
    return 0


def main() -> int:
    rustc = resolve_tool("rustc")
    if rustc is None:
        print("[kernel-host-tests] FAIL: rustc missing", file=sys.stderr)
        return 1

    failed = 0
    for name, source in TESTS:
        if run_one(rustc, name, source) != 0:
            failed = 1
    return failed


if __name__ == "__main__":
    sys.exit(main())
