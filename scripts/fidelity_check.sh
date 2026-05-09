#!/usr/bin/env bash
# fidelity_check.sh — UnboundOS aggregate fidelity gate.
#
# Runs every gate in order, stopping on the first failure. Mirrors
# the CI matrix. The /fidelity-check skill calls this script first,
# then dispatches the fidelity-gate-reviewer subagent on the diff.
#
# Each gate either succeeds silently or prints a one-line failure
# reason and a non-zero exit. Detailed output goes to stderr; the
# top-level summary table goes to stdout.

set -uo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

PASS=()
FAIL=()

run_gate() {
    local name="$1"
    shift
    if "$@" >/tmp/fidelity-$$.log 2>&1; then
        PASS+=("$name")
        return 0
    else
        FAIL+=("$name")
        echo "[fidelity] FAIL: $name" >&2
        echo "----- output -----" >&2
        cat /tmp/fidelity-$$.log >&2
        echo "------------------" >&2
        return 1
    fi
}

# 1. cargo fmt
run_gate "cargo fmt --check" cargo fmt --check || true

# 2. cargo clippy
run_gate "cargo clippy -D warnings" \
    cargo clippy --workspace --all-targets -- -D warnings || true

# 3. workspace tests (excluding kernel — it requires the custom target)
run_gate "cargo test --workspace --exclude kernel" \
    cargo test --workspace --exclude kernel || true

# 4. kernel build (custom target).
# Skipped silently if the toolchain doesn't have rust-src for build-std;
# CI must set this up explicitly.
if rustup component list --installed 2>/dev/null | grep -q rust-src; then
    run_gate "cargo build -p kernel (custom target)" \
        cargo build -p kernel \
        --target x86_64-unboundos.json \
        -Z build-std=core,alloc \
        -Z build-std-features=compiler-builtins-mem \
        -Z json-target-spec || true
else
    echo "[fidelity] skip: kernel build (rust-src not installed)" >&2
fi

# 5. address scan
run_gate "address-scan persistent fixtures" \
    python3 scripts/address_scan.py tests/golden_graphs tests/golden_models || true

# 6. no_std hygiene
run_gate "no_std hygiene" bash scripts/check_no_std.sh || true

# 7. no eval node
run_gate "no eval/runcode/dynamic-load nodes" \
    bash scripts/check_no_eval_node.sh || true

# 8. no serde derives on runtime types
run_gate "no Serialize/Deserialize on runtime types" \
    bash scripts/check_no_pointer_derive.sh || true

# 9. no POSIX paths in graph-visible code
run_gate "no POSIX path leakage in graph-visible code" \
    bash scripts/check_no_posix_paths.sh || true

# 10. dispatch boundary holds
run_gate "tensor backend symbols only inside dispatch.rs/kernels" \
    bash scripts/check_dispatch_boundary.sh || true

rm -f /tmp/fidelity-$$.log

echo
echo "=== Fidelity Check Summary ==="
echo "PASSED: ${#PASS[@]}"
for g in "${PASS[@]}"; do echo "  ✓ $g"; done
echo "FAILED: ${#FAIL[@]}"
for g in "${FAIL[@]}"; do echo "  ✗ $g"; done

if [ ${#FAIL[@]} -gt 0 ]; then
    echo
    echo "Verdict: BLOCK"
    exit 1
fi

echo
echo "Verdict: PROCEED"
exit 0
