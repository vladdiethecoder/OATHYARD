#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/asset_audit/latest}"
mkdir -p "$out"

python3 - "$out" <<'PY'
import csv
import datetime as dt
import hashlib
import json
import os
import struct
import sys
from pathlib import Path

ROOT = Path.cwd()
OUT = Path(sys.argv[1])
OUT.mkdir(parents=True, exist_ok=True)

CANONICAL_TRUTH_JOINTS = [
    "root", "spine_lower", "spine_upper", "neck_head",
    "shoulder_r", "elbow_r", "wrist_r", "shoulder_l", "elbow_l", "wrist_l",
    "hip_r", "knee_r", "ankle_r", "hip_l", "knee_l", "ankle_l",
]

IMG_EXTS = {".png", ".jpg", ".jpeg", ".webp"}
MODEL_EXTS = {".gltf", ".glb", ".obj", ".fbx", ".usd", ".usda", ".usdc", ".stl", ".blend"}


def rel(path: Path) -> str:
    try:
        return path.relative_to(ROOT).as_posix()
    except ValueError:
        return path.as_posix()


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def mtime_iso(path: Path) -> str:
    try:
        return dt.datetime.fromtimestamp(path.stat().st_mtime, tz=dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    except OSError:
        return ""


def read_json(path: Path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None


def png_size(path: Path):
    try:
        data = path.read_bytes()[:24]
    except OSError:
        return None
    if data.startswith(b"\x89PNG\r\n\x1a\n") and len(data) >= 24:
        return struct.unpack(">II", data[16:24])
    return None


def image_size(path: Path):
    if path.suffix.lower() == ".png":
        return png_size(path)
    return None


def accessor_count(gltf: dict, accessor_index):
    if accessor_index is None:
        return 0
    try:
        return int(gltf.get("accessors", [])[int(accessor_index)].get("count", 0))
    except Exception:
        return 0


def accessor_minmax(gltf: dict, accessor_index):
    try:
        acc = gltf.get("accessors", [])[int(accessor_index)]
        return acc.get("min"), acc.get("max")
    except Exception:
        return None, None


def gltf_metrics(path: Path):
    data = read_json(path)
    if not isinstance(data, dict):
        return {"format": path.suffix.lower().lstrip("."), "parse_error": "not JSON glTF or unreadable"}
    vertices = 0
    triangles = 0
    attrs = set()
    pos_bounds_min = []
    pos_bounds_max = []
    for mesh in data.get("meshes", []) or []:
        for prim in mesh.get("primitives", []) or []:
            attributes = prim.get("attributes", {}) or {}
            attrs.update(attributes.keys())
            pos_i = attributes.get("POSITION")
            vertices += accessor_count(data, pos_i)
            mn, mx = accessor_minmax(data, pos_i)
            if isinstance(mn, list) and isinstance(mx, list) and len(mn) >= 3 and len(mx) >= 3:
                pos_bounds_min.append(mn[:3])
                pos_bounds_max.append(mx[:3])
            mode = int(prim.get("mode", 4))
            idx_count = accessor_count(data, prim.get("indices"))
            if mode == 4:
                triangles += idx_count // 3 if idx_count else accessor_count(data, pos_i) // 3
            elif mode == 5:
                triangles += max(0, (idx_count if idx_count else accessor_count(data, pos_i)) - 2)
            elif mode == 6:
                triangles += max(0, (idx_count if idx_count else accessor_count(data, pos_i)) - 2)
    bounds_min = None
    bounds_max = None
    z_depth = None
    if pos_bounds_min and pos_bounds_max:
        bounds_min = [min(v[i] for v in pos_bounds_min) for i in range(3)]
        bounds_max = [max(v[i] for v in pos_bounds_max) for i in range(3)]
        z_depth = bounds_max[2] - bounds_min[2]
    images = data.get("images", []) or []
    textures = data.get("textures", []) or []
    materials = data.get("materials", []) or []
    mat_channels = set()
    for mat in materials:
        pbr = mat.get("pbrMetallicRoughness", {}) or {}
        if pbr.get("baseColorTexture"):
            mat_channels.add("base_color")
        if pbr.get("metallicRoughnessTexture"):
            mat_channels.add("metallic_roughness")
        if mat.get("normalTexture"):
            mat_channels.add("normal")
        if mat.get("occlusionTexture"):
            mat_channels.add("occlusion")
        if mat.get("emissiveTexture"):
            mat_channels.add("emissive")
    return {
        "format": "gltf",
        "asset_version": (data.get("asset") or {}).get("version", ""),
        "vertices": vertices,
        "triangles": triangles,
        "primitive_count": sum(len((m.get("primitives") or [])) for m in (data.get("meshes", []) or [])),
        "material_count": len(materials),
        "texture_count": len(textures) or len(images),
        "image_uris": [str(img.get("uri", "")) for img in images],
        "material_channels": sorted(mat_channels),
        "attributes": sorted(attrs),
        "uv_status": "present" if "TEXCOORD_0" in attrs else "missing",
        "normals_status": "present" if "NORMAL" in attrs else "missing",
        "tangents_status": "present" if "TANGENT" in attrs else "missing",
        "rig_status": "present" if data.get("skins") else "not_applicable_or_missing",
        "skin_weight_status": "present" if {"JOINTS_0", "WEIGHTS_0"}.issubset(attrs) else "not_applicable_or_missing",
        "animation_clip_count": len(data.get("animations", []) or []),
        "bounds_min": bounds_min,
        "bounds_max": bounds_max,
        "z_depth": z_depth,
    }


def texture_resolutions(root: Path, image_uris):
    result = []
    for uri in image_uris or []:
        p = (root / uri).resolve()
        if not p.is_file():
            p = (ROOT / uri).resolve()
        item = {"path": rel(p) if p.exists() else uri, "resolution": "unknown", "sha256": ""}
        if p.is_file():
            size = image_size(p)
            if size:
                item["resolution"] = f"{size[0]}x{size[1]}"
            item["sha256"] = sha256(p)
        result.append(item)
    return result


def collect_candidate_entries():
    entries_by_id = {}
    sources = []

    # Preferred split candidate manifest if present.
    candidate_manifest = ROOT / "assets" / "production_candidate_visual_manifest.json"
    if candidate_manifest.is_file():
        data = read_json(candidate_manifest)
        if isinstance(data, dict):
            sources.append(rel(candidate_manifest))
            for e in data.get("entries", []) or []:
                if e.get("id"):
                    entries_by_id[e["id"]] = dict(e, _manifest=rel(candidate_manifest))

    # Existing combined presentation/production-candidate manifest.
    for manifest_path in [ROOT / "assets" / "production_visual_manifest.json", ROOT / "assets" / "presentation_manifest.json"]:
        if manifest_path.is_file():
            data = read_json(manifest_path)
            if isinstance(data, dict):
                sources.append(rel(manifest_path))
                for e in data.get("entries", []) or []:
                    if e.get("id") and e.get("id") not in entries_by_id:
                        entries_by_id[e["id"]] = dict(e, _manifest=rel(manifest_path))

    # Raw model-candidate manifest.
    for path in sorted((ROOT / "assets" / "model_candidates").glob("*/model_candidate_manifest.json")):
        data = read_json(path)
        if isinstance(data, dict):
            sources.append(rel(path))
            for e in data.get("entries", []) or []:
                if e.get("id") and e.get("id") not in entries_by_id:
                    entries_by_id[e["id"]] = dict(e, _manifest=rel(path))

    return list(entries_by_id.values()), sorted(set(sources))


def source_summary(source_path: Path):
    data = read_json(source_path) if source_path.is_file() else None
    if not isinstance(data, dict):
        return {
            "source_prompt_or_image": "missing_source_json",
            "source_basis": {},
            "provenance": "missing",
            "license_status": "missing",
            "not_claimed": [],
            "truth_boundary": {},
        }
    basis = data.get("source_basis", {}) or {}
    cat = basis.get("category_source_fields", {}) or {}
    prompt_parts = []
    if data.get("prompt"):
        prompt_parts.append(str(data.get("prompt")))
    if data.get("source_image"):
        prompt_parts.append(str(data.get("source_image")))
    if cat:
        prompt_parts.append("category_fields=" + json.dumps(cat, sort_keys=True))
    return {
        "source_prompt_or_image": "; ".join(prompt_parts) if prompt_parts else "not recorded as Rodin prompt/image; repo source fields only",
        "source_basis": basis,
        "provenance": data.get("provenance", ""),
        "license_status": data.get("license_status", ""),
        "not_claimed": data.get("not_claimed", []),
        "truth_boundary": data.get("truth_boundary", {}),
    }


def classify_acceptance(entry: dict, source: dict, metrics: dict, production_manifest_contains_candidate: bool):
    blockers = []
    license_status = str(entry.get("license_status") or (entry.get("provenance_license") or {}).get("license_status") or source.get("license_status") or "unknown")
    provenance = str(entry.get("provenance") or (entry.get("provenance_license") or {}).get("provenance") or source.get("provenance") or "unknown")
    authoring = entry.get("authoring_process", {}) or {}
    if "pending" in license_status.lower() or license_status in {"", "unknown"}:
        blockers.append("license_or_project_license_pending")
    if "rodin" in provenance.lower() and not entry.get("rodin_task_uuid"):
        blockers.append("rodin_task_download_terms_receipt_missing")
    if "procedural" in provenance.lower() or "procedural" in str(authoring.get("method", "")).lower():
        blockers.append("procedural_model_candidate_not_dcc_source")
    if authoring and authoring.get("external_dcc_validation_claimed") is not True:
        blockers.append("external_dcc_validation_missing")
    if authoring and authoring.get("external_khronos_validation_claimed") is not True:
        blockers.append("external_khronos_gltf_validation_missing")
    source_file = str(entry.get("source_file") or entry.get("source") or "")
    if Path(source_file).suffix.lower() not in {".blend", ".usd", ".usda", ".usdc", ".fbx"}:
        blockers.append("source_not_dcc_or_interchange_file")
    if not metrics.get("vertices") or not metrics.get("triangles"):
        blockers.append("mesh_metric_missing")
    if metrics.get("uv_status") != "present":
        blockers.append("uv_missing")
    if metrics.get("normals_status") != "present":
        blockers.append("normals_missing")
    if "tangent" not in str(metrics.get("tangents_status", "")) or metrics.get("tangents_status") == "missing":
        blockers.append("tangents_missing_or_unverified")
    if not metrics.get("material_channels"):
        blockers.append("material_channels_missing")
    if entry.get("kind") in {"fighters", "fighter"}:
        rig = entry.get("rig") or entry.get("rig_validation") or (entry.get("validation_result", {}) or {}).get("rig_validation") or {}
        if not rig or rig.get("passed") is not True:
            blockers.append("fighter_rig_validation_missing")
        if metrics.get("skin_weight_status") != "present":
            blockers.append("fighter_skin_weights_missing")
        truth_map = entry.get("truth_joint_mapping") or {}
        if not truth_map:
            blockers.append("truth_joint_mapping_missing")
    contact = entry.get("contact_profile") or (entry.get("validation_result", {}) or {}).get("contact_profile")
    if not contact:
        blockers.append("contact_physics_profile_missing")
    screenshot = entry.get("in_engine_screenshot") or entry.get("in_engine_evidence") or {}
    backend = str(screenshot.get("backend", ""))
    if not screenshot:
        blockers.append("in_engine_screenshot_missing")
    elif "software" in backend or "deterministic_software" in backend:
        blockers.append("capture_backend_is_software_candidate_not_production_engine")
    if production_manifest_contains_candidate:
        blockers.append("candidate_entry_present_in_production_visual_manifest")
    if blockers:
        if "license_or_project_license_pending" in blockers:
            status = "candidate"
        elif any(b.endswith("missing") or "not_dcc" in b for b in blockers):
            status = "candidate"
        else:
            status = "source-approved"
    else:
        status = "production-ready"
    return status, blockers, license_status, provenance


def rodin_file_inventory():
    hits = []
    for base in [ROOT / "assets_src", ROOT / "assets", ROOT / "artifacts", ROOT / "external"]:
        if not base.exists():
            continue
        for p in base.rglob("*"):
            if not p.is_file():
                continue
            s = p.as_posix().lower()
            if "rodin" in s or "hyper3d" in s or "deemos" in s:
                hits.append({"path": rel(p), "suffix": p.suffix.lower(), "size": p.stat().st_size, "sha256": sha256(p)})
    return hits

entries, manifest_sources = collect_candidate_entries()
rodin_files = rodin_file_inventory()
production_manifest = ROOT / "assets" / "production_visual_manifest.json"
production_manifest_data = read_json(production_manifest) if production_manifest.is_file() else {}
production_manifest_contains_candidate = False
if isinstance(production_manifest_data, dict):
    for e in production_manifest_data.get("entries", []) or []:
        auth = e.get("authoring_process", {}) or {}
        prov = (e.get("provenance_license", {}) or {}).get("provenance", "")
        lic = (e.get("provenance_license", {}) or {}).get("license_status", "")
        if "candidate" in str(prov).lower() or "candidate" in str(lic).lower() or "procedural" in str(auth.get("method", "")).lower():
            production_manifest_contains_candidate = True
            break

records = []
for entry in entries:
    asset_id = entry.get("id", "<unknown>")
    kind = entry.get("kind") or entry.get("category") or entry.get("candidate_kind") or "unknown"
    source_file = entry.get("source_file") or entry.get("source") or ""
    source_path = ROOT / source_file if source_file else Path("")
    source = source_summary(source_path)
    runtime = (entry.get("runtime_export") or {}).get("source_candidate_gltf") or (entry.get("runtime_export") or {}).get("runtime_gltf") or entry.get("runtime_gltf") or ""
    runtime_path = ROOT / runtime if runtime else Path("")
    metrics = gltf_metrics(runtime_path) if runtime_path.is_file() and runtime_path.suffix.lower() == ".gltf" else {}
    # Prefer manifest metrics when present because candidate manifests were already audited.
    for k_src, k_dst in [("vertices", "vertices"), ("triangles", "triangles"), ("materials", "material_count"), ("primitives", "primitive_count")]:
        if entry.get(k_src) is not None and not metrics.get(k_dst):
            metrics[k_dst] = entry.get(k_src)
    tex_res = texture_resolutions(runtime_path.parent if runtime_path else ROOT, metrics.get("image_uris", []))
    status, blockers, license_status, provenance = classify_acceptance(entry, source, metrics, production_manifest_contains_candidate)
    commercial = "unverified"
    if "unlimited export" in license_status.lower() or "commercial" in license_status.lower():
        commercial = "unverified_from_local_artifact"
    protected_ip_risk = "low_known_source_risk_but_owner_legal_review_pending" if "repo_owned" in provenance else "unverified"
    export_date = mtime_iso(runtime_path) if runtime_path.is_file() else ""
    record = {
        "asset_name": asset_id,
        "kind": kind,
        "source_prompt_or_image_reference": source["source_prompt_or_image"],
        "generation_tool_and_version": entry.get("toolchain") or (entry.get("authoring_process", {}) or {}).get("toolchain") or "repo procedural/model-candidate tools; Rodin receipt not found",
        "export_format": metrics.get("format") or (runtime_path.suffix.lower().lstrip(".") if runtime else "missing"),
        "export_date": export_date,
        "license_terms_status": license_status,
        "commercial_use_allowed": commercial,
        "third_party_protected_ip_risk": protected_ip_risk,
        "polygon_count_triangles": metrics.get("triangles", 0),
        "vertex_count": metrics.get("vertices", 0),
        "texture_count": len(tex_res) or metrics.get("texture_count", 0),
        "texture_resolutions": tex_res,
        "material_channels_present": metrics.get("material_channels", []),
        "uv_status": metrics.get("uv_status", "unknown"),
        "rig_status": metrics.get("rig_status", "unknown"),
        "skin_weight_status": metrics.get("skin_weight_status", "unknown"),
        "topology_issues": "external topology/manifold validation not performed" if metrics else "missing runtime mesh parse",
        "manifold_watertight_status": "unverified",
        "normals_status": metrics.get("normals_status", "unknown"),
        "tangents_status": metrics.get("tangents_status", "unknown"),
        "scale_orientation": {"bounds_min": metrics.get("bounds_min"), "bounds_max": metrics.get("bounds_max"), "z_depth": metrics.get("z_depth")},
        "canonical_truth_joint_mapping_status": entry.get("truth_joint_mapping") or source.get("truth_boundary") or "unverified",
        "contact_physics_profile_status": entry.get("contact_profile") or (entry.get("validation_result", {}) or {}).get("contact_profile") or "missing",
        "runtime_suitability": "candidate_runtime_presentation_only" if status != "production-ready" else "production_runtime_candidate",
        "required_art_pass": "DCC/source approval, sculpt/retopo/UV/material polish, native renderer visual review",
        "required_technical_pass": "license/terms proof, glTF validator, topology/manifold check, rig/skin/contact profile validation, production renderer load, truth isolation",
        "acceptance_status": status,
        "acceptance_blockers": blockers,
        "source_file": source_file,
        "runtime_export": runtime,
        "manifest_source": entry.get("_manifest", ""),
    }
    records.append(record)

records.sort(key=lambda r: (str(r["kind"]), str(r["asset_name"])))

failures = []
if not records:
    failures.append("no generated/imported/model-candidate assets found to audit")
rodin_export_files = [f for f in rodin_files if f["suffix"] in MODEL_EXTS]
if not rodin_export_files:
    failures.append("no completed local Rodin model export files with Rodin path/name were found")
if production_manifest_contains_candidate:
    failures.append("candidate-only entries are present in assets/production_visual_manifest.json; move them to a candidate manifest")
not_prod = [r for r in records if r["acceptance_status"] != "production-ready"]
if not_prod:
    failures.append(f"{len(not_prod)} audited assets are not production-ready")
license_pending = [r for r in records if "pending" in str(r["license_terms_status"]).lower() or r["commercial_use_allowed"].startswith("unverified")]
if license_pending:
    failures.append(f"{len(license_pending)} audited assets have pending/unverified license or commercial-use status")

counts = {}
for r in records:
    counts[r["kind"]] = counts.get(r["kind"], 0) + 1
status_counts = {}
for r in records:
    status_counts[r["acceptance_status"]] = status_counts.get(r["acceptance_status"], 0) + 1

manifest = {
    "schema": "oathyard.rodin_asset_audit.v1",
    "tool": "tools/audit_rodin_assets.sh",
    "generated_at_utc": dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "passed": not failures,
    "baseline_status": "V0.5 candidate_asset_preview",
    "production_asset_ready": False,
    "in_engine_visual_ready": False,
    "high_fidelity_ready": False,
    "public_demo_visual_ready": False,
    "owner_visual_accepted": False,
    "manifest_sources": manifest_sources,
    "rodin_related_file_count": len(rodin_files),
    "rodin_model_export_count": len(rodin_export_files),
    "asset_count": len(records),
    "kind_counts": counts,
    "acceptance_status_counts": status_counts,
    "failed_check_count": len(failures),
    "failures": failures,
    "rodin_related_files_sample": rodin_files[:80],
    "assets": records,
}
(OUT / "rodin_asset_audit.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")

csv_fields = [
    "asset_name", "kind", "source_prompt_or_image_reference", "export_format", "export_date",
    "license_terms_status", "commercial_use_allowed", "third_party_protected_ip_risk",
    "polygon_count_triangles", "vertex_count", "texture_count", "uv_status", "rig_status",
    "skin_weight_status", "normals_status", "tangents_status", "manifold_watertight_status",
    "runtime_suitability", "acceptance_status", "source_file", "runtime_export", "manifest_source",
]
with (OUT / "rodin_asset_audit.csv").open("w", newline="", encoding="utf-8") as f:
    writer = csv.DictWriter(f, fieldnames=csv_fields)
    writer.writeheader()
    for r in records:
        writer.writerow({k: r.get(k, "") for k in csv_fields})

lines = [
    "# OATHYARD Rodin / Generated Asset Audit",
    "",
    f"Status: {'PASSED' if not failures else 'FAILED'}",
    "Evidence class: V0.5 candidate asset evidence unless explicitly stated otherwise.",
    "",
    "Readiness flags:",
    "",
    "- `candidate_asset_preview`: `true` for audited candidates with preview evidence",
    "- `production_asset_ready`: `false`",
    "- `in_engine_visual_ready`: `false`",
    "- `high_fidelity_ready`: `false`",
    "- `public_demo_visual_ready`: `false`",
    "- `owner_visual_accepted`: `false`",
    "",
    "## Summary",
    "",
    f"- Asset count audited: `{len(records)}`",
    f"- Kind counts: `{json.dumps(counts, sort_keys=True)}`",
    f"- Acceptance status counts: `{json.dumps(status_counts, sort_keys=True)}`",
    f"- Rodin-related files found: `{len(rodin_files)}`",
    f"- Rodin model exports found: `{len(rodin_export_files)}`",
    f"- Manifest sources: `{', '.join(manifest_sources) if manifest_sources else 'none'}`",
    "",
]
if failures:
    lines.extend(["## Failures / blockers", ""])
    lines.extend(f"- {f}" for f in failures)
    lines.append("")
lines.extend([
    "## Per-asset audit",
    "",
    "| Asset | Kind | Status | Tris | Verts | Textures | UV | Rig | Skin | Normals | Tangents | License | Runtime | Blockers |",
    "| --- | --- | --- | ---: | ---: | ---: | --- | --- | --- | --- | --- | --- | --- | --- |",
])
for r in records:
    blockers = ", ".join(r["acceptance_blockers"][:6])
    if len(r["acceptance_blockers"]) > 6:
        blockers += ", ..."
    lines.append(
        f"| `{r['asset_name']}` | `{r['kind']}` | `{r['acceptance_status']}` | "
        f"{r['polygon_count_triangles']} | {r['vertex_count']} | {r['texture_count']} | "
        f"`{r['uv_status']}` | `{r['rig_status']}` | `{r['skin_weight_status']}` | "
        f"`{r['normals_status']}` | `{r['tangents_status']}` | `{r['license_terms_status']}` | "
        f"`{r['runtime_export']}` | {blockers} |"
    )
lines.extend([
    "",
    "## Rodin/license conclusion",
    "",
    "No asset is accepted for shipping solely because it appears in a Rodin/model-candidate/contact-sheet preview. Current local evidence lacks a complete Rodin terms/account/task/download receipt packet. Commercial-use status remains unverified unless a generation-time plan/terms snapshot and owner/legal review are recorded.",
])
(OUT / "rodin_asset_audit.md").write_text("\n".join(lines) + "\n", encoding="utf-8")

if failures:
    raise SystemExit(1)
PY

echo "rodin asset audit: $out"
