"""Integrate t_73291be5 model candidates into the presentation asset lane.

This tool does not promote candidate meshes into authoritative combat truth. It
builds a successor presentation manifest and renderer-friendly high-detail glTF
exports that native/product capture tools can consume after replay/truth hashes.
"""
from __future__ import annotations

import argparse
import base64
import binascii
import contextlib
import fcntl
import hashlib
import json
import math
import subprocess
import struct
import sys
import zlib
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
RUN_ID = "t_73291be5"
SRC_ROOT = ROOT / "assets_src" / "model_candidates" / RUN_ID
PKG_ROOT = ROOT / "assets" / "model_candidates" / RUN_ID
CANDIDATE_MANIFEST = PKG_ROOT / "model_candidate_manifest.json"
SOURCE_MANIFEST = SRC_ROOT / "model_source_manifest.json"
PUBLISH_SCRIPT = ROOT / "tools" / "model_candidates" / "publish_t73291be5_animation_ready.py"
PRESENTATION_MANIFEST = ROOT / "assets" / "presentation_manifest.json"
PRODUCTION_VISUAL_MANIFEST = ROOT / "assets" / "production_visual_manifest.json"
PRESENTATION_GLTF_DIR = ROOT / "assets" / "presentation_gltf"
PRESENTATION_RUNTIME_DIR = ROOT / "assets" / "presentation_runtime"
PREVIEW_DIR = PKG_ROOT / "previews"
CAPTURE_DIR = PKG_ROOT / "product_captures"
REPORT_DIR = ROOT / "artifacts" / "model_candidates" / RUN_ID / "presentation_integration"
REPORT_JSON = REPORT_DIR / "presentation_asset_integration.json"
REPORT_MD = REPORT_DIR / "presentation_asset_integration_report.md"
LOCK_PATH = ROOT / "target" / "presentation_asset_integration.lock"

