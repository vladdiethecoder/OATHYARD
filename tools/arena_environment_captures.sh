#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/arena_environment/latest}"
mkdir -p "$out"

python3 - "$out" <<'PY'
import base64
import hashlib
import json
import math
import struct
import sys
import zlib
from pathlib import Path

ROOT = Path.cwd()
out = Path(sys.argv[1])
out.mkdir(parents=True, exist_ok=True)
MANIFEST = ROOT / "assets/runtime_manifest.json"
ARENAS = ["oathyard_verdict_ring", "training_yard"]
MODES = ["establishing", "gameplay", "contact"]
WIDTH = 1920
HEIGHT = 1080
SOURCE_BACKED_PATHS = [
    "assets_src/arenas/arenas.oysrc",
    "content/oathyard_content.manifest",
    "assets/runtime_manifest.json",
    "tools/asset_pipeline.py",
    "tools/arena_environment_captures.sh",
]
AUDIT_DIRECTIVE_MAPPING = {
    "oathyard_verdict_ring": ["ID-02", "COMP-01", "SCALE-02", "SC-01", "SC-02", "ORIG-01", "ORIG-02"],
    "training_yard": ["ID-01", "ID-03", "COMP-01", "SCALE-02", "SC-01", "SC-02", "ORIG-01", "ORIG-02"],
}


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def hash_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_ref(rel_path: str):
    path = ROOT / rel_path
    return {
        "path": rel_path,
        "sha256": sha256(path) if path.is_file() else "",
    }


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def color_factor_to_rgb(factor):
    return tuple(max(0, min(255, int(round(float(value) * 255)))) for value in factor[:3])


def load_arena(entry):
    mesh_path = ROOT / entry["runtime_mesh"]
    gltf_path = ROOT / entry["runtime_gltf"]
    mesh = read_json(mesh_path)
    gltf = read_json(gltf_path)
    encoded = gltf["buffers"][0]["uri"].split(",", 1)[1]
    decoded = base64.b64decode(encoded)
    vertex_count = int(gltf["accessors"][0]["count"])
    index_count = int(gltf["accessors"][1]["count"])
    positions = []
    for index in range(vertex_count):
        offset = index * 12
        positions.append(struct.unpack_from("<3f", decoded, offset))
    index_offset = vertex_count * 12
    indices = []
    for index in range(index_count):
        indices.append(struct.unpack_from("<H", decoded, index_offset + index * 2)[0])
    materials = gltf.get("materials", [])
    palette = [
        color_factor_to_rgb(material.get("pbrMetallicRoughness", {}).get("baseColorFactor", [0.45, 0.42, 0.36]))
        for material in materials
    ] or [(112, 104, 88)]
    material_hashes = {}
    for name, rel in entry.get("material_maps", {}).items():
        material_hashes[name] = sha256(ROOT / rel) if (ROOT / rel).is_file() else ""
    return {
        "entry": entry,
        "mesh": mesh,
        "gltf": gltf,
        "positions": positions,
        "indices": indices,
        "palette": palette,
        "material_hashes": material_hashes,
        "runtime_mesh_hash": sha256(mesh_path),
        "runtime_gltf_hash": sha256(gltf_path),
    }


def set_px(pixels, x, y, color):
    if 0 <= x < WIDTH and 0 <= y < HEIGHT:
        idx = (y * WIDTH + x) * 3
        pixels[idx:idx + 3] = bytes(color)


def fill_rect(pixels, x0, y0, x1, y1, color):
    x0, y0 = max(0, int(x0)), max(0, int(y0))
    x1, y1 = min(WIDTH, int(x1)), min(HEIGHT, int(y1))
    if x0 >= x1 or y0 >= y1:
        return
    row = bytes(color) * (x1 - x0)
    for y in range(y0, y1):
        start = (y * WIDTH + x0) * 3
        pixels[start:start + len(row)] = row


def draw_line(pixels, x0, y0, x1, y1, color):
    x0, y0, x1, y1 = int(x0), int(y0), int(x1), int(y1)
    dx = abs(x1 - x0)
    sx = 1 if x0 < x1 else -1
    dy = -abs(y1 - y0)
    sy = 1 if y0 < y1 else -1
    err = dx + dy
    while True:
        set_px(pixels, x0, y0, color)
        if x0 == x1 and y0 == y1:
            return
        e2 = 2 * err
        if e2 >= dy:
            err += dy
            x0 += sx
        if e2 <= dx:
            err += dx
            y0 += sy


