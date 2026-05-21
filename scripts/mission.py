#!/usr/bin/env python3
"""Validate and maintain UnboundOS mission state files."""

from __future__ import annotations

import argparse
import re
import sys
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEX = ROOT / ".codex"
MISSION = CODEX / "CURRENT_MISSION.md"
CAMPAIGN = CODEX / "CURRENT_CAMPAIGN.md"
LOG = CODEX / "MISSION_LOG.md"


REQUIRED_MISSION_KEYS = ("Mission:", "Campaign:", "Status:")
REQUIRED_CAMPAIGN_KEYS = ("Campaign:", "Active mission:", "Status:", "Publish policy:")


def read(path: Path) -> str:
    if not path.exists():
        raise SystemExit(f"[mission] missing required file: {path.relative_to(ROOT)}")
    return path.read_text(encoding="utf-8")


def value(text: str, key: str) -> str:
    prefix = f"{key}:"
    for line in text.splitlines():
        if line.startswith(prefix):
            return line[len(prefix) :].strip()
    return ""


def validate() -> int:
    mission_text = read(MISSION)
    campaign_text = read(CAMPAIGN)
    read(CODEX / "PROJECT_PLAN.md")
    read(LOG)

    errors: list[str] = []
    for key in REQUIRED_MISSION_KEYS:
        if key not in mission_text:
            errors.append(f"CURRENT_MISSION.md missing {key}")
    for key in REQUIRED_CAMPAIGN_KEYS:
        if key not in campaign_text:
            errors.append(f"CURRENT_CAMPAIGN.md missing {key}")

    mission_name = value(mission_text, "Mission")
    active = value(campaign_text, "Active mission")
    if mission_name and active and mission_name not in active:
        errors.append(
            "CURRENT_CAMPAIGN.md Active mission does not match "
            "CURRENT_MISSION.md Mission"
        )

    if "## Acceptance Criteria" not in mission_text:
        errors.append("CURRENT_MISSION.md missing Acceptance Criteria section")
    if "## Verification Commands" not in mission_text:
        errors.append("CURRENT_MISSION.md missing Verification Commands section")
    if "Stop rule:" not in campaign_text:
        errors.append("CURRENT_CAMPAIGN.md missing Stop rule")

    if errors:
        for error in errors:
            print(f"[mission] FAIL: {error}", file=sys.stderr)
        return 1

    print(f"[mission] OK: {mission_name}")
    return 0


def complete(args: argparse.Namespace) -> int:
    mission_text = read(MISSION)
    mission_name = value(mission_text, "Mission")
    if not mission_name:
        print("[mission] cannot complete unnamed mission", file=sys.stderr)
        return 1

    stamp = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    summary = args.summary.strip()
    entry = f"\n## {stamp} - {mission_name}\n\n- Status: completed\n- Summary: {summary}\n"
    LOG.write_text(read(LOG) + entry, encoding="utf-8")

    new_text = re.sub(r"^Status: .*$", "Status: completed", mission_text, count=1, flags=re.M)
    MISSION.write_text(new_text, encoding="utf-8")
    print(f"[mission] completed: {mission_name}")
    return 0


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="cmd", required=True)
    sub.add_parser("validate")
    complete_parser = sub.add_parser("complete")
    complete_parser.add_argument("--summary", required=True)
    args = parser.parse_args(argv[1:])

    if args.cmd == "validate":
        return validate()
    if args.cmd == "complete":
        return complete(args)
    raise AssertionError(args.cmd)


if __name__ == "__main__":
    sys.exit(main(sys.argv))
