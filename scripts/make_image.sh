#!/usr/bin/env bash
# make_image.sh — assemble a bootable image from a kernel ELF.
#
# Usage: ./scripts/make_image.sh <kernel-elf> <output-img>
#
# M0 builds a minimal GRUB/Multiboot2 ISO so the heartbeat can be
# tested under QEMU. This is a smoke-test boot path only; the
# spec-primary Limine handoff and real memory-map parsing remain M1.

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

if ! grub-file --is-x86-multiboot2 "$KERNEL_ELF"; then
    echo "[make-image] $KERNEL_ELF is not Multiboot2 bootable" >&2
    exit 1
fi

if ! command -v grub-mkrescue >/dev/null 2>&1; then
    echo "[make-image] grub-mkrescue missing" >&2
    exit 1
fi

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

mkdir -p "$TMPDIR/iso/boot/grub"
cp "$KERNEL_ELF" "$TMPDIR/iso/boot/kernel"
cat >"$TMPDIR/iso/boot/grub/grub.cfg" <<'GRUB'
set timeout=0
set default=0
serial --unit=0 --speed=115200 --word=8 --parity=no --stop=1
terminal_output serial

menuentry "UnboundOS M0 heartbeat" {
    multiboot2 /boot/kernel
    boot
}
GRUB

grub-mkrescue -o "$OUT_IMG" "$TMPDIR/iso" >/tmp/unboundos-grub-mkrescue.log 2>&1 || {
    echo "[make-image] grub-mkrescue failed" >&2
    cat /tmp/unboundos-grub-mkrescue.log >&2
    exit 1
}

echo "[make-image] wrote bootable Multiboot2 ISO: $OUT_IMG"
