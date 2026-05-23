#!/usr/bin/env bash
# gates.sh — UnboundOS sequential gate pipeline.
#
# Single entrypoint chaining every gate `/go` must respect, in order.
# Stops on the first failure. Mirrors toy-brain's tools/check_all.sh
# pattern but reuses scripts/fidelity_check.sh and scripts/qemu.sh.
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

TOTAL=8
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

qemu_smoke() {
    make -s image
    bash scripts/qemu.sh --headless --assert-heartbeat
}

step 1 "cargo fmt --check" cargo fmt --check
step 2 "cargo clippy -D warnings" \
    cargo clippy --workspace --all-targets -- -D warnings
step 3 "cargo test --workspace --exclude kernel" \
    cargo test --workspace --exclude kernel
step 4 "address-scan persistent fixtures" \
    python3 scripts/address_scan.py tests/golden_graphs tests/golden_models
step 5 "assistant-smoke" python3 scripts/check_assistant_smoke.py
step 6 "retrieval-smoke" python3 scripts/check_retrieval_smoke.py
step 7 "fidelity matrix" bash scripts/fidelity_check.sh
step 8 "qemu-smoke heartbeat" qemu_smoke

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
