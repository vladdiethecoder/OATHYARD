#!/usr/bin/env python3
import base64
import binascii
import hashlib
import json
import struct
import sys
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTENT_MANIFEST = ROOT / "content" / "oathyard_content.manifest"
ASSETS_SRC = ROOT / "assets_src"
ASSETS = ROOT / "assets"
RUNTIME_MANIFEST = ASSETS / "runtime_manifest.json"
PROVENANCE_REPORT = ASSETS / "asset_provenance_report.md"
VALIDATION_REPORT = ASSETS / "asset_validation_report.md"
GLTF_VALIDATION_REPORT = ASSETS / "gltf_validation_report.md"
GLTF_DIR = ASSETS / "gltf"
TEXTURE_DIR = ASSETS / "textures"
PBR_MATERIAL_SOURCE = ASSETS_SRC / "materials" / "pbr_materials.oysrc"
PBR_MATERIAL_DIR = ASSETS / "materials"
PBR_MATERIAL_MANIFEST = PBR_MATERIAL_DIR / "pbr_surface_manifest.json"
PBR_MATERIAL_ATLAS = PBR_MATERIAL_DIR / "pbr_surface_evidence.json"

PBR_REQUIRED_CHANNELS = [
    "albedo",
    "roughness_metallic",
    "normal_height",
    "edge_wear",
    "dirt",
    "blood_wetness",
    "cloth_grain",
    "steel_scratches",
    "leather_strain",
    "stone_dust",
    "stitching",
    "hair_skin_variation",
    "material_ids",
]

REQUIRED_COUNTS = {
    "fighters": 6,
    "weapons": 8,
    "armor": 6,
    "arenas": 2,
}

FORBIDDEN_SOURCE_MARKERS = [
    "copied_from",
    "scraped",
    "borrowed",
    "unlicensed",
    "placeholder",
    "todo_asset",
]

ASSET_RECORD_KIND = {
    "fighters": "fighter",
    "weapons": "weapon",
    "armor": "armor",
    "arenas": "arena",
}

CANONICAL_TRUTH_JOINTS = [
    "root",
    "spine_lower",
    "spine_upper",
    "neck_head",
    "shoulder_r",
    "elbow_r",
    "wrist_r",
    "shoulder_l",
    "elbow_l",
    "wrist_l",
    "hip_r",
    "knee_r",
    "ankle_r",
    "hip_l",
    "knee_l",
    "ankle_l",
]


def main() -> int:
    if len(sys.argv) != 2 or sys.argv[1] not in {"build", "validate"}:
        print("usage: asset_pipeline.py build|validate", file=sys.stderr)
        return 2
    if sys.argv[1] == "build":
        build()
    validate()
    return 0


def parse_manifest():
    if not CONTENT_MANIFEST.exists():
        raise SystemExit(f"missing content manifest: {CONTENT_MANIFEST}")
    sections = {}
    current = None
    for raw in CONTENT_MANIFEST.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or "=" in line and current is None:
            continue
        if line.startswith("[") and line.endswith("]"):
            current = line[1:-1]
            sections[current] = []
            continue
        if current:
            sections[current].append(line)
    return sections


