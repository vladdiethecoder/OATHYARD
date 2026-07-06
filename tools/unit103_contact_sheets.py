#!/usr/bin/env python3
"""Unit-104: Contact sheet + visual scoring with pixel-based readability analysis.

Reads the matrix JSON produced by `oathyard play --capture-roster-matrix <dir>`,
performs pixel analysis on each screenshot to compute honest readability scores,
then generates:
  - asset_matrix_contact_sheet.png  (all 22 assets, with honest scores)
  - fighters_contact_sheet.png
  - weapons_contact_sheet.png
  - armor_contact_sheet.png
  - arenas_contact_sheet.png
  - roster_asset_capture_matrix.md  (with pixel-based scores)
  - visual_scores.json  (updated with pixel-based scores)
  - visual_scores.md

Uses PIL (Pillow) for image analysis and composition — no fabrication.
"""
from __future__ import annotations

import json
import math
import sys
from pathlib import Path

try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError:
    print("ERROR: PIL/Pillow not available", file=sys.stderr)
    sys.exit(1)


def load_matrix(matrix_dir: Path) -> dict:
    matrix_path = matrix_dir / "roster_asset_capture_matrix.json"
    if not matrix_path.exists():
        print(f"ERROR: matrix JSON not found at {matrix_path}", file=sys.stderr)
        sys.exit(1)
    return json.loads(matrix_path.read_text())


