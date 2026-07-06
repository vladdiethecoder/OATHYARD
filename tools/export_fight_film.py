#!/usr/bin/env python3
"""
Unit-108: Fight-film export pipeline.

Post-processes the fight-film frame sequence produced by the native renderer:
  1. Updates the uncut manifest with video encoding info
  2. Encodes MP4 video from PNG frame sequence
  3. Generates a contact sheet (montage of key frames)
  4. Produces export report

Usage:
  python3 tools/export_fight_film.py <out_dir> [--cleanup-frames]
"""

import argparse
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path


def sha256_file(path: Path) -> str:
    import hashlib
    h = hashlib.sha256()
    h.update(path.read_bytes())
    return h.hexdigest()


def probe_video(path: Path) -> dict:
    """Get video metadata via ffprobe."""
    try:
        result = subprocess.run(
            ["ffprobe", "-v", "quiet", "-print_format", "json", "-show_format", "-show_streams",
             str(path)],
            capture_output=True, text=True, timeout=30
        )
        info = json.loads(result.stdout)
        streams = info.get("streams", [])
        video_stream = next((s for s in streams if s.get("codec_type") == "video"), {})
        return {
            "duration_seconds": float(info.get("format", {}).get("duration", 0)),
            "codec": video_stream.get("codec_name", "unknown"),
            "width": video_stream.get("width", 0),
            "height": video_stream.get("height", 0),
            "fps": video_stream.get("r_frame_rate", "0/0"),
        }
    except Exception as e:
        return {"error": str(e)}


