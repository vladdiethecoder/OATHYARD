#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/asset_atlas/latest}"
mkdir -p "$out"

python3 - "$out" <<'PY'
import hashlib
import html
import json
import os
import sys
from pathlib import Path

ROOT = Path.cwd()
out = Path(sys.argv[1])
out.mkdir(parents=True, exist_ok=True)

RUNTIME_MANIFEST = ROOT / "assets/runtime_manifest.json"
REQUIRED_COUNTS = {"fighters": 6, "weapons": 8, "armor": 6, "arenas": 2}
FORBIDDEN_SOURCE_MARKERS = [
    "copied_from",
    "scraped",
    "borrowed",
    "unlicensed",
    "placeholder",
    "todo_asset",
]


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(65536), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def add_check(checks, failures, asset_id, name, passed, detail):
    checks.append({"name": name, "passed": bool(passed), "detail": detail})
    if not passed:
        failures.append(f"{asset_id}: {name}: {detail}")


def parse_gltf_metrics(path: Path):
    gltf = read_json(path)
    vertex_count = 0
    index_count = 0
    z_depth_milli = 0
    for mesh in gltf.get("meshes", []):
        for primitive in mesh.get("primitives", []):
            position = primitive.get("attributes", {}).get("POSITION")
            if isinstance(position, int):
                vertex_count += int(gltf["accessors"][position]["count"])
            index_accessor = primitive.get("indices")
            if isinstance(index_accessor, int):
                index_count += int(gltf["accessors"][index_accessor]["count"])
    accessors = gltf.get("accessors", [])
    if accessors:
        bounds_min = accessors[0].get("min", [0, 0, 0])
        bounds_max = accessors[0].get("max", [0, 0, 0])
        if len(bounds_min) == 3 and len(bounds_max) == 3:
            z_depth_milli = int(round((bounds_max[2] - bounds_min[2]) * 1000.0))
    return {
        "asset_version": gltf.get("asset", {}).get("version", ""),
        "generator": gltf.get("asset", {}).get("generator", ""),
        "mesh_count": len(gltf.get("meshes", [])),
        "material_count": len(gltf.get("materials", [])),
        "primitive_count": sum(len(mesh.get("primitives", [])) for mesh in gltf.get("meshes", [])),
        "vertex_count": vertex_count,
        "index_count": index_count,
        "triangle_count": index_count // 3,
        "image_count": len(gltf.get("images", [])),
        "texture_count": len(gltf.get("textures", [])),
        "z_depth_milli": z_depth_milli,
        "external_uri_count": sum(
            1
            for buffer in gltf.get("buffers", [])
            if not str(buffer.get("uri", "")).startswith("data:")
        ),
    }


failures = []
entries = []
if not RUNTIME_MANIFEST.is_file():
    failures.append(f"missing runtime manifest: {RUNTIME_MANIFEST.relative_to(ROOT)}")
    manifest = {"schema": "missing", "entries": [], "asset_hash": ""}
else:
    manifest = read_json(RUNTIME_MANIFEST)
    if manifest.get("schema") != "oathyard.assets.v1":
        failures.append("runtime_manifest schema is not oathyard.assets.v1")
    if manifest.get("product") != "OATHYARD":
        failures.append("runtime_manifest product is not OATHYARD")
    if manifest.get("public_demo_ready") is not False:
        failures.append("runtime_manifest public_demo_ready must remain false")
    if manifest.get("release_candidate_ready") is not False:
        failures.append("runtime_manifest release_candidate_ready must remain false")

kind_counts = {kind: 0 for kind in REQUIRED_COUNTS}
total_vertices = 0
total_indices = 0
total_triangles = 0
total_primitives = 0
total_materials = 0

