#!/usr/bin/env python3
"""Fail-closed structural audit and detail image-rollup renderer for OATHYARD model candidates.

This lane audits candidate packages under assets/source/model_candidates/<run_id>,
assets/model_candidates/<run_id>, and artifacts/model_candidates/<run_id> without
promoting them into assets/gltf or the deterministic low-poly regression lane.
"""
from __future__ import annotations

import argparse
import binascii
import hashlib
import json
import math
import struct
import sys
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_RUN_ID = "t_73291be5"
REQUIRED_COUNTS = {"fighter": 6, "weapon": 8, "armor": 6, "arena": 2}
BUDGETS = {
    "fighter": {"min": 18_000, "target_max": 30_000, "hard_max": 40_000},
    "weapon_one_handed": {"min": 800, "target_max": 2_500, "hard_max": 4_000},
    "weapon_two_handed": {"min": 1_200, "target_max": 3_500, "hard_max": 5_000},
    "shield": {"min": 2_000, "target_max": 5_000, "hard_max": 8_000},
    "armor": {"min": 500, "target_max": 5_000, "hard_max": 8_000},
    "arena": {"min": 2_000, "target_max": 60_000, "hard_max": 90_000},
}
REQUIRED_CLIPS = {"idle", "walk", "attack"}
REQUIRED_STYLE_BASIS = {
    "docs/design/ART_DIRECTION_BRIEF.md",
    "content/oathyard_content.manifest",
    "assets/source/oysrc/traditions.oysrc",
    "assets/source/oysrc/weapons.oysrc",
    "assets/source/oysrc/armor.oysrc",
    "assets/source/oysrc/arenas.oysrc",
}
COMPONENT_INFO = {5120: ("b", 1), 5121: ("B", 1), 5122: ("h", 2), 5123: ("H", 2), 5125: ("I", 4), 5126: ("f", 4)}
TYPE_COMPS = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def png_chunk(kind: bytes, data: bytes) -> bytes:
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", binascii.crc32(kind + data) & 0xFFFFFFFF)


def write_png_rgb(path: Path, width: int, height: int, pixels) -> None:
    raw = bytearray()
    for row in pixels:
        raw.append(0)
        for r, g, b in row:
            raw.extend([max(0, min(255, int(r))), max(0, min(255, int(g))), max(0, min(255, int(b)))])
    data = (
        b"\x89PNG\r\n\x1a\n"
        + png_chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
        + png_chunk(b"IDAT", zlib.compress(bytes(raw), 9))
        + png_chunk(b"IEND", b"")
    )
    path.write_bytes(data)


def is_png(path: Path) -> bool:
    return path.is_file() and path.read_bytes()[:8] == b"\x89PNG\r\n\x1a\n"


def accessor_layout(gltf, accessor_id: int):
    acc = gltf["accessors"][accessor_id]
    view = gltf["bufferViews"][acc["bufferView"]]
    fmt_char, size = COMPONENT_INFO[acc["componentType"]]
    comps = TYPE_COMPS[acc["type"]]
    offset = int(view.get("byteOffset", 0)) + int(acc.get("byteOffset", 0))
    stride = int(view.get("byteStride", size * comps))
    return offset, stride, int(acc["count"]), fmt_char, comps


def read_accessor(gltf, buf: bytes, accessor_id: int):
    start, stride, count, fmt_char, comps = accessor_layout(gltf, accessor_id)
    fmt = "<" + fmt_char * comps
    rows = []
    for index in range(count):
        rows.append(struct.unpack_from(fmt, buf, start + index * stride))
    return rows


def resolve_uri(base: Path, uri: str) -> Path:
    if uri.startswith("data:"):
        raise ValueError("data URI is not permitted in the model-candidate lane")
    return (base.parent / uri).resolve()


def material_color(gltf, primitive) -> tuple[int, int, int]:
    material_index = primitive.get("material", 0)
    material = gltf.get("materials", [{}])[material_index]
    rgba = material.get("pbrMetallicRoughness", {}).get("baseColorFactor", [0.55, 0.50, 0.42, 1.0])
    return (
        max(0, min(255, int(round(float(rgba[0]) * 255)))),
        max(0, min(255, int(round(float(rgba[1]) * 255)))),
        max(0, min(255, int(round(float(rgba[2]) * 255)))),
    )


