#!/usr/bin/env python3
"""
Run Blender DCC import/mesh-sanity validation over OATHYARD model-candidate glTFs.

This is external DCC evidence only. It proves Blender can import the exact
source-candidate glTF bytes and records mesh-sanity/topology metrics. It does
not claim owner acceptance, production renderer evidence, legal clearance,
release readiness, or authoritative truth changes.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_RUN_ID = "t_73291be5"
POSITION_WELD_DISTANCE = 1.0e-6


def running_inside_blender() -> bool:
    try:
        import bpy  # type: ignore  # noqa: F401
        return True
    except Exception:
        return False


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def rel(path: Path) -> str:
    return path.resolve().relative_to(ROOT).as_posix()


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-id", default=DEFAULT_RUN_ID)
    parser.add_argument(
        "--out",
        default="",
        help="Evidence JSON path. Default: assets/model_candidates/<run-id>/validation/blender_dcc_validation.json",
    )
    parser.add_argument("--blender", default="blender")
    parser.add_argument("--inner", action="store_true")
    return parser.parse_args(argv)


def write_markdown(json_path: Path, md_path: Path) -> None:
    evidence = read_json(json_path)
    lines = [
        "# OATHYARD Blender DCC Validation Evidence",
        "",
        f"Status: {'PASSED' if evidence['passed'] else 'FAILED'}",
        "Evidence class: Blender DCC import and mesh-sanity validation for source-candidate glTF files only.",
        "",
        f"- Blender version: `{evidence.get('blender_version', '')}`",
        f"- Asset count: `{evidence['asset_count']}`",
        f"- Import passed assets: `{evidence['import_passed_asset_count']}`",
        f"- Mesh sanity passed assets: `{evidence['mesh_sanity_passed_asset_count']}`",
        f"- Position-weld distance: `{evidence['position_weld_distance']}`",
        f"- Position-welded closed-manifold assets: `{evidence['closed_manifold_asset_count']}`",
        f"- Position-welded open-boundary/non-manifold assets: `{evidence['open_boundary_asset_count']}`",
        "- External Khronos validation is separate and must remain hash-bound.",
        "- Topology/manifold validation: duplicate material/UV seam vertices are welded by exact Blender position tolerance before classifying holes.",
        "- Production asset ready: `false`",
        "- Owner visual accepted: `false`",
        "- Public demo ready: `false`",
        "- Release candidate ready: `false`",
        "- Truth mutation: `false`",
        "",
        "## Per-asset results",
        "",
        "| Asset | Kind | Import | Mesh sanity | Raw boundary edges | Welded topology status | Welded boundary edges | Welded loose edges | Welded zero-area faces | glTF |",
        "| --- | --- | --- | --- | --- | ---: | ---: | ---: | --- |",
    ]
    for item in evidence["results"]:
        lines.append(
            f"| `{item['asset_id']}` | `{item['kind']}` | `{item['import_passed']}` | `{item['mesh_sanity_passed']}` | {item['boundary_edges']} | `{item['topology_manifold_status']}` | {item['welded_boundary_edges']} | {item['welded_loose_edges']} | {item['welded_zero_area_faces']} | `{item['runtime_gltf']}` |"
        )
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def launch_blender(args: argparse.Namespace) -> int:
    out_json = ROOT / args.out if args.out else ROOT / "assets" / "model_candidates" / args.run_id / "validation" / "blender_dcc_validation.json"
    out_json.parent.mkdir(parents=True, exist_ok=True)
    command = [
        args.blender,
        "--background",
        "--python",
        str(Path(__file__).resolve()),
        "--",
        "--inner",
        "--run-id",
        args.run_id,
        "--out",
        rel(out_json),
    ]
    completed = subprocess.run(command, cwd=ROOT, text=True, capture_output=True)
    log_path = out_json.with_suffix(".blender.log")
    log_path.write_text((completed.stdout or "") + (completed.stderr or ""), encoding="utf-8")
    if completed.returncode != 0:
        sys.stderr.write(log_path.read_text(encoding="utf-8"))
        return completed.returncode
    write_markdown(out_json, out_json.with_suffix(".md"))
    evidence = read_json(out_json)
    print(
        json.dumps(
            {
                "evidence": rel(out_json),
                "evidence_sha256": sha256_file(out_json),
                "asset_count": evidence["asset_count"],
                "passed": evidence["passed"],
                "import_passed_asset_count": evidence["import_passed_asset_count"],
                "mesh_sanity_passed_asset_count": evidence["mesh_sanity_passed_asset_count"],
                "closed_manifold_asset_count": evidence["closed_manifold_asset_count"],
                "open_boundary_asset_count": evidence["open_boundary_asset_count"],
            },
            indent=2,
            sort_keys=True,
        )
    )
    return 0 if evidence["passed"] else 1


def run_inside_blender(args: argparse.Namespace) -> int:
    import bmesh  # type: ignore
    import bpy  # type: ignore

    manifest_path = ROOT / "assets" / "model_candidates" / args.run_id / "model_candidate_manifest.json"
    manifest = read_json(manifest_path)
    out_json = ROOT / args.out if args.out else ROOT / "assets" / "model_candidates" / args.run_id / "validation" / "blender_dcc_validation.json"
    out_json.parent.mkdir(parents=True, exist_ok=True)
    results: list[dict[str, Any]] = []
    for entry in manifest.get("entries", []):
        bpy.ops.object.select_all(action="SELECT")
        bpy.ops.object.delete()
        gltf_path = ROOT / entry["runtime_gltf"]
        bin_path = ROOT / entry["runtime_bin"]
        errors: list[str] = []
        import_passed = False
        mesh_sanity_passed = False
        mesh_count = object_count = vertices = polygons = triangles = edges = materials = 0
        boundary_edges = nonmanifold_edges = loose_edges = zero_area_faces = 0
        welded_vertices = welded_edges = welded_faces = 0
        welded_boundary_edges = welded_nonmanifold_edges = welded_loose_edges = welded_zero_area_faces = 0
        mesh_validate_changed = False
        try:
            bpy.ops.import_scene.gltf(filepath=str(gltf_path))
            import_passed = True
            object_count = len(bpy.context.scene.objects)
            meshes = [obj for obj in bpy.context.scene.objects if obj.type == "MESH"]
            mesh_count = len(meshes)
            material_names: set[str] = set()
            for obj in meshes:
                mesh = obj.data
                mesh_validate_changed = bool(mesh.validate(verbose=False)) or mesh_validate_changed
                mesh.update(calc_edges=True)
                vertices += len(mesh.vertices)
                polygons += len(mesh.polygons)
                triangles += sum(max(0, len(poly.vertices) - 2) for poly in mesh.polygons)
                edges += len(mesh.edges)
                for mat in mesh.materials:
                    if mat:
                        material_names.add(mat.name)
                bm = bmesh.new()
                bm.from_mesh(mesh)
                bm.edges.ensure_lookup_table()
                bm.faces.ensure_lookup_table()
                for edge in bm.edges:
                    linked = len(edge.link_faces)
                    if linked != 2:
                        nonmanifold_edges += 1
                    if linked == 1:
                        boundary_edges += 1
                    if linked == 0:
                        loose_edges += 1
                for face in bm.faces:
                    if abs(face.calc_area()) <= 1e-12:
                        zero_area_faces += 1
                bm.free()
                welded_bm = bmesh.new()
                welded_bm.from_mesh(mesh)
                welded_bm.verts.ensure_lookup_table()
                bmesh.ops.remove_doubles(
                    welded_bm,
                    verts=welded_bm.verts,
                    dist=POSITION_WELD_DISTANCE,
                )
                welded_bm.verts.ensure_lookup_table()
                welded_bm.edges.ensure_lookup_table()
                welded_bm.faces.ensure_lookup_table()
                welded_vertices += len(welded_bm.verts)
                welded_edges += len(welded_bm.edges)
                welded_faces += len(welded_bm.faces)
                for edge in welded_bm.edges:
                    linked = len(edge.link_faces)
                    if linked != 2:
                        welded_nonmanifold_edges += 1
                    if linked == 1:
                        welded_boundary_edges += 1
                    if linked == 0:
                        welded_loose_edges += 1
                for face in welded_bm.faces:
                    if abs(face.calc_area()) <= 1e-12:
                        welded_zero_area_faces += 1
                welded_bm.free()
            materials = len(material_names)
            mesh_sanity_passed = bool(
                import_passed
                and mesh_count > 0
                and vertices > 0
                and polygons > 0
                and not mesh_validate_changed
                and loose_edges == 0
                and zero_area_faces == 0
            )
        except Exception as exc:  # noqa: BLE001 - exact Blender import error belongs in evidence.
            errors.append(repr(exc))
        raw_topology_status = "raw_closed_manifold" if import_passed and nonmanifold_edges == 0 else "raw_open_boundary_edges_present"
        topology_manifold_validation_passed = bool(
            import_passed
            and welded_nonmanifold_edges == 0
            and welded_boundary_edges == 0
            and welded_loose_edges == 0
            and welded_zero_area_faces == 0
        )
        topology_status = "closed_manifold_after_position_weld" if topology_manifold_validation_passed else "open_boundary_edges_present_after_position_weld"
        results.append(
            {
                "asset_id": entry["id"],
                "kind": entry["kind"],
                "runtime_gltf": entry["runtime_gltf"],
                "runtime_gltf_sha256": sha256_file(gltf_path),
                "runtime_bin": entry["runtime_bin"],
                "runtime_bin_sha256": sha256_file(bin_path),
                "import_passed": import_passed,
                "mesh_sanity_passed": mesh_sanity_passed,
                "raw_topology_manifold_status": raw_topology_status,
                "topology_manifold_status": topology_status,
                "topology_manifold_validation_passed": topology_manifold_validation_passed,
                "position_weld_distance": POSITION_WELD_DISTANCE,
                "mesh_count": mesh_count,
                "object_count": object_count,
                "vertices": vertices,
                "polygons": polygons,
                "triangles": triangles,
                "edges": edges,
                "welded_vertices": welded_vertices,
                "welded_edges": welded_edges,
                "welded_faces": welded_faces,
                "welded_merged_vertices": max(0, vertices - welded_vertices),
                "materials": materials,
                "mesh_validate_changed": mesh_validate_changed,
                "nonmanifold_edges": nonmanifold_edges,
                "boundary_edges": boundary_edges,
                "loose_edges": loose_edges,
                "zero_area_faces": zero_area_faces,
                "welded_nonmanifold_edges": welded_nonmanifold_edges,
                "welded_boundary_edges": welded_boundary_edges,
                "welded_loose_edges": welded_loose_edges,
                "welded_zero_area_faces": welded_zero_area_faces,
                "errors": errors,
                "truth_mutation": False,
                "production_ready_after_this_evidence": False,
            }
        )
    import_passed_count = sum(1 for item in results if item["import_passed"])
    mesh_sanity_count = sum(1 for item in results if item["mesh_sanity_passed"])
    closed_count = sum(1 for item in results if item["topology_manifold_validation_passed"] is True)
    open_count = len(results) - closed_count
    passed = bool(results) and import_passed_count == len(results) and mesh_sanity_count == len(results)
    evidence = {
        "schema": "oathyard.blender_dcc_validation.v1",
        "tool": "tools/model_candidates/blender_validate_candidates.py",
        "run_id": args.run_id,
        "blender_version": bpy.app.version_string,
        "source_manifest": rel(manifest_path),
        "source_manifest_sha256": sha256_file(manifest_path),
        "asset_count": len(results),
        "passed": passed,
        "import_passed_asset_count": import_passed_count,
        "mesh_sanity_passed_asset_count": mesh_sanity_count,
        "position_weld_distance": POSITION_WELD_DISTANCE,
        "closed_manifold_asset_count": closed_count,
        "open_boundary_asset_count": open_count,
        "external_dcc_validation_claimed": passed,
        "topology_manifold_validation_passed": bool(results) and closed_count == len(results),
        "production_ready_after_this_evidence": False,
        "truth_mutation": False,
        "readiness_flags": {
            "production_asset_ready": False,
            "owner_visual_accepted": False,
            "public_demo_visual_ready": False,
            "release_candidate_ready": False,
        },
        "results": results,
    }
    out_json.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0 if passed else 1


def main() -> int:
    argv = sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else sys.argv[1:]
    args = parse_args(argv)
    if args.inner or running_inside_blender():
        return run_inside_blender(args)
    return launch_blender(args)


if __name__ == "__main__":
    raise SystemExit(main())
