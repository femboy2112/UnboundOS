#!/usr/bin/env python3
"""milestone_advance.py — canonical commit message generator.

Given a milestone ID (e.g. M0) and a step number, prints the commit
message the current-mission agent must use. The agent never invents
wording; this keeps commit history greppable and spec-citing.

Reads CURRENT_CAMPAIGN.md to extract the step's title and spec section
hints. stdlib only. Read-only.

Usage:
    python3 scripts/milestone_advance.py M0 3
    python3 scripts/milestone_advance.py M0 3 --one-liner "why this step"
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CAMPAIGN = ROOT / ".codex" / "CURRENT_CAMPAIGN.md"


def campaign_source(text: str) -> Path | None:
    match = re.search(r"`(docs/campaigns/[^`]+\.md)`", text)
    if not match:
        return None
    path = ROOT / match.group(1)
    return path if path.exists() else None


def find_step(text: str, step_n: int) -> tuple[str, str] | None:
    """Return (title, spec_section) for `# Step N — …` header."""
    pattern = re.compile(
        rf"^#\s+Step\s+{step_n}\s+[—-]\s+(.+?)$", re.MULTILINE
    )
    m = pattern.search(text)
    if not m:
        return None
    title = m.group(1).strip()
    spec_section = ""
    # Look at the next 30 lines for a spec-section hint
    tail = text[m.end():m.end() + 2000]
    spec_match = re.search(r"spec\s+§([\d\.,\s§]+)", tail, re.IGNORECASE)
    if spec_match:
        spec_section = spec_match.group(1).strip().rstrip(",.")
    return title, spec_section


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("milestone", help="e.g. M0")
    p.add_argument("step", type=int)
    p.add_argument("--one-liner", default="", help="optional rationale")
    args = p.parse_args()

    if not CAMPAIGN.exists():
        print("error: CURRENT_CAMPAIGN.md missing", file=sys.stderr)
        return 1

    text = CAMPAIGN.read_text()
    info = find_step(text, args.step)
    if info is None:
        source = campaign_source(text)
        if source is not None:
            text = source.read_text()
            info = find_step(text, args.step)
    if info is None:
        print(
            f"error: Step {args.step} not found in CURRENT_CAMPAIGN.md or campaign source",
            file=sys.stderr,
        )
        return 1
    title, spec = info

    spec_tag = f" (spec §{spec})" if spec else ""
    line1 = f"{args.milestone} step {args.step}: {title}{spec_tag}"
    body = args.one_liner or f"Implements Step {args.step} of the {args.milestone} campaign."
    print(line1)
    print()
    print(body)
    return 0


if __name__ == "__main__":
    sys.exit(main())
