#!/usr/bin/env python3
"""Unit-082 license/provenance metadata drift audit."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

OWNER_EVIDENCE = (
    "Unit-082 user-provided project context: Rodin/Meshy/model-generated asset use "
    "is owner-approved for internal/project use; this does not grant public-demo, "
    "release, store, legal, trademark, or owner visual acceptance."
)
FALSE_FIELDS = [
    "production_ready_visual",
    "owner_visual_acceptance",
    "public_demo_ready",
    "release_candidate_ready",
    "legal_clearance",
    "trademark_clearance",
    "store_readiness",
]


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("out", nargs="?", default="artifacts/license_metadata_drift/latest")
    args = parser.parse_args()
    root = Path.cwd()
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    model_files = sorted((root / "assets_src/model_candidates/t_73291be5").glob("*/*.model_source.json"))
    entries = []
    failures = []
    for path in model_files:
        data = read_json(path)
        entry = {
            "path": path.relative_to(root).as_posix(),
            "asset_id": data.get("asset_id", ""),
            "kind": data.get("kind", ""),
            "license_status": data.get("license_status", ""),
            "source_approved_for_project_use": data.get("source_approved_for_project_use"),
            "production_visual_candidate": data.get("production_visual_candidate"),
            "production_ready_visual": data.get("production_ready_visual"),
            "owner_visual_acceptance": data.get("owner_visual_acceptance"),
            "public_demo_ready": data.get("public_demo_ready"),
            "release_candidate_ready": data.get("release_candidate_ready"),
            "legal_clearance": data.get("legal_clearance"),
            "trademark_clearance": data.get("trademark_clearance"),
            "store_readiness": data.get("store_readiness"),
            "provenance": data.get("provenance", ""),
            "source_authoring_evidence_present": bool(data.get("source_authoring_evidence")),
        }
        if "pending_project_license_review" in json.dumps(data, sort_keys=True):
            failures.append(f"{entry['path']}: stale pending_project_license_review literal remains")
        if entry["license_status"] != "owner_approved_internal_project_use":
            failures.append(f"{entry['path']}: license_status not reconciled")
        if entry["source_approved_for_project_use"] is not True:
            failures.append(f"{entry['path']}: source_approved_for_project_use not true")
        if entry["production_visual_candidate"] is not True:
            failures.append(f"{entry['path']}: production_visual_candidate not true")
        for field in FALSE_FIELDS:
            if entry[field] is not False:
                failures.append(f"{entry['path']}: {field} must remain false")
        entries.append(entry)
    payload = {
        "schema": "oathyard.unit082.license_metadata_drift_audit.v1",
        "tool": "tools/license_metadata_drift_audit.py",
        "passed": not failures,
        "owner_approval_evidence": OWNER_EVIDENCE,
        "audited_model_source_count": len(entries),
        "stale_pending_json_count": sum(1 for entry in entries if entry["license_status"] != "owner_approved_internal_project_use"),
        "source_approved_for_project_use_count": sum(1 for entry in entries if entry["source_approved_for_project_use"] is True),
        "production_visual_candidate_count": sum(1 for entry in entries if entry["production_visual_candidate"] is True),
        "production_ready_visual_count": sum(1 for entry in entries if entry["production_ready_visual"] is True),
        "owner_visual_acceptance": False,
        "public_demo_ready": False,
        "release_candidate_ready": False,
        "legal_clearance": False,
        "trademark_clearance": False,
        "store_readiness": False,
        "failures": failures,
        "entries": entries,
    }
    (out / "license_metadata_drift_audit.json").write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    lines = [
        "# Unit-082 License Metadata Drift Audit",
        "",
        f"Status: {'PASSED' if not failures else 'FAILED'}",
        "",
        f"Owner-approval evidence: {OWNER_EVIDENCE}",
        "",
        f"- Audited model-source files: `{len(entries)}`",
        f"- Source-approved for project use: `{payload['source_approved_for_project_use_count']}`",
        f"- Production visual candidates: `{payload['production_visual_candidate_count']}`",
        f"- Production-ready visuals: `{payload['production_ready_visual_count']}`",
        "- Owner visual acceptance: `false`",
        "- Public demo ready: `false`",
        "- Release candidate ready: `false`",
        "- Legal/trademark/store clearance: `false` / `false` / `false`",
        "",
        "## Assets",
    ]
    for entry in entries:
        lines.append(f"- `{entry['asset_id']}` `{entry['kind']}` `{entry['path']}` license `{entry['license_status']}` production_ready_visual `{str(entry['production_ready_visual']).lower()}`")
    if failures:
        lines += ["", "## Failures"] + [f"- {failure}" for failure in failures]
    (out / "license_metadata_drift_audit.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0 if not failures else 1


if __name__ == "__main__":
    raise SystemExit(main())
