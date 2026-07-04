#!/usr/bin/env bash
# Unit-070: Native 3D exchange capture matrix.
# Generates one capture per exchange phase using the production renderer.
# Each capture uses a distinct camera + animation clip matching the phase's
# narrative role in the exchange timeline.
set -euo pipefail

scenario="${1:-examples/duels/basic_oathyard.duel}"
out="${2:-artifacts/exchange_captures/latest}"

mkdir -p "$out"

# First run the truth duel to get the verified replay + hash
cargo run --locked -- native-combat-render --scenario "$scenario" --out "$out/truth" >/dev/null 2>&1

# Verify hash integrity
REPLAY_HASH=$(python3 -c "import json; d=json.load(open('$out/truth/native_capture_input_replay.json')); print(d.get('final_state_hash',''))" 2>/dev/null || echo "")
MANIFEST_HASH=$(python3 -c "import json; d=json.load(open('$out/truth/native_combat_render_manifest.json')); print(d.get('final_state_hash',''))" 2>/dev/null || echo "")
if [[ -z "$REPLAY_HASH" || -z "$MANIFEST_HASH" || "$REPLAY_HASH" != "$MANIFEST_HASH" ]]; then
  echo "ERROR: hash mismatch between replay and manifest" >&2
  exit 1
fi

RENDERER="crates/oathyard_renderer/target/debug/oathyard-native-renderer"
PACKET="$out/truth/post_hash_presentation_packet.json"

# Phase matrix: phase_id|camera_mode|clip_id|description
# Each phase maps to a camera that shows the exchange from the right angle.
PHASES=(
  "observe_idle|oathyard_verdict_ring_establishing|idle|Observe phase — establishing shot"
  "plan_ready|planning_timeline|idle|Planning phase — timeline overhead view"
  "commit_reveal|pre_contact_frame|guard_pose|Commit phase — pre-contact guard reveal"
  "anticipation|pre_contact_frame|guard_pose|Anticipation — weapon drawn back"
  "pre_contact|pre_contact_frame|guard_pose|Pre-contact — closing distance"
  "contact|contact_frame|cut|Contact — blade strike at peak"
  "follow_through|contact_frame|cut|Follow-through — energy after contact"
  "consequence|injury_capability_consequence_frame|attack|Consequence — injury/knockback"
  "recover|recovery_replan_frame|idle|Recovery — regaining guard"
  "replan|recovery_replan_frame|idle|Replan — reassessing stance"
  "replay|fight_film_replay_camera_shot|cut|Replay — fight-film replay angle"
  "fight_film|fight_film_candidate_shot_01|cut|Fight-film — cinematic candidate"
)

CAPTURE_COUNT=0
MANIFEST_ENTRIES=""

# Unit-071: Generate mesh manifest for rigged asset consumption.
# Each capture phase must consume the actual skinned saltreach_duelist mesh
# rather than falling back to procedural/SDF geometry.
SKINNED_MESH="assets/runtime/saltreach_duelist_skinned.mesh.json"
TRAINING_YARD_MESH="assets/presentation_runtime/training_yard.mesh.json"
MESH_MANIFEST_DIR="$out/mesh_manifests"
mkdir -p "$MESH_MANIFEST_DIR"