def load_triangles(gltf_path: Path):
    gltf = read_json(gltf_path)
    buffers = gltf.get("buffers", [])
    if len(buffers) != 1:
        raise ValueError(f"{gltf_path} expected exactly one external buffer")
    bin_path = resolve_uri(gltf_path, buffers[0].get("uri", ""))
    buf = bin_path.read_bytes()
    triangles = []
    for mesh in gltf.get("meshes", []):
        for primitive in mesh.get("primitives", []):
            pos_acc = primitive.get("attributes", {}).get("POSITION")
            if pos_acc is None or primitive.get("indices") is None:
                continue
            positions = read_accessor(gltf, buf, int(pos_acc))
            indices = [int(row[0]) for row in read_accessor(gltf, buf, int(primitive["indices"]))]
            color = material_color(gltf, primitive)
            for i in range(0, len(indices) - 2, 3):
                a, b, c = indices[i], indices[i + 1], indices[i + 2]
                if a < len(positions) and b < len(positions) and c < len(positions):
                    triangles.append((color, positions[a], positions[b], positions[c]))
    return triangles


def fill_tri(pixels, p0, p1, p2, color):
    height, width = len(pixels), len(pixels[0])
    min_x = max(0, int(math.floor(min(p0[0], p1[0], p2[0]))))
    max_x = min(width - 1, int(math.ceil(max(p0[0], p1[0], p2[0]))))
    min_y = max(0, int(math.floor(min(p0[1], p1[1], p2[1]))))
    max_y = min(height - 1, int(math.ceil(max(p0[1], p1[1], p2[1]))))
    denom = (p1[1] - p2[1]) * (p0[0] - p2[0]) + (p2[0] - p1[0]) * (p0[1] - p2[1])
    if abs(denom) < 1e-7:
        return
    for y in range(min_y, max_y + 1):
        for x in range(min_x, max_x + 1):
            w0 = ((p1[1] - p2[1]) * (x - p2[0]) + (p2[0] - p1[0]) * (y - p2[1])) / denom
            w1 = ((p2[1] - p0[1]) * (x - p2[0]) + (p0[0] - p2[0]) * (y - p2[1])) / denom
            w2 = 1.0 - w0 - w1
            if w0 >= -0.01 and w1 >= -0.01 and w2 >= -0.01:
                pixels[y][x] = color


