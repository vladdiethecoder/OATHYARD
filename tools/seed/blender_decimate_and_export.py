#!/usr/bin/env python3
"""Decimate Meshy seed assets, synthesize placeholder PBR channels, export runtime meshes."""
import bpy
import bmesh
import json
import hashlib
import struct
import zlib
import os
from pathlib import Path

def sha256(path):
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        for chunk in iter(lambda: f.read(65536), b''):
            h.update(chunk)
    return h.hexdigest()

def write_png(path, width, height, rgba_flat):
    """Write a tiny RGBA PNG."""
    def chunk(tag, data):
        out = struct.pack('>I', len(data)) + tag + data
        out += struct.pack('>I', zlib.crc32(tag + data) & 0xffffffff)
        return out
    raw = b''
    w, h, r, g, b, a = width, height, rgba_flat[0], rgba_flat[1], rgba_flat[2], rgba_flat[3]
    row = bytes([0, r, g, b, a]) * w  # filter=none, one pixel repeated
    for _ in range(h):
        raw += row
    ihdr = struct.pack('>IIBBBBB', w, h, 8, 6, 0, 0, 0)  # 8-bit RGBA
    with open(path, 'wb') as f:
        f.write(b'\x89PNG\r\n\x1a\n')
        f.write(chunk(b'IHDR', ihdr))
        f.write(chunk(b'IDAT', zlib.compress(raw)))
        f.write(chunk(b'IEND', b''))

seeds = {
    'longsword': ('weapon', [180, 175, 165, 255]),           # dark steel base
    'witness_stone': ('arena', [140, 135, 125, 255]),        # stone grey
    'gambeson': ('armor', [215, 205, 185, 255]),             # off-white linen
    'fighter_mannequin': ('fighter', [185, 145, 120, 255]),  # skin tone
}

prompts = {
    'longsword': 'European medieval longsword, realistic proportions, 1.2m total length, straight double-edged blade, cross-guard, leather-wrapped grip, disc pommel, dark steel with subtle patina',
    'witness_stone': 'Weathered stone pillar monument, medieval judicial duel witness stone, 3 meters tall, ornate carved surface with ancient runic inscriptions, worn smooth by centuries of weather',
    'gambeson': 'Padded gambeson armor jacket, quilted linen layers, medieval European, visible quilting pattern, leather strap closures, natural off-white fabric, battle-worn but intact',
    'fighter_mannequin': 'Anatomically realistic male fighting mannequin in defensive combat stance, muscular build, unarmored, linen wrap shorts, T-pose compatible',
}

texture_dir = Path('assets/textures/production_seed')
texture_dir.mkdir(parents=True, exist_ok=True)
out_dir = Path('assets/production_seed_runtime')
out_dir.mkdir(exist_ok=True)

results = []
TARGET_VERTS = 30000

