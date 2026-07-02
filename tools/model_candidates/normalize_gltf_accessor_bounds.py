#!/usr/bin/env python3
"""
Normalize accessor min/max metadata for OATHYARD model-candidate glTF files.

This fixes Khronos Validator accessor-bound metadata mismatches without changing
geometry buffers, truth data, or readiness flags. It cascades glTF hashes through
the model-candidate, source, and presentation manifests so later validator
evidence can be tied to exact bytes.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import struct
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_RUN_ID = "t_73291be5"
COMPONENT_INFO = {
    5120: ("b", 1),
    5121: ("B", 1),
    5122: ("h", 2),
    5123: ("H", 2),
    5125: ("I", 4),
    5126: ("f", 4),
}
TYPE_COMPS = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT2": 4, "MAT3": 9, "MAT4": 16}


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, data: dict[str, Any]) -> None:
    path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def rel(path: Path) -> str:
    return path.resolve().relative_to(ROOT).as_posix()


def resolve_uri(gltf_path: Path, uri: str) -> Path:
    return (gltf_path.parent / uri).resolve()


def accessor_layout(gltf: dict[str, Any], accessor_id: int) -> tuple[int, int, int, str, int, int]:
    accessor = gltf["accessors"][accessor_id]
    view = gltf["bufferViews"][accessor["bufferView"]]
    component_type = int(accessor["componentType"])
    if component_type not in COMPONENT_INFO:
        raise ValueError(f"unsupported component type {component_type} in accessor {accessor_id}")
    fmt_char, component_size = COMPONENT_INFO[component_type]
    comps = TYPE_COMPS[accessor["type"]]
    start = int(view.get("byteOffset", 0)) + int(accessor.get("byteOffset", 0))
    stride = int(view.get("byteStride", component_size * comps))
    return start, stride, int(accessor["count"]), fmt_char, comps, component_type


def read_accessor(gltf: dict[str, Any], buffer_bytes: bytes, accessor_id: int) -> list[tuple[float, ...]]:
    start, stride, count, fmt_char, comps, _component_type = accessor_layout(gltf, accessor_id)
    fmt = "<" + fmt_char * comps
    rows = []
    for index in range(count):
        offset = start + index * stride
        rows.append(tuple(struct.unpack_from(fmt, buffer_bytes, offset)))
    return rows


def normalize_number(value: float, component_type: int) -> float | int:
    if component_type == 5126:
        # Match the exact float32 value stored in the buffer while keeping JSON deterministic.
        value = struct.unpack("<f", struct.pack("<f", float(value)))[0]
        if value == 0.0:
            return 0.0
        return float(value)
    return int(value)


def recompute_accessor_bounds(gltf_path: Path) -> dict[str, Any]:
    gltf = read_json(gltf_path)
    if len(gltf.get("buffers", [])) != 1:
        raise ValueError(f"{rel(gltf_path)} expected one external buffer")
    buffer_uri = gltf["buffers"][0].get("uri", "")
    if not buffer_uri or buffer_uri.startswith("data:"):
        raise ValueError(f"{rel(gltf_path)} expected non-data external buffer")
    buffer_path = resolve_uri(gltf_path, buffer_uri)
    buffer_bytes = buffer_path.read_bytes()
    old_hash = sha256_file(gltf_path)
    changed = []
    for accessor_id, accessor in enumerate(gltf.get("accessors", [])):
        if "min" not in accessor and "max" not in accessor:
            continue
        rows = read_accessor(gltf, buffer_bytes, accessor_id)
        if not rows:
            continue
        _start, _stride, _count, _fmt_char, comps, component_type = accessor_layout(gltf, accessor_id)
        mins = []
        maxs = []
        for comp in range(comps):
            values = [row[comp] for row in rows]
            mins.append(normalize_number(min(values), component_type))
            maxs.append(normalize_number(max(values), component_type))
        old_min = accessor.get("min")
        old_max = accessor.get("max")
        if old_min != mins or old_max != maxs:
            accessor["min"] = mins
            accessor["max"] = maxs
            changed.append({"accessor": accessor_id, "old_min": old_min, "old_max": old_max, "new_min": mins, "new_max": maxs})
    if changed:
        gltf.setdefault("extras", {})["oathyard_accessor_bounds_normalization"] = {
            "schema": "oathyard.model_candidate_accessor_bounds_normalization.v1",
            "tool": "tools/model_candidates/normalize_gltf_accessor_bounds.py",
            "method": "recomputed_accessor_min_max_from_external_buffer_bytes",
            "presentation_only": True,
            "truth_mutation": False,
            "production_ready_after_this_evidence": False,
            "external_dcc_validation_claimed": False,
        }
        write_json(gltf_path, gltf)
    return {
        "runtime_gltf": rel(gltf_path),
        "runtime_bin": rel(buffer_path),
        "changed_accessor_count": len(changed),
        "changed_accessors": changed,
        "old_gltf_sha256": old_hash,
        "new_gltf_sha256": sha256_file(gltf_path),
        "runtime_bin_sha256": sha256_file(buffer_path),
    }


def cascade_hashes(run_id: str) -> dict[str, Any]:
    candidate_manifest_path = ROOT / "assets" / "model_candidates" / run_id / "model_candidate_manifest.json"
    source_manifest_path = ROOT / "assets_src" / "model_candidates" / run_id / "model_source_manifest.json"
    presentation_manifest_path = ROOT / "assets" / "presentation_manifest.json"
    candidate_manifest = read_json(candidate_manifest_path)
    source_manifest = read_json(source_manifest_path)
    presentation_manifest = read_json(presentation_manifest_path)
    candidate_by_id = {entry["id"]: entry for entry in candidate_manifest.get("entries", [])}
    source_by_id = {entry["id"]: entry for entry in source_manifest.get("entries", [])}
    presentation_by_id = {entry["id"]: entry for entry in presentation_manifest.get("entries", [])}
    updates = []
    for asset_id, entry in candidate_by_id.items():
        gltf_path = ROOT / entry["runtime_gltf"]
        bin_path = ROOT / entry["runtime_bin"]
        gltf_hash = sha256_file(gltf_path)
        bin_hash = sha256_file(bin_path)
        entry.setdefault("sha256", {})["gltf"] = gltf_hash
        entry.setdefault("sha256", {})["bin"] = bin_hash
        if asset_id in source_by_id:
            source_by_id[asset_id].setdefault("sha256", {})["gltf"] = gltf_hash
            source_by_id[asset_id].setdefault("sha256", {})["bin"] = bin_hash
        if asset_id in presentation_by_id:
            presentation_by_id[asset_id]["source_candidate_gltf_hash"] = gltf_hash
            presentation_by_id[asset_id]["source_candidate_bin_hash"] = bin_hash
        updates.append({"asset_id": asset_id, "gltf_sha256": gltf_hash, "bin_sha256": bin_hash})
    write_json(candidate_manifest_path, candidate_manifest)
    write_json(source_manifest_path, source_manifest)
    presentation_manifest["candidate_manifest_hash"] = sha256_file(candidate_manifest_path)
    presentation_manifest["asset_hash"] = sha256_file(presentation_manifest_path) if presentation_manifest_path.is_file() else ""
    write_json(presentation_manifest_path, presentation_manifest)
    # Asset hash covers the written presentation manifest bytes; write a second time to make the value stable.
    presentation_manifest["asset_hash"] = sha256_file(presentation_manifest_path)
    write_json(presentation_manifest_path, presentation_manifest)
    return {
        "candidate_manifest": rel(candidate_manifest_path),
        "candidate_manifest_sha256": sha256_file(candidate_manifest_path),
        "source_manifest": rel(source_manifest_path),
        "source_manifest_sha256": sha256_file(source_manifest_path),
        "presentation_manifest": rel(presentation_manifest_path),
        "presentation_manifest_sha256": sha256_file(presentation_manifest_path),
        "updates": updates,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-id", default=DEFAULT_RUN_ID)
    parser.add_argument("--summary", default="")
    args = parser.parse_args()
    manifest = read_json(ROOT / "assets" / "model_candidates" / args.run_id / "model_candidate_manifest.json")
    results = [recompute_accessor_bounds(ROOT / entry["runtime_gltf"]) for entry in manifest.get("entries", [])]
    cascade = cascade_hashes(args.run_id)
    summary = {
        "schema": "oathyard.model_candidate_accessor_bounds_normalization_summary.v1",
        "run_id": args.run_id,
        "asset_count": len(results),
        "changed_asset_count": sum(1 for result in results if result["changed_accessor_count"] > 0),
        "changed_accessor_count": sum(result["changed_accessor_count"] for result in results),
        "truth_mutation": False,
        "production_ready_after_this_evidence": False,
        "external_dcc_validation_claimed": False,
        "results": results,
        "cascade": cascade,
    }
    text = json.dumps(summary, indent=2, sort_keys=True) + "\n"
    if args.summary:
        summary_path = ROOT / args.summary
        summary_path.parent.mkdir(parents=True, exist_ok=True)
        summary_path.write_text(text, encoding="utf-8")
    print(text, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