for item in manifest.get("entries", []):
    asset_id = item.get("id", "")
    kind = item.get("kind", "")
    checks = []
    if kind in kind_counts:
        kind_counts[kind] += 1
    else:
        failures.append(f"{asset_id}: unexpected kind: {kind}")

    source = ROOT / item.get("source", "")
    preview = ROOT / item.get("preview", "")
    mesh_path = ROOT / item.get("runtime_mesh", "")
    gltf_path = ROOT / item.get("runtime_gltf", "")
    material_maps = item.get("material_maps", {})

    source_text = source.read_text(encoding="utf-8") if source.is_file() else ""
    preview_text = preview.read_text(encoding="utf-8") if preview.is_file() else ""
    mesh = read_json(mesh_path) if mesh_path.is_file() else {}
    gltf_metrics = parse_gltf_metrics(gltf_path) if gltf_path.is_file() else {}

    add_check(checks, failures, asset_id, "source_exists", source.is_file(), item.get("source", ""))
    add_check(checks, failures, asset_id, "preview_exists", preview.is_file(), item.get("preview", ""))
    add_check(checks, failures, asset_id, "runtime_mesh_exists", mesh_path.is_file(), item.get("runtime_mesh", ""))
    add_check(checks, failures, asset_id, "runtime_gltf_exists", gltf_path.is_file(), item.get("runtime_gltf", ""))
    add_check(checks, failures, asset_id, "provenance_repo_owned", item.get("provenance") == "repo_owned_original_text_asset", item.get("provenance", ""))
    add_check(checks, failures, asset_id, "source_declares_repo_owned", "provenance=repo_owned" in source_text.lower(), item.get("source", ""))
    for marker in FORBIDDEN_SOURCE_MARKERS:
        add_check(checks, failures, asset_id, f"source_absent:{marker}", marker not in source_text.lower(), item.get("source", ""))
    add_check(checks, failures, asset_id, "preview_svg", preview_text.lstrip().startswith("<svg"), item.get("preview", ""))
    add_check(checks, failures, asset_id, "preview_labels_asset", asset_id in preview_text, item.get("preview", ""))
    add_check(checks, failures, asset_id, "preview_labels_kind", kind in preview_text, item.get("preview", ""))
    add_check(checks, failures, asset_id, "mesh_schema", mesh.get("schema") == "oathyard.runtime_asset.v1", item.get("runtime_mesh", ""))
    add_check(checks, failures, asset_id, "mesh_id_matches", mesh.get("id") == asset_id, mesh.get("id", ""))
    add_check(checks, failures, asset_id, "mesh_kind_matches", mesh.get("kind") == kind, mesh.get("kind", ""))
    add_check(checks, failures, asset_id, "mesh_hash_matches", mesh.get("hash") == item.get("hash"), mesh.get("hash", ""))
    add_check(checks, failures, asset_id, "gltf_version_2", gltf_metrics.get("asset_version") == "2.0", item.get("runtime_gltf", ""))
    add_check(checks, failures, asset_id, "gltf_generator_oathyard", "OATHYARD" in gltf_metrics.get("generator", ""), gltf_metrics.get("generator", ""))
    add_check(checks, failures, asset_id, "gltf_has_mesh", gltf_metrics.get("mesh_count", 0) >= 1, str(gltf_metrics.get("mesh_count", 0)))
    add_check(checks, failures, asset_id, "gltf_has_material", gltf_metrics.get("material_count", 0) >= 1, str(gltf_metrics.get("material_count", 0)))
    add_check(checks, failures, asset_id, "gltf_has_3d_z_depth", gltf_metrics.get("z_depth_milli", 0) > 0, str(gltf_metrics.get("z_depth_milli", 0)))
    add_check(checks, failures, asset_id, "gltf_no_external_buffers", gltf_metrics.get("external_uri_count", 0) == 0, str(gltf_metrics.get("external_uri_count", 0)))
    if kind == "fighters":
        add_check(
            checks,
            failures,
            asset_id,
            "fighter_canonical_joint_mapping",
            mesh.get("truth_joint_mapping") == "canonical_16_plus_grips",
            mesh.get("truth_joint_mapping", ""),
        )

    if kind == "arenas":
        add_check(checks, failures, asset_id, "arena_material_zone_count", len(mesh.get("material_zones", [])) >= 6, str(len(mesh.get("material_zones", []))))
        add_check(checks, failures, asset_id, "arena_manifest_material_maps", set(material_maps) == {"base", "normal", "orm"}, str(sorted(material_maps)))
        add_check(checks, failures, asset_id, "arena_runtime_material_maps", set(mesh.get("material_maps", {})) == {"base", "normal", "orm"}, str(sorted(mesh.get("material_maps", {}))))
        add_check(checks, failures, asset_id, "arena_lighting_anchors", len(mesh.get("lighting_anchors", [])) >= 3, str(len(mesh.get("lighting_anchors", []))))
        add_check(checks, failures, asset_id, "arena_duel_readable_landmarks", len(mesh.get("duel_readable_landmarks", [])) >= 4, str(len(mesh.get("duel_readable_landmarks", []))))
        add_check(checks, failures, asset_id, "arena_floor_contact_readability", len(mesh.get("floor_contact_readability", [])) >= 3, str(len(mesh.get("floor_contact_readability", []))))
        add_check(checks, failures, asset_id, "arena_composition_profile", bool(mesh.get("composition_profile")), str(mesh.get("composition_profile", "")))
        add_check(checks, failures, asset_id, "arena_scale_reference", bool(mesh.get("scale_reference")), str(mesh.get("scale_reference", "")))
        add_check(checks, failures, asset_id, "arena_silhouette_context", bool(mesh.get("silhouette_context")), str(mesh.get("silhouette_context", "")))
        add_check(checks, failures, asset_id, "arena_playable_space_cues", len(mesh.get("playable_space", [])) >= 4, str(len(mesh.get("playable_space", []))))
        add_check(checks, failures, asset_id, "arena_atmosphere_hooks", len(mesh.get("atmosphere_hooks", [])) >= 4, str(len(mesh.get("atmosphere_hooks", []))))
        add_check(checks, failures, asset_id, "arena_capture_ids", set(mesh.get("capture_ids", [])) == {"establishing", "gameplay", "contact"}, str(sorted(mesh.get("capture_ids", []))))
        originality = str(mesh.get("originality_notes", "")).lower()
        add_check(checks, failures, asset_id, "arena_originality_notes", bool(originality) and "repo_owned" in originality and not any(marker in originality for marker in ["copied_from", "scraped", "borrowed", "unlicensed"]), str(mesh.get("originality_notes", "")))
        add_check(checks, failures, asset_id, "arena_gltf_six_materials", gltf_metrics.get("material_count", 0) >= 6, str(gltf_metrics.get("material_count", 0)))
        add_check(checks, failures, asset_id, "arena_gltf_texture_maps", gltf_metrics.get("image_count", 0) == 3 and gltf_metrics.get("texture_count", 0) == 3, f"images {gltf_metrics.get('image_count', 0)} textures {gltf_metrics.get('texture_count', 0)}")
        for map_name, rel_path in material_maps.items():
            texture_path = ROOT / rel_path
            add_check(checks, failures, asset_id, f"arena_material_map_exists:{map_name}", texture_path.is_file(), rel_path)
            expected_hash = mesh.get("material_map_hashes", {}).get(map_name, "")
            observed_hash = sha256_file(texture_path) if texture_path.is_file() else ""
            add_check(checks, failures, asset_id, f"arena_material_map_hash:{map_name}", expected_hash == observed_hash and bool(observed_hash), expected_hash)

    total_vertices += gltf_metrics.get("vertex_count", 0)
    total_indices += gltf_metrics.get("index_count", 0)
    total_triangles += gltf_metrics.get("triangle_count", 0)
    total_primitives += gltf_metrics.get("primitive_count", 0)
    total_materials += gltf_metrics.get("material_count", 0)

    entry = {
        "id": asset_id,
        "kind": kind,
        "source": item.get("source", ""),
        "preview": item.get("preview", ""),
        "runtime_mesh": item.get("runtime_mesh", ""),
        "runtime_gltf": item.get("runtime_gltf", ""),
        "material_maps": material_maps,
        "arena_identity_metadata": {
            "composition_profile": mesh.get("composition_profile", ""),
            "scale_reference": mesh.get("scale_reference", ""),
            "silhouette_context": mesh.get("silhouette_context", ""),
            "playable_space": mesh.get("playable_space", []),
            "atmosphere_hooks": mesh.get("atmosphere_hooks", []),
            "capture_ids": mesh.get("capture_ids", []),
            "originality_notes": mesh.get("originality_notes", ""),
        } if kind == "arenas" else {},
        "provenance": item.get("provenance", ""),
        "source_sha256": sha256_file(source) if source.is_file() else "",
        "preview_sha256": sha256_file(preview) if preview.is_file() else "",
        "runtime_mesh_sha256": sha256_file(mesh_path) if mesh_path.is_file() else "",
        "runtime_gltf_sha256": sha256_file(gltf_path) if gltf_path.is_file() else "",
        "gltf_metrics": gltf_metrics,
        "checks": checks,
        "passed": all(check["passed"] for check in checks),
    }
    entries.append(entry)

