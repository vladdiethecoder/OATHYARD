#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/asset_budget/verify}"
manifest="${2:-assets/runtime_manifest.json}"
assets_root="${3:-assets}"
assets_src="${4:-assets_src}"

python3 - "$out" "$manifest" "$assets_root" "$assets_src" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

out = Path(sys.argv[1])
manifest_path = Path(sys.argv[2])
assets_root = Path(sys.argv[3])
assets_src = Path(sys.argv[4])
out.mkdir(parents=True, exist_ok=True)

BUDGETS = {
    "min_entries": 22,
    "min_fighters": 6,
    "min_weapons": 8,
    "min_armor": 6,
    "min_arenas": 2,
    "min_audio_events": 6,
    "min_vfx_events": 6,
    "max_total_gltf_bytes": 180_000,
    "max_total_runtime_mesh_bytes": 40_000,
    "max_total_preview_bytes": 20_000,
    "max_total_texture_bytes": 10_000,
    "max_total_vertices": 900,
    "max_total_indices": 3600,
    "max_total_triangles": 1200,
    "max_materials": 32,
    "max_primitives": 32,
    "max_embedded_buffer_bytes": 18_000,
    "min_arena_identity_vertices": 240,
    "min_arena_identity_triangles": 320,
}

checks = []


def check(check_id: str, passed: bool, detail: str) -> None:
    checks.append({"id": check_id, "passed": bool(passed), "detail": detail})


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def file_size(path: Path) -> int:
    return path.stat().st_size if path.is_file() else 0


try:
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
except Exception as error:  # noqa: BLE001 - audit artifact records exact failure.
    manifest = {}
    check("manifest_readable", False, str(error))
else:
    check("manifest_readable", True, manifest_path.as_posix())

entries = manifest.get("entries", [])
kind_counts = {}
asset_rows = []
total_gltf_bytes = 0
total_runtime_mesh_bytes = 0
total_preview_bytes = 0
total_texture_bytes = 0
total_vertices = 0
total_indices = 0
total_triangles = 0
total_materials = 0
total_primitives = 0
total_embedded_buffer_bytes = 0

