#!/usr/bin/env python3
"""Select current OATHYARD verification artifact roots without confusing stale baselines."""
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any

TIMESTAMP_RE = re.compile(r"^(\d{8}T\d{6}Z)_(.+)$")
KNOWN_HISTORICAL = {"20260704T010739Z_unit081_high_fidelity_gameplay_demo"}
KNOWN_UNIT081_POST_FIX = "20260704T031326Z_unit081_final_gates_after_perf_fix"


def classify(path: Path) -> dict[str, Any]:
    name = path.name
    match = TIMESTAMP_RE.match(name)
    timestamp = match.group(1) if match else ""
    slug = match.group(2) if match else name
    is_unit082 = "unit082" in slug
    is_unit081 = "unit081" in slug
    return {
        "path": path.as_posix(),
        "name": name,
        "timestamp": timestamp,
        "slug": slug,
        "unit": "unit082" if is_unit082 else ("unit081" if is_unit081 else "other"),
        "historical_baseline_only": name in KNOWN_HISTORICAL,
        "unit081_post_fix_baseline": name == KNOWN_UNIT081_POST_FIX,
        "current_run_candidate": is_unit082 and not (name in KNOWN_HISTORICAL),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("out", nargs="?", default="artifacts/root_selection/latest")
    parser.add_argument("--roots-dir", default="artifacts/verification")
    args = parser.parse_args()
    roots_dir = Path(args.roots_dir)
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    roots = [classify(path) for path in sorted(roots_dir.iterdir()) if path.is_dir()] if roots_dir.is_dir() else []
    roots.sort(key=lambda row: row.get("timestamp", ""))
    unit082 = [row for row in roots if row["current_run_candidate"]]
    unit081_post = [row for row in roots if row["unit081_post_fix_baseline"]]
    selected_current = unit082[-1] if unit082 else None
    payload = {
        "schema": "oathyard.artifact_root_selection.v1",
        "tool": "tools/artifact_root_selection.py",
        "roots_dir": roots_dir.as_posix(),
        "selected_current_root": selected_current,
        "unit081_post_fix_baseline": unit081_post[-1] if unit081_post else None,
        "historical_baseline_roots": [row for row in roots if row["historical_baseline_only"]],
        "wrapper_rc_does_not_imply_child_success": True,
        "selection_policy": "Prefer explicit Unit-082 current-run roots by timestamp; Unit-081 20260704T031326Z is post-fix baseline only; Unit-081 20260704T010739Z is historical baseline only.",
        "roots": roots,
    }
    (out / "latest_artifact_root_selection.json").write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    lines = [
        "# OATHYARD Latest Artifact Root Selection",
        "",
        f"Selected current root: `{selected_current['path'] if selected_current else 'none'}`",
        f"Unit-081 post-fix baseline: `{unit081_post[-1]['path'] if unit081_post else 'missing'}`",
        "Historical baseline only: `artifacts/verification/20260704T010739Z_unit081_high_fidelity_gameplay_demo`",
        "",
        "Wrapper rc=0 is not treated as child gate success; child command rc tables are authoritative.",
        "",
        "## Roots",
        "",
        "| Timestamp | Unit | Historical | Current candidate | Path |",
        "| --- | --- | ---: | ---: | --- |",
    ]
    for row in roots:
        lines.append(f"| `{row['timestamp']}` | `{row['unit']}` | `{str(row['historical_baseline_only']).lower()}` | `{str(row['current_run_candidate']).lower()}` | `{row['path']}` |")
    (out / "latest_artifact_root_selection.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(payload["selected_current_root"]["path"] if selected_current else "")
    return 0 if selected_current else 1


if __name__ == "__main__":
    raise SystemExit(main())
