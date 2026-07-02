#!/usr/bin/env python3
"""Generate fail-closed Blender weapon candidates from a concept roster.

This is an asset-candidate authoring tool, not a gameplay-truth tool. It writes
source .blend, exported .glb, preview .png, and manifests under
assets/production_candidates/<run_id>/weapons/<weapon_id>/.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import sys
from pathlib import Path

import bpy
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SPEC = ROOT / "assets_src/reference/concepts/weapon_diversity_concept_spec.json"

MATERIAL_COLORS = {
    "steel": ((0.58, 0.60, 0.58, 1.0), 1.0, 0.32),
    "dark_steel": ((0.16, 0.17, 0.17, 1.0), 1.0, 0.44),
    "black_iron": ((0.05, 0.055, 0.055, 1.0), 1.0, 0.50),
    "brass": ((0.55, 0.43, 0.21, 1.0), 1.0, 0.48),
    "leather": ((0.11, 0.06, 0.035, 1.0), 0.0, 0.72),
    "wood": ((0.22, 0.14, 0.075, 1.0), 0.0, 0.66),
    "ash_wood": ((0.34, 0.30, 0.23, 1.0), 0.0, 0.70),
    "blood": ((0.20, 0.016, 0.010, 1.0), 0.0, 0.68),
    "arcane": ((0.15, 0.42, 0.80, 1.0), 0.0, 0.18),
    "shadow": ((0.02, 0.022, 0.024, 1.0), 0.0, 0.85),
}


def parse_args():
    argv = sys.argv
    if "--" in argv:
        argv = argv[argv.index("--") + 1:]
    else:
        argv = []
    p = argparse.ArgumentParser()
    p.add_argument("--spec", default=str(DEFAULT_SPEC))
    p.add_argument("--run-id", required=True)
    p.add_argument("--only", default="", help="comma-separated weapon ids")
    p.add_argument("--samples", type=int, default=48)
    p.add_argument("--res-x", type=int, default=1200)
    p.add_argument("--res-y", type=int, default=1600)
    return p.parse_args(argv)


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def clear_scene() -> None:
    bpy.ops.object.select_all(action="SELECT")
    bpy.ops.object.delete()
    for collection in [bpy.data.meshes, bpy.data.materials, bpy.data.curves, bpy.data.images, bpy.data.lights, bpy.data.cameras]:
        for item in list(collection):
            if item.users == 0:
                collection.remove(item)


def material(name: str):
    color, metallic, roughness = MATERIAL_COLORS[name]
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes.get("Principled BSDF")
    if bsdf:
        bsdf.inputs["Base Color"].default_value = color
        bsdf.inputs["Metallic"].default_value = metallic
        bsdf.inputs["Roughness"].default_value = roughness
    return mat


def material_set():
    return {name: material(name) for name in MATERIAL_COLORS}


def apply_modifiers() -> None:
    for obj in list(bpy.context.scene.objects):
        if obj.type != "MESH":
            continue
        bpy.ops.object.select_all(action="DESELECT")
        bpy.context.view_layer.objects.active = obj
        obj.select_set(True)
        for mod in list(obj.modifiers):
            try:
                bpy.ops.object.modifier_apply(modifier=mod.name)
            except Exception:
                pass
        obj.select_set(False)


def add_cube(name: str, loc, scale, mat=None, bevel=0.0):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=loc)
    obj = bpy.context.object
    obj.name = name
    obj.dimensions = scale
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    if mat:
        obj.data.materials.append(mat)
    if bevel > 0:
        be = obj.modifiers.new("readability_bevel", "BEVEL")
        be.width = bevel
        be.segments = 2
        obj.modifiers.new("weighted_normals", "WEIGHTED_NORMAL")
    return obj


def add_cylinder(name: str, loc, radius, depth, mat=None, vertices=32, axis="Z", bevel=0.0):
    rot = (0.0, 0.0, 0.0)
    if axis == "X":
        rot = (0.0, math.radians(90), 0.0)
    elif axis == "Y":
        rot = (math.radians(90), 0.0, 0.0)
    bpy.ops.mesh.primitive_cylinder_add(vertices=vertices, radius=radius, depth=depth, location=loc, rotation=rot)
    obj = bpy.context.object
    obj.name = name
    if mat:
        obj.data.materials.append(mat)
    if bevel > 0:
        be = obj.modifiers.new("edge_softening", "BEVEL")
        be.width = bevel
        be.segments = 2
        obj.modifiers.new("weighted_normals", "WEIGHTED_NORMAL")
    return obj


def add_sphere(name: str, loc, radius, mat=None, segments=32):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=segments, ring_count=max(8, segments // 2), radius=radius, location=loc)
    obj = bpy.context.object
    obj.name = name
    if mat:
        obj.data.materials.append(mat)
    return obj


def add_torus(name: str, loc, major, minor, mat=None, rotation=(0.0, 0.0, 0.0)):
    bpy.ops.mesh.primitive_torus_add(major_radius=major, minor_radius=minor, major_segments=64, minor_segments=8, location=loc, rotation=rotation)
    obj = bpy.context.object
    obj.name = name
    if mat:
        obj.data.materials.append(mat)
    return obj


def add_cylinder_between(name: str, start, end, radius, mat=None, vertices=12):
    start = Vector(start)
    end = Vector(end)
    mid = (start + end) / 2
    direction = end - start
    length = direction.length
    bpy.ops.mesh.primitive_cylinder_add(vertices=vertices, radius=radius, depth=length, location=mid)
    obj = bpy.context.object
    obj.name = name
    obj.rotation_euler = direction.to_track_quat("Z", "Y").to_euler()
    if mat:
        obj.data.materials.append(mat)
    return obj


def add_prism(name: str, points_xz, thickness: float, mat=None, bevel=0.0):
    verts = []
    for x, z in points_xz:
        verts.append((x, -thickness / 2, z))
    for x, z in points_xz:
        verts.append((x, thickness / 2, z))
    n = len(points_xz)
    faces = [tuple(range(n)), tuple(range(n, 2 * n))]
    for i in range(n):
        faces.append((i, (i + 1) % n, (i + 1) % n + n, i + n))
    mesh = bpy.data.meshes.new(name + "_mesh")
    mesh.from_pydata(verts, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    if mat:
        obj.data.materials.append(mat)
    if bevel > 0:
        be = obj.modifiers.new("edge_bevel", "BEVEL")
        be.width = bevel
        be.segments = 2
        obj.modifiers.new("weighted_normals", "WEIGHTED_NORMAL")
    return obj


def add_blade(name: str, length: float, width: float, thickness: float, mat, curve=0.0, base_z=0.0, single_edge=False):
    segments = 8
    verts = []
    for i in range(segments + 1):
        t = i / segments
        z = base_z + t * length
        center_x = curve * (t * t - 0.15 * t)
        half = max(width * (1.0 - 0.82 * t) / 2, width * 0.035)
        yt = thickness * (1.0 - 0.25 * t) / 2
        if single_edge:
            verts.extend([(center_x - half * 0.75, -yt, z), (center_x + half, -yt, z), (center_x + half, yt, z), (center_x - half * 0.75, yt, z)])
        else:
            verts.extend([(center_x - half, 0, z), (center_x, -yt, z), (center_x + half, 0, z), (center_x, yt, z)])
    faces = []
    for i in range(segments):
        a = i * 4
        b = (i + 1) * 4
        faces += [(a, a + 1, b + 1, b), (a + 1, a + 2, b + 2, b + 1), (a + 2, a + 3, b + 3, b + 2), (a + 3, a, b, b + 3)]
    faces += [(0, 3, 2, 1), (segments * 4, segments * 4 + 1, segments * 4 + 2, segments * 4 + 3)]
    mesh = bpy.data.meshes.new(name + "_mesh")
    mesh.from_pydata(verts, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(mat)
    obj.modifiers.new("blade_weighted_normals", "WEIGHTED_NORMAL")
    be = obj.modifiers.new("sharpened_edge_bevel", "BEVEL")
    be.width = max(thickness * 0.12, 0.002)
    be.segments = 1
    return obj


def grip_wraps(prefix: str, zs, radius, mat):
    for i, z in enumerate(zs):
        add_cylinder(f"{prefix}_raised_wrap_{i}", (0, 0, z), radius, 0.012, mat, vertices=24, bevel=0.001)


def add_guard(width, z, mats, prefix="crossguard"):
    add_cylinder(f"{prefix}_bar", (0, 0, z), 0.018, width, mats["dark_steel"], vertices=16, axis="X", bevel=0.003)
    add_sphere(f"{prefix}_left_finial", (-width / 2, 0, z), 0.033, mats["brass"], segments=16)
    add_sphere(f"{prefix}_right_finial", (width / 2, 0, z), 0.033, mats["brass"], segments=16)


def add_guard_at(prefix, x, z, width, mats, radius=0.014):
    add_cylinder(f"{prefix}_bar", (x, 0, z), radius, width, mats["dark_steel"], vertices=16, axis="X", bevel=0.003)
    add_sphere(f"{prefix}_left_finial", (x - width / 2, 0, z), radius * 1.65, mats["brass"], segments=16)
    add_sphere(f"{prefix}_right_finial", (x + width / 2, 0, z), radius * 1.65, mats["brass"], segments=16)


def build_sword(spec, mats, variant):
    two_hand = variant in {"longsword", "greatsword", "curved_greatblade"}
    if variant == "greatsword":
        blade_len, blade_w, grip_len, guard_w = 1.28, 0.085, 0.36, 0.50
    elif variant == "curved_greatblade":
        blade_len, blade_w, grip_len, guard_w = 1.02, 0.078, 0.34, 0.20
    elif variant == "saber":
        blade_len, blade_w, grip_len, guard_w = 0.72, 0.065, 0.19, 0.18
    elif variant == "arming_sword":
        blade_len, blade_w, grip_len, guard_w = 0.76, 0.062, 0.19, 0.30
    else:
        blade_len, blade_w, grip_len, guard_w = 0.96, 0.068, 0.28, 0.38
    base_z = 0.04 + grip_len
    add_blade(f"{spec['id']}_blade", blade_len, blade_w, 0.020, mats["steel"], curve=0.18 if variant in {"curved_greatblade", "saber"} else 0.0, base_z=base_z, single_edge=variant in {"curved_greatblade", "saber"})
    if variant in {"curved_greatblade", "saber"}:
        add_torus(f"{spec['id']}_tsuba_guard", (0, 0, base_z - 0.012), 0.068 if variant == "curved_greatblade" else 0.050, 0.006, mats["brass"], rotation=(math.radians(90), 0, 0))
    else:
        add_guard(guard_w, base_z - 0.015, mats, f"{spec['id']}_guard")
    add_cylinder(f"{spec['id']}_grip_core", (0, 0, grip_len / 2 - 0.02), 0.030 if two_hand else 0.026, grip_len, mats["leather"], vertices=24, bevel=0.003)
    grip_wraps(spec["id"], [0.02 + i * (grip_len - 0.04) / 5 for i in range(6)], 0.033 if two_hand else 0.029, mats["brass"] if variant == "saber" else mats["leather"])
    add_cylinder(f"{spec['id']}_pommel", (0, 0, -0.055), 0.052 if two_hand else 0.043, 0.048, mats["dark_steel"], vertices=18, bevel=0.005)
    if variant == "greatsword":
        add_cylinder(f"{spec['id']}_left_parry_lug", (-0.060, 0, base_z + 0.18), 0.010, 0.13, mats["brass"], vertices=12, axis="X", bevel=0.002)
        add_cylinder(f"{spec['id']}_right_parry_lug", (0.060, 0, base_z + 0.18), 0.010, 0.13, mats["brass"], vertices=12, axis="X", bevel=0.002)


def build_dual_daggers(spec, mats):
    # Keep the weapons explicitly separate. The previous candidate accidentally
    # left guard pieces at world origin and read as one connected double weapon.
    for side, x, lean in [("left", -0.16, -0.035), ("right", 0.16, 0.035)]:
        blade = add_blade(f"{spec['id']}_{side}_blade", 0.46, 0.050, 0.016, mats["steel"], curve=0.04 if side == "right" else -0.04, base_z=0.18)
        blade.location.x += x
        blade.location.x += lean
        add_guard_at(f"{spec['id']}_{side}_guard", x, 0.165, 0.13, mats, radius=0.010)
        add_cylinder(f"{spec['id']}_{side}_grip", (x, 0, 0.075), 0.021, 0.17, mats["leather"], vertices=18, bevel=0.002)
        add_sphere(f"{spec['id']}_{side}_pommel", (x, 0, -0.025), 0.030, mats["brass"], segments=16)
        for i, z in enumerate([0.030, 0.070, 0.110]):
            add_cylinder(f"{spec['id']}_{side}_separate_wrap_{i}", (x, 0, z), 0.024, 0.008, mats["brass"], vertices=16, bevel=0.001)


def build_polearm(spec, mats, variant):
    shaft_len = 1.55 if variant in {"spear", "glaive", "halberd"} else 1.25
    add_cylinder(f"{spec['id']}_shaft", (0, 0, shaft_len / 2), 0.018, shaft_len, mats["ash_wood"], vertices=18, bevel=0.001)
    for z in [0.25, 0.62, 0.98]:
        add_cylinder(f"{spec['id']}_grip_band_{z:.2f}", (0, 0, z), 0.022, 0.030, mats["leather"], vertices=18)
    if variant == "spear":
        add_prism(f"{spec['id']}_leaf_spearhead", [(0, shaft_len + 0.26), (0.060, shaft_len + 0.10), (0.025, shaft_len), (-0.025, shaft_len), (-0.060, shaft_len + 0.10)], 0.018, mats["steel"], bevel=0.002)
        add_prism(f"{spec['id']}_butt_spike", [(0, -0.13), (0.032, 0.0), (-0.032, 0.0)], 0.014, mats["dark_steel"], bevel=0.001)
    elif variant == "glaive":
        add_prism(f"{spec['id']}_sweeping_glaive_blade", [(0.020, shaft_len - 0.05), (0.21, shaft_len + 0.12), (0.08, shaft_len + 0.36), (-0.020, shaft_len + 0.22)], 0.020, mats["steel"], bevel=0.003)
        add_cylinder(f"{spec['id']}_butt_cap", (0, 0, -0.025), 0.025, 0.045, mats["dark_steel"], vertices=18)
    else:
        add_prism(f"{spec['id']}_top_spike", [(0, shaft_len + 0.24), (0.040, shaft_len), (-0.040, shaft_len)], 0.018, mats["steel"], bevel=0.002)
        add_prism(f"{spec['id']}_axe_blade", [(0.015, shaft_len - 0.05), (0.26, shaft_len + 0.06), (0.16, shaft_len + 0.23), (0.025, shaft_len + 0.18)], 0.024, mats["steel"], bevel=0.003)
        add_prism(f"{spec['id']}_rear_hook", [(-0.018, shaft_len + 0.00), (-0.20, shaft_len + 0.05), (-0.11, shaft_len + 0.16), (-0.018, shaft_len + 0.13)], 0.020, mats["dark_steel"], bevel=0.002)


def build_axe(spec, mats, variant):
    shaft_len = 1.18 if variant == "dane_axe" else 0.82
    add_cylinder(f"{spec['id']}_shaft", (0, 0, shaft_len / 2), 0.024, shaft_len, mats["wood"], vertices=18, bevel=0.002)
    head_z = shaft_len + 0.02
    if variant == "dane_axe":
        points = [(0.00, head_z - 0.12), (0.30, head_z - 0.03), (0.25, head_z + 0.22), (0.04, head_z + 0.15), (-0.02, head_z + 0.02)]
    else:
        points = [(-0.02, head_z - 0.10), (0.23, head_z - 0.03), (0.24, head_z + 0.15), (0.02, head_z + 0.21), (-0.08, head_z + 0.08)]
    add_prism(f"{spec['id']}_readable_chopping_blade", points, 0.030, mats["steel"], bevel=0.004)
    add_cylinder(f"{spec['id']}_socket", (0, 0, head_z + 0.04), 0.045, 0.12, mats["dark_steel"], vertices=18, bevel=0.002)
    add_cylinder(f"{spec['id']}_butt_cap", (0, 0, -0.025), 0.027, 0.045, mats["brass"], vertices=16)


def build_blunt(spec, mats, variant):
    shaft_len = 0.68 if variant == "mace" else 0.92
    add_cylinder(f"{spec['id']}_grip", (0, 0, shaft_len / 2), 0.025, shaft_len, mats["leather"], vertices=18, bevel=0.002)
    head_z = shaft_len + 0.10
    if variant == "hammer":
        add_cube(f"{spec['id']}_colossal_hammer_head", (0, 0, head_z), (0.62, 0.22, 0.22), mats["dark_steel"], bevel=0.020)
        add_cube(f"{spec['id']}_left_striking_face", (-0.36, 0, head_z), (0.12, 0.25, 0.25), mats["steel"], bevel=0.015)
        add_cube(f"{spec['id']}_right_striking_face", (0.36, 0, head_z), (0.12, 0.25, 0.25), mats["steel"], bevel=0.015)
    elif variant == "flanged_maul":
        add_cylinder(f"{spec['id']}_core", (0, 0, head_z), 0.070, 0.18, mats["dark_steel"], vertices=18, bevel=0.004)
        for i, ang in enumerate([0, math.pi / 2, math.pi, 3 * math.pi / 2]):
            add_cube(f"{spec['id']}_heavy_flange_{i}", (math.cos(ang) * 0.065, math.sin(ang) * 0.010, head_z), (0.030 if i % 2 == 0 else 0.09, 0.10 if i % 2 == 0 else 0.030, 0.19), mats["steel"], bevel=0.003)
    else:
        add_cylinder(f"{spec['id']}_mace_core", (0, 0, head_z), 0.052, 0.16, mats["dark_steel"], vertices=16, bevel=0.003)
        for i in range(6):
            a = i * math.tau / 6
            add_cube(f"{spec['id']}_flange_{i}", (math.cos(a) * 0.058, math.sin(a) * 0.012, head_z), (0.022, 0.082, 0.15), mats["steel"], bevel=0.002)
    add_cylinder(f"{spec['id']}_pommel", (0, 0, -0.04), 0.035, 0.055, mats["brass"], vertices=16, bevel=0.002)


def build_flail(spec, mats):
    add_cylinder(f"{spec['id']}_grip", (-0.16, 0, 0.34), 0.025, 0.62, mats["leather"], vertices=18, bevel=0.002)
    chain = [(-0.16, 0, 0.66), (-0.05, 0, 0.78), (0.08, 0, 0.72), (0.19, 0, 0.58)]
    for i in range(len(chain) - 1):
        add_cylinder_between(f"{spec['id']}_chain_link_{i}", chain[i], chain[i + 1], 0.008, mats["dark_steel"], vertices=8)
        add_torus(f"{spec['id']}_visible_chain_ring_{i}", chain[i + 1], 0.025, 0.004, mats["steel"], rotation=(math.radians(90), 0, 0))
    add_sphere(f"{spec['id']}_weighted_ball", chain[-1], 0.075, mats["dark_steel"], segments=24)
    for i, a in enumerate([0, math.pi / 2, math.pi, 3 * math.pi / 2]):
        add_cylinder_between(f"{spec['id']}_ball_spike_{i}", chain[-1], (chain[-1][0] + math.cos(a) * 0.12, 0, chain[-1][2] + math.sin(a) * 0.12), 0.009, mats["steel"], vertices=8)


def build_staff(spec, mats):
    add_cylinder(f"{spec['id']}_long_staff", (0, 0, 0.72), 0.023, 1.44, mats["ash_wood"], vertices=20, bevel=0.002)
    for z in [0.08, 0.72, 1.36]:
        add_cylinder(f"{spec['id']}_wrap_{z:.2f}", (0, 0, z), 0.028, 0.055, mats["leather"], vertices=18)
    add_cylinder(f"{spec['id']}_top_cap", (0, 0, 1.47), 0.033, 0.040, mats["brass"], vertices=18)
    add_cylinder(f"{spec['id']}_bottom_cap", (0, 0, -0.03), 0.033, 0.040, mats["brass"], vertices=18)


def build_shield_weapon(spec, mats):
    add_cylinder(f"{spec['id']}_round_shield_disc", (-0.14, 0, 0.54), 0.26, 0.060, mats["dark_steel"], vertices=64, axis="Y", bevel=0.003)
    add_cylinder(f"{spec['id']}_shield_inner_face", (-0.14, -0.036, 0.54), 0.235, 0.010, mats["leather"], vertices=64, axis="Y")
    add_cylinder(f"{spec['id']}_shield_boss", (-0.14, -0.072, 0.54), 0.080, 0.040, mats["brass"], vertices=32, axis="Y", bevel=0.003)
    # Full, separate one-handed arming sword to avoid the prior missing-sidearm read.
    sx = 0.30
    sidearm = add_blade(f"{spec['id']}_one_handed_sword_blade", 0.56, 0.046, 0.015, mats["steel"], base_z=0.40)
    sidearm.location.x += sx
    add_guard_at(f"{spec['id']}_one_handed_sword_guard", sx, 0.385, 0.18, mats, radius=0.011)
    add_cylinder(f"{spec['id']}_one_handed_sword_grip", (sx, 0, 0.235), 0.022, 0.26, mats["leather"], vertices=18, bevel=0.002)
    add_sphere(f"{spec['id']}_one_handed_sword_pommel", (sx, 0, 0.075), 0.034, mats["brass"], segments=16)


def build_firearm(spec, mats, variant):
    barrel_len = 0.64 if variant == "revolver" else 0.42
    add_cylinder(f"{spec['id']}_barrel", (0.12, 0, 0.58), 0.034, barrel_len, mats["black_iron"], vertices=24, axis="X", bevel=0.003)
    add_cylinder(f"{spec['id']}_muzzle", (0.12 + barrel_len / 2, 0, 0.58), 0.046, 0.045, mats["steel"], vertices=24, axis="X", bevel=0.002)
    if variant == "revolver":
        # Cheat the cylinder face toward the review camera: category readability
        # matters more than physically perfect side projection for this proxy.
        add_cylinder(f"{spec['id']}_large_visible_chambered_revolver_cylinder", (-0.17, -0.060, 0.55), 0.125, 0.035, mats["dark_steel"], vertices=48, axis="Y", bevel=0.004)
        for i in range(6):
            a = i * math.tau / 6
            add_cylinder(f"{spec['id']}_black_chamber_port_{i}", (-0.17 + math.cos(a) * 0.063, -0.084, 0.55 + math.sin(a) * 0.063), 0.018, 0.010, mats["shadow"], vertices=18, axis="Y", bevel=0.001)
        add_cylinder(f"{spec['id']}_brass_cylinder_axis_pin", (-0.17, -0.092, 0.55), 0.026, 0.012, mats["brass"], vertices=18, axis="Y", bevel=0.001)
        add_cube(f"{spec['id']}_revolver_frame_bridge", (-0.04, 0, 0.55), (0.20, 0.050, 0.070), mats["dark_steel"], bevel=0.006)
        add_cube(f"{spec['id']}_trigger", (-0.10, -0.030, 0.335), (0.026, 0.025, 0.075), mats["black_iron"], bevel=0.004)
    else:
        add_cube(f"{spec['id']}_single_shot_breech_block", (-0.14, 0, 0.55), (0.16, 0.10, 0.12), mats["dark_steel"], bevel=0.008)
    add_cylinder_between(f"{spec['id']}_angled_grip", (-0.17, 0, 0.46), (-0.27, 0, 0.18), 0.035, mats["leather"], vertices=16)
    add_cube(f"{spec['id']}_trigger_guard", (-0.13, -0.005, 0.39), (0.10, 0.020, 0.070), mats["brass"], bevel=0.004)
    add_prism(f"{spec['id']}_hammer", [(-0.22, 0.66), (-0.28, 0.75), (-0.17, 0.70)], 0.018, mats["steel"], bevel=0.002)


def build_chain_hook(spec, mats):
    pts = [(-0.52, 0, 0.30), (-0.39, 0, 0.48), (-0.24, 0, 0.38), (-0.08, 0, 0.57), (0.09, 0, 0.45), (0.25, 0, 0.60)]
    for i in range(len(pts) - 1):
        add_cylinder_between(f"{spec['id']}_chain_segment_{i}", pts[i], pts[i + 1], 0.010, mats["dark_steel"], vertices=8)
        add_torus(f"{spec['id']}_chain_ring_{i}", pts[i], 0.031, 0.005, mats["steel"], rotation=(math.radians(90), 0, 0))
    # Large J-shaped grappling hook built from round metal segments so it stops
    # reading as a flat axe/flag.
    hook_path = [(0.25, 0, 0.57), (0.38, 0, 0.74), (0.58, 0, 0.70), (0.65, 0, 0.54), (0.58, 0, 0.38)]
    for i in range(len(hook_path) - 1):
        add_cylinder_between(f"{spec['id']}_round_grappling_hook_segment_{i}", hook_path[i], hook_path[i + 1], 0.024, mats["steel"], vertices=14)
    add_prism(f"{spec['id']}_open_hook_inward_barb", [(0.58, 0.38), (0.42, 0.46), (0.55, 0.56)], 0.030, mats["brass"], bevel=0.003)
    add_prism(f"{spec['id']}_outer_hook_point", [(0.65, 0.54), (0.76, 0.50), (0.63, 0.43)], 0.026, mats["steel"], bevel=0.002)
    add_sphere(f"{spec['id']}_hook_socket", (0.25, 0, 0.57), 0.040, mats["brass"], segments=18)
    add_sphere(f"{spec['id']}_counterweight", pts[0], 0.060, mats["black_iron"], segments=18)


def build_gauntlets(spec, mats):
    for side, x in [("left", -0.13), ("right", 0.13)]:
        add_cube(f"{spec['id']}_{side}_wrist_bracer", (x, 0, 0.20), (0.13, 0.07, 0.18), mats["dark_steel"], bevel=0.010)
        add_cube(f"{spec['id']}_{side}_palm_plate", (x, 0, 0.37), (0.16, 0.08, 0.16), mats["leather"], bevel=0.012)
        for i, dx in enumerate([-0.055, -0.018, 0.018, 0.055]):
            add_cube(f"{spec['id']}_{side}_knuckle_plate_{i}", (x + dx, -0.055, 0.48), (0.030, 0.030, 0.055), mats["brass"], bevel=0.006)
        add_cube(f"{spec['id']}_{side}_thumb_guard", (x + (0.095 if side == "left" else -0.095), -0.03, 0.38), (0.040, 0.045, 0.12), mats["steel"], bevel=0.006)


def build_arcane_focus(spec, mats):
    add_cylinder(f"{spec['id']}_short_shaft", (0, 0, 0.38), 0.022, 0.68, mats["black_iron"], vertices=18, bevel=0.002)
    add_sphere(f"{spec['id']}_focus_core", (0, 0, 0.86), 0.095, mats["arcane"], segments=32)
    add_torus(f"{spec['id']}_vertical_focus_ring", (0, 0, 0.86), 0.145, 0.006, mats["brass"], rotation=(math.radians(90), 0, 0))
    add_torus(f"{spec['id']}_cross_focus_ring", (0, 0, 0.86), 0.125, 0.005, mats["steel"], rotation=(0, math.radians(90), 0))
    add_prism(f"{spec['id']}_lower_finial", [(0, -0.06), (0.040, 0.10), (-0.040, 0.10)], 0.018, mats["brass"], bevel=0.002)


def dispatch_build(spec, mats):
    bp = spec["blueprint"]
    if bp in {"arming_sword", "longsword", "greatsword", "curved_greatblade", "saber"}:
        build_sword(spec, mats, bp)
    elif bp == "dual_daggers":
        build_dual_daggers(spec, mats)
    elif bp in {"spear", "glaive", "halberd"}:
        build_polearm(spec, mats, bp)
    elif bp in {"battle_axe", "dane_axe"}:
        build_axe(spec, mats, bp)
    elif bp == "hammer":
        build_blunt(spec, mats, "hammer")
    elif bp in {"mace", "flanged_maul"}:
        build_blunt(spec, mats, bp)
    elif bp == "chain_flail":
        build_flail(spec, mats)
    elif bp == "staff":
        build_staff(spec, mats)
    elif bp == "shield_weapon":
        build_shield_weapon(spec, mats)
    elif bp in {"hand_cannon", "revolver"}:
        build_firearm(spec, mats, "revolver" if bp == "revolver" else "hand_cannon")
    elif bp == "hooked_chain":
        build_chain_hook(spec, mats)
    elif bp == "gauntlets":
        build_gauntlets(spec, mats)
    elif bp == "arcane_focus":
        build_arcane_focus(spec, mats)
    else:
        raise ValueError(f"unknown blueprint {bp}")


def visible_mesh_bounds():
    coords = []
    for obj in bpy.context.scene.objects:
        if obj.type != "MESH" or obj.hide_render:
            continue
        coords.extend(obj.matrix_world @ Vector(corner) for corner in obj.bound_box)
    if not coords:
        return Vector((-0.5, -0.5, 0.0)), Vector((0.5, 0.5, 1.0))
    return Vector((min(c.x for c in coords), min(c.y for c in coords), min(c.z for c in coords))), Vector((max(c.x for c in coords), max(c.y for c in coords), max(c.z for c in coords)))


def point_camera_at(obj, target):
    direction = Vector(target) - obj.location
    obj.rotation_euler = direction.to_track_quat("-Z", "Y").to_euler()


def setup_render(samples, res_x, res_y):
    lo, hi = visible_mesh_bounds()
    center = (lo + hi) / 2
    size = hi - lo
    bpy.ops.object.light_add(type="AREA", location=(center.x - 1.0, center.y - 2.5, center.z + 1.7))
    light = bpy.context.object
    light.name = "candidate_large_softbox"
    light.data.energy = 800
    light.data.size = 3.5
    bpy.ops.object.light_add(type="POINT", location=(center.x + 0.85, center.y - 1.0, center.z + 0.55))
    rim = bpy.context.object
    rim.name = "candidate_edge_rim"
    rim.data.energy = 90
    bpy.ops.object.camera_add(location=(center.x + 0.10, center.y - 3.2, center.z + 0.05))
    cam = bpy.context.object
    cam.name = "candidate_orthographic_readability_camera"
    cam.data.type = "ORTHO"
    cam.data.ortho_scale = max(size.z * 1.24, size.x * 1.95, 0.95)
    point_camera_at(cam, center)
    bpy.context.scene.camera = cam
    bpy.context.scene.unit_settings.system = "METRIC"
    bpy.context.scene.render.engine = "BLENDER_EEVEE_NEXT"
    bpy.context.scene.eevee.taa_render_samples = samples
    bpy.context.scene.render.resolution_x = res_x
    bpy.context.scene.render.resolution_y = res_y
    bpy.context.scene.view_settings.view_transform = "Filmic"
    bpy.context.scene.view_settings.look = "Medium High Contrast"
    bpy.context.scene.view_settings.exposure = 0
    bpy.context.scene.view_settings.gamma = 1
    bpy.context.scene.world.color = (0.018, 0.018, 0.020)


def add_contact_helpers(spec):
    lo, hi = visible_mesh_bounds()
    helpers = {
        "READABILITY_bounds_min": lo,
        "READABILITY_bounds_max": hi,
        "CONTACT_primary_tip_or_strike": Vector((0, 0, hi.z)),
        "GRIP_primary": Vector((0, 0, max(lo.z + 0.16, 0.12))),
    }
    for name, loc in helpers.items():
        empty = bpy.data.objects.new(f"{spec['id']}_{name}", None)
        empty.empty_display_type = "SPHERE"
        empty.empty_display_size = 0.025
        empty.location = loc
        empty.hide_render = True
        bpy.context.collection.objects.link(empty)
    return list(helpers)


def render_one(spec, root_out: Path, source_spec_path: str, source_concept_sha: str, samples: int, res_x: int, res_y: int):
    clear_scene()
    mats = material_set()
    dispatch_build(spec, mats)
    helper_names = add_contact_helpers(spec)
    setup_render(samples, res_x, res_y)
    apply_modifiers()

    out = root_out / spec["id"]
    out.mkdir(parents=True, exist_ok=True)
    blend = out / f"{spec['id']}.source.blend"
    glb = out / f"{spec['id']}.candidate.glb"
    preview = out / f"{spec['id']}.preview.png"
    bpy.ops.wm.save_as_mainfile(filepath=str(blend))
    bpy.ops.export_scene.gltf(filepath=str(glb), export_format="GLB", export_apply=True)
    bpy.context.scene.render.filepath = str(preview)
    bpy.ops.render.render(write_still=True)

    meshes = [o for o in bpy.context.scene.objects if o.type == "MESH"]
    manifest = {
        "schema": "oathyard.blender_weapon_candidate.v1",
        "asset_id": spec["id"],
        "name": spec["name"],
        "concept_number": spec["number"],
        "source_spec": source_spec_path,
        "source_concept_sha256": source_concept_sha,
        "blueprint": spec["blueprint"],
        "concept_archetype": spec["concept_archetype"],
        "tags": spec["tags"],
        "damage_types": spec["damage_types"],
        "weight_class": spec["weight_class"],
        "parts": spec["parts"],
        "source_blend": blend.name,
        "runtime_export": glb.name,
        "preview_render": preview.name,
        "hashes": {
            "source_blend": sha256_file(blend),
            "runtime_export": sha256_file(glb),
            "preview_render": sha256_file(preview),
        },
        "metrics": {
            "mesh_objects": len(meshes),
            "vertices": sum(len(o.data.vertices) for o in meshes),
            "polygons": sum(len(o.data.polygons) for o in meshes),
        },
        "materials": sorted({slot.material.name for o in meshes for slot in o.material_slots if slot.material}),
        "contact_helpers": helper_names,
        "machine_candidate_readability": "unaudited",
        "truth_boundary": {"presentation_only": True, "truth_authoritative": False, "does_not_mutate_gameplay_truth": True},
        "not_claimed": ["owner visual acceptance", "public demo readiness", "release candidate readiness", "production asset completion", "native in-engine runtime capture"],
        "blockers": ["needs hostile visual audit", "needs UV/high-to-low bake/texture atlas", "needs native renderer import/capture", "needs physical-fidelity metadata contract", "needs owner/human acceptance"],
    }
    (out / "candidate_manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return manifest


def main():
    args = parse_args()
    spec_path = Path(args.spec)
    if not spec_path.is_absolute():
        spec_path = ROOT / spec_path
    data = json.loads(spec_path.read_text(encoding="utf-8"))
    wanted = {x.strip() for x in args.only.split(",") if x.strip()}
    weapons = [w for w in data["weapons"] if not wanted or w["id"] in wanted]
    root_out = ROOT / "assets" / "production_candidates" / args.run_id / "weapons"
    manifests = []
    for weapon in weapons:
        manifests.append(render_one(weapon, root_out, spec_path.relative_to(ROOT).as_posix(), data["source_concept_sha256"], args.samples, args.res_x, args.res_y))
    run_manifest = {
        "schema": "oathyard.blender_weapon_roster_run.v1",
        "run_id": args.run_id,
        "source_spec": spec_path.relative_to(ROOT).as_posix(),
        "source_concept_image": data["source_concept_image"],
        "source_concept_sha256": data["source_concept_sha256"],
        "candidate_count": len(manifests),
        "weapons": [{"id": m["asset_id"], "name": m["name"], "manifest": f"weapons/{m['asset_id']}/candidate_manifest.json", "preview": f"weapons/{m['asset_id']}/{m['preview_render']}"} for m in manifests],
        "truth_boundary": data["truth_boundary"],
        "not_claimed": data["not_claimed"],
    }
    run_dir = ROOT / "assets" / "production_candidates" / args.run_id
    (run_dir / "weapon_roster_run_manifest.json").write_text(json.dumps(run_manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps({"ok": True, "run_id": args.run_id, "candidate_count": len(manifests), "out": str(run_dir)}, indent=2))


if __name__ == "__main__":
    main()
