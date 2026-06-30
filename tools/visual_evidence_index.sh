#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/visual_evidence/verify}"
mkdir -p "$out"

python3 - "$out" <<'PY'
import hashlib
import json
import shutil
import sys
from pathlib import Path

out = Path(sys.argv[1])
ARTIFACTS = [
    {
        "id": "native_combat_manifest",
        "title": "Native Combat 3D Renderer Manifest",
        "path": "artifacts/native_combat/verify/native_combat_render_manifest.json",
        "role": "3D renderer manifest",
        "json_expect": {"schema": "oathyard.native_combat_render.v1", "renderer": "native-software-3d", "native_3d_runtime_geometry": True, "truth_mutation": False},
    },
    {
        "id": "native_combat_primary_frame",
        "title": "Native Combat 3D Primary Frame",
        "path": "artifacts/native_combat/verify/native_combat_render.ppm",
        "role": "3D renderer PPM capture",
        "ppm": True,
    },
    {
        "id": "native_combat_third_person",
        "title": "Native Combat Third-Person 3D Viewport",
        "path": "artifacts/native_combat/verify/native_combat_3d_third_person.ppm",
        "role": "depth-sorted runtime glTF viewport",
        "ppm": True,
    },
    {
        "id": "native_combat_first_person",
        "title": "Native Combat First-Person 3D Viewport",
        "path": "artifacts/native_combat/verify/native_combat_3d_first_person.ppm",
        "role": "depth-sorted runtime glTF viewport",
        "ppm": True,
    },
    {
        "id": "native_roster_manifest",
        "title": "Native Roster 3D Showcase Manifest",
        "path": "artifacts/native_roster/verify/native_roster_showcase_manifest.json",
        "role": "3D roster renderer manifest",
        "json_expect": {"schema": "oathyard.native_roster_showcase.v1", "game_is_3d": True, "truth_mutation": False},
    },
    {
        "id": "native_roster_frame",
        "title": "Native Roster 3D Showcase Frame",
        "path": "artifacts/native_roster/verify/native_roster_showcase_01_saltreach_duelist.ppm",
        "role": "3D roster PPM capture",
        "ppm": True,
    },
    {
        "id": "renderer_target_report",
        "title": "3D Renderer-Only Target Audit",
        "path": "artifacts/renderer_target/verify/native_presentation_target_report.md",
        "role": "renderer target audit",
        "markers": ["Status: PASSED", "3D renderer"],
    },
]

def sha(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()

def is_ppm(path: Path) -> bool:
    try:
        return path.read_bytes().startswith(b"P6\n")
    except OSError:
        return False

entries = []
failures = []
for spec in ARTIFACTS:
    path = Path(spec["path"])
    checks = {"exists": path.is_file() and path.stat().st_size > 0}
    if checks["exists"] and spec.get("ppm"):
        checks["ppm"] = is_ppm(path)
    if checks["exists"] and spec.get("markers"):
        text = path.read_text(encoding="utf-8", errors="ignore")
        for marker in spec["markers"]:
            checks[f"marker:{marker}"] = marker in text
    if checks["exists"] and spec.get("json_expect"):
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except Exception:
            data = {}
        for key, value in spec["json_expect"].items():
            checks[f"json:{key}"] = data.get(key) == value
    passed = all(checks.values())
    if not passed:
        failures.append(spec["id"])
    entry = {
        "id": spec["id"],
        "title": spec["title"],
        "path": spec["path"],
        "role": spec["role"],
        "checks": checks,
        "passed": passed,
        "sha256": sha(path) if path.is_file() else "",
    }
    entries.append(entry)

hash_lines = []
for entry in entries:
    if entry["sha256"]:
        hash_lines.append(f"{entry['sha256']}  {entry['path']}")
(out / "visual_evidence_hashes.sha256").write_text("\n".join(hash_lines) + ("\n" if hash_lines else ""), encoding="utf-8")
(out / "failed_visual_artifacts.txt").write_text("none\n" if not failures else "\n".join(failures) + "\n", encoding="utf-8")
manifest = {
    "schema": "oathyard.visual_evidence_index.v2",
    "tool": "tools/visual_evidence_index.sh",
    "source": "3d-renderer-only-evidence",
    "passed": not failures,
    "entry_count": len(entries),
    "failed_count": len(failures),
    "entries": entries,
}
(out / "visual_evidence_manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
report = [
    "# OATHYARD Visual Evidence Index",
    "",
    f"Status: {'PASSED' if not failures else 'FAILED'}",
    "- Scope: native 3D renderer evidence only.",
    "",
    "## Entries",
]
for entry in entries:
    report.append(f"- {'PASS' if entry['passed'] else 'FAIL'} `{entry['id']}` `{entry['path']}` role `{entry['role']}` sha `{entry['sha256'][:16]}`")
(out / "visual_evidence_report.md").write_text("\n".join(report) + "\n", encoding="utf-8")
# Keep the historical filename, but make it a text-free SVG index card rather than a renderer path dependency.
svg = [
    '<svg xmlns="http://www.w3.org/2000/svg" width="960" height="540" viewBox="0 0 960 540">',
    '<rect width="960" height="540" fill="#15120f"/>',
    '<text x="32" y="44" fill="#f3e8cf" font-family="monospace" font-size="20">OATHYARD 3D renderer evidence index</text>',
]
for index, entry in enumerate(entries):
    y = 88 + index * 52
    color = '#7a8f5f' if entry['passed'] else '#b45b4d'
    svg.append(f'<rect x="32" y="{y}" width="896" height="38" fill="{color}" opacity="0.55"/>')
    svg.append(f'<text x="48" y="{y+24}" fill="#f7efd9" font-family="monospace" font-size="14">{entry["id"]} {entry["path"]}</text>')
svg.append('</svg>')
(out / "visual_evidence_contact_sheet.svg").write_text("\n".join(svg) + "\n", encoding="utf-8")
if failures:
    raise SystemExit(1)
PY

echo "visual evidence indexed: $out"
