#!/usr/bin/env python3
"""
Add deterministic glTF tangent attributes to OATHYARD model-candidate runtime meshes.

This is candidate evidence only. It does not claim external DCC/Khronos validation,
production renderer capture, owner acceptance, or truth authority.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import struct
from pathlib import Path
from typing import Iterable

ROOT = Path(__file__).resolve().parents[2]
RUN_ID_DEFAULT = "t_73291be5"
FLOAT = 5126
UNSIGNED_BYTE = 5121
UNSIGNED_SHORT = 5123
UNSIGNED_INT = 5125
ARRAY_BUFFER = 34962
TYPE_COMPS = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4}
COMPONENT_INFO = {
    FLOAT: ("f", 4),
    UNSIGNED_BYTE: ("B", 1),
    UNSIGNED_SHORT: ("H", 2),
    UNSIGNED_INT: ("I", 4),
}
EPS = 1.0e-12


def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, data: dict) -> None:
    path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def resolve_uri(base: Path, uri: str) -> Path:
    return (base.parent / uri).resolve()


def accessor_layout(gltf: dict, accessor_id: int) -> tuple[int, int, int, str, int]:
    acc = gltf["accessors"][accessor_id]
    view = gltf["bufferViews"][acc["bufferView"]]
    component_type = int(acc["componentType"])
    if component_type not in COMPONENT_INFO:
        raise ValueError(f"unsupported componentType {component_type} in accessor {accessor_id}")
    fmt_char, component_size = COMPONENT_INFO[component_type]
    comps = TYPE_COMPS[acc["type"]]
    start = int(view.get("byteOffset", 0)) + int(acc.get("byteOffset", 0))
    stride = int(view.get("byteStride", component_size * comps))
    count = int(acc["count"])
    return start, stride, count, fmt_char, comps


def read_accessor(gltf: dict, buf: bytes, accessor_id: int) -> list[tuple[float, ...]]:
    start, stride, count, fmt_char, comps = accessor_layout(gltf, accessor_id)
    fmt = "<" + fmt_char * comps
    size = struct.calcsize(fmt)
    rows = []
    for i in range(count):
        off = start + i * stride
        rows.append(tuple(struct.unpack_from(fmt, buf, off)[:comps]))
    return rows


def read_indices(gltf: dict, buf: bytes, accessor_id: int | None, vertex_count: int) -> list[int]:
    if accessor_id is None:
        return list(range(vertex_count))
    return [int(row[0]) for row in read_accessor(gltf, buf, int(accessor_id))]


def sub3(a: Iterable[float], b: Iterable[float]) -> tuple[float, float, float]:
    ax, ay, az = a
    bx, by, bz = b
    return ax - bx, ay - by, az - bz


def dot3(a: Iterable[float], b: Iterable[float]) -> float:
    ax, ay, az = a
    bx, by, bz = b
    return ax * bx + ay * by + az * bz


def cross3(a: Iterable[float], b: Iterable[float]) -> tuple[float, float, float]:
    ax, ay, az = a
    bx, by, bz = b
    return ay * bz - az * by, az * bx - ax * bz, ax * by - ay * bx


def length3(v: Iterable[float]) -> float:
    return math.sqrt(dot3(v, v))


def normalize3(v: Iterable[float], fallback: tuple[float, float, float]) -> tuple[float, float, float]:
    x, y, z = v
    l = math.sqrt(x * x + y * y + z * z)
    if l <= EPS:
        return fallback
    return x / l, y / l, z / l


def add3(a: tuple[float, float, float], b: tuple[float, float, float]) -> tuple[float, float, float]:
    return a[0] + b[0], a[1] + b[1], a[2] + b[2]


def scale3(v: tuple[float, float, float], s: float) -> tuple[float, float, float]:
    return v[0] * s, v[1] * s, v[2] * s


def orthogonal_fallback(normal: tuple[float, float, float]) -> tuple[float, float, float]:
    axis = (1.0, 0.0, 0.0) if abs(normal[0]) < 0.9 else (0.0, 1.0, 0.0)
    return normalize3(cross3(axis, normal), (1.0, 0.0, 0.0))


def build_tangents(
    positions: list[tuple[float, ...]],
    normals: list[tuple[float, ...]],
    uvs: list[tuple[float, ...]],
    indices: list[int],
) -> list[tuple[float, float, float, float]]:
    if not (len(positions) == len(normals) == len(uvs)):
        raise ValueError("POSITION/NORMAL/TEXCOORD_0 accessor counts differ")
    count = len(positions)
    tan1 = [(0.0, 0.0, 0.0) for _ in range(count)]
    tan2 = [(0.0, 0.0, 0.0) for _ in range(count)]
    for tri in range(0, len(indices) - 2, 3):
        i1, i2, i3 = indices[tri], indices[tri + 1], indices[tri + 2]
        if min(i1, i2, i3) < 0 or max(i1, i2, i3) >= count:
            raise ValueError(f"triangle index out of bounds: {(i1, i2, i3)} for {count} vertices")
        p1, p2, p3 = positions[i1], positions[i2], positions[i3]
        w1, w2, w3 = uvs[i1], uvs[i2], uvs[i3]
        x1, y1, z1 = sub3(p2, p1)
        x2, y2, z2 = sub3(p3, p1)
        s1 = float(w2[0]) - float(w1[0])
        t1 = float(w2[1]) - float(w1[1])
        s2 = float(w3[0]) - float(w1[0])
        t2 = float(w3[1]) - float(w1[1])
        denom = s1 * t2 - s2 * t1
        if abs(denom) <= EPS:
            continue
        r = 1.0 / denom
        sdir = ((t2 * x1 - t1 * x2) * r, (t2 * y1 - t1 * y2) * r, (t2 * z1 - t1 * z2) * r)
        tdir = ((s1 * x2 - s2 * x1) * r, (s1 * y2 - s2 * y1) * r, (s1 * z2 - s2 * z1) * r)
        for i in (i1, i2, i3):
            tan1[i] = add3(tan1[i], sdir)
            tan2[i] = add3(tan2[i], tdir)
    tangents = []
    for i in range(count):
        n = normalize3((float(normals[i][0]), float(normals[i][1]), float(normals[i][2])), (0.0, 1.0, 0.0))
        t = tan1[i]
        # Gram-Schmidt orthogonalize tangent to normal.
        t = sub3(t, scale3(n, dot3(n, t)))
        if length3(t) <= EPS:
            t = orthogonal_fallback(n)
            w = 1.0
        else:
            t = normalize3(t, orthogonal_fallback(n))
            w = -1.0 if dot3(cross3(n, t), tan2[i]) < 0.0 else 1.0
        tangents.append((float(t[0]), float(t[1]), float(t[2]), float(w)))
    return tangents


def append_aligned(buf: bytearray, data: bytes) -> int:
    while len(buf) % 4:
        buf.append(0)
    offset = len(buf)
    buf.extend(data)
    while len(buf) % 4:
        buf.append(0)
    return offset


def pack_tangents(tangents: list[tuple[float, float, float, float]]) -> bytes:
    out = bytearray()
    for tangent in tangents:
        out.extend(struct.pack("<ffff", *tangent))
    return bytes(out)


def add_tangents_to_gltf(gltf_path: Path) -> dict:
    gltf = read_json(gltf_path)
    if len(gltf.get("buffers", [])) != 1:
        raise ValueError(f"{gltf_path} expected exactly one external buffer")
    buffer_uri = gltf["buffers"][0].get("uri", "")
    if not buffer_uri or buffer_uri.startswith("data:"):
        raise ValueError(f"{gltf_path} expected non-data external buffer uri")
    bin_path = resolve_uri(gltf_path, buffer_uri)
    buf = bytearray(bin_path.read_bytes())
    old_gltf_sha = sha256(gltf_path)
    old_bin_sha = sha256(bin_path)
    added = 0
    skipped_existing = 0
    for mesh in gltf.get("meshes", []) or []:
        for primitive in mesh.get("primitives", []) or []:
            attrs = primitive.setdefault("attributes", {})
            if "TANGENT" in attrs:
                skipped_existing += 1
                continue
            for required in ("POSITION", "NORMAL", "TEXCOORD_0"):
                if required not in attrs:
                    raise ValueError(f"{gltf_path} primitive missing {required}; cannot derive tangents")
            positions = read_accessor(gltf, bytes(buf), int(attrs["POSITION"]))
            normals = read_accessor(gltf, bytes(buf), int(attrs["NORMAL"]))
            uvs = read_accessor(gltf, bytes(buf), int(attrs["TEXCOORD_0"]))
            indices = read_indices(gltf, bytes(buf), primitive.get("indices"), len(positions))
            tangents = build_tangents(positions, normals, uvs, indices)
            tangent_bytes = pack_tangents(tangents)
            byte_offset = append_aligned(buf, tangent_bytes)
            buffer_view_id = len(gltf.setdefault("bufferViews", []))
            gltf["bufferViews"].append(
                {
                    "buffer": 0,
                    "byteLength": len(tangent_bytes),
                    "byteOffset": byte_offset,
                    "target": ARRAY_BUFFER,
                }
            )
            accessor_id = len(gltf.setdefault("accessors", []))
            gltf["accessors"].append(
                {
                    "bufferView": buffer_view_id,
                    "byteOffset": 0,
                    "componentType": FLOAT,
                    "count": len(tangents),
                    "type": "VEC4",
                }
            )
            attrs["TANGENT"] = accessor_id
            added += 1
    if added:
        gltf["buffers"][0]["byteLength"] = len(buf)
        extras = gltf.setdefault("extras", {})
        extras["oathyard_tangent_generation"] = {
            "schema": "oathyard.model_candidate_tangent_generation.v1",
            "tool": "tools/model_candidates/add_tangents_to_candidates.py",
            "method": "deterministic_position_normal_uv_triangle_tangent_basis",
            "presentation_only": True,
            "truth_mutation": False,
            "production_ready_after_this_evidence": False,
            "external_dcc_validation_claimed": False,
            "external_khronos_validation_claimed": False,
        }
        bin_path.write_bytes(bytes(buf))
        write_json(gltf_path, gltf)
    return {
        "gltf": rel(gltf_path),
        "bin": rel(bin_path),
        "added_tangent_primitives": added,
        "skipped_existing_tangent_primitives": skipped_existing,
        "old_gltf_sha256": old_gltf_sha,
        "new_gltf_sha256": sha256(gltf_path),
        "old_bin_sha256": old_bin_sha,
        "new_bin_sha256": sha256(bin_path),
    }


def rel(path: Path) -> str:
    return path.resolve().relative_to(ROOT).as_posix()


def update_manifest_hashes(run_id: str) -> None:
    candidate_manifest_path = ROOT / "assets" / "model_candidates" / run_id / "model_candidate_manifest.json"
    source_manifest_path = ROOT / "assets_src" / "model_candidates" / run_id / "model_source_manifest.json"
    candidate_manifest = read_json(candidate_manifest_path)
    source_manifest = read_json(source_manifest_path)
    by_id = {entry["id"]: entry for entry in candidate_manifest.get("entries", [])}
    for entry in candidate_manifest.get("entries", []):
        gltf_path = ROOT / entry["runtime_gltf"]
        bin_path = ROOT / entry["runtime_bin"]
        entry.setdefault("sha256", {})["gltf"] = sha256(gltf_path)
        entry.setdefault("sha256", {})["bin"] = sha256(bin_path)
    for entry in source_manifest.get("entries", []):
        candidate_entry = by_id[entry["id"]]
        gltf_path = ROOT / candidate_entry["runtime_gltf"]
        bin_path = ROOT / candidate_entry["runtime_bin"]
        entry.setdefault("sha256", {})["gltf"] = sha256(gltf_path)
        entry.setdefault("sha256", {})["bin"] = sha256(bin_path)
    write_json(candidate_manifest_path, candidate_manifest)
    write_json(source_manifest_path, source_manifest)


def assert_all_tangents(run_id: str) -> None:
    manifest = read_json(ROOT / "assets" / "model_candidates" / run_id / "model_candidate_manifest.json")
    missing = []
    for entry in manifest.get("entries", []):
        gltf_path = ROOT / entry["runtime_gltf"]
        gltf = read_json(gltf_path)
        for mesh_index, mesh in enumerate(gltf.get("meshes", []) or []):
            for prim_index, primitive in enumerate(mesh.get("primitives", []) or []):
                if "TANGENT" not in (primitive.get("attributes", {}) or {}):
                    missing.append(f"{entry['id']} mesh {mesh_index} primitive {prim_index}")
    if missing:
        raise SystemExit("missing TANGENT attributes:\n" + "\n".join(missing))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-id", default=RUN_ID_DEFAULT)
    parser.add_argument("--summary", default="")
    args = parser.parse_args()
    manifest = read_json(ROOT / "assets" / "model_candidates" / args.run_id / "model_candidate_manifest.json")
    results = []
    for entry in manifest.get("entries", []):
        results.append(add_tangents_to_gltf(ROOT / entry["runtime_gltf"]))
    update_manifest_hashes(args.run_id)
    assert_all_tangents(args.run_id)
    summary = {
        "schema": "oathyard.model_candidate_tangent_generation_summary.v1",
        "run_id": args.run_id,
        "asset_count": len(results),
        "total_added_tangent_primitives": sum(r["added_tangent_primitives"] for r in results),
        "total_existing_tangent_primitives": sum(r["skipped_existing_tangent_primitives"] for r in results),
        "production_ready_after_this_evidence": False,
        "truth_mutation": False,
        "external_dcc_validation_claimed": False,
        "external_khronos_validation_claimed": False,
        "results": results,
    }
    text = json.dumps(summary, indent=2, sort_keys=True) + "\n"
    if args.summary:
        path = ROOT / args.summary
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
    print(text, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
