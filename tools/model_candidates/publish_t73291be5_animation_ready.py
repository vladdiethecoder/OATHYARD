#!/usr/bin/env python3
"""Generate/publish OATHYARD t_73291be5 model candidates with animation-ready fighter glTFs.

Stdlib-only. This is a reproducible replacement for the missing runtime package
reported by media QA. It keeps every readiness/owner/truth claim false.
"""
from __future__ import annotations

import binascii
import hashlib
import json
import math
import random
import shutil
import struct
import zlib
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RUN_ID = "t_73291be5"
REPAIR_TASK = "t_49e26ba5"
SRC_ROOT = ROOT / "assets_src" / "model_candidates" / RUN_ID
PKG_ROOT = ROOT / "assets" / "model_candidates" / RUN_ID
GLTF_DIR = PKG_ROOT / "gltf"
BIN_DIR = PKG_ROOT / "bin"
TEX_DIR = PKG_ROOT / "textures"
ART_ROOT = ROOT / "artifacts" / "model_candidates" / RUN_ID
VALIDATION_DIR = ART_ROOT / "validation"
MOTION_DIR = ART_ROOT / "motion_evidence"
MANIFEST_PATH = PKG_ROOT / "model_candidate_manifest.json"
SOURCE_MANIFEST_PATH = SRC_ROOT / "model_source_manifest.json"
VALIDATION_JSON = VALIDATION_DIR / "model_candidate_animation_ready_validation.json"
VALIDATION_REPORT = VALIDATION_DIR / "model_candidate_animation_ready_validation_report.md"
MOTION_JSON = MOTION_DIR / "fighter_motion_evidence.json"
MOTION_REPORT = MOTION_DIR / "fighter_motion_evidence_report.md"
HANDOFF_PATH = ART_ROOT / "ANIMATION_READY_REPAIR_HANDOFF.md"

CANONICAL_TRUTH_JOINTS = [
    "root", "spine_lower", "spine_upper", "neck_head",
    "shoulder_r", "elbow_r", "wrist_r", "shoulder_l", "elbow_l", "wrist_l",
    "hip_r", "knee_r", "ankle_r", "hip_l", "knee_l", "ankle_l",
]
JOINT_INDEX = {name: idx for idx, name in enumerate(CANONICAL_TRUTH_JOINTS)}
FIGHTERS = ["saltreach_duelist", "oathyard_writ", "chainbreaker", "reed_sentinel", "gate_shield", "bruiser_oath"]
REQUIRED_CLIPS = ["idle", "walk", "attack"]
CRITICAL_BLEND_JOINTS = [
    "shoulder_r", "elbow_r", "shoulder_l", "elbow_l",
    "hip_r", "knee_r", "ankle_r", "hip_l", "knee_l", "ankle_l",
]
COMPONENT_INFO = {5120: ("b", 1), 5121: ("B", 1), 5122: ("h", 2), 5123: ("H", 2), 5125: ("I", 4), 5126: ("f", 4)}
TYPE_COMPS = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}

PALETTE = {
    "flesh": (0.58, 0.43, 0.34), "tendon": (0.72, 0.58, 0.48), "bone": (0.76, 0.70, 0.58),
    "cloth_linen": (0.63, 0.57, 0.48), "cloth_dark": (0.055, 0.060, 0.058),
    "oath_red": (0.46, 0.07, 0.055), "chalk_white": (0.86, 0.83, 0.72),
    "leather": (0.35, 0.20, 0.12), "buff_leather": (0.64, 0.50, 0.34), "ash_wood": (0.48, 0.41, 0.31),
    "iron_black": (0.08, 0.085, 0.09), "tarnished_steel": (0.43, 0.45, 0.43), "tempered_plate": (0.30, 0.32, 0.34),
    "mail_dark": (0.22, 0.23, 0.23), "lamellar_iron": (0.24, 0.21, 0.17),
    "stone_cold": (0.42, 0.43, 0.42), "clay": (0.37, 0.26, 0.18), "lamp_warm": (0.78, 0.54, 0.22),
    "wet_earth": (0.16, 0.11, 0.08), "dried_blood": (0.28, 0.035, 0.025), "edge_bright": (0.72, 0.74, 0.70),
}
METAL_KEYS = {"iron_black", "tarnished_steel", "tempered_plate", "mail_dark", "lamellar_iron", "edge_bright"}
ROUGH_KEYS = {"stone_cold", "clay", "wet_earth", "cloth_linen", "cloth_dark"}
STYLE_ANCHORS = [
    "readable duel silhouette before ornament",
    "cold oath-stone palette with restrained legal/faction accents",
    "tactile PBR material families: cloth, leather, mail, plate, ash wood, stone, blood/wear",
    "riggable truth-joint candidates with synthetic idle/walk/attack motion proof",
]
BUDGETS = {
    "fighter": {"min": 18_000, "target_max": 30_000, "hard_max": 40_000},
    "weapon_one_handed": {"min": 800, "target_max": 2_500, "hard_max": 4_000},
    "weapon_two_handed": {"min": 1_200, "target_max": 3_500, "hard_max": 5_000},
    "shield": {"min": 2_000, "target_max": 5_000, "hard_max": 8_000},
    "armor": {"min": 500, "target_max": 5_000, "hard_max": 8_000},
    "arena": {"min": 2_000, "target_max": 60_000, "hard_max": 90_000},
}


def ensure_dirs():
    for path in [SRC_ROOT, PKG_ROOT, GLTF_DIR, BIN_DIR, TEX_DIR, ART_ROOT, VALIDATION_DIR, MOTION_DIR]:
        path.mkdir(parents=True, exist_ok=True)
    for category in ["fighters", "weapons", "armor", "arenas"]:
        (SRC_ROOT / category).mkdir(parents=True, exist_ok=True)


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def parse_records(path: Path, record_kind: str) -> dict[str, dict[str, str]]:
    records = {}
    for raw in path.read_text(encoding="utf-8").splitlines():
        raw = raw.strip()
        if not raw or raw.startswith("#"):
            continue
        tokens = raw.split()
        if len(tokens) >= 2 and tokens[0] == record_kind:
            row = {"id": tokens[1]}
            for token in tokens[2:]:
                if "=" in token:
                    k, v = token.split("=", 1)
                    row[k] = v
            records[tokens[1]] = row
    return records


def parse_content_manifest(path: Path) -> dict[str, list[str]]:
    sections = {}
    current = None
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            current = line[1:-1]
            sections[current] = []
            continue
        if current and "=" not in line:
            sections[current].append(line)
    return sections


def v_add(a, b): return (a[0] + b[0], a[1] + b[1], a[2] + b[2])
def v_sub(a, b): return (a[0] - b[0], a[1] - b[1], a[2] - b[2])
def v_mul(a, s): return (a[0] * s, a[1] * s, a[2] * s)
def v_len(a): return math.sqrt(a[0] * a[0] + a[1] * a[1] + a[2] * a[2])
def v_norm(a):
    length = v_len(a)
    return (0.0, 1.0, 0.0) if length <= 1e-12 else (a[0] / length, a[1] / length, a[2] / length)
def v_cross(a, b): return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])


