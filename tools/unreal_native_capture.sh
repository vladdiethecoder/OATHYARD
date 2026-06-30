#!/usr/bin/env bash
set -euo pipefail

# OATHYARD Unreal native capture automation.
# Capture is presentation-only: it must consume already-rendered UE frames and never feed
# renderer/compositor state back into authoritative gameplay truth.

width=1920
height=1080
frame_count=1
mode="external-x11"
out="artifacts/unreal_capture/latest"
window_title="Unreal|OATHYARD"
display="${DISPLAY:-:0}"
capture_x=""
capture_y=""
editor="${UNREAL_EDITOR:-/home/vdubrov/UnrealEngine/Engine/Binaries/Linux/UnrealEditor}"
project="${UNREAL_PROJECT:-}"
map_name="${UNREAL_MAP:-}"
plan_only=0

usage() {
  cat <<'USAGE'
Usage:
  ./tools/unreal_native_capture.sh [options]

Purpose:
  Capture lossless 1920x1080 PNG frame(s) from a running Unreal Engine instance
  on Linux and emit deterministic metadata for OATHYARD truth-packet bridge
  ingestion. The capture path is presentation-only and records truth_mutation=false.

Primary mode for a running UE window:
  ./tools/unreal_native_capture.sh \
    --mode external-x11 \
    --window-title 'Unreal|OATHYARD' \
    --out artifacts/unreal_capture/latest

Alternative mode after a launchable UnrealEditor/project exists:
  ./tools/unreal_native_capture.sh \
    --mode unreal-highres \
    --editor /home/vdubrov/UnrealEngine/Engine/Binaries/Linux/UnrealEditor \
    --project /path/to/OATHYARD.uproject \
    --map /Game/Maps/TestScene \
    --out artifacts/unreal_capture/latest

Options:
  --mode external-x11|unreal-highres
      external-x11: use xdotool to locate an X11/XWayland UE window, then ffmpeg
      x11grab to write PNG frame(s). This requires a visible X11/XWayland surface.
      unreal-highres: launch UnrealEditor with -ExecCmds="HighResShot ...;Quit".
  --out DIR
      Output directory. Default: artifacts/unreal_capture/latest
  --width N --height N
      Capture resolution. Defaults: 1920x1080
  --frames N
      Number of PNG frames for external-x11 mode. Default: 1
  --window-title REGEX
      xdotool window-title search regex for external-x11. Default: Unreal|OATHYARD
  --display DISPLAY
      X display used by external-x11. Default: current DISPLAY or :0
  --x N --y N
      Manual X11 capture origin. If omitted, derived from the matched window.
  --geometry X,Y
      Manual X11 capture origin shorthand.
  --editor PATH --project PATH --map URL_OR_MAP
      Unreal high-resolution screenshot mode inputs.
  --plan-only | --dry-run
      Write manifest/report describing the exact capture plan and prerequisites,
      but do not capture and do not launch Unreal.
  --help
      Show this help.

Outputs:
  DIR/unreal_native_capture_manifest.json
  DIR/unreal_native_capture_report.md
  DIR/unreal_capture_1920x1080_000001.png     (external-x11 success)
  DIR/unreal_highres_1920x1080.png            (unreal-highres success)

Current known gate:
  Do not run a real UE capture until UE launch capability is confirmed. If the
  default /home/vdubrov/UnrealEngine/Engine/Binaries/Linux/UnrealEditor is still
  missing, unreal-highres mode fails before launching anything.
USAGE
}

fail() {
  echo "unreal_native_capture: $*" >&2
  exit 1
}

need_arg() {
  local opt="$1"
  local value="${2:-}"
  [[ -n "$value" ]] || fail "$opt requires an argument"
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --mode)
      need_arg "$1" "${2:-}"; mode="$2"; shift 2 ;;
    --out)
      need_arg "$1" "${2:-}"; out="$2"; shift 2 ;;
    --width)
      need_arg "$1" "${2:-}"; width="$2"; shift 2 ;;
    --height)
      need_arg "$1" "${2:-}"; height="$2"; shift 2 ;;
    --frames|--frame-count)
      need_arg "$1" "${2:-}"; frame_count="$2"; shift 2 ;;
    --window-title)
      need_arg "$1" "${2:-}"; window_title="$2"; shift 2 ;;
    --display)
      need_arg "$1" "${2:-}"; display="$2"; shift 2 ;;
    --x)
      need_arg "$1" "${2:-}"; capture_x="$2"; shift 2 ;;
    --y)
      need_arg "$1" "${2:-}"; capture_y="$2"; shift 2 ;;
    --geometry)
      need_arg "$1" "${2:-}"
      IFS=',' read -r capture_x capture_y <<<"$2"
      [[ -n "$capture_x" && -n "$capture_y" ]] || fail "--geometry must be X,Y"
      shift 2 ;;
    --editor)
      need_arg "$1" "${2:-}"; editor="$2"; shift 2 ;;
    --project)
      need_arg "$1" "${2:-}"; project="$2"; shift 2 ;;
    --map)
      need_arg "$1" "${2:-}"; map_name="$2"; shift 2 ;;
    --plan-only|--dry-run)
      plan_only=1; shift ;;
    --help|-h)
      usage; exit 0 ;;
    *)
      fail "unknown argument: $1" ;;
  esac
