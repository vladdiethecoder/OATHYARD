#!/usr/bin/env bash
set -uo pipefail

out="${1:-artifacts/asset_audit/latest}"
mkdir -p "$out"

./tools/audit_rodin_assets.sh "$out"
rc=$?

copy_if_present() {
  local src="$1"
  local dst="$2"
  if [[ -f "$src" ]]; then
    cp "$src" "$dst"
  fi
}

copy_if_present "$out/rodin_asset_audit.json" "$out/generated_asset_audit.json"
copy_if_present "$out/rodin_asset_audit.md" "$out/generated_asset_audit.md"
copy_if_present "$out/rodin_asset_audit.csv" "$out/generated_asset_audit.csv"

if [[ ! -f "$out/generated_asset_audit.md" ]]; then
  cat > "$out/generated_asset_audit.md" <<'MD'
# OATHYARD Generated Asset Audit

Status: FAILED

`tools/audit_rodin_assets.sh` did not produce the expected markdown report. Inspect `rodin_asset_audit.*` outputs and the wrapper log.
MD
fi

python3 - "$out" "$rc" <<'PY'
import json
import sys
from pathlib import Path

out = Path(sys.argv[1])
audit_rc = int(sys.argv[2])
audit_path = out / "generated_asset_audit.json"

if audit_path.is_file():
    audit = json.loads(audit_path.read_text(encoding="utf-8"))
else:
    audit = {
        "schema": "oathyard.generated_asset_audit_missing.v1",
        "passed": False,
        "production_asset_ready": False,
        "asset_count": 0,
        "acceptance_status_counts": {},
        "failures": ["generated_asset_audit.json missing"],
        "assets": [],
    }

assets = audit.get("assets", []) if isinstance(audit.get("assets", []), list) else []
quarantined = []
license_blocked = 0
for asset in assets:
    blockers = asset.get("acceptance_blockers", [])
    if not isinstance(blockers, list):
        blockers = [str(blockers)] if blockers else []
    license_status = str(asset.get("license_terms_status", ""))
    commercial = str(asset.get("commercial_use_allowed", ""))
    license_or_commercial_blocked = (
        "pending" in license_status.lower()
        or commercial.lower().startswith("unverified")
        or "license_or_project_license_pending" in blockers
    )
    if license_or_commercial_blocked:
        license_blocked += 1
    if asset.get("acceptance_status") != "production-ready" or license_or_commercial_blocked:
        quarantined.append(
            {
                "asset_name": asset.get("asset_name", ""),
                "kind": asset.get("kind", ""),
                "acceptance_status": asset.get("acceptance_status", ""),
                "license_terms_status": license_status,
                "commercial_use_allowed": commercial,
                "runtime_export": asset.get("runtime_export", ""),
                "source_file": asset.get("source_file", ""),
                "source_authoring_evidence": asset.get("source_authoring_evidence", {}),
                "manifest_source": asset.get("manifest_source", ""),
                "quarantine_reasons": blockers,
            }
        )

manifest = {
    "schema": "oathyard.generated_asset_quarantine.v1",
    "tool": "tools/audit_generated_assets.sh",
    "source_audit_manifest": "generated_asset_audit.json",
    "generated_asset_audit_rc": audit_rc,
    "candidate_asset_quarantine_active": bool(quarantined),
    "production_asset_ready": False,
    "in_engine_visual_ready": False,
    "high_fidelity_ready": False,
    "owner_visual_accepted": False,
    "public_demo_visual_ready": False,
    "release_candidate_ready": False,
    "legal_clearance": False,
    "trademark_clearance": False,
    "asset_count": audit.get("asset_count", len(assets)),
    "quarantined_asset_count": len(quarantined),
    "blocked_by_license_or_commercial_use": license_blocked,
    "acceptance_status_counts": audit.get("acceptance_status_counts", {}),
    "failures": audit.get("failures", []),
    "quarantine_policy": "Generated/Rodin/model-candidate assets remain candidate-only until source/provenance/license/DCC/external-validation/native-renderer/owner gates pass.",
    "quarantined_assets": quarantined,
}
(out / "generated_asset_quarantine_manifest.json").write_text(
    json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
)