for asset_id, (asset_class, base_rgba) in seeds.items():
    glb_path = Path(f'assets/source/production/{asset_id}.glb')

    # Clear
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete()
    for m in list(bpy.data.meshes):
        if m.users == 0: bpy.data.meshes.remove(m)
    for mat in list(bpy.data.materials):
        if mat.users == 0: bpy.data.materials.remove(mat)
    for img in list(bpy.data.images):
        if img.users == 0: bpy.data.images.remove(img)

    bpy.ops.import_scene.gltf(filepath=str(glb_path))
    meshes = [o for o in bpy.context.scene.objects if o.type == 'MESH']

    raw_verts = sum(len(m.data.vertices) for m in meshes)
    raw_tris = sum(len(m.data.polygons) for m in meshes)

    # Decimate each mesh
    for obj in meshes:
        bpy.context.view_layer.objects.active = obj
        obj.select_set(True)
        current_verts = len(obj.data.vertices)
        if current_verts > TARGET_VERTS:
            ratio = TARGET_VERTS / current_verts
            mod = obj.modifiers.new(name='Decimate', type='DECIMATE')
            mod.ratio = max(0.001, ratio)
            bpy.ops.object.modifier_apply(modifier='Decimate')
        obj.select_set(False)

    meshes = [o for o in bpy.context.scene.objects if o.type == 'MESH']
    total_verts = sum(len(m.data.vertices) for m in meshes)
    total_tris = sum(len(m.data.polygons) for m in meshes)

    # Manifold check
    bm = bmesh.new()
    for m in meshes:
        bm.from_mesh(m.data)
    bmesh.ops.remove_doubles(bm, verts=bm.verts[:], dist=1e-6)
    welded_boundary = sum(1 for e in bm.edges if e.is_boundary)
    welded_nonmanifold = sum(1 for e in bm.edges if not e.is_manifold)
    welded_closed = (welded_boundary == 0 and welded_nonmanifold == 0)
    bm.free()

    # Bounds
    all_positions = []
    for m in meshes:
        for v in m.data.vertices:
            all_positions.append([v.co[0], v.co[1], v.co[2]])
    if all_positions:
        bounds_min = [min(p[a] for p in all_positions) for a in range(3)]
        bounds_max = [max(p[a] for p in all_positions) for a in range(3)]
    else:
        bounds_min = [0,0,0]
        bounds_max = [0,0,0]

    # UV
    has_uv = any(len(m.data.uv_layers) > 0 for m in meshes)

    # Positions + indices
    all_pos = []
    all_idx = []
    vert_offset = 0
    for m in meshes:
        me = m.data
        me.calc_loop_triangles()
        for v in me.vertices:
            all_pos.append([v.co[0], v.co[1], v.co[2]])
        for tri in me.loop_triangles:
            all_idx.append(tri.vertices[0] + vert_offset)
            all_idx.append(tri.vertices[1] + vert_offset)
            all_idx.append(tri.vertices[2] + vert_offset)
        vert_offset += len(me.vertices)

    # Synthesize placeholder PBR textures (16x16)
    base_png = texture_dir / f'{asset_id}_base.png'
    normal_png = texture_dir / f'{asset_id}_normal.png'
    orm_png = texture_dir / f'{asset_id}_orm.png'
    write_png(base_png, 16, 16, base_rgba)
    write_png(normal_png, 16, 16, [128, 128, 255, 255])  # flat normal
    write_png(orm_png, 16, 16, [0, 128, 255, 255])  # O=0 R=0.5 M=1.0

    runtime_mesh = {
        'schema': 'oathyard.production_seed_runtime_mesh.v1',
        'asset_id': asset_id,
        'asset_class': asset_class,
        'source_glb': str(glb_path),
        'source_glb_sha256': sha256(str(glb_path)),
        'generation_tool': 'meshy-6',
        'text_prompt': prompts.get(asset_id, ''),
        'acceptance_state': 'source_approved',
        'license_status': 'meshy_commercial_use_paid_subscription',
        'decimation_applied': total_verts < raw_verts,
        'original_vertices': raw_verts,
        'original_triangles': raw_tris,
        'mesh_count': len(meshes),
        'total_vertices': total_verts,
        'total_faces': total_tris,
        'total_triangles': total_tris,
        'total_indices': len(all_idx),
        'bounds_min': bounds_min,
        'bounds_max': bounds_max,
        'material_count': 0,
        'has_uv': has_uv,
        'topology_manifold_validation_passed': welded_closed,
        'welded_boundary_edges': welded_boundary,
        'welded_nonmanifold_edges': welded_nonmanifold,
        'blender_version': bpy.app.version_string,
        'positions': all_pos,
        'indices': all_idx,
        'material_validation': {
            'base_normal_orm_present': True,
            'material_channels': ['base_color', 'normal', 'orm'],
            'image_uris': [
                str(base_png),
                str(normal_png),
                str(orm_png),
            ],
            'production_seed': True,
            'placeholder_textures': True,
        },
        'texture_hashes': {
            'base_color': sha256(str(base_png)),
            'normal': sha256(str(normal_png)),
            'orm': sha256(str(orm_png)),
        },
        'source_candidate_gltf': None,
        'production_seed_source': 'meshy_text_to_3d',
        'truth_mutation': False,
        'production_ready': False,
        'presentation_only': True,
    }

    out_path = out_dir / f'{asset_id}.mesh.json'
    out_path.write_text(json.dumps(runtime_mesh, indent=2))

    results.append({
        'asset_id': asset_id,
        'asset_class': asset_class,
        'original_verts': raw_verts,
        'decimated_verts': total_verts,
        'triangles': total_tris,
        'closed_manifold': welded_closed,
        'runtime_mesh': str(out_path),
        'runtime_mesh_sha256': sha256(str(out_path)),
    })

print(json.dumps(results, indent=2))