class MeshBuilder:
    def __init__(self, asset_id: str, kind: str, skin: bool = False):
        self.asset_id = asset_id
        self.kind = kind
        self.skin = skin
        self.positions = []
        self.normals = []
        self.uvs = []
        self.groups = {}
        self.material_order = []
        self.preview_view = "front"

    def material(self, key: str) -> str:
        if key not in self.groups:
            self.groups[key] = []
            self.material_order.append(key)
        return key

    def vertex(self, pos, normal, uv) -> int:
        idx = len(self.positions)
        self.positions.append(tuple(float(x) for x in pos))
        self.normals.append(v_norm(normal))
        self.uvs.append((float(uv[0]), float(uv[1])))
        return idx

    def tri(self, material, a, b, c):
        self.groups[self.material(material)].extend([a, b, c])

    def quad(self, material, a, b, c, d):
        self.tri(material, a, b, c)
        self.tri(material, a, c, d)

    def add_box(self, center, size, material):
        cx, cy, cz = center
        sx, sy, sz = size[0] / 2, size[1] / 2, size[2] / 2
        faces = [
            ((1, 0, 0), [(sx, -sy, -sz), (sx, -sy, sz), (sx, sy, sz), (sx, sy, -sz)]),
            ((-1, 0, 0), [(-sx, -sy, sz), (-sx, -sy, -sz), (-sx, sy, -sz), (-sx, sy, sz)]),
            ((0, 1, 0), [(-sx, sy, -sz), (sx, sy, -sz), (sx, sy, sz), (-sx, sy, sz)]),
            ((0, -1, 0), [(-sx, -sy, sz), (sx, -sy, sz), (sx, -sy, -sz), (-sx, -sy, -sz)]),
            ((0, 0, 1), [(sx, -sy, sz), (-sx, -sy, sz), (-sx, sy, sz), (sx, sy, sz)]),
            ((0, 0, -1), [(-sx, -sy, -sz), (sx, -sy, -sz), (sx, sy, -sz), (-sx, sy, -sz)]),
        ]
        for normal, offsets in faces:
            verts = [self.vertex((cx + x, cy + y, cz + z), normal, uv) for (x, y, z), uv in zip(offsets, [(0, 0), (1, 0), (1, 1), (0, 1)])]
            self.quad(material, verts[0], verts[1], verts[2], verts[3])

    def add_cylinder_between(self, p0, p1, radius, material, seg=16, cap=True):
        axis = v_sub(p1, p0)
        w = v_norm(axis)
        tmp = (0.0, 1.0, 0.0) if abs(w[1]) < 0.92 else (1.0, 0.0, 0.0)
        u = v_norm(v_cross(w, tmp))
        v = v_norm(v_cross(u, w))
        ring0, ring1 = [], []
        for i in range(seg):
            angle = 2 * math.pi * i / seg
            radial = v_add(v_mul(u, math.cos(angle) * radius), v_mul(v, math.sin(angle) * radius))
            normal = v_norm(radial)
            ring0.append(self.vertex(v_add(p0, radial), normal, (i / seg, 0.0)))
            ring1.append(self.vertex(v_add(p1, radial), normal, (i / seg, 1.0)))
        for i in range(seg):
            j = (i + 1) % seg
            self.quad(material, ring0[i], ring0[j], ring1[j], ring1[i])
        if cap:
            c0 = self.vertex(p0, v_mul(w, -1.0), (0.5, 0.5))
            c1 = self.vertex(p1, w, (0.5, 0.5))
            for i in range(seg):
                j = (i + 1) % seg
                self.tri(material, c0, ring0[j], ring0[i])
                self.tri(material, c1, ring1[i], ring1[j])

    def add_disc(self, center, normal_axis, radius, depth, material, seg=48):
        cx, cy, cz = center
        if normal_axis == "z":
            self.add_cylinder_between((cx, cy, cz - depth / 2), (cx, cy, cz + depth / 2), radius, material, seg, True)
        elif normal_axis == "y":
            self.add_cylinder_between((cx, cy - depth / 2, cz), (cx, cy + depth / 2, cz), radius, material, seg, True)
        else:
            self.add_cylinder_between((cx - depth / 2, cy, cz), (cx + depth / 2, cy, cz), radius, material, seg, True)

    def add_rivet(self, center, normal, radius, depth, material, seg=8):
        n = v_norm(normal)
        self.add_cylinder_between(v_sub(center, v_mul(n, depth * 0.35)), v_add(center, v_mul(n, depth * 0.65)), radius, material, seg, True)

    def add_extruded_xy(self, points, z_center, depth, material):
        if len(points) < 3:
            return
        zf, zb = z_center + depth / 2, z_center - depth / 2
        front = [self.vertex((x, y, zf), (0, 0, 1), (x, y)) for x, y in points]
        back = [self.vertex((x, y, zb), (0, 0, -1), (x, y)) for x, y in points]
        for i in range(1, len(points) - 1):
            self.tri(material, front[0], front[i], front[i + 1])
            self.tri(material, back[0], back[i + 1], back[i])
        for i in range(len(points)):
            j = (i + 1) % len(points)
            p_i, p_j = points[i], points[j]
            edge = (p_j[0] - p_i[0], p_j[1] - p_i[1], 0.0)
            normal = v_norm((edge[1], -edge[0], 0.0))
            a = self.vertex((p_i[0], p_i[1], zb), normal, (0, 0))
            b = self.vertex((p_j[0], p_j[1], zb), normal, (1, 0))
            c = self.vertex((p_j[0], p_j[1], zf), normal, (1, 1))
            d = self.vertex((p_i[0], p_i[1], zf), normal, (0, 1))
            self.quad(material, a, b, c, d)

    def triangle_count(self):
        return sum(len(v) for v in self.groups.values()) // 3

    def vertex_count(self):
        return len(self.positions)


def add_detail_rivets(builder, rng, count, mats, xspan, yspan, z=-0.11, radius=0.006, depth=0.009):
    for _ in range(count):
        x = rng.uniform(*xspan)
        y = rng.uniform(*yspan)
        mat = rng.choice(mats)
        builder.add_rivet((x, y, z), (0, 0, -1), radius * rng.uniform(0.7, 1.35), depth, mat, 8)


def build_fighter(asset_id, fields, loadout):
    b = MeshBuilder(asset_id, "fighter", skin=True)
    rng = random.Random(asset_id)
    mass = int(fields.get("body_mass_g", "82000"))
    reach = int(fields.get("reach_bias_mm", "0")) / 1000.0
    height = 1.62 + min(max(mass - 74000, 0), 26000) / 100000.0
    width = 0.30 + min(max(mass - 74000, 0), 26000) / 220000.0
    bulk = 1.0 + min(max(mass - 80000, -10000), 20000) / 100000.0
    shoulder_y, hip_y, knee_y, ankle_y, head_y = height * 0.78, height * 0.46, height * 0.30, 0.11, height * 0.93
    front_z, back_z = -0.10, 0.10
    b.add_cylinder_between((0, hip_y - 0.06, 0), (0, shoulder_y, 0), 0.18 * bulk, "flesh", 48, True)
    b.add_box((0, hip_y - 0.09, 0), (width * 1.40, 0.18, 0.24), "leather")
    b.add_cylinder_between((0, head_y - 0.05, 0), (0, head_y + 0.17, 0), 0.085, "flesh", 40, True)
    b.add_cylinder_between((0, shoulder_y - 0.04, 0), (0, head_y - 0.05, 0), 0.045, "tendon", 28, True)
    for side, sx in [("r", -1), ("l", 1)]:
        shoulder = (sx * width * 0.78, shoulder_y - 0.02, 0)
        elbow = (sx * (width * 1.03 + 0.03), shoulder_y - 0.35, -0.015 * sx)
        wrist = (sx * (width * 0.75 + 0.05 * abs(reach)), shoulder_y - 0.67, front_z)
        b.add_cylinder_between(shoulder, elbow, 0.052 * bulk, "flesh", 32, True)
        b.add_cylinder_between(elbow, wrist, 0.043 * bulk, "flesh", 32, True)
        b.add_box((wrist[0], wrist[1] - 0.025, wrist[2] - 0.012), (0.07, 0.055, 0.04), "flesh")
        for k in range(5):
            t = k / 4.0
            px = elbow[0] + (wrist[0] - elbow[0]) * t
            py = elbow[1] + (wrist[1] - elbow[1]) * t
            pz = elbow[2] + (wrist[2] - elbow[2]) * t
            b.add_cylinder_between((px - 0.012 * sx, py, pz), (px + 0.012 * sx, py + 0.004, pz), 0.055, "buff_leather", 12, True)
    stance_w = width * (0.56 if asset_id != "gate_shield" else 0.82)
    for side, sx in [("r", -1), ("l", 1)]:
        hip = (sx * stance_w * 0.45, hip_y, 0)
        knee = (sx * stance_w * 0.55, knee_y, -0.02 if asset_id in {"saltreach_duelist", "reed_sentinel"} else 0.015)
        ankle = (sx * stance_w * 0.65, ankle_y, -0.04 if side == "l" else 0.03)
        b.add_cylinder_between(hip, knee, 0.072 * bulk, "flesh", 34, True)
        b.add_cylinder_between(knee, ankle, 0.058 * bulk, "flesh", 34, True)
        b.add_box((ankle[0], 0.035, ankle[2] - 0.045), (0.15, 0.07, 0.24), "leather")
        b.add_cylinder_between((knee[0], knee[1], knee[2] - 0.01), (knee[0], knee[1], knee[2] - 0.05), 0.075, "buff_leather", 18, True)
    armor_key = loadout.get("armor", "gambeson")
    if armor_key in {"mail_hauberk", "lamellar", "heavy_plate", "bruiser_padded_plate"}:
        chest_mat = "mail_dark" if armor_key == "mail_hauberk" else ("lamellar_iron" if armor_key == "lamellar" else "tempered_plate")
        b.add_box((0, shoulder_y - 0.20, front_z - 0.026), (width * 1.35, 0.50, 0.055), chest_mat)
        b.add_box((0, hip_y + 0.06, front_z - 0.025), (width * 1.12, 0.25, 0.052), chest_mat)
        for sx in [-1, 1]:
            b.add_box((sx * width * 0.72, shoulder_y - 0.07, front_z - 0.018), (0.20, 0.14, 0.06), chest_mat)
    else:
        b.add_box((0, shoulder_y - 0.22, front_z - 0.02), (width * 1.22, 0.54, 0.04), "cloth_linen")
    if asset_id == "saltreach_duelist":
        for k in range(18): b.add_box((-0.20 + 0.38 * k / 17, shoulder_y - 0.02 - 0.52 * k / 17, front_z - 0.055), (0.072, 0.020, 0.025), "chalk_white")
        b.add_box((-0.06, head_y + 0.08, front_z - 0.08), (0.14, 0.045, 0.022), "oath_red")
    elif asset_id == "oathyard_writ":
        b.add_box((0, shoulder_y - 0.18, front_z - 0.06), (0.24, 0.63, 0.03), "cloth_dark")
        b.add_box((0, shoulder_y - 0.02, front_z - 0.084), (0.105, 0.020, 0.022), "oath_red")
        b.add_box((0, shoulder_y - 0.10, front_z - 0.084), (0.020, 0.13, 0.022), "oath_red")
    elif asset_id == "chainbreaker":
        for k in range(9): b.add_disc((-0.23 + k * 0.057, shoulder_y - 0.16 + (k % 2) * 0.035, front_z - 0.070), "z", 0.023, 0.014, "tarnished_steel", 12)
        b.add_box((0.18, hip_y + 0.15, front_z - 0.065), (0.09, 0.42, 0.035), "oath_red")
    elif asset_id == "reed_sentinel":
        for k in range(11): b.add_box((-0.06 + 0.012 * (k % 2), hip_y + 0.03 + k * 0.045, front_z - 0.060), (0.33, 0.012, 0.022), "ash_wood")
    elif asset_id == "gate_shield":
        b.add_disc((-0.30, shoulder_y - 0.28, front_z - 0.085), "z", 0.18, 0.045, "chalk_white", 56)
        b.add_disc((-0.30, shoulder_y - 0.28, front_z - 0.117), "z", 0.072, 0.055, "tempered_plate", 40)
    elif asset_id == "bruiser_oath":
        b.add_box((0, head_y + 0.03, front_z - 0.075), (0.20, 0.18, 0.040), "iron_black")
        b.add_box((0, shoulder_y - 0.22, front_z - 0.070), (width * 1.55, 0.60, 0.060), "cloth_dark")
    add_detail_rivets(b, rng, 450, ["tarnished_steel", "buff_leather", "chalk_white", "dried_blood"], (-width * 0.72, width * 0.72), (0.18, shoulder_y + 0.05), front_z - 0.095)
    add_detail_rivets(b, rng, 210, ["mail_dark", "leather", "tarnished_steel"], (-width * 0.88, width * 0.88), (shoulder_y - 0.42, shoulder_y + 0.02), back_z + 0.025, 0.005, 0.008)
    while b.triangle_count() < BUDGETS["fighter"]["min"]:
        add_detail_rivets(b, rng, 24, ["chalk_white", "tarnished_steel", "leather"], (-width * 0.75, width * 0.75), (0.25, shoulder_y + 0.08), front_z - 0.10, 0.0055, 0.008)
    return b


