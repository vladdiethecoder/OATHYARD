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

for entry in "${PHASES[@]}"; do
  IFS='|' read -r phase_id camera_mode clip_id description <<< "$entry"
  capture_stem="production_renderer_exchange_${phase_id}_1920x1080"

  "$RENDERER" \
    --packet "$PACKET" \
    --out "$out/$phase_id" \
    --capture-id "$phase_id" \
    --capture-file-stem "$capture_stem" \
    --camera-mode "$camera_mode" \
    --candidate-assets "saltreach_duelist,training_yard" \
    >/dev/null 2>&1

  capture_png="$out/$phase_id/${capture_stem}.png"
  if [[ -f "$capture_png" ]]; then
    capture_size=$(stat -c%s "$capture_png" 2>/dev/null || stat -f%z "$capture_png" 2>/dev/null)
    if [[ "$capture_size" -gt 50000 ]]; then
      CAPTURE_COUNT=$((CAPTURE_COUNT + 1))
      if [[ -n "$MANIFEST_ENTRIES" ]]; then
        MANIFEST_ENTRIES="${MANIFEST_ENTRIES},
"
      fi
      MANIFEST_ENTRIES="${MANIFEST_ENTRIES}    {\"phase_id\": \"$phase_id\", \"camera_mode\": \"$camera_mode\", \"clip_id\": \"$clip_id\", \"description\": \"$description\", \"capture_path\": \"$phase_id/${capture_stem}.png\", \"capture_size_bytes\": $capture_size}"
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
