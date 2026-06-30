#!/usr/bin/env python3
import json
import statistics
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BIN = ROOT / "target" / "debug" / "oathyard"
SCENARIO = ROOT / "examples" / "duels" / "basic_oathyard.duel"
PACKAGE = ROOT / "artifacts" / "package" / "oathyard-linux-x86_64.tar"


def main() -> int:
    out_dir = Path(sys.argv[1]) if len(sys.argv) == 2 else ROOT / "artifacts" / "perf" / "latest"
    if not out_dir.is_absolute():
        out_dir = ROOT / out_dir
    out_dir.mkdir(parents=True, exist_ok=True)
    ensure_binary()

    duel_dir = out_dir / "duel_measure"
    render_dir = out_dir / "native_render_measure"
    timings = {
        "duel_run_ms": measure_command(
            [BIN, "run", "--scenario", SCENARIO, "--out", duel_dir],
            samples=5,
        ),
        "replay_verify_ms": measure_command(
            [BIN, "replay", "--replay", duel_dir / "replay.json"],
            samples=5,
        ),
        "native_combat_render_ms": measure_command(
            [BIN, "native-combat-render", "--scenario", SCENARIO, "--out", render_dir],
            samples=3,
        ),
    }

    trace = json.loads((duel_dir / "trace.json").read_text(encoding="utf-8"))
    render_manifest = json.loads((render_dir / "native_combat_render_manifest.json").read_text(encoding="utf-8"))
    manifest = json.loads((ROOT / "assets" / "runtime_manifest.json").read_text(encoding="utf-8"))
    observed_frames = observed_truth_frames(trace)
    playback_frame_count = int(render_manifest["playback_loop"]["playback_frame_count"])
    live_loop_frame_count = int(render_manifest["live_render_loop"]["rendered_frame_count"])
    software_3d_viewports = render_manifest.get("software_3d_viewports", [])
    software_3d_sequence = render_manifest.get("software_3d_sequence", {})
    software_3d_sequence_frames = software_3d_sequence.get("frames", [])
    captured_frame_count = (
        int(render_manifest["state_frame_count"])
        + int(render_manifest["motion_frame_count"])
        + int(render_manifest["live_render_loop"]["sample_capture_count"])
        + len(software_3d_viewports)
        + int(software_3d_sequence.get("frame_count", 0))
        + len(render_manifest["resolution_captures"])
        + 2
    )
    summary = {
        "schema": "oathyard.performance.v1",
        "product": "OATHYARD",
        "public_demo_ready": False,
        "release_candidate_ready": False,
        "truth_hz": trace["truth_hz"],
        "scenario": trace["scenario_id"],
        "content_hash": trace["content_hash"],
        "initial_state_hash": trace["initial_state_hash"],
        "final_state_hash": trace["final_state_hash"],
        "turn_count": len(trace["turns"]),
        "observed_truth_frames_from_trace": observed_frames,
        "native_render": {
            "state_frame_count": int(render_manifest["state_frame_count"]),
            "motion_frame_count": int(render_manifest["motion_frame_count"]),
            "playback_frame_count": playback_frame_count,
            "playback_cycles": int(render_manifest["playback_loop"]["cycles"]),
            "nominal_playback_duration_ms": int(render_manifest["playback_loop"]["nominal_duration_ms"]),
            "live_loop_frame_count": live_loop_frame_count,
            "live_loop_sample_capture_count": int(render_manifest["live_render_loop"]["sample_capture_count"]),
            "nominal_live_loop_duration_ms": int(render_manifest["live_render_loop"]["nominal_duration_ms"]),
            "live_loop_hash": render_manifest["live_render_loop"]["loop_hash"],
            "software_3d_viewport_count": len(software_3d_viewports),
            "software_3d_shaded_triangle_count": sum(
                int(viewport.get("shaded_triangle_count", 0))
                for viewport in software_3d_viewports
            ),
            "software_3d_projection_model": "integer_depth_sorted_mesh_raster"
            if software_3d_viewports
            else "",
            "software_3d_sequence_frame_count": int(software_3d_sequence.get("frame_count", 0)),
            "software_3d_sequence_camera": software_3d_sequence.get("camera", ""),
            "software_3d_sequence_source": software_3d_sequence.get("source", ""),
            "software_3d_sequence_hash_chain": software_3d_sequence.get("frame_hash_chain", ""),
            "software_3d_sequence_shaded_triangle_count": sum(
                int(frame.get("shaded_triangle_count", 0))
                for frame in software_3d_sequence_frames
            ),
            "captured_frame_artifacts": captured_frame_count,
            "playback_final_hash": render_manifest["playback_loop"]["final_frame_hash"],
            "live_loop_final_hash": render_manifest["live_render_loop"]["final_frame_hash"],
        },
        "timings": timings,
        "derived": {
            "duel_run_min_ms_per_observed_trace_frame": divide_ms(
                timings["duel_run_ms"]["min"], observed_frames
            ),
            "native_render_min_ms_per_captured_frame_artifact": divide_ms(
                timings["native_combat_render_ms"]["min"], captured_frame_count
            ),
            "native_render_min_ms_per_playback_loop_frame": divide_ms(
                timings["native_combat_render_ms"]["min"], playback_frame_count
            ),
            "native_render_min_ms_per_live_loop_frame": divide_ms(
                timings["native_combat_render_ms"]["min"], live_loop_frame_count
            ),
        },
        "asset_budget": asset_budget(manifest),
        "package_budget": {
            "debug_executable_bytes": file_size(BIN),
            "package_tar_bytes": file_size(PACKAGE),
        },
        "notes": [
            "Measured by tools/performance_benchmark.py outside authoritative truth.",
            "Wall-clock timing is QA evidence only and never enters replay hashes.",
            "Native combat render timing measures the X11/XWayland overview, state frames, motion frames, 120-frame live loop, playback loop, resolution captures, and reports.",
            "Software 3D viewport captures are presentation-only filled mesh raster artifacts generated from runtime glTF after truth hashes.",
            "Software 3D replay sequence frames are presentation-only mesh motion artifacts generated from replay-derived motion frames after truth hashes.",
        ],
    }
    (out_dir / "performance_summary.json").write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (out_dir / "performance_summary.md").write_text(render_markdown(summary), encoding="utf-8")
    print("performance benchmark passed")
    print(f"out={out_dir.relative_to(ROOT)}")
    return 0


