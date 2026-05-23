#!/usr/bin/env python3
"""Boot UnboundOS in QEMU and verify the live serial operator shell."""

from __future__ import annotations

import os
import pty
import select
import signal
import subprocess
import sys
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
IMAGE = Path("/tmp/unboundos.img")
TIMEOUT_SECONDS = 30.0
QEMU_CPU = os.environ.get("QEMU_CPU", "qemu64")
QEMU_RAM = os.environ.get("QEMU_RAM", "512M")


COMMANDS: tuple[tuple[str, str], ...] = (
    (
        "help",
        "OK help commands=ping,graph,tokenize,toy,quant,kernels,retrieve,assistant,ssod,cpu,exit",
    ),
    ("ping", "OK pong"),
    ("  ping  ", "OK pong"),
    ("unknown", "ERR unknown_command"),
    ("graph", "OK graph graph_id=0x0000000000535453 nodes=3 wires=2 last_completed=3"),
    ("tokenize", "OK tokenize text=hello tokens=5"),
    ("toy", "OK toy text="),
    ("quant", "OK quant tokens=67,68,69 last=69"),
    ("kernels", "OK kernels matvec=8,-47 top=1,3"),
    ("retrieve", "OK retrieve count=2 context_len=111 top=index:spec-13.1"),
    (
        "assistant",
        "OK assistant graph=0x0000000000535453 nodes=3 wires=2 active_node=none last_completed_node=3",
    ),
    (
        "ssod",
        "OK ssod ssod reason=page_fault rip=0xffff800000001234 fault_family=cpu_exception vector=0x0e error_code=0x0000000000000002",
    ),
    ("cpu", "OK cpu tier=Sse2"),
    ("graph", "OK graph graph_id=0x0000000000535453 nodes=3 wires=2 last_completed=3"),
    ("retrieve", "OK retrieve count=2 context_len=111 top=index:spec-13.1"),
)

OVERLONG_COMMAND = "x" * 81


def run_make_image() -> None:
    subprocess.run(["make", "-s", "image"], cwd=ROOT, check=True)


def read_until(fd: int, needle: str, deadline: float, transcript: bytearray) -> None:
    wanted = needle.encode()
    while wanted not in transcript:
        if time.monotonic() > deadline:
            raise TimeoutError(f"missing {needle!r}")
        ready, _, _ = select.select([fd], [], [], 0.1)
        if not ready:
            continue
        chunk = os.read(fd, 4096)
        if not chunk:
            raise RuntimeError("QEMU serial closed before expected output")
        transcript.extend(chunk)


def main() -> int:
    run_make_image()

    master_fd, slave_fd = pty.openpty()
    proc = subprocess.Popen(
        [
            "qemu-system-x86_64",
            "-cpu",
            QEMU_CPU,
            "-m",
            QEMU_RAM,
            "-no-reboot",
            "-device",
            "isa-debug-exit,iobase=0xf4,iosize=0x04",
            "-cdrom",
            str(IMAGE),
            "-boot",
            "d",
            "-serial",
            "stdio",
            "-display",
            "none",
            "-monitor",
            "none",
        ],
        cwd=ROOT,
        stdin=slave_fd,
        stdout=slave_fd,
        stderr=slave_fd,
        close_fds=True,
        start_new_session=True,
    )
    os.close(slave_fd)

    transcript = bytearray()
    deadline = time.monotonic() + TIMEOUT_SECONDS
    try:
        read_until(master_fd, "UNBOUNDOS_SHELL_READY", deadline, transcript)
        for command, expected in COMMANDS:
            os.write(master_fd, command.encode() + b"\n")
            read_until(master_fd, expected, deadline, transcript)
        os.write(master_fd, OVERLONG_COMMAND.encode())
        read_until(master_fd, "ERR command_too_long", deadline, transcript)
        os.write(master_fd, b"exit\n")
        read_until(master_fd, "OK halt", deadline, transcript)
    except Exception as exc:
        print("[qemu-interactive] FAIL", file=sys.stderr)
        print(f"  - {exc}", file=sys.stderr)
        print("----- transcript -----", file=sys.stderr)
        print(transcript.decode("utf-8", "replace"), file=sys.stderr)
        print("----------------------", file=sys.stderr)
        return 1
    finally:
        try:
            os.killpg(proc.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        try:
            proc.wait(timeout=3)
        except subprocess.TimeoutExpired:
            try:
                os.killpg(proc.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            proc.wait(timeout=3)
        os.close(master_fd)

    print(
        f"[qemu-interactive] PASS: cpu={QEMU_CPU} ram={QEMU_RAM} "
        "live serial shell exercised graph/LLM/retrieval paths"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