def build_weapon(asset_id, fields):
    b = MeshBuilder(asset_id, "weapon")
    length = int(fields.get("length_mm", "900")) / 1000.0
    shaft = "ash_wood" if asset_id in {"ash_spear", "billhook"} else "leather"
    metal = "edge_bright" if "sword" in asset_id or "spear" in asset_id else "iron_black"
    if asset_id == "round_shield":
        b.add_disc((0, 0, 0), "z", 0.33, 0.09, "chalk_white", 96)
        b.add_disc((0, 0, -0.065), "z", 0.12, 0.08, "tempered_plate", 64)
        b.add_disc((0, 0, -0.092), "z", 0.35, 0.030, "oath_red", 112)
        for i in range(60):
            a = 2 * math.pi * i / 60
            r = 0.26 + 0.025 * (i % 2)
            b.add_rivet((math.cos(a) * r, math.sin(a) * r, -0.115), (0, 0, -1), 0.012, 0.018, "tarnished_steel", 10)
    else:
        b.add_cylinder_between((-length * 0.46, 0, 0), (length * 0.46, 0, 0), 0.026 if asset_id in {"ash_spear", "billhook"} else 0.038, shaft, 32, True)
        for k in range(14): b.add_cylinder_between((-length * 0.22 + k * length * 0.032, -0.038, 0), (-length * 0.22 + k * length * 0.032, 0.038, 0), 0.012, "leather", 12, True)
        if "sword" in asset_id:
            blade_len = length * (0.62 if asset_id == "arming_sword" else 0.72)
            start = -length * 0.08
            if asset_id == "curved_sword":
                for k in range(10):
                    x0, x1 = start + k * blade_len / 10, start + (k + 1) * blade_len / 10
                    curve = 0.055 * math.sin(k / 9 * math.pi)
                    b.add_extruded_xy([(x0, -0.026 + curve), (x1, -0.018 + curve + 0.024), (x1, 0.018 + curve + 0.024), (x0, 0.026 + curve)], -0.015, 0.035, metal)
            else:
                b.add_extruded_xy([(start, -0.035), (start + blade_len * 0.88, -0.028), (start + blade_len, 0), (start + blade_len * 0.88, 0.028), (start, 0.035)], -0.012, 0.032, metal)
            b.add_box((start - 0.06, 0, -0.012), (0.045, 0.32, 0.035), "tarnished_steel")
            b.add_disc((start - 0.23, 0, -0.006), "x", 0.050, 0.06, "tempered_plate", 24)
        elif asset_id == "ash_spear":
            tip = length * 0.48
            b.add_extruded_xy([(tip - 0.28, -0.050), (tip + 0.03, 0), (tip - 0.28, 0.050), (tip - 0.19, 0)], -0.015, 0.035, "edge_bright")
            b.add_box((tip - 0.33, 0, -0.012), (0.075, 0.12, 0.030), "tarnished_steel")
        elif asset_id == "bearded_axe":
            hx = length * 0.34
            b.add_extruded_xy([(hx - 0.12, 0.06), (hx + 0.16, 0.17), (hx + 0.10, -0.20), (hx - 0.07, -0.10)], -0.020, 0.060, "iron_black")
            b.add_extruded_xy([(hx + 0.04, -0.19), (hx + 0.18, -0.27), (hx + 0.08, -0.05)], -0.018, 0.045, "edge_bright")
        elif asset_id == "billhook":
            hx = length * 0.38
            b.add_extruded_xy([(hx - 0.18, 0.020), (hx + 0.05, 0.18), (hx + 0.22, 0.04), (hx + 0.08, -0.07), (hx - 0.08, -0.035)], -0.018, 0.050, "edge_bright")
            b.add_box((hx - 0.25, 0, -0.012), (0.08, 0.11, 0.030), "tarnished_steel")
        else:  # iron_maul
            hx = length * 0.34
            b.add_box((hx, 0, -0.018), (0.20, 0.22, 0.12), "iron_black")
            b.add_box((hx, 0.13, -0.018), (0.18, 0.035, 0.13), "edge_bright")
    return b


def build_armor(asset_id, _fields):
    b = MeshBuilder(asset_id, "armor")
    mat = {
        "gambeson": "cloth_linen", "mail_hauberk": "mail_dark", "heavy_plate": "tempered_plate",
        "lamellar": "lamellar_iron", "fencer_light": "buff_leather", "bruiser_padded_plate": "iron_black",
    }.get(asset_id, "cloth_linen")
    b.add_box((0, 0.9, 0), (0.62, 0.92, 0.18), mat)
    b.add_box((0, 0.35, -0.005), (0.52, 0.30, 0.16), mat)
    for sx in [-1, 1]:
        b.add_box((sx * 0.38, 1.05, 0), (0.18, 0.22, 0.16), mat)
        b.add_box((sx * 0.40, 0.75, 0), (0.10, 0.42, 0.12), "leather")
    detail = "chalk_white" if asset_id in {"gambeson", "fencer_light"} else "edge_bright"
    for k in range(12):
        y = 0.45 + k * 0.055
        b.add_box((0, y, -0.105), (0.56, 0.012, 0.020), detail)
    if asset_id == "mail_hauberk":
        for i in range(11):
            for j in range(5): b.add_disc((-0.25 + j * 0.125, 0.48 + i * 0.06, -0.11), "z", 0.012, 0.006, "tarnished_steel", 8)
    if asset_id in {"heavy_plate", "lamellar", "bruiser_padded_plate"}:
        for i in range(7): b.add_box((0, 0.52 + i * 0.08, -0.12), (0.58 - i * 0.015, 0.030, 0.030), "edge_bright")
    return b


def build_arena(asset_id, fields):
    b = MeshBuilder(asset_id, "arena")
    b.preview_view = "top"
    radius = int(fields.get("radius_mm", "5000")) / 1000.0
    floor = "stone_cold" if asset_id == "oathyard_verdict_ring" else "clay"
    size = radius * 2
    b.add_box((0, 0, -0.02), (size, size, 0.04), floor)
    if asset_id == "oathyard_verdict_ring":
        b.add_disc((0, 0, 0.02), "z", radius * 0.92, 0.045, "chalk_white", 160)
        b.add_disc((0, 0, 0.055), "z", radius * 0.42, 0.040, "oath_red", 120)
        for i in range(32):
            a = 2 * math.pi * i / 32
            b.add_box((math.cos(a) * radius * 0.78, math.sin(a) * radius * 0.78, 0.09), (0.16, 0.16, 0.18), "stone_cold")
    else:
        for i in range(-5, 6):
            b.add_box((i * radius / 5, 0, 0.04), (0.025, size, 0.035), "chalk_white")
            b.add_box((0, i * radius / 5, 0.04), (size, 0.025, 0.035), "chalk_white")
        for sx in [-1, 1]:
            for sy in [-1, 1]: b.add_box((sx * radius * 0.82, sy * radius * 0.82, 0.20), (0.12, 0.12, 0.40), "ash_wood")
    return b