def edge(a, b, c):
    return (c[0] - a[0]) * (b[1] - a[1]) - (c[1] - a[1]) * (b[0] - a[0])


def fill_triangle(pixels, pts, color):
    xs = [p[0] for p in pts]
    ys = [p[1] for p in pts]
    min_x, max_x = max(0, min(xs)), min(WIDTH - 1, max(xs))
    min_y, max_y = max(0, min(ys)), min(HEIGHT - 1, max(ys))
    area = edge(pts[0], pts[1], pts[2])
    if area == 0 or min_x >= max_x or min_y >= max_y:
        return False
    area_px = max(1, int(max_x - min_x) * int(max_y - min_y))
    step = 4 if area_px > 120_000 else 2 if area_px > 30_000 else 1
    wrote = False
    for y in range(int(min_y), int(max_y) + 1, step):
        for x in range(int(min_x), int(max_x) + 1, step):
            p = (x, y)
            w0 = edge(pts[1], pts[2], p)
            w1 = edge(pts[2], pts[0], p)
            w2 = edge(pts[0], pts[1], p)
            if (w0 >= 0 and w1 >= 0 and w2 >= 0) or (w0 <= 0 and w1 <= 0 and w2 <= 0):
                fill_rect(pixels, x, y, x + step, y + step, color)
                wrote = True
    if wrote:
        outline = (22, 24, 23)
        draw_line(pixels, pts[0][0], pts[0][1], pts[1][0], pts[1][1], outline)
        draw_line(pixels, pts[1][0], pts[1][1], pts[2][0], pts[2][1], outline)
        draw_line(pixels, pts[2][0], pts[2][1], pts[0][0], pts[0][1], outline)
    return wrote


def project(point, bounds, mode):
    min_x, max_x, min_y, max_y, min_z, max_z = bounds
    x, y, z = point
    span = max(max_x - min_x, max_y - min_y, 0.001)
    scale = 720.0 / span
    cx = WIDTH // 2
    cy = 592 if mode == "establishing" else 620
    depth_x = z * 210.0
    depth_y = z * 135.0
    sx = cx + x * scale + depth_x
    sy = cy - y * scale * 0.72 - depth_y
    return (int(round(sx)), int(round(sy)), int(round((y + z * 0.35) * 1000)))


