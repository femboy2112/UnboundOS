#!/usr/bin/env python3
"""Source-level M8 toy transformer smoke check."""

from __future__ import annotations

import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def require(text: str, needle: str, label: str, failures: list[str]) -> None:
    if needle not in text:
        failures.append(f"{label}: missing {needle!r}")


def forbid(text: str, needle: str, label: str, failures: list[str]) -> None:
    if needle in text:
        failures.append(f"{label}: unexpected {needle!r}")


def main() -> int:
    failures: list[str] = []
    toy = read("crates/llm/src/toy_transformer.rs")
    toy_lower = toy.lower()
    llm_lib = read("crates/llm/src/lib.rs")

    for surface in (
        "pub mod toy_transformer;",
        "pub fn generate_tokens",
        "pub fn generate_text",
        "ToyModelMetadata::m8_toy",
        "ToyGenerationConfig::deterministic",
        "TokenizerMetadata::raw_byte_to_token",
        "tokenizer::encode_raw_bytes",
        "tokenizer::decode_raw_bytes",
        "prompt_to_text_generation_is_deterministic",
        "prompt_to_text_uses_caller_provided_buffers",
        "generation_is_deterministic_for_same_prompt_seed_config_and_model",
        "assert_eq!(&first[..first_len], &[108, 32, 85, 110, 107, 65]);",
        'assert_eq!(text, "l UnkA");',
    ):
        haystack = llm_lib if surface == "pub mod toy_transformer;" else toy
        require(haystack, surface, "toy transformer deterministic evidence", failures)

    for caller_buffer in (
        "prompt_tokens: &mut [u32]",
        "generated_tokens: &mut [u32]",
        "output_bytes: &'a mut [u8]",
        "output: &mut [u32]",
    ):
        require(toy, caller_buffer, "caller-provided buffer contract", failures)

    for forbidden in (
        "unsafe",
        "avx2",
        "sse2",
        "matvec_q4_",
        "dispatch::",
    ):
        forbid(toy_lower, forbidden, "M8 toy path backend boundary", failures)

    if failures:
        print("[toy-transformer-smoke] FAIL")
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print("[toy-transformer-smoke] PASS: deterministic toy inference evidence reachable")
    return 0


if __name__ == "__main__":
    sys.exit(main())
