#!/usr/bin/env python3
"""Build the deterministic M9 toy UMDL fixture bytes."""

from __future__ import annotations

import argparse
from pathlib import Path


UMDL_MAGIC = b"UMDL"
HEADER_LEN = 152
CHECKSUM_SEED = 0xCBF29CE484222325
CHECKSUM_PRIME = 0x00000100000001B3


def put_u16(buf: bytearray, offset: int, value: int) -> None:
    buf[offset : offset + 2] = value.to_bytes(2, "little")


def put_u32(buf: bytearray, offset: int, value: int) -> None:
    buf[offset : offset + 4] = value.to_bytes(4, "little")


def put_u64(buf: bytearray, offset: int, value: int) -> None:
    buf[offset : offset + 8] = value.to_bytes(8, "little")


def checksum64(data: bytes) -> int:
    value = CHECKSUM_SEED
    for byte in data:
        value ^= byte
        value = (value * CHECKSUM_PRIME) & 0xFFFF_FFFF_FFFF_FFFF
    return value


def header_checksum(data: bytes) -> int:
    header = bytearray(data[:HEADER_LEN])
    header[144:152] = b"\x00" * 8
    return checksum64(header)


def build_valid() -> bytes:
    buf = bytearray(512)
    buf[0:4] = UMDL_MAGIC
    put_u16(buf, 4, 1)
    put_u16(buf, 6, 0)
    put_u32(buf, 8, HEADER_LEN)
    put_u32(buf, 12, 1)
    put_u32(buf, 16, 0)
    put_u32(buf, 20, 1)
    put_u64(buf, 24, 160)
    put_u64(buf, 32, 72)
    put_u64(buf, 40, 240)
    put_u64(buf, 48, 48)
    put_u64(buf, 56, 320)
    put_u64(buf, 64, 16)
    put_u64(buf, 72, 400)
    put_u64(buf, 80, 24)
    put_u64(buf, 88, 16)
    put_u64(buf, 96, 8)
    put_u64(buf, 104, 2)
    put_u32(buf, 112, 32)
    put_u32(buf, 116, 256)
    put_u32(buf, 120, 1)
    put_u32(buf, 124, 8)
    put_u32(buf, 128, 1)
    put_u32(buf, 132, 0)
    put_u64(buf, 136, 0x0000_0000_0009_0001)

    put_u32(buf, 160, 3)
    put_u32(buf, 164, 256)
    for offset in (200, 204, 208, 212):
        put_u32(buf, offset, 0xFFFF_FFFF)
    put_u32(buf, 216, 1)
    put_u32(buf, 220, 1)

    put_u32(buf, 240, 7)
    buf[244] = 0
    buf[245] = 0
    buf[246] = 2
    put_u32(buf, 248, 2)
    put_u32(buf, 252, 4)
    put_u64(buf, 264, 0)
    put_u64(buf, 272, 16)
    put_u32(buf, 280, 16)
    buf[320:336] = b"0123456789abcdef"

    put_u64(buf, 400, checksum64(buf[160:232]))
    put_u64(buf, 408, checksum64(buf[240:288]))
    put_u64(buf, 416, checksum64(buf[320:336]))
    put_u64(buf, 144, header_checksum(buf))
    return bytes(buf)


def build_bad_magic() -> bytes:
    buf = bytearray(build_valid())
    buf[0:4] = b"BAD!"
    return bytes(buf)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--bad-magic-output", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()

    valid = build_valid()
    bad_magic = build_bad_magic()
    if args.check:
        assert len(valid) == 512
        assert valid[0:4] == UMDL_MAGIC
        assert bad_magic[0:4] == b"BAD!"
        assert header_checksum(valid) == int.from_bytes(valid[144:152], "little")
        assert checksum64(valid[160:232]) == int.from_bytes(valid[400:408], "little")
        assert checksum64(valid[240:288]) == int.from_bytes(valid[408:416], "little")
        assert checksum64(valid[320:336]) == int.from_bytes(valid[416:424], "little")
    if args.output:
        args.output.write_bytes(valid)
    if args.bad_magic_output:
        args.bad_magic_output.write_bytes(bad_magic)
    print("[umdl-fixture] PASS: deterministic M9 fixture bytes reproducible")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