lines = [
    "# OATHYARD Generated Asset Quarantine",
    "",
    "Status: FAIL-CLOSED" if quarantined or audit_rc != 0 else "Status: PASSED",
    "",
    "No generated/model-candidate asset is promoted to production.",
    f"- Source audit rc: `{audit_rc}`",
    f"- Audited asset count: `{manifest['asset_count']}`",
    f"- Quarantined asset count: `{len(quarantined)}`",
    f"- Blocked by license/commercial-use evidence: `{license_blocked}`",
    "- Production asset ready: `false`",
    "- Owner visual accepted: `false`",
    "- Public demo visual ready: `false`",
    "- Release candidate ready: `false`",
    "",
    "## Quarantine policy",
    manifest["quarantine_policy"],
]
if audit.get("failures"):
    lines.extend(["", "## Source audit blockers"])
    lines.extend(f"- {failure}" for failure in audit.get("failures", []))
if quarantined:
    lines.extend(["", "## Quarantined assets"])
    for item in quarantined:
        reasons = ", ".join(item.get("quarantine_reasons", [])[:5])
        if len(item.get("quarantine_reasons", [])) > 5:
            reasons += ", ..."
        lines.append(
            f"- `{item['asset_name']}` `{item['kind']}` status `{item['acceptance_status']}`: {reasons}"
        )
(out / "generated_asset_quarantine_report.md").write_text(
    "\n".join(lines) + "\n", encoding="utf-8"
)

required_unblock_stages = [
    {
        "id": "license_and_commercial_clearance",
        "required_evidence": "approved project license/commercial-use decision for every source/runtime asset",
    },
    {
        "id": "dcc_or_openusd_source_authoring",
        "required_evidence": "source-authored Blender/OpenUSD/equivalent production asset files, not procedural candidate JSON only",
    },
    {
        "id": "external_geometry_validation",
        "required_evidence": "external DCC/Khronos/glTF/topology validation for mesh format, scale, normals, tangents, manifold/topology where relevant",
    },
    {
        "id": "material_texture_uv_completion",
        "required_evidence": "production UVs and material channels including base/albedo, normal, roughness/metallic/AO/detail/wetness/damage where relevant",
    },
    {
        "id": "rig_truth_contact_profile_validation",
        "required_evidence": "rig/skin/truth-joint/contact/physics profile validation for each runtime slot where applicable",
    },
    {
        "id": "native_renderer_capture_matrix",
        "required_evidence": "native 3D renderer capture evidence for required production closeups/gameplay states",
    },
    {
        "id": "owner_visual_acceptance",
        "required_evidence": "explicit owner/human acceptance after current native-renderer evidence packet",
    },
]

def rig_truth_contact_stage_needed(asset, blockers):
    explicit_blockers = {
        "fighter_rig_validation_missing",
        "fighter_skin_weights_missing",
        "truth_joint_mapping_missing",
        "contact_physics_profile_missing",
    }
    if any(str(blocker) in explicit_blockers for blocker in blockers):
        return True
    kind = str(asset.get("kind", ""))
    contact = asset.get("contact_physics_profile_status", {})
    if not isinstance(contact, dict) or contact.get("passed") is not True or contact.get("missing_fields"):
        return True
    truth_map = asset.get("canonical_truth_joint_mapping_status", {})
    if not isinstance(truth_map, dict):
        return True
    if kind in {"fighters", "fighter"}:
        if asset.get("rig_status") != "present":
            return True
        if asset.get("skin_weight_status") != "present":
            return True
        if int(truth_map.get("canonical_truth_joint_count", 0)) < 16:
            return True
    return False


