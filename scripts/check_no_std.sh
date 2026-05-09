#!/usr/bin/env bash
# check_no_std.sh — every kernel-linked crate must declare #![no_std].
set -euo pipefail
cd "$(git rev-parse --show-toplevel 2>/dev/null || pwd)"

bad=0
for entry in \
    kernel/src/main.rs \
    crates/umod/src/lib.rs \
    crates/umdl/src/lib.rs \
    crates/graph/src/lib.rs \
    crates/llm/src/lib.rs ; do
    if [ ! -f "$entry" ]; then
        echo "[no-std] missing entry file: $entry" >&2
        bad=1
        continue
    fi
    if ! grep -qE '^#!\[no_std\]' "$entry"; then
        echo "[no-std] $entry does not declare #![no_std]" >&2
        bad=1
    fi
done
exit $bad
