#!/usr/bin/env python3
"""Source-level M9 UMDL loader smoke check."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(text: str, needle: str, label: str, failures: list[str]) -> None:
    if needle not in text:
        failures.append(f"{label}: missing {needle!r}")


def main() -> int:
    failures: list[str] = []
    umdl = read("crates/umdl/src/lib.rs")
    fixture = read("scripts/make_umdl_fixture.py")
    malformed = ROOT / "tests/fuzz_corpus/umdl/bad-magic.umdl"

    for surface in (
        "pub fn parse(bytes: &[u8])",
        "pub fn validate_sections",
        "pub fn parse_umdl",
        "pub fn validate_tensor_descriptors",
        "pub fn load_model_view",
        "UmdlArenaReservations",
        "LoadedUmdlModel",
        "HeaderChecksumMismatch",
        "TokenizerSectionChecksumMismatch",
        "TensorSectionChecksumMismatch",
        "WeightBlobChecksumMismatch",
        "loads_read_only_model_view_and_arena_reservations",
        "load_model_view_rejects_simd_and_profile_budget_mismatch",
    ):
        require(umdl, surface, "UMDL loader evidence", failures)

    for fixture_evidence in (
        "def build_valid()",
        "def build_bad_magic()",
        "UMDL_MAGIC = b\"UMDL\"",
        "header_checksum(valid)",
        "checksum64(valid[160:232])",
    ):
        require(fixture, fixture_evidence, "deterministic fixture generator", failures)

    for forbidden in ("unsafe {", "unsafe fn"):
        if forbidden in umdl:
            failures.append(f"UMDL loader boundary: unexpected {forbidden!r}")
    if not malformed.exists():
        failures.append("malformed corpus: missing tests/fuzz_corpus/umdl/bad-magic.umdl")

    proc = subprocess.run(
        ["python3", "scripts/make_umdl_fixture.py", "--check"],
        cwd=ROOT,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        failures.append("deterministic fixture generator: --check failed")

    if failures:
        print("[umdl-smoke] FAIL")
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print("[umdl-smoke] PASS: UMDL loader and fixture evidence reachable")
    return 0


if __name__ == "__main__":
    sys.exit(main())
