#!/usr/bin/env bash
# qemu.sh — boot the UnboundOS kernel under QEMU.
#
# Usage:
#   ./scripts/qemu.sh                  # default qemu64, with display
#   ./scripts/qemu.sh --headless       # no display, capture serial to file
#   ./scripts/qemu.sh --cpu Skylake-Client
#   ./scripts/qemu.sh --no-serial      # exercise the no-UART fallback
#
# Environment overrides:
#   QEMU_CPU      override CPU model (default: qemu64)
#   QEMU_RAM      override RAM size (default: 512M)
#   IMAGE         override image path (default: /tmp/unboundos.img)
#   SERIAL_LOG    override serial-capture path (default: /tmp/unboundos-serial.log)

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

QEMU_CPU="${QEMU_CPU:-qemu64}"
QEMU_RAM="${QEMU_RAM:-512M}"
IMAGE="${IMAGE:-/tmp/unboundos.img}"
SERIAL_LOG="${SERIAL_LOG:-/tmp/unboundos-serial.log}"
HEADLESS=0
NO_SERIAL=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --headless)   HEADLESS=1; shift ;;
        --no-serial)  NO_SERIAL=1; shift ;;
        --cpu)        QEMU_CPU="$2"; shift 2 ;;
        --image)      IMAGE="$2"; shift 2 ;;
        *)            echo "[qemu] unknown arg: $1" >&2; exit 2 ;;
    esac
done

if [ ! -f "$IMAGE" ]; then
    echo "[qemu] image not found: $IMAGE" >&2
    echo "       run: ./scripts/make_image.sh <kernel-elf> $IMAGE" >&2
    exit 2
fi

ARGS=(
    -cpu "$QEMU_CPU"
    -m "$QEMU_RAM"
    -drive "format=raw,file=$IMAGE"
    -no-reboot
    -device "isa-debug-exit,iobase=0xf4,iosize=0x04"
)

if [ "$NO_SERIAL" -eq 1 ]; then
    ARGS+=(-serial none)
else
    ARGS+=(-serial "file:$SERIAL_LOG")
fi

if [ "$HEADLESS" -eq 1 ]; then
    ARGS+=(-display none)
fi

echo "[qemu] cpu=$QEMU_CPU ram=$QEMU_RAM image=$IMAGE"
[ "$NO_SERIAL" -eq 0 ] && echo "[qemu] serial → $SERIAL_LOG"

# 60s wall-clock budget; kernel must reach UNBOUNDOS_BOOT_OK before then.
exec timeout 60s qemu-system-x86_64 "${ARGS[@]}"
