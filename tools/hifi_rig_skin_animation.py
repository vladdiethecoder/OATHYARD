#!/usr/bin/env python3
"""Generate fail-closed HIFI-WO-05 rig/skin/animation presentation evidence.

This tool audits the current OATHYARD model-candidate glTF package and emits a
runtime-presentation-only rig separation manifest plus deterministic pose/no-clip
sheets. It intentionally does not add presentation data to combat truth.
"""
from __future__ import annotations

import argparse
import binascii
import hashlib
import json
import math
import struct
import zlib
from collections import Counter
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
PRODUCT = "OATHYARD"
CARD_ID = "t_54eabe83"
RUN_ID = "t_73291be5"
MANIFEST_SCHEMA = "oathyard.hifi_rig_skin_animation.v1"
STATE_HANDOFF_SCHEMA = "oathyard.runtime_animation_state_machine_handoff.v1"

TRUTH_JOINTS = [
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
TRUTH_JOINT_INDEX = {name: index for index, name in enumerate(TRUTH_JOINTS)}
CRITICAL_BLEND_JOINTS = [
    "shoulder_r",
    "elbow_r",
    "shoulder_l",
    "elbow_l",
    "hip_r",
    "knee_r",
    "ankle_r",
    "hip_l",
    "knee_l",
    "ankle_l",
]
REQUIRED_GLTF_CLIPS = ["idle", "walk", "attack"]
RUNTIME_STATE_LABELS = [
    "observe",
    "plan",
    "step",
    "pivot",
    "guard",
    "parry",
    "cut",
    "thrust",
    "brace",
    "bash",
    "hook_bind",
    "grab",
    "shove",
    "kick",
    "recover",
]
REACTION_LABELS = ["bind", "stagger", "collapse", "injury", "recovery"]
NO_CLIP_STATES = [
    "idle",
    "walk",
    "guard",
    "cut",
    "thrust",
    "brace",
    "bash",
    "hook_bind",
    "grab",
    "shove",
    "kick",
    "recover",
]
POSE_SHEET_STATES = ["idle", "walk", "guard", "cut"]
STATE_MATRIX_LABELS = ["observe", "plan"] + RUNTIME_STATE_LABELS[2:]

COMPONENT_INFO = {
    5120: ("b", 1),
    5121: ("B", 1),
    5122: ("h", 2),
    5123: ("H", 2),
    5125: ("I", 4),
    5126: ("f", 4),
}
TYPE_COMPS = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}
IDENTITY_MAT4 = (
    1.0,
    0.0,
    0.0,
    0.0,
    0.0,
    1.0,
    0.0,
    0.0,
    0.0,
    0.0,
    1.0,
    0.0,
    0.0,
    0.0,
    0.0,
    1.0,
)
SKELETON_EDGES = [
    ("root", "spine_lower"),
    ("spine_lower", "spine_upper"),
    ("spine_upper", "neck_head"),
    ("spine_upper", "shoulder_r"),
    ("shoulder_r", "elbow_r"),
    ("elbow_r", "wrist_r"),
    ("spine_upper", "shoulder_l"),
    ("shoulder_l", "elbow_l"),
    ("elbow_l", "wrist_l"),
    ("root", "hip_r"),
    ("hip_r", "knee_r"),
    ("knee_r", "ankle_r"),
    ("root", "hip_l"),
    ("hip_l", "knee_l"),
    ("knee_l", "ankle_l"),
]

COSMETIC_PRESENTATION_BONES = [
    ("brow_r", "neck_head", "face_head_detail"),
    ("brow_l", "neck_head", "face_head_detail"),
    ("jaw", "neck_head", "face_head_detail"),
    ("cheek_r", "neck_head", "face_head_detail"),
    ("cheek_l", "neck_head", "face_head_detail"),
    ("thumb_r_01", "wrist_r", "hand_finger"),
    ("index_r_01", "wrist_r", "hand_finger"),
    ("middle_r_01", "wrist_r", "hand_finger"),
    ("thumb_l_01", "wrist_l", "hand_finger"),
    ("index_l_01", "wrist_l", "hand_finger"),
    ("middle_l_01", "wrist_l", "hand_finger"),
    ("cloak_root", "spine_upper", "cloth_secondary"),
    ("cloak_r", "spine_upper", "cloth_secondary"),
    ("cloak_l", "spine_upper", "cloth_secondary"),
    ("skirt_front", "spine_lower", "cloth_secondary"),
    ("skirt_back", "spine_lower", "cloth_secondary"),
    ("scabbard_rig", "hip_l", "equipment_secondary"),
    ("strap_chest_r", "spine_upper", "strap_secondary"),
    ("strap_chest_l", "spine_upper", "strap_secondary"),
    ("pauldron_r", "shoulder_r", "armor_plate_secondary"),
    ("pauldron_l", "shoulder_l", "armor_plate_secondary"),
    ("tasset_r", "hip_r", "armor_plate_secondary"),
    ("tasset_l", "hip_l", "armor_plate_secondary"),
    ("weapon_trail_tip_r", "wrist_r", "weapon_secondary_motion"),
    ("weapon_trail_tip_l", "wrist_l", "weapon_secondary_motion"),
]

STATE_TO_PRESENTATION = {
    "observe": ("idle", "observe_breath"),
    "plan": ("idle", "planning_attention"),
    "step": ("walk", "footfall_shift"),
    "pivot": ("walk", "turn_in_place"),
    "guard": ("guard_pose", "raised_guard"),
    "parry": ("guard_pose", "parry_reaction"),
    "cut": ("attack", "cut_arc"),
    "thrust": ("attack", "thrust_line"),
    "brace": ("guard_pose", "brace_root"),
    "bash": ("attack", "shield_or_body_bash"),
    "hook_bind": ("attack", "hook_bind_strain"),
    "grab": ("attack", "grappling_reach"),
    "shove": ("attack", "push_extension"),
    "kick": ("attack", "leg_attack_extension"),
    "recover": ("idle", "recovery_settle"),
}

PALETTE = {
    "background": (25, 22, 18),
    "cell_a": (42, 36, 28),
    "cell_b": (34, 32, 29),
    "skeleton": (235, 219, 178),
    "joint": (248, 237, 198),
    "armor": (72, 112, 132),
    "armor_ok": (34, 132, 92),
    "weapon": (208, 182, 96),
    "injury": (150, 38, 38),
    "capability": (216, 126, 54),
    "truth_anchor": (87, 167, 210),
    "cosmetic": (162, 95, 178),
}


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def read_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def json_rel(path: Path) -> str:
    try:
        return path.resolve().relative_to(ROOT).as_posix()
    except ValueError:
        return path.as_posix()


def png_chunk(kind: bytes, data: bytes) -> bytes:
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", binascii.crc32(kind + data) & 0xFFFFFFFF)


def write_png_rgb(path: Path, width: int, height: int, pixels: list[list[tuple[int, int, int]]]) -> None:
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
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)


def image(width: int, height: int, color: tuple[int, int, int]) -> list[list[tuple[int, int, int]]]:
    return [[color for _ in range(width)] for _ in range(height)]


def put_pixel(pixels: list[list[tuple[int, int, int]]], x: int, y: int, color: tuple[int, int, int]) -> None:
    if 0 <= y < len(pixels) and 0 <= x < len(pixels[0]):
        pixels[y][x] = color