def stages_for_asset(asset, quarantine_entry):
    blockers = quarantine_entry.get("quarantine_reasons", [])
    joined = " ".join(str(blocker).lower() for blocker in blockers)
    stages = []
    source_authoring = asset.get("source_authoring_evidence", {})
    if not isinstance(source_authoring, dict):
        source_authoring = {}
    source_authoring_present = source_authoring.get("dcc_or_openusd_source_present") is True
    license_status = str(asset.get("license_terms_status", "")).lower()
    commercial = str(asset.get("commercial_use_allowed", "")).lower()
    if "license" in joined or "commercial" in joined or "pending" in license_status or commercial.startswith("unverified"):
        stages.append("license_and_commercial_clearance")
    source_authoring_blockers = {
        "procedural_model_candidate_not_dcc_source",
        "source_not_dcc_or_interchange_file",
    }
    if any(str(blocker) in source_authoring_blockers for blocker in blockers) or (("procedural" in joined or "interchange" in joined) and not source_authoring_present):
        stages.append("dcc_or_openusd_source_authoring")
    if (
        "external" in joined
        or "gltf" in joined
        or "topology" in joined
        or "manifold" in joined
        or "normal" in joined
        or "tangent" in joined
    ):
        stages.append("external_geometry_validation")
    material_fields = [
        asset.get("uv_status", ""),
        asset.get("material_channels_present", ""),
        asset.get("texture_resolutions", ""),
        asset.get("required_art_pass", ""),
    ]
    if "uv" in joined or "material" in joined or "texture" in joined or "tangent" in joined or any("missing" in str(field).lower() or "unverified" in str(field).lower() for field in material_fields):
        stages.append("material_texture_uv_completion")
    if rig_truth_contact_stage_needed(asset, blockers):
        stages.append("rig_truth_contact_profile_validation")
    if "capture" in joined or "renderer" in joined or "engine" in joined or not manifest.get("in_engine_visual_ready"):
        stages.append("native_renderer_capture_matrix")
    stages.append("owner_visual_acceptance")
    deduped = []
    for stage in stages:
        if stage not in deduped:
            deduped.append(stage)
    return deduped

assets_by_name = {str(asset.get("asset_name", "")): asset for asset in assets}
stage_counts = {stage["id"]: 0 for stage in required_unblock_stages}
per_asset_unblock_plan = []
for item in quarantined:
    asset = assets_by_name.get(str(item.get("asset_name", "")), {})
    stages = stages_for_asset(asset, item)
    for stage in stages:
        stage_counts[stage] = stage_counts.get(stage, 0) + 1
    per_asset_unblock_plan.append(
        {
            "asset_name": item.get("asset_name", ""),
            "kind": item.get("kind", ""),
            "acceptance_status": item.get("acceptance_status", ""),
            "runtime_export": item.get("runtime_export", ""),
            "source_file": item.get("source_file", ""),
            "source_authoring_evidence": asset.get("source_authoring_evidence", {}),
            "blocking_reasons": item.get("quarantine_reasons", []),
            "required_unblock_stages": stages,
            "next_smallest_unblock_step": stages[0] if stages else "none",
            "production_ready_after_this_matrix": False,
        }
    )

unblock_matrix = {
    "schema": "oathyard.generated_asset_production_unblock_matrix.v1",
    "tool": "tools/audit_generated_assets.sh",
    "source_audit_manifest": "generated_asset_audit.json",
    "source_quarantine_manifest": "generated_asset_quarantine_manifest.json",
    "production_asset_ready": False,
    "owner_visual_accepted": False,
    "public_demo_visual_ready": False,
    "release_candidate_ready": False,
    "required_unblock_stage_count": len(required_unblock_stages),
    "required_unblock_stages": required_unblock_stages,
    "asset_count": manifest["asset_count"],
    "quarantined_asset_count": len(quarantined),
    "blocked_asset_count": len(per_asset_unblock_plan),
    "stage_blocker_counts": stage_counts,
    "per_asset_unblock_plan": per_asset_unblock_plan,
}
(out / "generated_asset_production_unblock_matrix.json").write_text(
    json.dumps(unblock_matrix, indent=2, sort_keys=True) + "\n", encoding="utf-8"
)

matrix_lines = [
    "# OATHYARD Generated Asset Production Unblock Matrix",
    "",
    "Status: FAIL-CLOSED" if per_asset_unblock_plan or audit_rc != 0 else "Status: PASSED",
    "",
    "This matrix is evidence routing only. It does not promote generated/model-candidate assets to production.",
    f"- Required production unblock stages: `{len(required_unblock_stages)}`",
    f"- Audited asset count: `{manifest['asset_count']}`",
    f"- Blocked asset count: `{len(per_asset_unblock_plan)}`",
    "- Production asset ready: `false`",
    "- Owner visual accepted: `false`",
    "- Public demo visual ready: `false`",
    "- Release candidate ready: `false`",
    "",
    "## Stage blocker counts",
]
for stage in required_unblock_stages:
    matrix_lines.append(f"- `{stage['id']}`: `{stage_counts.get(stage['id'], 0)}`")