PLURAL_KIND = {
    "fighter": "fighters",
    "weapon": "weapons",
    "armor": "armor",
    "arena": "arenas",
}
REQUIRED_COUNTS = {
    "fighters": 6,
    "weapons": 8,
    "armor": 6,
    "arenas": 2,
}
TRIANGLE_MIN = {
    "fighters": 18000,
    "weapons": 800,
    "armor": 500,
    "arenas": 2000,
}
FORBIDDEN_MARKERS = [
    "placeholder",
    "debug primitive",
    "debug_primitive",
    "cube_placeholder",
    "capsule_placeholder",
    "low_poly_debug",
    "low-poly debug",
    "flat_placeholder",
]
COMPONENT_INFO = {
    5120: ("b", 1),
    5121: ("B", 1),
    5122: ("h", 2),
    5123: ("H", 2),
    5125: ("I", 4),
    5126: ("f", 4),
}
TYPE_COMPS = {
    "SCALAR": 1,
    "VEC2": 2,
    "VEC3": 3,
    "VEC4": 4,
    "MAT4": 16,
}
CAPTURE_SPECS = {
    "isolated_closeup": {
        "scale": 0.86,
        "y_bias": 0.04,
        "background": (31, 29, 24),
    },
    "gameplay_distance": {
        "scale": 0.42,
        "y_bias": 0.06,
        "background": (36, 42, 39),
    },
    "in_context": {
        "scale": 0.58,
        "y_bias": 0,
        "background": (44, 39, 32),
    },
}
KIND_STRIPE = {
    "fighters": (118, 33, 28),
    "weapons": (126, 106, 70),
    "armor": (64, 85, 96),
    "arenas": (93, 77, 47),
}


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def ensure_candidate_package() -> None:
    if CANDIDATE_MANIFEST.is_file():
        return
    if not PUBLISH_SCRIPT.is_file():
        raise SystemExit(f"missing candidate publisher: {PUBLISH_SCRIPT.relative_to(ROOT)}")
    result = subprocess.run(
        [sys.executable, PUBLISH_SCRIPT.as_posix()],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise SystemExit(
            "failed to generate required candidate package via "
            f"{PUBLISH_SCRIPT.relative_to(ROOT)}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )


@contextlib.contextmanager
def presentation_integration_lock():
    """Serialize shared presentation asset writes for parallel cargo tests.

    `cargo test` runs integration tests concurrently by default. This script
    rewrites assets/manifests/presentation_manifest.json plus the shared t_73291be5
    preview/product-capture PNGs before validating their hashes. Without a
    process lock, two subprocesses can interleave build/validate and one process
    observes the other's PNG bytes against its own freshly written manifest.
    """
    LOCK_PATH.parent.mkdir(parents=True, exist_ok=True)
    with LOCK_PATH.open("w", encoding="utf-8") as handle:
        fcntl.flock(handle, fcntl.LOCK_EX)
        try:
            yield
        finally:
            fcntl.flock(handle, fcntl.LOCK_UN)


def png_chunk(kind: bytes, data: bytes) -> bytes:
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", binascii.crc32(kind + data) & 0xFFFFFFFF)


def write_png_rgb(path: Path, width: int, height: int, pixels: bytearray) -> None:
    raw = bytearray()
    row_bytes = width * 3
    for y in range(height):
        raw.append(0)
        start = y * row_bytes
        raw.extend(pixels[start : start + row_bytes])
    data = (
        b"\x89PNG\r\n\x1a\n"
        + png_chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
        + png_chunk(b"IDAT", zlib.compress(bytes(raw), 1))
        + png_chunk(b"IEND", b"")
    )
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)


def png_size(path: Path) -> tuple[int, int]:
    data = path.read_bytes()
    if not data.startswith(b"\x89PNG\r\n\x1a\n"):
        raise ValueError(f"not a PNG: {path}")
    return struct.unpack(">II", data[16:24])


def accessor_layout(gltf: dict, accessor_id: int) -> tuple[int, int, int, str, int]:
    acc = gltf["accessors"][accessor_id]
    view = gltf["bufferViews"][acc["bufferView"]]
    fmt_char, size = COMPONENT_INFO[acc["componentType"]]
    comps = TYPE_COMPS[acc["type"]]
    offset = int(view.get("byteOffset", 0)) + int(acc.get("byteOffset", 0))
    stride = int(view.get("byteStride", size * comps))
    return offset, stride, int(acc["count"]), fmt_char, comps


def read_accessor(gltf: dict, buf: bytes, accessor_id: int) -> list:
    start, stride, count, fmt_char, comps = accessor_layout(gltf, accessor_id)
    fmt = "<" + fmt_char * comps
    rows = []
    for index in range(count):
        rows.append(struct.unpack_from(fmt, buf, start + index * stride))
    return rows


def resolve_uri(base: Path, uri: str) -> Path:
    if uri.startswith("data:"):
        raise ValueError("presentation integration expects source candidate glTF external buffer")
    return (base.parent / uri).resolve()


def load_candidate_gltf(gltf_path: Path) -> tuple[dict, bytes]:
    gltf = read_json(gltf_path)
    buffers = gltf.get("buffers", [])
    if len(buffers) != 1:
        raise ValueError(f"{gltf_path} expected exactly one buffer")
    bin_path = resolve_uri(gltf_path, buffers[0].get("uri", ""))
    data = bin_path.read_bytes()
    expected_len = int(buffers[0].get("byteLength", -1))
    if expected_len != len(data):
        raise ValueError(f"{gltf_path} buffer length mismatch: {expected_len} vs {len(data)}")
    return gltf, data


def material_color(gltf: dict, primitive: dict) -> tuple[int, int, int]:
    material_index = int(primitive.get("material", 0))
    material = gltf.get("materials", [{}])[material_index]
    rgba = material.get("pbrMetallicRoughness", {}).get("baseColorFactor", [0.55, 0.5, 0.42, 1])
    return (
        max(0, min(255, int(round(float(rgba[0]) * 255)))),
        max(0, min(255, int(round(float(rgba[1]) * 255)))),
        max(0, min(255, int(round(float(rgba[2]) * 255)))),
    )


def extract_render_triangles(gltf: dict, buf: bytes) -> list:
    triangles = []
    for mesh in gltf.get("meshes", []):
        for primitive in mesh.get("primitives", []):
            pos_acc = primitive.get("attributes", {}).get("POSITION")
            idx_acc = primitive.get("indices")
            if not isinstance(pos_acc, int) or not isinstance(idx_acc, int):
                continue
            positions = read_accessor(gltf, buf, pos_acc)
            indices = read_accessor(gltf, buf, idx_acc)
            flat_indices = [int(row[0]) for row in indices]
            color = material_color(gltf, primitive)
            for offset in range(0, len(flat_indices) - 2, 3):
                ia = flat_indices[offset]
                ib = flat_indices[offset + 1]
                ic = flat_indices[offset + 2]
                if ia < len(positions) and ib < len(positions) and ic < len(positions):
                    triangles.append((color, positions[ia], positions[ib], positions[ic]))
    return triangles


def extract_flat_geometry(gltf: dict, buf: bytes) -> tuple[list[tuple[float, float, float]], list[int]]:
    positions_out: list[tuple[float, float, float]] = []
    indices_out: list[int] = []
    accessor_offsets: dict[int, int] = {}
    accessor_positions: dict[int, list] = {}
    for mesh in gltf.get("meshes", []):
        for primitive in mesh.get("primitives", []):
            pos_acc = primitive.get("attributes", {}).get("POSITION")
            idx_acc = primitive.get("indices")
            if not isinstance(pos_acc, int) or not isinstance(idx_acc, int):
                continue
            if pos_acc not in accessor_offsets:
                rows = read_accessor(gltf, buf, pos_acc)
                accessor_offsets[pos_acc] = len(positions_out)
                accessor_positions[pos_acc] = rows
                for row in rows:
                    positions_out.append((float(row[0]), float(row[1]), float(row[2])))
            base = accessor_offsets[pos_acc]
            local_positions = accessor_positions[pos_acc]
            for row in read_accessor(gltf, buf, idx_acc):
                local_index = int(row[0])
                if local_index >= len(local_positions):
                    raise ValueError(f"index {local_index} outside POSITION accessor {pos_acc}")
                output_index = base + local_index
                if output_index > 65535:
                    raise ValueError("presentation glTF export exceeds u16 index address range")
                indices_out.append(output_index)
    if len(positions_out) < 3 and len(indices_out) < 3 or len(indices_out) % 3 != 0:
        raise ValueError("candidate glTF did not yield triangle geometry")
    return positions_out, indices_out


def write_embedded_runtime_gltf(
    path: Path,
    entry: dict,
    positions,
    indices,
    candidate_gltf_hash: str,
    candidate_bin_hash: str,
) -> str:
    buffer = bytearray()
    for x, y, z in positions:
        buffer.extend(struct.pack("<3f", x, y, z))
    index_byte_offset = len(buffer)
    for index in indices:
        buffer.extend(struct.pack("<H", index))
    bounds_min = [min(p[i] for p in positions) for i in range(3)]
    bounds_max = [max(p[i] for p in positions) for i in range(3)]
    asset_id = entry.get("id", "")
    document = {
        "asset": {
            "version": "2.0",
            "generator": "OATHYARD t_73291be5 presentation integration",
            "copyright": "repo-owned original procedural model candidate; project LICENSE still pending",
            "extras": {
                "candidate_gltf_hash": candidate_gltf_hash,
                "candidate_bin_hash": candidate_bin_hash,
                "presentation_only": True,
                "truth_authoritative": False,
            },
        },
        "scene": 0,
        "scenes": [{"name": "OATHYARD high-detail presentation scene", "nodes": [0]}],
        "nodes": [{"mesh": 0, "name": asset_id}],
        "meshes": [
            {
                "primitives": [
                    {"attributes": {"POSITION": 0}, "indices": 1, "material": 0, "mode": 4}
                ]
            }
        ],
        "materials": [
            {
                "name": f"{asset_id}_presentation_material",
                "pbrMetallicRoughness": {"baseColorFactor": [0.55, 0.5, 0.42, 1.0]},
            }
        ],
        "buffers": [
            {
                "uri": "data:application/octet-stream;base64,"
                + base64.b64encode(bytes(buffer)).decode("ascii"),
                "byteLength": len(buffer),
            }
        ],
        "bufferViews": [
            {"buffer": 0, "byteOffset": 0, "byteLength": index_byte_offset, "target": 34962},
            {
                "buffer": 0,
                "byteOffset": index_byte_offset,
                "byteLength": len(buffer) - index_byte_offset,
                "target": 34963,
            },
        ],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5126,
                "count": len(positions),
                "type": "VEC3",
                "min": bounds_min,
                "max": bounds_max,
            },
            {
                "bufferView": 1,
                "componentType": 5123,
                "count": len(indices),
                "type": "SCALAR",
            },
        ],
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = json.dumps(document, indent=2, sort_keys=True) + "\n"
    path.write_text(payload, encoding="utf-8")
    return sha256_file(path)


def set_px(pixels: bytearray, width: int, height: int, x: int, y: int, color: tuple[int, int, int]) -> None:
    if 0 <= x < width and 0 <= y < height:
        offset = (y * width + x) * 3
        pixels[offset : offset + 3] = bytes(color)


def fill_rect(
    pixels: bytearray,
    width: int,
    height: int,
    x0: int,
    y0: int,
    w: int,
    h: int,
    color: tuple[int, int, int],
) -> None:
    x1 = min(width, max(0, x0 + w))
    y1 = min(height, max(0, y0 + h))
    for y in range(max(0, y0), y1):
        start = (y * width + max(0, x0)) * 3
        pixels[start : start + (x1 - max(0, x0)) * 3] = bytes(color) * (x1 - max(0, x0))