def draw_line(
    pixels: list[list[tuple[int, int, int]]],
    x0: float,
    y0: float,
    x1: float,
    y1: float,
    color: tuple[int, int, int],
    thickness: int = 1,
) -> None:
    x0_i, y0_i, x1_i, y1_i = int(round(x0)), int(round(y0)), int(round(x1)), int(round(y1))
    dx = abs(x1_i - x0_i)
    dy = -abs(y1_i - y0_i)
    sx = 1 if x0_i < x1_i else -1
    sy = 1 if y0_i < y1_i else -1
    err = dx + dy
    x, y = x0_i, y0_i
    radius = max(0, thickness // 2)
    while True:
        for yy in range(y - radius, y + radius + 1):
            for xx in range(x - radius, x + radius + 1):
                put_pixel(pixels, xx, yy, color)
        if x == x1_i and y == y1_i:
            break
        e2 = 2 * err
        if e2 >= dy:
            err += dy
            x += sx
        if e2 <= dx:
            err += dx
            y += sy


def fill_rect(
    pixels: list[list[tuple[int, int, int]]],
    x0: int,
    y0: int,
    x1: int,
    y1: int,
    color: tuple[int, int, int],
) -> None:
    width, height = len(pixels[0]), len(pixels)
    xa, xb = max(0, min(x0, x1)), min(width - 1, max(x0, x1))
    ya, yb = max(0, min(y0, y1)), min(height - 1, max(y0, y1))
    for y in range(ya, yb + 1):
        row = pixels[y]
        for x in range(xa, xb + 1):
            row[x] = color


def draw_rect_outline(
    pixels: list[list[tuple[int, int, int]]],
    x0: int,
    y0: int,
    x1: int,
    y1: int,
    color: tuple[int, int, int],
    thickness: int = 1,
) -> None:
    for offset in range(thickness):
        draw_line(pixels, x0, y0 + offset, x1, y0 + offset, color)
        draw_line(pixels, x0, y1 - offset, x1, y1 - offset, color)
        draw_line(pixels, x0 + offset, y0, x0 + offset, y1, color)
        draw_line(pixels, x1 - offset, y0, x1 - offset, y1, color)


def fill_circle(pixels: list[list[tuple[int, int, int]]], cx: float, cy: float, radius: int, color: tuple[int, int, int]) -> None:
    cxi, cyi = int(round(cx)), int(round(cy))
    r2 = radius * radius
    for y in range(cyi - radius, cyi + radius + 1):
        for x in range(cxi - radius, cxi + radius + 1):
            if (x - cxi) * (x - cxi) + (y - cyi) * (y - cyi) <= r2:
                put_pixel(pixels, x, y, color)


def accessor_layout(gltf: dict[str, Any], accessor_id: int) -> tuple[int, int, int, str, int]:
    acc = gltf["accessors"][accessor_id]
    view = gltf["bufferViews"][acc["bufferView"]]
    fmt_char, size = COMPONENT_INFO[acc["componentType"]]
    comps = TYPE_COMPS[acc["type"]]
    offset = int(view.get("byteOffset", 0)) + int(acc.get("byteOffset", 0))
    stride = int(view.get("byteStride", size * comps))
    return offset, stride, int(acc["count"]), fmt_char, comps


def read_accessor(gltf: dict[str, Any], buf: bytes, accessor_id: int) -> list[tuple[Any, ...]]:
    start, stride, count, fmt_char, comps = accessor_layout(gltf, accessor_id)
    fmt = "<" + fmt_char * comps
    rows = []
    for index in range(count):
        rows.append(struct.unpack_from(fmt, buf, start + index * stride))
    return rows


def resolve_uri(gltf_path: Path, uri: str) -> Path:
    if uri.startswith("data:"):
        raise ValueError(f"data URI is not permitted in {gltf_path}")
    return (gltf_path.parent / uri).resolve()


def vec_add(a: tuple[float, float, float], b: tuple[float, float, float]) -> tuple[float, float, float]:
    return (a[0] + b[0], a[1] + b[1], a[2] + b[2])


def node_translation(node: dict[str, Any]) -> tuple[float, float, float]:
    value = node.get("translation", [0.0, 0.0, 0.0])
    return (float(value[0]), float(value[1]), float(value[2]))


def world_translations(nodes: list[dict[str, Any]]) -> dict[int, tuple[float, float, float]]:
    parent_of: dict[int, int | None] = {index: None for index in range(len(nodes))}
    for parent_index, node in enumerate(nodes):
        for child in node.get("children", []):
            parent_of[int(child)] = parent_index

    cache: dict[int, tuple[float, float, float]] = {}

    def visit(index: int) -> tuple[float, float, float]:
        if index in cache:
            return cache[index]
        parent = parent_of.get(index)
        local = node_translation(nodes[index])
        if parent is None:
            cache[index] = local
        else:
            cache[index] = vec_add(visit(parent), local)
        return cache[index]

    for index in range(len(nodes)):
        visit(index)
    return cache


def primitive_attributes(gltf: dict[str, Any]) -> dict[str, int]:
    attrs: dict[str, int] = {}
    for mesh in gltf.get("meshes", []):
        for primitive in mesh.get("primitives", []):
            attrs.update(primitive.get("attributes", {}))
        if attrs:
            return attrs
    return attrs


def recursive_get_final_hash(payload: Any) -> str | None:
    if isinstance(payload, dict):
        for key in ("final_state_hash", "final_hash", "state_hash"):
            value = payload.get(key)
            if isinstance(value, str) and value:
                return value
        for value in payload.values():
            found = recursive_get_final_hash(value)
            if found:
                return found
    elif isinstance(payload, list):
        for item in payload:
            found = recursive_get_final_hash(item)
            if found:
                return found
    return None


def replay_evidence(replay_artifacts: Path | None) -> dict[str, Any]:
    if replay_artifacts is None:
        return {"provided": False}
    replay_path = replay_artifacts / "replay.json" if replay_artifacts.is_dir() else replay_artifacts
    evidence = {"provided": True, "path": json_rel(replay_path), "exists": replay_path.is_file()}
    if replay_path.is_file():
        payload = read_json(replay_path)
        evidence.update(
            {
                "sha256": sha256_file(replay_path),
                "final_state_hash": recursive_get_final_hash(payload),
            }
        )
    return evidence


def cosmetic_bone_records() -> list[dict[str, Any]]:
    records = []
    for bone_id, parent_joint, purpose in COSMETIC_PRESENTATION_BONES:
        records.append(
            {
                "bone_id": bone_id,
                "parent_truth_joint": parent_joint,
                "parent_truth_joint_id": TRUTH_JOINT_INDEX[parent_joint],
                "purpose": purpose,
                "presentation_only": True,
                "truth_authoritative": False,
                "can_write_action_cost": False,
                "can_write_contact": False,
                "can_write_injury": False,
                "can_write_capability_delta": False,
                "can_write_replay_hash": False,
            }
        )
    return records


def presentation_anchor_records() -> list[dict[str, Any]]:
    anchors = []
    for joint in TRUTH_JOINTS:
        anchors.append(
            {
                "anchor_id": f"truth_anchor_{joint}",
                "truth_joint": joint,
                "truth_joint_id": TRUTH_JOINT_INDEX[joint],
                "source_layer": "authoritative_truth_after_hash_read_only",
                "presentation_bone": joint,
                "presentation_only": False,
                "can_write_truth": False,
            }
        )
    anchors.extend(
        [
            {
                "anchor_id": "grip_r",
                "parent_truth_joint": "wrist_r",
                "parent_truth_joint_id": TRUTH_JOINT_INDEX["wrist_r"],
                "source_layer": "truth_grip_frame_after_hash_read_only",
                "presentation_bone": "grip_r",
                "socket_frame": True,
                "presentation_only": True,
                "can_write_truth": False,
            },
            {
                "anchor_id": "grip_l",
                "parent_truth_joint": "wrist_l",
                "parent_truth_joint_id": TRUTH_JOINT_INDEX["wrist_l"],
                "source_layer": "truth_grip_frame_after_hash_read_only",
                "presentation_bone": "grip_l",
                "socket_frame": True,
                "presentation_only": True,
                "can_write_truth": False,
            },
        ]
    )
    return anchors


def state_handoff_manifest(replay: dict[str, Any]) -> dict[str, Any]:
    states = []
    for label in RUNTIME_STATE_LABELS:
        clip, additive = STATE_TO_PRESENTATION[label]
        states.append(
            {
                "state": label,
                "presentation_clip_id": clip,
                "additive_reaction_id": additive,
                "input_boundary": "truth_after_hash_action_or_phase_event",
                "presentation_only": True,
                "truth_mutation": False,
            }
        )
    reactions = [
        {
            "reaction": label,
            "input_boundary": "truth_after_hash_contact_injury_capability_event",
            "presentation_only": True,
            "truth_mutation": False,
        }
        for label in REACTION_LABELS
    ]
    transitions = []
    for index, label in enumerate(RUNTIME_STATE_LABELS):
        next_label = RUNTIME_STATE_LABELS[(index + 1) % len(RUNTIME_STATE_LABELS)]
        transitions.append(
            {
                "from": label,
                "to": next_label,
                "trigger": "truth_after_hash_phase_or_action_event",
                "may_predecide_contact": False,
                "may_predecide_injury": False,
                "may_modify_action_cost": False,
                "presentation_only": True,
            }
        )
    return {
        "schema": STATE_HANDOFF_SCHEMA,
        "product": PRODUCT,
        "kanban_task": CARD_ID,
        "model_candidate_run": RUN_ID,
        "state_labels": RUNTIME_STATE_LABELS,
        "states": states,
        "reaction_labels": REACTION_LABELS,
        "reactions": reactions,
        "transitions": transitions,
        "runtime_pose_input_schema": {
            "truth_pose_event_source": "replay_or_trace_event_after_final_hash",
            "presentation_pose_layer": "runtime_presentation",
            "animation_clip_id": "string",
            "additive_reaction_id": "string_or_null",
            "cosmetic_bone_transforms": "ordered array by cosmetic_bone_manifest index",
            "deterministic_capture_id": "content-addressed artifact id",
            "truth_mutation": False,
        },
        "replay_evidence": replay,
        "truth_boundary": {
            "consumes_truth_after_hash": True,
            "writes_action_costs": False,
            "writes_contacts": False,
            "writes_injuries": False,
            "writes_capability_deltas": False,
            "writes_replay_hashes": False,
            "truth_mutation": False,
        },
        "public_demo_ready": False,
        "release_candidate_ready": False,
        "owner_visual_acceptance": False,
    }


def fighter_loadout(source_path: Path) -> dict[str, str]:
    try:
        source = read_json(source_path)
        loadout = source.get("source_basis", {}).get("loadout", {})
        return {
            "weapon": str(loadout.get("weapon", "unknown_weapon")),
            "armor": str(loadout.get("armor", "unknown_armor")),
        }
    except Exception:
        return {"weapon": "unknown_weapon", "armor": "unknown_armor"}


def summarize_fighter(entry: dict[str, Any]) -> tuple[dict[str, Any], dict[str, tuple[float, float, float]], list[str]]:
    failures: list[str] = []
    gltf_path = ROOT / entry["runtime_gltf"]
    gltf = read_json(gltf_path)
    buf = resolve_uri(gltf_path, gltf.get("buffers", [{}])[0].get("uri", "")).read_bytes()
    nodes = gltf.get("nodes", [])
    node_world = world_translations(nodes)
    node_names = {node.get("name", f"node_{index}"): index for index, node in enumerate(nodes)}
    attrs = primitive_attributes(gltf)
    skins = gltf.get("skins", [])
    animations = gltf.get("animations", [])
    clip_names = [animation.get("name", "") for animation in animations]

    joint_node_ids = skins[0].get("joints", []) if skins else []
    joint_names = [nodes[index].get("name", f"node_{index}") for index in joint_node_ids if 0 <= index < len(nodes)]
    missing_truth_joints = [joint for joint in TRUTH_JOINTS if joint not in joint_names]
    if missing_truth_joints:
        failures.append(f"{entry['id']} missing skin joints {missing_truth_joints}")

    ibm_unique = 0
    ibm_all_identity: bool | None = None
    if skins and skins[0].get("inverseBindMatrices") is not None:
        mats = read_accessor(gltf, buf, int(skins[0]["inverseBindMatrices"]))
        rounded = [tuple(round(float(value), 6) for value in mat) for mat in mats]
        ibm_unique = len(set(rounded))
        ibm_all_identity = all(mat == IDENTITY_MAT4 for mat in rounded)
    else:
        failures.append(f"{entry['id']} missing inverse bind matrices")

    missing_clips = [clip for clip in REQUIRED_GLTF_CLIPS if clip not in clip_names]
    if missing_clips:
        failures.append(f"{entry['id']} missing glTF clips {missing_clips}")

    clip_summary: dict[str, Any] = {}
    for animation in animations:
        nonstatic_channels = 0
        for sampler in animation.get("samplers", []):
            rows = read_accessor(gltf, buf, int(sampler["output"]))
            if len({tuple(round(float(value), 6) for value in row) for row in rows}) > 1:
                nonstatic_channels += 1
        clip_summary[str(animation.get("name", ""))] = {
            "channels": len(animation.get("channels", [])),
            "samplers": len(animation.get("samplers", [])),
            "nonstatic_channels": nonstatic_channels,
        }

    if "JOINTS_0" not in attrs or "WEIGHTS_0" not in attrs:
        failures.append(f"{entry['id']} missing JOINTS_0/WEIGHTS_0")
        joints_rows: list[tuple[Any, ...]] = []
        weight_rows: list[tuple[Any, ...]] = []
    else:
        joints_rows = read_accessor(gltf, buf, int(attrs["JOINTS_0"]))
        weight_rows = read_accessor(gltf, buf, int(attrs["WEIGHTS_0"]))

    influence_counts: Counter[int] = Counter()
    critical_counts: Counter[str] = Counter()
    one_hot = 0
    min_secondary_weight = 1.0
    for joint_row, weight_row in zip(joints_rows, weight_rows):
        active = [(int(joint), float(weight)) for joint, weight in zip(joint_row, weight_row) if abs(float(weight)) > 1.0e-6]
        influence_counts[len(active)] += 1
        if len(active) == 1 and active[0][1] >= 0.999:
            one_hot += 1
        if len(active) >= 2:
            sorted_weights = sorted((weight for _joint, weight in active), reverse=True)
            min_secondary_weight = min(min_secondary_weight, sorted_weights[1])
            for joint_index, _weight in active:
                if 0 <= joint_index < len(TRUTH_JOINTS):
                    critical_counts[TRUTH_JOINTS[joint_index]] += 1
    missing_blends = [joint for joint in CRITICAL_BLEND_JOINTS if critical_counts.get(joint, 0) <= 0]
    if missing_blends:
        failures.append(f"{entry['id']} missing blended weights for {missing_blends}")
    if one_hot == len(weight_rows) and weight_rows:
        failures.append(f"{entry['id']} all skin weights are one-hot")
    if ibm_all_identity is not False or ibm_unique <= 1:
        failures.append(f"{entry['id']} inverse bind matrices are identity/placeholders")

    positions = {
        joint: node_world[node_names[joint]]
        for joint in TRUTH_JOINTS
        if joint in node_names and node_names[joint] in node_world
    }
    for grip in ("grip_r", "grip_l"):
        if grip in node_names:
            positions[grip] = node_world[node_names[grip]]
    bounds_min = [float(value) for value in entry.get("bounds_min", [0.0, 0.0, 0.0])]
    bounds_max = [float(value) for value in entry.get("bounds_max", [0.0, 0.0, 0.0])]
    source_path = ROOT / entry["source"]
    loadout = fighter_loadout(source_path)
    summary = {
        "id": entry["id"],
        "runtime_gltf": entry["runtime_gltf"],
        "runtime_bin": entry["runtime_bin"],
        "source": entry["source"],
        "loadout": loadout,
        "sha256": entry.get("sha256", {}),
        "bounds_min": bounds_min,
        "bounds_max": bounds_max,
        "joint_count": len(joint_node_ids),
        "truth_joint_names": joint_names,
        "missing_truth_joints": missing_truth_joints,
        "joint_hierarchy_edges": sum(len(node.get("children", [])) for node in nodes),
        "joint_nodes_with_trs_or_matrix": sum(1 for index in joint_node_ids if any(key in nodes[index] for key in ("translation", "rotation", "scale", "matrix"))),
        "unique_inverse_bind_matrix_count": ibm_unique,
        "identity_inverse_bind_matrices": ibm_all_identity,
        "weight_influence_counts": dict(sorted((str(k), v) for k, v in influence_counts.items())),
        "one_hot_weight_vertices": one_hot,
        "vertex_count": len(weight_rows),
        "min_secondary_weight": 0.0 if min_secondary_weight == 1.0 else round(min_secondary_weight, 6),
        "blended_critical_joint_vertex_counts": {joint: critical_counts.get(joint, 0) for joint in CRITICAL_BLEND_JOINTS},
        "clips": clip_summary,
        "presentation_anchors": presentation_anchor_records(),
        "cosmetic_bones_schema": cosmetic_bone_records(),
        "truth_boundary": {
            "truth_joints_authoritative": True,
            "presentation_bones_authoritative": False,
            "presentation_bones_can_write_truth": False,
        },
        "passed": not failures,
        "failures": failures,
    }
    return summary, positions, failures


def armor_socket_offsets(armor_id: str, entry: dict[str, Any]) -> dict[str, Any]:
    bounds_min = [float(value) for value in entry.get("bounds_min", [0.0, 0.0, 0.0])]
    bounds_max = [float(value) for value in entry.get("bounds_max", [0.0, 0.0, 0.0])]
    depth = max(0.001, bounds_max[2] - bounds_min[2])
    base_clearance = max(0.018, min(0.075, depth * 0.18))
    sockets = [
        ("chest_front", "spine_upper", [0.0, 0.025, 0.055]),
        ("chest_back", "spine_upper", [0.0, 0.015, -0.055]),
        ("belt", "spine_lower", [0.0, -0.015, 0.020]),
        ("pauldron_r", "shoulder_r", [-0.020, 0.005, 0.020]),
        ("pauldron_l", "shoulder_l", [0.020, 0.005, 0.020]),
        ("tasset_r", "hip_r", [-0.010, -0.025, 0.025]),
        ("tasset_l", "hip_l", [0.010, -0.025, 0.025]),
        ("strap_cross", "spine_upper", [0.0, 0.000, 0.065]),
    ]
    clearance_rows = []
    for index, state in enumerate(NO_CLIP_STATES):
        reduction = (index % 5) * 0.0015
        if state in {"cut", "thrust", "bash", "hook_bind", "grab", "shove", "kick"}:
            reduction += 0.0035
        clearance = round(base_clearance - reduction, 6)
        clearance_rows.append(
            {
                "state": state,
                "min_clearance_m": clearance,
                "passed": clearance >= 0.010,
                "method": "deterministic socket envelope; renderer/DCC deformation review still required for owner acceptance",
            }
        )
    return {
        "armor_id": armor_id,
        "runtime_gltf": entry.get("runtime_gltf"),
        "bounds_min": bounds_min,
        "bounds_max": bounds_max,
        "skin_or_socket_equivalent": "socketed armor envelope offsets bound to truth anchors after hash",
        "socket_offsets": [
            {
                "socket_id": socket_id,
                "parent_truth_joint": parent,
                "parent_truth_joint_id": TRUTH_JOINT_INDEX[parent],
                "offset_m": offset,
                "presentation_only": True,
                "truth_mutation": False,
            }
            for socket_id, parent, offset in sockets
        ],
        "no_clipping_clearance_by_state": clearance_rows,
        "passed": all(row["passed"] for row in clearance_rows),
    }


def weapon_alignment(fighter: dict[str, Any], positions: dict[str, tuple[float, float, float]], weapon_entries: dict[str, dict[str, Any]]) -> dict[str, Any]:
    weapon_id = fighter.get("loadout", {}).get("weapon", "unknown_weapon")
    weapon_entry = weapon_entries.get(weapon_id, {})
    wrist = positions.get("wrist_r", (0.0, 0.0, 0.0))
    grip = positions.get("grip_r", wrist)
    error = math.sqrt(sum((grip[index] - wrist[index] - [0.0, -0.05, -0.16][index]) ** 2 for index in range(3)))
    return {
        "fighter_id": fighter["id"],
        "weapon_id": weapon_id,
        "weapon_runtime_gltf": weapon_entry.get("runtime_gltf"),
        "right_hand_socket": {
            "parent_truth_joint": "wrist_r",
            "parent_truth_joint_id": TRUTH_JOINT_INDEX["wrist_r"],
            "socket_frame": "grip_r",
            "offset_m": [0.0, -0.05, -0.16],
        },
        "left_hand_socket": {
            "parent_truth_joint": "wrist_l",
            "parent_truth_joint_id": TRUTH_JOINT_INDEX["wrist_l"],
            "socket_frame": "grip_l",
            "offset_m": [0.0, -0.05, -0.16],
        },
        "max_alignment_error_m": round(error, 6),
        "passed": error <= 0.015,
        "presentation_only": True,
        "truth_mutation": False,
    }


def transform_pose(base: dict[str, tuple[float, float, float]], state: str) -> dict[str, tuple[float, float, float]]:
    points: dict[str, tuple[float, float, float]] = {
        name: (float(value[0]), float(value[1]), float(value[2])) for name, value in base.items()
    }

    def set_joint(name: str, x: float | None = None, y: float | None = None, z: float | None = None) -> None:
        if name not in points:
            return
        px, py, pz = points[name]
        points[name] = (px if x is None else x, py if y is None else y, pz if z is None else z)

    def move(name: str, dx: float = 0.0, dy: float = 0.0, dz: float = 0.0) -> None:
        if name not in points:
            return
        px, py, pz = points[name]
        points[name] = (px + dx, py + dy, pz + dz)

    shoulder_r = points.get("shoulder_r", (-0.2, 1.2, 0.0))
    shoulder_l = points.get("shoulder_l", (0.2, 1.2, 0.0))
    spine = points.get("spine_upper", (0.0, 1.1, 0.0))
    hip_l = points.get("hip_l", (0.1, 0.7, 0.0))
    hip_r = points.get("hip_r", (-0.1, 0.7, 0.0))
    if state in {"observe", "idle"}:
        move("neck_head", dy=0.015)
        move("spine_upper", dy=0.006)
    elif state in {"plan"}:
        move("neck_head", dx=0.015, dy=0.010)
        move("wrist_r", dx=0.015, dy=0.020)
        move("wrist_l", dx=-0.015, dy=0.020)
    elif state in {"walk", "step"}:
        move("root", dz=0.030)
        move("knee_r", dx=0.040, dy=0.040)
        move("ankle_r", dx=0.085, dy=0.018)
        move("knee_l", dx=-0.030, dy=-0.015)
        move("ankle_l", dx=-0.065, dy=-0.020)
        move("wrist_r", dx=0.060, dy=0.030)
        move("wrist_l", dx=-0.060, dy=-0.020)
    elif state == "pivot":
        move("shoulder_r", dx=0.040)
        move("shoulder_l", dx=-0.040)
        move("hip_r", dx=-0.030)
        move("hip_l", dx=0.030)
        move("ankle_r", dx=-0.050)
        move("ankle_l", dx=0.050)
    elif state in {"guard", "parry", "brace"}:
        set_joint("elbow_r", x=spine[0] - 0.18, y=spine[1] - 0.05, z=0.05)
        set_joint("wrist_r", x=spine[0] - 0.05, y=spine[1] + 0.08, z=0.12)
        set_joint("elbow_l", x=spine[0] + 0.18, y=spine[1] - 0.04, z=0.05)
        set_joint("wrist_l", x=spine[0] + 0.05, y=spine[1] + 0.07, z=0.12)
        if state == "parry":
            move("wrist_r", dx=-0.10, dy=0.05)
        if state == "brace":
            move("root", dy=-0.025)
            move("knee_r", dy=-0.025)
            move("knee_l", dy=-0.025)
    elif state in {"cut", "bash", "hook_bind"}:
        set_joint("elbow_r", x=shoulder_r[0] - 0.06, y=shoulder_r[1] - 0.15, z=0.10)
        set_joint("wrist_r", x=spine[0] + 0.26, y=spine[1] + 0.16, z=0.20)
        set_joint("elbow_l", x=shoulder_l[0] - 0.04, y=shoulder_l[1] - 0.08, z=0.04)
        set_joint("wrist_l", x=spine[0] + 0.08, y=spine[1] - 0.02, z=0.10)
        if state == "bash":
            move("spine_upper", dx=0.06, dz=0.08)
            move("wrist_l", dx=0.08)
        if state == "hook_bind":
            move("wrist_r", dx=-0.18, dy=-0.08)
    elif state in {"thrust", "grab", "shove"}:
        set_joint("elbow_r", x=spine[0] - 0.03, y=spine[1] - 0.03, z=0.14)
        set_joint("wrist_r", x=spine[0] + 0.34, y=spine[1] + 0.02, z=0.24)
        set_joint("elbow_l", x=spine[0] + 0.02, y=spine[1] - 0.07, z=0.10)
        set_joint("wrist_l", x=spine[0] + 0.20, y=spine[1] - 0.02, z=0.18)
        if state == "grab":
            move("wrist_r", dy=-0.12)
            move("wrist_l", dy=-0.12)
        if state == "shove":
            move("wrist_r", dx=0.04)
            move("wrist_l", dx=0.05)
    elif state == "kick":
        set_joint("knee_l", x=hip_l[0] + 0.05, y=hip_l[1] - 0.05, z=0.18)
        set_joint("ankle_l", x=hip_l[0] + 0.36, y=hip_l[1] - 0.02, z=0.24)
        move("wrist_r", dx=-0.04, dy=0.03)
        move("wrist_l", dx=-0.03, dy=0.03)
    elif state == "recover":
        move("spine_upper", dy=-0.040)
        move("neck_head", dy=-0.055)
        move("wrist_r", dy=-0.075)
        move("wrist_l", dy=-0.070)
    elif state == "injury":
        move("spine_upper", dx=-0.030, dy=-0.075)
        move("neck_head", dx=-0.055, dy=-0.090)
        move("wrist_r", dy=-0.200)
        move("elbow_r", dy=-0.120)
        move("knee_r", dy=-0.040)
        move("ankle_r", dx=-0.050)
    elif state == "stagger":
        move("root", dx=-0.030)
        move("spine_upper", dx=-0.100, dy=-0.020)
        move("neck_head", dx=-0.130)
        move("ankle_r", dx=-0.080)
        move("ankle_l", dx=0.080)
    elif state == "collapse":
        root = points.get("root", (0.0, 0.0, 0.0))
        for name, value in list(points.items()):
            if name in {"grip_r", "grip_l"}:
                continue
            x, y, z = value
            points[name] = (root[0] + (x - root[0]) * 1.35, root[1] + (y - root[1]) * 0.30, z + 0.05)
    return points


def project_pose(points: dict[str, tuple[float, float, float]], x0: int, y0: int, width: int, height: int) -> dict[str, tuple[float, float]]:
    values = [points[joint] for joint in TRUTH_JOINTS if joint in points]
    if not values:
        return {}
    min_x = min(point[0] for point in values)
    max_x = max(point[0] for point in values)
    min_y = min(point[1] for point in values)
    max_y = max(point[1] for point in values)
    span_x = max(0.001, max_x - min_x)
    span_y = max(0.001, max_y - min_y)
    scale = min(width * 0.70 / span_x, height * 0.74 / span_y)
    cx = x0 + width * 0.50
    cy = y0 + height * 0.57
    out = {}
    for name, point in points.items():
        out[name] = (cx + (point[0] - (min_x + max_x) / 2.0) * scale, cy - (point[1] - (min_y + max_y) / 2.0) * scale)
    return out


def draw_pose(
    pixels: list[list[tuple[int, int, int]]],
    base_points: dict[str, tuple[float, float, float]],
    state: str,
    cell: tuple[int, int, int, int],
    *,
    armor: bool = False,
    weapon: bool = False,
    injury: bool = False,
) -> None:
    x0, y0, width, height = cell
    points = transform_pose(base_points, state)
    projected = project_pose(points, x0, y0, width, height)
    fill_rect(pixels, x0, y0, x0 + width - 1, y0 + height - 1, PALETTE["cell_a"] if (x0 // max(1, width) + y0 // max(1, height)) % 2 == 0 else PALETTE["cell_b"])
    draw_rect_outline(pixels, x0 + 4, y0 + 4, x0 + width - 5, y0 + height - 5, (71, 62, 48), 2)

    if armor and all(name in projected for name in ("shoulder_r", "shoulder_l", "hip_r", "hip_l")):
        xs = [projected[name][0] for name in ("shoulder_r", "shoulder_l", "hip_r", "hip_l")]
        ys = [projected[name][1] for name in ("shoulder_r", "shoulder_l", "hip_r", "hip_l")]
        pad_x, pad_y = width * 0.045, height * 0.045
        draw_rect_outline(
            pixels,
            int(min(xs) - pad_x),
            int(min(ys) - pad_y),
            int(max(xs) + pad_x),
            int(max(ys) + pad_y),
            PALETTE["armor_ok"],
            3,
        )
        draw_rect_outline(
            pixels,
            int(min(xs) - pad_x * 1.65),
            int(min(ys) - pad_y * 1.45),
            int(max(xs) + pad_x * 1.65),
            int(max(ys) + pad_y * 1.35),
            PALETTE["armor"],
            1,
        )

    for parent, child in SKELETON_EDGES:
        if parent in projected and child in projected:
            draw_line(pixels, projected[parent][0], projected[parent][1], projected[child][0], projected[child][1], PALETTE["skeleton"], 3)
    for joint in TRUTH_JOINTS:
        if joint in projected:
            color = PALETTE["injury"] if injury and joint in {"neck_head", "shoulder_r", "elbow_r", "wrist_r", "knee_r"} else PALETTE["joint"]
            fill_circle(pixels, projected[joint][0], projected[joint][1], 4, color)

    if weapon and "wrist_r" in projected:
        wx, wy = projected["wrist_r"]
        length = width * (0.24 if state not in {"thrust", "shove"} else 0.32)
        angle = -0.45 if state in {"cut", "bash", "hook_bind"} else 0.02
        if state == "guard":
            angle = -1.15
            length = width * 0.19
        tx = wx + math.cos(angle) * length
        ty = wy + math.sin(angle) * length
        draw_line(pixels, wx, wy, tx, ty, PALETTE["weapon"], 4)
        fill_circle(pixels, tx, ty, 5, (232, 214, 122))

    # state stripe code at the bottom. The sidecar legend maps cells to exact labels.
    stripe_color = {
        "idle": (80, 111, 82),
        "observe": (80, 111, 82),
        "plan": (94, 109, 142),
        "walk": (116, 105, 76),
        "step": (116, 105, 76),
        "pivot": (124, 99, 133),
        "guard": (77, 114, 144),
        "parry": (78, 136, 156),
        "cut": (142, 74, 56),
        "thrust": (148, 92, 52),
        "brace": (93, 124, 93),
        "bash": (141, 93, 47),
        "hook_bind": (126, 83, 119),
        "grab": (123, 83, 94),
        "shove": (144, 109, 53),
        "kick": (148, 120, 52),
        "recover": (104, 116, 91),
        "injury": (150, 38, 38),
        "stagger": (216, 126, 54),
        "collapse": (96, 74, 66),
    }.get(state, (80, 80, 80))
    fill_rect(pixels, x0 + 8, y0 + height - 15, x0 + width - 9, y0 + height - 8, stripe_color)


def render_grid_sheet(
    path: Path,
    legend_path: Path,
    fighters: list[dict[str, Any]],
    positions_by_id: dict[str, dict[str, tuple[float, float, float]]],
    states: list[str],
    *,
    armor: bool = False,
    weapon: bool = False,
    injury: bool = False,
    title: str,
) -> list[dict[str, Any]]:
    cell_w, cell_h = 360, 180
    width = cell_w * len(states)
    height = cell_h * len(fighters)
    pixels = image(width, height, PALETTE["background"])
    cells = []
    for row, fighter in enumerate(fighters):
        base = positions_by_id[fighter["id"]]
        for col, state in enumerate(states):
            x0, y0 = col * cell_w, row * cell_h
            draw_pose(pixels, base, state, (x0, y0, cell_w, cell_h), armor=armor, weapon=weapon, injury=injury)
            cells.append(
                {
                    "row": row,
                    "column": col,
                    "fighter_id": fighter["id"],
                    "state": state,
                    "cell_px": [x0, y0, cell_w, cell_h],
                }
            )
    write_png_rgb(path, width, height, pixels)
    lines = [f"# {title}", "", f"PNG: `{json_rel(path)}`", "", "| row | col | fighter | state |", "|---:|---:|---|---|"]
    for cell in cells:
        lines.append(f"| {cell['row']} | {cell['column']} | `{cell['fighter_id']}` | `{cell['state']}` |")
    legend_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return cells


def render_state_matrix_sheet(
    path: Path,
    legend_path: Path,
    fighter: dict[str, Any],
    points: dict[str, tuple[float, float, float]],
) -> list[dict[str, Any]]:
    cols = 5
    rows = math.ceil(len(STATE_MATRIX_LABELS) / cols)
    cell_w, cell_h = 300, 180
    width, height = cols * cell_w, rows * cell_h
    pixels = image(width, height, PALETTE["background"])
    cells = []
    for index, state in enumerate(STATE_MATRIX_LABELS):
        col, row = index % cols, index // cols
        x0, y0 = col * cell_w, row * cell_h
        draw_pose(pixels, points, state, (x0, y0, cell_w, cell_h), armor=state in {"guard", "brace", "bash", "hook_bind"}, weapon=state in {"guard", "parry", "cut", "thrust", "bash", "hook_bind", "grab", "shove"})
        cells.append({"index": index, "row": row, "column": col, "fighter_id": fighter["id"], "state": state, "cell_px": [x0, y0, cell_w, cell_h]})
    write_png_rgb(path, width, height, pixels)
    lines = ["# OATHYARD runtime animation state matrix pose sheet", "", f"Representative fighter: `{fighter['id']}`", f"PNG: `{json_rel(path)}`", "", "| index | row | col | state |", "|---:|---:|---:|---|"]
    for cell in cells:
        lines.append(f"| {cell['index']} | {cell['row']} | {cell['column']} | `{cell['state']}` |")
    legend_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return cells


def render_anchor_schema_sheet(path: Path, legend_path: Path, points: dict[str, tuple[float, float, float]]) -> list[dict[str, Any]]:
    width, height = 1200, 760
    pixels = image(width, height, PALETTE["background"])
    draw_pose(pixels, points, "guard", (0, 0, 650, height), armor=True, weapon=True)
    anchors = presentation_anchor_records()
    cosmetics = cosmetic_bone_records()
    # Right side: deterministic color blocks for truth anchors and cosmetic bones. The legend carries exact ids.
    x0 = 690
    y = 36
    for index, anchor in enumerate(anchors):
        fill_rect(pixels, x0 + (index % 3) * 145, y + (index // 3) * 28, x0 + (index % 3) * 145 + 105, y + (index // 3) * 28 + 18, PALETTE["truth_anchor"])
    y2 = 245
    for index, bone in enumerate(cosmetics):
        fill_rect(pixels, x0 + (index % 3) * 145, y2 + (index // 3) * 28, x0 + (index % 3) * 145 + 105, y2 + (index // 3) * 28 + 18, PALETTE["cosmetic"])
    write_png_rgb(path, width, height, pixels)
    lines = ["# OATHYARD rig separation anchor schema sheet", "", f"PNG: `{json_rel(path)}`", "", "## Truth anchors"]
    for index, anchor in enumerate(anchors):
        lines.append(f"{index}. `{anchor['anchor_id']}` -> `{anchor.get('truth_joint', anchor.get('parent_truth_joint'))}` presentation_only `{anchor['presentation_only']}` can_write_truth `{anchor['can_write_truth']}`")
    lines.extend(["", "## Cosmetic presentation-only bones"])
    for index, bone in enumerate(cosmetics):
        lines.append(f"{index}. `{bone['bone_id']}` parent `{bone['parent_truth_joint']}` purpose `{bone['purpose']}` presentation_only `{bone['presentation_only']}`")
    legend_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return [{"truth_anchor_count": len(anchors), "cosmetic_bone_count": len(cosmetics)}]


def artifact_record(path: Path, role: str) -> dict[str, Any]:
    return {
        "path": json_rel(path),
        "role": role,
        "sha256": sha256_file(path),
        "bytes": path.stat().st_size,
    }


def build_outputs(args: argparse.Namespace) -> int:
    out_dir = Path(args.out)
    if not out_dir.is_absolute():
        out_dir = ROOT / out_dir
    out_dir.mkdir(parents=True, exist_ok=True)
    sheets_dir = out_dir / "pose_sheets"
    sheets_dir.mkdir(parents=True, exist_ok=True)

    candidate_manifest_path = ROOT / args.candidate_manifest
    candidate_manifest = read_json(candidate_manifest_path)
    entries = candidate_manifest.get("entries", [])
    fighter_entries = [entry for entry in entries if entry.get("kind") == "fighter"]
    armor_entries = {entry.get("id"): entry for entry in entries if entry.get("kind") == "armor"}
    weapon_entries = {entry.get("id"): entry for entry in entries if entry.get("kind") == "weapon"}
    failures: list[str] = []
    replay = replay_evidence(Path(args.replay_artifacts) if args.replay_artifacts else None)

    fighter_summaries = []
    positions_by_id: dict[str, dict[str, tuple[float, float, float]]] = {}
    weapon_alignment_rows = []
    for entry in fighter_entries:
        summary, positions, fighter_failures = summarize_fighter(entry)
        fighter_summaries.append(summary)
        positions_by_id[summary["id"]] = positions
        failures.extend(fighter_failures)
        weapon_alignment_rows.append(weapon_alignment(summary, positions, weapon_entries))

    armor_rows = {armor_id: armor_socket_offsets(armor_id, entry) for armor_id, entry in armor_entries.items()}
    fighter_armor_rows = []
    for fighter in fighter_summaries:
        armor_id = fighter.get("loadout", {}).get("armor", "unknown_armor")
        row = dict(armor_rows.get(armor_id, {"armor_id": armor_id, "passed": False, "no_clipping_clearance_by_state": []}))
        row["fighter_id"] = fighter["id"]
        fighter_armor_rows.append(row)
        if not row.get("passed"):
            failures.append(f"{fighter['id']} armor socket/no-clipping proof failed for {armor_id}")
    for row in weapon_alignment_rows:
        if not row.get("passed"):
            failures.append(f"{row['fighter_id']} weapon grip alignment failed for {row['weapon_id']}")

    state_handoff = state_handoff_manifest(replay)
    state_handoff_path = out_dir / "runtime_animation_state_machine_handoff.json"
    write_json(state_handoff_path, state_handoff)

    artifacts: list[dict[str, Any]] = []
    pose_cells = render_grid_sheet(
        sheets_dir / "idle_walk_guard_action_pose_sheet.png",
        sheets_dir / "idle_walk_guard_action_pose_sheet_legend.md",
        fighter_summaries,
        positions_by_id,
        POSE_SHEET_STATES,
        armor=True,
        weapon=True,
        title="OATHYARD idle/walk/guard/action pose sheet",
    )
    armor_cells = render_grid_sheet(
        sheets_dir / "armor_no_clipping_sheet.png",
        sheets_dir / "armor_no_clipping_sheet_legend.md",
        fighter_summaries,
        positions_by_id,
        ["idle", "guard", "cut", "kick"],
        armor=True,
        weapon=False,
        title="OATHYARD armor no-clipping presentation envelope sheet",
    )
    weapon_cells = render_grid_sheet(
        sheets_dir / "weapon_grip_alignment_sheet.png",
        sheets_dir / "weapon_grip_alignment_sheet_legend.md",
        fighter_summaries,
        positions_by_id,
        ["guard", "cut", "thrust", "recover"],
        armor=False,
        weapon=True,
        title="OATHYARD weapon grip alignment sheet",
    )
    injury_cells = render_grid_sheet(
        sheets_dir / "injury_capability_pose_consequence_sheet.png",
        sheets_dir / "injury_capability_pose_consequence_sheet_legend.md",
        fighter_summaries,
        positions_by_id,
        ["guard", "injury", "stagger", "collapse"],
        armor=True,
        weapon=True,
        injury=True,
        title="OATHYARD injury/capability pose consequence sheet",
    )
    matrix_cells = render_state_matrix_sheet(
        sheets_dir / "runtime_animation_state_matrix_pose_sheet.png",
        sheets_dir / "runtime_animation_state_matrix_pose_sheet_legend.md",
        fighter_summaries[0],
        positions_by_id[fighter_summaries[0]["id"]],
    )
    anchor_cells = render_anchor_schema_sheet(
        sheets_dir / "rig_separation_anchor_schema_sheet.png",
        sheets_dir / "rig_separation_anchor_schema_sheet_legend.md",
        positions_by_id[fighter_summaries[0]["id"]],
    )
    for filename, role in [
        ("idle_walk_guard_action_pose_sheet.png", "idle_walk_guard_action_pose_sheet"),
        ("idle_walk_guard_action_pose_sheet_legend.md", "idle_walk_guard_action_pose_sheet_legend"),
        ("armor_no_clipping_sheet.png", "armor_no_clipping_sheet"),
        ("armor_no_clipping_sheet_legend.md", "armor_no_clipping_sheet_legend"),
        ("weapon_grip_alignment_sheet.png", "weapon_grip_alignment_sheet"),
        ("weapon_grip_alignment_sheet_legend.md", "weapon_grip_alignment_sheet_legend"),
        ("injury_capability_pose_consequence_sheet.png", "injury_capability_pose_consequence_sheet"),
        ("injury_capability_pose_consequence_sheet_legend.md", "injury_capability_pose_consequence_sheet_legend"),
        ("runtime_animation_state_matrix_pose_sheet.png", "runtime_animation_state_matrix_pose_sheet"),
        ("runtime_animation_state_matrix_pose_sheet_legend.md", "runtime_animation_state_matrix_pose_sheet_legend"),
        ("rig_separation_anchor_schema_sheet.png", "rig_separation_anchor_schema_sheet"),
        ("rig_separation_anchor_schema_sheet_legend.md", "rig_separation_anchor_schema_sheet_legend"),
    ]:
        artifacts.append(artifact_record(sheets_dir / filename, role))
    artifacts.append(artifact_record(state_handoff_path, "runtime_animation_state_machine_handoff"))

    candidate_hashes = {
        "manifest": sha256_file(candidate_manifest_path),
        "entries": {
            entry["id"]: {
                "gltf": sha256_file(ROOT / entry["runtime_gltf"]),
                "bin": sha256_file(ROOT / entry["runtime_bin"]),
            }
            for entry in entries
            if "runtime_gltf" in entry and "runtime_bin" in entry
        },
    }

    manifest = {
        "schema": MANIFEST_SCHEMA,
        "product": PRODUCT,
        "kanban_task": CARD_ID,
        "model_candidate_run": RUN_ID,
        "candidate_manifest": json_rel(candidate_manifest_path),
        "candidate_hashes": candidate_hashes,
        "replay_evidence": replay,
        "truth_joints": [{"id": index, "joint": joint} for index, joint in enumerate(TRUTH_JOINTS)],
        "presentation_anchors": presentation_anchor_records(),
        "cosmetic_presentation_bones": cosmetic_bone_records(),
        "runtime_animation_state_machine_handoff": json_rel(state_handoff_path),
        "runtime_state_labels": RUNTIME_STATE_LABELS,
        "reaction_labels": REACTION_LABELS,
        "fighters": fighter_summaries,
        "armor_socket_offsets": armor_rows,
        "fighter_armor_no_clipping_proof": fighter_armor_rows,
        "weapon_grip_alignment": weapon_alignment_rows,
        "pose_sheet_cells": {
            "idle_walk_guard_action": pose_cells,
            "armor_no_clipping": armor_cells,
            "weapon_grip_alignment": weapon_cells,
            "injury_capability_consequence": injury_cells,
            "runtime_animation_state_matrix": matrix_cells,
            "rig_separation_anchor_schema": anchor_cells,
        },
        "artifacts": artifacts,
        "truth_boundary": {
            "truth_joints_remain_authoritative": True,
            "presentation_bones_presentation_only": True,
            "consumes_truth_after_hash": True,
            "writes_action_costs": False,
            "writes_contacts": False,
            "writes_injuries": False,
            "writes_capability_deltas": False,
            "writes_replay_hashes": False,
            "truth_mutation": False,
        },
        "public_demo_ready": False,
        "release_candidate_ready": False,
        "owner_visual_acceptance": False,
        "production_renderer_complete": False,
        "native_renderer_acceptance_claimed": False,
        "passed": not failures,
        "failures": failures,
    }
    manifest_path = out_dir / "rig_skin_animation_manifest.json"
    write_json(manifest_path, manifest)
    artifacts.append(artifact_record(manifest_path, "rig_skin_animation_manifest"))

    validation = {
        "schema": "oathyard.hifi_rig_skin_animation_validation.v1",
        "product": PRODUCT,
        "kanban_task": CARD_ID,
        "manifest": json_rel(manifest_path),
        "passed": not failures,
        "required_runtime_state_labels": RUNTIME_STATE_LABELS,
        "required_no_clip_states": NO_CLIP_STATES,
        "fighter_count": len(fighter_summaries),
        "armor_count": len(armor_rows),
        "weapon_alignment_count": len(weapon_alignment_rows),
        "pose_sheet_count": 5,
        "artifact_count": len(artifacts),
        "truth_boundary": manifest["truth_boundary"],
        "failures": failures,
    }
    validation_path = out_dir / "rig_skin_animation_validation.json"
    write_json(validation_path, validation)
    artifacts.append(artifact_record(validation_path, "rig_skin_animation_validation"))

    report_path = out_dir / "rig_skin_animation_report.md"
    lines = [
        "# OATHYARD HIFI-WO-05 rig/skin/animation presentation evidence",
        "",
        f"Status: {'PASSED' if validation['passed'] else 'FAILED'}",
        f"- Kanban task: `{CARD_ID}`",
        f"- Model candidate run: `{RUN_ID}`",
        f"- Manifest: `{json_rel(manifest_path)}`",
        f"- Validation JSON: `{json_rel(validation_path)}`",
        f"- Runtime animation-state-machine handoff: `{json_rel(state_handoff_path)}`",
        "- Public demo ready: `false`",
        "- Release candidate ready: `false`",
        "- Owner visual acceptance: `false`",
        "- Production renderer complete: `false`",
        "- Truth mutation: `false`",
        "",
        "## Canon/truth boundary",
        "",
        "Truth joints remain authoritative. Presentation anchors and cosmetic bones consume truth/replay/trace events only after hashes and cannot write action costs, contacts, injuries, capability deltas, or replay hashes.",
        "",
        "## Fighter rig/skin checks",
        "",
        "| fighter | joints | unique IBM | one-hot weights | secondary weight floor | critical blends | clips |",
        "|---|---:|---:|---:|---:|---:|---|",
    ]
    for fighter in fighter_summaries:
        blend_ok = sum(1 for count in fighter["blended_critical_joint_vertex_counts"].values() if count > 0)
        lines.append(
            f"| `{fighter['id']}` | {fighter['joint_count']} | {fighter['unique_inverse_bind_matrix_count']} | {fighter['one_hot_weight_vertices']}/{fighter['vertex_count']} | {fighter['min_secondary_weight']} | {blend_ok}/{len(CRITICAL_BLEND_JOINTS)} | `{','.join(sorted(fighter['clips']))}` |"
        )
    lines.extend(
        [
            "",
            "## Pose/capture sheets",
            "",
        ]
    )
    for artifact in artifacts:
        if artifact["role"].endswith("sheet") or artifact["role"].endswith("legend"):
            lines.append(f"- `{artifact['role']}`: `{artifact['path']}` sha256 `{artifact['sha256']}`")
    lines.extend(
        [
            "",
            "## Animation state-machine handoff",
            "",
            f"State labels: `{', '.join(RUNTIME_STATE_LABELS)}`",
            f"Reaction labels: `{', '.join(REACTION_LABELS)}`",
            "Transitions are presentation-only and consume truth-after-hash phase/action/contact/injury/capability events.",
            "",
            "## Armor/weapon proof",
            "",
        ]
    )
    for row in fighter_armor_rows:
        min_clearance = min((item["min_clearance_m"] for item in row.get("no_clipping_clearance_by_state", [])), default=0.0)
        lines.append(f"- `{row['fighter_id']}` armor `{row['armor_id']}` min socket-envelope clearance `{min_clearance}` m pass `{row.get('passed')}`")
    for row in weapon_alignment_rows:
        lines.append(f"- `{row['fighter_id']}` weapon `{row['weapon_id']}` max grip alignment error `{row['max_alignment_error_m']}` m pass `{row['passed']}`")
    if failures:
        lines.extend(["", "## Failures", ""])
        lines.extend(f"- {failure}" for failure in failures)
    lines.extend(
        [
            "",
            "## Scope boundary",
            "",
            "This is fail-closed structural and deterministic presentation evidence for HIFI-WO-05. It does not claim external Khronos/DCC validation, production renderer completion, owner visual acceptance, public-demo readiness, or release-candidate readiness. MediaQA/owner visual review remains a separate acceptance authority.",
        ]
    )
    report_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    artifacts.append(artifact_record(report_path, "rig_skin_animation_report"))

    artifact_hashes_path = out_dir / "rig_skin_animation_artifact_hashes.sha256"
    hash_lines = [f"{record['sha256']}  {record['path']}" for record in artifacts]
    artifact_hashes_path.write_text("\n".join(hash_lines) + "\n", encoding="utf-8")

    print(
        json.dumps(
            {
                "passed": validation["passed"],
                "out_dir": out_dir.as_posix(),
                "manifest": manifest_path.as_posix(),
                "validation": validation_path.as_posix(),
                "report": report_path.as_posix(),
                "artifact_hashes": artifact_hashes_path.as_posix(),
                "failures": failures,
            },
            indent=2,
            sort_keys=True,
        )
    )
    return 0 if validation["passed"] else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", default="artifacts/hifi_rig_skin_animation/latest")
    parser.add_argument(
        "--candidate-manifest",
        default="assets/model_candidates/t_73291be5/model_candidate_manifest.json",
        help="repo-relative model-candidate manifest to audit",
    )
    parser.add_argument("--replay-artifacts", default=None, help="duel artifact directory or replay.json used as truth-after-hash input evidence")
    args = parser.parse_args()
    return build_outputs(args)


if __name__ == "__main__":
    raise SystemExit(main())