for entry in entries:
    kind = entry.get("kind", "")
    kind_counts[kind] = kind_counts.get(kind, 0) + 1
    asset_id = entry.get("id", "<unknown>")
    gltf_path = Path(entry.get("runtime_gltf", ""))
    runtime_path = Path(entry.get("runtime_mesh", ""))
    preview_path = Path(entry.get("preview", ""))
    if not gltf_path.is_absolute():
        gltf_path = Path(gltf_path)
    if not runtime_path.is_absolute():
        runtime_path = Path(runtime_path)
    if not preview_path.is_absolute():
        preview_path = Path(preview_path)

    row = {
        "id": asset_id,
        "kind": kind,
        "gltf_bytes": file_size(gltf_path),
        "runtime_mesh_bytes": file_size(runtime_path),
        "preview_bytes": file_size(preview_path),
        "texture_bytes": sum(file_size(Path(path)) for path in entry.get("material_maps", {}).values()),
        "material_maps": entry.get("material_maps", {}),
        "vertices": 0,
        "indices": 0,
        "triangles": 0,
        "materials": 0,
        "primitives": 0,
        "embedded_buffer_bytes": 0,
        "source": entry.get("source", ""),
        "provenance": entry.get("provenance", ""),
        "hash": entry.get("hash", ""),
    }
    if gltf_path.is_file():
        try:
            gltf = json.loads(gltf_path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            gltf = {}
        accessors = gltf.get("accessors", [])
        if len(accessors) >= 2:
            row["vertices"] = int(accessors[0].get("count", 0))
            row["indices"] = int(accessors[1].get("count", 0))
            row["triangles"] = row["indices"] // 3
        row["materials"] = len(gltf.get("materials", []))
        row["primitives"] = sum(len(mesh.get("primitives", [])) for mesh in gltf.get("meshes", []))
        row["embedded_buffer_bytes"] = sum(int(buffer.get("byteLength", 0)) for buffer in gltf.get("buffers", []))

    total_gltf_bytes += row["gltf_bytes"]
    total_runtime_mesh_bytes += row["runtime_mesh_bytes"]
    total_preview_bytes += row["preview_bytes"]
    total_texture_bytes += row["texture_bytes"]
    total_vertices += row["vertices"]
    total_indices += row["indices"]
    total_triangles += row["triangles"]
    total_materials += row["materials"]
    total_primitives += row["primitives"]
    total_embedded_buffer_bytes += row["embedded_buffer_bytes"]
    asset_rows.append(row)


def count_source_records(path: Path, prefix: str) -> int:
    if not path.is_file():
        return 0
    return sum(1 for raw in path.read_text(encoding="utf-8").splitlines() if raw.strip().startswith(prefix))


audio_events = count_source_records(assets_src / "audio" / "audio.oysrc", "sound ")
vfx_events = count_source_records(assets_src / "vfx" / "vfx.oysrc", "vfx ")
source_bytes = sum(file_size(path) for path in sorted(assets_src.glob("*/*.oysrc")))
source_bytes += file_size(assets_src / "provenance.md")

check("manifest_schema", manifest.get("schema") == "oathyard.assets.v1", manifest.get("schema", ""))
check("public_demo_ready_false", manifest.get("public_demo_ready") is False, "false")
check("release_candidate_ready_false", manifest.get("release_candidate_ready") is False, "false")
check("entry_count", len(entries) >= BUDGETS["min_entries"], str(len(entries)))
check("fighter_count", kind_counts.get("fighters", 0) >= BUDGETS["min_fighters"], str(kind_counts.get("fighters", 0)))
check("weapon_count", kind_counts.get("weapons", 0) >= BUDGETS["min_weapons"], str(kind_counts.get("weapons", 0)))
check("armor_count", kind_counts.get("armor", 0) >= BUDGETS["min_armor"], str(kind_counts.get("armor", 0)))
check("arena_count", kind_counts.get("arenas", 0) >= BUDGETS["min_arenas"], str(kind_counts.get("arenas", 0)))
check("audio_event_count", audio_events >= BUDGETS["min_audio_events"], str(audio_events))
check("vfx_event_count", vfx_events >= BUDGETS["min_vfx_events"], str(vfx_events))
check("all_entries_have_provenance", all(row["provenance"] == "repo_owned_original_text_asset" for row in asset_rows), "repo-owned")
check("all_entries_have_runtime_metrics", all(row["vertices"] >= 3 and row["indices"] >= 3 and row["materials"] >= 1 for row in asset_rows), "vertices/indices/materials")
check("total_gltf_bytes_within_budget", total_gltf_bytes <= BUDGETS["max_total_gltf_bytes"], str(total_gltf_bytes))
check("runtime_mesh_bytes_within_budget", total_runtime_mesh_bytes <= BUDGETS["max_total_runtime_mesh_bytes"], str(total_runtime_mesh_bytes))
check("preview_bytes_within_budget", total_preview_bytes <= BUDGETS["max_total_preview_bytes"], str(total_preview_bytes))
check("texture_bytes_within_budget", total_texture_bytes <= BUDGETS["max_total_texture_bytes"], str(total_texture_bytes))
check(
    "arena_material_maps_exist",
    all(Path(path).is_file() for row in asset_rows for path in row["material_maps"].values()),
    "base/normal/orm maps for arena entries",
)
check("vertex_budget", total_vertices <= BUDGETS["max_total_vertices"], str(total_vertices))
check("index_budget", total_indices <= BUDGETS["max_total_indices"], str(total_indices))
check("triangle_budget", total_triangles <= BUDGETS["max_total_triangles"], str(total_triangles))
check("material_budget", total_materials <= BUDGETS["max_materials"], str(total_materials))
check("primitive_budget", total_primitives <= BUDGETS["max_primitives"], str(total_primitives))
check("embedded_buffer_budget", total_embedded_buffer_bytes <= BUDGETS["max_embedded_buffer_bytes"], str(total_embedded_buffer_bytes))
arena_rows = [row for row in asset_rows if row["kind"] == "arenas"]
check(
    "arena_identity_vertex_floor",
    bool(arena_rows) and all(row["vertices"] >= BUDGETS["min_arena_identity_vertices"] for row in arena_rows),
    ", ".join(f"{row['id']}={row['vertices']}" for row in arena_rows),
)
check(
    "arena_identity_triangle_floor",
    bool(arena_rows) and all(row["triangles"] >= BUDGETS["min_arena_identity_triangles"] for row in arena_rows),
    ", ".join(f"{row['id']}={row['triangles']}" for row in arena_rows),
)

passed = all(item["passed"] for item in checks)
summary = {
    "entry_count": len(entries),
    "kind_counts": kind_counts,
    "audio_event_count": audio_events,
    "vfx_event_count": vfx_events,
    "source_bytes": source_bytes,
    "total_gltf_bytes": total_gltf_bytes,
    "total_runtime_mesh_bytes": total_runtime_mesh_bytes,
    "total_preview_bytes": total_preview_bytes,
    "total_texture_bytes": total_texture_bytes,
    "total_vertices": total_vertices,
    "total_indices": total_indices,
    "total_triangles": total_triangles,
    "total_materials": total_materials,
    "total_primitives": total_primitives,
    "total_embedded_buffer_bytes": total_embedded_buffer_bytes,
}
artifact_hashes = {
    "runtime_manifest": sha256(manifest_path) if manifest_path.is_file() else "",
    "asset_validation_report": sha256(assets_root / "asset_validation_report.md") if (assets_root / "asset_validation_report.md").is_file() else "",
    "gltf_validation_report": sha256(assets_root / "gltf_validation_report.md") if (assets_root / "gltf_validation_report.md").is_file() else "",
    "asset_provenance_report": sha256(assets_root / "asset_provenance_report.md") if (assets_root / "asset_provenance_report.md").is_file() else "",
}

report = {
    "schema": "oathyard.asset_budget.v1",
    "product": "OATHYARD",
    "purpose": "local_asset_budget_regression_gate",
    "budgets": BUDGETS,
    "summary": summary,
    "assets": asset_rows,
    "artifact_hashes": artifact_hashes,
    "presentation_only": True,
    "truth_mutation": False,
    "external_khronos_validation_claimed": False,
    "owner_visual_acceptance_claimed": False,
    "public_demo_ready": False,
    "release_candidate_ready": False,
    "checks": checks,
    "passed": passed,
}
(out / "asset_budget.json").write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

lines = [
    "# OATHYARD Asset Budget Audit",
    "",
    f"Status: {'PASSED' if passed else 'FAILED'}",
    "",
    "- Purpose: `local_asset_budget_regression_gate`",
    f"- Entries: `{summary['entry_count']}`",
    f"- Kind counts: `fighters {kind_counts.get('fighters', 0)}` `weapons {kind_counts.get('weapons', 0)}` `armor {kind_counts.get('armor', 0)}` `arenas {kind_counts.get('arenas', 0)}`",
    f"- Audio events: `{audio_events}`",
    f"- VFX events: `{vfx_events}`",
    f"- Source bytes: `{source_bytes}`",
    f"- glTF bytes: `{total_gltf_bytes}` / `{BUDGETS['max_total_gltf_bytes']}`",
    f"- Runtime mesh bytes: `{total_runtime_mesh_bytes}` / `{BUDGETS['max_total_runtime_mesh_bytes']}`",
    f"- Preview bytes: `{total_preview_bytes}` / `{BUDGETS['max_total_preview_bytes']}`",
    f"- Texture bytes: `{total_texture_bytes}` / `{BUDGETS['max_total_texture_bytes']}`",
    f"- Vertices: `{total_vertices}` / `{BUDGETS['max_total_vertices']}`",
    f"- Indices: `{total_indices}` / `{BUDGETS['max_total_indices']}`",
    f"- Triangles: `{total_triangles}` / `{BUDGETS['max_total_triangles']}`",
    f"- Materials: `{total_materials}` / `{BUDGETS['max_materials']}`",
    f"- Primitives: `{total_primitives}` / `{BUDGETS['max_primitives']}`",
    f"- Arena identity floor: `vertices >= {BUDGETS['min_arena_identity_vertices']}` `triangles >= {BUDGETS['min_arena_identity_triangles']}`",
    "- External Khronos validation claimed: `false`",
    "- Owner visual acceptance claimed: `false`",
    "- Public demo ready: `false`",
    "- Release candidate ready: `false`",
    "",
    "## Checks",
    "",
]
for item in checks:
    state = "pass" if item["passed"] else "fail"
    lines.append(f"- `{item['id']}`: `{state}` - {item['detail']}")
lines.extend(["", "## Largest glTF Assets", ""])
for row in sorted(asset_rows, key=lambda item: item["gltf_bytes"], reverse=True)[:8]:
    lines.append(
        f"- `{row['id']}` `{row['kind']}`: glTF `{row['gltf_bytes']}` bytes, textures `{row['texture_bytes']}` bytes, vertices `{row['vertices']}`, triangles `{row['triangles']}`"
    )
lines.append("")
lines.append("This audit is a local budget regression gate. It does not prove production asset quality, external glTF validator acceptance, store-art readiness, or owner visual acceptance.")
(out / "asset_budget_report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")

if not passed:
    print("asset budget audit failed", file=sys.stderr)
    sys.exit(1)
PY
