#!/usr/bin/env python3
"""Source-level guard for the M6 storage resource namespace boundary."""

from __future__ import annotations

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
    umod = read("crates/umod/src/lib.rs")
    verifier = read("crates/graph/src/verifier.rs")
    corpus_note = read("tests/fuzz_corpus/umod/unknown_resource_syntax/path-shaped-resource-ref.txt")

    for accepted in (
        "graph:boot_graph.v1",
        "index:sector0_marker",
        "blob:raw-sector-00000000",
        "font:boot-font-8x16",
        "profile:qemu-storage-smoke",
    ):
        require(umod, accepted, "opaque resource acceptance", failures)

    for rejected in (
        "local://models/tiny.umdl",
        "/etc/models/tiny.umdl",
        r"C:\models\tiny.umdl",
        "model:../tiny",
        r"model:dir\tiny",
        "blob:fat32/sector0",
        "index:~profile",
    ):
        require(umod, rejected, "path-shaped resource rejection", failures)

    for guard in (
        "fn looks_like_path",
        "bytes.starts_with(b\"local://\")",
        "matches!(*byte, b'/' | b'\\\\' | b'~')",
        "ResourceRefError::LooksLikeAPath",
    ):
        require(umod, guard, "resource parser path guard", failures)

    require(verifier, "parse_resource_ref", "graph verifier resource delegation", failures)
    require(
        verifier,
        "path-shaped-resource-ref.bin",
        "graph verifier malformed resource fixture",
        failures,
    )
    require(corpus_note, "path-shaped opaque ref", "malformed corpus note", failures)

    if failures:
        print("[storage-namespace] FAIL")
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print("[storage-namespace] PASS: graph-visible storage refs stay opaque")
    return 0


if __name__ == "__main__":
    sys.exit(main())
