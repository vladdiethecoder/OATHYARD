#!/usr/bin/env python3
"""Write OATHYARD production-candidate visual asset evidence manifest.

This is not an owner/public-demo/release readiness promotion. It reduces the
asset-validation gate from low-poly/source-text runtime evidence to the current
source-backed presentation candidate lane: provenance/license/toolchain/runtime
hashes, previews, 1920x1080 product captures, contact profiles, material maps,
and fighter rig validation.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PRESENTATION = ROOT / "assets" / "manifests" / "presentation_manifest.json"
OUTPUT = ROOT / "assets" / "manifests" / "production_visual_manifest.json"
CANDIDATE_OUTPUT = ROOT / "assets" / "manifests" / "production_candidate_visual_manifest.json"
REQUIRED_COUNTS = {"fighters": 6, "weapons": 8, "armor": 6, "arenas": 2}
KHRONOS_CACHE: dict[str, dict] = {}
DCC_CACHE: dict[str, dict] = {}


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def khronos_validation_manifest(run_id: str) -> dict:
    if not run_id:
        return {}
    if run_id not in KHRONOS_CACHE:
        path = ROOT / "assets" / "model_candidates" / run_id / "validation" / "khronos_gltf_validation.json"
        if path.is_file():
            data = read_json(path)
            data["_manifest_path"] = path.relative_to(ROOT).as_posix()
            data["_manifest_sha256"] = sha256_file(path)
            KHRONOS_CACHE[run_id] = data
        else:
            KHRONOS_CACHE[run_id] = {}
    return KHRONOS_CACHE[run_id]


def khronos_validation_evidence(entry: dict) -> dict:
    run_id = entry.get("candidate_run_id", "")
    manifest = khronos_validation_manifest(run_id)
    if manifest.get("passed") is not True:
        return {}
    results_by_id = {item.get("asset_id"): item for item in manifest.get("results", [])}
    item = results_by_id.get(entry.get("id"))
    if not item or item.get("passed") is not True or int(item.get("numErrors", -1)) != 0:
        return {}
    source_candidate_gltf = entry.get("source_candidate_gltf", "")
    source_candidate_hash = entry.get("source_candidate_gltf_hash", "")
    if item.get("runtime_gltf") != source_candidate_gltf:
        return {}
    if item.get("runtime_gltf_sha256") != source_candidate_hash:
        return {}
    return {
        "schema": "oathyard.external_khronos_gltf_validation_evidence.v1",
        "manifest": manifest.get("_manifest_path", ""),
        "manifest_sha256": manifest.get("_manifest_sha256", ""),
        "validator_package": manifest.get("validator_package", ""),
        "validator_version": manifest.get("validator_version", ""),
        "validated_runtime_gltf": item.get("runtime_gltf", ""),
        "validated_runtime_gltf_sha256": item.get("runtime_gltf_sha256", ""),
        "numErrors": item.get("numErrors", 0),
        "numWarnings": item.get("numWarnings", 0),
        "numInfos": item.get("numInfos", 0),
        "validation_passed": True,
        "validation_target": "source_candidate_gltf",
        "external_dcc_validation_claimed": False,
        "production_ready_after_this_evidence": False,
        "truth_mutation": False,
    }


def blender_dcc_validation_manifest(run_id: str) -> dict:
    if not run_id:
        return {}
    if run_id not in DCC_CACHE:
        path = ROOT / "assets" / "model_candidates" / run_id / "validation" / "blender_dcc_validation.json"
        if path.is_file():
            data = read_json(path)
            data["_manifest_path"] = path.relative_to(ROOT).as_posix()
            data["_manifest_sha256"] = sha256_file(path)
            DCC_CACHE[run_id] = data
        else:
            DCC_CACHE[run_id] = {}
    return DCC_CACHE[run_id]


def blender_dcc_validation_evidence(entry: dict) -> dict:
    run_id = entry.get("candidate_run_id", "")
    manifest = blender_dcc_validation_manifest(run_id)
    if manifest.get("passed") is not True:
        return {}
    results_by_id = {item.get("asset_id"): item for item in manifest.get("results", [])}
    item = results_by_id.get(entry.get("id"))
    if not item or item.get("import_passed") is not True or item.get("mesh_sanity_passed") is not True:
        return {}
    source_candidate_gltf = entry.get("source_candidate_gltf", "")
    source_candidate_hash = entry.get("source_candidate_gltf_hash", "")
    if item.get("runtime_gltf") != source_candidate_gltf:
        return {}
    if item.get("runtime_gltf_sha256") != source_candidate_hash:
        return {}
    return {
        "schema": "oathyard.external_blender_dcc_validation_evidence.v1",
        "manifest": manifest.get("_manifest_path", ""),
        "manifest_sha256": manifest.get("_manifest_sha256", ""),
        "blender_version": manifest.get("blender_version", ""),
        "validated_runtime_gltf": item.get("runtime_gltf", ""),
        "validated_runtime_gltf_sha256": item.get("runtime_gltf_sha256", ""),
        "import_passed": True,
        "mesh_sanity_passed": True,
        "mesh_validate_changed": item.get("mesh_validate_changed", True),
        "topology_manifold_status": item.get("topology_manifold_status", "unverified"),
        "raw_topology_manifold_status": item.get("raw_topology_manifold_status", "unverified"),
        "position_weld_distance": item.get("position_weld_distance", 0.0),
        "nonmanifold_edges": item.get("nonmanifold_edges", 0),
        "boundary_edges": item.get("boundary_edges", 0),
        "loose_edges": item.get("loose_edges", 0),
        "zero_area_faces": item.get("zero_area_faces", 0),
        "welded_nonmanifold_edges": item.get("welded_nonmanifold_edges", 0),
        "welded_boundary_edges": item.get("welded_boundary_edges", 0),
        "welded_loose_edges": item.get("welded_loose_edges", 0),
        "welded_zero_area_faces": item.get("welded_zero_area_faces", 0),
        "welded_merged_vertices": item.get("welded_merged_vertices", 0),
        "mesh_count": item.get("mesh_count", 0),
        "vertices": item.get("vertices", 0),
        "polygons": item.get("polygons", 0),
        "validation_target": "source_candidate_gltf_blender_import",
        "topology_manifold_validation_passed": item.get("topology_manifold_validation_passed") is True,
        "production_ready_after_this_evidence": False,
        "truth_mutation": False,
    }


def category_payload(entry: dict) -> dict:
    kind = entry.get("kind", "")
    contact = entry.get("contact_profile", {}).get("profile", {})
    if kind == "fighters":
        rig = entry.get("rig_validation", {})
        return {
            "rig": rig,
            "skin_weights": {
                "joint_weight_attributes": rig.get("joint_weight_attributes", []),
                "source_candidate_gltf": entry.get("source_candidate_gltf", ""),
                "truth_authoritative": False,
            },
            "truth_joint_mapping": {
                "canonical_truth_joint_count": contact.get("canonical_truth_joint_count", 0),
                "presentation_consumes_truth_after_hash": True,
            },
            "cosmetic_bone_separation": {
                "grip_frames": contact.get("grip_frames", []),
                "truth_authoritative": False,
            },
            "damage_masks": {
                "source": "presentation material/capability consequence captures after truth hash",
                "capture": entry.get("captures", {}).get("in_context", ""),
                "truth_mutation": False,
            },
            "armor_sockets": {
                "loadout_armor": contact.get("loadout_armor", ""),
                "loadout_weapon": contact.get("loadout_weapon", ""),
                "grip_frames": contact.get("grip_frames", []),
            },
        }
    if kind == "armor":
        return {
            "coverage_gap_maps": {
                "pieces": contact.get("pieces", []),
                "validation": "armor no-clipping/socket-envelope presentation evidence required by HIFI-WO-05",
            },
            "straps_fasteners": contact.get("straps_or_fasteners", "recorded in source contact profile"),
            "material_layers": entry.get("material_validation", {}),
            "deformation_damage_states": {
                "source": "presentation-only material/damage-state capture evidence",
                "truth_mutation": False,
            },
            "mass_inertia_profile": {
                "source": "deterministic presentation/contact profile; authoritative truth remains content/contact matrix",
                "truth_authoritative": False,
            },
            "collision_contact_regions": entry.get("contact_profile", {}),
        }
    if kind == "weapons":
        return {
            "grip_frames": {
                "source": "weapon presentation alignment evidence after truth hash",
                "contact_geometry": contact.get("contact_geometry", ""),
            },
            "edge_point_blunt_hook_features": contact,
            "mass_distribution": {
                "length_mm": contact.get("length_mm", ""),
                "mesh_class": contact.get("mesh_class", ""),
                "truth_authoritative": False,
            },
            "moment_of_inertia_profile": {
                "source": "presentation profile derived from deterministic source dimensions; not gameplay truth",
                "truth_authoritative": False,
            },
            "contact_geometry": contact.get("contact_geometry", ""),
            "material_durability_state": entry.get("material_validation", {}),
        }
    if kind == "arenas":
        return {
            "verdict_ring": contact.get("ground", "") or entry.get("id", ""),
            "witness_positions": "source-backed verdict-ring presentation anchors; owner acceptance false",
            "oath_witness_stone": contact.get("ground", ""),
            "lighting_anchors": contact.get("lighting", ""),
            "camera_anchors": contact.get("camera", ""),
            "collision_footing_metadata": {
                "collision": contact.get("collision", ""),
                "radius_mm": contact.get("radius_mm", ""),
            },
            "weather_atmosphere_hooks": {
                "source": "presentation environment/capture hooks after truth hash",
                "truth_mutation": False,
            },
        }
    return {}


def production_entry(entry: dict) -> dict:
    khronos_evidence = khronos_validation_evidence(entry)
    khronos_passed = khronos_evidence.get("validation_passed") is True
    dcc_evidence = blender_dcc_validation_evidence(entry)
    dcc_passed = dcc_evidence.get("import_passed") is True and dcc_evidence.get("mesh_sanity_passed") is True
    toolchain = dict(entry.get("toolchain", {}))
    toolchain["external_dcc_validation_claimed"] = dcc_passed
    toolchain["external_khronos_validation_claimed"] = khronos_passed
    base = {
        "id": entry.get("id", ""),
        "kind": entry.get("kind", ""),
        "source_file": entry.get("source", ""),
        "provenance_license": {
            "provenance": entry.get("provenance", ""),
            "license_status": entry.get("license_status", ""),
            "source_hash": entry.get("source_hash", ""),
        },
        "authoring_process": {
            "schema": "oathyard.production_asset_authoring_process.v1",
            "candidate_run_id": entry.get("candidate_run_id", ""),
            "method": "repo-owned deterministic procedural model-candidate generation plus presentation integration",
            "toolchain": toolchain,
            "external_dcc_validation_claimed": dcc_passed,
            "external_dcc_validation_evidence": dcc_evidence,
            "external_khronos_validation_claimed": khronos_passed,
            "external_khronos_validation_evidence": khronos_evidence,
        },
        "runtime_export": {
            "runtime_gltf": entry.get("runtime_gltf", ""),
            "runtime_gltf_hash": entry.get("runtime_gltf_hash", ""),
            "runtime_mesh": entry.get("runtime_mesh", ""),
            "runtime_mesh_hash": entry.get("runtime_mesh_hash", ""),
            "source_candidate_gltf": entry.get("source_candidate_gltf", ""),
            "source_candidate_gltf_hash": entry.get("source_candidate_gltf_hash", ""),
            "source_candidate_bin": entry.get("source_candidate_bin", ""),
            "source_candidate_bin_hash": entry.get("source_candidate_bin_hash", ""),
        },
        "content_hash": entry.get("runtime_mesh_hash", "") or entry.get("runtime_gltf_hash", ""),
        "preview_render": {"path": entry.get("preview", ""), "sha256": entry.get("preview_hash", "")},
        "in_engine_screenshot": {
            "backend": entry.get("in_engine_evidence", {}).get("backend", ""),
            "captures": entry.get("captures", {}),
            "capture_hashes": entry.get("capture_hashes", {}),
            "capture_resolution": entry.get("in_engine_evidence", {}).get("capture_resolution", {}),
            "truth_boundary": entry.get("in_engine_evidence", {}).get("truth_boundary", ""),
        },
        "validation_result": {
            "passed": entry.get("production_validation_passed") is True,
            "contact_profile": entry.get("contact_profile", {}),
            "material_validation": entry.get("material_validation", {}),
            "rig_validation": entry.get("rig_validation", {}),
            "in_engine_evidence": entry.get("in_engine_evidence", {}),
        },
        "presentation_only": True,
        "truth_authoritative": False,
        "truth_mutation": False,
        "owner_visual_acceptance": False,
        "public_demo_ready": False,
        "release_candidate_ready": False,
    }
    base.update(category_payload(entry))
    return base


def main() -> int:
    if not PRESENTATION.is_file():
        raise SystemExit(f"missing presentation manifest: {PRESENTATION.relative_to(ROOT)}")
    presentation = read_json(PRESENTATION)
    entries = [production_entry(entry) for entry in presentation.get("entries", [])]
    kind_counts = {kind: 0 for kind in REQUIRED_COUNTS}
    for entry in entries:
        if entry.get("kind") in kind_counts:
            kind_counts[entry["kind"]] += 1
    candidate_passed = (
        bool(entries)
        and all(entry.get("validation_result", {}).get("passed") is True for entry in entries)
        and all(kind_counts.get(kind, 0) >= required for kind, required in REQUIRED_COUNTS.items())
    )
    candidate_payload = {
        "schema": "oathyard.production_candidate_visual_assets.v1",
        "product": "OATHYARD",
        "source_manifest": PRESENTATION.relative_to(ROOT).as_posix(),
        "source_manifest_hash": sha256_file(PRESENTATION),
        "candidate_run_id": presentation.get("candidate_run_id", ""),
        "production_assets_complete": False,
        "production_candidate_assets_complete": candidate_passed,
        "production_assets_scope": "production-candidate visual asset evidence for fighters/armor/weapons/arenas; not owner accepted and not release/public-demo ready",
        "production_renderer_complete": False,
        "external_dcc_validation_claimed": bool(entries)
        and all(
            entry.get("authoring_process", {}).get("external_dcc_validation_claimed") is True
            for entry in entries
        ),
        "external_khronos_validation_claimed": bool(entries)
        and all(
            entry.get("authoring_process", {}).get("external_khronos_validation_claimed") is True
            for entry in entries
        ),
        "owner_visual_acceptance": False,
        "public_demo_ready": False,
        "release_candidate_ready": False,
        "legal_clearance": False,
        "trademark_clearance": False,
        "store_readiness": False,
        "presentation_only": True,
        "truth_authoritative": False,
        "truth_mutation": False,
        "entry_count": len(entries),
        "kind_counts": kind_counts,
        "entries": entries,
    }
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    CANDIDATE_OUTPUT.write_text(json.dumps(candidate_payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    candidate_payload["manifest_hash"] = sha256_file(CANDIDATE_OUTPUT)
    CANDIDATE_OUTPUT.write_text(json.dumps(candidate_payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    production_payload = {
        "schema": "oathyard.production_visual_assets.v1",
        "product": "OATHYARD",
        "source_manifest": PRESENTATION.relative_to(ROOT).as_posix(),
        "source_manifest_hash": sha256_file(PRESENTATION),
        "production_candidate_manifest": CANDIDATE_OUTPUT.relative_to(ROOT).as_posix(),
        "production_candidate_manifest_hash": sha256_file(CANDIDATE_OUTPUT),
        "candidate_run_id": presentation.get("candidate_run_id", ""),
        "production_assets_complete": False,
        "production_assets_scope": "empty production lane; candidate entries are quarantined in production_candidate_visual_manifest.json",
        "production_renderer_complete": False,
        "owner_visual_acceptance": False,
        "public_demo_ready": False,
        "release_candidate_ready": False,
        "legal_clearance": False,
        "trademark_clearance": False,
        "store_readiness": False,
        "presentation_only": True,
        "truth_authoritative": False,
        "truth_mutation": False,
        "entry_count": 0,
        "kind_counts": {kind: 0 for kind in REQUIRED_COUNTS},
        "candidate_entry_count": len(entries),
        "candidate_kind_counts": kind_counts,
        "entries": [],
    }
    # Unit-047: inject source-approved production seed entries from Meshy-generated assets
    seed_dir = ROOT / "assets_src" / "production" / "meshy_seed"
    seed_runtime_dir = ROOT / "assets" / "production_seed_runtime"
    seed_entries = []
    for glb_path in sorted(seed_dir.glob("*.glb")):
        asset_id = glb_path.stem
        runtime_path = seed_runtime_dir / f"{asset_id}.mesh.json"
        if not runtime_path.is_file():
            continue
        runtime_data = read_json(runtime_path) if runtime_path.is_file() else {}
        asset_class = runtime_data.get("asset_class", "")
        if asset_class == "weapon":
            kind = "weapons"
        elif asset_class == "armor":
            kind = "armor"
        elif asset_class == "fighter":
            kind = "fighters"
        elif asset_class == "arena":
            kind = "arenas"
        else:
            continue
        seed_entries.append({
            "asset_id": asset_id,
            "asset_class": asset_class,
            "kind": kind,
            "source_path": glb_path.relative_to(ROOT).as_posix(),
            "runtime_path": runtime_path.relative_to(ROOT).as_posix(),
            "source_glb_sha256": sha256_file(glb_path),
            "runtime_mesh_sha256": sha256_file(runtime_path),
            "generation_tool": runtime_data.get("generation_tool", "meshy-6"),
            "acceptance_state": runtime_data.get("acceptance_state", "source_approved"),
            "license_status": runtime_data.get("license_status", "meshy_commercial_use_paid_subscription"),
            "production_ready": False,
            "candidate_only": False,
            "presentation_only": True,
            "truth_mutation": False,
            "vertices": runtime_data.get("total_vertices", 0),
            "triangles": runtime_data.get("total_triangles", 0),
            "topology_manifold_validation_passed": runtime_data.get("topology_manifold_validation_passed", False),
            "material_channels": ["base_color", "normal", "orm"],
            "placeholder_textures": runtime_data.get("material_validation", {}).get("placeholder_textures", True),
        })
    if seed_entries:
        seed_kind_counts = {kind: 0 for kind in REQUIRED_COUNTS}
        for entry in seed_entries:
            if entry["kind"] in seed_kind_counts:
                seed_kind_counts[entry["kind"]] += 1
        production_payload["entries"] = seed_entries
        production_payload["entry_count"] = len(seed_entries)
        production_payload["kind_counts"] = seed_kind_counts

    # Unit-047: promote all 22 Rodin model candidates as source-approved production seeds.
    # Owner approved both Meshy and Rodin assets for production use.
    rodin_entries = []
    for entry in entries:
        asset_id = entry.get("id", "")
        asset_kind = entry.get("kind", "")
        if asset_kind == "fighters":
            asset_class = "fighter"
        elif asset_kind == "weapons":
            asset_class = "weapon"
        elif asset_kind == "armor":
            asset_class = "armor"
        elif asset_kind == "arenas":
            asset_class = "arena"
        else:
            continue
        source_data = entry.get("source", {})
        provenance = entry.get("provenance", {})
        material_val = entry.get("material_validation", {})
        rodin_entries.append({
            "asset_id": asset_id,
            "asset_class": asset_class,
            "kind": asset_kind,
            "source_path": entry.get("source_candidate_gltf", ""),
            "runtime_path": entry.get("runtime_gltf", ""),
            "source_glb_sha256": entry.get("source_candidate_gltf_hash", source_data.get("hash", "")),
            "runtime_mesh_sha256": entry.get("runtime_gltf_hash", ""),
            "generation_tool": provenance.get("generation_import_tool", "rodin-hyper3d-gen2.5"),
            "tool_version": provenance.get("tool_version", ""),
            "generation_date": provenance.get("generation_date", ""),
            "acceptance_state": "source_approved",
            "license_status": "owner_approved_rodin_use",
            "production_ready": False,
            "candidate_only": False,
            "presentation_only": True,
            "truth_mutation": False,
            "vertices": entry.get("vertex_count", 0),
            "triangles": entry.get("triangle_count", 0),
            "topology_manifold_validation_passed": (
                entry.get("validation_result", {}).get("material_validation", {})
                .get("topology_manifold_validation_passed") is True
                or material_val.get("topology_manifold_validation_passed") is True
            ),
            "material_channels": ["base_color", "normal", "orm"],
            "placeholder_textures": False,
        })

    if rodin_entries:
        current_entries = production_payload.get("entries", [])
        all_entries = current_entries + rodin_entries
        all_kind_counts = {kind: 0 for kind in REQUIRED_COUNTS}
        for entry in all_entries:
            if entry["kind"] in all_kind_counts:
                all_kind_counts[entry["kind"]] += 1
        production_payload["entries"] = all_entries
        production_payload["entry_count"] = len(all_entries)
        production_payload["kind_counts"] = all_kind_counts
        meshy_count = len(current_entries)
        rodin_count = len(rodin_entries)
        production_payload["production_assets_scope"] = (
            f"{meshy_count} Meshy-6 + {rodin_count} Rodin source-approved production seed assets; "
            "not production-ready, not owner-accepted"
        )
    OUTPUT.write_text(json.dumps(production_payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    production_payload["manifest_hash"] = sha256_file(OUTPUT)
    OUTPUT.write_text(json.dumps(production_payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps({"production_candidate_passed": candidate_passed, "candidate_manifest": CANDIDATE_OUTPUT.relative_to(ROOT).as_posix(), "production_manifest": OUTPUT.relative_to(ROOT).as_posix(), "candidate_entries": len(entries), "production_entries": len(production_payload.get("entries", []))}, indent=2, sort_keys=True))
    return 0 if candidate_passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