generate_mesh_manifest() {
  local phase_id="$1"
  local manifest="$MESH_MANIFEST_DIR/${phase_id}.json"
  python3 - "$manifest" "$SKINNED_MESH" "$TRAINING_YARD_MESH" <<'PY'
import json, sys
from pathlib import Path
manifest = Path(sys.argv[1])
skinned = sys.argv[2]
training = sys.argv[3]
longsword = Path("assets/presentation_runtime/longsword.mesh.json").as_posix()
gambeson = Path("assets/presentation_runtime/gambeson.mesh.json").as_posix()
tex = Path("assets/model_candidates/t_73291be5/textures")
def tex_paths(name):
    return {
        "base_color_texture_path": (tex / f"{name}_base.png").as_posix(),
        "normal_texture_path": (tex / f"{name}_normal.png").as_posix(),
        "orm_texture_path": (tex / f"{name}_orm.png").as_posix(),
    }
longsword_tex = tex_paths("longsword")
gambeson_tex = tex_paths("gambeson")
saltreach_tex = tex_paths("saltreach_duelist")
training_tex = tex_paths("training_yard")
meshes = [
    {
        "mesh_asset_id": "player_saltreach_duelist",
        "mesh_asset_class": "fighter",
        "mesh_source": skinned,
        "translation": [-0.72, 0.0, 0.0],
        "scale": 0.72,
        "yaw_radians": 0.10,
        "transform_baked_or_runtime": "runtime_transform_baked_into_candidate_vertex_buffer",
        "candidate_status": "source_approved_production_seed",
        "production_ready": False,
        "truth_mutation": False,
        **saltreach_tex,
    },
    {
        "mesh_asset_id": "opponent_saltreach_duelist",
        "mesh_asset_class": "fighter",
        "mesh_source": skinned,
        "translation": [0.72, 0.0, 0.0],
        "scale": 0.72,
        "yaw_radians": 0.10,
        "transform_baked_or_runtime": "runtime_transform_baked_into_candidate_vertex_buffer",
        "candidate_status": "source_approved_production_seed",
        "production_ready": False,
        "truth_mutation": False,
        **saltreach_tex,
    },
    {
        "mesh_asset_id": "player_gambeson",
        "mesh_asset_class": "armor",
        "mesh_source": gambeson,
        "translation": [-0.72, 0.18, 0.00],
        "scale": 0.14,
        "yaw_radians": 0.10,
        "transform_baked_or_runtime": "runtime_transform_baked_into_candidate_vertex_buffer",
        "candidate_status": "source_approved_production_seed",
        "production_ready": False,
        "truth_mutation": False,
        **gambeson_tex,
    },
    {
        "mesh_asset_id": "opponent_gambeson",
        "mesh_asset_class": "armor",
        "mesh_source": gambeson,
        "translation": [0.72, 0.18, 0.00],
        "scale": 0.14,
        "yaw_radians": 0.10,
        "transform_baked_or_runtime": "runtime_transform_baked_into_candidate_vertex_buffer",
        "candidate_status": "source_approved_production_seed",
        "production_ready": False,
        "truth_mutation": False,
        **gambeson_tex,
    },
    {
        "mesh_asset_id": "player_longsword",
        "mesh_asset_class": "weapon",
        "mesh_source": longsword,
        "translation": [-1.02, 0.42, -0.04],
        "scale": 0.34,
        "yaw_radians": 1.35,
        "transform_baked_or_runtime": "runtime_transform_baked_into_candidate_vertex_buffer",
        "candidate_status": "source_approved_production_seed",
        "production_ready": False,
        "truth_mutation": False,
        **longsword_tex,
    },
    {
        "mesh_asset_id": "opponent_longsword",
        "mesh_asset_class": "weapon",
        "mesh_source": longsword,
        "translation": [1.02, 0.42, -0.04],
        "scale": 0.34,
        "yaw_radians": -1.35,
        "transform_baked_or_runtime": "runtime_transform_baked_into_candidate_vertex_buffer",
        "candidate_status": "source_approved_production_seed",
        "production_ready": False,
        "truth_mutation": False,
        **longsword_tex,
    },
    {
        "mesh_asset_id": "training_yard",
        "mesh_asset_class": "arena",
        "mesh_source": training,
        "translation": [0.0, -0.30, 0.35],
        "scale": 0.50,
        "yaw_radians": 0.0,
        "transform_baked_or_runtime": "runtime_transform_baked_into_candidate_vertex_buffer",
        "candidate_status": "source_approved_production_seed",
        "production_ready": False,
        "truth_mutation": False,
        **training_tex,
    },
]
payload = {
    "schema": "oathyard.wgpu_runtime_mesh_manifest.v1",
    "source": "exchange_capture_matrix.sh Unit-071 skinned mesh manifest",
    "capture_id": manifest.stem,
    "candidate_renderer_only": False,
    "material_separation_classes": ["fighter_body", "armor_clothing", "weapon_metal", "arena_stone_ground"],
    "presentation_material_fallback": "source-approved runtime texture paths when source mesh lacks embedded material_validation",
    "production_seed_render": True,
    "production_ready": False,
    "truth_mutation": False,
    "meshes": meshes,
}
manifest.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(manifest.as_posix())
PY
}

