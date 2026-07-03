#!/usr/bin/env bash
set -euo pipefail

# Unit-072: Native window/swapchain playable smoke test
# Opens a real OS window, creates a wgpu surface/swapchain,
# presents rigged saltreach_duelist/training_yard mesh frames,
# and records windowed runtime evidence.

DUEL="${1:?usage: $0 <duel_path> [out_dir]}"
OUT="${2:-artifacts/native_windowed/latest}"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RENDERER_BIN="$REPO_ROOT/crates/oathyard_renderer/target/debug/oathyard-native-renderer"

# Build the renderer if needed
if [[ ! -x "$RENDERER_BIN" ]]; then
  cargo build --manifest-path "$REPO_ROOT/crates/oathyard_renderer/Cargo.toml"
fi

mkdir -p "$OUT"

# Step 1: Generate the post-hash presentation packet (reuse native_combat_render pipeline)
PACKET_DIR="$OUT/packet"
mkdir -p "$PACKET_DIR"
echo "=== Generating post-hash presentation packet ==="
if [[ -x "$REPO_ROOT/tools/generate_presentation_packet.sh" ]]; then
  "$REPO_ROOT/tools/generate_presentation_packet.sh" "$DUEL" "$PACKET_DIR"
else
  # Directly via cargo run
  cargo run --locked -- --scenario "$DUEL" --packet-out "$PACKET_DIR/post_hash_presentation_packet.json" 2>/dev/null || true
fi

PACKET="$OUT/post_hash_presentation_packet.json"
if [[ ! -f "$PACKET" ]]; then
  # Use the native_combat_render to produce the packet
  cargo run --locked -- native-combat-render --scenario "$DUEL" --out "$OUT" 2>/dev/null || true
  PACKET="$OUT/post_hash_presentation_packet.json"
fi

if [[ ! -f "$PACKET" ]]; then
  echo "ERROR: could not generate presentation packet. Run native_combat_render first." >&2
  exit 1
fi

# Step 2: Generate mesh manifest for rigged assets
MESH_MANIFEST="$OUT/mesh_manifests/windowed_mesh_manifest.json"
mkdir -p "$(dirname "$MESH_MANIFEST")"
if [[ -f "$REPO_ROOT/assets/runtime/saltreach_duelist_skinned.mesh.json" ]]; then
  python3 - "$MESH_MANIFEST" "$REPO_ROOT" <<'PY'
import json, sys
from pathlib import Path
manifest = Path(sys.argv[1])
root = Path(sys.argv[2])
skinned = root / "assets/runtime/saltreach_duelist_skinned.mesh.json"
training = root / "assets/presentation_runtime/training_yard.mesh.json"
meshes = [
    {"mesh_asset_id":"player_saltreach_duelist","mesh_asset_class":"fighter","mesh_source":str(skinned),
     "translation":[-0.72,0.0,0.0],"scale":0.72,"yaw_radians":0.10,
     "transform_baked_or_runtime":"runtime_transform_baked_into_candidate_vertex_buffer",
     "candidate_status":"source_approved_production_seed","production_ready":False,"truth_mutation":False},
    {"mesh_asset_id":"opponent_saltreach_duelist","mesh_asset_class":"fighter","mesh_source":str(skinned),
     "translation":[0.72,0.0,0.0],"scale":0.72,"yaw_radians":0.10,
     "transform_baked_or_runtime":"runtime_transform_baked_into_candidate_vertex_buffer",
     "candidate_status":"source_approved_production_seed","production_ready":False,"truth_mutation":False},
    {"mesh_asset_id":"training_yard","mesh_asset_class":"arena","mesh_source":str(training),
     "translation":[0.0,-0.18,0.0],"scale":0.82,"yaw_radians":0.0,
     "transform_baked_or_runtime":"runtime_transform_baked_into_candidate_vertex_buffer",
     "candidate_status":"source_approved_production_seed","production_ready":False,"truth_mutation":False},
]
payload = {"schema":"oathyard.wgpu_runtime_mesh_manifest.v1",
 "source":"run_native_windowed_game.sh Unit-072",
 "capture_id":"windowed_smoke","candidate_renderer_only":False,
 "production_seed_render":True,"production_ready":False,"truth_mutation":False,
 "meshes":meshes}
manifest.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
fi

# Step 3: Run the windowed smoke test
WINDOWED_OUT="$OUT/windowed_smoke"
mkdir -p "$WINDOWED_OUT"

echo ""
echo "=== Launching native windowed renderer (smoke mode) ==="
echo "  This will open a real OS window, render the rigged duel scene to a"
echo "  wgpu surface/swapchain, present frames, and auto-exit after smoke frames."

SMOKE_FRAMES="${SMOKE_FRAMES:-60}"

# Detect if we have display access
DISPLAY_AVAILABLE=false
if [[ -n "${DISPLAY:-}" ]] || [[ -n "${WAYLAND_DISPLAY:-}" ]] || [[ -n "${XDG_SESSION_TYPE:-}" ]]; then
  DISPLAY_AVAILABLE=true
fi

if [[ "$DISPLAY_AVAILABLE" == "true" ]]; then
  "$RENDERER_BIN" \
    --windowed \
    --packet "$PACKET" \
    --out "$WINDOWED_OUT" \
    --mesh-manifest-json "$MESH_MANIFEST" \
    --candidate-assets "saltreach_duelist,training_yard" \
    --smoke-frames "$SMOKE_FRAMES" \
    --auto-exit \
    >"$WINDOWED_OUT/windowed_output.log" 2>&1
  WINDOWED_RC=$?
  echo "  windowed smoke exit: $WINDOWED_RC"
else
  echo "  WARNING: no display detected — windowed smoke skipped"
  echo "  native_windowed_execution: false (no display available)"
  # Write a blocker manifest
  python3 - "$WINDOWED_OUT" <<'PY'
import json, sys
from pathlib import Path
out = Path(sys.argv[1])
manifest = {
    "schema": "oathyard.native_window_runtime.v1",
    "product": "OATHYARD", "unit": "Unit-072",
    "native_windowed_execution": False,
    "windowed_blocker": "no display available (DISPLAY/WAYLAND_DISPLAY not set)",
    "smoke_mode": True, "frames_presented": 0,
    "owner_visual_acceptance": False, "public_demo_ready": False,
    "release_candidate_ready": False, "truth_mutation": False,
}
(out / "native_window_runtime_manifest.json").write_text(json.dumps(manifest, indent=2))
PY
  WINDOWED_RC=0
fi

# Step 4: Check results
echo ""
echo "=== Windowed smoke results ==="
if [[ -f "$WINDOWED_OUT/native_window_runtime_manifest.json" ]]; then
  python3 -c "
import json
d = json.load(open('$WINDOWED_OUT/native_window_runtime_manifest.json'))
print(f'  native_windowed_execution: {d.get(\"native_windowed_execution\")}')
print(f'  frames_presented: {d.get(\"frames_presented\")}')
print(f'  mesh_geometry_consumed: {d.get(\"mesh_geometry_consumed\")}')
print(f'  mesh_asset_count: {d.get(\"mesh_asset_count\")}')
print(f'  saltreach_consumed: {d.get(\"saltreach_duelist_consumed\")}')
print(f'  truth_mutation: {d.get(\"truth_mutation\")}')
"
fi

echo ""
echo "Windowed smoke output: $WINDOWED_OUT"
exit $WINDOWED_RC
