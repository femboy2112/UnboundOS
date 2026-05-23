#!/usr/bin/env python3
"""Source-level M11 assistant explanation smoke check."""

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
    assistant = read("crates/llm/src/assistant.rs")
    graph = read("crates/graph/src/lib.rs")
    ssod = read("kernel/src/ssod.rs")
    makefile = read("Makefile")
    verify = read("scripts/verify.py")
    gates = read("scripts/gates.sh")

    for surface in (
        "pub struct GraphExplanationInput",
        "pub fn explain_graph",
        "graph_explanation_formats_display_snapshot_fields",
        "pub struct GraphExplanationSnapshot",
        "pub const fn explanation_snapshot",
        "explanation_snapshot_is_copied_from_display_state",
    ):
        haystack = assistant if "GraphExplanationInput" in surface or "explain_graph" in surface or "graph_explanation" in surface else graph
        require(haystack, surface, "graph explanation evidence", failures)

    for surface in (
        "pub struct SsodExplanationInput",
        "pub fn explain_ssod",
        "ssod_explanation_formats_structured_diagnostic_fields",
        "pub struct SsodExplanationSnapshot",
        "pub fn from_diagnostic",
        "pub enum SsodFaultFamily",
    ):
        haystack = assistant if "SsodExplanationInput" in surface or "explain_ssod" in surface or "ssod_explanation" in surface else ssod
        require(haystack, surface, "SSOD explanation evidence", failures)

    for surface in (
        "pub struct StructuredActionBuffer",
        "pub fn push",
        "action_buffer_uses_caller_provided_storage",
        "action_buffer_reports_overflow_and_rejects_unknown_kind",
    ):
        require(assistant, surface, "action-buffer evidence", failures)

    for surface in (
        "pub enum AssistantExplainRequest",
        "pub struct AssistantExplainResponse",
        "pub fn assistant_explain",
        "AssistantActionError::UnsupportedRequest",
        "AssistantActionError::ActionBufferRequired",
        "assistant_explain_routes_graph_requests_without_actions",
        "assistant_explain_routes_ssod_requests_and_buffers_actions",
        "assistant_explain_rejects_unsupported_requests",
    ):
        require(assistant, surface, "unified assistant surface evidence", failures)

    for forbidden in (
        "GraphRuntime",
        "graph_compile_verified",
        "graph_load_from_umod",
        "unsafe {",
        "unsafe fn",
        "std::thread",
        "spawn(",
        "eval(",
    ):
        forbid(assistant, forbidden, "assistant no-direct-mutation boundary", failures)

    require(makefile, "assistant-smoke", "assistant smoke make target", failures)
    require(makefile, "scripts/check_assistant_smoke.py", "assistant smoke make target", failures)
    require(
        verify,
        "scripts/check_assistant_smoke.py",
        "assistant smoke mission verification",
        failures,
    )
    require(gates, "assistant-smoke", "assistant smoke aggregate gates", failures)
    require(
        gates,
        "scripts/check_assistant_smoke.py",
        "assistant smoke aggregate gates",
        failures,
    )

    if failures:
        print("[assistant-smoke] FAIL")
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print("[assistant-smoke] PASS: assistant explanation evidence reachable")
    return 0


if __name__ == "__main__":
    sys.exit(main())