def ensure_binary():
    if BIN.exists():
        return
    subprocess.run(["cargo", "build", "--locked"], cwd=ROOT, check=True)


def measure_command(command, samples):
    durations = []
    printable = [str(part.relative_to(ROOT)) if isinstance(part, Path) and part.is_absolute() else str(part) for part in command]
    for _ in range(samples):
        start = time.perf_counter_ns()
        subprocess.run([str(part) for part in command], cwd=ROOT, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        end = time.perf_counter_ns()
        durations.append((end - start) / 1_000_000.0)
    return {
        "command": printable,
        "samples": samples,
        "min": round(min(durations), 3),
        "max": round(max(durations), 3),
        "mean": round(statistics.fmean(durations), 3),
    }


def observed_truth_frames(trace):
    max_frame = 0
    for turn in trace["turns"]:
        for contact in turn.get("contacts", []):
            max_frame = max(max_frame, int(contact["frame"]))
        for cost in turn.get("costs", []):
            max_frame = max(max_frame, int(cost["current_cost_frames"]))
    return max(max_frame, len(trace["turns"]))


def asset_budget(manifest):
    entries = manifest["entries"]
    return {
        "asset_hash": manifest["asset_hash"],
        "entries": len(entries),
        "runtime_mesh_bytes": tree_size(ROOT / "assets" / "runtime"),
        "runtime_gltf_bytes": tree_size(ROOT / "assets" / "gltf"),
        "preview_bytes": tree_size(ROOT / "assets" / "previews"),
        "source_asset_bytes": tree_size(ROOT / "assets_src"),
        "runtime_manifest_bytes": file_size(ROOT / "assets" / "runtime_manifest.json"),
    }


def tree_size(path):
    total = 0
    for child in path.rglob("*"):
        if child.is_file():
            total += child.stat().st_size
    return total


def file_size(path):
    return path.stat().st_size if path.exists() else 0


def divide_ms(ms, frames):
    if frames <= 0:
        return 0.0
    return round(ms / frames, 6)


def render_markdown(summary):
    timings = summary["timings"]
    budget = summary["asset_budget"]
    package = summary["package_budget"]
    native_render = summary["native_render"]
    lines = [
        "# OATHYARD Performance Summary",
        "",
        "Status: PASSED",
        "",
        f"- Scenario: `{summary['scenario']}`",
        f"- Truth step rate: `{summary['truth_hz']} Hz`",
        f"- Content hash: `{summary['content_hash']}`",
        f"- Final state hash: `{summary['final_state_hash']}`",
        f"- Turn count: `{summary['turn_count']}`",
        f"- Observed trace frame span: `{summary['observed_truth_frames_from_trace']}`",
        f"- Native render state frames: `{native_render['state_frame_count']}`",
        f"- Native render motion frames: `{native_render['motion_frame_count']}`",
        f"- Native playback loop frames: `{native_render['playback_frame_count']}`",
        f"- Native playback nominal duration: `{native_render['nominal_playback_duration_ms']} ms`",
        f"- Native live render loop frames: `{native_render['live_loop_frame_count']}`",
        f"- Native live render loop sample captures: `{native_render['live_loop_sample_capture_count']}`",
        f"- Native live render loop nominal duration: `{native_render['nominal_live_loop_duration_ms']} ms`",
        f"- Native software 3D viewport captures: `{native_render['software_3d_viewport_count']}`",
        f"- Native software 3D shaded triangles: `{native_render['software_3d_shaded_triangle_count']}`",
        f"- Native software 3D projection model: `{native_render['software_3d_projection_model']}`",
        f"- Native software 3D replay sequence frames: `{native_render['software_3d_sequence_frame_count']}`",
        f"- Native software 3D replay sequence camera: `{native_render['software_3d_sequence_camera']}`",
        f"- Native software 3D replay sequence source: `{native_render['software_3d_sequence_source']}`",
        f"- Native software 3D replay sequence shaded triangles: `{native_render['software_3d_sequence_shaded_triangle_count']}`",
        "",
        "## Timings",
        "",
    ]
    for name, data in timings.items():
        lines.append(
            f"- `{name}`: min `{data['min']} ms`, mean `{data['mean']} ms`, max `{data['max']} ms`, samples `{data['samples']}`"
        )
    lines.extend(
        [
            f"- Duel run min per observed trace frame: `{summary['derived']['duel_run_min_ms_per_observed_trace_frame']} ms`",
            f"- Native render min per captured frame artifact: `{summary['derived']['native_render_min_ms_per_captured_frame_artifact']} ms`",
            f"- Native render min per playback loop frame: `{summary['derived']['native_render_min_ms_per_playback_loop_frame']} ms`",
            f"- Native render min per live-loop frame: `{summary['derived']['native_render_min_ms_per_live_loop_frame']} ms`",
            "",
            "## Budgets",
            "",
            f"- Asset hash: `{budget['asset_hash']}`",
            f"- Runtime entries: `{budget['entries']}`",
            f"- Runtime mesh bytes: `{budget['runtime_mesh_bytes']}`",
            f"- Runtime glTF bytes: `{budget['runtime_gltf_bytes']}`",
            f"- Preview bytes: `{budget['preview_bytes']}`",
            f"- Source asset bytes: `{budget['source_asset_bytes']}`",
            f"- Runtime manifest bytes: `{budget['runtime_manifest_bytes']}`",
            f"- Debug executable bytes: `{package['debug_executable_bytes']}`",
            f"- Package tar bytes: `{package['package_tar_bytes']}`",
            "",
            "## Constraints",
            "",
            "- Timing is measured outside authoritative truth.",
            "- Wall-clock data is not replay input and is not hashed into gameplay state.",
            "- Public demo ready: `false`",
            "- Release candidate ready: `false`",
        ]
    )
    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    raise SystemExit(main())