def budget_kind(asset_id, kind):
    if kind == "fighter": return "fighter"
    if kind == "armor": return "armor"
    if kind == "arena": return "arena"
    if asset_id == "round_shield": return "shield"
    if asset_id in {"longsword", "ash_spear", "billhook", "iron_maul"}: return "weapon_two_handed"
    return "weapon_one_handed"


def ensure_candidate_budget(builder, asset_id, budget_key):
    """Add deterministic authored detail until every candidate family clears its lane budget.

    The first candidate generator enforced the fighter triangle floor but only recorded the
    non-fighter floors.  This pass keeps the same source-backed procedural lane and adds
    visible straps, rivets, collars, rings, stones, laces, and measured-yard markers rather
    than inflating counts with hidden geometry.
    """
    target = BUDGETS[budget_key]["min"]
    if builder.triangle_count() >= target:
        return builder
    rng = random.Random(f"{asset_id}:candidate_budget_detail")
    if budget_key in {"weapon_one_handed", "weapon_two_handed"} and asset_id != "round_shield":
        xs = [p[0] for p in builder.positions]
        x0, x1 = min(xs), max(xs)
        span = max(x1 - x0, 0.25)
        n = 0
        while builder.triangle_count() < target:
            x = x0 + span * (0.12 + 0.76 * ((n * 37) % 101) / 100.0)
            if n % 4 == 0:
                builder.add_cylinder_between((x, -0.050, -0.042), (x, 0.050, -0.042), 0.010, "leather", 10, True)
            elif n % 4 == 1:
                builder.add_rivet((x, 0.065, -0.045), (0, 0, -1), 0.010, 0.014, "tarnished_steel", 8)
                builder.add_rivet((x, -0.065, -0.045), (0, 0, -1), 0.010, 0.014, "tarnished_steel", 8)
            elif n % 4 == 2:
                builder.add_box((x, 0.0, -0.052), (0.030, 0.115, 0.018), "edge_bright")
            else:
                y = rng.choice([-0.038, 0.038])
                builder.add_cylinder_between((x - 0.022, y, -0.046), (x + 0.022, y, -0.046), 0.007, "edge_bright", 8, True)
            n += 1
    elif budget_key == "armor":
        n = 0
        while builder.triangle_count() < target:
            row = n // 12
            col = n % 12
            x = -0.28 + col * 0.051
            y = 0.40 + row * 0.052
            if n % 3 == 0:
                builder.add_disc((x, y, -0.128), "z", 0.010, 0.007, "tarnished_steel", 8)
            elif n % 3 == 1:
                builder.add_box((x, y, -0.130), (0.033, 0.010, 0.014), "edge_bright")
            else:
                builder.add_cylinder_between((x - 0.014, y, -0.130), (x + 0.014, y + 0.018, -0.130), 0.0045, "leather", 8, True)
            n += 1
    elif budget_key == "arena":
        xs = [p[0] for p in builder.positions]
        ys = [p[1] for p in builder.positions]
        radius = max(max(abs(x) for x in xs), max(abs(y) for y in ys), 1.0)
        n = 0
        while builder.triangle_count() < target:
            angle = 2 * math.pi * ((n * 17) % 97) / 97.0
            ring = 0.34 + 0.56 * ((n * 29) % 53) / 52.0
            x = math.cos(angle) * radius * ring
            y = math.sin(angle) * radius * ring
            if n % 5 == 0:
                builder.add_box((x, y, 0.105), (0.115, 0.035, 0.055), "chalk_white")
            elif n % 5 == 1:
                builder.add_disc((x, y, 0.095), "z", 0.035, 0.018, "stone_cold", 10)
            elif n % 5 == 2:
                builder.add_box((x, y, 0.075), (0.060, 0.060, 0.050), "ash_wood")
            elif n % 5 == 3:
                builder.add_box((x, y, 0.062), (0.090, 0.012, 0.035), "dried_blood")
            else:
                builder.add_cylinder_between((x - 0.040, y, 0.072), (x + 0.040, y, 0.072), 0.007, "lamp_warm", 8, True)
            n += 1
    if builder.triangle_count() > BUDGETS[budget_key]["hard_max"]:
        raise ValueError(f"{asset_id} candidate detail exceeded hard max: {builder.triangle_count()} > {BUDGETS[budget_key]['hard_max']}")
    return builder


def build_asset(asset):
    if asset["kind"] == "fighter": builder = build_fighter(asset["id"], asset["fields"], asset["loadout"])
    elif asset["kind"] == "weapon": builder = build_weapon(asset["id"], asset["fields"])
    elif asset["kind"] == "armor": builder = build_armor(asset["id"], asset["fields"])
    elif asset["kind"] == "arena": builder = build_arena(asset["id"], asset["fields"])
    else: raise ValueError(asset)
    return ensure_candidate_budget(builder, asset["id"], budget_kind(asset["id"], asset["kind"]))


def gather_assets():
    sections = parse_content_manifest(ROOT / "content" / "oathyard_content.manifest")
    fighters = parse_records(ROOT / "assets_src" / "fighters" / "traditions.oysrc", "fighter")
    weapons = parse_records(ROOT / "assets_src" / "weapons" / "weapons.oysrc", "weapon")
    armors = parse_records(ROOT / "assets_src" / "armor" / "armor.oysrc", "armor")
    arenas = parse_records(ROOT / "assets_src" / "arenas" / "arenas.oysrc", "arena")
    assets = []
    for row in sections.get("fighters", []):
        asset_id, weapon, armor = row.split(":")[:3]
        assets.append({"id": asset_id, "kind": "fighter", "category": "fighters", "fields": fighters[asset_id], "loadout": {"weapon": weapon, "armor": armor}})
    for row in sections.get("weapons", []):
        asset_id = row.split(":")[0]
        assets.append({"id": asset_id, "kind": "weapon", "category": "weapons", "fields": weapons[asset_id], "loadout": {}})
    for row in sections.get("armor", []):
        asset_id = row.split(":")[0]
        assets.append({"id": asset_id, "kind": "armor", "category": "armor", "fields": armors[asset_id], "loadout": {}})
    for row in sections.get("arenas", []):
        asset_id = row.split(":")[0]
        assets.append({"id": asset_id, "kind": "arena", "category": "arenas", "fields": arenas[asset_id], "loadout": {}})
    return assets


def png_chunk(kind: bytes, data: bytes) -> bytes:
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", binascii.crc32(kind + data) & 0xFFFFFFFF)


def write_png_rgb(path: Path, width: int, height: int, pixels):
    raw = bytearray()
    for row in pixels:
        raw.append(0)
        for r, g, b in row:
            raw.extend([max(0, min(255, int(r))), max(0, min(255, int(g))), max(0, min(255, int(b)))])
    data = b"\x89PNG\r\n\x1a\n" + png_chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)) + png_chunk(b"IDAT", zlib.compress(bytes(raw), 9)) + png_chunk(b"IEND", b"")
    path.write_bytes(data)


def write_textures(asset_id, primary_materials):
    colors = [PALETTE.get(m, (0.5, 0.5, 0.5)) for m in primary_materials if m in PALETTE] or [(0.45, 0.42, 0.36)]
    avg = tuple(sum(c[i] for c in colors) / len(colors) for i in range(3))
    files = {"base": f"{asset_id}_base.png", "normal": f"{asset_id}_normal.png", "orm": f"{asset_id}_orm.png"}
    base_rows, normal_rows, orm_rows = [], [], []
    seed = int(hashlib.sha256(asset_id.encode()).hexdigest()[:8], 16)
    rng = random.Random(seed)
    for y in range(32):
        br, nr, orow = [], [], []
        for x in range(32):
            noise = 0.82 + 0.28 * rng.random()
            stripe = 1.10 if (x + y) % 7 == 0 else 1.0
            br.append(tuple(int(max(0, min(1, c * noise * stripe)) * 255) for c in avg))
            nr.append((128 + (x % 5) * 3, 128 + (y % 5) * 3, 255))
            orow.append((160, 70 if any(m in METAL_KEYS for m in primary_materials) else 20, 210 if any(m in ROUGH_KEYS for m in primary_materials) else 165))
        base_rows.append(br); normal_rows.append(nr); orm_rows.append(orow)
    write_png_rgb(TEX_DIR / files["base"], 32, 32, base_rows)
    write_png_rgb(TEX_DIR / files["normal"], 32, 32, normal_rows)
    write_png_rgb(TEX_DIR / files["orm"], 32, 32, orm_rows)
    return files


def material_record(asset_id, key):
    color = PALETTE.get(key, (0.5, 0.5, 0.5))
    metal = 0.75 if key in METAL_KEYS else 0.0
    rough = 0.86 if key in ROUGH_KEYS else 0.58
    return {
        "name": f"{asset_id}_{key}",
        "extras": {"material_family": key, "truth_authoritative": False},
        "pbrMetallicRoughness": {
            "baseColorFactor": [round(color[0], 6), round(color[1], 6), round(color[2], 6), 1.0],
            "baseColorTexture": {"index": 0}, "metallicFactor": metal, "roughnessFactor": rough,
            "metallicRoughnessTexture": {"index": 2},
        },
        "normalTexture": {"index": 1, "scale": 0.7}, "occlusionTexture": {"index": 2, "strength": 0.65},
    }


