#!/usr/bin/env python3
"""Source-level M12 local retrieval smoke check."""

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
    retrieval = read("crates/llm/src/retrieval.rs")
    assistant = read("crates/llm/src/assistant.rs")
    makefile = read("Makefile")
    verify = read("scripts/verify.py")
    gates = read("scripts/gates.sh")

    require(lib, "pub mod retrieval;", "LLM retrieval module surface", failures)

    for surface in (
        "pub struct RetrievalQuery",
        "pub struct RetrievalDocumentRef",
        "pub struct RetrievalResult",
        "pub struct RetrievalResultBuffer",
        "retrieval_query_is_fixed_width_data",
        "document_ref_accepts_opaque_ids_and_rejects_path_shapes",
        "retrieval_result_buffer_uses_caller_storage",
    ):
        require(retrieval, surface, "retrieval contract evidence", failures)

    for surface in (
        "pub struct RetrievalIndexSnapshot",
        "index_snapshot_is_read_only_view_over_caller_documents",
        "index_snapshot_rejects_empty_duplicate_and_invalid_refs",
    ):
        require(retrieval, surface, "retrieval index evidence", failures)

    for surface in (
        "pub fn retrieve_top_k",
        "retrieve_top_k_ranks_matches_deterministically",
        "retrieve_top_k_reports_overflow_and_unsupported_query",
    ):
        require(retrieval, surface, "retrieval ranking evidence", failures)

    for surface in (
        "pub fn pack_retrieval_context",
        "pack_retrieval_context_preserves_ids_and_boundaries",
        "pack_retrieval_context_rejects_overflow_and_mismatched_results",
    ):
        require(retrieval, surface, "retrieval context-packing evidence", failures)

    for surface in (
        "pub struct AssistantRetrievalRequest",
        "pub struct AssistantRetrievalResponse",
        "pub fn assistant_retrieve_context",
        "AssistantRetrievalError::Action",
        "assistant_retrieve_context_packs_explanatory_context",
        "assistant_retrieve_context_buffers_optional_actions",
        "assistant_retrieve_context_requires_action_buffer_and_maps_retrieval_errors",
    ):
        require(assistant, surface, "assistant retrieval evidence", failures)

    for text, label in (
        (retrieval, "retrieval boundary"),
        (assistant, "assistant retrieval boundary"),
    ):
        for forbidden in (
            "unsafe {",
            "unsafe fn",
            "std::fs",
            "File::",
            "std::thread",
            "spawn(",
            "eval(",
            "GraphRuntime",
            "graph_compile_verified",
            "graph_load_from_umod",
        ):
            forbid(text, forbidden, label, failures)

    for forbidden in ('"local://', '"/', '"../', '"~/', '"\\\\'):
        forbid(retrieval, forbidden, "retrieval host-path boundary", failures)

    require(makefile, "retrieval-smoke", "retrieval smoke make target", failures)
    require(
        makefile,
        "scripts/check_retrieval_smoke.py",
        "retrieval smoke make target",
        failures,
    )
    require(
        verify,
        "scripts/check_retrieval_smoke.py",
        "retrieval smoke mission verification",
        failures,
    )
    require(gates, "retrieval-smoke", "retrieval smoke aggregate gates", failures)
    require(
        gates,
        "scripts/check_retrieval_smoke.py",
        "retrieval smoke aggregate gates",
        failures,
    )

    if failures:
        print("[retrieval-smoke] FAIL")
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print("[retrieval-smoke] PASS: local retrieval evidence reachable")
    return 0


if __name__ == "__main__":
    sys.exit(main())
