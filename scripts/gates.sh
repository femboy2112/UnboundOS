#!/usr/bin/env bash
# gates.sh — UnboundOS sequential gate pipeline.
#
# Single entrypoint chaining every spec gate `/go` must respect, in order.
# Stops on the first failure. This is intentionally broader than a quick smoke:
# every DONE milestone's catalog gate must stay reproducible from checkout.
#
# Called by `make gates` and by the current-mission agent during the
# /go preflight burst. Read-only — never commits, never pushes.
#
# Exits 0 with "Verdict: PROCEED" if every gate passes.
# Exits 1 with "Verdict: BLOCK" on the first failure (subsequent
# gates are skipped to keep the operator focused on the root cause).

set -uo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

TOTAL=24
PASS=()
FAILED=""

step() {
    local n="$1" name="$2"
    shift 2
    if [ -n "$FAILED" ]; then
        echo "[$n/$TOTAL] SKIP: $name (prior gate failed)"
        return 0
    fi
    echo "[$n/$TOTAL] RUN:  $name"
    if "$@" >/tmp/gates-$$.log 2>&1; then
        PASS+=("$name")
        echo "[$n/$TOTAL] PASS: $name"
    else
        FAILED="$name"
        echo "[$n/$TOTAL] FAIL: $name" >&2
        echo "----- output -----" >&2
        cat /tmp/gates-$$.log >&2
        echo "------------------" >&2
    fi
}

step 1 "mission state" python3 scripts/mission.py validate
step 2 "cargo fmt --check" cargo fmt --check
step 3 "cargo clippy -D warnings" \
    cargo clippy --workspace --all-targets -- -D warnings
step 4 "cargo test --workspace --exclude kernel" \
    cargo test --workspace --exclude kernel
step 5 "kernel host module tests" python3 scripts/check_kernel_host_tests.py
step 6 "kernel custom-target release build" make -s kernel
step 7 "address-scan persistent fixtures" \
    python3 scripts/address_scan.py tests/golden_graphs tests/golden_models
step 8 "fidelity matrix" bash scripts/fidelity_check.sh
step 9 "ui-smoke" python3 scripts/check_ui_smoke.py
step 10 "tokenizer-smoke" python3 scripts/check_tokenizer_smoke.py
step 11 "toy-transformer-smoke" python3 scripts/check_toy_transformer_smoke.py
step 12 "umdl-smoke" python3 scripts/check_umdl_smoke.py
step 13 "quantized-smoke" python3 scripts/check_quantized_smoke.py
step 14 "assistant-smoke" python3 scripts/check_assistant_smoke.py
step 15 "retrieval-smoke" python3 scripts/check_retrieval_smoke.py
step 16 "qemu heartbeat" make -s qemu-headless
step 17 "qemu interactive serial shell" make -s qemu-interactive-smoke
step 18 "qemu no-serial fallback" make -s qemu-no-serial
step 19 "qemu SSOD divide_error" make -s qemu-fault-de
step 20 "qemu SSOD invalid_opcode" make -s qemu-fault-ud
step 21 "qemu SSOD page_fault" make -s qemu-fault-pf
step 22 "qemu M2 arena/memory dump" make -s qemu-m2-dump
step 23 "qemu initial graph boot" make -s qemu-graph-boot
step 24 "qemu M6 storage marker" make -s qemu-storage-smoke

rm -f /tmp/gates-$$.log

echo
echo "=== Gates Summary ==="
echo "PASSED: ${#PASS[@]}/$TOTAL"
for g in "${PASS[@]}"; do echo "  ✓ $g"; done

if [ -n "$FAILED" ]; then
    echo "FAILED: $FAILED"
    echo
    echo "Verdict: BLOCK"
    exit 1
fi

echo
echo "Verdict: PROCEED"
exit 0