for kind, minimum in REQUIRED_COUNTS.items():
    if kind_counts.get(kind, 0) < minimum:
        failures.append(f"{kind}: count {kind_counts.get(kind, 0)} below required {minimum}")

all_passed = not failures and all(entry["passed"] for entry in entries)
atlas_manifest = {
    "schema": "oathyard.asset_visual_atlas.v1",
    "product": "OATHYARD",
    "tool": "tools/asset_visual_atlas.sh",
    "source_manifest": "assets/runtime_manifest.json",
    "asset_hash": manifest.get("asset_hash", ""),
    "entry_count": len(entries),
    "kind_counts": kind_counts,
    "required_counts": REQUIRED_COUNTS,
    "total_vertices": total_vertices,
    "total_indices": total_indices,
    "total_triangles": total_triangles,
    "total_primitives": total_primitives,
    "total_materials": total_materials,
    "assets_with_3d_depth": sum(1 for entry in entries if entry["gltf_metrics"].get("z_depth_milli", 0) > 0),
    "production_placeholder_markers_present": False,
    "all_assets_source_backed": all(bool(entry["source_sha256"]) for entry in entries),
    "all_assets_runtime_backed": all(bool(entry["runtime_mesh_sha256"]) and bool(entry["runtime_gltf_sha256"]) for entry in entries),
    "all_previews_present": all(bool(entry["preview_sha256"]) for entry in entries),
    "external_khronos_validation_claimed": False,
    "owner_visual_acceptance_claimed": False,
    "public_demo_ready": False,
    "release_candidate_ready": False,
    "failed_check_count": len(failures),
    "failed_asset_reduction": "failed_asset_visuals.txt",
    "contact_sheet": "asset_visual_atlas.svg",
    "passed": all_passed,
    "entries": entries,
}

