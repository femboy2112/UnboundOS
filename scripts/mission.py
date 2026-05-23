#!/usr/bin/env python3
"""Validate and maintain UnboundOS mission state files."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CODEX = ROOT / ".codex"
MISSION = CODEX / "CURRENT_MISSION.md"
CAMPAIGN = CODEX / "CURRENT_CAMPAIGN.md"
LOG = CODEX / "MISSION_LOG.md"
ROOT_MISSION = ROOT / "CURRENT_MISSION.md"
ROOT_CAMPAIGN = ROOT / "CURRENT_CAMPAIGN.md"


REQUIRED_MISSION_KEYS = ("Mission:", "Campaign:", "Status:")
REQUIRED_CAMPAIGN_KEYS = (
    "Campaign:",
    "Active mission:",
    "Status:",
    "Stop rule:",
    "Bundle policy:",
    "Publish policy:",
    "Main policy:",
    "Campaign branch:",
)


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


def current_branch() -> str:
    proc = subprocess.run(
        ["git", "branch", "--show-current"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.returncode != 0:
        return ""
    return proc.stdout.strip()


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

    publish_policy = value(campaign_text, "Publish policy").lower()
    main_policy = value(campaign_text, "Main policy").lower()
    if "campaign branch" not in publish_policy:
        errors.append("CURRENT_CAMPAIGN.md Publish policy must name the campaign branch")
    for required in ("never merge to main", "never push main", "force-push"):
        if required not in main_policy:
            errors.append(f"CURRENT_CAMPAIGN.md Main policy missing {required!r}")

    branch = current_branch()
    campaign_branch = value(campaign_text, "Campaign branch")
    if not branch:
        errors.append("could not determine current git branch")
    elif branch == "main":
        errors.append("refusing mission work on main")
    elif campaign_branch and branch != campaign_branch:
        errors.append(
            f"current branch {branch!r} does not match Campaign branch {campaign_branch!r}"
        )

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
    validate_top_level_pointer(ROOT_MISSION, ".codex/CURRENT_MISSION.md", mission_name, errors)
    validate_top_level_pointer(ROOT_CAMPAIGN, ".codex/CURRENT_CAMPAIGN.md", active, errors)
    if errors:
        for error in errors:
            print(f"[mission] FAIL: {error}", file=sys.stderr)
        return 1

    print(f"[mission] OK: {mission_name}")
    return 0


def validate_top_level_pointer(
    path: Path, target: str, expected_name: str, errors: list[str]
) -> None:
    if not path.exists():
        errors.append(f"{path.name} compatibility pointer missing")
        return
    text = path.read_text(encoding="utf-8")
    if target not in text:
        errors.append(f"{path.name} must point to {target}")
    if expected_name and expected_name not in text:
        errors.append(f"{path.name} snapshot missing current name {expected_name!r}")
    if "M0 Boot Heartbeat" in text or "campaign/m0-boot-heartbeat" in text:
        errors.append(f"{path.name} still contains stale M0 mission state")


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