def shade(color, depth):
    adjust = max(-24, min(28, depth // 700))
    return tuple(max(0, min(255, c + adjust)) for c in color)


def draw_mesh(pixels, arena, mode):
    positions = arena["positions"]
    indices = arena["indices"]
    xs, ys, zs = [p[0] for p in positions], [p[1] for p in positions], [p[2] for p in positions]
    bounds = (min(xs), max(xs), min(ys), max(ys), min(zs), max(zs))
    triangles = []
    palette = arena["palette"]
    for tri_index in range(0, len(indices), 3):
        tri = indices[tri_index:tri_index + 3]
        projected = [project(positions[i], bounds, mode) for i in tri]
        depth = sum(p[2] for p in projected) // 3
        base = palette[(tri_index // 3) % len(palette)]
        if mode == "contact" and (tri_index // 3) % 7 == 0:
            base = (170, 54, 38)
        triangles.append((depth, [(p[0], p[1]) for p in projected], shade(base, depth)))
    for _, pts, color in sorted(triangles, key=lambda item: item[0]):
        fill_triangle(pixels, pts, color)
    return bounds


def draw_ring_guides(pixels, color, radius_x, radius_y, cx=WIDTH // 2, cy=620):
    points = []
    for step in range(96):
        angle = 2.0 * math.pi * step / 96.0
        points.append((int(cx + math.cos(angle) * radius_x), int(cy + math.sin(angle) * radius_y)))
    for a, b in zip(points, points[1:] + points[:1]):
        draw_line(pixels, a[0], a[1], b[0], b[1], color)
    for step in range(0, 96, 12):
        draw_line(pixels, cx, cy, points[step][0], points[step][1], tuple(max(0, c - 36) for c in color))


def draw_training_guides(pixels, color, overlay=False):
    lane = tuple(max(0, c - 38) for c in color)
    faint = tuple(max(0, c - (72 if not overlay else 52)) for c in color)
    fill_rect(pixels, 430, 340, 1490, 350, lane)
    fill_rect(pixels, 430, 770, 1490, 780, lane)
    fill_rect(pixels, 430, 340, 440, 780, lane)
    fill_rect(pixels, 1480, 340, 1490, 780, lane)
    fill_rect(pixels, 948, 360, 972, 760, tuple(max(0, c - 18) for c in color))
    for x0, x1 in [(585, 780), (1140, 1335)]:
        fill_rect(pixels, x0, 482, x1, 520, color)
        fill_rect(pixels, x0, 608, x1, 646, color)
    for y in [420, 500, 620, 700]:
        fill_rect(pixels, 530, y - 4, 1390, y + 4, faint)
    for x in [705, 820, 1100, 1215]:
        fill_rect(pixels, x - 6, 418, x + 6, 728, tuple(max(0, c - 64) for c in color))
    for x, y in [(650, 560), (760, 560), (1160, 560), (1270, 560), (830, 458), (1090, 662)]:
        fill_rect(pixels, x - 18, y - 18, x + 18, y + 18, tuple(max(0, c - 14) for c in color))
    if overlay:
        for x in [540, 650, 760, 960, 1160, 1270, 1380]:
            draw_line(pixels, x, 352, x, 762, (90, 96, 88))
        for y in [392, 454, 560, 666, 728]:
            draw_line(pixels, 452, y, 1468, y, (90, 96, 88))


def draw_verdict_context(pixels, mode):
    fill_rect(pixels, 0, 90, WIDTH, 190, (20, 26, 30))
    fill_rect(pixels, 650, 118, 1270, 188, (70, 67, 58))
    fill_rect(pixels, 720, 72, 1200, 132, (48, 53, 58))
    fill_rect(pixels, 810, 54, 1110, 84, (150, 144, 119))
    fill_rect(pixels, 620, 190, 1300, 216, (40, 45, 47))
    for x in [700, 790, 880, 1040, 1130, 1220]:
        fill_rect(pixels, x - 8, 186, x + 8, 258, (83, 74, 60))
    fill_rect(pixels, 812, 232, 1108, 258, (96, 84, 62))
    for x in [470, 1450]:
        fill_rect(pixels, x - 22, 218, x + 22, 620, (44, 48, 48))
        fill_rect(pixels, x - 54, 196, x + 54, 228, (91, 82, 68))
        fill_rect(pixels, x - 42, 264, x + 42, 296, (33, 42, 48))
    for x in [740, 1180]:
        fill_rect(pixels, x - 44, 852, x - 18, 986, (42, 42, 36))
        fill_rect(pixels, x + 18, 852, x + 44, 986, (42, 42, 36))
        fill_rect(pixels, x - 58, 836, x + 58, 862, (93, 83, 64))
    for x in [1320, 1370, 1420]:
        fill_rect(pixels, x - 9, 610, x + 9, 782, (30, 34, 35))
        draw_line(pixels, x - 28, 650, x + 28, 650, (96, 82, 58))
        draw_line(pixels, x - 28, 714, x + 28, 714, (96, 82, 58))
    for x in [498, 534, 570]:
        draw_line(pixels, x, 666, x + 58, 784, (62, 73, 76))
        draw_line(pixels, x + 58, 784, x + 96, 666, (62, 73, 76))
    fill_rect(pixels, 870, 858, 1050, 906, (18, 22, 23))
    draw_line(pixels, 870, 858, 960, 800, (118, 105, 76))
    draw_line(pixels, 1050, 858, 960, 800, (118, 105, 76))
    for offset in [-220, 0, 220]:
        draw_line(pixels, 960 + offset, 90, 960, 585, (48, 74, 88))
    for y in [388, 462, 536]:
        draw_line(pixels, 492, y, 1428, y + 18, (62, 61, 55))
    if mode == "contact":
        fill_rect(pixels, 874, 512, 1046, 544, (170, 58, 44))


def draw_training_context(pixels, mode):
    fill_rect(pixels, 0, 90, WIDTH, 190, (64, 49, 34))
    fill_rect(pixels, 180, 196, 1740, 268, (78, 58, 36))
    fill_rect(pixels, 210, 260, 1710, 300, (48, 42, 32))
    for x in [250, 470, 1450, 1670]:
        fill_rect(pixels, x - 16, 258, x + 16, 840, (72, 54, 35))
        fill_rect(pixels, x - 30, 246, x + 30, 274, (139, 102, 52))
    for y in [320, 812]:
        draw_line(pixels, 240, y, 1680, y, (150, 112, 66))
        draw_line(pixels, 240, y + 16, 1680, y + 16, (88, 68, 44))
    for x in [360, 1560]:
        fill_rect(pixels, x - 42, 146, x + 42, 196, (216, 154, 74))
        fill_rect(pixels, x - 16, 196, x + 16, 292, (92, 66, 38))
        for dy in [0, 15, 30]:
            draw_line(pixels, x - 86, 214 + dy, x + 86, 224 + dy, (114, 80, 43))
    fill_rect(pixels, 1455, 500, 1590, 664, (82, 58, 36))
    for y in [522, 560, 598, 636]:
        draw_line(pixels, 1450, y, 1596, y - 52, (174, 151, 102))
        draw_line(pixels, 1516, y, 1578, y + 24, (74, 82, 74))
    fill_rect(pixels, 310, 612, 398, 720, (68, 77, 76))
    fill_rect(pixels, 338, 570, 372, 612, (96, 104, 98))
    fill_rect(pixels, 300, 732, 430, 760, (92, 66, 40))
    fill_rect(pixels, 1520, 360, 1620, 438, (70, 58, 42))
    for y in [382, 410]:
        draw_line(pixels, 1528, y, 1612, y, (202, 170, 104))
    if mode == "contact":
        fill_rect(pixels, 800, 510, 1120, 540, (196, 119, 50))


def draw_fighter(pixels, x, y, facing, color, contact=False):
    fill_rect(pixels, x - 28, y - 138, x + 28, y - 42, color)
    fill_rect(pixels, x - 18, y - 184, x + 18, y - 148, tuple(min(255, c + 26) for c in color))
    fill_rect(pixels, x - 44, y - 112, x + 44, y - 66, tuple(max(0, c - 32) for c in color))
    draw_line(pixels, x, y - 78, x + facing * (210 if contact else 145), y - (132 if contact else 118), (56, 62, 62))
    draw_line(pixels, x - 16, y - 42, x - 58, y + 44, (34, 42, 42))
    draw_line(pixels, x + 16, y - 42, x + 56, y + 44, (34, 42, 42))


def draw_contact_marks(pixels, arena_id):
    color = (190, 58, 38) if arena_id == "oathyard_verdict_ring" else (210, 130, 40)
    if arena_id == "oathyard_verdict_ring":
        for offset in range(-4, 5):
            draw_line(pixels, 820, 420 + offset, 1100, 484 - offset, color)
        for radius in [54, 72, 94]:
            draw_ring_guides(pixels, (132, 38, 32), radius * 2, radius, cx=958, cy=456)
        for x in [710, 1220, 940, 1010]:
            fill_rect(pixels, x - 9, 682, x + 9, 780, color)
    else:
        for offset in range(-5, 6):
            draw_line(pixels, 710, 520 + offset, 1210, 548 + offset, color)
        for y in [430, 492, 612, 736]:
            fill_rect(pixels, 640, y - 6, 1280, y + 6, tuple(max(0, c - 36) for c in color))
        for x in [710, 830, 1090, 1210]:
            fill_rect(pixels, x - 14, 660, x + 14, 752, color)


def draw_capture(arena, mode, debug_overlay=False):
    arena_id = arena["entry"]["id"]
    bg = (26, 31, 31) if arena_id == "oathyard_verdict_ring" else (40, 32, 24)
    pixels = bytearray(bg * (WIDTH * HEIGHT))
    fill_rect(pixels, 0, 0, WIDTH, 90, (18, 22, 22) if arena_id == "oathyard_verdict_ring" else (55, 43, 32))
    if arena_id == "training_yard":
        draw_training_context(pixels, mode)
        draw_training_guides(pixels, (210, 183, 122) if mode == "establishing" else (184, 148, 89), debug_overlay)
    else:
        draw_verdict_context(pixels, mode)
        if mode == "establishing":
            draw_ring_guides(pixels, (214, 204, 160), 710, 440)
        elif mode == "gameplay":
            draw_ring_guides(pixels, (170, 162, 132), 650, 385)
        else:
            draw_ring_guides(pixels, (176, 116, 88), 650, 385)
    bounds = draw_mesh(pixels, arena, mode)
    if arena_id == "training_yard":
        draw_training_guides(pixels, (226, 199, 132) if mode == "establishing" else (198, 160, 94), debug_overlay)
    if mode in {"gameplay", "contact"}:
        draw_fighter(pixels, 760, 675, 1, (43, 112, 118), mode == "contact")
        draw_fighter(pixels, 1160, 675, -1, (132, 74, 61), mode == "contact")
    if mode == "contact":
        draw_contact_marks(pixels, arena_id)
    if debug_overlay:
        # Deterministic zone swatches keyed by material map hashes; kept out of clean product-like captures.
        for idx, (name, digest) in enumerate(sorted(arena["material_hashes"].items())):
            color = tuple(int(digest[i:i + 2], 16) for i in (0, 2, 4)) if digest else (90, 90, 90)
            fill_rect(pixels, 42 + idx * 110, 934, 118 + idx * 110, 1004, color)
    png = write_png_bytes(WIDTH, HEIGHT, pixels)
    return png, bounds


def write_png_bytes(width, height, pixels):
    raw = bytearray()
    stride = width * 3
    for y in range(height):
        raw.append(0)
        raw.extend(pixels[y * stride:(y + 1) * stride])
    def chunk(kind, payload):
        return struct.pack(">I", len(payload)) + kind + payload + struct.pack(">I", zlib.crc32(kind + payload) & 0xFFFFFFFF)
    data = b"\x89PNG\r\n\x1a\n"
    data += chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
    data += chunk(b"IDAT", zlib.compress(bytes(raw), 9))
    data += chunk(b"IEND", b"")
    return data


manifest = read_json(MANIFEST)
entries = {entry["id"]: entry for entry in manifest.get("entries", []) if entry.get("kind") == "arenas"}
failures = []
captures = []
debug_overlays = []
identity_entries = []
for arena_id in ARENAS:
    entry = entries.get(arena_id)
    if not entry:
        failures.append(f"missing arena entry {arena_id}")
        continue
    arena = load_arena(entry)
    mesh = arena["mesh"]
    if len(mesh.get("material_zones", [])) < 6:
        failures.append(f"{arena_id} lacks six material zones")
    if set(entry.get("material_maps", {})) != {"base", "normal", "orm"}:
        failures.append(f"{arena_id} lacks base/normal/orm maps")
    identity = {
        "arena_id": arena_id,
        "source_backing": [file_ref(path) for path in SOURCE_BACKED_PATHS],
        "asset_references": [
            file_ref(entry.get("source", "")),
            file_ref(entry.get("preview", "")),
            file_ref(entry["runtime_mesh"]),
            file_ref(entry["runtime_gltf"]),
        ] + [file_ref(path) for path in sorted(entry.get("material_maps", {}).values())],
        "world_entity_definition": {
            "radius_mm": mesh.get("radius_mm", 0),
            "ground": mesh.get("ground", ""),
            "collision": mesh.get("collision", ""),
            "camera_anchor": mesh.get("camera_anchor", ""),
            "lighting": mesh.get("lighting", ""),
            "visual_identity": mesh.get("visual_identity", ""),
        },
        "composition_parameters": {
            "environment_profile": mesh.get("environment_profile", ""),
            "composition_profile": mesh.get("composition_profile", ""),
            "scale_reference": mesh.get("scale_reference", ""),
            "silhouette_context": mesh.get("silhouette_context", ""),
            "playable_space": mesh.get("playable_space", []),
            "duel_readable_landmarks": mesh.get("duel_readable_landmarks", []),
            "floor_contact_readability": mesh.get("floor_contact_readability", []),
            "lighting_anchors": mesh.get("lighting_anchors", []),
            "atmosphere_hooks": mesh.get("atmosphere_hooks", []),
            "capture_ids": mesh.get("capture_ids", []),
            "originality_notes": mesh.get("originality_notes", ""),
        },
        "mapped_audit_directives": AUDIT_DIRECTIVE_MAPPING.get(arena_id, []),
        "clean_capture_policy": "individual establishing/gameplay/contact PNGs omit hash swatches; *_audit_overlay_1920x1080.png carries deterministic material/hash overlay evidence",
        "environment_profile": mesh.get("environment_profile", ""),
        "composition_profile": mesh.get("composition_profile", ""),
        "scale_reference": mesh.get("scale_reference", ""),
        "silhouette_context": mesh.get("silhouette_context", ""),
        "playable_space": mesh.get("playable_space", []),
        "duel_readable_landmarks": mesh.get("duel_readable_landmarks", []),
        "floor_contact_readability": mesh.get("floor_contact_readability", []),
        "lighting_anchors": mesh.get("lighting_anchors", []),
        "atmosphere_hooks": mesh.get("atmosphere_hooks", []),
        "capture_ids": mesh.get("capture_ids", []),
        "originality_notes": mesh.get("originality_notes", ""),
        "runtime_mesh": entry["runtime_mesh"],
        "runtime_gltf": entry["runtime_gltf"],
        "runtime_mesh_sha256": arena["runtime_mesh_hash"],
        "runtime_gltf_sha256": arena["runtime_gltf_hash"],
        "material_map_hashes": arena["material_hashes"],
        "truth_mutation": False,
        "owner_visual_acceptance_claimed": False,
    }
    identity_entries.append(identity)
    for required in ["environment_profile", "composition_profile", "scale_reference", "silhouette_context", "originality_notes"]:
        if not identity[required]:
            failures.append(f"{arena_id} identity metadata missing {required}")
    for required_list, minimum in [("playable_space", 4), ("duel_readable_landmarks", 4), ("floor_contact_readability", 3), ("lighting_anchors", 3), ("atmosphere_hooks", 4)]:
        if len(identity[required_list]) < minimum:
            failures.append(f"{arena_id} identity metadata {required_list} below {minimum}")
    if set(identity["capture_ids"]) != {"establishing", "gameplay", "contact"}:
        failures.append(f"{arena_id} identity capture_ids must be establishing, gameplay, contact")
    if "repo_owned" not in identity["originality_notes"] or any(marker in identity["originality_notes"] for marker in ["copied_from", "scraped", "borrowed", "unlicensed"]):
        failures.append(f"{arena_id} originality notes must be repo-owned and free of external-source markers")
    if arena_id == "oathyard_verdict_ring" and "judgment_axis" not in identity["composition_profile"]:
        failures.append(f"{arena_id} identity must preserve judgment-axis composition")
    if arena_id == "training_yard" and "rectangular_drill" not in identity["composition_profile"]:
        failures.append(f"{arena_id} identity must preserve rectangular-drill composition")
    if arena_id == "training_yard" and "no_ring_backdrop" not in identity["silhouette_context"]:
        failures.append(f"{arena_id} identity must quarantine verdict-ring-like backdrop language")
    for mode in MODES:
        file = f"{arena_id}_{mode}_1920x1080.png"
        png, bounds = draw_capture(arena, mode)
        path = out / file
        path.write_bytes(png)
        captures.append({
            "arena_id": arena_id,
            "mode": mode,
            "file": file,
            "width": WIDTH,
            "height": HEIGHT,
            "sha256": hash_bytes(png),
            "runtime_mesh": entry["runtime_mesh"],
            "runtime_gltf": entry["runtime_gltf"],
            "runtime_mesh_sha256": arena["runtime_mesh_hash"],
            "runtime_gltf_sha256": arena["runtime_gltf_hash"],
            "debug_overlay": False,
            "clean_product_capture": True,
            "material_map_hashes": arena["material_hashes"],
            "world_identity": {
                "composition_profile": mesh.get("composition_profile", ""),
                "scale_reference": mesh.get("scale_reference", ""),
                "silhouette_context": mesh.get("silhouette_context", ""),
                "playable_space": mesh.get("playable_space", []),
                "atmosphere_hooks": mesh.get("atmosphere_hooks", []),
                "originality_notes": mesh.get("originality_notes", ""),
            },
            "bounds": {
                "min_x": round(bounds[0], 6),
                "max_x": round(bounds[1], 6),
                "min_y": round(bounds[2], 6),
                "max_y": round(bounds[3], 6),
                "min_z": round(bounds[4], 6),
                "max_z": round(bounds[5], 6),
            },
            "truth_mutation": False,
        })
    overlay_file = f"{arena_id}_audit_overlay_1920x1080.png"
    overlay_png, overlay_bounds = draw_capture(arena, "gameplay", debug_overlay=True)
    (out / overlay_file).write_bytes(overlay_png)
    debug_overlays.append({
        "arena_id": arena_id,
        "mode": "audit_overlay",
        "file": overlay_file,
        "width": WIDTH,
        "height": HEIGHT,
        "sha256": hash_bytes(overlay_png),
        "runtime_mesh": entry["runtime_mesh"],
        "runtime_gltf": entry["runtime_gltf"],
        "runtime_mesh_sha256": arena["runtime_mesh_hash"],
        "runtime_gltf_sha256": arena["runtime_gltf_hash"],
        "debug_overlay": True,
        "clean_product_capture": False,
        "material_map_hashes": arena["material_hashes"],
        "bounds": {
            "min_x": round(overlay_bounds[0], 6),
            "max_x": round(overlay_bounds[1], 6),
            "min_y": round(overlay_bounds[2], 6),
            "max_y": round(overlay_bounds[3], 6),
            "min_z": round(overlay_bounds[4], 6),
            "max_z": round(overlay_bounds[5], 6),
        },
        "truth_mutation": False,
    })

passed = not failures and len(captures) == len(ARENAS) * len(MODES)
payload = {
    "schema": "oathyard.arena_environment_captures.v2",
    "product": "OATHYARD",
    "tool": "tools/arena_environment_captures.sh",
    "source_manifest": "assets/runtime_manifest.json",
    "asset_hash": manifest.get("asset_hash", ""),
    "arena_count": len(ARENAS),
    "capture_count": len(captures),
    "audit_overlay_capture_count": len(debug_overlays),
    "world_identity_manifest": "arena_world_identity_manifest.json",
    "required_modes": MODES,
    "clean_product_like_capture_modes": MODES,
    "debug_overlay_captures": debug_overlays,
    "production_renderer_complete": False,
    "owner_visual_acceptance_claimed": False,
    "public_demo_ready": False,
    "release_candidate_ready": False,
    "truth_mutation": False,
    "current_blockers": [
        "software raster PNG captures are local verification evidence, not owner visual acceptance",
        "external Khronos/DCC/renderer acceptance is not claimed",
    ],
    "failed_check_count": len(failures),
    "passed": passed,
    "captures": captures,
    "world_identity": identity_entries,
    "failures": failures,
}
identity_payload = {
    "schema": "oathyard.arena_world_identity.v1",
    "product": "OATHYARD",
    "tool": "tools/arena_environment_captures.sh",
    "source_manifest": "assets/runtime_manifest.json",
    "asset_hash": manifest.get("asset_hash", ""),
    "production_renderer_complete": False,
    "owner_visual_acceptance_claimed": False,
    "public_demo_ready": False,
    "release_candidate_ready": False,
    "truth_mutation": False,
    "arenas": identity_entries,
    "debug_overlay_captures": debug_overlays,
    "source_backing": [file_ref(path) for path in SOURCE_BACKED_PATHS],
    "passed": passed,
}
(out / "arena_world_identity_manifest.json").write_text(json.dumps(identity_payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
(out / "arena_environment_capture_manifest.json").write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
(out / "failed_arena_environment_captures.txt").write_text("none\n" if not failures else "\n".join(failures) + "\n", encoding="utf-8")
report = [
    "# OATHYARD Arena Environment Captures",
    "",
    f"Status: {'PASSED' if passed else 'FAILED'}",
    f"- Asset hash: `{manifest.get('asset_hash', '')}`",
    f"- Capture count: `{len(captures)}`",
    f"- Audit overlay capture count: `{len(debug_overlays)}`",
    "- World identity manifest: `arena_world_identity_manifest.json`",
    "- Modes: `establishing`, `gameplay`, `contact` for each arena",
    "- Clean/debug split: individual required PNGs omit material-hash swatches; `*_audit_overlay_1920x1080.png` carries overlay evidence",
    "- Source: runtime manifest + runtime mesh metadata + runtime glTF geometry/material maps",
    "- Truth mutation: `false`",
    "- Owner visual acceptance claimed: `false`",
    "- Public demo ready: `false`",
    "- Release candidate ready: `false`",
    "",
    "## Captures",
]
for capture in captures:
    report.append(
        f"- `{capture['arena_id']}` `{capture['mode']}` `{capture['file']}` sha `{capture['sha256'][:16]}` bounds z `{capture['bounds']['min_z']}..{capture['bounds']['max_z']}` composition `{capture['world_identity']['composition_profile']}`"
    )
report.extend(["", "## Audit overlays"])
for overlay in debug_overlays:
    report.append(
        f"- `{overlay['arena_id']}` `{overlay['file']}` sha `{overlay['sha256'][:16]}` debug_overlay `{str(overlay['debug_overlay']).lower()}`"
    )
report.extend(["", "## World identity metadata"])
for identity in identity_entries:
    report.append(
        f"- `{identity['arena_id']}` profile `{identity['environment_profile']}` scale `{identity['scale_reference']}` silhouette `{identity['silhouette_context']}` playable `{','.join(identity['playable_space'])}`"
    )
report.extend(["", "## Audit directive mapping"])
for identity in identity_entries:
    report.append(
        f"- `{identity['arena_id']}` directives `{','.join(identity['mapped_audit_directives'])}` assets `{len(identity['asset_references'])}` source_backing `{len(identity['source_backing'])}`"
    )
if failures:
    report.extend(["", "## Failures"] + [f"- {failure}" for failure in failures])
report.extend(["", "## Explicit blockers", *[f"- {blocker}" for blocker in payload["current_blockers"]]])
(out / "arena_environment_capture_report.md").write_text("\n".join(report) + "\n", encoding="utf-8")

sheet = [
    '<svg xmlns="http://www.w3.org/2000/svg" width="1320" height="980" viewBox="0 0 1320 980">',
    '<rect width="100%" height="100%" fill="#111414"/>',
    '<text x="24" y="36" fill="#f1eadb" font-family="monospace" font-size="22">OATHYARD arena environment capture sheet</text>',
    '<text x="24" y="62" fill="#b8c5c4" font-family="monospace" font-size="13">establishing/gameplay/contact captures; source-backed runtime glTF + material-map evidence; owner visual acceptance not claimed</text>',
]
thumb_w = 400
thumb_h = 225
for idx, capture in enumerate(captures):
    col = idx % 3
    row = idx // 3
    x = 24 + col * 430
    y = 92 + row * 420
    sheet.append(f'<rect x="{x}" y="{y}" width="410" height="350" fill="#1d2525" stroke="#607170"/>')
    embedded_png = base64.b64encode((out / capture["file"]).read_bytes()).decode("ascii")
    sheet.append(f'<image href="data:image/png;base64,{embedded_png}" x="{x+5}" y="{y+8}" width="{thumb_w}" height="{thumb_h}" preserveAspectRatio="xMidYMid slice"/>')
    sheet.append(f'<text x="{x+14}" y="{y+260}" fill="#f2ead8" font-family="monospace" font-size="14">{capture["arena_id"]}</text>')
    sheet.append(f'<text x="{x+14}" y="{y+284}" fill="#b8c5c4" font-family="monospace" font-size="12">mode {capture["mode"]} | sha {capture["sha256"][:16]}</text>')
    sheet.append(f'<text x="{x+14}" y="{y+308}" fill="#b8c5c4" font-family="monospace" font-size="12">gltf {Path(capture["runtime_gltf"]).name} | z {capture["bounds"]["min_z"]}..{capture["bounds"]["max_z"]}</text>')
sheet.append('</svg>')
(out / "arena_environment_contact_sheet.svg").write_text("\n".join(sheet) + "\n", encoding="utf-8")
if not passed:
    sys.stderr.write("arena environment captures failed\n")
    sys.exit(1)
print(f"arena environment captures passed: {len(captures)} captures")
PY