def normalize_influences(influences):
    filtered = [(name, max(0.0, float(weight))) for name, weight in influences if weight > 1e-6 and name in JOINT_INDEX]
    filtered.sort(key=lambda item: item[1], reverse=True)
    filtered = filtered[:4] or [("root", 1.0)]
    total = sum(w for _, w in filtered)
    joints = [JOINT_INDEX[name] for name, _ in filtered]
    weights = [w / total for _, w in filtered]
    while len(joints) < 4:
        joints.append(0); weights.append(0.0)
    weights[0] += 1.0 - sum(weights)
    return tuple(joints), tuple(weights)


def blended_weight_for_position(pos, mins, maxs):
    x, y, _z = pos
    height = max(maxs[1] - mins[1], 1e-6)
    y01 = (y - mins[1]) / height
    span_x = max(abs(mins[0]), abs(maxs[0]), 0.25)
    side = "l" if x >= 0.0 else "r"
    xabs = abs(x)
    if y01 < 0.13:
        return normalize_influences([(f"ankle_{side}", 0.66), (f"knee_{side}", 0.34)])
    if y01 < 0.34:
        return normalize_influences([(f"knee_{side}", 0.54), (f"hip_{side}", 0.33), (f"ankle_{side}", 0.13)])
    if xabs > span_x * 0.50 and y01 > 0.40:
        if y01 > 0.68:
            return normalize_influences([(f"shoulder_{side}", 0.58), (f"elbow_{side}", 0.29), ("spine_upper", 0.13)])
        if y01 > 0.51:
            return normalize_influences([(f"elbow_{side}", 0.55), (f"wrist_{side}", 0.30), (f"shoulder_{side}", 0.15)])
        return normalize_influences([(f"wrist_{side}", 0.61), (f"elbow_{side}", 0.33), (f"shoulder_{side}", 0.06)])
    if y01 < 0.53 and xabs > span_x * 0.13:
        return normalize_influences([(f"hip_{side}", 0.54), ("spine_lower", 0.32), (f"knee_{side}", 0.14)])
    if y01 > 0.84:
        return normalize_influences([("neck_head", 0.72), ("spine_upper", 0.28)])
    if y01 > 0.63:
        return normalize_influences([("spine_upper", 0.66), ("spine_lower", 0.22), ("neck_head", 0.12)])
    return normalize_influences([("spine_lower", 0.62), ("spine_upper", 0.24), (f"hip_{side}", 0.14)])


def skeleton_world_layout(mins, maxs):
    height = max(maxs[1] - mins[1], 1e-6)
    span_x = max(abs(mins[0]), abs(maxs[0]), 0.25)
    min_z = mins[2]
    shoulder_y, hip_y, knee_y, ankle_y, head_y = mins[1] + height * 0.78, mins[1] + height * 0.46, mins[1] + height * 0.30, mins[1] + height * 0.065, mins[1] + height * 0.92
    shoulder_x, elbow_x, wrist_x = span_x * 0.66, span_x * 0.90, span_x * 0.72
    hip_x, knee_x, ankle_x = span_x * 0.22, span_x * 0.30, span_x * 0.39
    front_z = min_z * 0.40
    world = {
        "root": (0.0, mins[1], 0.0), "spine_lower": (0.0, hip_y, 0.0), "spine_upper": (0.0, shoulder_y, 0.0), "neck_head": (0.0, head_y, 0.0),
        "shoulder_r": (-shoulder_x, shoulder_y - height * 0.012, 0.0), "elbow_r": (-elbow_x, shoulder_y - height * 0.19, front_z * 0.2), "wrist_r": (-wrist_x, shoulder_y - height * 0.37, front_z),
        "shoulder_l": (shoulder_x, shoulder_y - height * 0.012, 0.0), "elbow_l": (elbow_x, shoulder_y - height * 0.19, -front_z * 0.2), "wrist_l": (wrist_x, shoulder_y - height * 0.37, front_z),
        "hip_r": (-hip_x, hip_y, 0.0), "knee_r": (-knee_x, knee_y, front_z * 0.15), "ankle_r": (-ankle_x, ankle_y, -front_z * 0.30),
        "hip_l": (hip_x, hip_y, 0.0), "knee_l": (knee_x, knee_y, -front_z * 0.15), "ankle_l": (ankle_x, ankle_y, front_z * 0.30),
    }
    parents = {"root": None, "spine_lower": "root", "spine_upper": "spine_lower", "neck_head": "spine_upper", "shoulder_r": "spine_upper", "elbow_r": "shoulder_r", "wrist_r": "elbow_r", "shoulder_l": "spine_upper", "elbow_l": "shoulder_l", "wrist_l": "elbow_l", "hip_r": "root", "knee_r": "hip_r", "ankle_r": "knee_r", "hip_l": "root", "knee_l": "hip_l", "ankle_l": "knee_l"}
    return world, parents


def rel_translation(world, parents, joint):
    parent = parents[joint]
    base = (0.0, 0.0, 0.0) if parent is None else world[parent]
    return [round(world[joint][i] - base[i], 6) for i in range(3)]


def inverse_bind_matrix(point):
    x, y, z = point
    return (1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -x, -y, -z, 1.0)


def quat(axis, degrees):
    ax, ay, az = axis
    length = math.sqrt(ax * ax + ay * ay + az * az) or 1.0
    ax, ay, az = ax / length, ay / length, az / length
    radians = math.radians(degrees)
    s = math.sin(radians * 0.5)
    return (round(ax * s, 7), round(ay * s, 7), round(az * s, 7), round(math.cos(radians * 0.5), 7))


def pack_rows(rows):
    flat = []
    for row in rows:
        flat.extend(float(v) for v in row)
    return struct.pack("<" + "f" * len(flat), *flat)


