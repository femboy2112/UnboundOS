#!/usr/bin/env python3
"""Source-level M5 UI smoke check.

This is intentionally graphical-hardware-free: it proves the minimal UI
surfaces are wired into source and regular host/kernel verification can run
without requiring a framebuffer-capable QEMU display.
"""

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
    framebuffer = read("kernel/src/framebuffer.rs")
    graph_lib = read("crates/graph/src/lib.rs")
    graph_loader = read("crates/graph/src/loader.rs")
    qemu = read("scripts/qemu.sh")

    for label in (
        "GRAPH_ID=",
        "NODES=",
        "WIRES=",
        "ACTIVE_NODE=",
        "LAST_COMPLETED_NODE=",
    ):
        require(framebuffer, label, "framebuffer graph-state render", failures)
    require(framebuffer, "pub fn render_graph_state", "framebuffer graph-state render", failures)
    require(framebuffer, "pub fn write_hex_u64", "framebuffer graph-id render", failures)
    require(framebuffer, "pub fn write_dec_u32", "framebuffer count render", failures)

    for getter in (
        "pub const fn graph_id(&self)",
        "pub const fn node_count(&self)",
        "pub const fn wire_count(&self)",
        "pub const fn active_node(&self)",
        "pub const fn last_completed_node(&self)",
    ):
        require(graph_lib, getter, "graph display snapshot", failures)
    require(graph_lib, "pub(crate) const fn new(", "graph display constructor boundary", failures)
    require(graph_loader, "compiled_handle_exposes_read_only_display_state", "graph display test", failures)

    if graph_lib.count("pub fn graph_compile_verified") != 1:
        failures.append("graph compile gate: expected exactly one public compile function")
    if "struct GraphRuntime {" not in graph_loader:
        failures.append("graph runtime boundary: runtime type not private to loader.rs")
    if "no-serial boot reached debug-exit BOOT_OK" not in qemu:
        failures.append("no-serial smoke: debug-exit assertion missing")

    if failures:
        print("[ui-smoke] FAIL")
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print("[ui-smoke] PASS: framebuffer and graph-state display evidence reachable")
    return 0


if __name__ == "__main__":
    sys.exit(main())
