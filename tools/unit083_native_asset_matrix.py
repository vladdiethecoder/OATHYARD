#!/usr/bin/env python3
"""Unit-083 native production-renderer asset capture matrix.

Creates and validates current-run native wgpu production-renderer evidence for the
22 source-approved generated/model-candidate assets without promoting production,
owner, public-demo, release, legal, trademark, or store readiness.
"""
from __future__ import annotations

import argparse
import binascii
import hashlib
import importlib.util
import json
import math
import os
import shutil
import struct
import subprocess
import sys
import zlib
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

EXPECTED_ASSETS = [
    ("oathyard_verdict_ring", "arena"),
    ("training_yard", "arena"),
    ("bruiser_padded_plate", "armor"),
    ("fencer_light", "armor"),
    ("gambeson", "armor"),
    ("heavy_plate", "armor"),
    ("lamellar", "armor"),
    ("mail_hauberk", "armor"),
    ("bruiser_oath", "fighter"),
    ("chainbreaker", "fighter"),
    ("gate_shield", "fighter"),
    ("oathyard_writ", "fighter"),
    ("reed_sentinel", "fighter"),
    ("saltreach_duelist", "fighter"),
    ("arming_sword", "weapon"),
    ("ash_spear", "weapon"),
    ("bearded_axe", "weapon"),
    ("billhook", "weapon"),
    ("curved_sword", "weapon"),
    ("iron_maul", "weapon"),
    ("longsword", "weapon"),
    ("round_shield", "weapon"),
]
EXPECTED_ASSET_IDS = [asset_id for asset_id, _class in EXPECTED_ASSETS]
EXPECTED_TRUTH_HASH = "f17c8f76b9dfae86"
BACKEND_ID = "oathyard-native-wgpu-production-v1"
SCHEMA = "oathyard.native_asset_capture_matrix.v1"
FORBIDDEN_SUFFIXES = {".svg", ".ppm", ".pbm", ".pgm", ".xpm", ".html"}


def utc_now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def rel_to_root(path: Path, root: Path) -> str:
    try:
        return path.resolve().relative_to(root.resolve()).as_posix()
    except Exception:
        return path.as_posix()


def norm_class(value: str) -> str:
    value = str(value or "").lower()
    return {
        "arenas": "arena",
        "arena": "arena",
        "armor": "armor",
        "armors": "armor",
        "fighters": "fighter",
        "fighter": "fighter",
        "weapons": "weapon",
        "weapon": "weapon",
    }.get(value, value or "unknown")


def sanitize(value: str) -> str:
    return "".join(ch if ch.isalnum() or ch in "_-" else "_" for ch in value)


def png_dimensions(path: Path) -> tuple[int, int]:
    with path.open("rb") as f:
        header = f.read(24)
    if not header.startswith(b"\x89PNG\r\n\x1a\n"):
        raise ValueError("not a PNG file")
    if len(header) < 24 or header[12:16] != b"IHDR":
        raise ValueError("PNG missing IHDR")
    return struct.unpack(">II", header[16:24])


def paeth(a: int, b: int, c: int) -> int:
    p = a + b - c
    pa = abs(p - a)
    pb = abs(p - b)
    pc = abs(p - c)
    if pa <= pb and pa <= pc:
        return a
    if pb <= pc:
        return b
    return c