def build():
    sections = parse_manifest()
    ASSETS.mkdir(exist_ok=True)
    GLTF_DIR.mkdir(parents=True, exist_ok=True)
    TEXTURE_DIR.mkdir(parents=True, exist_ok=True)
    PBR_MATERIAL_DIR.mkdir(parents=True, exist_ok=True)
    pbr_surfaces = parse_pbr_material_source()
    pbr_manifest = write_pbr_material_library(pbr_surfaces)
    entries = []
    for kind in ["fighters", "weapons", "armor", "arenas"]:
        for row in sections.get(kind, []):
            parts = row.split(":")
            asset_id = parts[0]
            source = Path(parts[-1])
            source_path = ROOT / source
            source_text = source_path.read_text(encoding="utf-8")
            digest = sha256_text(source_text + "\n" + row)
            material_maps = {}
            if kind == "arenas":
                material_maps = write_arena_material_maps(
                    asset_id,
                    asset_source_fields(asset_id, kind, source_path),
                    digest,
                )
            entries.append(
                {
                    "id": asset_id,
                    "kind": kind,
                    "source": source.as_posix(),
                    "preview": "",
                    "runtime_mesh": f"assets/runtime/{asset_id}.mesh.json",
                    "runtime_gltf": f"assets/runtime/candidate/gltf/{asset_id}.gltf",
                    "material_maps": material_maps,
                    "pbr_material_profile": material_profile_for_entry(asset_id, kind, pbr_surfaces),
                    "hash": digest,
                    "provenance": "repo_owned_original_text_asset",
                }
            )
    runtime_dir = ASSETS / "runtime"
    runtime_dir.mkdir(exist_ok=True)
    for entry in entries:
        runtime_path = ROOT / entry["runtime_mesh"]
        runtime_payload = {
            "schema": "oathyard.runtime_asset.v1",
            "id": entry["id"],
            "kind": entry["kind"],
            "source": entry["source"],
            "hash": entry["hash"],
            "runtime_gltf": entry["runtime_gltf"],
            "pbr_material_profile": entry["pbr_material_profile"],
            "pbr_material_schema": "oathyard.pbr_surface.v1",
            "pbr_material_manifest": PBR_MATERIAL_MANIFEST.relative_to(ROOT).as_posix(),
            "truth_joint_mapping": "canonical_16_plus_grips"
            if entry["kind"] == "fighters"
            else "not_applicable",
            "presentation_only": entry["kind"] not in {"weapons", "armor"},
        }
        if entry["kind"] == "arenas":
            fields = asset_source_fields(entry["id"], entry["kind"], ROOT / entry["source"])
            runtime_payload.update(arena_runtime_metadata(entry, fields))
        runtime_path.write_text(
            json.dumps(runtime_payload, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        gltf_path = ROOT / entry["runtime_gltf"]
        gltf_path.write_text(render_gltf(entry), encoding="utf-8")
    manifest = {
        "schema": "oathyard.assets.v1",
        "product": "OATHYARD",
        "public_demo_ready": False,
        "release_candidate_ready": False,
        "pbr_material_manifest": PBR_MATERIAL_MANIFEST.relative_to(ROOT).as_posix(),
        "pbr_material_atlas": PBR_MATERIAL_ATLAS.relative_to(ROOT).as_posix(),
        "pbr_material_source": PBR_MATERIAL_SOURCE.relative_to(ROOT).as_posix(),
        "pbr_material_hash": pbr_manifest["material_hash"],
        "pbr_all_required_channels_covered": pbr_manifest["all_required_channels_covered"],
        "entries": entries,
        "asset_hash": sha256_text(json.dumps(entries, sort_keys=True)),
    }
    RUNTIME_MANIFEST.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    PROVENANCE_REPORT.write_text(render_provenance(entries, manifest["asset_hash"]), encoding="utf-8")


def validate():
    sections = parse_manifest()
    failures = []
    pbr_surfaces = []
    try:
        pbr_surfaces = parse_pbr_material_source()
    except SystemExit as exc:
        failures.append(str(exc))
    failures.extend(validate_pbr_material_library(pbr_surfaces))
    for kind, minimum in REQUIRED_COUNTS.items():
        count = len(sections.get(kind, []))
        if count < minimum:
            failures.append(f"{kind} count {count} is below required {minimum}")
    for kind in ["fighters", "weapons", "armor", "arenas"]:
        for row in sections.get(kind, []):
            parts = row.split(":")
            asset_id = parts[0]
            source = ROOT / parts[-1]
            if not source.exists():
                failures.append(f"{asset_id} missing source {source}")
                continue
            text = source.read_text(encoding="utf-8").lower()
            if "provenance=repo_owned" not in text:
                failures.append(f"{asset_id} source lacks repo-owned provenance")
            for marker in FORBIDDEN_SOURCE_MARKERS:
                if marker in text:
                    failures.append(f"{asset_id} source contains forbidden marker {marker}")
            if kind == "fighters" and "rig_joint_order=root,spine_lower,spine_upper" not in text:
                failures.append(f"{asset_id} fighter source lacks canonical rig mapping")
            if kind == "arenas":
                fields = asset_source_fields(asset_id, kind, source)
                for required in [
                    "environment_profile",
                    "material_zones",
                    "material_maps",
                    "lighting_anchors",
                    "landmarks",
                    "floor_contact",
                    "composition_profile",
                    "scale_reference",
                    "silhouette_context",
                    "playable_space",
                    "atmosphere_hooks",
                    "capture_ids",
                    "originality_notes",
                ]:
                    if not fields.get(required):
                        failures.append(f"{asset_id} arena source lacks {required}")
                if len(list_field(fields, "material_zones")) < 6:
                    failures.append(f"{asset_id} arena source needs at least six material zones")
                if len(list_field(fields, "playable_space")) < 4:
                    failures.append(f"{asset_id} arena source needs at least four playable-space cues")
                if len(list_field(fields, "atmosphere_hooks")) < 4:
                    failures.append(f"{asset_id} arena source needs at least four atmosphere hooks")
                if set(list_field(fields, "capture_ids")) != {"establishing", "gameplay", "contact"}:
                    failures.append(f"{asset_id} arena source capture ids must be establishing, gameplay, contact")
                if asset_id == "oathyard_verdict_ring":
                    if "judgment_axis" not in fields.get("composition_profile", ""):
                        failures.append(f"{asset_id} arena source must preserve judgment-axis composition")
                    if "entry_break" not in fields.get("silhouette_context", ""):
                        failures.append(f"{asset_id} arena source must preserve visible entry-break silhouette")
                if asset_id == "training_yard":
                    if "rectangular_drill" not in fields.get("composition_profile", ""):
                        failures.append(f"{asset_id} arena source must preserve rectangular-drill composition")
                    if "no_ring_backdrop" not in fields.get("silhouette_context", ""):
                        failures.append(f"{asset_id} arena source must quarantine verdict-ring-like backdrop language")
            runtime = ASSETS / "runtime" / f"{asset_id}.mesh.json"
            gltf = GLTF_DIR / f"{asset_id}.gltf"
            if RUNTIME_MANIFEST.exists() and not runtime.exists():
                failures.append(f"{asset_id} missing runtime asset {runtime}")
            if RUNTIME_MANIFEST.exists() and not gltf.exists():
                failures.append(f"{asset_id} missing glTF runtime asset {gltf}")
            elif RUNTIME_MANIFEST.exists():
                failures.extend(validate_gltf(gltf, asset_id, kind, parts[-1]))
            if kind == "arenas" and RUNTIME_MANIFEST.exists() and runtime.exists():
                failures.extend(validate_arena_runtime_asset(runtime, asset_id))
    if not RUNTIME_MANIFEST.exists():
        failures.append("missing runtime manifest; run ./tools/build_assets.sh")
    else:
        data = json.loads(RUNTIME_MANIFEST.read_text(encoding="utf-8"))
        if data.get("schema") != "oathyard.assets.v1":
            failures.append("runtime manifest schema mismatch")
        if data.get("public_demo_ready") is not False:
            failures.append("public_demo_ready must remain false")
        if data.get("release_candidate_ready") is not False:
            failures.append("release_candidate_ready must remain false")
        for entry in data.get("entries", []):
            if "runtime_gltf" not in entry:
                failures.append(f"{entry.get('id', '<unknown>')} missing runtime_gltf manifest path")
            if not entry.get("pbr_material_profile"):
                failures.append(f"{entry.get('id', '<unknown>')} missing pbr_material_profile")
            if entry.get("kind") == "arenas":
                maps = entry.get("material_maps", {})
                if set(maps) != {"base", "normal", "orm"}:
                    failures.append(f"{entry.get('id', '<unknown>')} missing arena material map paths in manifest")
        if data.get("pbr_all_required_channels_covered") is not True:
            failures.append("runtime manifest pbr material channel coverage failed")
        if data.get("pbr_material_manifest") != PBR_MATERIAL_MANIFEST.relative_to(ROOT).as_posix():
            failures.append("runtime manifest pbr material manifest path mismatch")
    VALIDATION_REPORT.parent.mkdir(exist_ok=True)
    VALIDATION_REPORT.write_text(render_validation(failures), encoding="utf-8")
    GLTF_VALIDATION_REPORT.write_text(render_gltf_validation(failures), encoding="utf-8")
    if failures:
        for failure in failures:
            print(f"asset validation failed: {failure}", file=sys.stderr)
        raise SystemExit(1)
    print("asset validation passed")


def parse_pbr_material_source():
    if not PBR_MATERIAL_SOURCE.exists():
        raise SystemExit(f"missing PBR material source: {PBR_MATERIAL_SOURCE}")
    surfaces = []
    text = PBR_MATERIAL_SOURCE.read_text(encoding="utf-8")
    if "provenance=repo_owned" not in text:
        raise SystemExit("PBR material source lacks repo-owned provenance")
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or not line.startswith("surface "):
            continue
        tokens = line.split()
        fields = {"id": tokens[1]}
        for token in tokens[2:]:
            if "=" not in token:
                raise SystemExit(f"malformed PBR material token '{token}' in {tokens[1]}")
            key, value = token.split("=", 1)
            fields[key] = value
        surfaces.append(normalize_pbr_surface(fields))
    if not surfaces:
        raise SystemExit("PBR material source contains no surface rows")
    return surfaces


def normalize_pbr_surface(fields):
    required = [
        "id", "applies", "material_ids", "albedo", "metallic", "roughness", "normal", "height",
        "edge_wear", "dirt", "blood_wetness", "cloth_grain", "steel_scratches", "leather_strain",
        "stone_dust", "stitching", "hair_skin_variation",
    ]
    missing = [key for key in required if key not in fields]
    if missing:
        raise SystemExit(f"PBR surface {fields.get('id', '<unknown>')} missing {','.join(missing)}")
    albedo = [int(part) for part in fields["albedo"].split(",")]
    if len(albedo) != 3 or any(part < 0 or part > 255 for part in albedo):
        raise SystemExit(f"PBR surface {fields['id']} has invalid albedo")
    surface = {
        "schema": "oathyard.pbr_surface.v1",
        "id": fields["id"],
        "applies_to": fields["applies"].split(","),
        "material_ids": fields["material_ids"].split(","),
        "albedo_rgb": albedo,
        "metallic_permille": int(fields["metallic"]),
        "roughness_permille": int(fields["roughness"]),
        "normal_permille": int(fields["normal"]),
        "height_permille": int(fields["height"]),
        "edge_wear_permille": int(fields["edge_wear"]),
        "dirt_permille": int(fields["dirt"]),
        "blood_wetness_permille": int(fields["blood_wetness"]),
        "cloth_grain_permille": int(fields["cloth_grain"]),
        "steel_scratches_permille": int(fields["steel_scratches"]),
        "leather_strain_permille": int(fields["leather_strain"]),
        "stone_dust_permille": int(fields["stone_dust"]),
        "stitching_permille": int(fields["stitching"]),
        "hair_skin_variation_permille": int(fields["hair_skin_variation"]),
        "truth_authoritative": False,
        "presentation_only": True,
    }
    for key, value in surface.items():
        if key.endswith("_permille") and (value < 0 or value > 1000):
            raise SystemExit(f"PBR surface {fields['id']} has out-of-range {key}")
    return surface


def write_pbr_material_library(surfaces):
    coverage = pbr_material_coverage(surfaces)
    pixels = pbr_material_atlas_pixels(surfaces, 1024, 512)
    write_pbr_material_evidence(PBR_MATERIAL_ATLAS, 1024, 512, pixels)
    manifest = {
        "schema": "oathyard.pbr_surface_manifest.v1",
        "product": "OATHYARD",
        "source": PBR_MATERIAL_SOURCE.relative_to(ROOT).as_posix(),
        "source_hash": sha256_file(PBR_MATERIAL_SOURCE),
        "surface_count": len(surfaces),
        "required_channels": coverage,
        "all_required_channels_covered": all(item["covered"] for item in coverage),
        "asset_classes_covered": sorted({klass for surface in surfaces for klass in surface["applies_to"]}),
        "nonvisual_material_evidence": PBR_MATERIAL_ATLAS.relative_to(ROOT).as_posix(),
        "nonvisual_material_evidence_hash": sha256_file(PBR_MATERIAL_ATLAS),
        "truth_authoritative": False,
        "presentation_only": True,
        "material_maps_affect_replay_hash": False,
        "surfaces": surfaces,
    }
    manifest["material_hash"] = sha256_text(json.dumps(manifest["surfaces"], sort_keys=True))
    PBR_MATERIAL_MANIFEST.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return manifest


def validate_pbr_material_library(surfaces):
    failures = []
    if not surfaces:
        return ["PBR material source yielded no surfaces"]
    if not PBR_MATERIAL_MANIFEST.is_file():
        failures.append("missing PBR material manifest; run ./tools/build_assets.sh")
    if not PBR_MATERIAL_ATLAS.is_file():
        failures.append("missing PBR material atlas; run ./tools/build_assets.sh")
    classes = {klass for surface in surfaces for klass in surface["applies_to"]}
    for klass in ["weapons", "armor", "arenas", "fighters"]:
        if klass not in classes:
            failures.append(f"PBR material source missing asset class {klass}")
    for item in pbr_material_coverage(surfaces):
        if not item["covered"]:
            failures.append(f"PBR material source missing required channel {item['channel']}")
    if PBR_MATERIAL_MANIFEST.is_file():
        try:
            manifest = json.loads(PBR_MATERIAL_MANIFEST.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            failures.append(f"PBR material manifest JSON parse failed: {exc}")
        else:
            if manifest.get("schema") != "oathyard.pbr_surface_manifest.v1":
                failures.append("PBR material manifest schema mismatch")
            if manifest.get("all_required_channels_covered") is not True:
                failures.append("PBR material manifest required channel coverage failed")
            if manifest.get("material_maps_affect_replay_hash") is not False:
                failures.append("PBR material manifest must not affect replay hash")
    return failures


def pbr_material_coverage(surfaces):
    def any_positive(key):
        return any(surface.get(key, 0) > 0 for surface in surfaces)

    checks = {
        "albedo": all(surface.get("albedo_rgb") for surface in surfaces),
        "roughness_metallic": any_positive("roughness_permille") and any_positive("metallic_permille"),
        "normal_height": any_positive("normal_permille") and any_positive("height_permille"),
        "edge_wear": any_positive("edge_wear_permille"),
        "dirt": any_positive("dirt_permille"),
        "blood_wetness": any_positive("blood_wetness_permille"),
        "cloth_grain": any_positive("cloth_grain_permille"),
        "steel_scratches": any_positive("steel_scratches_permille"),
        "leather_strain": any_positive("leather_strain_permille"),
        "stone_dust": any_positive("stone_dust_permille"),
        "stitching": any_positive("stitching_permille"),
        "hair_skin_variation": any_positive("hair_skin_variation_permille"),
        "material_ids": all(surface.get("material_ids") for surface in surfaces),
    }
    return [{"channel": channel, "covered": bool(checks.get(channel))} for channel in PBR_REQUIRED_CHANNELS]


def material_profile_for_entry(asset_id, kind, surfaces):
    if kind == "weapons":
        preferred = ["ash_wood_grain_dented", "tempered_steel_edge_worn"] if "spear" in asset_id else ["tempered_steel_edge_worn", "ash_wood_grain_dented"]
    elif kind == "armor":
        if "mail" in asset_id:
            preferred = ["riveted_mail_oiled", "strained_buff_leather"]
        elif "gambeson" in asset_id or "fencer" in asset_id:
            preferred = ["quilted_linen_stitched", "strained_buff_leather"]
        else:
            preferred = ["tempered_steel_edge_worn", "strained_buff_leather"]
    elif kind == "fighters":
        preferred = ["skin_hair_variation", "quilted_linen_stitched"]
    elif kind == "arenas":
        preferred = ["chalked_stone_dust", "ash_wood_grain_dented"]
    else:
        preferred = []
    by_id = {surface["id"]: surface for surface in surfaces}
    selected = [by_id[surface_id] for surface_id in preferred if surface_id in by_id]
    if not selected:
        selected = [surface for surface in surfaces if kind in surface["applies_to"]][:1]
    return {
        "schema": "oathyard.pbr_material_profile.v1",
        "asset_id": asset_id,
        "asset_kind": kind,
        "source": PBR_MATERIAL_SOURCE.relative_to(ROOT).as_posix(),
        "base_surface_ids": [surface["id"] for surface in selected],
        "material_ids": sorted({material_id for surface in selected for material_id in surface["material_ids"]}),
        "presentation_only": True,
        "truth_authoritative": False,
        "material_maps_affect_replay_hash": False,
    }


def pbr_material_atlas_pixels(surfaces, width, height):
    pixels = []
    cols = 4
    cell_w = width // cols
    cell_h = height // 2
    for y in range(height):
        row = min(1, y // cell_h)
        for x in range(width):
            col = min(cols - 1, x // cell_w)
            index = min(len(surfaces) - 1, row * cols + col)
            pixels.append(pbr_surface_pixel(surfaces[index], x - col * cell_w, y - row * cell_h))
    return pixels


def pbr_surface_pixel(surface, x, y):
    jitter = int(sha256_text(f"{surface['id']}:{x}:{y}")[:2], 16) - 128
    r, g, b = surface["albedo_rgb"]
    grain = ((surface["normal_permille"] + surface["height_permille"]) * jitter) // 900
    scratch = surface["steel_scratches_permille"] // 18 if (x + y * 3) % 17 == 0 else 0
    stitch = surface["stitching_permille"] // 28 if (x % 37 == 0 or y % 23 == 0) else 0
    cloth = -(surface["cloth_grain_permille"] // 34) if (x + y) % 9 == 0 else 0
    leather = surface["leather_strain_permille"] // 32 if (x * 2 - y) % 29 == 0 else 0
    stone = surface["stone_dust_permille"] // 20 if (x * 5 + y * 7) % 41 < 3 else 0
    wet = surface["blood_wetness_permille"] // 10
    dirt = surface["dirt_permille"] // 35
    edge = surface["edge_wear_permille"] // 18 if x < 8 or y < 8 or x > 240 or y > 240 else 0
    return (
        max(0, min(255, r + grain + scratch + stitch + leather + stone + edge - dirt + wet + cloth)),
        max(0, min(255, g + grain + scratch // 2 + stitch + leather // 2 + stone - dirt - wet // 5 + cloth)),
        max(0, min(255, b + grain + scratch // 3 + stitch // 2 + stone - dirt - wet // 3 + cloth)),
    )


def write_pbr_material_evidence(path: Path, width: int, height: int, pixels):
    payload = bytes(channel for rgb in pixels for channel in rgb)
    evidence = {
        "schema": "oathyard.pbr_surface_evidence.v1",
        "width": width,
        "height": height,
        "pixel_hash": hashlib.sha256(payload).hexdigest(),
        "distinct_color_count": len(set(pixels)),
        "visual_evidence": False,
        "native_3d_render_capture": False,
        "truth_mutation": False,
    }
    path.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def render_gltf(entry):
    fields = asset_source_fields(entry["id"], entry["kind"], ROOT / entry["source"])
    positions, indices = gltf_geometry(entry, fields)
    buffer_bytes = pack_gltf_buffer(positions, indices)
    position_byte_length = len(positions) * 12
    index_byte_offset = position_byte_length
    index_byte_length = len(indices) * 2
    mins = [min(point[axis] for point in positions) for axis in range(3)]
    maxs = [max(point[axis] for point in positions) for axis in range(3)]
    node_extras = {
        "asset_id": entry["id"],
        "canonical_truth_joints": CANONICAL_TRUTH_JOINTS
        if entry["kind"] == "fighters"
        else [],
        "external_validator": "not_available_local_structural_validation_only",
        "grip_frames": ["grip_r", "grip_l"] if entry["kind"] == "fighters" else [],
        "hash": entry["hash"],
        "kind": entry["kind"],
        "pbr_material_manifest": PBR_MATERIAL_MANIFEST.relative_to(ROOT).as_posix(),
        "pbr_material_profile": entry.get("pbr_material_profile", {}),
        "pbr_material_schema": "oathyard.pbr_surface.v1",
        "presentation_only": entry["kind"] not in {"weapons", "armor"},
        "product": "OATHYARD",
        "provenance": entry["provenance"],
        "source": entry["source"],
        "truth_authoritative": False,
    }
    if entry["kind"] == "arenas":
        node_extras.update(arena_gltf_extras(entry, fields))
    document = {
        "accessors": [
            {
                "bufferView": 0,
                "byteOffset": 0,
                "componentType": 5126,
                "count": len(positions),
                "max": [round(value, 6) for value in maxs],
                "min": [round(value, 6) for value in mins],
                "type": "VEC3",
            },
            {
                "bufferView": 1,
                "byteOffset": 0,
                "componentType": 5123,
                "count": len(indices),
                "type": "SCALAR",
            },
        ],
        "asset": {
            "copyright": "PENDING / UNLICENSED",
            "generator": "OATHYARD deterministic text asset pipeline",
            "version": "2.0",
        },
        "bufferViews": [
            {
                "buffer": 0,
                "byteLength": position_byte_length,
                "byteOffset": 0,
                "target": 34962,
            },
            {
                "buffer": 0,
                "byteLength": index_byte_length,
                "byteOffset": index_byte_offset,
                "target": 34963,
            },
        ],
        "buffers": [
            {
                "byteLength": len(buffer_bytes),
                "uri": "data:application/octet-stream;base64,"
                + base64.b64encode(buffer_bytes).decode("ascii"),
            }
        ],
        "materials": gltf_materials(entry, fields),
        "meshes": [
            {
                "name": f"{entry['id']}_mesh",
                "primitives": [
                    {
                        "attributes": {"POSITION": 0},
                        "indices": 1,
                        "material": 0,
                        "mode": 4,
                    }
                ],
            }
        ],
        "nodes": [
            {
                "extras": node_extras,
                "mesh": 0,
                "name": entry["id"],
            }
        ],
        "scene": 0,
        "scenes": [{"name": "OATHYARD generated asset scene", "nodes": [0]}],
    }
    if entry["kind"] == "arenas":
        maps = entry.get("material_maps", {})
        document["images"] = [
            {
                "mimeType": "image/png",
                "name": f"{entry['id']}_{name}",
                "uri": relative_path_from_gltf(path),
            }
            for name, path in maps.items()
        ]
        document["textures"] = [
            {"name": f"{entry['id']}_{name}_texture", "source": index}
            for index, name in enumerate(maps.keys())
        ]
    return json.dumps(document, indent=2, sort_keys=True) + "\n"


def asset_source_fields(asset_id, kind, source_path):
    record_kind = ASSET_RECORD_KIND[kind]
    for raw in source_path.read_text(encoding="utf-8").splitlines():
        tokens = raw.split()
        if len(tokens) >= 2 and tokens[0] == record_kind and tokens[1] == asset_id:
            fields = {}
            for token in tokens[2:]:
                if "=" in token:
                    key, value = token.split("=", 1)
                    fields[key] = value
            return fields
    raise SystemExit(f"{asset_id} missing source record in {source_path}")


def gltf_geometry(entry, fields):
    kind = entry["kind"]
    if kind == "fighters":
        mass = int_field(fields, "body_mass_g", 82000)
        reach = int_field(fields, "reach_bias_mm", 0) / 1000.0
        height = 1.58 + min(max(mass - 70000, 0), 32000) / 100000.0
        width = 0.34 + min(max(mass - 74000, 0), 26000) / 260000.0
        front = [
            (0.0, 0.0),
            (0.0, height),
            (-width, height * 0.66),
            (width, height * 0.66),
            (-width * 0.66, 0.0),
            (width * 0.66, 0.0),
            (reach, height * 0.52),
        ]
        front_indices = [0, 2, 1, 0, 1, 3, 0, 4, 2, 0, 3, 5, 2, 6, 3]
        return extruded_mesh(front, front_indices, 0.16)
    if kind == "weapons":
        length = int_field(fields, "length_mm", 900) / 1000.0
        mesh = fields.get("mesh", "")
        if "round" in mesh:
            radius = max(length * 0.5, 0.18)
            front = [(0.0, 0.0)]
            for x, y in [
                (0.0, radius),
                (radius * 0.866, radius * 0.5),
                (radius * 0.866, -radius * 0.5),
                (0.0, -radius),
                (-radius * 0.866, -radius * 0.5),
                (-radius * 0.866, radius * 0.5),
            ]:
                front.append((x, y))
            front_indices = []
            for side in range(1, 7):
                front_indices.extend([0, side, 1 if side == 6 else side + 1])
            return extruded_mesh(front, front_indices, 0.10)
        half_width = 0.026 if "spear" in mesh else 0.045
        if "maul" in entry["id"] or "axe" in entry["id"]:
            half_width = 0.09
        front = [
            (0.0, -half_width),
            (length, -half_width * 0.55),
            (length, half_width * 0.55),
            (0.0, half_width),
            (length * 0.78, -half_width * 2.2),
            (length * 0.94, 0.0),
            (length * 0.78, half_width * 2.2),
        ]
        front_indices = [0, 1, 2, 0, 2, 3, 4, 5, 6]
        return extruded_mesh(front, front_indices, 0.08)
    if kind == "armor":
        pieces = fields.get("pieces", "").split(",")
        height = 0.72 + min(len([part for part in pieces if part]), 4) * 0.06
        shoulder = 0.42
        waist = 0.30
        front = [
            (-shoulder, height),
            (shoulder, height),
            (-waist, 0.0),
            (waist, 0.0),
            (0.0, height * 0.56),
        ]
        front_indices = [0, 2, 4, 0, 4, 1, 1, 4, 3, 2, 3, 4]
        return extruded_mesh(front, front_indices, 0.18)
    if kind == "arenas":
        return arena_environment_mesh(entry["id"], fields)
    raise SystemExit(f"unsupported asset kind {kind}")


ARENA_PERIMETER_32 = [
    (1000, 0),
    (981, 195),
    (924, 383),
    (831, 556),
    (707, 707),
    (556, 831),
    (383, 924),
    (195, 981),
    (0, 1000),
    (-195, 981),
    (-383, 924),
    (-556, 831),
    (-707, 707),
    (-831, 556),
    (-924, 383),
    (-981, 195),
    (-1000, 0),
    (-981, -195),
    (-924, -383),
    (-831, -556),
    (-707, -707),
    (-556, -831),
    (-383, -924),
    (-195, -981),
    (0, -1000),
    (195, -981),
    (383, -924),
    (556, -831),
    (707, -707),
    (831, -556),
    (924, -383),
    (981, -195),
]


def arena_environment_mesh(asset_id, fields):
    """Source-backed arena identity mesh: readable floor, rim, landmarks, and context.

    This is still the local deterministic presentation-art lane, not production-renderer
    acceptance. The extra geometry gives current-run captures enough silhouette/context
    to falsify the previous "flat token arena" failure without mutating combat truth.
    """
    radius = int_field(fields, "radius_mm", 5000) / 1000.0
    if asset_id == "training_yard":
        return training_yard_environment_mesh(radius, fields)
    oval_y = 1.0
    rim_height = 0.34
    anchor_height = 0.72
    positions = [(0.0, 0.0, 0.0)]

    lower_start = len(positions)
    for x_permille, y_permille in ARENA_PERIMETER_32:
        positions.append((radius * x_permille / 1000.0, radius * y_permille * oval_y / 1000.0, 0.0))
    upper_start = len(positions)
    for x_permille, y_permille in ARENA_PERIMETER_32:
        positions.append(
            (
                radius * 1.018 * x_permille / 1000.0,
                radius * 1.018 * y_permille * oval_y / 1000.0,
                rim_height,
            )
        )

    indices = []
    perimeter_count = len(ARENA_PERIMETER_32)
    for side in range(perimeter_count):
        lower_a = lower_start + side
        lower_b = lower_start + ((side + 1) % perimeter_count)
        upper_a = upper_start + side
        upper_b = upper_start + ((side + 1) % perimeter_count)
        indices.extend([0, lower_a, lower_b])
        append_quad(indices, lower_a, lower_b, upper_b, upper_a)

    append_ring_band(positions, indices, radius, oval_y, 0.54, 0.59, 0.026, step=2)
    append_verdict_ring_identity(positions, indices, radius, oval_y, rim_height, anchor_height)
    return positions, indices


def training_yard_environment_mesh(radius, fields):
    """Rectangular practical drill yard, deliberately not a verdict-ring variant."""
    positions = []
    indices = []
    yard_x = radius * 0.68
    yard_y = radius * 0.46
    append_flat_rect(positions, indices, -yard_x, -yard_y, 0.000, yard_x, yard_y)
    append_flat_rect(positions, indices, -yard_x, -yard_y - 0.16, 0.075, yard_x, -yard_y - 0.09)
    append_flat_rect(positions, indices, -yard_x, yard_y + 0.09, 0.075, yard_x, yard_y + 0.16)
    append_flat_rect(positions, indices, -yard_x - 0.16, -yard_y, 0.075, -yard_x - 0.09, yard_y)
    append_flat_rect(positions, indices, yard_x + 0.09, -yard_y, 0.075, yard_x + 0.16, yard_y)
    append_training_yard_identity(positions, indices, radius, 0.70, 0.10, 0.62)
    return positions, indices


def append_verdict_ring_identity(positions, indices, radius, oval_y, rim_height, anchor_height):
    append_ring_band(positions, indices, radius, oval_y, 0.18, 0.23, 0.046, step=2)
    append_box(positions, indices, -1.62, radius * 0.78, rim_height, 1.62, radius * 0.90, 0.78)
    append_box(positions, indices, -2.20, radius * 0.92, 0.10, 2.20, radius * 0.98, 1.04)
    append_box(positions, indices, -1.05, radius * 0.985, 0.54, 1.05, radius * 1.01, 1.22)
    append_box(positions, indices, -2.45, radius * 0.735, 0.44, 2.45, radius * 0.765, 0.62)
    append_box(positions, indices, -2.75, radius * 0.690, 0.22, -2.55, radius * 0.880, 0.86)
    append_box(positions, indices, 2.55, radius * 0.690, 0.22, 2.75, radius * 0.880, 0.86)
    append_box(positions, indices, -0.26, -radius * 0.88, rim_height, 0.26, -radius * 0.72, 0.64)
    append_box(positions, indices, -0.82, -radius * 0.92, 0.04, -0.56, -radius * 0.71, 0.82)
    append_box(positions, indices, 0.56, -radius * 0.92, 0.04, 0.82, -radius * 0.71, 0.82)
    for center_x, center_y in [
        (0.0, radius * 0.70 * oval_y),
        (radius * 0.70, 0.0),
        (0.0, -radius * 0.70 * oval_y),
        (-radius * 0.70, 0.0),
    ]:
        append_pyramid(positions, indices, center_x, center_y, 0.18, rim_height, anchor_height)
    for center_x, center_y in [
        (-radius * 0.56, radius * 0.42 * oval_y),
        (radius * 0.56, radius * 0.42 * oval_y),
        (-radius * 0.56, -radius * 0.42 * oval_y),
        (radius * 0.56, -radius * 0.42 * oval_y),
    ]:
        append_box(positions, indices, center_x - 0.10, center_y - 0.10, 0.02, center_x + 0.10, center_y + 0.10, 0.50)
        append_box(positions, indices, center_x - 0.34, center_y - 0.018, 0.38, center_x + 0.34, center_y + 0.018, 0.46)
    for y_scale in [-0.34, 0.34]:
        append_flat_rect(positions, indices, -radius * 0.50, radius * y_scale * oval_y, 0.052, radius * 0.50, radius * y_scale * oval_y + 0.024)
    for x_scale in [-0.40, -0.22, 0.22, 0.40]:
        append_flat_rect(positions, indices, radius * x_scale - 0.012, -radius * 0.48, 0.048, radius * x_scale + 0.012, radius * 0.48)


def append_training_yard_identity(positions, indices, radius, oval_y, rim_height, anchor_height):
    inset_x = radius * 0.58
    inset_y = radius * 0.38 * oval_y
    z0 = 0.030
    z1 = 0.060
    append_flat_rect(positions, indices, -inset_x, -inset_y, z1, inset_x, -inset_y + 0.045)
    append_flat_rect(positions, indices, -inset_x, inset_y - 0.045, z1, inset_x, inset_y)
    append_flat_rect(positions, indices, -inset_x, -inset_y, z1, -inset_x + 0.045, inset_y)
    append_flat_rect(positions, indices, inset_x - 0.045, -inset_y, z1, inset_x, inset_y)
    append_flat_rect(positions, indices, -0.035, -inset_y, z1, 0.035, inset_y)
    for x0, x1 in [(-1.18, -0.42), (0.42, 1.18)]:
        append_flat_rect(positions, indices, x0, -0.34, z1, x1, -0.12)
        append_flat_rect(positions, indices, x0, 0.12, z1, x1, 0.34)
    append_box(positions, indices, -0.82, -0.08, z0, -0.30, 0.08, z1)
    append_box(positions, indices, 0.30, -0.08, z0, 0.82, 0.08, z1)
    for y_scale in [-0.30, -0.15, 0.15, 0.30]:
        append_flat_rect(positions, indices, -inset_x * 0.88, radius * y_scale * oval_y, z1, inset_x * 0.88, radius * y_scale * oval_y + 0.036)
    for x_scale in [-0.32, -0.16, 0.16, 0.32]:
        append_box(positions, indices, radius * x_scale - 0.018, -inset_y * 0.70, z0, radius * x_scale + 0.018, inset_y * 0.70, z1)
    for center_x, center_y in [
        (-radius * 0.76, -radius * 0.48 * oval_y),
        (radius * 0.76, -radius * 0.48 * oval_y),
        (radius * 0.76, radius * 0.48 * oval_y),
        (-radius * 0.76, radius * 0.48 * oval_y),
    ]:
        append_box(positions, indices, center_x - 0.09, center_y - 0.09, 0.0, center_x + 0.09, center_y + 0.09, 0.58)
        append_pyramid(positions, indices, center_x, center_y, 0.16, 0.58, anchor_height)
    append_box(positions, indices, radius * 0.45, -radius * 0.10 * oval_y, 0.02, radius * 0.72, radius * 0.10 * oval_y, 0.52)
    for offset in [-0.16, 0.0, 0.16]:
        append_box(positions, indices, radius * 0.50, (offset - 0.010) * radius * oval_y, 0.46, radius * 0.76, (offset + 0.010) * radius * oval_y, 0.68)
    append_box(positions, indices, radius * 0.47, radius * 0.25 * oval_y, 0.02, radius * 0.77, radius * 0.35 * oval_y, 0.72)
    append_box(positions, indices, radius * 0.49, radius * 0.27 * oval_y, 0.68, radius * 0.75, radius * 0.33 * oval_y, 0.88)
    append_box(positions, indices, -radius * 0.74, -radius * 0.12 * oval_y, 0.02, -radius * 0.58, radius * 0.12 * oval_y, 0.36)
    append_box(positions, indices, -radius * 0.80, -radius * 0.18 * oval_y, 0.02, -radius * 0.72, radius * 0.18 * oval_y, 0.48)
    append_box(positions, indices, -radius * 0.82, radius * 0.25 * oval_y, 0.02, -radius * 0.62, radius * 0.40 * oval_y, 0.34)
    for x_scale in [-0.82, -0.76, -0.70, 0.70, 0.76, 0.82]:
        append_box(positions, indices, radius * x_scale - 0.030, -radius * 0.58 * oval_y, 0.04, radius * x_scale + 0.030, radius * 0.58 * oval_y, 0.12)


def append_ring_band(positions, indices, radius, oval_y, inner_scale, outer_scale, z, step=1):
    perimeter = ARENA_PERIMETER_32[::step]
    inner_start = len(positions)
    for x_permille, y_permille in perimeter:
        positions.append((radius * inner_scale * x_permille / 1000.0, radius * inner_scale * y_permille * oval_y / 1000.0, z))
    outer_start = len(positions)
    for x_permille, y_permille in perimeter:
        positions.append((radius * outer_scale * x_permille / 1000.0, radius * outer_scale * y_permille * oval_y / 1000.0, z + 0.016))
    count = len(perimeter)
    for side in range(count):
        append_quad(
            indices,
            inner_start + side,
            inner_start + ((side + 1) % count),
            outer_start + ((side + 1) % count),
            outer_start + side,
        )


def append_quad(indices, a, b, c, d):
    indices.extend([a, b, c, a, c, d])


def append_flat_rect(positions, indices, min_x, min_y, z, max_x, max_y):
    start = len(positions)
    positions.extend(
        [
            (min_x, min_y, z),
            (max_x, min_y, z),
            (max_x, max_y, z),
            (min_x, max_y, z),
        ]
    )
    append_quad(indices, start, start + 1, start + 2, start + 3)


def append_box(positions, indices, min_x, min_y, min_z, max_x, max_y, max_z):
    start = len(positions)
    positions.extend(
        [
            (min_x, min_y, min_z),
            (max_x, min_y, min_z),
            (max_x, max_y, min_z),
            (min_x, max_y, min_z),
            (min_x, min_y, max_z),
            (max_x, min_y, max_z),
            (max_x, max_y, max_z),
            (min_x, max_y, max_z),
        ]
    )
    b0, b1, b2, b3, t0, t1, t2, t3 = range(start, start + 8)
    append_quad(indices, b0, b1, b2, b3)
    append_quad(indices, t3, t2, t1, t0)
    append_quad(indices, b0, t0, t1, b1)
    append_quad(indices, b1, t1, t2, b2)
    append_quad(indices, b2, t2, t3, b3)
    append_quad(indices, b3, t3, t0, b0)


def append_pyramid(positions, indices, center_x, center_y, half, base_z, tip_z):
    start = len(positions)
    positions.extend(
        [
            (center_x - half, center_y - half, base_z),
            (center_x + half, center_y - half, base_z),
            (center_x + half, center_y + half, base_z),
            (center_x - half, center_y + half, base_z),
            (center_x, center_y, tip_z),
        ]
    )
    b0, b1, b2, b3, tip = range(start, start + 5)
    indices.extend(
        [
            b0,
            b1,
            tip,
            b1,
            b2,
            tip,
            b2,
            b3,
            tip,
            b3,
            b0,
            tip,
            b0,
            b2,
            b1,
            b0,
            b3,
            b2,
        ]
    )


def extruded_mesh(front_xy, front_indices, depth):
    front_z = depth / 2.0
    back_z = -depth / 2.0
    positions = [(x, y, front_z) for x, y in front_xy] + [(x, y, back_z) for x, y in front_xy]
    count = len(front_xy)
    indices = list(front_indices)
    for a, b, c in chunks(front_indices, 3):
        indices.extend([c + count, b + count, a + count])

    edge_counts = {}
    edge_order = []
    for a, b, c in chunks(front_indices, 3):
        for u, v in [(a, b), (b, c), (c, a)]:
            key = tuple(sorted((u, v)))
            edge_counts[key] = edge_counts.get(key, 0) + 1
            if key not in edge_order:
                edge_order.append(key)
    for a, b in edge_order:
        if edge_counts[a, b] != 1:
            continue
        indices.extend([a, b, b + count, a, b + count, a + count])
    return positions, indices


def chunks(items, size):
    for index in range(0, len(items), size):
        chunk = items[index : index + size]
        if len(chunk) == size:
            yield chunk


def int_field(fields, key, default):
    try:
        return int(fields.get(key, default))
    except ValueError as exc:
        raise SystemExit(f"invalid integer field {key}={fields.get(key)}") from exc


def pack_gltf_buffer(positions, indices):
    data = bytearray()
    for point in positions:
        data.extend(struct.pack("<3f", *point))
    for index in indices:
        data.extend(struct.pack("<H", index))
    return bytes(data)


def color_factor(digest):
    return [round(int(digest[i : i + 2], 16) / 255.0, 6) for i in (0, 2, 4)] + [1.0]


def list_field(fields, key):
    return [part for part in fields.get(key, "").split(",") if part]


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            digest.update(chunk)
    return digest.hexdigest()


def relative_path_from_gltf(path: str) -> str:
    candidate = Path(path)
    if candidate.is_absolute():
        return "../" + candidate.relative_to(ASSETS).as_posix()
    return "../" + candidate.relative_to("assets").as_posix()


def arena_material_map_hashes(material_maps):
    return {name: sha256_file(ROOT / path) for name, path in material_maps.items()}


def arena_candidate_source_hash(asset_id):
    candidate = ASSETS_SRC / "model_candidates" / "t_73291be5" / "arenas" / f"{asset_id}.model_source.json"
    if not candidate.is_file():
        return {"candidate_source": "", "candidate_source_sha256": ""}
    return {
        "candidate_source": candidate.relative_to(ROOT).as_posix(),
        "candidate_source_sha256": sha256_file(candidate),
    }


def arena_runtime_metadata(entry, fields):
    material_maps = entry.get("material_maps", {})
    metadata = {
        "radius_mm": int_field(fields, "radius_mm", 0),
        "ground": fields.get("ground", ""),
        "collision": fields.get("collision", ""),
        "camera_anchor": fields.get("camera", ""),
        "lighting": fields.get("lighting", ""),
        "visual_identity": fields.get("visual", ""),
        "environment_profile": fields.get("environment_profile", ""),
        "material_zones": list_field(fields, "material_zones"),
        "material_maps": material_maps,
        "material_map_hashes": arena_material_map_hashes(material_maps),
        "lighting_anchors": list_field(fields, "lighting_anchors"),
        "duel_readable_landmarks": list_field(fields, "landmarks"),
        "floor_contact_readability": list_field(fields, "floor_contact"),
        "composition_profile": fields.get("composition_profile", ""),
        "scale_reference": fields.get("scale_reference", ""),
        "silhouette_context": fields.get("silhouette_context", ""),
        "playable_space": list_field(fields, "playable_space"),
        "atmosphere_hooks": list_field(fields, "atmosphere_hooks"),
        "capture_ids": list_field(fields, "capture_ids"),
        "originality_notes": fields.get("originality_notes", ""),
        "production_target_layer": "runtime_presentation",
        "truth_authoritative": False,
        "truth_mutation": False,
        "owner_visual_acceptance_claimed": False,
        "public_demo_ready": False,
        "release_candidate_ready": False,
    }
    metadata.update(arena_candidate_source_hash(entry["id"]))
    return metadata


def arena_gltf_extras(entry, fields):
    extras = arena_runtime_metadata(entry, fields)
    extras["source_backed_runtime_art"] = True
    extras["renderer_consumption_boundary"] = "presentation_after_truth_hash"
    return extras


def gltf_materials(entry, fields):
    if entry["kind"] != "arenas":
        return [
            {
                "extras": {
                    "pbr_material_manifest": PBR_MATERIAL_MANIFEST.relative_to(ROOT).as_posix(),
                    "pbr_material_profile": entry.get("pbr_material_profile", {}),
                    "pbr_material_schema": "oathyard.pbr_surface.v1",
                    "source": "deterministic_repo_owned_text_asset",
                    "style": "inked_grim",
                },
                "name": f"{entry['id']}_material",
                "pbrMetallicRoughness": {
                    "baseColorFactor": color_factor(entry["hash"]),
                    "metallicFactor": 0.0,
                    "roughnessFactor": 0.94,
                },
            }
        ]
    materials = []
    palette = arena_palette(entry["id"])
    zones = list_field(fields, "material_zones") or ["stone_floor"]
    for index, zone in enumerate(zones[:6]):
        base_color, roughness, metallic = palette[index % len(palette)]
        materials.append(
            {
                "extras": {
                    "material_family": zone,
                    "pbr_material_manifest": PBR_MATERIAL_MANIFEST.relative_to(ROOT).as_posix(),
                    "pbr_material_profile": entry.get("pbr_material_profile", {}),
                    "pbr_material_schema": "oathyard.pbr_surface.v1",
                    "source": "deterministic_repo_owned_text_asset",
                    "truth_authoritative": False,
                },
                "name": f"{entry['id']}_{zone}",
                "normalTexture": {"index": 1, "scale": 0.65},
                "occlusionTexture": {"index": 2, "strength": 0.62},
                "pbrMetallicRoughness": {
                    "baseColorFactor": base_color,
                    "baseColorTexture": {"index": 0},
                    "metallicFactor": metallic,
                    "metallicRoughnessTexture": {"index": 2},
                    "roughnessFactor": roughness,
                },
            }
        )
    return materials


def arena_palette(asset_id):
    if asset_id == "training_yard":
        return [
            ([0.37, 0.26, 0.18, 1.0], 0.88, 0.0),
            ([0.78, 0.72, 0.58, 1.0], 0.92, 0.0),
            ([0.30, 0.22, 0.14, 1.0], 0.74, 0.0),
            ([0.95, 0.79, 0.42, 1.0], 0.68, 0.0),
            ([0.17, 0.18, 0.17, 1.0], 0.58, 0.0),
            ([0.42, 0.34, 0.24, 1.0], 0.84, 0.0),
        ]
    return [
        ([0.38, 0.39, 0.38, 1.0], 0.86, 0.0),
        ([0.90, 0.86, 0.72, 1.0], 0.91, 0.0),
        ([0.10, 0.11, 0.12, 1.0], 0.55, 0.35),
        ([0.25, 0.38, 0.48, 1.0], 0.63, 0.0),
        ([0.47, 0.42, 0.35, 1.0], 0.78, 0.0),
        ([0.50, 0.12, 0.10, 1.0], 0.73, 0.0),
    ]


def write_arena_material_maps(asset_id, fields, digest):
    maps = {
        "base": TEXTURE_DIR / f"{asset_id}_base.png",
        "normal": TEXTURE_DIR / f"{asset_id}_normal.png",
        "orm": TEXTURE_DIR / f"{asset_id}_orm.png",
    }
    for map_kind, path in maps.items():
        write_png_rgb(path, 32, 32, arena_map_pixels(asset_id, fields, digest, map_kind, 32, 32))
    return {name: path.relative_to(ROOT).as_posix() for name, path in maps.items()}


def arena_map_pixels(asset_id, fields, digest, map_kind, width, height):
    zones = list_field(fields, "material_zones") or [asset_id]
    palette = arena_palette(asset_id)
    pixels = []
    for y in range(height):
        for x in range(width):
            zone_index = (x * len(zones)) // width
            jitter = int(sha256_text(f"{asset_id}:{map_kind}:{digest}:{x}:{y}:{zones[zone_index]}")[:2], 16)
            base_color, roughness, metallic = palette[zone_index % len(palette)]
            if map_kind == "base":
                pixels.append(
                    tuple(
                        int(max(0, min(255, channel * 255 + (jitter % 19) - 9)))
                        for channel in base_color[:3]
                    )
                )
            elif map_kind == "normal":
                pixels.append((128 + (jitter % 17) - 8, 128 + ((jitter // 3) % 17) - 8, 238))
            elif map_kind == "orm":
                occlusion = 190 + (jitter % 34)
                rough = int(max(0, min(255, roughness * 255)))
                metal = int(max(0, min(255, metallic * 255)))
                pixels.append((occlusion, rough, metal))
            else:
                raise SystemExit(f"unsupported arena material map {map_kind}")
    return pixels


def write_png_rgb(path: Path, width: int, height: int, pixels):
    raw = bytearray()
    for y in range(height):
        raw.append(0)
        row = pixels[y * width : (y + 1) * width]
        for r, g, b in row:
            raw.extend([r, g, b])
    def chunk(kind, payload):
        return (
            struct.pack(">I", len(payload))
            + kind
            + payload
            + struct.pack(">I", binascii.crc32(kind + payload) & 0xFFFFFFFF)
        )
    payload = b"\x89PNG\r\n\x1a\n"
    payload += chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
    payload += chunk(b"IDAT", zlib.compress(bytes(raw), 9))
    payload += chunk(b"IEND", b"")
    path.write_bytes(payload)


def validate_arena_runtime_asset(path: Path, asset_id: str):
    failures = []
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        return [f"{asset_id} runtime mesh JSON parse failed: {exc}"]
    for key in [
        "environment_profile",
        "material_zones",
        "material_maps",
        "material_map_hashes",
        "lighting_anchors",
        "duel_readable_landmarks",
        "floor_contact_readability",
        "composition_profile",
        "scale_reference",
        "silhouette_context",
        "playable_space",
        "atmosphere_hooks",
        "capture_ids",
        "originality_notes",
    ]:
        if not data.get(key):
            failures.append(f"{asset_id} runtime arena metadata missing {key}")
    if len(data.get("material_zones", [])) < 6:
        failures.append(f"{asset_id} runtime arena metadata has fewer than six material zones")
    if len(data.get("lighting_anchors", [])) < 3:
        failures.append(f"{asset_id} runtime arena metadata has fewer than three lighting anchors")
    if len(data.get("duel_readable_landmarks", [])) < 4:
        failures.append(f"{asset_id} runtime arena metadata has fewer than four landmarks")
    if len(data.get("floor_contact_readability", [])) < 3:
        failures.append(f"{asset_id} runtime arena metadata has fewer than three floor/contact cues")
    if len(data.get("playable_space", [])) < 4:
        failures.append(f"{asset_id} runtime arena metadata has fewer than four playable-space cues")
    if len(data.get("atmosphere_hooks", [])) < 4:
        failures.append(f"{asset_id} runtime arena metadata has fewer than four atmosphere hooks")
    if set(data.get("capture_ids", [])) != {"establishing", "gameplay", "contact"}:
        failures.append(f"{asset_id} runtime arena capture ids must be establishing, gameplay, contact")
    if asset_id == "oathyard_verdict_ring":
        if "judgment_axis" not in data.get("composition_profile", ""):
            failures.append(f"{asset_id} runtime arena must preserve judgment-axis composition")
        if "entry_break" not in data.get("silhouette_context", ""):
            failures.append(f"{asset_id} runtime arena must preserve visible entry-break silhouette")
    if asset_id == "training_yard":
        if "rectangular_drill" not in data.get("composition_profile", ""):
            failures.append(f"{asset_id} runtime arena must preserve rectangular-drill composition")
        if "no_ring_backdrop" not in data.get("silhouette_context", ""):
            failures.append(f"{asset_id} runtime arena must quarantine verdict-ring-like backdrop language")
    maps = data.get("material_maps", {})
    hashes = data.get("material_map_hashes", {})
    if set(maps) != {"base", "normal", "orm"}:
        failures.append(f"{asset_id} runtime arena material maps must include base, normal, orm")
    for name, rel_path in maps.items():
        texture = ROOT / rel_path
        if not texture.is_file():
            failures.append(f"{asset_id} missing arena material map {name}: {rel_path}")
        elif hashes.get(name) != sha256_file(texture):
            failures.append(f"{asset_id} arena material map hash mismatch for {name}")
    for key in [
        "truth_authoritative",
        "truth_mutation",
        "owner_visual_acceptance_claimed",
        "public_demo_ready",
        "release_candidate_ready",
    ]:
        if data.get(key) is not False:
            failures.append(f"{asset_id} runtime arena metadata must keep {key}=false")
    if data.get("production_target_layer") != "runtime_presentation":
        failures.append(f"{asset_id} runtime arena production_target_layer must be runtime_presentation")
    return failures


def validate_gltf(path, asset_id, kind, source):
    failures = []
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        return [f"{asset_id} glTF JSON parse failed: {exc}"]
    if data.get("asset", {}).get("version") != "2.0":
        failures.append(f"{asset_id} glTF asset.version is not 2.0")
    if data.get("scene") != 0:
        failures.append(f"{asset_id} glTF scene must be 0")
    if not data.get("scenes") or data["scenes"][0].get("nodes") != [0]:
        failures.append(f"{asset_id} glTF scene does not reference node 0")
    nodes = data.get("nodes", [])
    meshes = data.get("meshes", [])
    buffers = data.get("buffers", [])
    buffer_views = data.get("bufferViews", [])
    accessors = data.get("accessors", [])
    materials = data.get("materials", [])
    images = data.get("images", [])
    textures = data.get("textures", [])
    if len(nodes) != 1 or nodes[0].get("mesh") != 0:
        failures.append(f"{asset_id} glTF must contain one node referencing mesh 0")
    else:
        extras = nodes[0].get("extras", {})
        if extras.get("product") != "OATHYARD":
            failures.append(f"{asset_id} glTF extras missing OATHYARD product")
        if extras.get("asset_id") != asset_id or extras.get("kind") != kind:
            failures.append(f"{asset_id} glTF extras id/kind mismatch")
        if extras.get("source") != source:
            failures.append(f"{asset_id} glTF extras source mismatch")
        if extras.get("provenance") != "repo_owned_original_text_asset":
            failures.append(f"{asset_id} glTF extras provenance mismatch")
        if extras.get("truth_authoritative") is not False:
            failures.append(f"{asset_id} glTF must not be truth-authoritative")
        if extras.get("pbr_material_schema") != "oathyard.pbr_surface.v1":
            failures.append(f"{asset_id} glTF extras missing PBR material schema")
        if not extras.get("pbr_material_profile"):
            failures.append(f"{asset_id} glTF extras missing PBR material profile")
        if kind == "fighters" and extras.get("canonical_truth_joints") != CANONICAL_TRUTH_JOINTS:
            failures.append(f"{asset_id} glTF fighter extras lack canonical truth joints")
        if kind == "arenas":
            for key in [
                "environment_profile",
                "material_zones",
                "material_maps",
                "material_map_hashes",
                "lighting_anchors",
                "duel_readable_landmarks",
                "floor_contact_readability",
                "composition_profile",
                "scale_reference",
                "silhouette_context",
                "playable_space",
                "atmosphere_hooks",
                "capture_ids",
                "originality_notes",
            ]:
                if not extras.get(key):
                    failures.append(f"{asset_id} glTF arena extras missing {key}")
            if extras.get("renderer_consumption_boundary") != "presentation_after_truth_hash":
                failures.append(f"{asset_id} glTF arena extras missing presentation-after-hash boundary")
    if not meshes or not meshes[0].get("primitives"):
        failures.append(f"{asset_id} glTF missing mesh primitive")
    else:
        primitive = meshes[0]["primitives"][0]
        if primitive.get("attributes", {}).get("POSITION") != 0:
            failures.append(f"{asset_id} glTF primitive missing POSITION accessor")
        if primitive.get("indices") != 1 or primitive.get("mode", 4) != 4:
            failures.append(f"{asset_id} glTF primitive indices/mode mismatch")
    if len(accessors) != 2:
        failures.append(f"{asset_id} glTF must contain position and index accessors")
    else:
        if accessors[0].get("componentType") != 5126 or accessors[0].get("type") != "VEC3":
            failures.append(f"{asset_id} glTF position accessor mismatch")
        if accessors[1].get("componentType") != 5123 or accessors[1].get("type") != "SCALAR":
            failures.append(f"{asset_id} glTF index accessor mismatch")
        if accessors[0].get("count", 0) < 3 or accessors[1].get("count", 0) < 3:
            failures.append(f"{asset_id} glTF accessor counts are too small")
        mins = accessors[0].get("min", [])
        maxs = accessors[0].get("max", [])
        if len(mins) != 3 or len(maxs) != 3:
            failures.append(f"{asset_id} glTF position accessor lacks 3D bounds")
        elif maxs[2] <= mins[2]:
            failures.append(f"{asset_id} glTF geometry has no Z depth")
    if len(buffer_views) != 2:
        failures.append(f"{asset_id} glTF must contain two bufferViews")
    else:
        if buffer_views[0].get("target") != 34962 or buffer_views[1].get("target") != 34963:
            failures.append(f"{asset_id} glTF bufferView targets mismatch")
    if len(buffers) != 1:
        failures.append(f"{asset_id} glTF must contain one embedded buffer")
    else:
        uri = buffers[0].get("uri", "")
        prefix = "data:application/octet-stream;base64,"
        if not uri.startswith(prefix):
            failures.append(f"{asset_id} glTF buffer is not embedded base64")
        else:
            try:
                decoded = base64.b64decode(uri[len(prefix) :], validate=True)
            except ValueError as exc:
                failures.append(f"{asset_id} glTF buffer base64 decode failed: {exc}")
            else:
                if len(decoded) != buffers[0].get("byteLength"):
                    failures.append(f"{asset_id} glTF buffer byteLength mismatch")
                view_total = sum(view.get("byteLength", 0) for view in buffer_views)
                if len(decoded) != view_total:
                    failures.append(f"{asset_id} glTF bufferView byteLength sum mismatch")
    if kind == "arenas":
        if len(materials) < 6:
            failures.append(f"{asset_id} arena glTF must contain at least six material zones")
        if len(images) != 3 or len(textures) != 3:
            failures.append(f"{asset_id} arena glTF must contain base, normal, and ORM texture maps")
        for material in materials[:6]:
            extras = material.get("extras", {})
            if extras.get("truth_authoritative") is not False:
                failures.append(f"{asset_id} arena material {material.get('name', '<unnamed>')} must be presentation-only")
            if extras.get("pbr_material_schema") != "oathyard.pbr_surface.v1":
                failures.append(f"{asset_id} arena material {material.get('name', '<unnamed>')} lacks PBR schema")
        for image in images:
            uri = image.get("uri", "")
            if uri.startswith("data:"):
                continue
            if not (path.parent / uri).resolve().is_file():
                failures.append(f"{asset_id} arena texture image missing: {uri}")
    else:
        if not materials:
            failures.append(f"{asset_id} glTF missing material")
        elif materials[0].get("extras", {}).get("pbr_material_schema") != "oathyard.pbr_surface.v1":
            failures.append(f"{asset_id} glTF material lacks PBR schema")
    return failures


def render_provenance(entries, asset_hash):
    lines = [
        "# OATHYARD Asset Provenance Report",
        "",
        f"- Asset hash: `{asset_hash}`",
        "- Source policy: repo-owned original text assets",
        "- Public demo ready: `false`",
        "- Release candidate ready: `false`",
        "",
    ]
    for entry in entries:
        lines.append(
            f"- `{entry['id']}` ({entry['kind']}): source `{entry['source']}`, runtime `{entry['runtime_mesh']}`, glTF `{entry['runtime_gltf']}`, preview `{entry['preview']}`, hash `{entry['hash']}`"
        )
        if entry.get("pbr_material_profile"):
            lines.append(
                f"  - pbr material profile: {','.join(entry['pbr_material_profile']['base_surface_ids'])}"
            )
        if entry.get("material_maps"):
            maps = ", ".join(f"{name} `{path}`" for name, path in entry["material_maps"].items())
            lines.append(f"  - arena material maps: {maps}")
    return "\n".join(lines) + "\n"


def render_validation(failures):
    lines = ["# OATHYARD Asset Validation Report", ""]
    if failures:
        lines.append("Status: FAILED")
        lines.append("")
        for failure in failures:
            lines.append(f"- {failure}")
    else:
        lines.append("Status: PASSED")
        lines.append("")
        lines.append("- Provenance present")
        lines.append("- Source assets exist")
        lines.append("- Runtime assets exist")
        lines.append("- glTF runtime assets exist and pass local structural validation")
        lines.append("- PBR material source/manifest cover weapons, armor, arenas, fighters, and required channels")
        lines.append("- Runtime glTF geometry has nonzero Z depth for native 3D presentation")
        lines.append("- Arena material zones, maps, lighting, landmarks, floor/contact, composition, scale, silhouette, playable-space, atmosphere, capture-id, and originality metadata are present")
        lines.append("- Previews exist")
        lines.append("- Production placeholder markers absent")
        lines.append("- Fighter rig mapping includes canonical truth joints")
    return "\n".join(lines) + "\n"


def render_gltf_validation(failures):
    lines = ["# OATHYARD glTF Validation Report", ""]
    if failures:
        lines.append("Status: FAILED")
        lines.append("")
        for failure in failures:
            lines.append(f"- {failure}")
    else:
        lines.append("Status: PASSED")
        lines.append("")
        lines.append("- Local structural glTF 2.0 validation passed")
        lines.append("- Runtime glTF meshes include nonzero Z depth for 3D presentation")
        lines.append("- Embedded buffers decode and match declared byte lengths")
        lines.append("- Asset extras preserve OATHYARD source, provenance, and truth-authority metadata")
        lines.append("- glTF material extras reference source-backed PBR material profiles")
        lines.append("- Arena glTFs include six material zones plus base/normal/ORM map references")
        lines.append("- Fighter glTF extras include the canonical 16 truth joints plus grip frame metadata")
        lines.append("- External Khronos validator: unavailable in this environment, not claimed")
    return "\n".join(lines) + "\n"


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def escape(text: str) -> str:
    return (
        text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
    )


if __name__ == "__main__":
    raise SystemExit(main())
