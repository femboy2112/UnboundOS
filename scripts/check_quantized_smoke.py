#!/usr/bin/env python3
"""Source-level M10 quantized inference smoke check."""

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
    lib = read("crates/llm/src/lib.rs")
    dispatch = read("crates/llm/src/dispatch.rs")
    scalar = read("crates/llm/src/kernels/scalar.rs")
    quantized = read("crates/llm/src/quantized.rs")

    for surface in (
        "pub mod kernels;",
        "pub mod quantized;",
        "ProjectI8Kernel",
    ):
        require(lib, surface, "LLM quantized module surface", failures)

    for surface in (
        "pub fn project_i8_i8_i32",
        "ScalarKernelError",
        "scalar_i8_projection_is_deterministic",
        "scalar_i8_projection_uses_caller_output_buffer",
    ):
        require(scalar, surface, "scalar quantized kernel evidence", failures)

    for surface in (
        "project_i8_i8_i32: kernels::scalar::project_i8_i8_i32",
        "dispatch_table_routes_quantized_projection_through_scalar_kernel",
    ):
        require(dispatch, surface, "dispatch-table quantized evidence", failures)

    for surface in (
        "pub fn next_token_step",
        "pub fn stream_tokens",
        "QuantizedStepBuffers",
        "QuantizedStreamBuffers",
        "QuantizedStreamState",
        "quantized_next_token_step_is_deterministic",
        "quantized_stream_produces_stable_token_sequence",
        "assert_eq!(output, [67, 68, 69]);",
    ):
        require(quantized, surface, "quantized inference evidence", failures)

    for text, label in (
        (scalar, "scalar kernel boundary"),
        (quantized, "quantized inference boundary"),
    ):
        for forbidden in ("unsafe {", "unsafe fn", "matvec_q4_avx2", "matvec_q4_sse2"):
            forbid(text, forbidden, label, failures)

    if failures:
        print("[quantized-smoke] FAIL")
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print("[quantized-smoke] PASS: quantized inference evidence reachable")
    return 0


if __name__ == "__main__":
    sys.exit(main())
