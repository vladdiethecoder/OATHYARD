#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/research_audit/latest}"
mkdir -p "$out"

python3 - "$out" <<'PY'
import json
import re
import sys
from pathlib import Path

out = Path(sys.argv[1])
root = Path.cwd()
required_docs = [
    Path("docs/research/FRONTIER_TECH_LEVERAGE.md"),
    Path("docs/decisions/0002-high-fidelity-production-target.md"),
    Path("docs/decisions/0003-truth-vs-presentation-layering.md"),
    Path("docs/decisions/0004-renderer-and-asset-pipeline.md"),
]
required_sections = [
    "NVIDIA MotionBricks / MotionBricks-style smart primitives",
    "NVIDIA Warp",
    "NVIDIA Isaac Lab",
    "Newton Physics",
    "MuJoCo / MuJoCo Warp (MJWarp)",
    "NVIDIA PhysX",
    "Project Chrono",
    "Dense contact, FEM, SPH, DEM, MPM, cloth, deformable, and granular solver families",
    "Nanite/Lumen-class renderer target, RTX/path tracing, and upscaling references",
    "glTF/GLB runtime asset delivery",
    "OpenUSD / AOUSD source interchange",
    "NVIDIA Audio2Face-3D / facial animation",
    "Generative 3D tools and neural asset/model generators",
]
required_fields = [
    "Source link:",
    "Date/version:",
    "License/availability:",
    "Hardware/toolchain requirements:",
    "Intended OATHYARD layer:",
    "Determinism risk:",
    "IP/provenance risk:",
    "Integration plan:",
    "Fallback plan:",
    "Acceptance checks:",
]
checks = []
failures = []

def record(check_id, passed, detail):
    row = {"id": check_id, "passed": bool(passed), "detail": detail}
    checks.append(row)
    if not passed:
        failures.append(f"{check_id}: {detail}")

for path in required_docs:
    record(f"doc_exists_{path.as_posix().replace('/', '_')}", (root / path).is_file(), path.as_posix())

register_path = root / required_docs[0]
register = register_path.read_text(encoding="utf-8") if register_path.is_file() else ""

sections = {}
for match in re.finditer(r"^### (.+)$", register, re.MULTILINE):
    title = match.group(1).strip()
    start = match.end()
    next_match = re.search(r"^### ", register[start:], re.MULTILINE)
    end = start + next_match.start() if next_match else len(register)
    sections[title] = register[start:end]

for title in required_sections:
    section = sections.get(title, "")
    record(f"section_{re.sub(r'[^a-z0-9]+', '_', title.lower()).strip('_')}", bool(section), title)
    for field in required_fields:
        record(
            f"field_{re.sub(r'[^a-z0-9]+', '_', title.lower()).strip('_')}_{field.lower().split(':')[0].replace('/', '_').replace(' ', '_')}",
            field in section,
            f"{title} / {field}",
        )

for token in [
    "runtime_authoritative_truth` is forbidden by default",
    "PresentationBricks",
    "truth_mutation:false",
    "Do not call any of these high fidelity",
    "./tools/research_audit.sh",
    "./tools/presentation_truth_isolation.sh",
    "./tools/capture_high_fidelity_screens.sh",
    "./tools/visual_benchmark.sh",
    "./tools/sim_reference_compare.sh",
    "./tools/ai_planner_audit.sh",
    "./tools/final_acceptance.sh",
]:
    record(f"register_token_{re.sub(r'[^a-z0-9]+', '_', token.lower()).strip('_')[:80]}", token in register, token)

layer_hits = {"offline_research_authoring": 0, "runtime_presentation": 0, "runtime_authoritative_truth": 0}
for layer in layer_hits:
    layer_hits[layer] = register.count(f"`{layer}`")
record("layer_offline_present", layer_hits["offline_research_authoring"] >= 1, str(layer_hits))
record("layer_presentation_present", layer_hits["runtime_presentation"] >= 1, str(layer_hits))
record("layer_truth_forbidden_present", "runtime_authoritative_truth` is forbidden by default" in register, str(layer_hits))

adr_text = "\n\n".join((root / p).read_text(encoding="utf-8") if (root / p).is_file() else "" for p in required_docs[1:])
for token in [
    "current_fidelity_tier: Tier 0 / debug-local verification",
    "runtime_authoritative_truth",
    "PresentationBricks",
    "OpenUSD or equivalent",
    "glTF/GLB or equivalent",
    "owner_visual_acceptance:false",
    "public_demo_ready:false",
    "release_candidate_ready:false",
]:
    record(f"adr_token_{re.sub(r'[^a-z0-9]+', '_', token.lower()).strip('_')[:80]}", token in adr_text, token)

passed = not failures
manifest = {
    "schema": "oathyard.frontier_research_audit.v1",
    "tool": "tools/research_audit.sh",
    "passed": passed,
    "doc_count": len(required_docs),
    "technology_section_count": len(required_sections),
    "field_count_per_section": len(required_fields),
    "failed_check_count": len(failures),
    "checks": checks,
}
(out / "research_audit_manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
(out / "failed_research_checks.txt").write_text("none\n" if passed else "\n".join(failures) + "\n", encoding="utf-8")
report = [
    "# OATHYARD Frontier Research Audit",
    "",
    f"Status: {'PASSED' if passed else 'FAILED'}",
    f"- Required docs: `{len(required_docs)}`",
    f"- Required technology sections: `{len(required_sections)}`",
    f"- Fields per section: `{len(required_fields)}`",
    f"- Failed checks: `{len(failures)}`",
    "- Readiness claims: public-demo/release/owner acceptance must remain false.",
    "",
    "## Documents",
]
for path in required_docs:
    report.append(f"- `{path.as_posix()}`")
if failures:
    report.extend(["", "## Failures"] + [f"- {f}" for f in failures])
(out / "research_audit_report.md").write_text("\n".join(report) + "\n", encoding="utf-8")
if not passed:
    raise SystemExit(1)
PY

echo "research audit: $out"
