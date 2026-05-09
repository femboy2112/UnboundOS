#!/usr/bin/env bash
# hook_post_cargo_fmt.sh — PostToolUse reminder for rustfmt drift.
#
# After an Edit / Write / MultiEdit on any *.rs file, run
# `cargo fmt --check` across the workspace and surface the diff as a
# reminder. Never blocks — formatting is a style concern, not a
# correctness one, and `make fmt` / the /fidelity-check gate will fail
# the gate authoritatively.

set -uo pipefail

# Read the hook payload off stdin so the harness doesn't deadlock.
payload=$(cat)

# Extract tool_name and file_path with a small Python helper so we
# don't take a jq dependency. The script lives in /tmp because the
# hook runs in the repo root and we don't want to pollute scripts/.
file_path=$(printf '%s' "$payload" | python3 -c '
import json, sys
try:
    d = json.load(sys.stdin)
except Exception:
    sys.exit(0)
t = d.get("tool_name") or ""
if t not in ("Edit", "Write", "MultiEdit"):
    sys.exit(0)
fp = (d.get("tool_input") or {}).get("file_path") or ""
print(fp)
')

# Out of scope unless the edit was on a *.rs file.
case "$file_path" in
    *.rs) ;;
    *) exit 0 ;;
esac

# cargo may legitimately not be installed in some sandboxes; do not
# error out if so. Silent success keeps the harness usable.
if ! command -v cargo >/dev/null 2>&1; then
    exit 0
fi

if cargo fmt --check >/dev/null 2>&1; then
    exit 0
fi

echo "[cargo-fmt] reminder: cargo fmt --check reported drift after editing $file_path." >&2
echo "[cargo-fmt] run \`cargo fmt\` (or \`make fmt\`) to clean up." >&2
exit 0
