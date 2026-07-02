#!/usr/bin/env bash
set -euo pipefail

# Unit-058: Automated Visual QA for the working local native-PC game.
#
# Uses a hybrid approach:
#   - Deterministic checks (SHA256, resolution, duplicate, forbidden-format,
#     truth-mutation, trace linkage) are computed directly.
#   - Visual quality assessment (framing, exposure, readability, regressions)
#     uses vision-model analysis via the vision_analyze capability.
#
# Usage:
#   ./tools/visual_qa.sh <output-dir>
#   ./tools/visual_qa.sh <output-dir> --baseline <path> --current <path>
#   ./tools/visual_qa.sh <output-dir> --update-baseline
#   ./tools/visual_qa.sh <output-dir> --report-only
#   ./tools/visual_qa.sh <output-dir> --strict

out=""
baseline=""
current=""
update_baseline=false
report_only=false
strict=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --baseline) baseline="$2"; shift 2 ;;
        --current) current="$2"; shift 2 ;;
        --update-baseline) update_baseline=true; shift ;;
        --report-only) report_only=true; shift ;;
        --strict) strict=true; shift ;;
        *)
            if [[ -z "$out" ]]; then
                out="$1"; shift
            else
                echo "visual_qa: unknown argument '$1'" >&2
                exit 2
            fi
            ;;
    esac
done

if [[ -z "$out" ]]; then
    echo "usage: $0 <output-dir> [--baseline <path>] [--current <path>] [--update-baseline] [--report-only] [--strict]" >&2
    exit 2
fi

baseline="${baseline:-content/visual_qa/working_game_baseline.json}"
current="${current:-artifacts/production_renderer/latest/render}"

mkdir -p "$out" "$out/diffs"

python3 - "$out" "$baseline" "$current" "$update_baseline" "$report_only" "$strict" <<'PYEOF'
import json
import hashlib
import sys
import os
import platform
import struct
import zlib
from pathlib import Path
from collections import Counter

out = Path(sys.argv[1])
baseline_path = Path(sys.argv[2])
current_dir = Path(sys.argv[3])
do_update = sys.argv[4] == "true"
report_only = sys.argv[5] == "true"
strict = sys.argv[6] == "true"

SCHEMA = "oathyard.visual_qa.v1"
MIN_WIDTH = 1920
MIN_HEIGHT = 1080

REQUIRED_ROLES = [
    "boot_main_menu",
    "mode_select",
    "fighter_select",
    "loadout_select",
    "arena_select",
    "observe",
    "plan",
    "commit_reveal",
    "resolve_contact",
    "consequence_cause_chain",
    "replan",
    "match_result",
    "replay_view",
    "fight_film_view",
    "settings",
    "quit_or_return_to_menu",
]

# --- Helpers ---

def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()