(out / "asset_visual_atlas_manifest.json").write_text(
    json.dumps(atlas_manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
)
(out / "failed_asset_visuals.txt").write_text("none\n" if not failures else "\n".join(failures) + "\n", encoding="utf-8")

report = [
    "# OATHYARD Asset Visual Atlas",
    "",
    f"Status: {'PASSED' if all_passed else 'FAILED'}",
    f"- Asset hash: `{manifest.get('asset_hash', '')}`",
    f"- Entries: `{len(entries)}`",
    f"- Kind counts: `fighters {kind_counts.get('fighters', 0)}` `weapons {kind_counts.get('weapons', 0)}` `armor {kind_counts.get('armor', 0)}` `arenas {kind_counts.get('arenas', 0)}`",
    f"- Runtime vertices: `{total_vertices}`",
    f"- Runtime indices: `{total_indices}`",
    f"- Runtime triangles: `{total_triangles}`",
    f"- Runtime primitives: `{total_primitives}`",
    f"- Runtime materials: `{total_materials}`",
    f"- Assets with 3D Z depth: `{sum(1 for entry in entries if entry['gltf_metrics'].get('z_depth_milli', 0) > 0)}`",
    f"- Failed checks: `{len(failures)}`",
    f"- Failed asset reduction: `{'none' if not failures else 'failed_asset_visuals.txt'}`",
    "- All assets source backed: `true`" if atlas_manifest["all_assets_source_backed"] else "- All assets source backed: `false`",
    "- All assets runtime backed: `true`" if atlas_manifest["all_assets_runtime_backed"] else "- All assets runtime backed: `false`",
    "- All previews present: `true`" if atlas_manifest["all_previews_present"] else "- All previews present: `false`",
    "- Production placeholder markers present: `false`",
    "- External Khronos validation claimed: `false`",
    "- Owner visual acceptance claimed: `false`",
    "- Public demo ready: `false`",
    "- Release candidate ready: `false`",
    "",
    "## Entries",
]
for entry in entries:
    metrics = entry["gltf_metrics"]
    state = "passed" if entry["passed"] else "failed"
    arena_maps = f" material_maps `{len(entry.get('material_maps', {}))}` composition `{entry.get('arena_identity_metadata', {}).get('composition_profile', '')}`" if entry["kind"] == "arenas" else ""
    report.append(
        f"- `{state}` `{entry['id']}` `{entry['kind']}` vertices `{metrics.get('vertex_count', 0)}` "
        f"indices `{metrics.get('index_count', 0)}` triangles `{metrics.get('triangle_count', 0)}` "
        f"z-depth `{metrics.get('z_depth_milli', 0)}`{arena_maps} preview `{entry['preview']}` glTF `{entry['runtime_gltf']}`"
    )
if failures:
    report.extend(["", "## Reduced Failures"])
    report.extend(f"- {failure}" for failure in failures)
(out / "asset_visual_atlas_report.md").write_text("\n".join(report) + "\n", encoding="utf-8")

hash_lines = []
for entry in entries:
    for key in ["source", "preview", "runtime_mesh", "runtime_gltf"]:
        digest_key = f"{key}_sha256" if key == "source" else f"{key}_sha256"
        digest = entry.get(digest_key, "")
        if digest:
            hash_lines.append(f"{digest}  {entry[key]}")
    for rel_path in entry.get("material_maps", {}).values():
        path = ROOT / rel_path
        if path.is_file():
            hash_lines.append(f"{sha256_file(path)}  {rel_path}")
(out / "asset_visual_atlas_hashes.sha256").write_text("\n".join(sorted(hash_lines)) + "\n", encoding="utf-8")

cols = 3
card_w = 360
card_h = 214
rows = max(1, (len(entries) + cols - 1) // cols)
width = 40 + cols * card_w
height = 130 + rows * card_h
svg = [
    f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">',
    '<rect width="100%" height="100%" fill="#101414"/>',
    '<text x="24" y="36" fill="#f2f0e8" font-family="monospace" font-size="23">OATHYARD runtime asset visual atlas</text>',
    f'<text x="24" y="64" fill="#c6d0cf" font-family="monospace" font-size="13">status: {"PASSED" if all_passed else "FAILED"} | assets: {len(entries)} | hash: {manifest.get("asset_hash", "")[:16]} | owner visual acceptance not claimed</text>',
    f'<text x="24" y="88" fill="#96adb0" font-family="monospace" font-size="12">fighters {kind_counts.get("fighters", 0)} | weapons {kind_counts.get("weapons", 0)} | armor {kind_counts.get("armor", 0)} | arenas {kind_counts.get("arenas", 0)} | external Khronos validation false</text>',
]
for idx, entry in enumerate(entries):
    col = idx % cols
    row = idx // cols
    x = 24 + col * card_w
    y = 116 + row * card_h
    preview_rel = os.path.relpath(ROOT / entry["preview"], out)
    metrics = entry["gltf_metrics"]
    fill = "#1c2526" if entry["passed"] else "#3a1f1d"
    title = html.escape(f"{entry['id']} {entry['kind']}")
    path_text = html.escape(entry["runtime_gltf"])
    svg.extend(
        [
            f'<rect x="{x}" y="{y}" width="{card_w - 18}" height="{card_h - 18}" rx="6" fill="{fill}" stroke="#566a6f"/>',
            f'<text x="{x + 12}" y="{y + 23}" fill="#f2f0e8" font-family="monospace" font-size="14">{title}</text>',
            f'<text x="{x + 12}" y="{y + 42}" fill="#9db6bb" font-family="monospace" font-size="10">{path_text[:58]}</text>',
            f'<image x="{x + 12}" y="{y + 54}" width="{card_w - 42}" height="112" href="{html.escape(preview_rel)}" preserveAspectRatio="xMidYMid meet"/>',
            f'<text x="{x + 12}" y="{y + 182}" fill="#c6d0cf" font-family="monospace" font-size="10">v {metrics.get("vertex_count", 0)} | tri {metrics.get("triangle_count", 0)} | z {metrics.get("z_depth_milli", 0)} | src {entry["source_sha256"][:8]}</text>',
        ]
    )
svg.append("</svg>")
(out / "asset_visual_atlas.svg").write_text("\n".join(svg) + "\n", encoding="utf-8")

if not all_passed:
    sys.stderr.write(f"asset visual atlas failed with {len(failures)} failure(s); see {out / 'failed_asset_visuals.txt'}\n")
    sys.exit(1)

print(f"asset visual atlas passed: {len(entries)} runtime assets indexed")
PY
