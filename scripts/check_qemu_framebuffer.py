#!/usr/bin/env python3
"""Boot QEMU and verify rendered framebuffer memory through the monitor."""

from __future__ import annotations

import os
import re
import signal
import socket
import subprocess
import sys
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
IMAGE = Path("/tmp/unboundos.img")
SERIAL_LOG = Path("/tmp/unboundos-framebuffer-serial.log")
MONITOR = Path("/tmp/unboundos-framebuffer-monitor.sock")
VNC_DISPLAY = "127.0.0.1:77"
TIMEOUT_SECONDS = 30.0
MIN_RENDERED_WORDS = 128
QEMU_CPU = os.environ.get("QEMU_CPU", "qemu64")
QEMU_RAM = os.environ.get("QEMU_RAM", "512M")


def wait_for(path: Path, needle: str, deadline: float) -> None:
    while time.monotonic() < deadline:
        if path.exists() and needle in path.read_text(encoding="utf-8", errors="replace"):
            return
        time.sleep(0.1)
    raise TimeoutError(f"missing {needle!r} in {path}")


def hmp_command(command: str) -> bytes:
    deadline = time.monotonic() + TIMEOUT_SECONDS
    while time.monotonic() < deadline:
        try:
            sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            sock.settimeout(2)
            sock.connect(str(MONITOR))
            break
        except OSError:
            time.sleep(0.1)
    else:
        raise TimeoutError("monitor socket did not appear")

    with sock:
        prompt_deadline = time.monotonic() + 3.0
        while time.monotonic() < prompt_deadline:
            try:
                greeting = sock.recv(4096)
            except socket.timeout:
                continue
            if b"(qemu)" in greeting:
                break
        sock.sendall(command.encode("ascii") + b"\n")
        chunks = []
        end = time.monotonic() + 3.0
        while time.monotonic() < end:
            try:
                chunk = sock.recv(4096)
            except socket.timeout:
                break
            if not chunk:
                break
            chunks.append(chunk)
            if b"(qemu)" in chunk:
                break
        return b"".join(chunks)


def serial_hex(name: str) -> int:
    text = SERIAL_LOG.read_text(encoding="utf-8", errors="replace")
    match = re.search(rf"^{name}=0x([0-9a-fA-F]+)$", text, re.MULTILINE)
    if match is None:
        raise RuntimeError(f"missing {name} in serial log")
    return int(match.group(1), 16)


def nonzero_words_from_xp(output: bytes) -> int:
    words = re.findall(rb"0x[0-9a-fA-F]{8}", output)
    return sum(1 for word in words if int(word, 16) != 0)


def main() -> int:
    subprocess.run(["make", "-s", "image"], cwd=ROOT, check=True)
    for path in (SERIAL_LOG, MONITOR):
        try:
            path.unlink()
        except FileNotFoundError:
            pass

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
            f"file:{SERIAL_LOG}",
            "-monitor",
            f"unix:{MONITOR},server,nowait",
            "-vnc",
            VNC_DISPLAY,
        ],
        cwd=ROOT,
        start_new_session=True,
    )

    try:
        deadline = time.monotonic() + TIMEOUT_SECONDS
        wait_for(SERIAL_LOG, "UNBOUNDOS_FRAMEBUFFER_RENDERED", deadline)
        framebuffer_addr = serial_hex("framebuffer_addr")
        width = serial_hex("framebuffer_width")
        height = serial_hex("framebuffer_height")
        output = hmp_command(f"xp /4096wx 0x{framebuffer_addr:x}")
        if b"Cannot" in output or b"Error" in output:
            raise RuntimeError(output.decode("utf-8", "replace"))
        if width < 640 or height < 480:
            raise RuntimeError(f"unexpected framebuffer size {width}x{height}")
        lit_words = nonzero_words_from_xp(output)
        if lit_words < MIN_RENDERED_WORDS:
            raise RuntimeError(
                f"framebuffer memory appears blank: lit_words={lit_words}; "
                f"monitor output={output.decode('utf-8', 'replace')[:1000]}"
            )
    except Exception as exc:
        print("[qemu-framebuffer] FAIL", file=sys.stderr)
        print(f"  - {exc}", file=sys.stderr)
        if SERIAL_LOG.exists():
            print("----- serial -----", file=sys.stderr)
            print(SERIAL_LOG.read_text(encoding="utf-8", errors="replace"), file=sys.stderr)
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

    print(
        f"[qemu-framebuffer] PASS: cpu={QEMU_CPU} ram={QEMU_RAM} "
        f"{width}x{height} framebuffer has {lit_words} lit words"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
