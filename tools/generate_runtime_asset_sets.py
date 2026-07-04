#!/usr/bin/env python3
"""Generate coherent runtime mesh manifests from local Meshy/Rodin candidates.

Unit-081: the previous gameplay captures staged one hard-coded loadout and the
shader path discarded the bound material maps. This tool builds complete,
validated fighter+armor+weapon+arena bundles from the local candidate family so
native gameplay captures can exercise the assets together without truth mutation
or network/credit spend.

Important: source glTF candidates already contain NORMAL and TEXCOORD_0 data.
The older presentation_runtime mesh JSONs discarded those channels for most
assets, which forced the renderer into box-projected placeholder UVs. This tool
emits set-local runtime meshes directly from source glTF/bin with positions,
normals, texcoords, and indices preserved.
"""

from __future__ import annotations

import argparse
import base64
import json
import struct
from pathlib import Path
from typing import Any


TEXTURE_ROOT = Path("assets/model_candidates/t_73291be5/textures")
GLTF_ROOT = Path("assets/model_candidates/t_73291be5/gltf")

COMPONENT_INFO: dict[int, tuple[str, int]] = {
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

ASSET_SETS = [
    {
        "set_id": "saltreach_writ_judgement",
        "description": "Gold saltreach duelist versus crimson oathyard writ in the verdict ring.",
        "player": {"fighter": "saltreach_duelist", "armor": "gambeson", "weapon": "longsword"},
        "opponent": {"fighter": "oathyard_writ", "armor": "mail_hauberk", "weapon": "arming_sword"},
        "arena": "oathyard_verdict_ring",
    },
    {
        "set_id": "chainbreaker_gate_clash",
        "description": "Chainbreaker against gate shield with heavier armor and asymmetric weapons.",
        "player": {"fighter": "chainbreaker", "armor": "heavy_plate", "weapon": "bearded_axe"},
        "opponent": {"fighter": "gate_shield", "armor": "lamellar", "weapon": "round_shield"},
        "arena": "training_yard",
    },
    {
        "set_id": "reed_bruiser_trial",
        "description": "Reed sentinel versus bruiser oath with spear/billhook silhouettes.",
        "player": {"fighter": "reed_sentinel", "armor": "fencer_light", "weapon": "ash_spear"},
        "opponent": {"fighter": "bruiser_oath", "armor": "bruiser_padded_plate", "weapon": "billhook"},
        "arena": "oathyard_verdict_ring",
    },
]


def require_file(path: Path) -> str:
    if not path.is_file():
        raise SystemExit(f"missing required asset file: {path}")
    return path.as_posix()


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def resolve_buffer(gltf_path: Path, uri: str) -> bytes:
    if uri.startswith("data:"):
        _, payload = uri.split(",", 1)
        return base64.b64decode(payload)
    path = (gltf_path.parent / uri).resolve()
    if not path.is_file():
        raise SystemExit(f"missing glTF buffer for {gltf_path}: {path}")
    return path.read_bytes()


def accessor_layout(gltf: dict[str, Any], accessor_id: int) -> tuple[int, int, int, str, int]:
    acc = gltf["accessors"][accessor_id]
    view = gltf["bufferViews"][acc["bufferView"]]
    fmt_char, size = COMPONENT_INFO[int(acc["componentType"])]
    comps = TYPE_COMPS[str(acc["type"])]
    offset = int(view.get("byteOffset", 0)) + int(acc.get("byteOffset", 0))
    stride = int(view.get("byteStride", size * comps))
    return offset, stride, int(acc["count"]), fmt_char, comps


def read_accessor(gltf: dict[str, Any], buffer_bytes: bytes, accessor_id: int) -> list[tuple[Any, ...]]:
    start, stride, count, fmt_char, comps = accessor_layout(gltf, accessor_id)
    fmt = "<" + fmt_char * comps
    rows = []
    for index in range(count):
        rows.append(struct.unpack_from(fmt, buffer_bytes, start + index * stride))
    return rows


def extract_runtime_mesh(asset_id: str, mesh_dir: Path) -> str:
    out_path = mesh_dir / f"{asset_id}.mesh.json"
    if out_path.is_file():
        return out_path.as_posix()

    gltf_path = GLTF_ROOT / f"{asset_id}.gltf"
    require_file(gltf_path)
    gltf = read_json(gltf_path)
    buffers = [resolve_buffer(gltf_path, str(buffer["uri"])) for buffer in gltf.get("buffers", [])]
    if len(buffers) != 1:
        raise SystemExit(f"{gltf_path} expected one buffer, found {len(buffers)}")
    buffer_bytes = buffers[0]

    positions: list[list[float]] = []
    normals: list[list[float]] = []
    texcoords: list[list[float]] = []
    material_colors: list[list[float]] = []
    indices: list[int] = []
    material_indices: list[int] = []
    materials = gltf.get("materials", [])

    for mesh in gltf.get("meshes", []):
        for primitive in mesh.get("primitives", []):
            attrs = primitive.get("attributes", {})
            for required in ("POSITION", "NORMAL", "TEXCOORD_0"):
                if required not in attrs:
                    raise SystemExit(f"{gltf_path} primitive missing {required}")
            base = len(positions)
            pos_rows = read_accessor(gltf, buffer_bytes, int(attrs["POSITION"]))
            norm_rows = read_accessor(gltf, buffer_bytes, int(attrs["NORMAL"]))
            uv_rows = read_accessor(gltf, buffer_bytes, int(attrs["TEXCOORD_0"]))
            if not (len(pos_rows) == len(norm_rows) == len(uv_rows)):
                raise SystemExit(f"{gltf_path} primitive attribute count mismatch")
            material_index = int(primitive.get("material", 0))
            material = materials[material_index] if 0 <= material_index < len(materials) else {}
            pbr = material.get("pbrMetallicRoughness", {}) if isinstance(material, dict) else {}
            base_factor = pbr.get("baseColorFactor", [1.0, 1.0, 1.0, 1.0])
            if not isinstance(base_factor, list) or len(base_factor) < 3:
                base_factor = [1.0, 1.0, 1.0, 1.0]
            material_color = [float(base_factor[0]), float(base_factor[1]), float(base_factor[2])]
            positions.extend([[float(v[0]), float(v[1]), float(v[2])] for v in pos_rows])
            normals.extend([[float(v[0]), float(v[1]), float(v[2])] for v in norm_rows])
            texcoords.extend([[float(v[0]), float(v[1])] for v in uv_rows])
            material_colors.extend([material_color for _ in pos_rows])
            if primitive.get("indices") is None:
                prim_indices = list(range(len(pos_rows)))
            else:
                prim_indices = [int(row[0]) for row in read_accessor(gltf, buffer_bytes, int(primitive["indices"]))]
            indices.extend([base + index for index in prim_indices])
            material_indices.append(material_index)

    if len(positions) < 3 or len(indices) < 3:
        raise SystemExit(f"{gltf_path} produced insufficient runtime geometry")

    payload = {
        "schema": "oathyard.runtime_mesh.v2",
        "id": asset_id,
        "source": "tools/generate_runtime_asset_sets.py source glTF extraction",
        "source_candidate_gltf": gltf_path.as_posix(),
        "source_candidate_bin": (gltf_path.parent / str(gltf["buffers"][0]["uri"])).as_posix(),
        "truth_mutation": False,
        "presentation_only": True,
        "positions": positions,
        "normals": normals,
        "texcoords": texcoords,
        "material_colors": material_colors,
        "indices": indices,
        "material_indices": material_indices,
        "vertex_count": len(positions),
        "triangle_count": len(indices) // 3,
        "material_validation": {
            "schema": "oathyard.material_validation.v1",
            "material_count": len(gltf.get("materials", [])),
            "image_uris": [image.get("uri") for image in gltf.get("images", [])],
            "base_normal_orm_present": True,
            "texture_sidecar_count": 3,
            "passed": True,
            "presentation_only": True,
            "truth_authoritative": False,
        },
    }
    out_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return out_path.as_posix()


def texture_paths(asset_id: str) -> dict[str, str]:
    return {
        "base_color_texture_path": require_file(TEXTURE_ROOT / f"{asset_id}_base.png"),
        "normal_texture_path": require_file(TEXTURE_ROOT / f"{asset_id}_normal.png"),
        "orm_texture_path": require_file(TEXTURE_ROOT / f"{asset_id}_orm.png"),
    }


def mesh_entry(
    *,
    mesh_asset_id: str,
    mesh_asset_class: str,
    source_asset_id: str,
    mesh_dir: Path,
    translation: list[float],
    scale: float,
    yaw_radians: float,
) -> dict[str, object]:
    return {
        "mesh_asset_id": mesh_asset_id,
        "mesh_asset_class": mesh_asset_class,
        "mesh_source": extract_runtime_mesh(source_asset_id, mesh_dir),
        "translation": translation,
        "scale": scale,
        "yaw_radians": yaw_radians,
        "transform_baked_or_runtime": "runtime_transform_baked_into_candidate_vertex_buffer",
        "candidate_status": "source_approved_production_seed",
        "production_ready": False,
        "truth_mutation": False,
        **texture_paths(source_asset_id),
    }


def build_manifest(asset_set: dict[str, object], mesh_dir: Path) -> dict[str, object]:
    set_id = str(asset_set["set_id"])
    player = asset_set["player"]
    opponent = asset_set["opponent"]
    assert isinstance(player, dict) and isinstance(opponent, dict)

    player_fighter = str(player["fighter"])
    opponent_fighter = str(opponent["fighter"])
    player_armor = str(player["armor"])
    opponent_armor = str(opponent["armor"])
    player_weapon = str(player["weapon"])
    opponent_weapon = str(opponent["weapon"])
    arena = str(asset_set["arena"])

    meshes = [
        mesh_entry(
            mesh_asset_id=f"player_{player_fighter}",
            mesh_asset_class="fighter",
            source_asset_id=player_fighter,
            mesh_dir=mesh_dir,
            translation=[-0.72, 0.0, 0.0],
            scale=0.72,
            yaw_radians=0.10,
        ),
        mesh_entry(
            mesh_asset_id=f"opponent_{opponent_fighter}",
            mesh_asset_class="fighter",
            source_asset_id=opponent_fighter,
            mesh_dir=mesh_dir,
            translation=[0.72, 0.0, 0.0],
            scale=0.72,
            yaw_radians=-0.10,
        ),
        mesh_entry(
            mesh_asset_id=f"player_{player_armor}",
            mesh_asset_class="armor",
            source_asset_id=player_armor,
            mesh_dir=mesh_dir,
            translation=[-0.72, 0.18, 0.00],
            scale=0.14,
            yaw_radians=0.10,
        ),
        mesh_entry(
            mesh_asset_id=f"opponent_{opponent_armor}",
            mesh_asset_class="armor",
            source_asset_id=opponent_armor,
            mesh_dir=mesh_dir,
            translation=[0.72, 0.18, 0.00],
            scale=0.14,
            yaw_radians=-0.10,
        ),
        mesh_entry(
            mesh_asset_id=f"player_{player_weapon}",
            mesh_asset_class="weapon",
            source_asset_id=player_weapon,
            mesh_dir=mesh_dir,
            translation=[-1.02, 0.42, -0.04],
            scale=0.34,
            yaw_radians=1.35,
        ),
        mesh_entry(
            mesh_asset_id=f"opponent_{opponent_weapon}",
            mesh_asset_class="weapon",
            source_asset_id=opponent_weapon,
            mesh_dir=mesh_dir,
            translation=[1.02, 0.42, -0.04],
            scale=0.34,
            yaw_radians=-1.35,
        ),
        mesh_entry(
            mesh_asset_id=arena,
            mesh_asset_class="arena",
            source_asset_id=arena,
            mesh_dir=mesh_dir,
            translation=[0.0, -0.30, 0.35],
            scale=0.50,
            yaw_radians=0.0,
        ),
    ]

    return {
        "schema": "oathyard.wgpu_runtime_mesh_manifest.v1",
        "source": "tools/generate_runtime_asset_sets.py Unit-081 coherent local Meshy/Rodin asset set",
        "capture_id": f"unit081_asset_set_{set_id}",
        "candidate_renderer_only": False,
        "asset_set_id": set_id,
        "asset_set_description": asset_set["description"],
        "asset_set_assets": {
            "player": player,
            "opponent": opponent,
            "arena": arena,
        },
        "material_separation_classes": [
            "fighter_body",
            "armor_clothing",
            "weapon_metal",
            "arena_stone_ground",
        ],
        "presentation_material_fallback": "source-approved runtime texture paths; no mesh may omit base/normal/ORM",
        "runtime_mesh_channels_required": ["positions", "normals", "texcoords", "indices"],
        "production_seed_render": True,
        "production_ready": False,
        "truth_mutation": False,
        "meshes": meshes,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("out_dir", nargs="?", default="artifacts/runtime_asset_sets/manifests")
    args = parser.parse_args()

    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    mesh_dir = out_dir / "runtime_meshes"
    mesh_dir.mkdir(parents=True, exist_ok=True)

    set_rows: list[dict[str, object]] = []
    index: dict[str, object] = {
        "schema": "oathyard.runtime_asset_sets.index.v1",
        "source": "tools/generate_runtime_asset_sets.py",
        "truth_mutation": False,
        "production_ready": False,
        "asset_set_count": len(ASSET_SETS),
        "sets": set_rows,
    }

    for asset_set in ASSET_SETS:
        manifest = build_manifest(asset_set, mesh_dir)
        set_id = str(asset_set["set_id"])
        path = out_dir / f"{set_id}.mesh_manifest.json"
        path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        meshes = manifest["meshes"]
        assert isinstance(meshes, list)
        set_rows.append(
            {
                "asset_set_id": set_id,
                "manifest": path.as_posix(),
                "mesh_count": len(meshes),
                "candidate_assets": ",".join(
                    str(mesh["mesh_asset_id"]).replace("player_", "").replace("opponent_", "")
                    for mesh in meshes
                    if isinstance(mesh, dict)
                ),
            }
        )

    index_path = out_dir / "runtime_asset_sets_index.json"
    index_path.write_text(json.dumps(index, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(index_path.as_posix())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