def analyze_screenshot(img_path: Path, kind: str) -> dict:
    """Perform pixel-based visual readability analysis."""
    if not img_path.exists():
        return {
            "foreground_coverage": 0.0,
            "edge_contrast": 0.0,
            "silhouette_nonuniformity": 0.0,
            "gameplay_readability_score": 0.0,
            "visibility_score": 0.0,
            "framing_score": 0.0,
            "contrast_score": 0.0,
            "silhouette_score": 0.0,
            "pixel_analysis_notes": "screenshot not found",
        }

    img = Image.open(img_path).convert("RGB")
    w, h = img.size

    # Sample center region (skip UI overlays at edges)
    x0 = w // 6
    x1 = w * 5 // 6
    y0 = h // 6
    y1 = h * 5 // 6

    # Background reference: average of 4 corners of sample region
    bg_pixels = [
        img.getpixel((x0 + 10, y0 + 10)),
        img.getpixel((x1 - 10, y0 + 10)),
        img.getpixel((x0 + 10, y1 - 10)),
        img.getpixel((x1 - 10, y1 - 10)),
    ]
    bg_r = sum(p[0] for p in bg_pixels) // 4
    bg_g = sum(p[1] for p in bg_pixels) // 4
    bg_b = sum(p[2] for p in bg_pixels) // 4

    # Threshold for foreground detection
    threshold = {"fighter": 25, "armor": 25, "weapon": 20, "arena": 30}.get(kind, 30)

    # Count foreground pixels with sampling
    fg_count = 0
    total = 0
    fg_colors: list[tuple[int, int, int]] = []
    pixels = img.load()
    for y in range(y0, y1, 2):
        for x in range(x0, x1, 2):
            total += 1
            raw = pixels[x, y]
            if isinstance(raw, (int, float)):
                r = g = b = int(raw)
            else:
                r, g, b = raw[0], raw[1], raw[2]
            dr = abs(r - bg_r)
            dg = abs(g - bg_g)
            db = abs(b - bg_b)
            if dr + dg + db > threshold * 3:
                fg_count += 1
                fg_colors.append((r, g, b))

    foreground_coverage = fg_count / total if total > 0 else 0.0

    # Edge contrast: horizontal gradient magnitude in foreground regions
    edge_sum = 0.0
    edge_count = 0
    for y in range(y0 + 2, y1 - 2, 4):
        for x in range(x0 + 2, x1 - 2, 4):
            raw1 = pixels[x, y]
            raw2 = pixels[x + 2, y]
            if raw1 is None or raw2 is None:
                continue
            p1 = (int(raw1),) * 3 if isinstance(raw1, (int, float)) else (raw1[0], raw1[1], raw1[2])
            p2 = (int(raw2),) * 3 if isinstance(raw2, (int, float)) else (raw2[0], raw2[1], raw2[2])
            grad = (abs(p1[0] - p2[0]) + abs(p1[1] - p2[1]) + abs(p1[2] - p2[2])) / 3.0
            if grad > 10.0:
                edge_sum += grad
                edge_count += 1

    edge_contrast = (edge_sum / edge_count / 255.0) if edge_count > 0 else 0.0

    # Silhouette non-uniformity: color variance of foreground
    n = len(fg_colors)
    if n > 10:
        r_mean = sum(c[0] for c in fg_colors) / n
        g_mean = sum(c[1] for c in fg_colors) / n
        b_mean = sum(c[2] for c in fg_colors) / n
        var = sum(
            (c[0] - r_mean) ** 2 + (c[1] - g_mean) ** 2 + (c[2] - b_mean) ** 2
            for c in fg_colors
        ) / n / 3.0
        silhouette_nonuniformity = math.sqrt(var) / 255.0
    else:
        silhouette_nonuniformity = 0.0

    # Compute individual scores (0-5 scale each)
    # Visibility: foreground coverage
    cov_thresholds = {
        "fighter": (0.08, 0.03),
        "weapon": (0.05, 0.02),
        "armor": (0.06, 0.03),
        "arena": (0.15, 0.08),
    }
    high_cov, med_cov = cov_thresholds.get(kind, (0.08, 0.03))
    visibility_score = (
        5.0 if foreground_coverage > high_cov * 2
        else 4.0 if foreground_coverage > high_cov * 1.5
        else 3.0 if foreground_coverage > high_cov
        else 2.0 if foreground_coverage > med_cov
        else 1.0 if foreground_coverage > med_cov * 0.5
        else 0.0
    )

    # Framing: asset centered and not clipped (based on coverage being in a good range)
    # Too little = too far, too much = clipped
    optimal_ranges = {
        "fighter": (0.03, 0.80),
        "weapon": (0.03, 0.50),
        "armor": (0.04, 0.35),
        "arena": (0.10, 0.60),
    }
    opt_low, opt_high = optimal_ranges.get(kind, (0.05, 0.30))
    if opt_low <= foreground_coverage <= opt_high:
        framing_score = 4.0 if foreground_coverage > opt_low * 1.5 else 3.0
    elif foreground_coverage < opt_low:
        framing_score = 1.0 if foreground_coverage > opt_low * 0.5 else 0.0
    else:
        framing_score = 2.0  # clipped but visible

    # Contrast: edge detail
    contrast_score = (
        5.0 if edge_contrast > 0.15
        else 4.0 if edge_contrast > 0.10
        else 3.0 if edge_contrast > 0.08
        else 2.0 if edge_contrast > 0.04
        else 1.0 if edge_contrast > 0.02
        else 0.0
    )

    # Silhouette: color variation (distinguishable features)
    silhouette_score = (
        5.0 if silhouette_nonuniformity > 0.15
        else 4.0 if silhouette_nonuniformity > 0.10
        else 3.0 if silhouette_nonuniformity > 0.08
        else 2.0 if silhouette_nonuniformity > 0.05
        else 1.0 if silhouette_nonuniformity > 0.02
        else 0.0
    )

    # Gameplay readability: composite (weighted average)
    gameplay_readability = (
        visibility_score * 0.30
        + framing_score * 0.25
        + contrast_score * 0.25
        + silhouette_score * 0.20
    )

    notes_parts = []
    if foreground_coverage < med_cov:
        notes_parts.append(f"low coverage {foreground_coverage:.1%}")
    if edge_contrast < 0.04:
        notes_parts.append(f"weak contrast {edge_contrast:.3f}")
    if silhouette_nonuniformity < 0.05:
        notes_parts.append(f"flat appearance {silhouette_nonuniformity:.3f}")
    if not notes_parts:
        notes_parts.append("pixel analysis: adequate visibility + contrast")

    return {
        "foreground_coverage": round(foreground_coverage, 4),
        "edge_contrast": round(edge_contrast, 4),
        "silhouette_nonuniformity": round(silhouette_nonuniformity, 4),
        "visibility_score": round(visibility_score, 1),
        "framing_score": round(framing_score, 1),
        "contrast_score": round(contrast_score, 1),
        "silhouette_score": round(silhouette_score, 1),
        "gameplay_readability_score": round(gameplay_readability, 1),
        "pixel_analysis_notes": "; ".join(notes_parts),
    }


def compute_visual_status(readability: float, mesh_consumed: bool, capture_ok: bool) -> tuple[str, str]:
    """Honest pass/warn/fail based on pixel analysis + geometry data."""
    if not capture_ok or not mesh_consumed:
        return "fail", "capture failed or mesh not consumed"
    if readability >= 3.0:
        return "pass", f"gameplay-readable (score {readability:.1f}/5.0)"
    elif readability >= 1.5:
        return "warn", f"marginal readability (score {readability:.1f}/5.0)"
    else:
        return "fail", f"poor readability (score {readability:.1f}/5.0)"


