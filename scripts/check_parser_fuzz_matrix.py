#!/usr/bin/env python3
"""Run deterministic malformed persistent-artifact corpus gates."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
UMOD_CORPUS = ROOT / "tests/fuzz_corpus/umod"
UMDL_CORPUS = ROOT / "tests/fuzz_corpus/umdl"


def expected_line(path: Path) -> str:
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.startswith("expected:"):
            return line.split(":", 1)[1].strip()
    raise ValueError(f"{path}: missing expected line")


def validate_metadata() -> list[str]:
    failures: list[str] = []

    for artifact in sorted(UMOD_CORPUS.glob("*/*.bin")):
        metadata = artifact.with_suffix(".txt")
        if not metadata.exists():
            failures.append(f"{artifact}: missing sibling .txt")
            continue
        expected = expected_line(metadata)
        if expected == "unspecified" or not expected.startswith("GraphLoadError::"):
            failures.append(f"{metadata}: invalid expected {expected!r}")

    for artifact in sorted(UMDL_CORPUS.glob("*.umdl")):
        metadata = artifact.with_suffix(".txt")
        if not metadata.exists():
            failures.append(f"{artifact}: missing sibling .txt")
            continue
        expected = expected_line(metadata)
        if expected == "unspecified" or not expected.startswith("UmdlLoadError::"):
            failures.append(f"{metadata}: invalid expected {expected!r}")

    return failures


def run_case(name: str, command: list[str]) -> None:
    print(f"[parser-fuzz-matrix] RUN {name}: {' '.join(command)}", flush=True)
    subprocess.run(command, cwd=ROOT, check=True)
    print(f"[parser-fuzz-matrix] PASS {name}", flush=True)


def main() -> int:
    failures = validate_metadata()
    if failures:
        print("[parser-fuzz-matrix] FAIL metadata", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    try:
        run_case(
            "UMOD malformed corpus",
            [
                "cargo",
                "test",
                "-p",
                "graph",
                "malformed_corpus_fixtures_reject_with_declared_errors",
            ],
        )
        run_case(
            "UMDL malformed corpus",
            [
                "cargo",
                "test",
                "-p",
                "umdl",
                "malformed_corpus_fixtures_reject_with_declared_errors",
            ],
        )
    except subprocess.CalledProcessError as exc:
        print(
            f"[parser-fuzz-matrix] FAIL: {' '.join(exc.cmd)} exited {exc.returncode}",
            file=sys.stderr,
        )
        return exc.returncode

    print("[parser-fuzz-matrix] PASS: UMOD and UMDL malformed corpus gates", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