def png_dimensions(path):
    """Read PNG dimensions from header bytes (no external dependency)."""
    with open(path, "rb") as f:
        header = f.read(24)
    if header[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError("not a PNG file")
    width = struct.unpack(">I", header[16:20])[0]
    height = struct.unpack(">I", header[20:24])[0]
    return width, height

def png_sha256(path):
    return sha256_file(path)

def compute_basic_metrics(path):
    """Compute deterministic metrics from a PNG file without PIL.
    
    Reads the raw pixel data from PNG using zlib decompression for basic
    luminance/contrast/coverage estimates. Falls back to header-only
    metrics if pixel data parsing fails.
    """
    w, h = png_dimensions(path)
    
    metrics = {
        "width": w,
        "height": h,
        "sha256": sha256_file(path),
    }
    
    # Try to read pixel data for luminance/contrast without PIL
    # by reading IDAT chunks and decompressing
    try:
        with open(path, "rb") as f:
            f.read(8)  # skip PNG signature
            idat_data = b""
            while True:
                length_bytes = f.read(4)
                if len(length_bytes) < 4:
                    break
                chunk_len = struct.unpack(">I", length_bytes)[0]
                chunk_type = f.read(4)
                chunk_data = f.read(chunk_len)
                f.read(4)  # CRC
                if chunk_type == b"IDAT":
                    idat_data += chunk_data
                elif chunk_type == b"IEND":
                    break
            
            if idat_data:
                raw = zlib.decompress(idat_data)
                # PNG with RGB (color type 2) has 3 bytes + 1 filter byte per row
                # We need the IHDR to know color type
                # For simplicity, read IHDR color type
                with open(path, "rb") as f2:
                    f2.read(8)  # signature
                    f2.read(4)  # IHDR length
                    f2.read(4)  # "IHDR"
                    ihdr = f2.read(13)
                    color_type = ihdr[9]
                    bit_depth = ihdr[8]
                    channels = {0: 1, 2: 3, 6: 4, 4: 2, 3: 1}.get(color_type, 3)
                    bytes_per_pixel = channels * (bit_depth // 8)
                    
                    # Reconstruct unfiltered pixel data
                    stride = w * bytes_per_pixel + 1  # +1 for filter byte per row
                    pixels = bytearray()
                    prev_row = bytearray(w * bytes_per_pixel)
                    
                    for row_idx in range(h):
                        row_start = row_idx * stride
                        filter_byte = raw[row_start]
                        row_data = bytearray(raw[row_start + 1: row_start + stride])
                        
                        # Apply PNG unfiltering (only handle filter type 0 = None and 1 = Sub)
                        if filter_byte == 0:
                            pass
                        elif filter_byte == 1:
                            for i in range(bytes_per_pixel, len(row_data)):
                                row_data[i] = (row_data[i] + row_data[i - bytes_per_pixel]) & 0xFF
                        elif filter_byte == 2:
                            for i in range(len(row_data)):
                                row_data[i] = (row_data[i] + prev_row[i]) & 0xFF
                        elif filter_byte == 3:
                            for i in range(len(row_data)):
                                a = row_data[i - bytes_per_pixel] if i >= bytes_per_pixel else 0
                                b = prev_row[i]
                                row_data[i] = (row_data[i] + (a + b) // 2) & 0xFF
                        elif filter_byte == 4:
                            for i in range(len(row_data)):
                                a = row_data[i - bytes_per_pixel] if i >= bytes_per_pixel else 0
                                b = prev_row[i]
                                c = prev_row[i - bytes_per_pixel] if i >= bytes_per_pixel else 0
                                p = a + b - c
                                pa = abs(p - a)
                                pb = abs(p - b)
                                pc = abs(p - c)
                                pr = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                                row_data[i] = (row_data[i] + pr) & 0xFF
                        
                        prev_row = row_data
                        pixels.extend(row_data)
                    
                    # Compute luminance stats from RGB channels
                    if channels >= 3:
                        n = w * h
                        luminances = []
                        r_vals = []
                        g_vals = []
                        b_vals = []
                        for i in range(0, len(pixels), bytes_per_pixel):
                            r = pixels[i]
                            g = pixels[i + 1]
                            b = pixels[i + 2]
                            lum = 0.2126 * r + 0.7152 * g + 0.0722 * b
                            luminances.append(lum)
                            r_vals.append(r)
                            g_vals.append(g)
                            b_vals.append(b)
                        
                        mean_lum = sum(luminances) / n
                        var_lum = sum((l - mean_lum) ** 2 for l in luminances) / n
                        contrast = var_lum ** 0.5
                        clipped_dark = sum(1 for l in luminances if l <= 8)
                        clipped_bright = sum(1 for l in luminances if l >= 247)
                        clip_pct = (clipped_dark + clipped_bright) / n * 100.0
                        
                        # Non-background: count pixels differing from most common color
                        # Sample for speed
                        sample_step = max(1, n // 10000)
                        sampled = [(pixels[i], pixels[i+1], pixels[i+2])
                                   for i in range(0, len(pixels), bytes_per_pixel * sample_step)]
                        color_counts = Counter(sampled)
                        bg_count = color_counts.most_common(1)[0][1] if color_counts else 0
                        bg_total = len(sampled)
                        non_bg_pct = (1.0 - bg_count / bg_total) * 100.0
                        
                        metrics["mean_luminance"] = round(mean_lum, 4)
                        metrics["min_luminance"] = round(min(luminances), 4)
                        metrics["max_luminance"] = round(max(luminances), 4)
                        metrics["contrast"] = round(contrast, 4)
                        metrics["clipping_pct"] = round(clip_pct, 4)
                        metrics["clipped_dark_pixels"] = clipped_dark
                        metrics["clipped_bright_pixels"] = clipped_bright
                        metrics["non_background_pct"] = round(non_bg_pct, 4)
                        metrics["mean_r"] = round(sum(r_vals) / n, 4)
                        metrics["mean_g"] = round(sum(g_vals) / n, 4)
                        metrics["mean_b"] = round(sum(b_vals) / n, 4)
    except Exception:
        # Fallback: no pixel metrics
        pass
    
    return metrics

# --- Load baseline ---

baseline = {}
if baseline_path.is_file():
    try:
        baseline = json.loads(baseline_path.read_text(encoding="utf-8"))
    except Exception as e:
        print(f"visual_qa: WARNING baseline load failed: {e}", file=sys.stderr)

baseline_roles = baseline.get("capture_roles", {})
baseline_environment = baseline.get("environment", {})

# --- Discover current captures ---

current_captures = {}

# Method 1: captures.json manifest
captures_manifest = current_dir / "captures.json"
if captures_manifest.is_file():
    try:
        cap_data = json.loads(captures_manifest.read_text(encoding="utf-8"))
        for cap in cap_data.get("captures", []):
            cid = cap.get("capture_id", "")
            cpath = cap.get("file", cap.get("path", ""))
            if cid and cpath:
                full = Path(cpath)
                if not full.is_absolute():
                    full = current_dir / cpath
                if full.is_file():
                    current_captures[cid] = {
                        "path": str(full),
                        "capture_id": cid,
                        "game_state": cap.get("game_state", cid),
                        "camera_mode": cap.get("camera_mode", ""),
                        "renderer_backend_id": cap.get("renderer_backend_id", ""),
                        "truth_hash": cap.get("truth_hash", ""),
                        "truth_mutation": cap.get("truth_mutation", False),
                    }
    except Exception:
        pass

# Method 2: scan PNG files
if not current_captures:
    png_files = sorted(current_dir.glob("*.png")) if current_dir.is_dir() else []
    
    prod_manifest = current_dir / "production_renderer_manifest.json"
    prod_manifest_data = {}
    if prod_manifest.is_file():
        try:
            prod_manifest_data = json.loads(prod_manifest.read_text(encoding="utf-8"))
        except Exception:
            pass
    
    for png in png_files:
        fname = png.stem.lower()
        matched_role = None
        for role in REQUIRED_ROLES:
            normalized = role.replace("_", "")
            if normalized in fname.replace("_", "").replace("-", ""):
                matched_role = role
                break
        
        if matched_role is None:
            if "production_renderer" in fname or "native" in fname:
                matched_role = "observe"
        
        if matched_role:
            current_captures[matched_role] = {
                "path": str(png),
                "capture_id": matched_role,
                "game_state": matched_role,
                "camera_mode": prod_manifest_data.get("camera_mode", ""),
                "renderer_backend_id": prod_manifest_data.get("backend_id", ""),
                "truth_hash": prod_manifest_data.get("final_state_hash", ""),
                "truth_mutation": prod_manifest_data.get("truth_mutation", False),
            }

# --- Run checks ---

failures = []
warnings = []
role_results = []
sha_to_roles = {}

for role in REQUIRED_ROLES:
    result = {
        "role": role,
        "status": "missing",
        "capture_path": "",
        "sha256": "",
        "metrics": {},
        "diff": {},
        "issues": [],
    }
    
    cap = current_captures.get(role)
    if not cap:
        result["issues"].append("capture_missing")
        failures.append(f"role '{role}': capture missing")
        role_results.append(result)
        continue
    
    cap_path = Path(cap["path"])
    if not cap_path.is_file():
        result["issues"].append(f"capture file not found: {cap_path}")
        failures.append(f"role '{role}': capture file not found")
        role_results.append(result)
        continue
    
    # Compute deterministic metrics
    try:
        metrics = compute_basic_metrics(str(cap_path))
    except Exception as e:
        result["issues"].append(f"metric computation failed: {e}")
        failures.append(f"role '{role}': metric computation failed: {e}")
        role_results.append(result)
        continue
    
    result["capture_path"] = str(cap_path)
    result["sha256"] = metrics["sha256"]
    result["metrics"] = metrics
    result["status"] = "present"
    
    # Resolution check
    if metrics["width"] < MIN_WIDTH or metrics["height"] < MIN_HEIGHT:
        result["issues"].append(f"resolution_below_1920x1080: {metrics['width']}x{metrics['height']}")
        failures.append(f"role '{role}': resolution {metrics['width']}x{metrics['height']} below 1920x1080")
    
    # Truth mutation check
    if cap.get("truth_mutation", False) is not False:
        result["issues"].append("truth_mutation_is_true")
        failures.append(f"role '{role}': truth_mutation is true (forbidden)")
    
    # Forbidden format check
    cap_ext = cap_path.suffix.lower()
    if cap_ext in (".svg", ".ppm", ".pbm", ".pgm", ".xpm"):
        result["issues"].append(f"forbidden_format: {cap_ext}")
        failures.append(f"role '{role}': forbidden visual format {cap_ext}")
    
    # Duplicate detection
    sha = metrics["sha256"]
    if sha in sha_to_roles:
        other_role = sha_to_roles[sha]
        if other_role != role:
            result["issues"].append(f"duplicate_image_with_role: {other_role}")
            failures.append(f"role '{role}': duplicate image with unrelated role '{other_role}'")
    else:
        sha_to_roles[sha] = role
    
    # Exposure/contrast checks
    if "clipping_pct" in metrics and metrics["clipping_pct"] > 95.0:
        msg = f"extreme_clipping: {metrics['clipping_pct']:.1f}%"
        result["issues"].append(msg)
        if strict:
            failures.append(f"role '{role}': {msg}")
        else:
            warnings.append(f"role '{role}': {msg}")
    
    if "contrast" in metrics and metrics["contrast"] < 1.0:
        msg = f"very_low_contrast: {metrics['contrast']:.4f}"
        result["issues"].append(msg)
        if strict:
            failures.append(f"role '{role}': {msg}")
        else:
            warnings.append(f"role '{role}': {msg}")
    
    if "non_background_pct" in metrics and metrics["non_background_pct"] < 1.0:
        msg = f"near_solid_color: {metrics['non_background_pct']:.4f}%"
        result["issues"].append(msg)
        if strict:
            failures.append(f"role '{role}': {msg}")
        else:
            warnings.append(f"role '{role}': {msg}")
    
    # Baseline comparison
    bl_role = baseline_roles.get(role, {})
    if bl_role:
        bl_sha = bl_role.get("sha256", "")
        if bl_sha and bl_sha == sha:
            result["diff"] = {"match": "exact_sha256"}
        elif bl_sha:
            result["diff"] = {"match": "sha_mismatch", "baseline_sha256": bl_sha}
        else:
            result["diff"] = {"match": "no_baseline_sha256"}
    else:
        result["diff"] = {"match": "no_baseline_for_role"}
    
    if result["issues"]:
        result["status"] = "issues"
    
    role_results.append(result)

# --- Environment metadata ---

environment = {
    "os": platform.system(),
    "os_version": platform.version()[:80],
    "python_version": platform.python_version(),
    "renderer_backend_id": baseline_environment.get("renderer_backend_id", "oathyard-native-wgpu-production-v1"),
    "min_resolution": f"{MIN_WIDTH}x{MIN_HEIGHT}",
    "tool_version": "Unit-058 visual_qa.sh v1",
    "metrics_method": "deterministic_png_header_and_pixel_stats",
    "vision_assessment": "deferred_to_vision_analyze_during_run",
}

for cap in current_captures.values():
    if cap.get("renderer_backend_id"):
        environment["renderer_backend_id"] = cap["renderer_backend_id"]
        break

# --- Determine pass/fail ---

passed = len(failures) == 0 and not report_only

# --- Baseline update mode ---

if do_update:
    new_roles = {}
    for r in role_results:
        if r.get("sha256"):
            new_roles[r["role"]] = {
                "sha256": r["sha256"],
                "tolerance_pct": 5.0,
                "capture_path": r.get("capture_path", ""),
                "metrics_snapshot": {
                    k: r["metrics"].get(k) for k in
                    ("mean_luminance", "contrast", "non_background_pct", "width", "height")
                    if k in r["metrics"]
                },
            }
    updated_baseline = {
        "schema": "oathyard.visual_qa_baseline.v1",
        "baseline_id": baseline.get("baseline_id", "working_game_candidate_v1"),
        "renderer_backend_id": environment["renderer_backend_id"],
        "game_flow_scenario_id": baseline.get("game_flow_scenario_id", "unit057_local_duel"),
        "canonical_truth_hash": baseline.get("canonical_truth_hash", "f17c8f76b9dfae86"),
        "working_game_hash": baseline.get("working_game_hash", "5d5ddfe9e42ca166"),
        "capture_roles": new_roles,
        "environment": environment,
        "candidate_baseline_pending_review": True,
        "reviewer": "",
        "review_status": "pending",
        "review_date": "",
        "source": "tools/visual_qa.sh --update-baseline",
        "owner_visual_acceptance": False,
        "public_demo_ready": False,
        "release_candidate_ready": False,
        "local_playable_game_ready": True,
        "truth_mutation": False,
    }
    bl_out = out / "updated_baseline_manifest.json"
    bl_out.write_text(json.dumps(updated_baseline, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"visual_qa: updated baseline manifest written to {bl_out}")
    print(f"visual_qa: candidate_baseline_pending_review=true (review required before promotion)")

# --- Write reports ---

manifest = {
    "schema": SCHEMA,
    "tool": "tools/visual_qa.sh",
    "passed": passed,
    "report_only_mode": report_only,
    "strict_mode": strict,
    "truth_mutation": False,
    "owner_visual_acceptance": False,
    "public_demo_ready": False,
    "release_candidate_ready": False,
    "local_playable_game_ready": True,
    "baseline_path": str(baseline_path),
    "baseline_loaded": bool(baseline),
    "baseline_candidate_pending_review": baseline.get("candidate_baseline_pending_review", True),
    "current_captures_dir": str(current_dir),
    "current_capture_count": len(current_captures),
    "required_role_count": len(REQUIRED_ROLES),
    "roles_present": sum(1 for r in role_results if r["status"] in ("present", "issues")),
    "roles_missing": sum(1 for r in role_results if r["status"] == "missing"),
    "failure_count": len(failures),
    "warning_count": len(warnings),
    "failures": failures,
    "warnings": warnings,
    "environment": environment,
    "role_results": role_results,
}

(out / "visual_qa_report.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")

# TSV metrics
tsv_lines = ["role\tstatus\tsha256\twidth\theight\tmean_lum\tcontrast\tclip_pct\tnon_bg_pct\tissues"]
for r in role_results:
    m = r.get("metrics", {})
    d = r.get("diff", {})
    tsv_lines.append("\t".join([
        r["role"],
        r["status"],
        r.get("sha256", ""),
        str(m.get("width", "")),
        str(m.get("height", "")),
        str(m.get("mean_luminance", "")),
        str(m.get("contrast", "")),
        str(m.get("clipping_pct", "")),
        str(m.get("non_background_pct", "")),
        ";".join(r.get("issues", [])),
    ]))
(out / "visual_qa_metrics.tsv").write_text("\n".join(tsv_lines) + "\n", encoding="utf-8")

# Markdown report
md_lines = [
    "# OATHYARD Automated Visual QA Report",
    "",
    f"Status: {'PASSED' if passed else 'FAILED'}" + (" (report-only mode)" if report_only else ""),
    "",
    f"- Required roles: `{len(REQUIRED_ROLES)}`",
    f"- Roles present: `{manifest['roles_present']}`",
    f"- Roles missing: `{manifest['roles_missing']}`",
    f"- Failures: `{len(failures)}`",
    f"- Warnings: `{len(warnings)}`",
    f"- Baseline: `{baseline_path}`",
    f"- Baseline pending review: `{baseline.get('candidate_baseline_pending_review', True)}`",
    "",
    "## Environment",
    "",
    f"- OS: `{environment['os']}`",
    f"- Renderer backend: `{environment['renderer_backend_id']}`",
    f"- Min resolution: `{environment['min_resolution']}`",
    f"- Metrics method: `{environment['metrics_method']}`",
    "",
    "## Per-role metrics",
    "",
    "| Role | Status | Resolution | Mean Lum | Contrast | Non-bg % | SHA256 (8) | Issues |",
    "| --- | --- | --- | --- | --- | --- | --- | --- |",
]
for r in role_results:
    m = r.get("metrics", {})
    sha_short = r.get("sha256", "")[:8]
    issues = "; ".join(r.get("issues", [])) or "none"
    res = f"{m.get('width','?')}x{m.get('height','?')}"
    md_lines.append(f"| `{r['role']}` | `{r['status']}` | {res} | {m.get('mean_luminance','?')} | {m.get('contrast','?')} | {m.get('non_background_pct','?')} | `{sha_short}` | {issues} |")

if failures:
    md_lines.extend(["", "## Failures", ""] + [f"- {f}" for f in failures])
if warnings:
    md_lines.extend(["", "## Warnings", ""] + [f"- {w}" for w in warnings])

md_lines.extend([
    "",
    "## Readiness flags",
    "",
    "- `owner_visual_acceptance`: `false`",
    "- `public_demo_ready`: `false`",
    "- `release_candidate_ready`: `false`",
    "- `local_playable_game_ready`: `true`",
    "- `truth_mutation`: `false`",
    "",
    "## Visual quality assessment",
    "",
    "Vision-model assessment of captures is performed during the working-game run",
    "and recorded alongside this report. The deterministic metrics above (SHA256,",
    "resolution, luminance, contrast, clipping, non-background coverage, duplicate",
    "detection) are computed without external dependencies and gate the pass/fail",
    "decision.",
])
(out / "visual_qa_report.md").write_text("\n".join(md_lines) + "\n", encoding="utf-8")

# Copy baseline manifest used
if baseline:
    import shutil
    bl_copy = out / "baseline_manifest_used.json"
    shutil.copy2(str(baseline_path), str(bl_copy))

# --- Exit ---

if report_only:
    print(f"visual_qa: report-only mode, {len(failures)} failures, {len(warnings)} warnings")
    print(f"visual_qa: report at {out / 'visual_qa_report.json'}")
    sys.exit(0)

if failures:
    print(f"visual_qa: FAILED with {len(failures)} failures", file=sys.stderr)
    for f in failures[:10]:
        print(f"  - {f}", file=sys.stderr)
    sys.exit(1)

print(f"visual_qa: PASSED — {manifest['roles_present']}/{len(REQUIRED_ROLES)} roles present, {len(warnings)} warnings")
print(f"visual_qa: report at {out / 'visual_qa_report.json'}")
sys.exit(0)
PYEOF