done

[[ "$mode" == "external-x11" || "$mode" == "unreal-highres" ]] || fail "unsupported mode: $mode"
[[ "$width" =~ ^[0-9]+$ && "$height" =~ ^[0-9]+$ ]] || fail "width/height must be positive integers"
[[ "$frame_count" =~ ^[0-9]+$ && "$frame_count" -ge 1 ]] || fail "--frames must be a positive integer"

mkdir -p "$out"

write_manifest() {
  local status="$1"
  local reason="${2:-}"
  OY_UNREAL_CAPTURE_OUT="$out" \
  OY_UNREAL_CAPTURE_STATUS="$status" \
  OY_UNREAL_CAPTURE_REASON="$reason" \
  OY_UNREAL_CAPTURE_MODE="$mode" \
  OY_UNREAL_CAPTURE_WIDTH="$width" \
  OY_UNREAL_CAPTURE_HEIGHT="$height" \
  OY_UNREAL_CAPTURE_FRAME_COUNT="$frame_count" \
  OY_UNREAL_CAPTURE_WINDOW_TITLE="$window_title" \
  OY_UNREAL_CAPTURE_DISPLAY="$display" \
  OY_UNREAL_CAPTURE_X="$capture_x" \
  OY_UNREAL_CAPTURE_Y="$capture_y" \
  OY_UNREAL_CAPTURE_EDITOR="$editor" \
  OY_UNREAL_CAPTURE_PROJECT="$project" \
  OY_UNREAL_CAPTURE_MAP="$map_name" \
  OY_UNREAL_CAPTURE_PLAN_ONLY="$plan_only" \
  python3 - <<'PY'
import hashlib
import json
import os
import struct
from pathlib import Path

out = Path(os.environ["OY_UNREAL_CAPTURE_OUT"])
status = os.environ["OY_UNREAL_CAPTURE_STATUS"]
reason = os.environ["OY_UNREAL_CAPTURE_REASON"]
mode = os.environ["OY_UNREAL_CAPTURE_MODE"]
width = int(os.environ["OY_UNREAL_CAPTURE_WIDTH"])
height = int(os.environ["OY_UNREAL_CAPTURE_HEIGHT"])
frame_count = int(os.environ["OY_UNREAL_CAPTURE_FRAME_COUNT"])
window_title = os.environ["OY_UNREAL_CAPTURE_WINDOW_TITLE"]
display = os.environ["OY_UNREAL_CAPTURE_DISPLAY"]
capture_x = os.environ["OY_UNREAL_CAPTURE_X"]
capture_y = os.environ["OY_UNREAL_CAPTURE_Y"]
editor = os.environ["OY_UNREAL_CAPTURE_EDITOR"]
project = os.environ["OY_UNREAL_CAPTURE_PROJECT"]
map_name = os.environ["OY_UNREAL_CAPTURE_MAP"]
plan_only = os.environ["OY_UNREAL_CAPTURE_PLAN_ONLY"] == "1"


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def png_dimensions(path: Path):
    with path.open("rb") as f:
        header = f.read(24)
    if not header.startswith(b"\x89PNG\r\n\x1a\n"):
        raise ValueError(f"not a PNG file: {path.name}")
    return struct.unpack(">II", header[16:24])

frames = []
if mode == "external-x11":
    candidates = sorted(out.glob(f"unreal_capture_{width}x{height}_*.png"))
else:
    candidate = out / f"unreal_highres_{width}x{height}.png"
    candidates = [candidate] if candidate.is_file() else []

for frame in candidates:
    w, h = png_dimensions(frame)
    if (w, h) != (width, height):
        raise SystemExit(f"capture dimension mismatch for {frame}: got {w}x{h}, expected {width}x{height}")
    frames.append({
        "file": frame.name,
        "format": "png",
        "lossless": True,
        "width": w,
        "height": h,
        "size_bytes": frame.stat().st_size,
        "sha256": sha256(frame),
    })

chain = ""
for frame in frames:
    chain = hashlib.sha256((chain + frame["sha256"]).encode("ascii")).hexdigest()

external_template = (
    f"xdotool search --name {window_title!r}; "
    f"xdotool getwindowgeometry --shell <WINDOW_ID>; "
    f"ffmpeg -hide_banner -loglevel error -y -video_size {width}x{height} "
    f"-framerate 1 -f x11grab -i {display}+<X>,<Y> -frames:v {frame_count} "
    f"{out.as_posix()}/unreal_capture_{width}x{height}_%06d.png"
)
highres_template = (
    f"{editor} <PROJECT> <MAP> -game -windowed -ResX={width} -ResY={height} "
    f"-ExecCmds=\"HighResShot filename={out.as_posix()}/unreal_highres_{width}x{height}.png {width}x{height};Quit\""
)

blockers = []
if mode == "unreal-highres":
    if not Path(editor).is_file():
        blockers.append(f"missing launchable UnrealEditor binary: {editor}")
    if not project:
        blockers.append("missing --project / UNREAL_PROJECT for unreal-highres mode")
    elif not Path(project).is_file():
        blockers.append(f"missing Unreal project file: {project}")
if mode == "external-x11":
    if not capture_x or not capture_y:
        blockers.append("real capture requires a matching X11/XWayland window or manual --x/--y origin")

manifest = {
    "schema": "oathyard.unreal_native_capture.v1",
    "tool": "tools/unreal_native_capture.sh",
    "task_id": "t_2717c1c3",
    "status": status,
    "status_reason": reason,
    "mode": mode,
    "capture_attempted": bool(frames) or (status == "capture_attempted"),
    "plan_only": plan_only,
    "width": width,
    "height": height,
    "requested_frame_count": frame_count,
    "captured_frame_count": len(frames),
    "frames": frames,
    "frame_hash_chain": chain,
    "output_directory": out.as_posix(),
    "trigger_mechanism": {
        "external_x11": "CLI invocation after UE has a visible X11/XWayland window; xdotool locates the window and ffmpeg x11grab captures the fixed region.",
        "unreal_highres": "Unreal command-line -ExecCmds invokes the HighResShot console command and quits after writing the requested PNG.",
    }[mode],
    "command_templates": {
        "external_x11": external_template,
        "unreal_highres": highres_template,
    },
    "prerequisites": {
        "ffmpeg_x11grab": "required for external-x11",
        "xdotool": "required for external-x11 window discovery",
        "unreal_editor": "required for unreal-highres",
        "ue_project": "required for unreal-highres",
    },
    "local_inputs": {
        "window_title_regex": window_title,
        "display": display,
        "capture_x": capture_x,
        "capture_y": capture_y,
        "editor_path": editor,
        "project_path": project,
        "map": map_name,
    },
    "presentation_only": True,
    "truth_mutation": False,
    "authoritative_truth_input": False,
    "suitable_for_hash_chain_ingestion": len(frames) > 0 and not blockers,
    "readiness_claims": {
        "production_renderer_complete": False,
        "owner_visual_acceptance": False,
        "public_demo_ready": False,
        "release_candidate_ready": False,
    },
    "sources": [
        "Epic Unreal Engine 5.8 Taking Screenshots documentation: HighResShot supports filename=PATH and XxY size parameters.",
        "Epic Unreal Engine 5.8 Command-Line Arguments documentation: -windowed, -ResX, -ResY key-value arguments configure launch resolution.",
        "Epic Unreal Engine 5.8 Run Automation Tests documentation: -ExecCmds=\"...;Quit\" command-line execution pattern.",
        "FFmpeg Capture/Desktop wiki: Linux x11grab uses -video_size WIDTHxHEIGHT -f x11grab -i DISPLAY+X,Y.",
    ],
    "blockers": blockers,
}

(out / "unreal_native_capture_manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")

lines = [
    "# OATHYARD Unreal Native Capture Automation",
    "",
    f"Status: {status}",
    "",
    "## Capture command",
    "",
    "External running-window capture:",
    "",
    f"`{external_template}`",
    "",
    "Built-in Unreal HighResShot capture after UE launch is confirmed:",
    "",
    f"`{highres_template}`",
    "",
    "## Trigger mechanism",
    "",
    f"- Active mode: `{mode}`",
    f"- Trigger: {manifest['trigger_mechanism']}",
    "",
    "## Output directory",
    "",
    f"- `{out.as_posix()}`",
    "- Manifest: `unreal_native_capture_manifest.json`",
    "- Report: `unreal_native_capture_report.md`",
    "",
    "## Frame evidence",
    "",
    f"- Requested resolution: `{width}x{height}`",
    f"- Captured frames: `{len(frames)}`",
    f"- Frame hash chain: `{chain or 'not available until capture succeeds'}`",
    "- Presentation only: `true`",
    "- Truth mutation: `false`",
    "",
    "## Blockers",
]
if blockers:
    lines.extend(f"- {b}" for b in blockers)
else:
    lines.append("- none recorded by this script")
if reason:
    lines.extend(["", "## Status reason", "", f"- {reason}"])
lines.extend([
    "",
    "## Scope boundary",
    "",
    "This automation records renderer/compositor pixels only. It does not prove production renderer completion, high-fidelity product quality, owner visual acceptance, public-demo readiness, release-candidate readiness, legal clearance, or any authoritative combat truth behavior.",
])
(out / "unreal_native_capture_report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
PY
}

if [[ "$plan_only" -eq 1 ]]; then
  write_manifest "planned" "plan-only mode: no UE launch and no compositor capture attempted"
  echo "unreal native capture plan: $out"
  exit 0
fi

normalize_display() {
  local d="$1"
  if [[ "$d" =~ ^:[0-9]+$ ]]; then
    printf '%s.0' "$d"
  else
    printf '%s' "$d"
  fi
}

capture_external_x11() {
  require_cmd ffmpeg
  require_cmd xdotool

  local x="$capture_x"
  local y="$capture_y"
  local window_id="manual"
  local window_w="unknown"
  local window_h="unknown"
  local norm_display
  norm_display="$(normalize_display "$display")"

  if [[ -z "$x" || -z "$y" ]]; then
    local ids
    ids="$(DISPLAY="$display" xdotool search --name "$window_title" 2>/dev/null || true)"
    [[ -n "$ids" ]] || fail "no X11/XWayland window matched title regex: $window_title"
    window_id="$(printf '%s\n' "$ids" | tail -n 1)"
    local geom
    geom="$(DISPLAY="$display" xdotool getwindowgeometry --shell "$window_id")"
    eval "$geom"
    x="${X:?}"
    y="${Y:?}"
    window_w="${WIDTH:?}"
    window_h="${HEIGHT:?}"
    [[ "$window_w" -ge "$width" && "$window_h" -ge "$height" ]] || fail "matched window $window_id is ${window_w}x${window_h}, smaller than required ${width}x${height}"
  fi

  capture_x="$x"
  capture_y="$y"
  export OY_UNREAL_CAPTURE_X="$capture_x"
  export OY_UNREAL_CAPTURE_Y="$capture_y"

  local pattern="$out/unreal_capture_${width}x${height}_%06d.png"
  ffmpeg -hide_banner -loglevel error -y \
    -video_size "${width}x${height}" \
    -framerate 1 \
    -f x11grab \
    -i "${norm_display}+${x},${y}" \
    -frames:v "$frame_count" \
    "$pattern"

  write_manifest "captured" "external-x11 window_id=$window_id window_size=${window_w}x${window_h} origin=${x},${y}"
  echo "unreal native x11 capture: $out"
}

capture_unreal_highres() {
  [[ -x "$editor" ]] || fail "UnrealEditor missing or not executable: $editor"
  [[ -n "$project" ]] || fail "--project is required for unreal-highres mode"
  [[ -f "$project" ]] || fail "Unreal project file missing: $project"

  local frame="$out/unreal_highres_${width}x${height}.png"
  local exec_cmd="HighResShot filename=$frame ${width}x${height};Quit"
  local cmd=("$editor" "$project")
  if [[ -n "$map_name" ]]; then
    cmd+=("$map_name")
  fi
  cmd+=("-game" "-windowed" "-ResX=$width" "-ResY=$height" "-unattended" "-nop4" "-nosplash" "-ExecCmds=$exec_cmd")
  "${cmd[@]}"

  [[ -s "$frame" ]] || fail "Unreal HighResShot did not create expected frame: $frame"
  write_manifest "captured" "unreal-highres HighResShot completed"
  echo "unreal native highres capture: $out"
}

case "$mode" in
  external-x11)
    capture_external_x11 ;;
  unreal-highres)
    capture_unreal_highres ;;
esac