if per_asset_unblock_plan:
    matrix_lines.extend(["", "## Per-asset next unblock step"])
    for item in per_asset_unblock_plan:
        matrix_lines.append(
            f"- `{item['asset_name']}` `{item['kind']}`: `{item['next_smallest_unblock_step']}`"
        )
(out / "generated_asset_production_unblock_matrix.md").write_text(
    "\n".join(matrix_lines) + "\n", encoding="utf-8"
)

state_counts = audit.get("asset_state_counts", {}) if isinstance(audit.get("asset_state_counts", {}), dict) else {}
facet_counts = audit.get("state_facet_counts", {}) if isinstance(audit.get("state_facet_counts", {}), dict) else {}
summary_lines = [
    "# OATHYARD Generated Asset State Summary",
    "",
    "Status: FAIL-CLOSED" if quarantined or audit_rc != 0 else "Status: PASSED",
    "",
    f"- audited_assets: `{manifest['asset_count']}`",
    f"- quarantined_assets: `{len(quarantined)}`",
    f"- production_ready: `{facet_counts.get('production_ready', sum(1 for asset in assets if asset.get('production_ready') is True))}`",
    f"- candidate_only: `{facet_counts.get('candidate_only', sum(1 for asset in assets if asset.get('candidate_only') is True))}`",
    f"- license_pending: `{facet_counts.get('license_pending', sum(1 for asset in assets if asset.get('asset_state') == 'license-pending'))}`",
    f"- native_production_capture: `{facet_counts.get('native_production_capture', 0)}`",
    "",
    "## Acceptance status counts",
]
for status, count in sorted((audit.get("acceptance_status_counts", {}) or {}).items()):
    summary_lines.append(f"- {status}: `{count}`")
summary_lines.extend(["", "## Asset state counts"])
for state, count in sorted(state_counts.items()):
    summary_lines.append(f"- {state}: `{count}`")
(out / "asset_state_summary.md").write_text("\n".join(summary_lines) + "\n", encoding="utf-8")

blocked_lines = [
    "# OATHYARD Blocked Generated Asset Evidence",
    "",
    "Status: FAIL-CLOSED" if per_asset_unblock_plan or audit_rc != 0 else "Status: PASSED",
    "",
    "Missing evidence is listed per asset. This report does not promote generated/model-candidate assets to production.",
]
if per_asset_unblock_plan:
    for item in per_asset_unblock_plan:
        reasons = item.get("blocking_reasons", [])
        blocked_lines.extend([
            "",
            f"## `{item.get('asset_name', '')}` `{item.get('kind', '')}`",
            f"- acceptance_state: `{item.get('acceptance_status', '')}`",
            f"- next_smallest_unblock_step: `{item.get('next_smallest_unblock_step', '')}`",
            f"- blockers: `{', '.join(str(reason) for reason in reasons) if reasons else 'none'}`",
        ])
        if "license_or_project_license_pending" in reasons:
            blocked_lines.append("- missing evidence: license_or_project_license_pending / commercial-use clearance not recorded")
        if "capture_backend_is_software_candidate_not_production_engine" in reasons or "native_renderer_capture_matrix" in item.get("required_unblock_stages", []):
            blocked_lines.append("- missing evidence: native production renderer capture evidence missing")
        blocked_lines.append("- production_ready_after_this_report: `false`")
(out / "blocked_asset_evidence.md").write_text("\n".join(blocked_lines) + "\n", encoding="utf-8")
PY

latest="artifacts/asset_audit/latest"
if [[ "$out" != "$latest" ]]; then
  mkdir -p "$latest"
  for artifact in \
    generated_asset_audit.json generated_asset_audit.md generated_asset_audit.csv \
    rodin_asset_audit.json rodin_asset_audit.md rodin_asset_audit.csv \
    generated_asset_quarantine_manifest.json generated_asset_quarantine_report.md \
    generated_asset_production_unblock_matrix.json generated_asset_production_unblock_matrix.md \
    blocked_asset_evidence.md asset_state_summary.md; do
    copy_if_present "$out/$artifact" "$latest/$artifact"
  done
fi

printf 'generated asset audit: %s rc=%s\n' "$out" "$rc"
exit "$rc"
