#!/usr/bin/env bash
# make_image.sh — assemble a bootable image from a kernel ELF.
#
# Usage: ./scripts/make_image.sh <kernel-elf> <output-img>
#
# This script is intentionally a stub. The final implementation
# depends on which bootloader the project commits to (Limine is the
# baseline per spec §3.1). Two paths are common:
#
#   1) Limine: copy a pre-built limine.bin + the kernel ELF into a
#      FAT32 partition on a raw image, then `limine bios-install`
#      against it.
#   2) Multiboot2/GRUB: build an ISO with grub-mkrescue.
#
# Until the bootloader is committed, this script just verifies the
# input ELF exists and prints a clear message. The /qemu-smoke skill
# will fail the gate until make_image.sh is implemented; this is by
# design — boot without a real image is a fiction we don't ship.

set -euo pipefail

KERNEL_ELF="${1:-}"
OUT_IMG="${2:-/tmp/unboundos.img}"

if [ -z "$KERNEL_ELF" ]; then
    echo "Usage: $0 <kernel-elf> [output-img]" >&2
    exit 2
fi

if [ ! -f "$KERNEL_ELF" ]; then
    echo "[make-image] kernel ELF not found: $KERNEL_ELF" >&2
    exit 1
fi

# Sanity check the ELF.
if ! file "$KERNEL_ELF" | grep -q 'ELF 64-bit'; then
    echo "[make-image] $KERNEL_ELF is not a 64-bit ELF" >&2
    exit 1
fi

echo "[make-image] kernel ELF OK: $KERNEL_ELF"
echo "[make-image] target image:  $OUT_IMG"
echo
echo "[make-image] STUB — bootloader integration not yet committed."
echo "[make-image] To proceed, choose one of:"
echo "  1. Limine: install limine bios files, build a FAT32 image with"
echo "     limine.cfg + kernel ELF, then run 'limine bios-install \$OUT_IMG'."
echo "  2. Multiboot2: build an ISO via 'grub-mkrescue -o \$OUT_IMG iso/'."
echo
echo "[make-image] writing placeholder image so /qemu-smoke can fail loudly..."
# Write a tiny image so qemu launch fails cleanly with an obvious
# 'no bootable disk' error rather than 'file not found'. The kernel
# ELF is appended for inspection convenience.
truncate -s 1M "$OUT_IMG"
exit 0
