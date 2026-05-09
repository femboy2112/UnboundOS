#!/usr/bin/env bash
# check_no_posix_paths.sh — POSIX-style path strings must not appear
# in graph-visible code or .MOD fixture descriptions. Storage adapter
# internals are exempt.
set -euo pipefail
cd "$(git rev-parse --show-toplevel 2>/dev/null || pwd)"

# Grep for path-shaped string literals in graph-visible crates.
# Scoped to crates/graph and crates/llm where graph-visible code
# lives. The umod/umdl crates parse opaque IDs, not paths; their
# parsers may legitimately mention path-shaped error messages, so
# we don't scan them here.
if rg -nE '"(/[A-Za-z]|\.\./|local://|\\\\)' --type rust crates/graph crates/llm 2>/dev/null; then
    echo "[no-posix-paths] graph-visible code contains a path-shaped string literal above." >&2
    exit 1
fi
exit 0