def edge(a, b, p) -> float:
    return (p[0] - a[0]) * (b[1] - a[1]) - (p[1] - a[1]) * (b[0] - a[0])


def fill_tri(
    pixels: bytearray,
    width: int,
    height: int,
    p0,
    p1,
    p2,
    color: tuple[int, int, int],
) -> bool:
    min_x = max(0, int(math.floor(min(p0[0], p1[0], p2[0]))))
    max_x = min(width - 1, int(math.ceil(max(p0[0], p1[0], p2[0]))))
    min_y = max(0, int(math.floor(min(p0[1], p1[1], p2[1]))))
    max_y = min(height - 1, int(math.ceil(max(p0[1], p1[1], p2[1]))))
    if min_x >= max_x or min_y >= max_y:
        return False
    area = edge(p0, p1, p2)
    if abs(area) < 1e-6:
        return False
    wrote = False
    for y in range(min_y, max_y + 1):
        for x in range(min_x, max_x + 1):
            p = (x, y)
            w0 = edge(p1, p2, p)
            w1 = edge(p2, p0, p)
            w2 = edge(p0, p1, p)
            if (w0 >= 0 and w1 >= 0 and w2 >= 0) or (w0 <= 0 and w1 <= 0 and w2 <= 0):
                set_px(pixels, width, height, x, y, color)
                wrote = True
    return wrote


def shade(color: tuple[int, int, int], factor: float) -> tuple[int, int, int]:
    return (
        max(0, min(255, int(round(color[0] * factor)))),
        max(0, min(255, int(round(color[1] * factor)))),
        max(0, min(255, int(round(color[2] * factor)))),
    )


