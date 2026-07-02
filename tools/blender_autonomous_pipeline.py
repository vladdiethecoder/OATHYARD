#!/usr/bin/env python3
"""Headless Blender production-candidate asset generator for OATHYARD.

Run inside Blender:
  blender --background --python tools/blender_autonomous_pipeline.py -- --asset longsword --source assets/source/model_candidates/t_73291be5/weapons/longsword.model_source.json --run-id <id>

This creates candidate evidence only. It never writes canonical assets/gltf and never
claims owner/public/release readiness.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from pathlib import Path

import bpy
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[1]


def point_camera_at(obj, target: Vector) -> None:
    direction = target - obj.location
    obj.rotation_euler = direction.to_track_quat('-Z', 'Y').to_euler()


def visible_mesh_bounds():
    coords = []
    for obj in bpy.context.scene.objects:
        if obj.type != 'MESH' or obj.hide_render:
            continue
        coords.extend(obj.matrix_world @ Vector(corner) for corner in obj.bound_box)
    if not coords:
        return Vector((0, 0, 0)), Vector((1, 1, 1))
    lo = Vector((min(c.x for c in coords), min(c.y for c in coords), min(c.z for c in coords)))
    hi = Vector((max(c.x for c in coords), max(c.y for c in coords), max(c.z for c in coords)))
    return lo, hi


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


def reset_scene() -> None:
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete()


def mat(name: str, color, metallic=0.0, roughness=0.55):
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    bsdf = m.node_tree.nodes.get('Principled BSDF')
    if bsdf:
        bsdf.inputs['Base Color'].default_value = color
        bsdf.inputs['Metallic'].default_value = metallic
        bsdf.inputs['Roughness'].default_value = roughness
    return m


def add_cube(name: str, loc, scale, material, bevel=0.0):
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=loc)
    obj = bpy.context.object
    obj.name = name
    obj.dimensions = scale
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    if material:
        obj.data.materials.append(material)
    if bevel > 0:
        mod = obj.modifiers.new('readability_bevel', 'BEVEL')
        mod.width = bevel
        mod.segments = 3
        mod.affect = 'EDGES'
        obj.modifiers.new('weighted_normals', 'WEIGHTED_NORMAL')
    return obj


def add_cylinder(name: str, loc, radius, depth, material, vertices=32, rotation=(0, 0, 0), bevel=0.0):
    bpy.ops.mesh.primitive_cylinder_add(vertices=vertices, radius=radius, depth=depth, location=loc, rotation=rotation)
    obj = bpy.context.object
    obj.name = name
    if material:
        obj.data.materials.append(material)
    if bevel > 0:
        mod = obj.modifiers.new('edge_softening', 'BEVEL')
        mod.width = bevel
        mod.segments = 2
        obj.modifiers.new('weighted_normals', 'WEIGHTED_NORMAL')
    return obj


def add_blade(name: str, length: float, width: float, thickness: float, material):
    # Tapered diamond-ish blade prism along +Z with central ridge vertices.
    z0 = 0.0
    z1 = length
    w0 = width / 2
    w1 = width * 0.18
    t = thickness / 2
    verts = [
        (-w0, 0, z0), (0, -t, z0), (w0, 0, z0), (0, t, z0),
        (-w1, 0, z1), (0, -t * 0.45, z1), (w1, 0, z1), (0, t * 0.45, z1),
    ]
    faces = [
        (0, 1, 5, 4), (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7),
        (0, 3, 2, 1), (4, 5, 6, 7),
    ]
    mesh = bpy.data.meshes.new(name + '_mesh')
    mesh.from_pydata(verts, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    obj.modifiers.new('blade_weighted_normals', 'WEIGHTED_NORMAL')
    bevel = obj.modifiers.new('subtle_edge_bevel_for_readability', 'BEVEL')
    bevel.width = 0.006
    bevel.segments = 2
    # Fuller grooves as slightly darker inset raised/flat strips.
    fuller_mat = mat('darkened_fuller_shadow_steel', (0.33, 0.35, 0.35, 1), 1.0, 0.48)
    add_cube('left_fuller_shadow', (-width * 0.17, -thickness * 0.54, length * 0.48), (width * 0.08, 0.006, length * 0.58), fuller_mat, bevel=0.002)
    add_cube('right_fuller_shadow', (width * 0.17, -thickness * 0.54, length * 0.48), (width * 0.08, 0.006, length * 0.58), fuller_mat, bevel=0.002)
    return obj


def build_longsword(source: dict, out_root: Path, source_path: str) -> dict:
    reset_scene()
    asset_id = source.get('asset_id', 'longsword')
    steel = mat('cold_scratched_steel_pbr_proxy', (0.62, 0.64, 0.62, 1), 1.0, 0.34)
    dark_steel = mat('dark_oath_iron_crossguard', (0.18, 0.19, 0.18, 1), 1.0, 0.42)
    leather = mat('strained_black_leather_wrap', (0.08, 0.045, 0.03, 1), 0.0, 0.72)
    brass = mat('muted_oath_brass_wear', (0.55, 0.43, 0.22, 1), 1.0, 0.48)
    blood = mat('old_blood_edge_wash_marker', (0.19, 0.018, 0.011, 1), 0.0, 0.66)

    length_m = 1.22
    blade_len = 0.92
    blade = add_blade('longsword_straight_double_edge_contact_blade', blade_len, 0.070, 0.022, steel)
    blade.location.z = 0.23

    # Crossguard, grip, pommel.
    add_cube('longsword_crossguard_quillons', (0, 0, 0.215), (0.34, 0.052, 0.045), dark_steel, bevel=0.012)
    add_cube('crossguard_left_flared_terminal', (-0.19, 0, 0.215), (0.055, 0.065, 0.060), brass, bevel=0.014)
    add_cube('crossguard_right_flared_terminal', (0.19, 0, 0.215), (0.055, 0.065, 0.060), brass, bevel=0.014)
    add_cylinder('two_hand_grip_core', (0, 0, 0.08), 0.035, 0.24, leather, vertices=32, rotation=(0, 0, 0), bevel=0.004)
    # grip rings / wraps
    for i, z in enumerate([-.025, .015, .055, .095, .135, .175]):
        add_cylinder(f'raised_leather_wrap_ridge_{i}', (0, 0, z), 0.038, 0.010, brass if i in {0,5} else leather, vertices=32, bevel=0.002)
    add_cylinder('faceted_wheel_pommel_counterweight', (0, 0, -0.075), 0.062, 0.055, dark_steel, vertices=16, bevel=0.008)
    add_cylinder('pommel_oath_pin', (0, 0, -0.110), 0.026, 0.018, brass, vertices=16, bevel=0.003)

    # Edge wear / blood cue: tiny visible markers near one edge, presentation-only.
    for i, z in enumerate([0.44, 0.61, 0.78, 0.96]):
        add_cube(f'asymmetric_edge_wear_notch_{i}', (0.038, -0.014, z), (0.012, 0.004, 0.035), blood if i == 2 else dark_steel, bevel=0.001)

    # Add dimension/contact helper empties as named objects, hidden from render but in source.
    for name, loc in {
        'CONTACT_edge_left': (-0.041, 0, 0.70),
        'CONTACT_edge_right': (0.041, 0, 0.70),
        'CONTACT_point_tip': (0, 0, 1.15),
        'GRIP_two_hand_lower': (0, 0, 0.03),
        'GRIP_two_hand_upper': (0, 0, 0.15),
    }.items():
        empty = bpy.data.objects.new(name, None)
        empty.empty_display_type = 'SPHERE'
        empty.empty_display_size = 0.025
        empty.location = loc
        empty.hide_render = True
        bpy.context.collection.objects.link(empty)

    # Camera/light for preview. Use an orthographic auto-fit so the full weapon
    # from pommel to tip is visible; prior fixed perspective framing cropped the blade.
    bpy.ops.object.light_add(type='AREA', location=(-1.2, -2.8, 2.5))
    light = bpy.context.object
    light.name = 'large_softbox_cold_oath_light'
    light.data.energy = 900
    light.data.size = 3.8
    bpy.ops.object.light_add(type='POINT', location=(0.75, -1.0, 1.25))
    rim = bpy.context.object
    rim.name = 'small_edge_highlight_rim_light'
    rim.data.energy = 85
    lo, hi = visible_mesh_bounds()
    center = (lo + hi) / 2
    size = hi - lo
    bpy.ops.object.camera_add(location=(center.x + 0.34, center.y - 3.2, center.z + 0.04))
    camera = bpy.context.object
    camera.name = 'full_longsword_readability_camera'
    camera.data.type = 'ORTHO'
    camera.data.ortho_scale = max(size.z * 1.22, size.x * 2.25, 1.45)
    point_camera_at(camera, center)
    bpy.context.scene.camera = camera

    # Set origin/units and render settings.
    bpy.context.scene.unit_settings.system = 'METRIC'
    bpy.context.scene.render.engine = 'BLENDER_EEVEE_NEXT'
    bpy.context.scene.eevee.taa_render_samples = 96
    bpy.context.scene.render.resolution_x = 1080
    bpy.context.scene.render.resolution_y = 1600
    bpy.context.scene.view_settings.view_transform = 'Filmic'
    bpy.context.scene.view_settings.look = 'Medium High Contrast'
    bpy.context.scene.view_settings.exposure = 0
    bpy.context.scene.view_settings.gamma = 1

    # Apply modifiers for exported mesh fidelity.
    for obj in bpy.context.scene.objects:
        bpy.ops.object.select_all(action='DESELECT')
        if obj.type == 'MESH':
            bpy.context.view_layer.objects.active = obj
            obj.select_set(True)
            for mod in list(obj.modifiers):
                try:
                    bpy.ops.object.modifier_apply(modifier=mod.name)
                except Exception:
                    pass
            obj.select_set(False)

    out_root.mkdir(parents=True, exist_ok=True)
    blend = out_root / f'{asset_id}.source.blend'
    glb = out_root / f'{asset_id}.candidate.glb'
    preview = out_root / f'{asset_id}.preview.png'
    bpy.ops.wm.save_as_mainfile(filepath=str(blend))
    bpy.ops.export_scene.gltf(filepath=str(glb), export_format='GLB', export_apply=True)
    bpy.context.scene.render.filepath = str(preview)
    bpy.ops.render.render(write_still=True)

    mesh_objs = [o for o in bpy.context.scene.objects if o.type == 'MESH']
    verts = sum(len(o.data.vertices) for o in mesh_objs)
    polys = sum(len(o.data.polygons) for o in mesh_objs)
    manifest = {
        'schema': 'oathyard.blender_production_candidate.v1',
        'asset_id': asset_id,
        'kind': 'weapon',
        'source_json': source_path,
        'source_blend': blend.name,
        'runtime_export': glb.name,
        'preview_render': preview.name,
        'hashes': {
            'source_blend': sha256_file(blend),
            'runtime_export': sha256_file(glb),
            'preview_render': sha256_file(preview),
        },
        'metrics': {'mesh_objects': len(mesh_objs), 'vertices': verts, 'polygons': polys},
        'materials': [m.name for m in bpy.data.materials],
        'contact_helpers': ['CONTACT_edge_left','CONTACT_edge_right','CONTACT_point_tip','GRIP_two_hand_lower','GRIP_two_hand_upper'],
        'truth_boundary': {'presentation_only': True, 'truth_authoritative': False, 'does_not_mutate_gameplay_truth': True},
        'not_claimed': ['owner visual acceptance','public demo readiness','release candidate readiness','production asset completion','external Khronos validation','native in-engine runtime capture'],
        'blockers': ['needs hostile visual audit', 'needs true high-to-low bake/UV texture atlas', 'needs native renderer import/capture', 'needs owner/human acceptance'],
    }
    (out_root / 'candidate_manifest.json').write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n', encoding='utf-8')
    return manifest


def parse_args():
    import sys
    argv = sys.argv
    if '--' in argv:
        argv = argv[argv.index('--') + 1:]
    else:
        argv = []
    p = argparse.ArgumentParser()
    p.add_argument('--asset', default='longsword')
    p.add_argument('--source', required=True)
    p.add_argument('--run-id', default='manual')
    p.add_argument('--out-root', default=None)
    return p.parse_args(argv)


if __name__ == '__main__':
    args = parse_args()
    source = json.loads((ROOT / args.source).read_text(encoding='utf-8') if not Path(args.source).is_absolute() else Path(args.source).read_text(encoding='utf-8'))
    out = Path(args.out_root) if args.out_root else ROOT / 'assets' / 'production_candidates' / args.run_id / 'weapons' / args.asset
    if not out.is_absolute():
        out = ROOT / out
    manifest = build_longsword(source, out, str(Path(args.source).as_posix()))
    print(json.dumps({'ok': True, 'out': str(out), 'manifest': manifest}, indent=2))
