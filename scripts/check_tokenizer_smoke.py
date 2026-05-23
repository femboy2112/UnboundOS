#!/usr/bin/env python3
"""Source-level M7 tokenizer smoke check."""

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
    tokenizer = read("crates/llm/src/tokenizer.rs")
    umdl = read("crates/umdl/src/lib.rs")

    require(umdl, "pub const M7_SUPPORTED_TOKENIZER", "supported tokenizer constant", failures)
    require(umdl, "TokenizerType::RawByteToToken", "raw-byte tokenizer support", failures)
    require(umdl, "UnsupportedTokenizerType", "unsupported tokenizer error", failures)

    for forbidden in ("ByteFallbackBpe", "SentencepieceUnigram"):
        require(tokenizer, f"!is_supported(TokenizerType::{forbidden})", "single-family support test", failures)

    for surface in (
        "pub fn encode_raw_bytes",
        "pub fn decode_raw_bytes",
        "pub fn round_trip_raw_bytes",
        "OutputOverflow",
        "InvalidTokenId",
        "InvalidUtf8",
        "representative_prompts_round_trip",
    ):
        require(tokenizer, surface, "tokenizer encode/decode evidence", failures)

    if failures:
        print("[tokenizer-smoke] FAIL")
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print("[tokenizer-smoke] PASS: raw-byte tokenizer round-trip evidence reachable")
    return 0


if __name__ == "__main__":
    sys.exit(main())