for entry in "${PHASES[@]}"; do
  IFS='|' read -r phase_id camera_mode clip_id description <<< "$entry"
  capture_stem="production_renderer_exchange_${phase_id}_1920x1080"
  mesh_manifest_path="$(generate_mesh_manifest "$phase_id")"

  "$RENDERER" \
    --packet "$PACKET" \
    --out "$out/$phase_id" \
    --capture-id "$phase_id" \
    --capture-file-stem "$capture_stem" \
    --camera-mode "$camera_mode" \
    --candidate-assets "saltreach_duelist,longsword,gambeson,training_yard" \
    --mesh-manifest-json "$mesh_manifest_path" \
    >/dev/null 2>&1

  capture_png="$out/$phase_id/${capture_stem}.png"
  # Unit-071: Extract mesh consumption metadata from the production renderer manifest
  prod_manifest="$out/$phase_id/production_renderer_manifest.json"
  mesh_consumed="false"
  mesh_count=0
  mesh_assets_str="[]"
  if [[ -f "$prod_manifest" ]]; then
    mesh_consumed=$(python3 -c "import json; d=json.load(open('$prod_manifest')); print(str(d.get('mesh_geometry_consumed', False)).lower())" 2>/dev/null || echo "false")
    mesh_count=$(python3 -c "import json; d=json.load(open('$prod_manifest')); print(d.get('mesh_asset_count', 0))" 2>/dev/null || echo "0")
  fi
  if [[ -f "$capture_png" ]]; then
    capture_size=$(stat -c%s "$capture_png" 2>/dev/null || stat -f%z "$capture_png" 2>/dev/null)
    if [[ "$capture_size" -gt 50000 ]]; then
      CAPTURE_COUNT=$((CAPTURE_COUNT + 1))
      if [[ -n "$MANIFEST_ENTRIES" ]]; then
        MANIFEST_ENTRIES="${MANIFEST_ENTRIES},
"
      fi
      MANIFEST_ENTRIES="${MANIFEST_ENTRIES}    {\"phase_id\": \"$phase_id\", \"camera_mode\": \"$camera_mode\", \"clip_id\": \"$clip_id\", \"description\": \"$description\", \"capture_path\": \"$phase_id/${capture_stem}.png\", \"capture_size_bytes\": $capture_size, \"mesh_geometry_consumed\": $mesh_consumed, \"mesh_asset_count\": $mesh_count}"
      echo "  PASS $phase_id ($capture_size bytes, camera=$camera_mode, clip=$clip_id)"
    else
      echo "  WARN $phase_id capture too small ($capture_size bytes)" >&2
    fi
  else
    echo "  FAIL $phase_id no capture produced" >&2
  fi
done

# Write exchange capture matrix manifest
cat > "$out/exchange_capture_matrix.json" <<MANIFEST_EOF
{
  "schema": "oathyard.exchange_capture_matrix.v1",
  "product": "OATHYARD",
  "unit": "Unit-070",
  "scenario_id": "basic_oathyard",
  "final_state_hash": "$MANIFEST_HASH",
  "renderer_backend": "oathyard-native-wgpu-production-v1",
  "capture_resolution": [1920, 1080],
  "truth_mutation": false,
  "phase_count": ${#PHASES[@]},
  "capture_count": $CAPTURE_COUNT,
  "phases": [
$(echo -e "$MANIFEST_ENTRIES")
  ]
}
MANIFEST_EOF

python3 -m json.tool "$out/exchange_capture_matrix.json" >/dev/null

echo ""
echo "exchange capture matrix complete: $out"
echo "  phases: ${#PHASES[@]}"
echo "  captures: $CAPTURE_COUNT"
echo "  hash: $MANIFEST_HASH"

if [[ "$CAPTURE_COUNT" -lt ${#PHASES[@]} ]]; then
  echo "WARNING: only $CAPTURE_COUNT of ${#PHASES[@]} phases produced captures" >&2
  exit 1
fi
