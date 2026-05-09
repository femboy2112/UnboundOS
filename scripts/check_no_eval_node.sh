#!/usr/bin/env bash
# check_no_eval_node.sh — eval / runcode / dynamic-load nodes are
# forbidden (CLAUDE.md §2 rule 9). Generated code stays text until
# manually loaded as a verified module.
set -euo pipefail
cd "$(git rev-parse --show-toplevel 2>/dev/null || pwd)"

if rg -nE '\b(EvalNode|RunCode|RunGenerated|DynamicLoad|requires_dynamic_module_load|LlmEval)\b' \
        --type rust crates kernel 2>/dev/null; then
    echo "[no-eval-node] forbidden node-type or capability mention found above." >&2
    exit 1
fi
exit 0
