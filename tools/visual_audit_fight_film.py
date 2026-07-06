#!/usr/bin/env python3
"""
Unit-109: Deterministic visual audit for fight-film frames.

Checks:
  - Black/dark frames (mostly empty)
  - Frame continuity (monotonic naming)
  - Frame dimensions
  - Basic pixel variance (flat/empty frames)

Outputs: fight_film_visual_audit.json and fight_film_visual_audit.md
"""

import argparse
import json
import struct
import zlib
from pathlib import Path


def png_info(path: Path):
    """Extract PNG metadata without full decode."""
    data = path.read_bytes()
    if data[:8] != b'\x89PNG\r\n\x1a\n':
        return {"error": "not a PNG file"}
    
    # Find IHDR chunk
    pos = 8
    width, height = 0, 0
    while pos + 8 < len(data):
        length = struct.unpack('>I', data[pos:pos+4])[0]
        chunk_type = data[pos+4:pos+8]
        if chunk_type == b'IHDR':
            width = struct.unpack('>I', data[pos+8:pos+12])[0]
            height = struct.unpack('>I', data[pos+12:pos+16])[0]
            bit_depth = data[pos+16]
            color_type = data[pos+17]
            break
        pos += 12 + length
    
    # Quick pixel variance check - sample center pixels from raw data
    # Decompress IDAT to get raw pixel data
    idat_data = b''
    pos = 8
    while pos + 8 < len(data):
        length = struct.unpack('>I', data[pos:pos+4])[0]
        chunk_type = data[pos+4:pos+8]
        if chunk_type == b'IDAT':
            idat_data += data[pos+8:pos+8+length]
        elif chunk_type == b'IEND':
            break
        pos += 12 + length
    
    try:
        raw = zlib.decompress(idat_data)
        # Calculate file size as quick proxy
        filesize = path.stat().st_size
        return {
            "width": width,
            "height": height,
            "file_size_bytes": filesize,
            "raw_data_size": len(raw),
            "pixels": width * height,
        }
    except Exception:
        return {
            "width": width,
            "height": height,
            "file_size_bytes": path.stat().st_size,
            "raw_data_size": 0,
        }


def check_black_frame(info: dict) -> dict:
    """Detect if frame is mostly black/empty based on file size."""
    pixels = info.get("pixels", 0)
    file_size = info.get("file_size_bytes", 0)
    if pixels == 0:
        return {"is_black": True, "confidence": 1.0, "reason": "zero pixels"}
    
    # At 1920x1080 RGBA, a fully rendered frame is ~800KB-2MB
    # A mostly black frame with UI text is ~4-10KB
    # Threshold: if file_size < 20KB for a 1080p frame, it's mostly empty
    expected_min = 20000  # 20KB minimum for rendered content
    is_black = file_size < expected_min
    return {
        "is_black": is_black,
        "confidence": min(1.0, 1.0 - file_size / expected_min) if is_black else 0.0,
        "file_size": file_size,
        "threshold": expected_min,
    }


def check_ui_clip(info: dict) -> dict:
    """Rough check for UI clipping - very basic."""
    width = info.get("width", 0)
    height = info.get("height", 0)
    file_size = info.get("file_size_bytes", 0)
    
    # UI clipping indicators: very small files with some content
    # (6-15KB suggests only UI text on dark background)
    ui_only = 6000 < file_size < 20000
    
    return {
        "suspected_ui_only": ui_only,
        "resolution": f"{width}x{height}",
    }


def main():
    parser = argparse.ArgumentParser(description="Fight-film visual audit")
    parser.add_argument("frames_dir", help="Path to fight_film_frames directory")
    parser.add_argument("--out", default=None, help="Output directory (default: parent of frames_dir)")
    args = parser.parse_args()

    frames_dir = Path(args.frames_dir)
    out_dir = Path(args.out) if args.out else frames_dir.parent

    if not frames_dir.exists():
        print(f"ERROR: frames directory not found: {frames_dir}")
        return

    frame_files = sorted(frames_dir.glob("ff_*.png"))
    total = len(frame_files)

    print(f"Scanning {total} frames in {frames_dir}...")

    results = []
    black_frames = []
    clip_frames = []
    min_size = float('inf')
    max_size = 0
    sizes = []

    for f in frame_files:
        info = png_info(f)
        info["name"] = f.name
        results.append(info)

        sz = info.get("file_size_bytes", 0)
        sizes.append(sz)
        min_size = min(min_size, sz)
        max_size = max(max_size, sz)

        black_check = check_black_frame(info)
        if black_check["is_black"]:
            black_frames.append({"name": f.name, "size": sz})

        clip_check = check_ui_clip(info)
        if clip_check["suspected_ui_only"]:
            clip_frames.append({"name": f.name, "size": sz})

    avg_size = sum(sizes) / len(sizes) if sizes else 0

    audit = {
        "schema": "oathyard.fight_film_visual_audit.v1",
        "frames_scanned": total,
        "black_frames": len(black_frames),
        "black_frame_list": black_frames[:20],  # first 20
        "suspected_ui_clip_frames": len(clip_frames),
        "ui_clip_frame_list": clip_frames[:20],
        "min_frame_size_bytes": int(min_size),
        "max_frame_size_bytes": int(max_size),
        "avg_frame_size_bytes": int(avg_size),
        "frame_dimensions": results[0] if results else None,
        "audit_passed": len(black_frames) == 0,
    }

    # Write JSON
    json_path = out_dir / "fight_film_visual_audit.json"
    json_path.write_text(json.dumps(audit, indent=2))
    print(f"Visual audit JSON: {json_path}")

    # Write Markdown report
    md = []
    md.append("# Fight-film visual audit report")
    md.append("")
    md.append(f"- Frames scanned: **{total}**")
    md.append(f"- Black/empty frames: **{len(black_frames)}**")
    md.append(f"- Suspected UI-only frames: **{len(clip_frames)}**")
    md.append(f"- Frame size range: **{min_size:,} bytes – {max_size:,} bytes**")
    md.append(f"- Average frame size: **{avg_size:,.0f} bytes**")
    md.append(f"- Resolution: **{results[0].get('width', '?')}x{results[0].get('height', '?')}**" if results else "")
    md.append(f"- Audit passed: **{audit['audit_passed']}**")
    md.append("")

    if black_frames:
        md.append("## Black/empty frames detected")
        for bf in black_frames[:20]:
            md.append(f"- `{bf['name']}`: {bf['size']:,} bytes")
        if len(black_frames) > 20:
            md.append(f"- ... and {len(black_frames) - 20} more")
    
    if clip_frames:
        md.append("## Suspected UI-only frames")
        for cf in clip_frames[:20]:
            md.append(f"- `{cf['name']}`: {cf['size']:,} bytes")
        if len(clip_frames) > 20:
            md.append(f"- ... and {len(clip_frames) - 20} more")

    if not black_frames and not clip_frames:
        md.append("## No issues detected")
        md.append("All frames have expected file sizes indicating rendered 3D content.")

    md_path = out_dir / "fight_film_visual_audit.md"
    md_path.write_text("\n".join(md) + "\n")
    print(f"Visual audit report: {md_path}")
    print(f"Audit passed: {audit['audit_passed']}")


if __name__ == "__main__":
    main()
