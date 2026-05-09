#!/usr/bin/env bash
# check_dispatch_boundary.sh — backend-specific tensor symbols may
# only be referenced from crates/llm/src/dispatch.rs and from
# crates/llm/src/kernels/. Anywhere else is a fidelity violation.
set -euo pipefail
cd "$(git rev-parse --show-toplevel 2>/dev/null || pwd)"

if rg -nE '\bmatvec_q4_(avx512|avx2|avx|sse2|scalar)\b|\bsoftmax_(avx2|avx|sse2|scalar)\b|\brope_(avx2|avx|sse2|scalar)\b' \
        --type rust crates kernel \
        --glob '!crates/llm/src/dispatch.rs' \
        --glob '!crates/llm/src/kernels/**' 2>/dev/null; then
    echo "[dispatch-boundary] backend-specific tensor symbol referenced outside dispatch boundary." >&2
    exit 1
fi
exit 0