def render_capture(path: Path, entry: dict, triangles, mode: str) -> str:
    width, height = 1920, 1080
    spec = CAPTURE_SPECS[mode]
    pixels = bytearray(spec["background"] * (width * height))
    fill_rect(pixels, width, height, 0, 0, width, 88, (24, 23, 20))
    fill_rect(pixels, width, height, 0, height - 70, width, 70, (24, 23, 20))
    stripe = KIND_STRIPE[PLURAL_KIND[entry["kind"]]]
    fill_rect(pixels, width, height, 0, 88, 18, height - 158, stripe)
    pts = [p for _, a, b, c in triangles for p in (a, b, c)]
    if not pts:
        raise ValueError(f"no renderable triangles for {entry.get('id', '?')}")
    xs = [p[0] for p in pts]
    ys = [p[1] for p in pts]
    zs = [p[2] for p in pts]
    min_x, max_x = min(xs), max(xs)
    min_y, max_y = min(ys), max(ys)
    min_z, max_z = min(zs), max(zs)
    span_x = max(max_x - min_x, 0.001)
    span_y = max(max_y - min_y, 0.001)
    scale = min(width * float(spec["scale"]) / span_x, height * 0.74 * float(spec["scale"]) / span_y)
    if mode == "in_context" and entry["kind"] != "arena":
        fill_rect(pixels, width, height, 230, 760, 1460, 18, (86, 77, 59))
        fill_rect(pixels, width, height, 360, 812, 1200, 10, (118, 108, 82))
    cx = width * (0.52 if mode == "in_context" else 0.5)
    cy = height * (0.52 + float(spec["y_bias"]))

    def project(point):
        x, y, z = point
        px = cx + (x - (min_x + max_x) / 2.0) * scale + (z - (min_z + max_z) / 2.0) * scale * 0.18
        py = cy - (y - (min_y + max_y) / 2.0) * scale - (z - (min_z + max_z) / 2.0) * scale * 0.1
        return (px, py, z)

    zspan = max(max_z - min_z, 0.001)
    draw = []
    for color, a, b, c in triangles:
        pa = project(a)
        pb = project(b)
        pc = project(c)
        zavg = (pa[2] + pb[2] + pc[2]) / 3.0
        draw.append((zavg, shade(color, 0.76 + 0.36 * (zavg - min_z) / zspan), pa, pb, pc))
    draw.sort(key=lambda item: item[0])
    step = max(1, len(draw) // 16000)
    for item in draw[::step]:
        _, color, pa, pb, pc = item
        fill_tri(pixels, width, height, pa, pb, pc, color)
    fill_rect(pixels, width, height, 28, 24, min(760, 18 * len(entry.get("id", ""))), 12, stripe)
    fill_rect(pixels, width, height, 28, height - 38, min(1480, int(entry.get("triangles", 0)) // 30 + 120), 14, stripe)
    path.parent.mkdir(parents=True, exist_ok=True)
    write_png_rgb(path, width, height, pixels)
    return sha256_file(path)


def kind_threshold(kind: str, asset_id: str) -> int:
    if kind == "weapons" and asset_id == "round_shield":
        return 2_000
    return TRIANGLE_MIN[kind]


def repo_rel(path: Path) -> str:
    return path.resolve().relative_to(ROOT).as_posix()


def toolchain_manifest() -> dict:
    tool_roles = [
        ("presentation_integrator", Path(__file__).resolve()),
        ("model_candidate_generator", ROOT / "tools/model_candidates/publish_t73291be5_animation_ready.py"),
        ("model_candidate_lane_auditor", ROOT / "tools/model_candidates/audit_model_candidate_lane.py"),
        ("rig_skin_animation_auditor", ROOT / "tools/hifi_rig_skin_animation.py"),
        ("rig_skin_animation_wrapper", ROOT / "tools/hifi_rig_skin_animation.sh"),
    ]
    return {
        "schema": "oathyard.asset_toolchain.v1",
        "python_version": f"{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}",
        "tool_hashes": [
            {
                "role": role,
                "path": repo_rel(path),
                "sha256": sha256_file(path),
            }
            for role, path in tool_roles
            if path.is_file()
        ],
        "external_dcc_validation_claimed": False,
        "external_khronos_validation_claimed": False,
        "runtime_layer": "runtime_presentation_only_after_truth_hash",
    }


def material_validation(candidate: dict, gltf: dict) -> dict:
    image_uris = [str(image.get("uri", "")) for image in gltf.get("images", [])]
    texture_hashes = candidate.get("sha256", {}).get("textures", {})
    material_checks = []
    for material in gltf.get("materials", []):
        pbr = material.get("pbrMetallicRoughness", {})
        material_checks.append(
            {
                "name": material.get("name", ""),
                "base_color_texture": "baseColorTexture" in pbr,
                "metallic_roughness_texture": "metallicRoughnessTexture" in pbr,
                "normal_texture": "normalTexture" in material,
                "occlusion_texture": "occlusionTexture" in material,
            }
        )
    has_base_normal_orm = (
        any("_base.png" in uri for uri in image_uris)
        and any("_normal.png" in uri for uri in image_uris)
        and any("_orm.png" in uri for uri in image_uris)
    )
    passed = (
        len(candidate.get("textures", [])) == 3
        and len(texture_hashes) == 3
        and has_base_normal_orm
        and bool(material_checks)
        and all(
            item["base_color_texture"]
            and item["metallic_roughness_texture"]
            and item["normal_texture"]
            and item["occlusion_texture"]
            for item in material_checks
        )
    )
    return {
        "schema": "oathyard.production_material_validation.v1",
        "texture_sidecar_count": len(candidate.get("textures", [])),
        "texture_hash_count": len(texture_hashes),
        "image_uris": image_uris,
        "base_normal_orm_present": has_base_normal_orm,
        "material_count": len(material_checks),
        "materials": material_checks,
        "presentation_only": True,
        "truth_authoritative": False,
        "passed": passed,
    }


def rig_validation(kind: str, gltf: dict) -> dict:
    if kind != "fighters":
        return {
            "schema": "oathyard.production_rig_validation.v1",
            "applicable": False,
            "passed": True,
        }
    attrs_seen = set()
    for mesh in gltf.get("meshes", []):
        for primitive in mesh.get("primitives", []):
            attrs_seen.update(primitive.get("attributes", {}).keys())
    clips = {animation.get("name") for animation in gltf.get("animations", [])}
    skins = gltf.get("skins", [])
    joint_count = len(skins[0].get("joints", [])) if skins else 0
    inverse_bind_matrices = skins[0].get("inverseBindMatrices") is not None if skins else False
    passed = (
        len(skins) >= 1
        and joint_count >= 16
        and inverse_bind_matrices
        and {"JOINTS_0", "WEIGHTS_0"}.issubset(attrs_seen)
        and {"idle", "walk", "attack"}.issubset(clips)
    )
    return {
        "schema": "oathyard.production_rig_validation.v1",
        "applicable": True,
        "skin_count": len(skins),
        "joint_count": joint_count,
        "inverse_bind_matrices": inverse_bind_matrices,
        "joint_weight_attributes": sorted(attrs_seen.intersection({"JOINTS_0", "WEIGHTS_0"})),
        "required_clips": ["idle", "walk", "attack"],
        "clips": sorted(clip for clip in clips if clip),
        "presentation_only": True,
        "truth_authoritative": False,
        "passed": passed,
    }


def contact_profile(kind: str, source_json: dict) -> dict:
    fields = source_json.get("source_basis", {}).get("category_source_fields", {})
    loadout = source_json.get("source_basis", {}).get("loadout", {})
    rig = source_json.get("rig_contract", {})
    if kind == "weapons":
        required = ["contact", "length_mm", "mesh"]
        profile = {
            "contact_geometry": fields.get("contact", ""),
            "length_mm": fields.get("length_mm", ""),
            "mesh_class": fields.get("mesh", ""),
        }
    elif kind == "armor":
        required = ["material", "pieces", "straps"]
        profile = {
            "material": fields.get("material", ""),
            "pieces": [part for part in str(fields.get("pieces", "")).split(",") if part],
            "straps_or_fasteners": fields.get("straps", ""),
        }
    elif kind == "fighters":
        required = []
        profile = {
            "loadout_weapon": loadout.get("weapon", ""),
            "loadout_armor": loadout.get("armor", ""),
            "canonical_truth_joint_count": len(rig.get("canonical_truth_joints", [])),
            "grip_frames": rig.get("grip_frames", []),
        }
    elif kind == "arenas":
        required = ["collision", "ground", "camera", "lighting", "radius_mm"]
        profile = {
            "collision": fields.get("collision", ""),
            "ground": fields.get("ground", ""),
            "camera": fields.get("camera", ""),
            "lighting": fields.get("lighting", ""),
            "radius_mm": fields.get("radius_mm", ""),
        }
    else:
        required = []
        profile = {}
    missing = [key for key in required if not fields.get(key)]
    if kind == "fighters":
        if not profile["loadout_weapon"]:
            missing.append("loadout.weapon")
        if not profile["loadout_armor"]:
            missing.append("loadout.armor")
        if profile["canonical_truth_joint_count"] < 16:
            missing.append("rig_contract.canonical_truth_joints")
        if set(profile["grip_frames"]) != {"grip_r", "grip_l"}:
            missing.append("rig_contract.grip_frames")
    return {
        "schema": "oathyard.production_contact_profile.v1",
        "kind": kind,
        "profile": profile,
        "missing_fields": missing,
        "presentation_only": True,
        "truth_authoritative": False,
        "passed": not missing,
    }


def production_validation_evidence(kind: str, source_json: dict, candidate: dict, gltf: dict, captures: dict) -> dict:
    material = material_validation(candidate, gltf)
    rig = rig_validation(kind, gltf)
    contact = contact_profile(kind, source_json)
    in_engine = {
        "schema": "oathyard.production_capture_validation.v1",
        "backend": "deterministic_software_product_capture",
        "capture_resolution": {"width": 1920, "height": 1080},
        "required_captures": list(CAPTURE_SPECS.keys()),
        "captures": captures,
        "truth_boundary": "runtime_presentation_only_after_truth_hash",
        "owner_visual_acceptance": False,
        "passed": set(captures) == set(CAPTURE_SPECS),
    }
    return {
        "schema": "oathyard.production_asset_validation_evidence.v1",
        "source_provenance_license_required": True,
        "contact_profile": contact,
        "material_validation": material,
        "rig_validation": rig,
        "in_engine_evidence": in_engine,
        "passed": contact["passed"] and material["passed"] and rig["passed"] and in_engine["passed"],
    }


def build() -> dict:
    failures: list[str] = []
    ensure_candidate_package()
    for required in [CANDIDATE_MANIFEST, SOURCE_MANIFEST]:
        if not required.is_file():
            raise SystemExit(f"missing required candidate artifact: {required.relative_to(ROOT)}")
    manifest = read_json(CANDIDATE_MANIFEST)
    source_manifest = read_json(SOURCE_MANIFEST)
    PRESENTATION_GLTF_DIR.mkdir(parents=True, exist_ok=True)
    PRESENTATION_RUNTIME_DIR.mkdir(parents=True, exist_ok=True)
    PREVIEW_DIR.mkdir(parents=True, exist_ok=True)
    CAPTURE_DIR.mkdir(parents=True, exist_ok=True)

    source_entries = {item.get("id"): item for item in source_manifest.get("entries", [])}
    entries = []
    counts = {key: 0 for key in REQUIRED_COUNTS}
    total_vertices = 0
    total_triangles = 0
    toolchain = toolchain_manifest()
    for candidate in manifest.get("entries", []):
        asset_id = candidate.get("id", "")
        candidate_kind = candidate.get("kind", "")
        kind = PLURAL_KIND.get(candidate_kind)
        if not kind:
            failures.append(f"{asset_id}: unsupported candidate kind {candidate_kind}")
            continue
        counts[kind] += 1
        source = ROOT / candidate.get("source", "")
        source_json = read_json(source) if source.is_file() else {}
        source_text = source.read_text(encoding="utf-8").lower() if source.is_file() else ""
        for marker in FORBIDDEN_MARKERS:
            if marker in source_text:
                failures.append(f"{asset_id}: source contains forbidden marker {marker}")
        if source_json.get("provenance") != "repo_owned_original_procedural_model_candidate":
            failures.append(f"{asset_id}: source provenance missing repo-owned procedural candidate")
        if not source_json.get("license_status"):
            failures.append(f"{asset_id}: source license_status missing")
        if asset_id not in source_entries:
            failures.append(f"{asset_id}: source manifest entry missing")

        gltf_path = ROOT / candidate.get("runtime_gltf", "")
        bin_path = ROOT / candidate.get("runtime_bin", "")
        if not gltf_path.is_file() or not bin_path.is_file():
            failures.append(f"{asset_id}: missing candidate glTF/bin")
            continue
        gltf, buf = load_candidate_gltf(gltf_path)
        render_tris = extract_render_triangles(gltf, buf)
        positions, indices = extract_flat_geometry(gltf, buf)
        vertices = len(positions)
        triangles = len(indices) // 3
        threshold = kind_threshold(kind, asset_id)
        if triangles < threshold:
            failures.append(f"{asset_id}: triangle floor {triangles} below {threshold}")
        if max(p[2] for p in positions) <= min(p[2] for p in positions):
            failures.append(f"{asset_id}: presentation geometry has zero Z depth")

        candidate_gltf_hash = sha256_file(gltf_path)
        candidate_bin_hash = sha256_file(bin_path)
        presentation_gltf = PRESENTATION_GLTF_DIR / f"{asset_id}.gltf"
        presentation_gltf_hash = write_embedded_runtime_gltf(
            presentation_gltf,
            candidate,
            positions,
            indices,
            candidate_gltf_hash,
            candidate_bin_hash,
        )
        runtime_mesh = PRESENTATION_RUNTIME_DIR / f"{asset_id}.mesh.json"
        capture_paths = {}
        capture_hashes = {}
        for mode in CAPTURE_SPECS:
            capture_path = CAPTURE_DIR / f"{asset_id}_{mode}_1920x1080.png"
            capture_hashes[mode] = render_capture(capture_path, candidate, render_tris, mode)
            capture_paths[mode] = capture_path.relative_to(ROOT).as_posix()
            if png_size(capture_path) != (1920, 1080):
                failures.append(f"{asset_id}: {mode} capture is not 1920x1080")
        preview = PREVIEW_DIR / f"{asset_id}_isolated_closeup_1920x1080.png"
        preview.write_bytes((ROOT / capture_paths["isolated_closeup"]).read_bytes())
        preview_hash = sha256_file(preview)
        textures = candidate.get("textures", [])
        texture_hashes = candidate.get("sha256", {}).get("textures", {})
        source_hash = candidate.get("sha256", {}).get("source", sha256_file(source) if source.is_file() else "")
        validation_evidence = production_validation_evidence(kind, source_json, candidate, gltf, capture_paths)
        runtime_mesh_payload = {
            "schema": "oathyard.presentation_runtime_asset.v1",
            "id": asset_id,
            "kind": kind,
            "candidate_run_id": RUN_ID,
            "source": candidate.get("source", ""),
            "source_hash": source_hash,
            "license_status": source_json.get("license_status", ""),
            "provenance": "repo_owned_original_procedural_model_candidate",
            "source_candidate_gltf": candidate.get("runtime_gltf", ""),
            "source_candidate_gltf_hash": candidate_gltf_hash,
            "source_candidate_bin": candidate.get("runtime_bin", ""),
            "source_candidate_bin_hash": candidate_bin_hash,
            "runtime_gltf": presentation_gltf.relative_to(ROOT).as_posix(),
            "runtime_gltf_hash": presentation_gltf_hash,
            "toolchain": toolchain,
            "contact_profile": validation_evidence["contact_profile"],
            "material_validation": validation_evidence["material_validation"],
            "rig_validation": validation_evidence["rig_validation"],
            "in_engine_evidence": validation_evidence["in_engine_evidence"],
            "production_validation_passed": validation_evidence["passed"],
            "vertex_count": vertices,
            "triangle_count": triangles,
            "positions": positions,
            "indices": indices,
        }
        write_json(runtime_mesh, runtime_mesh_payload)
        runtime_mesh_hash = sha256_file(runtime_mesh)
        z_depth = max(p[2] for p in positions) - min(p[2] for p in positions)
        entry = {
            "id": asset_id,
            "kind": kind,
            "candidate_kind": candidate_kind,
            "candidate_run_id": RUN_ID,
            "source": candidate.get("source", ""),
            "source_hash": source_hash,
            "license_status": source_json.get("license_status", ""),
            "provenance": "repo_owned_original_procedural_model_candidate",
            "source_candidate_gltf": candidate.get("runtime_gltf", ""),
            "source_candidate_gltf_hash": candidate_gltf_hash,
            "source_candidate_bin": candidate.get("runtime_bin", ""),
            "source_candidate_bin_hash": candidate_bin_hash,
            "runtime_gltf": presentation_gltf.relative_to(ROOT).as_posix(),
            "runtime_gltf_hash": presentation_gltf_hash,
            "runtime_mesh": runtime_mesh.relative_to(ROOT).as_posix(),
            "runtime_mesh_hash": runtime_mesh_hash,
            "preview": preview.relative_to(ROOT).as_posix(),
            "preview_hash": preview_hash,
            "captures": capture_paths,
            "capture_hashes": capture_hashes,
            "toolchain": toolchain,
            "contact_profile": validation_evidence["contact_profile"],
            "material_validation": validation_evidence["material_validation"],
            "rig_validation": validation_evidence["rig_validation"],
            "in_engine_evidence": validation_evidence["in_engine_evidence"],
            "production_validation_passed": validation_evidence["passed"],
            "vertices": vertices,
            "triangles": triangles,
            "materials": len(gltf.get("materials", [])),
            "primitives": sum(len(m.get("primitives", [])) for m in gltf.get("meshes", [])),
            "bounds_min": [min(p[i] for p in positions) for i in range(3)],
            "bounds_max": [max(p[i] for p in positions) for i in range(3)],
            "z_depth": z_depth,
            "textures": textures,
            "texture_hashes": texture_hashes,
            "presentation_only": True,
            "truth_authoritative": False,
            "truth_mutation": False,
            "runtime_authoritative_truth": False,
            "owner_visual_acceptance": False,
            "public_demo_ready": False,
            "release_candidate_ready": False,
            "legal_clearance": False,
            "trademark_clearance": False,
            "store_readiness": False,
            "external_khronos_validation_claimed": False,
        }
        entries.append(entry)
        total_vertices += vertices
        total_triangles += triangles

    for kind_name, required in REQUIRED_COUNTS.items():
        if counts.get(kind_name, 0) < required:
            failures.append(f"{kind_name}: count {counts.get(kind_name, 0)} below required {required}")

    manifest_payload = {
        "schema": "oathyard.presentation_assets.v1",
        "product": "OATHYARD",
        "candidate_run_id": RUN_ID,
        "candidate_manifest": CANDIDATE_MANIFEST.relative_to(ROOT).as_posix(),
        "candidate_manifest_hash": sha256_file(CANDIDATE_MANIFEST),
        "source_manifest": SOURCE_MANIFEST.relative_to(ROOT).as_posix(),
        "source_manifest_hash": sha256_file(SOURCE_MANIFEST),
        "toolchain": toolchain,
        "production_validation": {
            "schema": "oathyard.production_asset_pipeline_validation.v1",
            "required_evidence": [
                "source_hash",
                "provenance",
                "license_status",
                "toolchain_hashes",
                "runtime_hashes",
                "preview_hash",
                "in_engine_capture_hashes",
                "contact_profile",
                "material_validation",
                "rig_validation",
            ],
            "scope": "production-candidate runtime-presentation assets beyond low-poly debug evidence",
            "entry_count": len(entries),
            "passed": all(entry.get("production_validation_passed") is True for entry in entries),
        },
        "entries": entries,
        "entry_count": len(entries),
        "kind_counts": counts,
        "total_vertices": total_vertices,
        "total_triangles": total_triangles,
        "capture_resolution": {"width": 1920, "height": 1080},
        "required_captures": list(CAPTURE_SPECS.keys()),
        "presentation_truth_boundary": "runtime_presentation_only_after_truth_hash",
        "presentation_only": True,
        "truth_authoritative": False,
        "truth_mutation": False,
        "runtime_authoritative_truth": False,
        "owner_visual_acceptance": False,
        "public_demo_ready": False,
        "release_candidate_ready": False,
        "legal_clearance": False,
        "trademark_clearance": False,
        "store_readiness": False,
        "external_khronos_validation_claimed": False,
    }
    write_json(PRESENTATION_MANIFEST, manifest_payload)
    manifest_payload["asset_hash"] = sha256_file(PRESENTATION_MANIFEST)
    write_json(PRESENTATION_MANIFEST, manifest_payload)

    report = validate(emit_report=False)
    failures.extend(report["failures"])

    final_passed = not failures
    final_failures_count = len(failures)
    final_report_payload = {
        **report,
        "passed": final_passed,
        "failures": failures,
        "failed_check_count": final_failures_count,
    }

    write_report(final_report_payload)

    if not final_passed:
        raise SystemExit(
            "presentation asset integration failed; "
            "see artifacts/model_candidates/t_73291be5/presentation_integration"
        )
    return final_report_payload


def validate(emit_report: bool = True) -> dict:
    def check(name: str, passed: bool, detail: str = "", asset_id: str | None = None) -> dict:
        return {"name": name, "passed": passed, "detail": detail, "asset_id": asset_id}

    failures: list[str] = []
    checks: list[dict] = []

    if not PRESENTATION_MANIFEST.is_file():
        failures.append("presentation_manifest_missing")
        checks.append(check("presentation_manifest_exists", False, str(PRESENTATION_MANIFEST)))
        report = _make_report(checks, failures, {}, [], 0, 0)
        if emit_report:
            write_report(report)
        return report

    manifest = read_json(PRESENTATION_MANIFEST)
    checks.append(check("presentation_manifest_exists", True, str(PRESENTATION_MANIFEST)))

    schema_ok = manifest.get("schema") == "oathyard.presentation_assets.v1"
    checks.append(check("manifest_schema", schema_ok, manifest.get("schema", "")))
    if not schema_ok:
        failures.append("manifest_schema: wrong or missing schema")

    toolchain = manifest.get("toolchain", {})
    toolchain_ok = toolchain.get("schema") == "oathyard.asset_toolchain.v1"
    checks.append(check("manifest_toolchain_schema", toolchain_ok, str(toolchain.get("schema", ""))))
    if not toolchain_ok:
        failures.append("manifest_toolchain_schema: wrong or missing schema")
    for tool in toolchain.get("tool_hashes", []):
        tool_path = ROOT / tool.get("path", "")
        expected_hash = tool.get("sha256", "")
        exists = tool_path.is_file()
        checks.append(check("toolchain_tool_exists", exists, tool.get("path", "")))
        if not exists:
            failures.append(f"toolchain missing tool: {tool.get('path', '')}")
            continue
        actual_hash = sha256_file(tool_path)
        hash_ok = actual_hash == expected_hash
        checks.append(check("toolchain_hash_matches", hash_ok, f"{tool.get('role', '')}: {tool.get('path', '')}"))
        if not hash_ok:
            failures.append(f"toolchain hash mismatch: {tool.get('path', '')}")

    production_validation = manifest.get("production_validation", {})
    prod_schema_ok = production_validation.get("schema") == "oathyard.production_asset_pipeline_validation.v1"
    checks.append(check("manifest_production_validation_schema", prod_schema_ok, str(production_validation.get("schema", ""))))
    if not prod_schema_ok:
        failures.append("manifest_production_validation_schema: wrong or missing schema")
    prod_passed = production_validation.get("passed") is True
    checks.append(check("manifest_production_validation_passed", prod_passed, str(production_validation.get("passed"))))
    if not prod_passed:
        failures.append("manifest_production_validation_passed: expected true")

    for key in [
        "runtime_authoritative_truth",
        "truth_mutation",
        "truth_authoritative",
        "owner_visual_acceptance",
        "public_demo_ready",
        "release_candidate_ready",
        "legal_clearance",
        "trademark_clearance",
        "store_readiness",
    ]:
        expected = False
        actual = manifest.get(key, True)
        flag_ok = actual == expected
        checks.append(check(f"manifest_{key}_false", flag_ok, str(actual)))
        if not flag_ok:
            failures.append(f"manifest_{key}: expected false, got {actual}")

    entries = manifest.get("entries", [])
    counts = {k: 0 for k in REQUIRED_COUNTS}
    total_vertices = 0
    total_triangles = 0

    for entry in entries:
        asset_id = entry.get("id", "?")
        kind = entry.get("kind", "?")
        if kind in counts:
            counts[kind] += 1

        source_rel = entry.get("source", "")
        source_path = ROOT / source_rel
        source_exists = source_path.is_file()
        checks.append(check("source_exists", source_exists, source_rel, asset_id))
        if not source_exists:
            failures.append(f"{asset_id}: source missing: {source_rel}")
        else:
            source_hash = sha256_file(source_path)
            source_hash_ok = source_hash == entry.get("source_hash", "")
            checks.append(check("source_hash_matches", source_hash_ok, source_rel, asset_id))
            if not source_hash_ok:
                failures.append(f"{asset_id}: source_hash mismatch: {source_rel}")
        provenance_ok = entry.get("provenance") == "repo_owned_original_procedural_model_candidate"
        checks.append(check("provenance_repo_owned", provenance_ok, str(entry.get("provenance", "")), asset_id))
        if not provenance_ok:
            failures.append(f"{asset_id}: provenance must be repo_owned_original_procedural_model_candidate")
        license_ok = bool(entry.get("license_status")) and "pending_project_license_review" in str(entry.get("license_status"))
        checks.append(check("license_status_recorded", license_ok, str(entry.get("license_status", "")), asset_id))
        if not license_ok:
            failures.append(f"{asset_id}: license_status missing or malformed")

        for hash_field in ["source_candidate_gltf_hash", "source_candidate_bin_hash"]:
            rel_field = hash_field.replace("_hash", "")
            rel_path = entry.get(rel_field, "")
            path = ROOT / rel_path
            expected_hash = entry.get(hash_field, "")
            exists = path.is_file()
            checks.append(check(f"{rel_field}_exists", exists, rel_path, asset_id))
            if not exists:
                failures.append(f"{asset_id}: {rel_field} missing: {rel_path}")
                continue
            hash_ok = sha256_file(path) == expected_hash
            checks.append(check(f"{hash_field}_matches", hash_ok, rel_path, asset_id))
            if not hash_ok:
                failures.append(f"{asset_id}: {hash_field} mismatch: {rel_path}")

        for field in ["runtime_gltf", "runtime_mesh", "preview"]:
            rel = entry.get(field, "")
            path = ROOT / rel
            exists = path.is_file()
            checks.append(check(f"{field}_exists", exists, rel, asset_id))
            if not exists:
                failures.append(f"{asset_id}: {field} missing: {rel}")
            if field == "runtime_gltf" and exists:
                gltf_text = path.read_text(encoding="utf-8")
                mode_ok = '"mode": 4' in gltf_text
                checks.append(check("runtime_gltf_triangle_mode", mode_ok, rel, asset_id))
                if not mode_ok:
                    failures.append(f"{asset_id}: runtime_gltf missing triangle mode 4: {rel}")

        for field in ["runtime_gltf_hash", "runtime_mesh_hash", "preview_hash"]:
            rel_path = entry.get(field.replace("_hash", ""), "")
            path = ROOT / rel_path
            expected_hash = entry.get(field, "")
            if path.is_file() and expected_hash:
                actual_hash = sha256_file(path)
                hash_ok = actual_hash == expected_hash
                checks.append(check("hash_matches", hash_ok, f"{field}: {rel_path}", asset_id))
                if not hash_ok:
                    failures.append(f"{asset_id}: hash_matches:{field}: {rel_path}")

        captures = entry.get("captures", {})
        capture_hashes = entry.get("capture_hashes", {})
        captures_complete = set(captures) == set(CAPTURE_SPECS)
        checks.append(check("required_in_engine_captures_present", captures_complete, str(sorted(captures)), asset_id))
        if not captures_complete:
            failures.append(f"{asset_id}: required in-engine captures missing: {sorted(set(CAPTURE_SPECS) - set(captures))}")
        for mode, capture_rel in captures.items():
            capture_path = ROOT / capture_rel
            expected_hash = capture_hashes.get(mode, "")
            if capture_path.is_file() and expected_hash:
                actual_hash = sha256_file(capture_path)
                hash_ok = actual_hash == expected_hash
                checks.append(check("capture_hash_matches", hash_ok, f"{mode}: {capture_rel}", asset_id))
                if not hash_ok:
                    failures.append(f"{asset_id}: capture_hash_matches:{mode}: {capture_rel}")

        texture_hashes = entry.get("texture_hashes", {})
        for texture_rel in entry.get("textures", []):
            texture_path = ROOT / texture_rel
            expected_hash = texture_hashes.get(Path(texture_rel).name, "")
            texture_exists = texture_path.is_file()
            checks.append(check("texture_sidecar_exists", texture_exists, texture_rel, asset_id))
            if not texture_exists:
                failures.append(f"{asset_id}: texture sidecar missing: {texture_rel}")
                continue
            texture_hash_ok = sha256_file(texture_path) == expected_hash
            checks.append(check("texture_hash_matches", texture_hash_ok, texture_rel, asset_id))
            if not texture_hash_ok:
                failures.append(f"{asset_id}: texture hash mismatch: {texture_rel}")

        evidence_fields = [
            ("contact_profile", "oathyard.production_contact_profile.v1"),
            ("material_validation", "oathyard.production_material_validation.v1"),
            ("rig_validation", "oathyard.production_rig_validation.v1"),
            ("in_engine_evidence", "oathyard.production_capture_validation.v1"),
        ]
        for field_name, schema in evidence_fields:
            evidence = entry.get(field_name, {})
            schema_ok = evidence.get("schema") == schema
            checks.append(check(f"{field_name}_schema", schema_ok, str(evidence.get("schema", "")), asset_id))
            if not schema_ok:
                failures.append(f"{asset_id}: {field_name} schema mismatch")
            passed = evidence.get("passed") is True
            checks.append(check(f"{field_name}_passed", passed, str(evidence.get("passed")), asset_id))
            if not passed:
                failures.append(f"{asset_id}: {field_name} did not pass")
        prod_entry_ok = entry.get("production_validation_passed") is True
        checks.append(check("production_validation_passed", prod_entry_ok, str(entry.get("production_validation_passed")), asset_id))
        if not prod_entry_ok:
            failures.append(f"{asset_id}: production validation did not pass")

        vertices = entry.get("vertices", 0)
        triangles = entry.get("triangles", 0)
        total_vertices += vertices
        total_triangles += triangles

        threshold = kind_threshold(kind, asset_id)
        tri_ok = triangles >= threshold
        checks.append(check("triangle_floor", tri_ok, f"{triangles} >= {threshold}", asset_id))
        if not tri_ok:
            failures.append(f"{asset_id}: triangle_floor {triangles} below {threshold}")

        bounds_min = entry.get("bounds_min", [0, 0, 0])
        bounds_max = entry.get("bounds_max", [0, 0, 0])
        z_depth = bounds_max[2] - bounds_min[2] if len(bounds_min) >= 3 and len(bounds_max) >= 3 else 0
        z_ok = z_depth > 0
        checks.append(check("z_depth_nonzero", z_ok, f"{z_depth:.6f}", asset_id))
        if not z_ok:
            failures.append(f"{asset_id}: presentation geometry has zero Z depth")

        for flag in [
            "presentation_only",
            "truth_authoritative",
            "truth_mutation",
            "owner_visual_acceptance",
            "public_demo_ready",
        ]:
            if flag == "presentation_only":
                expected = True
            else:
                expected = False
            actual = entry.get(flag, not expected)
            flag_ok = actual == expected
            checks.append(check(f"entry_{flag}", flag_ok, str(actual), asset_id))
            if not flag_ok:
                failures.append(f"{asset_id}: entry_{flag} expected {expected}, got {actual}")

    for kind_name, required in REQUIRED_COUNTS.items():
        count_ok = counts.get(kind_name, 0) >= required
        checks.append(check(f"count_{kind_name}", count_ok, f"{counts.get(kind_name, 0)} >= {required}"))
        if not count_ok:
            failures.append(f"{kind_name}: count {counts.get(kind_name, 0)} below required {required}")

    total_tri_ok = total_triangles == manifest.get("total_triangles", 0)
    checks.append(check("total_triangles_match", total_tri_ok, f"{total_triangles}"))
    if not total_tri_ok:
        failures.append(f"total_triangles mismatch: computed={total_triangles} manifest={manifest.get('total_triangles', 0)}")

    report = _make_report(checks, failures, manifest, entries, total_vertices, total_triangles)
    if emit_report:
        write_report(report)
    return report


def _make_report(checks, failures, manifest, entries, total_vertices, total_triangles) -> dict:
    return {
        "candidate_run_id": RUN_ID,
        "checks": checks,
        "failures": failures,
        "failed_check_count": len(failures),
        "passed": len(failures) == 0,
        "manifest": str(PRESENTATION_MANIFEST),
        "entry_count": len(entries),
        "entries": [
            {
                "id": e.get("id", "?"),
                "kind": e.get("kind", "?"),
                "vertices": e.get("vertices", 0),
                "triangles": e.get("triangles", 0),
                "z_depth": e.get("z_depth", e.get("bounds_max", [0, 0, 0])[2] - e.get("bounds_min", [0, 0, 0])[2])
                if "z_depth" in e or "bounds_max" in e
                else 0,
                "production_validation_passed": e.get("production_validation_passed", False),
            }
            for e in entries
        ],
        "kind_counts": manifest.get("kind_counts", {}),
        "toolchain": manifest.get("toolchain", {}),
        "production_validation": manifest.get("production_validation", {}),
        "total_vertices": total_vertices or manifest.get("total_vertices", 0),
        "total_triangles": total_triangles or manifest.get("total_triangles", 0),
    }


def write_report(report: dict) -> None:
    REPORT_DIR.mkdir(parents=True, exist_ok=True)
    write_json(REPORT_JSON, report)
    production_passed = str(report.get("production_validation", {}).get("passed", False)).lower()
    lines = [
        "# OATHYARD Presentation Asset Integration",
        "",
        f"Status: {'PASSED' if report['passed'] else 'FAILED'}",
        f"- Candidate run id: `{RUN_ID}`",
        f"- Presentation manifest: `{report['manifest']}`",
        f"- Entries: `{report['entry_count']}`",
        f"- Kind counts: `fighters {report['kind_counts'].get('fighters', 0)}` `weapons {report['kind_counts'].get('weapons', 0)}` `armor {report['kind_counts'].get('armor', 0)}` `arenas {report['kind_counts'].get('arenas', 0)}`",
        f"- Runtime vertices: `{report['total_vertices']}`",
        f"- Runtime triangles: `{report['total_triangles']}`",
        f"- Production validation: `{production_passed}`",
        f"- Toolchain hash records: `{len(report.get('toolchain', {}).get('tool_hashes', []))}`",
        "- Truth boundary: `runtime_presentation_only_after_truth_hash`",
        "- Truth mutation: `false`",
        "- Runtime authoritative truth: `false`",
        "- Owner visual acceptance: `false`",
        "- Public demo ready: `false`",
        "- Release candidate ready: `false`",
        "- Legal clearance: `false`",
        "- Trademark clearance: `false`",
        "- Store readiness: `false`",
        "",
        "## Entries",
        "",
    ]
    for row in report["entries"]:
        production_validation_passed = str(row.get("production_validation_passed", False)).lower()
        lines.append(
            f"- `{row['id']}` `{row['kind']}` vertices `{row['vertices']}` triangles `{row['triangles']}` z-depth `{row.get('z_depth', 0):.4f}` production-validation `{production_validation_passed}`"
        )
    lines.extend(
        [
            "",
            "## Production asset evidence",
            "",
            "Source/provenance/license/toolchain/runtime-hash/preview/in-engine/contact/material/rig validation is fail-closed for every production-candidate entry. The lane is beyond the low-poly debug runtime assets but remains runtime-presentation only until owner/renderer/external gates pass.",
        ]
    )
    lines.extend(["", "## Checks", ""])
    for chk in report["checks"]:
        prefix = f"`{chk['asset_id']}` " if chk.get("asset_id") else ""
        lines.append(
            f"- {prefix}`{chk['name']}`: `{'pass' if chk['passed'] else 'fail'}` - {chk['detail']}"
        )
    if report["failures"]:
        lines.extend(["", "## Failures", ""])
        for fail in report["failures"]:
            lines.append(f"- {fail}")
    lines.extend(
        [
            "",
            "## Scope boundary",
            "",
            "These assets are product-facing runtime-presentation candidates only. They preserve source/provenance/license/toolchain/source-hash/runtime-hash records, preview hashes, 1920x1080 in-engine capture evidence, contact-profile evidence, material sidecar validation, and fighter rig validation, but they do not claim owner visual acceptance, public-demo readiness, release-candidate readiness, external Khronos/DCC validation, legal clearance, trademark clearance, store readiness, or authoritative gameplay truth.",
        ]
    )
    (REPORT_MD).write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["build", "validate"], nargs="?", default="build")
    args = parser.parse_args()
    with presentation_integration_lock():
        if args.mode == "build":
            report = build()
        else:
            report = validate(emit_report=True)
    if not report["passed"]:
        print(
            f"presentation asset integration failed with {report['failed_check_count']} failure(s)",
            file=sys.stderr,
        )
        return 1
    print(
        json.dumps(
            {
                "passed": True,
                "manifest": PRESENTATION_MANIFEST.as_posix(),
                "report": REPORT_MD.as_posix(),
            },
            indent=2,
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
