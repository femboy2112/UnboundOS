#!/usr/bin/env python3
"""
hook_pre_graph_runtime.py — PreToolUse guard for the §1.9 / §5.7
single verifier gate.

Receives the standard Claude Code hook payload on stdin (Edit / Write
/ MultiEdit). Inspects the new file content. If it contains a direct
`GraphRuntime { ... }` struct literal or a `GraphRuntime::new` /
`GraphRuntime::from` constructor anywhere except
`crates/graph/src/loader.rs`, exits with code 2 and a structured
explanation. Exit 0 otherwise.

This mechanically enforces CLAUDE.md §2 H2 and spec §5.7. The rule
exists because every legal path to a runtime graph runs through
`graph_load_from_umod` → verifier → `graph_compile_verified`, and
those bind the only legal `GraphRuntime { ... }` construction site
inside the graph crate's loader module.
"""

from __future__ import annotations

import json
import os
import re
import sys

ALLOWED_PATH = "crates/graph/src/loader.rs"
PATTERN = re.compile(r"\bGraphRuntime\s*\{|\bGraphRuntime::(new|from)\b")


def collect_new_content(payload: dict) -> str:
    """Concatenate every chunk of new content produced by the tool."""
    inp = payload.get("tool_input") or {}
    parts: list[str] = []
    if "new_string" in inp:
        parts.append(str(inp.get("new_string") or ""))
    if "content" in inp:
        parts.append(str(inp.get("content") or ""))
    for edit in inp.get("edits") or []:
        parts.append(str((edit or {}).get("new_string") or ""))
    return "\n".join(parts)


def relpath(file_path: str, cwd: str) -> str:
    if not file_path:
        return ""
    try:
        return os.path.relpath(file_path, cwd) if cwd else file_path
    except ValueError:
        return file_path


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except json.JSONDecodeError:
        # Malformed input is not our problem to police; allow.
        return 0

    tool_name = payload.get("tool_name") or ""
    if tool_name not in ("Edit", "Write", "MultiEdit"):
        return 0

    inp = payload.get("tool_input") or {}
    file_path = str(inp.get("file_path") or "")
    cwd = payload.get("cwd") or os.getcwd()
    rel = relpath(file_path, cwd)

    # Only Rust source files are in scope. UMOD and UMDL fixtures are
    # binary; markdown can mention the type freely.
    if not rel.endswith(".rs"):
        return 0

    # The single legal site.
    if rel.replace(os.sep, "/") == ALLOWED_PATH:
        return 0

    new = collect_new_content(payload)
    match = PATTERN.search(new)
    if not match:
        return 0

    sys.stderr.write(
        "[graph-runtime-gate] BLOCKED: "
        f"`{match.group(0)}` in {rel}\n"
        "Direct GraphRuntime construction is allowed only inside "
        f"{ALLOWED_PATH} (CLAUDE.md §2 H2; spec §5.7).\n"
        "The legal path is: bytes → graph_load_from_umod → verifier "
        "→ graph_compile_verified → GraphRuntimeHandle.\n"
    )
    return 2


if __name__ == "__main__":
    sys.exit(main())
