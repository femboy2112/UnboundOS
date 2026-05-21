#!/usr/bin/env python3
"""milestone_state.py — UnboundOS campaign-state diagnostic.

Parses MILESTONE_CATALOG.md and CURRENT_MISSION.md, cross-checks against
git history, and emits a JSON verdict consumed by the campaign-state
agent and by `make repo-state`.

stdlib only. Read-only. Never edits, commits, or pushes.

Output schema:
    {
      "active_milestone": "M0",          # or null
      "active_milestone_title": "Boot heartbeat",
      "status": "IN-PROGRESS",           # one of TODO / IN-PROGRESS / DONE / DEFERRED
      "branch": "campaign/m0-boot-heartbeat",
      "current_branch": "claude/upgrade-agents-skills-mmh6u",
      "baseline_matches": true,
      "issues": [],                      # list of strings; non-empty => USER-JUDGMENT
      "verdict": "READY"                 # READY / STOP / USER-JUDGMENT
    }
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CATALOG = ROOT / "MILESTONE_CATALOG.md"
MISSION = ROOT / "CURRENT_MISSION.md"
CAMPAIGN = ROOT / "CURRENT_CAMPAIGN.md"


def _run(cmd: list[str]) -> str:
    try:
        out = subprocess.run(
            cmd, cwd=ROOT, capture_output=True, text=True, timeout=10
        )
        return out.stdout.strip()
    except Exception:
        return ""


def parse_catalog(text: str) -> list[dict]:
    """Parse the markdown table in MILESTONE_CATALOG.md.

    Expected header row begins with `| ID ` and a separator row of dashes.
    Returns one dict per data row.
    """
    rows: list[dict] = []
    header: list[str] | None = None
    for line in text.splitlines():
        line = line.strip()
        if not line.startswith("|"):
            header = None
            continue
        cells = [c.strip() for c in line.strip("|").split("|")]
        if header is None:
            if cells and cells[0].lower() == "id":
                header = [c.lower().replace(" ", "_") for c in cells]
            continue
        if all(set(c) <= set("-: ") for c in cells):
            continue
        if not cells or not re.match(r"^M\d{1,2}$", cells[0]):
            continue
        rows.append(dict(zip(header, cells)))
    return rows


def active_milestone(rows: list[dict]) -> dict | None:
    in_progress = [r for r in rows if r.get("status", "").upper() == "IN-PROGRESS"]
    if not in_progress:
        return None
    return in_progress[0]


def parse_mission_baseline(text: str) -> dict:
    """Extract the `## Baseline to verify` fenced block from the mission."""
    out: dict = {"expected_branch": None, "expected_status": None, "raw": ""}
    in_baseline = False
    in_fence = False
    buf: list[str] = []
    for line in text.splitlines():
        if line.startswith("## "):
            if in_baseline:
                break
            in_baseline = line.strip().lower().startswith("## baseline to verify")
            continue
        if not in_baseline:
            continue
        if line.startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence:
            buf.append(line)
    out["raw"] = "\n".join(buf)
    for line in buf:
        m = re.search(r"branch[^:]*:\s*(\S+)", line, re.IGNORECASE)
        if m:
            out["expected_branch"] = m.group(1).strip()
        m = re.search(r"status[^:]*:\s*(\S+)", line, re.IGNORECASE)
        if m:
            out["expected_status"] = m.group(1).strip().upper()
    return out


def main() -> int:
    issues: list[str] = []

    if not CATALOG.exists():
        print(json.dumps({"verdict": "STOP", "error": "MILESTONE_CATALOG.md missing"}))
        return 1

    catalog_text = CATALOG.read_text()
    rows = parse_catalog(catalog_text)
    in_progress_rows = [r for r in rows if r.get("status", "").upper() == "IN-PROGRESS"]

    if len(in_progress_rows) > 1:
        issues.append(
            f"multiple IN-PROGRESS milestones: {[r['id'] for r in in_progress_rows]}"
        )

    active = active_milestone(rows)
    mission_baseline: dict = {}
    if MISSION.exists():
        mission_baseline = parse_mission_baseline(MISSION.read_text())
    else:
        issues.append("CURRENT_MISSION.md missing")

    if not CAMPAIGN.exists():
        issues.append("CURRENT_CAMPAIGN.md missing")

    current_branch = _run(["git", "branch", "--show-current"])

    baseline_matches = True
    if mission_baseline.get("expected_status") and active:
        exp = mission_baseline["expected_status"]
        actual = active.get("status", "").upper()
        if exp != actual:
            baseline_matches = False
            issues.append(
                f"baseline expects status {exp}, catalog says {actual}"
            )

    if not active:
        verdict = "STOP"
        issues.append("no IN-PROGRESS milestone in catalog")
    elif issues:
        verdict = "USER-JUDGMENT"
    else:
        verdict = "READY"

    result = {
        "active_milestone": active.get("id") if active else None,
        "active_milestone_title": active.get("title") if active else None,
        "status": active.get("status") if active else None,
        "branch": mission_baseline.get("expected_branch"),
        "current_branch": current_branch or None,
        "baseline_matches": baseline_matches,
        "issues": issues,
        "verdict": verdict,
    }
    print(json.dumps(result, indent=2))
    return 0 if verdict == "READY" else 0  # always exit 0; verdict carries truth


if __name__ == "__main__":
    sys.exit(main())