def generate_contact_sheet(frames_dir: Path, output_path: Path, cols: int = 8,
                           thumbnail_width: int = 240, max_frames: int = 64):
    """Generate a contact sheet PNG from the fight-film frames using ffmpeg."""
    frames = sorted(frames_dir.glob("ff_*.png"))
    if not frames:
        print("  WARNING: no frames found for contact sheet")
        return False

    # Sample evenly across available frames
    total = len(frames)
    sample_count = min(total, max_frames)
    step = max(1, total // sample_count)
    sampled = frames[::step][:sample_count]

    # Use ffmpeg tile filter to create a contact sheet
    rows = (sample_count + cols - 1) // cols
    try:
        # Create a temporary directory with symlinks matching the naming pattern
        # ffmpeg's tile filter needs sequential naming
        tmp_dir = frames_dir.parent / ".contact_sheet_tmp"
        tmp_dir.mkdir(exist_ok=True)
        for i, f in enumerate(sampled):
            link = tmp_dir / f"img_{i:04d}.png"
            if not link.exists():
                os.symlink(f.resolve(), link)

        # Build tile grid
        cmd = [
            "ffmpeg", "-y",
            "-pattern_type", "sequence",
            "-i", str(tmp_dir / "img_%04d.png"),
            "-vf", f"tile={cols}x{rows}:margin=2:padding=2",
            "-frames:v", "1",
            str(output_path),
        ]
        subprocess.run(cmd, capture_output=True, timeout=120, check=True)

        # Clean up tmp dir
        for f in tmp_dir.iterdir():
            f.unlink()
        tmp_dir.rmdir()

        print(f"  contact sheet: {output_path} ({sample_count} frames)")
        return True
    except subprocess.CalledProcessError as e:
        print(f"  WARNING: contact sheet failed: {e.stderr.decode()[:200]}")
        return False
    except Exception as e:
        print(f"  WARNING: contact sheet error: {e}")
        return False


def encode_video(frames_dir: Path, output_path: Path, fps: int = 12):
    """Encode MP4 from PNG frame sequence using ffmpeg."""
    frames = sorted(frames_dir.glob("ff_*.png"))
    if not frames:
        print("  WARNING: no frames found for video encoding")
        return None

    # Try available encoders in order of preference
    encoder_priority = [
        ("h264_nvenc", "mp4", {"-preset": "p7", "-tune": "hq", "-cq": "23"}),
        ("libopenh264", "mp4", {"-b:v": "5M"}),
        ("h264_vaapi", "mp4", {}),
        ("mpeg4", "mp4", {"-b:v": "5M", "-q:v": "3"}),
        ("libvpx-vp9", "webm", {"-b:v": "2M", "-cpu-used": "2"}),
    ]

    # Check which encoder is available
    try:
        encoders_out = subprocess.run(
            ["ffmpeg", "-encoders"], capture_output=True, text=True, timeout=10
        ).stdout
    except Exception:
        encoders_out = ""

    for enc_name, container, extra_opts in encoder_priority:
        if enc_name not in encoders_out and enc_name != "mpeg4":
            print(f"  encoder '{enc_name}' not available, trying next...")
            continue

        ext = container if container else "mp4"
        output_path = output_path.with_suffix(f".{ext}")

        try:
            cmd = [
                "ffmpeg", "-y",
                "-framerate", str(fps),
                "-pattern_type", "glob",
                "-i", str(frames_dir / "ff_*.png"),
                "-c:v", enc_name,
            ]
            for k, v in extra_opts.items():
                cmd.extend([k, str(v)])
            cmd.extend([
                "-pix_fmt", "yuv420p",
                "-vf", "scale=1920:1080:force_original_aspect_ratio=decrease,pad=1920:1080:(ow-iw)/2:(oh-ih)/2",
                str(output_path),
            ])

            result = subprocess.run(cmd, capture_output=True, timeout=300)
            if result.returncode != 0:
                stderr = result.stderr.decode()[-500:]
                print(f"  encoder '{enc_name}' failed: {stderr[:200]}")
                continue

            if output_path.exists() and output_path.stat().st_size > 0:
                sha = sha256_file(output_path)
                probe = probe_video(output_path)
                print(f"  video: {output_path}")
                print(f"  encoder: {enc_name}")
                print(f"  sha256: {sha}")
                print(f"  probe: {probe}")
                return {"sha256": sha, "probe": probe, "encoder": enc_name, "path": str(output_path)}
            else:
                print(f"  encoder '{enc_name}' produced empty output")

        except subprocess.TimeoutExpired:
            print(f"  encoder '{enc_name}' timed out")
        except Exception as e:
            print(f"  encoder '{enc_name}' error: {e}")

    print("  ERROR: no encoder succeeded")
    return None


def main():
    parser = argparse.ArgumentParser(description="Fight-film export pipeline")
    parser.add_argument("out_dir", help="Output directory (same as --out passed to renderer)")
    parser.add_argument("--cleanup-frames", action="store_true",
                        help="Remove frame PNGs after encoding")
    args = parser.parse_args()

    out_dir = Path(args.out_dir).resolve()
    if not out_dir.exists():
        print(f"ERROR: output directory not found: {out_dir}")
        sys.exit(1)

    manifest_path = out_dir / "fight_film_uncut_manifest.json"
    if not manifest_path.exists():
        print(f"No fight_film_uncut_manifest.json found in {out_dir}")
        print("(Fight film may not have been played)")
        sys.exit(0)

    manifest = json.loads(manifest_path.read_text())
    frames_dir = out_dir / "fight_film_frames"

    print(f"=== fight-film export ===")
    print(f"out_dir: {out_dir}")
    print(f"manifest: {manifest_path}")
    print(f"frames_dir: {frames_dir}")
    print(f"turn_count: {manifest.get('turn_count')}")
    print(f"frame_count: {manifest.get('frame_count')}")
    print(f"final_truth_hash: {manifest.get('final_truth_hash')}")

    frame_count_before = 0
    if frames_dir.exists():
        frame_count_before = len(list(frames_dir.glob("ff_*.png")))
    print(f"frames on disk: {frame_count_before}")

    # Step 1: Generate contact sheet
    contact_sheet_path = out_dir / "fight_film_contact_sheet.png"
    if not contact_sheet_path.exists():
        print("\n--- Generating contact sheet ---")
        generate_contact_sheet(frames_dir, contact_sheet_path)
    else:
        print(f"\n--- Contact sheet already exists: {contact_sheet_path} ---")

    # Step 2: Encode video
    video_path = out_dir / "fight_film_uncut.mp4"
    encoded_info = None
    if not video_path.exists() and frames_dir.exists():
        print("\n--- Encoding video ---")
        encoded_info = encode_video(frames_dir, output_path=video_path)
    elif video_path.exists():
        print(f"\n--- Video already exists: {video_path} ---")
        encoded_info = {"sha256": sha256_file(video_path), "probe": probe_video(video_path)}

    # Step 3: Update manifest with encoding info
    if encoded_info:
        manifest["encoded_video_present"] = True
        manifest["encoder_name"] = "ffmpeg"
        manifest["video_sha256"] = encoded_info.get("sha256", "")
        # Check ffmpeg version
        try:
            ver = subprocess.run(["ffmpeg", "-version"], capture_output=True, text=True, timeout=10)
            manifest["encoder_version"] = ver.stdout.split("\n")[0] if ver.stdout else "unknown"
        except Exception:
            manifest["encoder_version"] = "unknown"

        if "probe" in encoded_info and encoded_info["probe"]:
            p = encoded_info["probe"]
            if "duration_seconds" in p and p["duration_seconds"]:
                manifest["duration_seconds"] = p["duration_seconds"]
            if "fps" in p and p["fps"]:
                manifest["video_fps"] = p["fps"]

    # Update frame count from actual files
    if frames_dir.exists():
        actual_frames = len(list(frames_dir.glob("ff_*.png")))
        manifest["frame_count"] = actual_frames

    manifest_path.write_text(json.dumps(manifest, indent=2))
    print(f"\n--- Manifest updated: {manifest_path} ---")

    # Step 4: Generate export report
    report_path = out_dir / "fight_film_export_report.md"
    lines = []
    lines.append("# Fight-film export report")
    lines.append("")
    lines.append(f"- Out dir: `{out_dir}`")
    lines.append(f"- Schema: `oathyard.uncut_fight_film.v1`")
    lines.append(f"- Truth hash: `{manifest.get('final_truth_hash', 'unknown')}`")
    lines.append(f"- Turn count: {manifest.get('turn_count', 0)}")
    lines.append(f"- Frame count: {manifest.get('frame_count', 0)}")
    lines.append(f"- FPS: {manifest.get('fps', 0)}")
    lines.append(f"- Duration: {manifest.get('duration_seconds', 0):.2f}s")
    lines.append(f"- Truth mutation: {manifest.get('truth_mutation', 'unknown')}")
    lines.append(f"- Omitted turn count: {manifest.get('omitted_turn_count', 'unknown')}")
    lines.append(f"- Fabricated contact count: {manifest.get('fabricated_contact_count', 'unknown')}")
    lines.append(f"- Encoded video: {manifest.get('encoded_video_present', False)}")
    lines.append(f"- Encoder: {manifest.get('encoder_name', 'none')}")
    lines.append(f"- Video SHA256: `{manifest.get('video_sha256', '')}`")
    lines.append("")

    # Files present
    lines.append("## Files produced")
    for f in ["fight_film_uncut_manifest.json", "fight_film_timeline.tsv",
              "fight_film_contact_sheet.png", "fight_film_uncut.mp4"]:
        p = out_dir / f
        if p.exists():
            sz = p.stat().st_size
            lines.append(f"- `{f}` ({sz:,} bytes)")
        else:
            lines.append(f"- `{f}` — NOT PRESENT")

    lines.append("")
    lines.append("## Frames directory")
    if frames_dir.exists():
        frame_files = sorted(frames_dir.glob("ff_*.png"))
        lines.append(f"- {len(frame_files)} frames in `{frames_dir}`")
        if frame_files:
            lines.append(f"- First: `{frame_files[0].name}`")
            lines.append(f"- Last: `{frame_files[-1].name}`")
    else:
        lines.append("- No frames directory found")

    lines.append("")
    lines.append("## Readiness flags")
    for flag in ["owner_visual_acceptance", "public_demo_ready", "release_candidate_ready",
                 "production_asset_ready"]:
        val = manifest.get(flag, "NOT SET")
        lines.append(f"- `{flag}`: {val}")

    report_path.write_text("\n".join(lines) + "\n")
    print(f"\n--- Export report: {report_path} ---")

    # Step 5: Cleanup (optional)
    if args.cleanup_frames:
        if frames_dir.exists():
            import shutil
            shutil.rmtree(frames_dir)
            print(f"--- Frames cleaned up: {frames_dir} ---")

    print("\n=== fight-film export complete ===")


if __name__ == "__main__":
    main()