def darken(color, factor: float) -> tuple[int, int, int]:
    return (
        max(0, min(255, int(color[0] * factor))),
        max(0, min(255, int(color[1] * factor))),
        max(0, min(255, int(color[2] * factor))),
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-id", default=DEFAULT_RUN_ID)
    parser.add_argument("--require-vision-audit", action="store_true")
    args = parser.parse_args()

    run_id = args.run_id
    src_root = ROOT / "assets_src" / "model_candidates" / run_id
    pkg_root = ROOT / "assets" / "model_candidates" / run_id
    art_root = ROOT / "artifacts" / "model_candidates" / run_id
    validation_dir = art_root / "validation"
    visual_dir = art_root / "visual_audit"
    validation_dir.mkdir(parents=True, exist_ok=True)
    visual_dir.mkdir(parents=True, exist_ok=True)

    manifest_path = pkg_root / "model_candidate_manifest.json"
    source_manifest_path = src_root / "model_source_manifest.json"
    generator_path = ROOT / "tools" / "model_candidates" / "publish_t73291be5_animation_ready.py"
    animation_validation_path = validation_dir / "model_candidate_animation_ready_validation.json"
    visual_audit_md = visual_dir / "model_candidate_detail_visual_audit.md"
    visual_audit_json = visual_dir / "model_candidate_detail_visual_audit.json"

    failures: list[str] = []
    checks: list[dict] = []

    def check(name: str, passed: bool, detail, asset_id: str | None = None) -> None:
        record = {"name": name, "passed": bool(passed), "detail": str(detail)}
        if asset_id:
            record["asset_id"] = asset_id
        checks.append(record)
        if not passed:
            prefix = f"{asset_id}: " if asset_id else ""
            failures.append(f"{prefix}{name}: {detail}")

    check("source_root_exists", src_root.is_dir(), src_root.relative_to(ROOT) if src_root.exists() else src_root)
    check("package_root_exists", pkg_root.is_dir(), pkg_root.relative_to(ROOT) if pkg_root.exists() else pkg_root)
    check("artifact_root_exists", art_root.is_dir(), art_root.relative_to(ROOT) if art_root.exists() else art_root)
    check("manifest_exists", manifest_path.is_file(), manifest_path.relative_to(ROOT) if manifest_path.exists() else manifest_path)
    check("source_manifest_exists", source_manifest_path.is_file(), source_manifest_path.relative_to(ROOT) if source_manifest_path.exists() else source_manifest_path)
    check("generator_tool_exists", generator_path.is_file(), generator_path.relative_to(ROOT) if generator_path.exists() else generator_path)
    check("animation_validation_exists", animation_validation_path.is_file(), animation_validation_path.relative_to(ROOT) if animation_validation_path.exists() else animation_validation_path)

    manifest = read_json(manifest_path) if manifest_path.is_file() else {"entries": []}
    source_manifest = read_json(source_manifest_path) if source_manifest_path.is_file() else {"entries": []}
    animation_validation = read_json(animation_validation_path) if animation_validation_path.is_file() else {}

    check("manifest_schema", manifest.get("schema") == "oathyard.model_candidate_manifest.v2", manifest.get("schema"))
    check("source_manifest_schema", source_manifest.get("schema") == "oathyard.model_candidate_source_manifest.v2", source_manifest.get("schema"))
    for key in ["truth_authoritative", "public_demo_ready", "release_candidate_ready", "owner_visual_acceptance", "external_khronos_validation_claimed"]:
        expected = False
        actual = manifest.get(key)
        check(f"manifest_{key}_false", actual is expected, actual)
    source_basis = set(manifest.get("source_basis", []))
    check("source_basis_complete", REQUIRED_STYLE_BASIS.issubset(source_basis), sorted(REQUIRED_STYLE_BASIS - source_basis))
    check("animation_validation_passed", animation_validation.get("passed") is True, animation_validation.get("failures", []))

    entries = manifest.get("entries", [])
    counts: dict[str, int] = {key: 0 for key in REQUIRED_COUNTS}
    rows = []
    source_entries_by_id = {item.get("id"): item for item in source_manifest.get("entries", [])}
    for entry in entries:
        asset_id = entry.get("id", "<missing>")
        kind = entry.get("kind", "")
        counts[kind] = counts.get(kind, 0) + 1
        source_path = ROOT / entry.get("source", "")
        gltf_path = ROOT / entry.get("runtime_gltf", "")
        bin_path = ROOT / entry.get("runtime_bin", "")
        texture_paths = [ROOT / path for path in entry.get("textures", [])]
        budget = BUDGETS.get(entry.get("budget_kind", ""), {})
        triangles = int(entry.get("triangles", -1))
        check("source_json_exists", source_path.is_file(), entry.get("source", ""), asset_id)
        source_json = read_json(source_path) if source_path.is_file() else {}
        check("source_json_schema", source_json.get("schema") == "oathyard.model_candidate_source.v2", source_json.get("schema"), asset_id)
        check("source_json_id_matches", source_json.get("asset_id") == asset_id, source_json.get("asset_id"), asset_id)
        check("source_json_provenance_repo_owned", source_json.get("provenance") == "repo_owned_original_procedural_model_candidate", source_json.get("provenance"), asset_id)
        check("source_manifest_has_entry", asset_id in source_entries_by_id, asset_id, asset_id)
        check("gltf_exists", gltf_path.is_file(), entry.get("runtime_gltf", ""), asset_id)
        check("bin_exists", bin_path.is_file(), entry.get("runtime_bin", ""), asset_id)
        check("three_texture_sidecars_listed", len(texture_paths) == 3, len(texture_paths), asset_id)
        for tex in texture_paths:
            check("texture_sidecar_png", is_png(tex), tex.relative_to(ROOT) if tex.exists() else tex, asset_id)
        check("triangle_min_budget", bool(budget) and triangles >= int(budget.get("min", 10**9)), f"{triangles} / min {budget.get('min')}", asset_id)
        check("triangle_hard_budget", bool(budget) and triangles <= int(budget.get("hard_max", -1)), f"{triangles} / hard {budget.get('hard_max')}", asset_id)
        for key in ["truth_authoritative", "public_demo_ready", "release_candidate_ready", "owner_visual_acceptance"]:
            check(f"entry_{key}_false", entry.get(key) is False, entry.get(key), asset_id)
        gltf_metrics = {"materials": 0, "primitives": 0, "images": 0, "external_buffer": False, "z_depth": 0.0}
        if gltf_path.is_file():
            try:
                gltf = read_json(gltf_path)
                check("gltf_version_2", gltf.get("asset", {}).get("version") == "2.0", gltf.get("asset", {}).get("version"), asset_id)
                buffers = gltf.get("buffers", [])
                check("gltf_one_external_bin", len(buffers) == 1 and not str(buffers[0].get("uri", "")).startswith("data:"), buffers, asset_id)
                if buffers and bin_path.is_file():
                    check("gltf_buffer_length_matches_bin", int(buffers[0].get("byteLength", -1)) == bin_path.stat().st_size, f"{buffers[0].get('byteLength')} vs {bin_path.stat().st_size}", asset_id)
                accessors = gltf.get("accessors", [])
                if accessors:
                    mins = accessors[0].get("min", [0, 0, 0])
                    maxs = accessors[0].get("max", [0, 0, 0])
                    if len(mins) == 3 and len(maxs) == 3:
                        gltf_metrics["z_depth"] = float(maxs[2]) - float(mins[2])
                gltf_metrics.update(
                    {
                        "materials": len(gltf.get("materials", [])),
                        "primitives": sum(len(mesh.get("primitives", [])) for mesh in gltf.get("meshes", [])),
                        "images": len(gltf.get("images", [])),
                        "external_buffer": bool(buffers and not str(buffers[0].get("uri", "")).startswith("data:")),
                    }
                )
                check("gltf_nonzero_z_depth", gltf_metrics["z_depth"] > 0.0, gltf_metrics["z_depth"], asset_id)
                check("gltf_has_materials", gltf_metrics["materials"] >= 1, gltf_metrics["materials"], asset_id)
                check("gltf_has_primitives", gltf_metrics["primitives"] >= 1, gltf_metrics["primitives"], asset_id)
                images = gltf.get("images", [])
                image_uris = [image.get("uri", "") for image in images]
                check("gltf_images_base_normal_orm", any("_base.png" in uri for uri in image_uris) and any("_normal.png" in uri for uri in image_uris) and any("_orm.png" in uri for uri in image_uris), image_uris, asset_id)
                for material in gltf.get("materials", []):
                    pbr = material.get("pbrMetallicRoughness", {})
                    check("material_has_base_texture", "baseColorTexture" in pbr, material.get("name", ""), asset_id)
                    check("material_has_orm_texture", "metallicRoughnessTexture" in pbr, material.get("name", ""), asset_id)
                    check("material_has_normal_texture", "normalTexture" in material, material.get("name", ""), asset_id)
                    check("material_has_occlusion_texture", "occlusionTexture" in material, material.get("name", ""), asset_id)
                if kind == "fighter":
                    attrs_seen = set()
                    for mesh in gltf.get("meshes", []):
                        for primitive in mesh.get("primitives", []):
                            attrs_seen.update(primitive.get("attributes", {}).keys())
                    clips = {animation.get("name") for animation in gltf.get("animations", [])}
                    check("fighter_has_skin", len(gltf.get("skins", [])) >= 1, len(gltf.get("skins", [])), asset_id)
                    check("fighter_has_joint_weight_attributes", {"JOINTS_0", "WEIGHTS_0"}.issubset(attrs_seen), sorted(attrs_seen), asset_id)
                    check("fighter_required_clips_present", REQUIRED_CLIPS.issubset(clips), sorted(REQUIRED_CLIPS - clips), asset_id)
            except Exception as error:  # noqa: BLE001 - exact error belongs in audit artifact.
                check("gltf_parse_and_structure", False, repr(error), asset_id)
        rows.append(
            {
                "id": asset_id,
                "kind": kind,
                "budget_kind": entry.get("budget_kind", ""),
                "triangles": triangles,
                "vertices": entry.get("vertices", 0),
                "materials": gltf_metrics["materials"],
                "primitives": gltf_metrics["primitives"],
                "z_depth": gltf_metrics["z_depth"],
                "source": entry.get("source", ""),
                "runtime_gltf": entry.get("runtime_gltf", ""),
                "runtime_bin": entry.get("runtime_bin", ""),
            }
        )

    for kind, required in REQUIRED_COUNTS.items():
        check(f"count_{kind}", counts.get(kind, 0) >= required, f"{counts.get(kind, 0)} / {required}")

    if args.require_vision_audit:
        check("vision_audit_markdown_present", visual_audit_md.is_file() and visual_audit_md.stat().st_size > 0, visual_audit_md.relative_to(ROOT) if visual_audit_md.exists() else visual_audit_md)
        check("vision_audit_json_present", visual_audit_json.is_file() and visual_audit_json.stat().st_size > 0, visual_audit_json.relative_to(ROOT) if visual_audit_json.exists() else visual_audit_json)
        if visual_audit_json.is_file():
            vision = read_json(visual_audit_json)
            check("vision_audit_readiness_false", vision.get("owner_visual_acceptance") is False and vision.get("public_demo_ready") is False and vision.get("release_candidate_ready") is False, vision)

    tool_hashes = {}
    for path in [generator_path, Path(__file__).resolve()]:
        if path.is_file():
            tool_hashes[path.relative_to(ROOT).as_posix()] = sha256_file(path)

    report = {
        "schema": "oathyard.model_candidate_lane_audit.v1",
        "product": "OATHYARD",
        "run_id": run_id,
        "candidate_package": pkg_root.relative_to(ROOT).as_posix(),
        "candidate_source_root": src_root.relative_to(ROOT).as_posix(),
        "artifact_root": art_root.relative_to(ROOT).as_posix(),
        "source_manifest": source_manifest_path.relative_to(ROOT).as_posix(),
        "manifest": manifest_path.relative_to(ROOT).as_posix(),
        "vision_audit_markdown": visual_audit_md.relative_to(ROOT).as_posix(),
        "vision_audit_json": visual_audit_json.relative_to(ROOT).as_posix(),
        "tool_hashes": tool_hashes,
        "counts": counts,
        "required_counts": REQUIRED_COUNTS,
        "budgets": BUDGETS,
        "entries": rows,
        "checks": checks,
        "failed_check_count": len(failures),
        "failures": failures,
        "truth_authoritative": False,
        "public_demo_ready": False,
        "release_candidate_ready": False,
        "owner_visual_acceptance": False,
        "external_khronos_validation_claimed": False,
        "native_renderer_acceptance_claimed": False,
        "passed": not failures,
    }

    json_path = validation_dir / "model_candidate_lane_audit.json"
    md_path = validation_dir / "model_candidate_lane_audit_report.md"
    json_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    lines = [
        "# OATHYARD model-candidate lane audit",
        "",
        f"Status: {'PASSED' if report['passed'] else 'FAILED'}",
        f"- Run id: `{run_id}`",
        f"- Candidate source root: `{report['candidate_source_root']}`",
        f"- Candidate package: `{report['candidate_package']}`",
        f"- Artifact root: `{report['artifact_root']}`",
        f"- Manifest: `{report['manifest']}`",
        f"- Source manifest: `{report['source_manifest']}`",
        f"- Vision audit markdown: `{report['vision_audit_markdown']}`",
        "- Truth authoritative: `false`",
        "- Public demo ready: `false`",
        "- Release candidate ready: `false`",
        "- Owner visual acceptance: `false`",
        "- External Khronos validation claimed: `false`",
        "- Native renderer acceptance claimed: `false`",
        "",
        "## Counts",
        "",
    ]
    for kind, required in REQUIRED_COUNTS.items():
        lines.append(f"- `{kind}`: `{counts.get(kind, 0)}` / `{required}`")
    lines.extend(["", "## Per-asset metrics", ""])
    for row in rows:
        lines.append(
            f"- `{row['id']}` `{row['kind']}` budget `{row['budget_kind']}` vertices `{row['vertices']}` "
            f"triangles `{row['triangles']}` materials `{row['materials']}` primitives `{row['primitives']}` z-depth `{row['z_depth']:.4f}`"
        )
    lines.extend(["", "## Checks", ""])
    for item in checks:
        prefix = f"`{item.get('asset_id')}` " if item.get("asset_id") else ""
        lines.append(f"- {prefix}`{item['name']}`: `{'pass' if item['passed'] else 'fail'}` - {item['detail']}")
    if failures:
        lines.extend(["", "## Failures", ""])
        lines.extend(f"- {failure}" for failure in failures)
    lines.extend(
        [
            "",
            "## Scope boundary",
            "",
            "This audit validates the source-backed candidate lane structure, local glTF/package shape, triangle-budget floors/ceilings, material sidecars, fighter rig/motion structure, and readiness boundaries. It does not claim high-fidelity completion, external Khronos validation, Blender/DCC round trip, native renderer acceptance, owner visual acceptance, public-demo readiness, or release-candidate readiness.",
        ]
    )
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    if failures:
        print(f"model candidate lane audit failed with {len(failures)} failure(s); see {md_path}", file=sys.stderr)
        return 1
    print(json.dumps({"passed": True, "audit_json": json_path.as_posix(), "audit_report": md_path.as_posix()}, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