def write_gltf(builder, source_rel, budget):
    asset_id = builder.asset_id
    textures = write_textures(asset_id, builder.material_order)
    bin_buf = bytearray(); buffer_views = []; accessors = []
    def align4():
        while len(bin_buf) % 4: bin_buf.append(0)
        return len(bin_buf)
    def add_view(data, target=None):
        offset = align4(); bin_buf.extend(data)
        rec = {"buffer": 0, "byteOffset": offset, "byteLength": len(data)}
        if target is not None: rec["target"] = target
        buffer_views.append(rec); return len(buffer_views) - 1
    def add_accessor(view, component_type, count, accessor_type, **extra):
        rec = {"bufferView": view, "byteOffset": 0, "componentType": component_type, "count": count, "type": accessor_type}
        rec.update(extra); accessors.append(rec); return len(accessors) - 1
    mins = [min(p[i] for p in builder.positions) for i in range(3)]; maxs = [max(p[i] for p in builder.positions) for i in range(3)]
    pos_acc = add_accessor(add_view(b"".join(struct.pack("<3f", *p) for p in builder.positions), 34962), 5126, len(builder.positions), "VEC3", min=[round(v, 6) for v in mins], max=[round(v, 6) for v in maxs])
    norm_acc = add_accessor(add_view(b"".join(struct.pack("<3f", *n) for n in builder.normals), 34962), 5126, len(builder.normals), "VEC3")
    uv_acc = add_accessor(add_view(b"".join(struct.pack("<2f", *uv) for uv in builder.uvs), 34962), 5126, len(builder.uvs), "VEC2")
    joint_acc = weight_acc = ibm_acc = None
    world = parents = None
    if builder.skin:
        joints, weights = [], []
        for pos in builder.positions:
            j, w = blended_weight_for_position(pos, mins, maxs); joints.append(j); weights.append(w)
        joint_acc = add_accessor(add_view(b"".join(struct.pack("<4H", *j) for j in joints), 34962), 5123, len(joints), "VEC4")
        weight_acc = add_accessor(add_view(b"".join(struct.pack("<4f", *w) for w in weights), 34962), 5126, len(weights), "VEC4")
        world, parents = skeleton_world_layout(mins, maxs)
        ibm_acc = add_accessor(add_view(b"".join(struct.pack("<16f", *inverse_bind_matrix(world[j])) for j in CANONICAL_TRUTH_JOINTS)), 5126, len(CANONICAL_TRUTH_JOINTS), "MAT4")
    materials = [material_record(asset_id, k) for k in builder.material_order]
    primitives = []
    mat_index = {k: i for i, k in enumerate(builder.material_order)}
    for key in builder.material_order:
        idxs = builder.groups[key]
        if not idxs: continue
        idx_acc = add_accessor(add_view(b"".join(struct.pack("<I", int(i)) for i in idxs), 34963), 5125, len(idxs), "SCALAR")
        attrs = {"POSITION": pos_acc, "NORMAL": norm_acc, "TEXCOORD_0": uv_acc}
        if builder.skin:
            attrs["JOINTS_0"] = joint_acc; attrs["WEIGHTS_0"] = weight_acc
        primitives.append({"attributes": attrs, "indices": idx_acc, "material": mat_index[key], "mode": 4})
    images = [{"uri": f"../textures/{textures['base']}", "mimeType": "image/png", "name": f"{asset_id}_base"}, {"uri": f"../textures/{textures['normal']}", "mimeType": "image/png", "name": f"{asset_id}_normal"}, {"uri": f"../textures/{textures['orm']}", "mimeType": "image/png", "name": f"{asset_id}_orm"}]
    nodes = [{"name": asset_id, "mesh": 0, "extras": {"product": "OATHYARD", "asset_id": asset_id, "kind": builder.kind, "source": source_rel, "provenance": "repo_owned_original_procedural_model_candidate", "truth_authoritative": False, "public_demo_ready": False, "release_candidate_ready": False, "owner_visual_acceptance": False, "model_candidate_run": RUN_ID, "animation_ready_repair_task": REPAIR_TASK, "style_anchors": STYLE_ANCHORS, "budget_min_triangles": budget["min"], "budget_target_max_triangles": budget["target_max"], "budget_hard_max_triangles": budget["hard_max"], "triangle_count": builder.triangle_count(), "texture_set": [f"../textures/{textures['base']}", f"../textures/{textures['normal']}", f"../textures/{textures['orm']}"]}}]
    skins = []; scene_nodes = [0]; animations = []
    if builder.skin:
        node_by_joint = {}
        for joint in CANONICAL_TRUTH_JOINTS:
            node_by_joint[joint] = len(nodes)
            nodes.append({"name": joint, "translation": rel_translation(world, parents, joint), "rotation": [0.0, 0.0, 0.0, 1.0], "extras": {"canonical_truth_joint": True, "bind_pose_source": REPAIR_TASK, "truth_authoritative": False}})
        for child, parent in parents.items():
            if parent is not None: nodes[node_by_joint[parent]].setdefault("children", []).append(node_by_joint[child])
        grip_r = len(nodes); nodes.append({"name": "grip_r", "translation": [0.0, -0.05, -0.16], "rotation": [0.0, 0.0, 0.0, 1.0], "extras": {"socket_frame": True, "parent_joint": "wrist_r", "truth_authoritative": False}})
        grip_l = len(nodes); nodes.append({"name": "grip_l", "translation": [0.0, -0.05, -0.16], "rotation": [0.0, 0.0, 0.0, 1.0], "extras": {"socket_frame": True, "parent_joint": "wrist_l", "truth_authoritative": False}})
        nodes[node_by_joint["wrist_r"]].setdefault("children", []).append(grip_r); nodes[node_by_joint["wrist_l"]].setdefault("children", []).append(grip_l)
        nodes[0]["skin"] = 0
        nodes[0]["extras"].update({"canonical_truth_joints": CANONICAL_TRUTH_JOINTS, "grip_frames": ["grip_r", "grip_l"], "animation_clips": REQUIRED_CLIPS, "animation_ready_scope": "synthetic glTF channels for idle/walk/attack motion QA; native/DCC/owner acceptance not claimed"})
        skins.append({"name": f"{asset_id}_canonical_skin", "joints": [node_by_joint[j] for j in CANONICAL_TRUTH_JOINTS], "skeleton": node_by_joint["root"], "inverseBindMatrices": ibm_acc})
        scene_nodes = [0, node_by_joint["root"]]
        def add_clip(name, times, channels):
            time_acc = add_accessor(add_view(pack_rows([(t,) for t in times])), 5126, len(times), "SCALAR", min=[min(times)], max=[max(times)])
            samplers = []; channel_records = []
            for joint, path, values in channels:
                out_type = "VEC4" if path == "rotation" else "VEC3"
                out_acc = add_accessor(add_view(pack_rows(values)), 5126, len(values), out_type)
                sid = len(samplers); samplers.append({"input": time_acc, "interpolation": "LINEAR", "output": out_acc})
                channel_records.append({"sampler": sid, "target": {"node": node_by_joint[joint], "path": path}})
            return {"name": name, "samplers": samplers, "channels": channel_records}
        def rots(axis, degrees): return [quat(axis, deg) for deg in degrees]
        animations = [
            add_clip("idle", [0, .5, 1, 1.5, 2], [("spine_upper", "rotation", rots((0,0,1), [-1.5,.8,1.5,-.8,-1.5])), ("neck_head", "rotation", rots((0,1,0), [0,2.2,0,-2,0])), ("shoulder_r", "rotation", rots((1,0,0), [1.5,-1.2,1.4,-1,1.5])), ("shoulder_l", "rotation", rots((1,0,0), [-1.5,1.1,-1.3,1,-1.5]))]),
            add_clip("walk", [0, .25, .5, .75, 1], [("root", "translation", [(0,0,0),(0,.01,.018),(0,0,.036),(0,.01,.054),(0,0,.072)]), ("hip_r", "rotation", rots((1,0,0), [18,-10,-18,10,18])), ("knee_r", "rotation", rots((1,0,0), [-8,28,-6,18,-8])), ("hip_l", "rotation", rots((1,0,0), [-18,10,18,-10,-18])), ("knee_l", "rotation", rots((1,0,0), [-6,18,-8,28,-6])), ("shoulder_r", "rotation", rots((1,0,0), [-14,8,14,-8,-14])), ("shoulder_l", "rotation", rots((1,0,0), [14,-8,-14,8,14]))]),
            add_clip("attack", [0, .18, .36, .62, .90], [("root", "translation", [(0,0,0),(0,.006,-.018),(0,.004,.045),(0,0,.015),(0,0,0)]), ("spine_upper", "rotation", rots((0,1,0), [0,-12,22,7,0])), ("shoulder_r", "rotation", rots((0,0,1), [-6,-38,46,18,-6])), ("elbow_r", "rotation", rots((1,0,0), [0,-52,-18,8,0])), ("wrist_r", "rotation", rots((0,0,1), [0,-18,34,7,0])), ("hip_l", "rotation", rots((1,0,0), [0,6,-10,-4,0])), ("knee_l", "rotation", rots((1,0,0), [0,-8,18,4,0]))]),
        ]
    document = {"asset": {"version": "2.0", "generator": f"OATHYARD {RUN_ID} deterministic animation-ready model-candidate generator {REPAIR_TASK}", "copyright": "repo-owned original procedural model candidate; no external asset source"}, "scene": 0, "scenes": [{"name": f"{asset_id}_candidate_scene", "nodes": scene_nodes}], "nodes": nodes, "meshes": [{"name": f"{asset_id}_mesh", "primitives": primitives}], "materials": materials, "samplers": [{"magFilter": 9729, "minFilter": 9987, "wrapS": 10497, "wrapT": 10497}], "textures": [{"sampler": 0, "source": i} for i in range(3)], "images": images, "buffers": [{"uri": f"../bin/{asset_id}.bin", "byteLength": len(bin_buf)}], "bufferViews": buffer_views, "accessors": accessors}
    if skins: document["skins"] = skins
    if animations: document["animations"] = animations
    bin_path = BIN_DIR / f"{asset_id}.bin"; gltf_path = GLTF_DIR / f"{asset_id}.gltf"
    bin_path.write_bytes(bytes(bin_buf)); gltf_path.write_text(json.dumps(document, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return {"gltf": gltf_path, "bin": bin_path, "textures": [TEX_DIR / textures["base"], TEX_DIR / textures["normal"], TEX_DIR / textures["orm"]], "vertices": builder.vertex_count(), "triangles": builder.triangle_count(), "materials": len(builder.material_order), "primitives": len(primitives), "bounds_min": mins, "bounds_max": maxs}


def write_source_spec(asset, builder, budget, output):
    path = SRC_ROOT / asset["category"] / f"{asset['id']}.model_source.json"
    record = {"schema": "oathyard.model_candidate_source.v2", "product": "OATHYARD", "kanban_task": RUN_ID, "repair_task": REPAIR_TASK, "asset_id": asset["id"], "kind": asset["kind"], "provenance": "repo_owned_original_procedural_model_candidate", "license_status": "owner_approved_internal_project_use", "source_approved_for_project_use": True, "production_visual_candidate": True, "production_ready_visual": False, "owner_visual_acceptance": False, "public_demo_ready": False, "release_candidate_ready": False, "legal_clearance": False, "trademark_clearance": False, "store_readiness": False, "license_status_evidence": "Unit-082 owner-provided project context approves repo-owned/Rodin/Meshy/model-generated asset use for internal/project use; this does not grant public-demo, release, store, legal, trademark, or owner visual acceptance.", "source_basis": {"content_manifest": "content/oathyard_content.manifest", "category_source_fields": asset["fields"], "loadout": asset["loadout"], "art_direction_brief": "docs/design/ART_DIRECTION_BRIEF.md"}, "art_direction": {"style_target": "grounded stylized realism for dark-fantasy judicial duels", "anchors": STYLE_ANCHORS}, "technical_budget": budget, "generated_metrics": {"vertices": output["vertices"], "triangles": output["triangles"], "materials": output["materials"], "primitive_material_groups": output["primitives"]}, "truth_boundary": {"truth_authoritative": False, "presentation_only": True, "does_not_mutate_gameplay_truth": True}, "rig_contract": {"canonical_truth_joints": CANONICAL_TRUTH_JOINTS if asset["kind"] == "fighter" else [], "grip_frames": ["grip_r", "grip_l"] if asset["kind"] == "fighter" else [], "max_vertex_influences": 4 if asset["kind"] == "fighter" else 0, "motion_proof_status": "synthetic glTF idle/walk/attack channels present" if asset["kind"] == "fighter" else "not_applicable"}, "not_claimed": ["owner visual acceptance", "public demo readiness", "release candidate readiness", "external Khronos validation", "Blender/DCC round trip", "native renderer animation capture"]}
    path.write_text(json.dumps(record, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return path


def accessor_layout(gltf, accessor_id):
    acc = gltf["accessors"][accessor_id]; view = gltf["bufferViews"][acc["bufferView"]]
    fmt_char, size = COMPONENT_INFO[acc["componentType"]]; comps = TYPE_COMPS[acc["type"]]
    return view.get("byteOffset", 0) + acc.get("byteOffset", 0), view.get("byteStride", size * comps), acc["count"], fmt_char, comps


def read_accessor(gltf, buf, accessor_id):
    start, stride, count, fmt_char, comps = accessor_layout(gltf, accessor_id)
    fmt = "<" + fmt_char * comps
    return [struct.unpack_from(fmt, buf, start + i * stride) for i in range(count)]


def resolve_buffer_path(gltf_path, gltf):
    uri = gltf["buffers"][0]["uri"]
    if uri.startswith("data:"):
        raise ValueError("data URI not supported for model candidate audit")
    return (gltf_path.parent / uri).resolve()


def summarize_entry(entry):
    gltf_path = ROOT / entry["runtime_gltf"]
    gltf = json.loads(gltf_path.read_text(encoding="utf-8"))
    buf = resolve_buffer_path(gltf_path, gltf).read_bytes()
    nodes = gltf.get("nodes", []); skins = gltf.get("skins", []); animations = gltf.get("animations", [])
    primitive = gltf["meshes"][0]["primitives"][0]; attrs = primitive.get("attributes", {})
    summary = {"id": entry["id"], "kind": entry["kind"], "gltf": gltf_path.as_posix(), "animations": len(animations), "clips": {}, "skins": len(skins), "joint_count": 0, "joint_hierarchy_edges": sum(len(n.get("children", [])) for n in nodes), "joint_nodes_with_trs_or_matrix": 0, "identity_inverse_bind_matrices": None, "unique_inverse_bind_matrix_count": None, "weight_influence_counts": {}, "one_hot_weight_vertices": 0, "vertex_count": 0, "blended_critical_joint_vertex_counts": {name: 0 for name in CRITICAL_BLEND_JOINTS}}
    for anim in animations:
        nonstatic = 0
        for sampler in anim.get("samplers", []):
            vals = read_accessor(gltf, buf, sampler["output"])
            if len({tuple(round(float(v), 6) for v in row) for row in vals}) > 1: nonstatic += 1
        summary["clips"][anim.get("name", "")] = {"channels": len(anim.get("channels", [])), "samplers": len(anim.get("samplers", [])), "nonstatic_channels": nonstatic}
    if skins:
        joint_node_ids = skins[0].get("joints", [])
        summary["joint_count"] = len(joint_node_ids)
        summary["joint_nodes_with_trs_or_matrix"] = sum(1 for i in joint_node_ids if any(k in nodes[i] for k in ("translation", "rotation", "scale", "matrix")))
        if skins[0].get("inverseBindMatrices") is not None:
            mats = read_accessor(gltf, buf, skins[0]["inverseBindMatrices"])
            rounded = [tuple(round(float(v), 6) for v in m) for m in mats]
            ident = (1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0)
            summary["unique_inverse_bind_matrix_count"] = len(set(rounded))
            summary["identity_inverse_bind_matrices"] = all(m == ident for m in rounded)
    if entry["kind"] == "fighter":
        joints = read_accessor(gltf, buf, attrs["JOINTS_0"]); weights = read_accessor(gltf, buf, attrs["WEIGHTS_0"])
        inf_counts = Counter(); blend_counts = Counter(); one_hot = 0
        for js, ws in zip(joints, weights):
            active = [(int(j), float(w)) for j, w in zip(js, ws) if abs(float(w)) > 1e-6]
            inf_counts[len(active)] += 1
            if len(active) == 1 and active[0][1] >= 0.999: one_hot += 1
            if len(active) >= 2:
                for j, _w in active:
                    if 0 <= j < len(CANONICAL_TRUTH_JOINTS): blend_counts[CANONICAL_TRUTH_JOINTS[j]] += 1
        summary["weight_influence_counts"] = dict(sorted(inf_counts.items()))
        summary["one_hot_weight_vertices"] = one_hot
        summary["vertex_count"] = len(weights)
        summary["blended_critical_joint_vertex_counts"] = {name: blend_counts.get(name, 0) for name in CRITICAL_BLEND_JOINTS}
    return summary


def fill_tri(pix, p0, p1, p2, color):
    h, w = len(pix), len(pix[0])
    min_x, max_x = max(0, int(math.floor(min(p0[0], p1[0], p2[0])))), min(w - 1, int(math.ceil(max(p0[0], p1[0], p2[0]))))
    min_y, max_y = max(0, int(math.floor(min(p0[1], p1[1], p2[1])))), min(h - 1, int(math.ceil(max(p0[1], p1[1], p2[1]))))
    denom = (p1[1] - p2[1]) * (p0[0] - p2[0]) + (p2[0] - p1[0]) * (p0[1] - p2[1])
    if abs(denom) < 1e-6: return
    for y in range(min_y, max_y + 1):
        for x in range(min_x, max_x + 1):
            w0 = ((p1[1] - p2[1]) * (x - p2[0]) + (p2[0] - p1[0]) * (y - p2[1])) / denom
            w1 = ((p2[1] - p0[1]) * (x - p2[0]) + (p0[0] - p2[0]) * (y - p2[1])) / denom
            w2 = 1.0 - w0 - w1
            if w0 >= -0.01 and w1 >= -0.01 and w2 >= -0.01: pix[y][x] = color


def validate_package(manifest):
    failures = []
    package_checks = {
        "runtime_package_dir": {"path": PKG_ROOT.as_posix(), "exists": PKG_ROOT.is_dir()},
        "runtime_manifest": {"path": MANIFEST_PATH.as_posix(), "exists": MANIFEST_PATH.is_file()},
        "artifact_dir": {"path": ART_ROOT.as_posix(), "exists": ART_ROOT.is_dir()},
        "source_manifest": {"path": SOURCE_MANIFEST_PATH.as_posix(), "exists": SOURCE_MANIFEST_PATH.is_file()},
    }
    for name, check in package_checks.items():
        if not check["exists"]: failures.append(f"missing {name}: {check['path']}")
    for key in ["public_demo_ready", "release_candidate_ready", "owner_visual_acceptance", "truth_authoritative"]:
        if manifest.get(key) is not False: failures.append(f"manifest {key} must remain false")
    summaries = []
    for entry in manifest["entries"]:
        gltf_path = ROOT / entry["runtime_gltf"]; bin_path = ROOT / entry["runtime_bin"]
        if not gltf_path.is_file(): failures.append(f"{entry['id']} missing glTF {gltf_path}"); continue
        if not bin_path.is_file(): failures.append(f"{entry['id']} missing bin {bin_path}"); continue
        if entry.get("public_demo_ready") is not False or entry.get("release_candidate_ready") is not False or entry.get("owner_visual_acceptance") is not False or entry.get("truth_authoritative") is not False: failures.append(f"{entry['id']} readiness/truth flags must remain false")
        summary = summarize_entry(entry); summaries.append(summary)
        if entry["kind"] == "fighter":
            for clip in REQUIRED_CLIPS:
                m = summary["clips"].get(clip)
                if not m: failures.append(f"{entry['id']} missing {clip} animation clip")
                elif m["channels"] <= 0 or m["samplers"] <= 0 or m["nonstatic_channels"] <= 0: failures.append(f"{entry['id']} {clip} animation lacks usable nonstatic channels")
            if summary["joint_count"] != len(CANONICAL_TRUTH_JOINTS): failures.append(f"{entry['id']} joint count {summary['joint_count']} != {len(CANONICAL_TRUTH_JOINTS)}")
            if summary["joint_hierarchy_edges"] < len(CANONICAL_TRUTH_JOINTS) - 1: failures.append(f"{entry['id']} joint hierarchy edges too low: {summary['joint_hierarchy_edges']}")
            if summary["joint_nodes_with_trs_or_matrix"] < len(CANONICAL_TRUTH_JOINTS): failures.append(f"{entry['id']} bind-pose TRS missing")
            if summary["identity_inverse_bind_matrices"] is not False or summary["unique_inverse_bind_matrix_count"] <= 1: failures.append(f"{entry['id']} inverse bind matrices are placeholder/identity")
            if summary["one_hot_weight_vertices"] == summary["vertex_count"]: failures.append(f"{entry['id']} weights remain 100% one-hot")
            for joint, count in summary["blended_critical_joint_vertex_counts"].items():
                if count <= 0: failures.append(f"{entry['id']} no blended vertices involving {joint}")
    payload = {"schema": "oathyard.model_candidate_animation_ready_validation.v1", "product": "OATHYARD", "kanban_task": REPAIR_TASK, "model_candidate_run": RUN_ID, "passed": not failures, "package_checks": package_checks, "required_clips": REQUIRED_CLIPS, "critical_blend_joints": CRITICAL_BLEND_JOINTS, "entries": summaries, "public_demo_ready": False, "release_candidate_ready": False, "owner_visual_acceptance": False, "truth_authoritative": False, "failures": failures}
    VALIDATION_JSON.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    lines = ["# OATHYARD t_73291be5 animation-ready validation", "", f"Status: {'PASSED' if payload['passed'] else 'FAILED'}", f"- Repair task: `{REPAIR_TASK}`", f"- Runtime package: `{PKG_ROOT}`", f"- Runtime manifest: `{MANIFEST_PATH}`", f"- Artifact dir: `{ART_ROOT}`", "- Public demo ready: `false`", "- Release candidate ready: `false`", "- Owner visual acceptance: `false`", "- Truth authoritative: `false`", "", "## Fighter rig/motion checks", ""]
    for s in summaries:
        if s["kind"] != "fighter": continue
        missing = [j for j, c in s["blended_critical_joint_vertex_counts"].items() if c <= 0]
        lines.append(f"- `{s['id']}`: animations `{s['animations']}`, clips `{','.join(sorted(s['clips']))}`, hierarchy_edges `{s['joint_hierarchy_edges']}`, joint_trs `{s['joint_nodes_with_trs_or_matrix']}`, unique_ibm `{s['unique_inverse_bind_matrix_count']}`, one_hot `{s['one_hot_weight_vertices']}/{s['vertex_count']}`, weight_influence_counts `{s['weight_influence_counts']}`")
        lines.append(f"  - critical blend joints missing: `{missing}`")
    if failures:
        lines.extend(["", "## Failures"]); lines.extend(f"- {f}" for f in failures)
    lines.extend(["", "## Scope boundary", "", "This validates glTF structural animation readiness for the generated candidate package. It does not claim native renderer capture, Blender/DCC round trip, owner visual acceptance, public-demo readiness, or release-candidate readiness."])
    VALIDATION_REPORT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return payload


def write_motion_evidence(validation):
    fighters = [e for e in validation["entries"] if e["kind"] == "fighter"]
    evidence = {"schema": "oathyard.fighter_motion_evidence.v1", "product": "OATHYARD", "kanban_task": REPAIR_TASK, "model_candidate_run": RUN_ID, "source_validation": VALIDATION_JSON.as_posix(), "clip_names_required": REQUIRED_CLIPS, "fighters": fighters}
    MOTION_JSON.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    lines = ["# OATHYARD fighter motion evidence", "", f"Source validation: `{VALIDATION_JSON}`", "", "| fighter | idle | walk | attack | hierarchy | TRS joints | unique IBM | one-hot weights | blended critical joints |", "|---|---:|---:|---:|---:|---:|---:|---:|---:|"]
    for m in fighters:
        clips = m["clips"]; blended_ok = sum(1 for c in m["blended_critical_joint_vertex_counts"].values() if c > 0)
        lines.append(f"| `{m['id']}` | {clips.get('idle', {}).get('channels', 0)} | {clips.get('walk', {}).get('channels', 0)} | {clips.get('attack', {}).get('channels', 0)} | {m['joint_hierarchy_edges']} | {m['joint_nodes_with_trs_or_matrix']} | {m['unique_inverse_bind_matrix_count']} | {m['one_hot_weight_vertices']}/{m['vertex_count']} | {blended_ok}/{len(CRITICAL_BLEND_JOINTS)} |")
    lines.append("\nReadiness flags remain false. This is structural glTF motion evidence, not owner/native-renderer acceptance.")
    MOTION_REPORT.write_text("\n".join(lines) + "\n", encoding="utf-8")



def write_handoff(validation):
    lines = ["# OATHYARD t_73291be5 animation-ready repair handoff", "", f"Repair task: `{REPAIR_TASK}`", "", "## Runtime package", "", f"- Candidate manifest: `{MANIFEST_PATH}`", f"- Runtime package dir: `{PKG_ROOT}`", f"- Artifact dir: `{ART_ROOT}`", f"- Animation-ready validation JSON: `{VALIDATION_JSON}`", f"- Animation-ready validation report: `{VALIDATION_REPORT}`", f"- Motion evidence JSON: `{MOTION_JSON}`", f"- Motion evidence report: `{MOTION_REPORT}`", "", "## What changed from the QA blocker", "", "- Regenerated/published the missing repo-owned runtime package and artifact directory at the expected OATHYARD paths.", "- Added joint parent-child hierarchy and bind-pose TRS to all six fighter glTFs.", "- Added per-joint inverse bind matrices instead of identity placeholders.", "- Added <=4 influence blended skin weights spanning shoulders, elbows, hips, knees, and ankles.", "- Added glTF animation clips named `idle`, `walk`, and `attack` with non-static sampled channels for every fighter.", "", "## Explicit non-claims", "", "Public demo readiness, release candidate readiness, owner visual acceptance, truth authority, external Khronos validation, Blender/DCC round-trip, and native renderer animation capture remain false/not claimed.", "", f"Validation status: `{'PASSED' if validation['passed'] else 'FAILED'}`"]
    if validation.get("failures"):
        lines.extend(["", "## Failures"]); lines.extend(f"- {f}" for f in validation["failures"])
    HANDOFF_PATH.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main():
    if PKG_ROOT.exists(): shutil.rmtree(PKG_ROOT)
    if ART_ROOT.exists(): shutil.rmtree(ART_ROOT)
    ensure_dirs()
    assets = gather_assets(); builders = []; manifest_entries = []; source_manifest_entries = []
    for asset in assets:
        b = build_asset(asset); builders.append(b); bk = budget_kind(asset["id"], asset["kind"]); source_rel_tmp = f"assets/source/model_candidates/{RUN_ID}/{asset['category']}/{asset['id']}.model_source.json"
        output = write_gltf(b, source_rel_tmp, BUDGETS[bk]); source_path = write_source_spec(asset, b, BUDGETS[bk], output)
        entry = {"id": asset["id"], "kind": asset["kind"], "category": asset["category"], "budget_kind": bk, "source": source_path.relative_to(ROOT).as_posix(), "source_abs": source_path.as_posix(), "runtime_gltf": output["gltf"].relative_to(ROOT).as_posix(), "runtime_gltf_abs": output["gltf"].as_posix(), "runtime_bin": output["bin"].relative_to(ROOT).as_posix(), "textures": [p.relative_to(ROOT).as_posix() for p in output["textures"]], "triangles": output["triangles"], "vertices": output["vertices"], "materials": output["materials"], "primitives": output["primitives"], "bounds_min": [round(v, 6) for v in output["bounds_min"]], "bounds_max": [round(v, 6) for v in output["bounds_max"]], "sha256": {"source": sha256_file(source_path), "gltf": sha256_file(output["gltf"]), "bin": sha256_file(output["bin"]), "textures": {p.name: sha256_file(p) for p in output["textures"]}}, "truth_authoritative": False, "public_demo_ready": False, "release_candidate_ready": False, "owner_visual_acceptance": False}
        if asset["kind"] == "fighter": entry["animation_ready_repair"] = {"kanban_task": REPAIR_TASK, "required_clips": REQUIRED_CLIPS, "motion_proof_status": "glTF animation channels present; native/DCC/owner acceptance not claimed"}
        manifest_entries.append(entry); source_manifest_entries.append({k: entry[k] for k in ["id", "kind", "source", "triangles", "sha256"]})
    manifest = {"schema": "oathyard.model_candidate_manifest.v2", "product": "OATHYARD", "kanban_task": RUN_ID, "repair_task": REPAIR_TASK, "source_basis": ["docs/design/ART_DIRECTION_BRIEF.md", "content/oathyard_content.manifest", "assets/source/oysrc/traditions.oysrc", "assets/source/oysrc/weapons.oysrc", "assets/source/oysrc/armor.oysrc", "assets/source/oysrc/arenas.oysrc"], "provenance": "repo_owned_original_procedural_model_candidate", "truth_authoritative": False, "public_demo_ready": False, "release_candidate_ready": False, "owner_visual_acceptance": False, "external_khronos_validation_claimed": False, "animation_ready_repair": {"kanban_task": REPAIR_TASK, "scope": "fighter skeleton hierarchy, bind pose, blended skin weights, and idle/walk/attack glTF animation clips", "not_claimed": ["owner visual acceptance", "public demo readiness", "release candidate readiness", "external Khronos validation", "Blender/DCC round trip", "native renderer animation capture"]}, "entries": manifest_entries}
    MANIFEST_PATH.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    source_manifest = {"schema": "oathyard.model_candidate_source_manifest.v2", "product": "OATHYARD", "kanban_task": RUN_ID, "repair_task": REPAIR_TASK, "entries": source_manifest_entries, "not_claimed": ["owner visual acceptance", "public demo readiness", "release candidate readiness", "external Khronos validation", "Blender/DCC round trip", "native renderer animation capture"]}
    SOURCE_MANIFEST_PATH.write_text(json.dumps(source_manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    validation = validate_package(manifest)
    write_motion_evidence(validation); write_handoff(validation)
    print(json.dumps({"passed": validation["passed"], "runtime_package_dir": PKG_ROOT.as_posix(), "artifact_dir": ART_ROOT.as_posix(), "validation_json": VALIDATION_JSON.as_posix(), "motion_evidence_json": MOTION_JSON.as_posix(), "failures": validation["failures"]}, indent=2, sort_keys=True))
    return 0 if validation["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
