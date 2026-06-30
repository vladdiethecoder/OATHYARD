#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/runtime_3d/verify}"
manifest="${2:-assets/runtime_manifest.json}"
combat="${3:-artifacts/native_combat/verify/native_combat_render_manifest.json}"
mkdir -p "$out"

python3 - "$out" "$manifest" "$combat" <<'PY'
import json
import sys
from pathlib import Path

out = Path(sys.argv[1])
manifest_path = Path(sys.argv[2])
combat_path = Path(sys.argv[3])

failures = []
entries = []


def check(name, passed, detail):
    if not passed:
        failures.append(f"{name}: {detail}")


def read_json(path):
    return json.loads(path.read_text(encoding="utf-8"))


def markdown_bool(value):
    return "true" if value else "false"


check("runtime_manifest_exists", manifest_path.is_file(), manifest_path.as_posix())
check("native_combat_manifest_exists", combat_path.is_file(), combat_path.as_posix())

manifest = read_json(manifest_path) if manifest_path.is_file() else {}
combat = read_json(combat_path) if combat_path.is_file() else {}

check("runtime_manifest_schema", manifest.get("schema") == "oathyard.assets.v1", manifest.get("schema"))
check("product", manifest.get("product") == "OATHYARD", manifest.get("product"))
check("public_demo_ready_false", manifest.get("public_demo_ready") is False, manifest.get("public_demo_ready"))
check(
    "release_candidate_ready_false",
    manifest.get("release_candidate_ready") is False,
    manifest.get("release_candidate_ready"),
)

total_vertices = 0
total_triangles = 0
assets_with_depth = 0
for entry in manifest.get("entries", []):
    gltf_path = Path(entry.get("runtime_gltf", ""))
    asset_id = entry.get("id", "")
    kind = entry.get("kind", "")
    ok = gltf_path.is_file()
    check(f"{asset_id}.gltf_exists", ok, gltf_path.as_posix())
    z_depth = 0.0
    vertex_count = 0
    triangle_count = 0
    if ok:
        gltf = read_json(gltf_path)
        accessors = gltf.get("accessors", [])
        if accessors:
            bounds_min = accessors[0].get("min", [0, 0, 0])
            bounds_max = accessors[0].get("max", [0, 0, 0])
            if len(bounds_min) == 3 and len(bounds_max) == 3:
                z_depth = bounds_max[2] - bounds_min[2]
            vertex_count = int(accessors[0].get("count", 0))
            index_count = int(accessors[1].get("count", 0)) if len(accessors) > 1 else 0
            triangle_count = index_count // 3
        check(f"{asset_id}.gltf_version", gltf.get("asset", {}).get("version") == "2.0", gltf_path.as_posix())
        check(f"{asset_id}.z_depth_nonzero", z_depth > 0.0, z_depth)
        check(f"{asset_id}.triangles_present", triangle_count > 0, triangle_count)
    if z_depth > 0.0:
        assets_with_depth += 1
    total_vertices += vertex_count
    total_triangles += triangle_count
    entries.append(
        {
            "id": asset_id,
            "kind": kind,
            "runtime_gltf": entry.get("runtime_gltf", ""),
            "vertex_count": vertex_count,
            "triangle_count": triangle_count,
            "z_depth_milli": int(round(z_depth * 1000.0)),
            "has_3d_depth": z_depth > 0.0,
        }
    )

combat_schema_ok = combat.get("schema") == "oathyard.native_combat_render.v1"
check("combat_schema", combat_schema_ok, combat.get("schema"))
check("combat_source_after_hash", combat.get("source") == "truth-after-hash-duel-result", combat.get("source"))
check("combat_truth_mutation_false", combat.get("truth_mutation") is False, combat.get("truth_mutation"))
check("combat_native_3d_runtime_geometry", combat.get("native_3d_runtime_geometry") is True, combat.get("native_3d_runtime_geometry"))
check("combat_projection_uses_z_depth", combat.get("projection_uses_z_depth") is True, combat.get("projection_uses_z_depth"))
check(
    "combat_projection_model",
    combat.get("projection_model") == "integer_oblique_depth_projection",
    combat.get("projection_model"),
)

active_assets = []
if combat:
    active_assets.append(combat.get("arena_asset", {}))
    for fighter in combat.get("silhouette_fighters", []):
        active_assets.append(fighter.get("weapon_asset", {}))
        active_assets.append(fighter.get("armor_asset", {}))
for asset in active_assets:
    asset_id = asset.get("id", "<missing>")
    geometry = asset.get("geometry", {})
    check(
        f"combat_asset_{asset_id}.has_nonzero_z_depth",
        geometry.get("has_nonzero_z_depth") is True and int(geometry.get("z_depth_milli", 0)) > 0,
        geometry,
    )
    check(
        f"combat_asset_{asset_id}.projection_uses_z_depth",
        geometry.get("projection_uses_z_depth") is True,
        geometry,
    )

passed = not failures
payload = {
    "schema": "oathyard.runtime_3d_audit.v1",
    "product": "OATHYARD",
    "runtime_manifest": manifest_path.as_posix(),
    "native_combat_manifest": combat_path.as_posix(),
    "entry_count": len(entries),
    "assets_with_3d_depth": assets_with_depth,
    "total_vertices": total_vertices,
    "total_triangles": total_triangles,
    "native_3d_runtime_geometry": combat.get("native_3d_runtime_geometry") is True,
    "projection_uses_z_depth": combat.get("projection_uses_z_depth") is True,
    "projection_model": combat.get("projection_model", ""),
    "truth_mutation": False,
    "presentation_only": True,
    "owner_visual_acceptance_claimed": False,
    "public_demo_ready": False,
    "release_candidate_ready": False,
    "failed_check_count": len(failures),
    "passed": passed,
    "entries": entries,
    "failures": failures,
}
(out / "runtime_3d_audit.json").write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

lines = [
    "# OATHYARD Runtime 3D Audit",
    "",
    f"Status: {'PASSED' if passed else 'FAILED'}",
    f"- Runtime assets: `{len(entries)}`",
    f"- Assets with nonzero Z depth: `{assets_with_depth}`",
    f"- Total vertices: `{total_vertices}`",
    f"- Total triangles: `{total_triangles}`",
    f"- Native 3D runtime geometry: `{markdown_bool(combat.get('native_3d_runtime_geometry') is True)}`",
    f"- Projection model: `{combat.get('projection_model', '')}`",
    f"- Projection uses Z depth: `{markdown_bool(combat.get('projection_uses_z_depth') is True)}`",
    "- Truth mutation: `none`",
    "- Owner visual acceptance claimed: `false`",
    "- Public demo ready: `false`",
    "- Release candidate ready: `false`",
    "",
    "## Runtime Assets",
]
for entry in entries:
    lines.append(
        f"- `{entry['id']}` `{entry['kind']}` z-depth `{entry['z_depth_milli']}` vertices `{entry['vertex_count']}` triangles `{entry['triangle_count']}`"
    )
if failures:
    lines.extend(["", "## Failures"])
    lines.extend(f"- {failure}" for failure in failures)
(out / "runtime_3d_audit_report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")

if not passed:
    sys.stderr.write(f"runtime 3D audit failed with {len(failures)} failure(s); see {out / 'runtime_3d_audit_report.md'}\n")
    sys.exit(1)

print(f"runtime 3D audit passed: {assets_with_depth}/{len(entries)} assets have nonzero Z depth")
PY
