#!/usr/bin/env bash
# qemu.sh — boot the UnboundOS kernel under QEMU.
#
# Usage:
#   ./scripts/qemu.sh                  # default qemu64, with display
#   ./scripts/qemu.sh --headless       # no display, capture serial to file
#   ./scripts/qemu.sh --cpu Skylake-Client
#   ./scripts/qemu.sh --no-serial      # exercise the no-UART fallback
#   ./scripts/qemu.sh --assert-heartbeat
#   ./scripts/qemu.sh --assert-ssod reason
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
ASSERT_HEARTBEAT=0
ASSERT_SSOD_REASON=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --headless)          HEADLESS=1; shift ;;
        --no-serial)         NO_SERIAL=1; shift ;;
        --assert-heartbeat)  ASSERT_HEARTBEAT=1; shift ;;
        --assert-ssod)       ASSERT_SSOD_REASON="$2"; shift 2 ;;
        --cpu)               QEMU_CPU="$2"; shift 2 ;;
        --image)             IMAGE="$2"; shift 2 ;;
        *)                   echo "[qemu] unknown arg: $1" >&2; exit 2 ;;
    esac
done

if [ "$ASSERT_HEARTBEAT" -eq 1 ] && [ "$NO_SERIAL" -eq 1 ]; then
    echo "[qemu] --assert-heartbeat requires serial capture" >&2
    exit 2
fi

if [ -n "$ASSERT_SSOD_REASON" ] && [ "$NO_SERIAL" -eq 1 ]; then
    echo "[qemu] --assert-ssod requires serial capture" >&2
    exit 2
fi

if [ "$ASSERT_HEARTBEAT" -eq 1 ] && [ -n "$ASSERT_SSOD_REASON" ]; then
    echo "[qemu] choose only one assertion mode" >&2
    exit 2
fi

if [ ! -f "$IMAGE" ]; then
    echo "[qemu] image not found: $IMAGE" >&2
    echo "       run: ./scripts/make_image.sh <kernel-elf> $IMAGE" >&2
    exit 2
fi

ARGS=(
    -cpu "$QEMU_CPU"
    -m "$QEMU_RAM"
    -no-reboot
    -device "isa-debug-exit,iobase=0xf4,iosize=0x04"
)

if file "$IMAGE" | grep -q 'ISO 9660'; then
    ARGS+=(-cdrom "$IMAGE" -boot d)
else
    ARGS+=(-drive "format=raw,file=$IMAGE")
fi

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

assert_heartbeat_order() {
    local log="$1"
    local expected=(
        'UNBOUNDOS_BOOT_BEGIN$'
        'UNBOUNDOS_CPU_PROFILE='
        'UNBOUNDOS_MEMMAP_OK='
        'UNBOUNDOS_IDT_OK$'
        'UNBOUNDOS_BOOT_OK$'
    )
    local idx=0
    local line

    while IFS= read -r line; do
        if [[ "$line" =~ ${expected[$idx]} ]]; then
            idx=$((idx + 1))
            if [ "$idx" -eq "${#expected[@]}" ]; then
                return 0
            fi
        fi
    done <"$log"

    echo "[qemu] missing heartbeat marker ${expected[$idx]} in $log" >&2
    return 1
}

assert_ssod_record() {
    local log="$1"
    local reason="$2"

    grep -q '^UNBOUNDOS_SSOD_BEGIN$' "$log" || {
        echo "[qemu] missing UNBOUNDOS_SSOD_BEGIN in $log" >&2
        return 1
    }
    grep -q "^reason=${reason}$" "$log" || {
        echo "[qemu] missing reason=${reason} in $log" >&2
        return 1
    }
    grep -Eq '^rip=0x[0-9a-fA-F]+$' "$log" || {
        echo "[qemu] missing rip field in $log" >&2
        return 1
    }
    grep -q '^UNBOUNDOS_SSOD_END$' "$log" || {
        echo "[qemu] missing UNBOUNDOS_SSOD_END in $log" >&2
        return 1
    }
}

if [ "$ASSERT_HEARTBEAT" -eq 0 ] && [ -z "$ASSERT_SSOD_REASON" ] && [ "$HEADLESS" -eq 0 ]; then
    # 60s wall-clock budget; kernel must reach UNBOUNDOS_BOOT_OK before then.
    exec timeout 60s qemu-system-x86_64 "${ARGS[@]}"
fi

rm -f "$SERIAL_LOG"
: >"$SERIAL_LOG"

timeout 60s qemu-system-x86_64 "${ARGS[@]}" &
QEMU_PID=$!

for _ in $(seq 1 600); do
    if [ -n "$ASSERT_SSOD_REASON" ] && grep -q '^UNBOUNDOS_SSOD_END$' "$SERIAL_LOG" 2>/dev/null; then
        kill "$QEMU_PID" >/dev/null 2>&1 || true
        wait "$QEMU_PID" >/dev/null 2>&1 || true
        assert_ssod_record "$SERIAL_LOG" "$ASSERT_SSOD_REASON"
        echo "[qemu] SSOD assertion passed for $ASSERT_SSOD_REASON"
        exit 0
    fi
    if grep -q '^UNBOUNDOS_BOOT_OK$' "$SERIAL_LOG" 2>/dev/null; then
        kill "$QEMU_PID" >/dev/null 2>&1 || true
        wait "$QEMU_PID" >/dev/null 2>&1 || true
        if [ -n "$ASSERT_SSOD_REASON" ]; then
            echo "[qemu] observed UNBOUNDOS_BOOT_OK while waiting for SSOD" >&2
            exit 1
        fi
        if [ "$ASSERT_HEARTBEAT" -eq 1 ]; then
            assert_heartbeat_order "$SERIAL_LOG"
            echo "[qemu] heartbeat assertion passed"
        else
            echo "[qemu] boot heartbeat reached UNBOUNDOS_BOOT_OK"
        fi
        exit 0
    fi
    if ! kill -0 "$QEMU_PID" >/dev/null 2>&1; then
        wait "$QEMU_PID" || true
        break
    fi
    sleep 0.1
done

kill "$QEMU_PID" >/dev/null 2>&1 || true
wait "$QEMU_PID" >/dev/null 2>&1 || true
if [ "$ASSERT_HEARTBEAT" -eq 1 ]; then
    assert_heartbeat_order "$SERIAL_LOG"
fi
if [ -n "$ASSERT_SSOD_REASON" ]; then
    assert_ssod_record "$SERIAL_LOG" "$ASSERT_SSOD_REASON"
fi
echo "[qemu] UNBOUNDOS_BOOT_OK not observed in $SERIAL_LOG" >&2
exit 1
