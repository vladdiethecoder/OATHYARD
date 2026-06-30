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
PRESENTATION = ROOT / "assets" / "presentation_manifest.json"
OUTPUT = ROOT / "assets" / "production_visual_manifest.json"
REQUIRED_COUNTS = {"fighters": 6, "weapons": 8, "armor": 6, "arenas": 2}


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


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
            "toolchain": entry.get("toolchain", {}),
            "external_dcc_validation_claimed": False,
            "external_khronos_validation_claimed": False,
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
    payload = {
        "schema": "oathyard.production_visual_assets.v1",
        "product": "OATHYARD",
        "source_manifest": PRESENTATION.relative_to(ROOT).as_posix(),
        "source_manifest_hash": sha256_file(PRESENTATION),
        "candidate_run_id": presentation.get("candidate_run_id", ""),
        "production_assets_complete": False,
        "production_candidate_assets_complete": candidate_passed,
        "production_assets_scope": "production-candidate visual asset evidence for fighters/armor/weapons/arenas; not owner accepted and not release/public-demo ready",
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
        "entry_count": len(entries),
        "kind_counts": kind_counts,
        "entries": entries,
    }
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    payload["manifest_hash"] = sha256_file(OUTPUT)
    OUTPUT.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps({"production_candidate_passed": candidate_passed, "manifest": OUTPUT.relative_to(ROOT).as_posix(), "entries": len(entries)}, indent=2, sort_keys=True))
    return 0 if candidate_passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
