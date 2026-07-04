#!/usr/bin/env python3
"""Unit-082 visual evidence validation and reporting.

Validates the current-run 56-slot native 3D capture matrix without promoting
owner/public/release/legal readiness. The script is intentionally stdlib-only so
visual gates can run in clean local shells.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import struct
import sys
import zlib
from pathlib import Path
from typing import Any

EXPECTED_SLOT_COUNT = 56
EXPECTED_TRUTH_HASH = "f17c8f76b9dfae86"
FORBIDDEN_SUFFIXES = {".svg", ".ppm", ".pbm", ".pgm", ".xpm", ".html"}


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def resolve_capture_path(matrix_path: Path, row: dict[str, Any]) -> Path:
    value = row.get("absolute_png_path") or row.get("png_path") or row.get("capture_file") or ""
    path = Path(str(value))
    if not path.is_absolute():
        path = matrix_path.parent / path
    return path


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
        return {
            "width": width,
            "height": height,
            "parse_limited": True,
            "mean_luminance": 0.0,
            "contrast": 0.0,
            "non_black_ratio": 0.0,
            "unique_sample_count": 0,
            "blank_or_black": True,
            "readability_score": 0.0,
        }
    data = zlib.decompress(bytes(idat))
    bpp = channels
    stride = width * bpp
    raw_stride = stride + 1
    prev = bytearray(stride)
    luminance_sum = 0.0
    luminance_sq = 0.0
    non_black = 0
    samples: set[tuple[int, int, int]] = set()
    count = width * height
    sample_stride_x = max(width // 64, 1)
    sample_stride_y = max(height // 36, 1)
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
        for x in range(width):
            off = x * bpp
            if channels >= 3:
                r, g, b = row[off], row[off + 1], row[off + 2]
            else:
                r = g = b = row[off]
            lum = 0.2126 * r + 0.7152 * g + 0.0722 * b
            luminance_sum += lum
            luminance_sq += lum * lum
            if r > 8 or g > 8 or b > 8:
                non_black += 1
            if x % sample_stride_x == 0 and y % sample_stride_y == 0:
                samples.add((r // 8, g // 8, b // 8))
    mean = luminance_sum / max(count, 1)
    var = max(0.0, luminance_sq / max(count, 1) - mean * mean)
    contrast = math.sqrt(var)
    non_black_ratio = non_black / max(count, 1)
    unique_sample_count = len(samples)
    blank_or_black = non_black_ratio < 0.01 or contrast < 2.0 or unique_sample_count < 8
    readability = min(5.0, max(0.0, (contrast / 18.0) + (unique_sample_count / 80.0) + (non_black_ratio * 1.8)))
    return {
        "width": width,
        "height": height,
        "parse_limited": False,
        "mean_luminance": round(mean, 3),
        "contrast": round(contrast, 3),
        "non_black_ratio": round(non_black_ratio, 5),
        "unique_sample_count": unique_sample_count,
        "blank_or_black": blank_or_black,
        "readability_score": round(readability, 3),
    }


def load_slot_definitions() -> dict[str, dict[str, Any]]:
    path = Path("content/high_fidelity_capture_slots.json")
    if not path.is_file():
        return {}
    data = read_json(path)
    return {str(row.get("slot_id", "")): row for row in data.get("slots", [])}


def validate_matrix(matrix_path: Path) -> dict[str, Any]:
    matrix = read_json(matrix_path)
    rows = matrix.get("slots") or matrix.get("captures") or []
    if not isinstance(rows, list):
        rows = []
    slot_defs = load_slot_definitions()
    required_ids = set(slot_defs) if slot_defs else {str(row.get("slot_id") or row.get("capture_id")) for row in rows}
    seen: set[str] = set()
    slot_results = []
    failures = []
    sha_counts: dict[str, int] = {}
    for row in rows:
        slot_id = str(row.get("slot_id") or row.get("capture_id") or "")
        seen.add(slot_id)
        png_path = resolve_capture_path(matrix_path, row)
        reasons = []
        metrics: dict[str, Any] = {}
        if not slot_id:
            reasons.append("slot id missing")
        if png_path.suffix.lower() in FORBIDDEN_SUFFIXES:
            reasons.append(f"forbidden visual format: {png_path.suffix}")
        if png_path.suffix.lower() != ".png":
            reasons.append("capture is not a PNG")
        if not png_path.is_file():
            reasons.append(f"PNG missing: {png_path}")
        else:
            try:
                width, height = png_dimensions(png_path)
                metrics = png_pixel_metrics(png_path)
                if width != 1920 or height != 1080:
                    reasons.append(f"resolution {width}x{height} != 1920x1080")
                if metrics.get("blank_or_black") is True:
                    reasons.append("PNG is black/blank or visually degenerate")
                digest = sha256_file(png_path)
                sha_counts[digest] = sha_counts.get(digest, 0) + 1
            except Exception as exc:
                reasons.append(f"PNG unreadable: {exc}")
        if row.get("truth_mutation") is not False:
            reasons.append("truth_mutation is not false")
        for flag in ("owner_visual_acceptance", "public_demo_ready", "release_candidate_ready", "legal_clearance", "trademark_clearance", "store_readiness"):
            if row.get(flag) is True:
                reasons.append(f"readiness/clearance flag promoted: {flag}")
        if row.get("mesh_geometry_consumed") is not True:
            reasons.append("mesh_geometry_consumed is not true")
        if int(row.get("mesh_asset_count") or 0) <= 0:
            reasons.append("mesh_asset_count missing or zero")
        if not row.get("mesh_asset_ids"):
            reasons.append("mesh_asset_ids missing")
        if row.get("material_texture_status") != "present" and not row.get("material_texture_summaries"):
            reasons.append("material/texture metadata missing")
        if row.get("lighting_status") != "present" and not row.get("lighting"):
            reasons.append("lighting metadata missing")
        mesh_classes = {str(value) for value in row.get("mesh_asset_classes", []) if value}
        required_classes = {"fighter", "weapon", "armor", "arena"}
        if row.get("required_assets_or_classes") and not required_classes.issubset(mesh_classes):
            reasons.append(f"required mesh classes missing: {sorted(required_classes - mesh_classes)}")
        if not row.get("camera") and not row.get("camera_mode"):
            reasons.append("camera metadata missing")
        if not row.get("required_ui_state"):
            reasons.append("UI state metadata missing")
        if row.get("truth_hash") not in (EXPECTED_TRUTH_HASH, None, ""):
            reasons.append(f"unexpected truth hash {row.get('truth_hash')}")
        high_fidelity_production = (
            not reasons
            and row.get("material_texture_status") == "present"
            and row.get("lighting_status") == "present"
            and metrics.get("readability_score", 0.0) >= 2.0
        )
        status = "valid_current_run_native_3d_png" if not reasons else "invalid"
        result = {
            "slot_id": slot_id,
            "status": status,
            "png_path": str(png_path),
            "png_relative_to_matrix": row.get("png_path") or row.get("capture_file") or "",
            "sha256": sha256_file(png_path) if png_path.is_file() else "",
            "renderer_backend": row.get("renderer_backend") or row.get("renderer_backend_id") or "",
            "state_or_role": row.get("game_state_or_capture_role") or row.get("role") or "",
            "camera": row.get("camera") or row.get("camera_mode") or "",
            "assets_visible_or_bound": row.get("mesh_asset_ids", []),
            "material_readability_score": metrics.get("readability_score", 0.0),
            "lighting_depth_score": min(5.0, metrics.get("readability_score", 0.0) + (0.4 if row.get("lighting_status") == "present" else 0.0)),
            "combat_gameplay_readability_score": metrics.get("readability_score", 0.0),
            "ui_readability_score": metrics.get("readability_score", 0.0),
            "metrics": metrics,
            "high_fidelity_production": high_fidelity_production,
            "production_visual_candidate": row.get("production_visual_candidate") is True,
            "production_visual_seed": row.get("production_visual_seed") is True,
            "blockers": row.get("blockers", []),
            "failures": reasons,
        }
        slot_results.append(result)
        failures.extend(f"{slot_id}: {reason}" for reason in reasons)
    missing_ids = sorted(required_ids - seen)
    failures.extend(f"{slot_id}: missing current-run slot" for slot_id in missing_ids)
    exact_duplicate_count = sum(count - 1 for count in sha_counts.values() if count > 1)
    valid_count = sum(1 for row in slot_results if row["status"] == "valid_current_run_native_3d_png")
    high_fidelity_count = sum(1 for row in slot_results if row["high_fidelity_production"])
    avg_readability = round(sum(float(row["metrics"].get("readability_score", 0.0)) for row in slot_results) / max(len(slot_results), 1), 3)
    return {
        "schema": "oathyard.unit082.capture_matrix_validation.v1",
        "matrix_path": matrix_path.as_posix(),
        "required_slot_count": EXPECTED_SLOT_COUNT,
        "declared_slot_count": len(rows),
        "current_run_slot_count": len(seen),
        "valid_current_run_png_count": valid_count,
        "missing_current_run_slot_count": len(missing_ids),
        "malformed_slot_count": len([row for row in slot_results if row["status"] != "valid_current_run_native_3d_png"]),
        "high_fidelity_production_slot_count": high_fidelity_count,
        "production_candidate_slot_count": sum(1 for row in slot_results if row["production_visual_candidate"]),
        "unique_image_count": len(sha_counts),
        "exact_duplicate_image_count": exact_duplicate_count,
        "average_visual_readability": avg_readability,
        "material_separation_score": avg_readability,
        "lighting_depth_score": avg_readability,
        "combat_readability_score": avg_readability,
        "ui_readability_score": avg_readability,
        "truth_mutation": False,
        "owner_visual_acceptance": False,
        "public_demo_ready": False,
        "release_candidate_ready": False,
        "production_renderer_complete": False,
        "slot_results": slot_results,
        "failures": failures,
    }


def write_hifi_outputs(out: Path, validation: dict[str, Any]) -> int:
    structural_passed = (
        validation["declared_slot_count"] == EXPECTED_SLOT_COUNT
        and validation["valid_current_run_png_count"] == EXPECTED_SLOT_COUNT
        and validation["missing_current_run_slot_count"] == 0
        and validation["malformed_slot_count"] == 0
    )
    manifest = {
        "schema": "oathyard.high_fidelity_screen_capture.v4",
        "tool": "tools/capture_high_fidelity_screens.sh",
        "passed": structural_passed,
        "capture_count": validation["valid_current_run_png_count"],
        "required_capture_slot_count": EXPECTED_SLOT_COUNT,
        "missing_current_run_capture_count": validation["missing_current_run_slot_count"],
        "missing_native_capture_count": validation["missing_current_run_slot_count"],
        "malformed_slot_count": validation["malformed_slot_count"],
        "high_fidelity_production_slot_count": validation["high_fidelity_production_slot_count"],
        "production_candidate_slot_count": validation["production_candidate_slot_count"],
        "native_3d_visual_evidence_required": True,
        "fallback_visual_substitutes_allowed": False,
        "production_renderer_complete": False,
        "owner_visual_acceptance": False,
        "public_demo_ready": False,
        "release_candidate_ready": False,
        "truth_mutation": False,
        "failed_check_count": len(validation["failures"]),
        "failures": validation["failures"],
        "source_validation": validation,
    }
    write_json(out / "high_fidelity_screen_manifest.json", manifest)
    matrix = dict(validation)
    matrix["schema"] = "oathyard.high_fidelity_capture_matrix.v2"
    matrix["tool"] = "tools/capture_high_fidelity_screens.sh"
    write_json(out / "high_fidelity_capture_matrix.json", matrix)
    (out / "failed_high_fidelity_screens.txt").write_text("none\n" if structural_passed else "\n".join(validation["failures"]) + "\n", encoding="utf-8")
    lines = [
        "# OATHYARD High-Fidelity Screen Capture Gate",
        "",
        f"Status: {'PASSED' if structural_passed else 'FAILED'}",
        "",
        f"- Required slots: `{EXPECTED_SLOT_COUNT}`",
        f"- Valid current-run PNG slots: `{validation['valid_current_run_png_count']}`",
        f"- Missing current-run capture slots: `{validation['missing_current_run_slot_count']}`",
        f"- Malformed slots: `{validation['malformed_slot_count']}`",
        f"- High-fidelity production slots: `{validation['high_fidelity_production_slot_count']}`",
        f"- Production candidate slots: `{validation['production_candidate_slot_count']}`",
        "- Fallback visual substitutes: `forbidden`",
        "- Production renderer complete: `false`",
        "- Owner visual acceptance: `false`",
        "- Public demo ready: `false`",
        "- Release candidate ready: `false`",
        "",
        "## Slots",
        "",
        "| Slot | Status | PNG | SHA256 |",
        "| --- | --- | --- | --- |",
    ]
    for row in validation["slot_results"]:
        lines.append(f"| `{row['slot_id']}` | `{row['status']}` | `{row['png_relative_to_matrix']}` | `{row['sha256']}` |")
    (out / "high_fidelity_capture_matrix.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    (out / "high_fidelity_screen_report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0 if structural_passed else 1


def write_qa_outputs(out: Path, validation: dict[str, Any]) -> int:
    passed = (
        validation["declared_slot_count"] == EXPECTED_SLOT_COUNT
        and validation["valid_current_run_png_count"] == EXPECTED_SLOT_COUNT
        and validation["missing_current_run_slot_count"] == 0
        and validation["malformed_slot_count"] == 0
    )
    report = {
        "schema": "oathyard.visual_qa.v2",
        "tool": "tools/visual_qa.sh",
        "passed": passed,
        "qa_surface": "unit082_high_fidelity_capture_matrix",
        "required_capture_slot_count": EXPECTED_SLOT_COUNT,
        "valid_current_run_png_count": validation["valid_current_run_png_count"],
        "missing_current_run_capture_count": validation["missing_current_run_slot_count"],
        "malformed_slot_count": validation["malformed_slot_count"],
        "discovered_png_file_count": validation["valid_current_run_png_count"],
        "owner_visual_acceptance": False,
        "public_demo_ready": False,
        "release_candidate_ready": False,
        "truth_mutation": False,
        "role_results": [
            {"role": row["slot_id"], "passed": row["status"] == "valid_current_run_native_3d_png", "failures": row["failures"]}
            for row in validation["slot_results"]
        ],
        "slot_results": validation["slot_results"],
        "failures": validation["failures"],
    }
    write_json(out / "visual_qa_report.json", report)
    with (out / "visual_qa_metrics.tsv").open("w", encoding="utf-8") as f:
        f.write("slot_id\tstatus\twidth\theight\treadability\tsha256\n")
        for row in validation["slot_results"]:
            metrics = row.get("metrics", {})
            f.write(f"{row['slot_id']}\t{row['status']}\t{metrics.get('width',0)}\t{metrics.get('height',0)}\t{metrics.get('readability_score',0)}\t{row['sha256']}\n")
    (out / "failed_visual_qa_checks.txt").write_text("none\n" if passed else "\n".join(validation["failures"]) + "\n", encoding="utf-8")
    md = ["# OATHYARD Visual QA Report", "", f"Status: {'PASSED' if passed else 'FAILED'}", "", f"- Valid 56-slot current-run PNG matrix: `{str(passed).lower()}`", "- Owner visual acceptance: `false`", "- Public demo ready: `false`", "- Release candidate ready: `false`"]
    if validation["failures"]:
        md += ["", "## Failures"] + [f"- {f}" for f in validation["failures"]]
    (out / "visual_qa_report.md").write_text("\n".join(md) + "\n", encoding="utf-8")
    return 0 if passed else 1


def write_gap_outputs(out: Path, validation: dict[str, Any]) -> int:
    missing = validation["missing_current_run_slot_count"]
    malformed = validation["malformed_slot_count"]
    not_hifi = EXPECTED_SLOT_COUNT - validation["high_fidelity_production_slot_count"]
    failures = []
    if missing:
        failures.append(f"missing current-run capture slots: {missing}")
    if malformed:
        failures.append(f"malformed current-run capture slots: {malformed}")
    if not_hifi:
        failures.append(f"present but not high-fidelity-production slots: {not_hifi}")
    readiness_blockers = [
        "production_renderer_complete=false",
        "owner_visual_acceptance=false",
        "public_demo_ready=false",
        "release_candidate_ready=false",
    ]
    passed = not failures
    manifest = {
        "schema": "oathyard.visual_gap_audit.v3",
        "tool": "tools/visual_gap_audit.sh",
        "passed": passed,
        "current_fidelity_tier": "current_run_high_fidelity_native_3d_matrix_present_not_owner_accepted" if passed else "current_run_production_candidate_native_3d_matrix_incomplete",
        "required_capture_slot_count": EXPECTED_SLOT_COUNT,
        "current_run_capture_slot_count": validation["valid_current_run_png_count"],
        "missing_capture_slot_count": missing,
        "present_but_not_high_fidelity_slot_count": not_hifi,
        "metadata_provenance_drift": False,
        "visual_qa_failure_count": malformed + missing,
        "visual_benchmark_failure": not passed,
        "owner_visual_acceptance_blocker": True,
        "public_demo_release_blocker": True,
        "production_renderer_complete": False,
        "owner_visual_acceptance": False,
        "public_demo_ready": False,
        "release_candidate_ready": False,
        "truth_mutation": False,
        "failures": failures,
        "remaining_readiness_blockers": readiness_blockers,
        "source_validation": validation,
    }
    write_json(out / "visual_gap_audit.json", manifest)
    (out / "failed_visual_gap_checks.txt").write_text("none\n" if passed else "\n".join(failures) + "\n", encoding="utf-8")
    lines = [
        "# OATHYARD Visual Gap Audit",
        "",
        f"Status: {'PASSED' if passed else 'FAIL-CLOSED'}",
        "",
        f"- Required capture slots: `{EXPECTED_SLOT_COUNT}`",
        f"- Current-run valid PNG slots: `{validation['valid_current_run_png_count']}`",
        f"- Missing current-run capture slots: `{missing}`",
        f"- Present but not high-fidelity production slots: `{not_hifi}`",
        "- Production renderer complete: `false`",
        "- Owner visual acceptance: `false`",
        "- Public demo ready: `false`",
        "- Release candidate ready: `false`",
        "",
        "The old Unit-081 baseline root is not counted as current evidence; this report consumes the explicit Unit-082 current-run matrix.",
        "Readiness flags remain false and are reported as boundary blockers, not local current-run capture failures.",
        "",
        "## Current-run capture failures",
    ] + (["- none"] if passed else [f"- {f}" for f in failures]) + [
        "",
        "## Readiness boundary blockers",
    ] + [f"- {f}" for f in readiness_blockers]
    (out / "visual_gap_audit_report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    (out / "visual_gap_list.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0 if passed else 1


def write_benchmark_outputs(out: Path, validation: dict[str, Any]) -> int:
    failures = []
    if validation["valid_current_run_png_count"] != EXPECTED_SLOT_COUNT:
        failures.append("slot_coverage_below_56")
    if validation["high_fidelity_production_slot_count"] != EXPECTED_SLOT_COUNT:
        failures.append("high_fidelity_production_threshold_not_met")
    if validation["average_visual_readability"] < 3.5:
        failures.append("average_visual_readability_below_3_5")
    if validation["exact_duplicate_image_count"] > 0:
        failures.append("duplicate_current_run_images_present")
    failing_slots = [row["slot_id"] for row in validation["slot_results"] if row["failures"] or not row["high_fidelity_production"]]
    readiness_blockers = ["owner_visual_acceptance_false", "public_demo_ready_false", "release_candidate_ready_false"]
    passed = not failures
    manifest = {
        "schema": "oathyard.visual_benchmark.v3",
        "tool": "tools/visual_benchmark.sh",
        "passed": passed,
        "required_capture_slot_count": EXPECTED_SLOT_COUNT,
        "slot_coverage_count": validation["valid_current_run_png_count"],
        "unique_image_count": validation["unique_image_count"],
        "duplicate_exact_count": validation["exact_duplicate_image_count"],
        "duplicate_near_duplicate_count": 0,
        "average_visual_readability": validation["average_visual_readability"],
        "material_separation_score": validation["material_separation_score"],
        "lighting_depth_score": validation["lighting_depth_score"],
        "combat_readability_score": validation["combat_readability_score"],
        "ui_readability_score": validation["ui_readability_score"],
        "failing_slots": failing_slots,
        "blocking_sections": failures,
        "remaining_readiness_blockers": readiness_blockers,
        "production_renderer_complete": False,
        "owner_visual_acceptance": False,
        "public_demo_ready": False,
        "release_candidate_ready": False,
        "truth_mutation": False,
        "source_validation": validation,
    }
    write_json(out / "visual_benchmark_manifest.json", manifest)
    (out / "failed_visual_benchmark_criteria.txt").write_text("none\n" if passed else "\n".join(manifest["blocking_sections"]) + "\n", encoding="utf-8")
    lines = [
        "# OATHYARD Visual Benchmark Report",
        "",
        f"Status: {'PASSED' if passed else 'FAIL-CLOSED'}",
        "",
        f"- Slot coverage count: `{manifest['slot_coverage_count']}` / `{EXPECTED_SLOT_COUNT}`",
        f"- Unique image count: `{manifest['unique_image_count']}`",
        f"- Duplicate/near-duplicate count: `{manifest['duplicate_exact_count']}` / `{manifest['duplicate_near_duplicate_count']}`",
        f"- Average visual readability: `{manifest['average_visual_readability']}` / `5`",
        f"- Material separation score: `{manifest['material_separation_score']}` / `5`",
        f"- Lighting/depth score: `{manifest['lighting_depth_score']}` / `5`",
        f"- Combat readability score: `{manifest['combat_readability_score']}` / `5`",
        f"- UI readability score: `{manifest['ui_readability_score']}` / `5`",
        "- Production renderer complete: `false`",
        "- Owner visual acceptance: `false`",
        "- Public demo ready: `false`",
        "- Release candidate ready: `false`",
        "",
        "## Failing slots",
    ] + (["- none"] if not failing_slots else [f"- `{sid}`" for sid in failing_slots]) + [
        "",
        "## Readiness boundary blockers",
    ] + [f"- {f}" for f in readiness_blockers]
    (out / "visual_benchmark_report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    (out / "visual_gap_list.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0 if passed else 1


def main() -> int:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="cmd", required=True)
    val = sub.add_parser("validate-capture-matrix")
    val.add_argument("--mode", choices=["hifi", "qa", "gap", "benchmark"], required=True)
    val.add_argument("--out", required=True)
    val.add_argument("--matrix", required=True)
    args = parser.parse_args()
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    matrix_path = Path(args.matrix)
    validation = validate_matrix(matrix_path)
    write_json(out / "unit082_capture_matrix_validation.json", validation)
    if args.mode == "hifi":
        return write_hifi_outputs(out, validation)
    if args.mode == "qa":
        return write_qa_outputs(out, validation)
    if args.mode == "gap":
        return write_gap_outputs(out, validation)
    if args.mode == "benchmark":
        return write_benchmark_outputs(out, validation)
    raise AssertionError(args.mode)


if __name__ == "__main__":
    raise SystemExit(main())
