#!/usr/bin/env bash
# check_no_pointer_derive.sh — runtime types (NodeRuntime, WireRuntime,
# GraphRuntime, anything ending in Runtime) must not derive Serialize
# or Deserialize. Persistent format types are #[repr(C)] plain ints.
set -euo pipefail
cd "$(git rev-parse --show-toplevel 2>/dev/null || pwd)"

# Find files in the workspace that mention BOTH a *Runtime type and
# a serde derive. This is a coarse test; the format-engineer subagent
# does a more thorough audit when needed.
hits=0
while IFS= read -r f; do
    if grep -qE 'pub struct [A-Z][A-Za-z0-9_]*Runtime' "$f" 2>/dev/null \
       && grep -qE '#\[derive\([^)]*\b(Serialize|Deserialize)\b' "$f" 2>/dev/null; then
        echo "[no-pointer-derive] $f mentions a *Runtime type AND a serde derive." >&2
        hits=1
    fi
done < <(find crates kernel -type f -name '*.rs')

exit $hits