def make_contact_sheet(
    assets: list[dict],
    matrix_dir: Path,
    output_name: str,
    title: str,
    cols: int = 4,
    thumb_w: int = 480,
    thumb_h: int = 270,
) -> Path:
    """Create a contact sheet PNG from a list of asset entries."""
    rows = (len(assets) + cols - 1) // cols
    label_h = 40
    cell_w = thumb_w
    cell_h = thumb_h + label_h
    padding = 10
    title_h = 50

    canvas_w = cols * cell_w + (cols + 1) * padding
    canvas_h = rows * cell_h + (rows + 1) * padding + title_h

    canvas = Image.new("RGB", (canvas_w, canvas_h), (20, 20, 24))
    draw = ImageDraw.Draw(canvas)

    try:
        font_title = ImageFont.truetype(
            "/usr/share/fonts/liberation-sans/LiberationSans-Bold.ttf", 24
        )
        font_label = ImageFont.truetype(
            "/usr/share/fonts/liberation-sans/LiberationSans-Regular.ttf", 14
        )
        font_score = ImageFont.truetype(
            "/usr/share/fonts/liberation-sans/LiberationSans-Bold.ttf", 12
        )
    except (OSError, IOError):
        font_title = ImageFont.load_default()
        font_label = ImageFont.load_default()
        font_score = ImageFont.load_default()

    draw.text((padding, 10), title, fill=(255, 255, 255), font=font_title)

    for idx, asset in enumerate(assets):
        col = idx % cols
        row = idx // cols
        x = padding + col * (cell_w + padding)
        y = title_h + padding + row * (cell_h + padding)

        asset_id = asset["asset_id"]
        kind = asset["kind"]
        status = asset.get("visual_status", "unknown")
        readability = asset.get("gameplay_readability_score", 0.0)

        screenshot_path = matrix_dir / asset.get("screenshot_path", "")
        if screenshot_path.exists():
            try:
                img = Image.open(screenshot_path).convert("RGB")
                img.thumbnail((thumb_w, thumb_h), Image.Resampling.LANCZOS)
                paste_x = x + (cell_w - img.width) // 2
                paste_y = y + (thumb_h - img.height) // 2
                canvas.paste(img, (paste_x, paste_y))
            except Exception as e:
                draw.text(
                    (x + 10, y + thumb_h // 2),
                    f"IMG ERROR: {e}",
                    fill=(255, 0, 0),
                    font=font_label,
                )
        else:
            draw.rectangle([x, y, x + thumb_w, y + thumb_h], fill=(40, 20, 20))
            draw.text(
                (x + 10, y + thumb_h // 2),
                "NO SCREENSHOT",
                fill=(255, 80, 80),
                font=font_label,
            )

        status_color = {"pass": (80, 255, 80), "warn": (255, 200, 80), "fail": (255, 80, 80)}.get(
            status, (200, 200, 200)
        )
        draw.rectangle([x, y + thumb_h, x + cell_w, y + cell_h], fill=(30, 30, 36))
        draw.text((x + 5, y + thumb_h + 3), f"{asset_id}", fill=(255, 255, 255), font=font_label)
        score_text = f"[{status}] {readability:.1f}/5"
        draw.text((x + cell_w - 100, y + thumb_h + 5), score_text, fill=status_color, font=font_score)

    output_path = matrix_dir / output_name
    canvas.save(str(output_path), "PNG")
    return output_path


def update_matrix_with_pixel_scores(matrix: dict, matrix_dir: Path) -> dict:
    """Update matrix assets with pixel-based visual scores."""
    for asset in matrix.get("assets", []):
        screenshot_path = matrix_dir / asset.get("screenshot_path", "")
        kind = asset.get("kind", "unknown")
        mesh_consumed = asset.get("mesh_geometry_consumed", False)

        # Check if screenshot exists (capture_ok)
        capture_ok = screenshot_path.exists() and screenshot_path.stat().st_size > 1000

        # Run pixel analysis
        scores = analyze_screenshot(screenshot_path, kind)
        asset.update(scores)

        # Compute honest visual status
        status, notes = compute_visual_status(
            scores["gameplay_readability_score"], mesh_consumed, capture_ok
        )
        asset["visual_status"] = status
        asset["visual_notes"] = notes

    # Update capture summary
    statuses = [a.get("visual_status", "fail") for a in matrix.get("assets", [])]
    summary = matrix.get("capture_summary", {})
    summary["captured"] = sum(1 for s in statuses if s in ("pass", "warn"))
    summary["failed"] = sum(1 for s in statuses if s == "fail")
    summary["passed"] = sum(1 for s in statuses if s == "pass")
    summary["warned"] = sum(1 for s in statuses if s == "warn")
    matrix["capture_summary"] = summary

    # Write updated matrix back
    matrix_path = matrix_dir / "roster_asset_capture_matrix.json"
    matrix_path.write_text(json.dumps(matrix, indent=2) + "\n")

    return matrix


def generate_markdown(matrix: dict, matrix_dir: Path, sheets: dict[str, Path]) -> None:
    md_path = matrix_dir / "roster_asset_capture_matrix.md"
    lines = [
        "# OATHYARD Roster Asset Capture Matrix (Unit-104)",
        "",
        f"**Schema**: `{matrix.get('schema', 'unknown')}`",
        f"**Generated by executable**: `{matrix.get('generated_by_executable', False)}`",
        f"**Command**: `{matrix.get('executable_command', 'N/A')}`",
        f"**Package-relative paths only**: `{matrix.get('package_relative_paths_only', True)}`",
        "",
        "## Readiness Boundary",
        "",
        f"- `truth_mutation`: **{matrix.get('truth_mutation', 'N/A')}**",
        f"- `production_asset_ready`: **{matrix.get('production_asset_ready', 'N/A')}**",
        f"- `owner_visual_accepted`: **{matrix.get('owner_visual_accepted', 'N/A')}**",
        f"- `public_demo_ready`: **{matrix.get('public_demo_ready', 'N/A')}**",
        f"- `release_candidate_ready`: **{matrix.get('release_candidate_ready', 'N/A')}**",
        "",
        "## Asset Counts",
        "",
        f"- Total: **{matrix.get('asset_count_total', 'N/A')}**",
    ]
    kc = matrix.get("kind_counts", {})
    for kind in ["fighter", "weapon", "armor", "arena"]:
        lines.append(f"  - {kind}: **{kc.get(kind, 'N/A')}**")

    summary = matrix.get("capture_summary", {})
    lines.extend(
        [
            "",
            "## Capture Summary (Pixel-Analyzed)",
            "",
            f"- Passed: **{summary.get('passed', 'N/A')}**",
            f"- Warned: **{summary.get('warned', 'N/A')}**",
            f"- Failed: **{summary.get('failed', 'N/A')}**",
            f"- Resolution: **{matrix.get('capture_resolution', 'N/A')}**",
            f"- Backend: `{matrix.get('renderer_backend', 'N/A')}`",
            "",
            "## Contact Sheets",
            "",
        ]
    )
    for name, path in sheets.items():
        lines.append(f"- **{name}**: `{path.name}`")

    lines.extend(
        [
            "",
            "## Per-Asset Details (Pixel-Analyzed)",
            "",
            "| Asset | Kind | Status | Readability | Visibility | Framing | Contrast | Silhouette | Coverage | EdgeContrast |",
            "|---|---|---|---|---|---|---|---|---|---|",
        ]
    )

    for asset in matrix.get("assets", []):
        lines.append(
            f"| {asset['asset_id']} | {asset['kind']} | **{asset.get('visual_status', '?')}** | "
            f"{asset.get('gameplay_readability_score', 0):.1f} | "
            f"{asset.get('visibility_score', 0):.1f} | "
            f"{asset.get('framing_score', 0):.1f} | "
            f"{asset.get('contrast_score', 0):.1f} | "
            f"{asset.get('silhouette_score', 0):.1f} | "
            f"{asset.get('foreground_coverage', 0):.1%} | "
            f"{asset.get('edge_contrast', 0):.3f} |"
        )

    # Honest summary of issues
    warns = [a for a in matrix.get("assets", []) if a.get("visual_status") == "warn"]
    fails = [a for a in matrix.get("assets", []) if a.get("visual_status") == "fail"]
    if warns or fails:
        lines.extend(["", "## Honest Issues", ""])
        for a in warns:
            lines.append(f"- **WARN** `{a['asset_id']}`: {a.get('visual_notes', 'unknown')}")
        for a in fails:
            lines.append(f"- **FAIL** `{a['asset_id']}`: {a.get('visual_notes', 'unknown')}")

    md_path.write_text("\n".join(lines) + "\n")


def generate_scores_json_and_md(matrix: dict, matrix_dir: Path) -> None:
    scores_path = matrix_dir / "visual_scores.json"
    md_path = matrix_dir / "visual_scores.md"

    scores = []
    for a in matrix.get("assets", []):
        scores.append({
            "asset_id": a["asset_id"],
            "kind": a["kind"],
            "visual_status": a.get("visual_status", "unknown"),
            "gameplay_readability_score": a.get("gameplay_readability_score", 0),
            "visibility_score": a.get("visibility_score", 0),
            "framing_score": a.get("framing_score", 0),
            "contrast_score": a.get("contrast_score", 0),
            "silhouette_score": a.get("silhouette_score", 0),
            "foreground_coverage": a.get("foreground_coverage", 0),
            "edge_contrast": a.get("edge_contrast", 0),
            "silhouette_nonuniformity": a.get("silhouette_nonuniformity", 0),
            "vertex_count": a.get("vertex_count", 0),
            "triangle_count": a.get("triangle_count", 0),
            "mesh_geometry_consumed": a.get("mesh_geometry_consumed", False),
            "material_texture_binding": a.get("material_texture_binding", False),
            "notes": a.get("visual_notes", ""),
            "pixel_analysis_notes": a.get("pixel_analysis_notes", ""),
        })

    scores_json = {
        "schema": "oathyard.roster_visual_scores.v1",
        "generated_by_executable": True,
        "asset_count": len(scores),
        "scoring_method": "pixel_analysis",
        "scores": scores,
    }
    scores_path.write_text(json.dumps(scores_json, indent=2) + "\n")

    lines = [
        "# Visual Scores (Unit-104 — Pixel-Analyzed)",
        "",
        f"Asset count: **{len(scores)}**",
        f"Scoring method: **pixel analysis** (foreground coverage, edge contrast, silhouette variance)",
        "",
        "| Asset | Kind | Status | Readability | Visibility | Framing | Contrast | Silhouette | Coverage |",
        "|---|---|---|---|---|---|---|---|---|",
    ]
    for s in scores:
        lines.append(
            f"| {s['asset_id']} | {s['kind']} | **{s['visual_status']}** | "
            f"{s['gameplay_readability_score']:.1f} | "
            f"{s['visibility_score']:.1f} | "
            f"{s['framing_score']:.1f} | "
            f"{s['contrast_score']:.1f} | "
            f"{s['silhouette_score']:.1f} | "
            f"{s['foreground_coverage']:.1%} |"
        )
    md_path.write_text("\n".join(lines) + "\n")


def main() -> None:
    if len(sys.argv) < 2:
        print("usage: generate_contact_sheets.py <matrix_dir>", file=sys.stderr)
        sys.exit(1)
    matrix_dir = Path(sys.argv[1])
    matrix = load_matrix(matrix_dir)

    # Unit-104: Run pixel analysis to update visual scores honestly
    matrix = update_matrix_with_pixel_scores(matrix, matrix_dir)

    assets = matrix.get("assets", [])

    sheets: dict[str, Path] = {}

    # Full contact sheet
    sheets["All Assets"] = make_contact_sheet(
        assets,
        matrix_dir,
        "asset_matrix_contact_sheet.png",
        f"OATHYARD Roster Capture Matrix — All 22 Assets (Pixel-Analyzed)",
    )

    # Per-kind sheets
    for kind in ["fighter", "weapon", "armor", "arena"]:
        kind_assets = [a for a in assets if a["kind"] == kind]
        if kind_assets:
            plural = {"fighter": "Fighters", "weapon": "Weapons", "armor": "Armor", "arena": "Arenas"}[kind]
            output_name = f"{kind}s_contact_sheet.png"
            if kind == "armor":
                output_name = "armor_contact_sheet.png"
            sheets[plural] = make_contact_sheet(
                kind_assets,
                matrix_dir,
                output_name,
                f"OATHYARD {plural} ({len(kind_assets)} assets — Pixel-Analyzed)",
                cols=min(len(kind_assets), 4),
            )

    generate_markdown(matrix, matrix_dir, sheets)
    generate_scores_json_and_md(matrix, matrix_dir)

    # Print summary
    summary = matrix.get("capture_summary", {})
    print(f"Contact sheets + pixel analysis complete in {matrix_dir}")
    print(f"  Passed: {summary.get('passed', 0)}")
    print(f"  Warned: {summary.get('warned', 0)}")
    print(f"  Failed: {summary.get('failed', 0)}")
    for name, path in sheets.items():
        print(f"  {name}: {path.name}")


if __name__ == "__main__":
    main()