def png_pixel_metrics(path: Path) -> dict[str, Any]:
    with path.open("rb") as f:
        sig = f.read(8)
        if sig != b"\x89PNG\r\n\x1a\n":
            raise ValueError("not a PNG file")
        width = height = bit_depth = color_type = None
        idat = bytearray()
        while True:
            raw_len = f.read(4)
            if len(raw_len) < 4:
                break
            length = struct.unpack(">I", raw_len)[0]
            chunk_type = f.read(4)
            chunk = f.read(length)
            f.read(4)
            if chunk_type == b"IHDR":
                width, height = struct.unpack(">II", chunk[:8])
                bit_depth = chunk[8]
                color_type = chunk[9]
            elif chunk_type == b"IDAT":
                idat.extend(chunk)
            elif chunk_type == b"IEND":
                break
    if width is None or height is None or bit_depth is None or color_type is None:
        raise ValueError("PNG missing IHDR data")
    channels = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}.get(color_type)
    if channels is None or bit_depth != 8:
        return {"width": width, "height": height, "blank_or_black": True, "readability_score": 0.0, "contrast": 0.0, "unique_sample_count": 0, "non_black_ratio": 0.0}
    data = zlib.decompress(bytes(idat))
    bpp = channels
    stride = width * bpp
    raw_stride = stride + 1
    prev = bytearray(stride)
    luminance_sum = 0.0
    luminance_sq = 0.0
    non_black = 0
    samples: set[tuple[int, int, int]] = set()
    sample_count = 0
    sample_stride_x = max(width // 96, 1)
    sample_stride_y = max(height // 54, 1)
    for y in range(height):
        start = y * raw_stride
        ftype = data[start]
        row = bytearray(data[start + 1 : start + 1 + stride])
        for i in range(stride):
            left = row[i - bpp] if i >= bpp else 0
            up = prev[i]
            ul = prev[i - bpp] if i >= bpp else 0
            if ftype == 1:
                row[i] = (row[i] + left) & 0xFF
            elif ftype == 2:
                row[i] = (row[i] + up) & 0xFF
            elif ftype == 3:
                row[i] = (row[i] + ((left + up) // 2)) & 0xFF
            elif ftype == 4:
                row[i] = (row[i] + paeth(left, up, ul)) & 0xFF
            elif ftype != 0:
                raise ValueError(f"unsupported PNG filter {ftype}")
        prev = row
        if y % sample_stride_y != 0:
            continue
        for x in range(0, width, sample_stride_x):
            off = x * bpp
            if channels >= 3:
                r, g, b = row[off], row[off + 1], row[off + 2]
            else:
                r = g = b = row[off]
            lum = 0.2126 * r + 0.7152 * g + 0.0722 * b
            luminance_sum += lum
            luminance_sq += lum * lum
            sample_count += 1
            if r > 8 or g > 8 or b > 8:
                non_black += 1
            samples.add((r // 8, g // 8, b // 8))
    mean = luminance_sum / max(sample_count, 1)
    var = max(0.0, luminance_sq / max(sample_count, 1) - mean * mean)
    contrast = math.sqrt(var)
    non_black_ratio = non_black / max(sample_count, 1)
    unique_sample_count = len(samples)
    blank_or_black = non_black_ratio < 0.01 or contrast < 2.0 or unique_sample_count < 8
    readability = min(5.0, max(0.0, (contrast / 18.0) + (unique_sample_count / 80.0) + (non_black_ratio * 1.8)))
    return {
        "width": width,
        "height": height,
        "mean_luminance": round(mean, 3),
        "contrast": round(contrast, 3),
        "non_black_ratio": round(non_black_ratio, 5),
        "unique_sample_count": unique_sample_count,
        "blank_or_black": blank_or_black,
        "readability_score": round(readability, 3),
    }


def load_generator_module(root: Path):
    path = root / "tools" / "generate_runtime_asset_sets.py"
    spec = importlib.util.spec_from_file_location("oathyard_generate_runtime_asset_sets", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot import {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def accessor_count(gltf: dict[str, Any], accessor_index: Any) -> int:
    if accessor_index is None:
        return 0
    try:
        return int(gltf.get("accessors", [])[int(accessor_index)].get("count", 0))
    except Exception:
        return 0


def parse_gltf_metadata(gltf_path: Path, texture_paths: list[Path]) -> dict[str, Any]:
    gltf = read_json(gltf_path)
    attrs: set[str] = set()
    vertices = 0
    indices = 0
    triangles = 0
    bounds_min: list[list[float]] = []
    bounds_max: list[list[float]] = []
    for mesh in gltf.get("meshes", []) or []:
        for prim in mesh.get("primitives", []) or []:
            a = prim.get("attributes", {}) or {}
            attrs.update(str(k) for k in a)
            pos_i = a.get("POSITION")
            vertices += accessor_count(gltf, pos_i)
            acc = (gltf.get("accessors", []) or [])[int(pos_i)] if pos_i is not None else {}
            if isinstance(acc.get("min"), list) and isinstance(acc.get("max"), list) and len(acc["min"]) >= 3 and len(acc["max"]) >= 3:
                bounds_min.append([float(x) for x in acc["min"][:3]])
                bounds_max.append([float(x) for x in acc["max"][:3]])
            idx_count = accessor_count(gltf, prim.get("indices"))
            indices += idx_count if idx_count else accessor_count(gltf, pos_i)
            mode = int(prim.get("mode", 4))
            n = idx_count if idx_count else accessor_count(gltf, pos_i)
            if mode == 4:
                triangles += n // 3
            elif mode in (5, 6):
                triangles += max(0, n - 2)
    mat_channels: set[str] = set()
    for mat in gltf.get("materials", []) or []:
        pbr = mat.get("pbrMetallicRoughness", {}) or {}
        if pbr.get("baseColorTexture"):
            mat_channels.add("base_color")
        if pbr.get("metallicRoughnessTexture"):
            mat_channels.add("orm")
        if mat.get("normalTexture"):
            mat_channels.add("normal")
        if mat.get("occlusionTexture"):
            mat_channels.add("occlusion")
        if mat.get("emissiveTexture"):
            mat_channels.add("emissive")
    texture_records = []
    for tex in texture_paths:
        record: dict[str, Any] = {"path": tex.as_posix(), "exists": tex.is_file(), "sha256": "", "dimensions": [0, 0], "channel": "unknown"}
        name = tex.name
        if name.endswith("_base.png"):
            record["channel"] = "base_color"
        elif name.endswith("_normal.png"):
            record["channel"] = "normal"
        elif name.endswith("_orm.png"):
            record["channel"] = "orm"
        if tex.is_file():
            record["sha256"] = sha256_file(tex)
            try:
                record["dimensions"] = list(png_dimensions(tex))
            except Exception:
                record["dimensions"] = [0, 0]
        texture_records.append(record)
    z_depth = None
    if bounds_min and bounds_max:
        mn = [min(v[i] for v in bounds_min) for i in range(3)]
        mx = [max(v[i] for v in bounds_max) for i in range(3)]
        z_depth = mx[2] - mn[2]
    else:
        mn = mx = None
    return {
        "source_candidate_gltf": gltf_path.as_posix(),
        "source_candidate_gltf_sha256": sha256_file(gltf_path),
        "vertex_count": vertices,
        "index_count": indices,
        "triangle_count": triangles,
        "mesh_count": len(gltf.get("meshes", []) or []),
        "primitive_count": sum(len(m.get("primitives", []) or []) for m in gltf.get("meshes", []) or []),
        "material_count": len(gltf.get("materials", []) or []),
        "texture_count": len(texture_records),
        "texture_channels": sorted({str(t["channel"]) for t in texture_records}),
        "material_channels_present": sorted(mat_channels),
        "texture_records": texture_records,
        "uv_status": "present" if "TEXCOORD_0" in attrs else "missing",
        "normal_status": "present" if "NORMAL" in attrs else "missing",
        "tangent_status": "present" if "TANGENT" in attrs else "missing",
        "rig_status": "present" if gltf.get("skins") else "not_applicable_or_missing",
        "skin_weight_status": "present" if {"JOINTS_0", "WEIGHTS_0"}.issubset(attrs) else "not_applicable_or_missing",
        "animation_status": "present" if gltf.get("animations") else "not_applicable_or_missing",
        "animation_clip_count": len(gltf.get("animations", []) or []),
        "joint_count": sum(len(s.get("joints", []) or []) for s in gltf.get("skins", []) or []),
        "bounds": {"min": mn, "max": mx, "z_depth": z_depth, "nonzero_z_depth": bool(z_depth and z_depth > 0.0)},
    }


def load_assets(root: Path) -> list[dict[str, Any]]:
    path = root / "assets" / "model_candidates" / "t_73291be5" / "model_candidate_manifest.json"
    data = read_json(path)
    entries = {str(e.get("id")): e for e in data.get("entries", []) or []}
    assets = []
    missing = [asset_id for asset_id in EXPECTED_ASSET_IDS if asset_id not in entries]
    if missing:
        raise RuntimeError(f"missing expected source-approved asset entries: {missing}")
    for asset_id, asset_class in EXPECTED_ASSETS:
        entry = entries[asset_id]
        gltf_path = root / str(entry.get("runtime_gltf", f"assets/model_candidates/t_73291be5/gltf/{asset_id}.gltf"))
        texture_values = entry.get("textures") or [
            f"assets/model_candidates/t_73291be5/textures/{asset_id}_base.png",
            f"assets/model_candidates/t_73291be5/textures/{asset_id}_normal.png",
            f"assets/model_candidates/t_73291be5/textures/{asset_id}_orm.png",
        ]
        source_path = root / str(entry.get("source", f"assets_src/model_candidates/t_73291be5/{asset_class}s/{asset_id}.model_source.json"))
        source_authoring_path = ""
        if source_path.is_file():
            try:
                src = read_json(source_path)
                source_authoring_path = str((src.get("source_authoring_evidence") or {}).get("source_file") or "")
            except Exception:
                source_authoring_path = ""
        texture_paths = [root / str(v) for v in texture_values]
        meta = parse_gltf_metadata(gltf_path, texture_paths) if gltf_path.is_file() else {}
        assets.append({
            "asset_id": asset_id,
            "asset_class": asset_class,
            "manifest_kind": entry.get("kind") or entry.get("category") or asset_class,
            "source_path": rel_to_root(source_path, root),
            "source_sha256": sha256_file(source_path) if source_path.is_file() else "",
            "source_authoring_path": source_authoring_path,
            "source_authoring_sha256": sha256_file(root / source_authoring_path) if source_authoring_path and (root / source_authoring_path).is_file() else "",
            "runtime_path": rel_to_root(gltf_path, root),
            "runtime_sha256": sha256_file(gltf_path) if gltf_path.is_file() else "",
            "entry": entry,
            "gltf_metadata": meta,
            "texture_paths": texture_paths,
        })
    return assets


def role_specs(asset_class: str, asset_id: str) -> list[dict[str, Any]]:
    if asset_class == "fighter":
        camera = "fighter_closeup_01"
        context = "fighter_native_closeup_context"
    elif asset_class == "armor":
        camera = "armor_loadout_family_closeup_01"
        context = "armor_closeup_torso_context"
    elif asset_class == "weapon":
        camera = "weapon_family_closeup_01"
        context = "weapon_closeup_grip_context"
    elif asset_id == "training_yard":
        camera = "training_yard_establishing"
        context = "arena_establishing_context"
    else:
        camera = "oathyard_verdict_ring_establishing"
        context = "arena_establishing_context"
    return [
        {"role": "asset_turntable_front", "camera_mode": camera, "yaw_radians": 0.0, "class_context": context},
        {"role": "asset_turntable_three_quarter", "camera_mode": camera, "yaw_radians": 0.72, "class_context": context},
    ]


def scale_for_class(asset_class: str) -> float:
    return {"fighter": 0.95, "armor": 0.78, "weapon": 0.88, "arena": 1.05}.get(asset_class, 0.9)


def run_logged(cmd: list[str], log: Path, cwd: Path) -> int:
    log.parent.mkdir(parents=True, exist_ok=True)
    with log.open("w", encoding="utf-8") as f:
        f.write("COMMAND:" + " ".join(subprocess.list2cmdline([part]) for part in cmd) + "\n")
        f.write(f"START_UTC:{utc_now()}\n")
        proc = subprocess.run(cmd, cwd=cwd, stdout=f, stderr=subprocess.STDOUT, text=True)
        f.write(f"END_UTC:{utc_now()}\n")
        f.write(f"EXIT_CODE:{proc.returncode}\n")
    log.with_suffix(".rc").write_text(f"{proc.returncode}\n", encoding="utf-8")
    return proc.returncode


def make_post_hash_packet(root: Path, out: Path, scenario: str) -> dict[str, Any]:
    truth = out / "truth"
    logs = out / "logs"
    truth.mkdir(parents=True, exist_ok=True)
    logs.mkdir(parents=True, exist_ok=True)
    rc = run_logged(["./tools/run_duel.sh", scenario, "--out", truth.as_posix()], logs / "truth_engine.log", root)
    if rc != 0:
        raise RuntimeError(f"truth engine failed rc={rc}; log={logs / 'truth_engine.log'}")
    rc = run_logged(["./tools/replay_verify.sh", (truth / "replay.json").as_posix()], logs / "replay_verify.log", root)
    if rc != 0:
        raise RuntimeError(f"replay verification failed rc={rc}; log={logs / 'replay_verify.log'}")
    replay = read_json(truth / "replay.json")
    trace = read_json(truth / "trace.json")
    packet = {
        "schema": "oathyard.post_hash_presentation_packet.v1",
        "source": "tools/capture_native_asset_matrix.sh after run_duel + replay_verify",
        "scenario_path": scenario,
        "scenario_id": trace.get("scenario_id") or replay.get("scenario_canonical") or "unknown",
        "content_hash": trace.get("content_hash") or replay.get("content_hash"),
        "final_state_hash": trace.get("final_state_hash") or replay.get("final_state_hash"),
        "end_condition": trace.get("end_condition"),
        "end_condition_status": replay.get("end_condition_status"),
        "end_condition_winner": replay.get("end_condition_winner"),
        "replay_json": (truth / "replay.json").as_posix(),
        "trace_json": (truth / "trace.json").as_posix(),
        "replay_json_sha256": sha256_file(truth / "replay.json"),
        "trace_json_sha256": sha256_file(truth / "trace.json"),
        "duel_report_sha256": sha256_file(truth / "duel_report.md"),
        "generated_after_replay_verify": True,
        "presentation_only": True,
        "truth_mutation": False,
        "renderer_consumption_layer": "runtime_presentation",
    }
    write_json(out / "post_hash_presentation_packet.json", packet)
    return packet


def capture_matrix(root: Path, out: Path, scenario: str) -> int:
    out.mkdir(parents=True, exist_ok=True)
    logs = out / "logs"
    logs.mkdir(exist_ok=True)
    renderer_bin = root / "crates" / "oathyard_renderer" / "target" / "debug" / "oathyard-native-renderer"
    if not renderer_bin.is_file():
        rc = run_logged(["cargo", "build", "--locked", "--manifest-path", "crates/oathyard_renderer/Cargo.toml"], logs / "cargo_build_renderer.log", root)
        if rc != 0:
            raise RuntimeError(f"renderer build failed rc={rc}; log={logs / 'cargo_build_renderer.log'}")
    packet = make_post_hash_packet(root, out, scenario)
    generator = load_generator_module(root)
    assets = load_assets(root)
    mesh_dir = out / "runtime_meshes"
    mesh_dir.mkdir(exist_ok=True)
    mesh_manifest_dir = out / "mesh_manifests"
    mesh_manifest_dir.mkdir(exist_ok=True)
    capture_root = out / "captures"
    capture_root.mkdir(exist_ok=True)
    asset_manifest = root / "assets" / "manifests" / "production_candidate_visual_manifest.json"
    asset_manifest_sha = sha256_file(asset_manifest)
    rows: list[dict[str, Any]] = []
    source_commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
    for asset in assets:
        asset_id = asset["asset_id"]
        asset_class = asset["asset_class"]
        attempts = []
        asset_failures: list[str] = []
        try:
            mesh_source = Path(generator.extract_runtime_mesh(asset_id, mesh_dir))
        except SystemExit as exc:
            mesh_source = Path("")
            asset_failures.append(f"missing_runtime_asset: {exc}")
        except Exception as exc:
            mesh_source = Path("")
            asset_failures.append(f"missing_runtime_asset: {exc}")
        for role in role_specs(asset_class, asset_id):
            cap_id = f"unit083_{asset_id}_{role['role']}"
            safe_cap_id = sanitize(cap_id)
            cap_dir = capture_root / asset_id / role["role"]
            cap_dir.mkdir(parents=True, exist_ok=True)
            role_manifest = mesh_manifest_dir / f"{safe_cap_id}.mesh_manifest.json"
            command: list[str] = []
            attempt: dict[str, Any] = {
                "capture_id": cap_id,
                "capture_role": role["role"],
                "class_context_role": role["class_context"],
                "camera_mode": role["camera_mode"],
                "lighting_preset": "unit083_native_asset_key_fill_rim_contact_shadow",
                "truth_mutation": False,
                "status": "not_attempted",
                "blocker": "",
                "command": "",
                "log_path": "",
                "renderer_manifest_path": "",
                "png_path": "",
                "png_sha256": "",
                "resolution": [0, 0],
                "pixel_metrics": {},
            }
            if not mesh_source.is_file():
                attempt["status"] = "failed"
                attempt["blocker"] = "missing_runtime_asset"
                attempts.append(attempt)
                continue
            manifest = {
                "schema": "oathyard.wgpu_runtime_mesh_manifest.v1",
                "source": "tools/capture_native_asset_matrix.sh Unit-083 single source-approved asset capture",
                "capture_id": cap_id,
                "asset_id": asset_id,
                "asset_class": asset_class,
                "asset_capture_role": role["role"],
                "candidate_renderer_only": False,
                "production_seed_render": True,
                "production_ready": False,
                "truth_mutation": False,
                "meshes": [
                    {
                        "mesh_asset_id": asset_id,
                        "mesh_asset_class": asset_class,
                        "mesh_source": mesh_source.as_posix(),
                        "translation": [0.0, 0.0, 0.0],
                        "scale": scale_for_class(asset_class),
                        "yaw_radians": role["yaw_radians"],
                        "transform_baked_or_runtime": "runtime_transform_baked_into_candidate_vertex_buffer",
                        "candidate_status": "source_approved_native_renderer_candidate_not_production_ready",
                        "production_ready": False,
                        "truth_mutation": False,
                        "base_color_texture_path": f"assets/model_candidates/t_73291be5/textures/{asset_id}_base.png",
                        "normal_texture_path": f"assets/model_candidates/t_73291be5/textures/{asset_id}_normal.png",
                        "orm_texture_path": f"assets/model_candidates/t_73291be5/textures/{asset_id}_orm.png",
                    }
                ],
            }
            write_json(role_manifest, manifest)
            file_stem = f"production_renderer_{safe_cap_id}_1920x1080"
            command = [
                renderer_bin.as_posix(),
                "--packet", (out / "post_hash_presentation_packet.json").as_posix(),
                "--out", cap_dir.as_posix(),
                "--capture-id", cap_id,
                "--capture-file-stem", file_stem,
                "--camera-mode", str(role["camera_mode"]),
                "--candidate-assets", asset_id,
                "--asset-manifest-sha256", asset_manifest_sha,
                "--mesh-manifest-json", role_manifest.as_posix(),
            ]
            attempt["command"] = " ".join(subprocess.list2cmdline([part]) for part in command)
            log_path = cap_dir / "renderer.log"
            attempt["log_path"] = rel_to_root(log_path, root)
            rc = run_logged(command, log_path, root)
            png = cap_dir / f"{file_stem}.png"
            renderer_manifest = cap_dir / "production_renderer_manifest.json"
            attempt["renderer_manifest_path"] = rel_to_root(renderer_manifest, root)
            attempt["png_path"] = rel_to_root(png, root)
            if rc != 0:
                attempt["status"] = "failed"
                attempt["blocker"] = "renderer_load_failure"
                attempts.append(attempt)
                continue
            try:
                prod = read_json(renderer_manifest)
                width, height = png_dimensions(png)
                metrics = png_pixel_metrics(png)
                digest = sha256_file(png)
                attempt["resolution"] = [width, height]
                attempt["png_sha256"] = digest
                attempt["pixel_metrics"] = metrics
                if prod.get("backend_id") != BACKEND_ID:
                    raise ValueError("wrong_backend")
                if prod.get("mesh_geometry_consumed") is not True:
                    raise ValueError("mesh_geometry_not_consumed")
                if int(prod.get("mesh_asset_count") or 0) <= 0:
                    raise ValueError("mesh_asset_count_zero")
                mesh_assets = prod.get("mesh_assets") or []
                if not mesh_assets:
                    raise ValueError("mesh_metadata_missing")
                material = (mesh_assets[0].get("material_texture_summary") or {}) if isinstance(mesh_assets[0], dict) else {}
                if material.get("material_texture_binding") is not True:
                    raise ValueError("missing_material_textures")
                if width != 1920 or height != 1080:
                    raise ValueError("wrong_resolution")
                if metrics.get("blank_or_black") is True:
                    raise ValueError("black_blank_capture")
                attempt["status"] = "captured"
                attempt["renderer_backend_id"] = prod.get("backend_id")
                attempt["renderer_stack"] = prod.get("renderer_stack")
                attempt["mesh_geometry_consumed"] = prod.get("mesh_geometry_consumed")
                attempt["mesh_asset_count"] = prod.get("mesh_asset_count")
                attempt["mesh_assets"] = prod.get("mesh_assets")
                attempt["visual_features"] = prod.get("visual_features")
            except Exception as exc:
                attempt["status"] = "failed"
                blocker = str(exc)
                if blocker not in {"wrong_backend", "mesh_geometry_not_consumed", "mesh_asset_count_zero", "mesh_metadata_missing", "missing_material_textures", "wrong_resolution", "black_blank_capture"}:
                    blocker = "invalid_renderer_output"
                attempt["blocker"] = blocker
            attempts.append(attempt)
        captured_attempts = [a for a in attempts if a.get("status") == "captured"]
        failed_attempts = [a for a in attempts if a.get("status") != "captured"]
        success = bool(attempts) and not failed_attempts
        blockers = sorted({str(a.get("blocker")) for a in failed_attempts if a.get("blocker")})
        if success:
            blockers = ["owner_visual_acceptance_false"]
            low_res = any((rec.get("dimensions") or [0, 0])[0] < 1024 or (rec.get("dimensions") or [0, 0])[1] < 1024 for rec in asset["gltf_metadata"].get("texture_records", []))
            if low_res:
                blockers.insert(0, "candidate_texture_resolution_not_production_quality")
        row = {
            "asset_id": asset_id,
            "asset_class": asset_class,
            "source_approved_project_use": True,
            "native_production_renderer_capture_attempted": True,
            "native_production_renderer_capture_present": success,
            "production_visual_candidate": success,
            "production_ready": False,
            "owner_visual_accepted": False,
            "owner_visual_acceptance": False,
            "public_demo_ready": False,
            "release_candidate_ready": False,
            "legal_clearance": False,
            "trademark_clearance": False,
            "store_readiness": False,
            "truth_mutation": False,
            "status": "captured" if success else "blocked",
            "blockers": blockers,
            "next_smallest_unblock_step": "owner_visual_acceptance" if success else (blockers[0] if blockers else "native_renderer_capture_matrix"),
            "source_path": asset["source_path"],
            "source_sha256": asset["source_sha256"],
            "source_authoring_path": asset["source_authoring_path"],
            "source_authoring_sha256": asset["source_authoring_sha256"],
            "runtime_path": asset["runtime_path"],
            "runtime_sha256": asset["runtime_sha256"],
            "runtime_mesh_path": rel_to_root(mesh_source, root) if mesh_source.is_file() else "",
            "runtime_mesh_sha256": sha256_file(mesh_source) if mesh_source.is_file() else "",
            "mesh_geometry_consumed": success,
            "mesh_asset_count": 1 if success else 0,
            "mesh_metadata": asset["gltf_metadata"],
            "material_texture_binding": success,
            "material_texture_binding_reason": "base/normal/ORM texture channels bound by native renderer" if success else "native renderer capture did not complete",
            "material_quality_status": "candidate_material_channel_complete_not_production_quality_or_owner_accepted" if success else "unverified",
            "capture_attempts": attempts,
            "canonical_capture": captured_attempts[0] if captured_attempts else None,
            "successful_capture_count": len(captured_attempts),
            "failed_capture_count": len(failed_attempts),
        }
        rows.append(row)
    captured_assets = [row for row in rows if row["native_production_renderer_capture_present"]]
    failed_assets = [row for row in rows if not row["native_production_renderer_capture_present"]]
    per_class = {}
    for _asset_id, cls in EXPECTED_ASSETS:
        class_rows = [row for row in rows if row["asset_class"] == cls]
        per_class[cls] = {
            "expected": sum(1 for _aid, c in EXPECTED_ASSETS if c == cls),
            "attempted": len(class_rows),
            "captured": sum(1 for row in class_rows if row["native_production_renderer_capture_present"]),
            "failed": sum(1 for row in class_rows if not row["native_production_renderer_capture_present"]),
        }
    manifest = {
        "schema": SCHEMA,
        "tool": "tools/capture_native_asset_matrix.sh",
        "generated_at_utc": utc_now(),
        "current_run_evidence": True,
        "source_commit": source_commit,
        "scenario": scenario,
        "renderer_backend_id": BACKEND_ID,
        "renderer_version_backend": "oathyard-native-wgpu-production-v1 direct wgpu/Vulkan offscreen production renderer",
        "asset_count_expected": len(EXPECTED_ASSETS),
        "asset_count_attempted": len(rows),
        "asset_count_captured": len(captured_assets),
        "asset_count_failed": len(failed_assets),
        "software_candidate_blocker_closed_count": len(captured_assets),
        "per_class_counts": per_class,
        "truth_mutation": False,
        "owner_visual_acceptance": False,
        "owner_visual_accepted": False,
        "public_demo_ready": False,
        "release_candidate_ready": False,
        "legal_clearance": False,
        "trademark_clearance": False,
        "store_readiness": False,
        "production_ready_asset_count": 0,
        "fallback_visual_substitutes_allowed": False,
        "expected_assets": EXPECTED_ASSET_IDS,
        "failed_assets": [row["asset_id"] for row in failed_assets],
        "assets": rows,
    }
    write_json(out / "native_asset_capture_matrix_manifest.json", manifest)
    report = render_matrix_report(manifest)
    (out / "native_asset_capture_matrix_report.md").write_text(report, encoding="utf-8")
    if out.name == "native_asset_capture_matrix":
        for name in ["native_asset_capture_matrix_manifest.json", "native_asset_capture_matrix_report.md"]:
            shutil.copy2(out / name, out.parent / name)
    print(json.dumps({"asset_count_attempted": len(rows), "asset_count_captured": len(captured_assets), "asset_count_failed": len(failed_assets), "manifest": (out / "native_asset_capture_matrix_manifest.json").as_posix()}, sort_keys=True))
    return 0 if not failed_assets else 1


def render_matrix_report(manifest: dict[str, Any]) -> str:
    lines = [
        "# Unit-083 Native Asset Capture Matrix",
        "",
        "Status: " + ("PASSED" if manifest["asset_count_failed"] == 0 else "FAIL-CLOSED"),
        "",
        f"- schema: `{manifest['schema']}`",
        f"- source_commit: `{manifest['source_commit']}`",
        f"- renderer_backend_id: `{manifest['renderer_backend_id']}`",
        f"- asset_count_expected: `{manifest['asset_count_expected']}`",
        f"- asset_count_attempted: `{manifest['asset_count_attempted']}`",
        f"- asset_count_captured: `{manifest['asset_count_captured']}`",
        f"- asset_count_failed: `{manifest['asset_count_failed']}`",
        f"- software_candidate_blocker_closed_count: `{manifest['software_candidate_blocker_closed_count']}`",
        "- truth_mutation: `false`",
        "- owner_visual_acceptance: `false`",
        "- public_demo_ready: `false`",
        "- release_candidate_ready: `false`",
        "- production_ready_asset_count: `0`",
        "",
        "## Per-class counts",
        "",
        "| Class | Expected | Attempted | Captured | Failed |",
        "| --- | ---: | ---: | ---: | ---: |",
    ]
    for cls, counts in sorted(manifest["per_class_counts"].items()):
        lines.append(f"| `{cls}` | {counts['expected']} | {counts['attempted']} | {counts['captured']} | {counts['failed']} |")
    lines.extend(["", "## Per-asset rows", "", "| Asset | Class | Status | Captures | Blockers / next step | Canonical PNG | SHA256 |", "| --- | --- | --- | ---: | --- | --- | --- |"])
    for row in manifest["assets"]:
        canonical = row.get("canonical_capture") or {}
        blockers = ", ".join(row.get("blockers") or [])
        lines.append(f"| `{row['asset_id']}` | `{row['asset_class']}` | `{row['status']}` | {row['successful_capture_count']} | {blockers}; next=`{row['next_smallest_unblock_step']}` | `{canonical.get('png_path','')}` | `{canonical.get('png_sha256','')}` |")
    lines.append("")
    return "\n".join(lines)


def resolve_capture_path(matrix_path: Path, value: str) -> Path:
    path = Path(value)
    if not path.is_absolute():
        path = Path.cwd() / path
    return path


def validate_matrix(matrix_path: Path) -> dict[str, Any]:
    data = read_json(matrix_path)
    failures: list[str] = []
    if data.get("schema") != SCHEMA:
        failures.append(f"wrong schema: {data.get('schema')}")
    if data.get("current_run_evidence") is not True:
        failures.append("current_run_evidence is not true")
    for flag in ("truth_mutation", "owner_visual_acceptance", "owner_visual_accepted", "public_demo_ready", "release_candidate_ready", "legal_clearance", "trademark_clearance", "store_readiness"):
        if data.get(flag) is not False:
            failures.append(f"{flag} is not false")
    if int(data.get("asset_count_expected") or 0) != 22:
        failures.append("asset_count_expected is not 22")
    rows_value = data.get("assets")
    rows: list[dict[str, Any]] = rows_value if isinstance(rows_value, list) else []
    ids = [str(row.get("asset_id")) for row in rows]
    for asset_id in EXPECTED_ASSET_IDS:
        if asset_id not in ids:
            failures.append(f"missing expected asset row: {asset_id}")
    if len(rows) != 22:
        failures.append(f"asset row count {len(rows)} != 22")
    valid_assets = 0
    per_asset = []
    png_cache: dict[str, tuple[int, int, dict[str, Any], str]] = {}
    for row in rows:
        asset_id = str(row.get("asset_id", ""))
        row_failures: list[str] = []
        if row.get("production_ready") is True:
            row_failures.append("production_ready cannot be true from native capture alone")
        for flag in ("owner_visual_acceptance", "owner_visual_accepted", "public_demo_ready", "release_candidate_ready", "legal_clearance", "trademark_clearance", "store_readiness"):
            if row.get(flag) is True:
                row_failures.append(f"readiness flag promoted: {flag}")
        if row.get("native_production_renderer_capture_attempted") is not True:
            row_failures.append("asset not attempted")
        if row.get("native_production_renderer_capture_present") is True:
            if row.get("mesh_geometry_consumed") is not True:
                row_failures.append("mesh_geometry_consumed false for successful asset")
            if int(row.get("mesh_asset_count") or 0) <= 0:
                row_failures.append("mesh_asset_count missing for successful asset")
            if row.get("material_texture_binding") is not True:
                row_failures.append("material metadata/binding missing for successful asset")
            attempts_value = row.get("capture_attempts")
            attempt_rows: list[dict[str, Any]] = attempts_value if isinstance(attempts_value, list) else []
            attempts = [a for a in attempt_rows if a.get("status") == "captured"]
            if not attempts:
                row_failures.append("successful asset has no captured PNG attempt")
            for attempt in attempts:
                png_value = str(attempt.get("png_path") or "")
                suffix = Path(png_value).suffix.lower()
                if suffix in FORBIDDEN_SUFFIXES:
                    row_failures.append(f"forbidden visual substitute suffix: {suffix}")
                    continue
                if suffix != ".png":
                    row_failures.append("capture is not PNG")
                    continue
                png = resolve_capture_path(matrix_path, png_value)
                if not png.is_file():
                    row_failures.append(f"PNG missing: {png_value}")
                    continue
                try:
                    cache_key = png.resolve().as_posix()
                    cached = png_cache.get(cache_key)
                    if cached is None:
                        width, height = png_dimensions(png)
                        metrics = png_pixel_metrics(png)
                        digest = sha256_file(png)
                        png_cache[cache_key] = (width, height, metrics, digest)
                    else:
                        width, height, metrics, digest = cached
                    if [width, height] != [1920, 1080]:
                        row_failures.append(f"PNG resolution {width}x{height} != 1920x1080")
                    if attempt.get("png_sha256") and attempt.get("png_sha256") != digest:
                        row_failures.append("PNG sha256 mismatch")
                    if metrics.get("blank_or_black") is True:
                        row_failures.append("black/blank PNG")
                except Exception as exc:
                    row_failures.append(f"PNG unreadable: {exc}")
                if attempt.get("renderer_backend_id") not in (BACKEND_ID, None, ""):
                    row_failures.append("wrong renderer backend")
                if attempt.get("renderer_backend_id") in (None, ""):
                    row_failures.append("renderer backend missing")
                if attempt.get("mesh_geometry_consumed") is not True:
                    row_failures.append("capture mesh_geometry_consumed false")
                if not attempt.get("mesh_assets"):
                    row_failures.append("capture mesh/material metadata missing")
        else:
            if not row.get("blockers"):
                row_failures.append("failed asset has no exact blocker")
            for attempt in row.get("capture_attempts", []) or []:
                if attempt.get("status") == "failed" and (not attempt.get("blocker") or not attempt.get("log_path")):
                    row_failures.append("failed capture missing blocker/log path")
        if not row_failures:
            valid_assets += 1
        else:
            failures.extend(f"{asset_id}: {failure}" for failure in row_failures)
        per_asset.append({"asset_id": asset_id, "status": "valid" if not row_failures else "invalid", "failures": row_failures})
    passed = not failures
    return {
        "schema": "oathyard.native_asset_capture_matrix.validation.v1",
        "matrix": matrix_path.as_posix(),
        "passed": passed,
        "expected_asset_count": 22,
        "valid_asset_count": valid_assets,
        "failure_count": len(failures),
        "failures": failures,
        "per_asset": per_asset,
        "owner_visual_acceptance": False,
        "public_demo_ready": False,
        "release_candidate_ready": False,
        "truth_mutation": False,
    }


def write_validation_outputs(matrix: Path, out: Path, mode: str) -> int:
    out.mkdir(parents=True, exist_ok=True)
    result = validate_matrix(matrix)
    matrix_data = read_json(matrix)
    asset_rows = matrix_data.get("assets", []) if isinstance(matrix_data.get("assets"), list) else []
    captured = sum(1 for row in asset_rows if row.get("native_production_renderer_capture_present") is True)
    failed = len(asset_rows) - captured
    if mode == "qa":
        payload = {**result, "schema": "oathyard.visual_qa.unit083_native_asset_matrix.v1", "asset_capture_coverage": captured, "asset_capture_failed": failed, "production_ready_asset_count": 0}
        write_json(out / "visual_qa_report.json", payload)
        report_name = "visual_qa_report.md"
        title = "OATHYARD Visual QA - Unit-083 Native Asset Matrix"
    elif mode == "gap":
        payload = {**result, "schema": "oathyard.visual_gap_audit.unit083_native_asset_matrix.v1", "asset_native_capture_matrix_present": True, "asset_capture_coverage": captured, "asset_capture_failed": failed, "gameplay_capture_matrix_separate": True, "production_ready_asset_count": 0}
        write_json(out / "visual_gap_audit.json", payload)
        write_json(out / "visual_gap_unit083_native_asset_matrix.json", payload)
        report_name = "visual_gap_audit_report.md"
        title = "OATHYARD Visual Gap Audit - Unit-083 Native Asset Matrix"
    elif mode == "benchmark":
        payload = {**result, "schema": "oathyard.visual_benchmark.unit083_native_asset_matrix.v1", "candidate_evidence_package": True, "asset_native_capture_matrix_present": True, "asset_capture_coverage": captured, "asset_capture_failed": failed, "production_ready_asset_count": 0, "production_renderer_complete": False}
        write_json(out / "visual_benchmark_manifest.json", payload)
        report_name = "visual_benchmark_report.md"
        title = "OATHYARD Visual Benchmark - Unit-083 Native Asset Matrix"
    else:
        payload = result
        report_name = "native_asset_capture_matrix_validation_report.md"
        title = "Unit-083 Native Asset Capture Matrix Validation"
    lines = [
        f"# {title}",
        "",
        "Status: " + ("PASSED" if result["passed"] else "FAILED"),
        "",
        f"- matrix: `{matrix.as_posix()}`",
        f"- expected_asset_count: `22`",
        f"- valid_asset_count: `{result['valid_asset_count']}`",
        f"- captured_assets: `{captured}`",
        f"- failed_assets: `{failed}`",
        "- production_ready_asset_count: `0`",
        "- owner_visual_acceptance: `false`",
        "- public_demo_ready: `false`",
        "- release_candidate_ready: `false`",
        "- truth_mutation: `false`",
        "",
    ]
    if result["failures"]:
        lines.extend(["## Failures", ""] + [f"- {failure}" for failure in result["failures"]])
    else:
        lines.extend(["## Failures", "", "none"])
    (out / report_name).write_text("\n".join(lines) + "\n", encoding="utf-8")
    if mode == "gap":
        (out / "visual_gap_list.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    if mode == "benchmark":
        (out / "failed_visual_benchmark_criteria.txt").write_text("none\n" if result["passed"] else "\n".join(result["failures"]) + "\n", encoding="utf-8")
    print(json.dumps({"mode": mode, "passed": result["passed"], "valid_asset_count": result["valid_asset_count"], "out": out.as_posix()}, sort_keys=True))
    return 0 if result["passed"] else 1


def png_chunk(kind: bytes, data: bytes) -> bytes:
    return struct.pack(">I", len(data)) + kind + data + struct.pack(">I", binascii.crc32(kind + data) & 0xFFFFFFFF)


def write_test_png(path: Path, *, black: bool) -> None:
    width, height = 1920, 1080
    if black:
        row = b"\x00" + (b"\x00\x00\x00" * width)
    else:
        row_pixels = bytearray()
        for x in range(width):
            row_pixels.extend(((x * 3) % 256, (64 + x * 5) % 256, (192 - x) % 256))
        row = b"\x00" + bytes(row_pixels)
    raw = row * height
    data = b"\x89PNG\r\n\x1a\n" + png_chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)) + png_chunk(b"IDAT", zlib.compress(raw, 1)) + png_chunk(b"IEND", b"")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return
    rows = bytearray()
    for y in range(height):
        rows.append(0)
        if black:
            rows.extend(b"\x00\x00\x00" * width)
        else:
            row = bytearray()
            for x in range(width):
                row.extend(((x + y) % 256, (x * 2) % 256, (y * 3) % 256))
            rows.extend(row)
    data = b"\x89PNG\r\n\x1a\n" + png_chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)) + png_chunk(b"IDAT", zlib.compress(bytes(rows), 6)) + png_chunk(b"IEND", b"")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)


def selftest(out: Path) -> int:
    out.mkdir(parents=True, exist_ok=True)
    good_png = out / "good.png"
    black_png = out / "black.png"
    write_test_png(good_png, black=False)
    write_test_png(black_png, black=True)
    def make_row(asset_id: str, cls: str) -> dict[str, Any]:
        digest = sha256_file(good_png)
        return {
            "asset_id": asset_id,
            "asset_class": cls,
            "native_production_renderer_capture_attempted": True,
            "native_production_renderer_capture_present": True,
            "production_ready": False,
            "owner_visual_acceptance": False,
            "owner_visual_accepted": False,
            "public_demo_ready": False,
            "release_candidate_ready": False,
            "legal_clearance": False,
            "trademark_clearance": False,
            "store_readiness": False,
            "mesh_geometry_consumed": True,
            "mesh_asset_count": 1,
            "material_texture_binding": True,
            "blockers": ["owner_visual_acceptance_false"],
            "capture_attempts": [{"status": "captured", "png_path": good_png.as_posix(), "png_sha256": digest, "renderer_backend_id": BACKEND_ID, "mesh_geometry_consumed": True, "mesh_asset_count": 1, "mesh_assets": [{"mesh_asset_id": asset_id, "material_texture_summary": {"material_texture_binding": True}}]}],
        }
    base = {"schema": SCHEMA, "current_run_evidence": True, "asset_count_expected": 22, "truth_mutation": False, "owner_visual_acceptance": False, "owner_visual_accepted": False, "public_demo_ready": False, "release_candidate_ready": False, "legal_clearance": False, "trademark_clearance": False, "store_readiness": False, "assets": [make_row(a, c) for a, c in EXPECTED_ASSETS]}
    cases = []
    def run_case(name: str, mutate) -> None:
        payload = json.loads(json.dumps(base))
        mutate(payload)
        path = out / f"{name}.json"
        write_json(path, payload)
        result = validate_matrix(path)
        cases.append({"case": name, "passed": result["passed"], "failure_count": result["failure_count"], "failures": result["failures"][:5]})
    run_case("valid", lambda p: None)
    run_case("missing_asset_row", lambda p: p["assets"].pop())
    run_case("missing_png", lambda p: p["assets"][0]["capture_attempts"][0].update({"png_path": (out / "missing.png").as_posix()}))
    run_case("black_blank_png", lambda p: p["assets"][0]["capture_attempts"][0].update({"png_path": black_png.as_posix(), "png_sha256": sha256_file(black_png)}))
    run_case("wrong_backend", lambda p: p["assets"][0]["capture_attempts"][0].update({"renderer_backend_id": "software_candidate"}))
    run_case("mesh_geometry_false", lambda p: p["assets"][0].update({"mesh_geometry_consumed": False}))
    run_case("material_missing", lambda p: p["assets"][0].update({"material_texture_binding": False}))
    run_case("stale_not_current", lambda p: p.update({"current_run_evidence": False}))
    run_case("production_ready_true", lambda p: p["assets"][0].update({"production_ready": True}))
    run_case("owner_flag_true", lambda p: p["assets"][0].update({"owner_visual_acceptance": True}))
    run_case("truth_mutation_true", lambda p: p.update({"truth_mutation": True}))
    run_case("forbidden_svg", lambda p: p["assets"][0]["capture_attempts"][0].update({"png_path": (out / "fake.svg").as_posix()}))
    ok = any(c["case"] == "valid" and c["passed"] for c in cases) and all((c["case"] == "valid") == c["passed"] for c in cases)
    write_json(out / "unit083_native_asset_capture_matrix_selftest.json", {"schema": "oathyard.unit083.native_asset_matrix.selftest.v1", "passed": ok, "cases": cases})
    print(json.dumps({"passed": ok, "out": out.as_posix()}, sort_keys=True))
    return 0 if ok else 1


def main() -> int:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="cmd", required=True)
    cap = sub.add_parser("capture")
    cap.add_argument("out")
    cap.add_argument("--scenario", default="examples/duels/basic_oathyard.duel")
    val = sub.add_parser("validate")
    val.add_argument("--matrix", required=True)
    val.add_argument("--out", required=True)
    val.add_argument("--mode", choices=["qa", "gap", "benchmark", "plain"], default="plain")
    st = sub.add_parser("selftest")
    st.add_argument("out")
    args = parser.parse_args()
    root = Path.cwd()
    if args.cmd == "capture":
        return capture_matrix(root, Path(args.out), args.scenario)
    if args.cmd == "validate":
        return write_validation_outputs(Path(args.matrix), Path(args.out), args.mode)
    if args.cmd == "selftest":
        return selftest(Path(args.out))
    raise AssertionError(args.cmd)


if __name__ == "__main__":
    raise SystemExit(main())
