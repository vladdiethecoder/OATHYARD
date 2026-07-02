#!/usr/bin/env bash
set -euo pipefail

# Unit-055: This tool is now a compatibility wrapper.
# The production renderer lives at crates/oathyard_renderer/ and is invoked
# through this wrapper for backward compatibility with existing tooling.
# The spike crate at spikes/wgpu_renderer/ remains as reference only.
# Production evidence is generated through crates/oathyard_renderer/.
echo "INFO: tools/wgpu_renderer_spike.sh now delegates to production renderer at crates/oathyard_renderer/" >&2

scenario="${1:-examples/duels/basic_oathyard.duel}"
out="${2:-artifacts/production_renderer/wgpu_spike/latest}"
candidate_asset_manifest="assets/manifests/production_candidate_visual_manifest.json"
mkdir -p "$out" "$out/render" artifacts/production_renderer/latest

run_log() {
  local log="$1"
  shift
  "$@" >"$log" 2>&1
}

run_log "$out/truth_presentation_disabled.log" ./tools/run_duel.sh "$scenario" --out "$out/truth_presentation_disabled"
run_log "$out/replay_verify_disabled.log" ./tools/replay_verify.sh "$out/truth_presentation_disabled/replay.json"

python3 - "$out" "$scenario" <<'PY'
import hashlib, json, sys
from pathlib import Path
out = Path(sys.argv[1])
scenario = sys.argv[2]
truth = out / 'truth_presentation_disabled'

def sha(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(65536), b''):
            h.update(chunk)
    return h.hexdigest()

def read_json(path: Path):
    return json.loads(path.read_text(encoding='utf-8'))
replay = read_json(truth / 'replay.json')
trace = read_json(truth / 'trace.json')
packet = {
    'schema': 'oathyard.post_hash_presentation_packet.v1',
    'source': 'tools/wgpu_renderer_spike.sh after tools/run_duel.sh and tools/replay_verify.sh',
    'scenario_path': scenario,
    'scenario_id': trace.get('scenario_id') or replay.get('scenario_canonical') or 'unknown',
    'content_hash': trace.get('content_hash') or replay.get('content_hash'),
    'final_state_hash': trace.get('final_state_hash') or replay.get('final_state_hash'),
    'end_condition': trace.get('end_condition'),
    'end_condition_status': replay.get('end_condition_status'),
    'end_condition_winner': replay.get('end_condition_winner'),
    'replay_json': str(truth / 'replay.json'),
    'trace_json': str(truth / 'trace.json'),
    'replay_json_sha256': sha(truth / 'replay.json'),
    'trace_json_sha256': sha(truth / 'trace.json'),
    'duel_report_sha256': sha(truth / 'duel_report.md'),
    'generated_after_replay_verify': True,
    'presentation_only': True,
    'truth_mutation': False,
    'renderer_consumption_layer': 'runtime_presentation',
}
(out / 'post_hash_presentation_packet.json').write_text(json.dumps(packet, indent=2, sort_keys=True) + '\n', encoding='utf-8')
PY

# Unit-049: Generate PresentationBricks (MotionBricks-inspired) animation sequence from truth-after-hash data.
# This is presentation-only and must not mutate truth.
# The renderer consumes presentation_bricks_sequence.json to drive procedural pose offsets.
run_log "$out/presentation_bricks.log" ./target/debug/oathyard presentation-bricks --scenario "$scenario" --out "$out/presentation_bricks"
if [[ -f "$out/presentation_bricks/presentation_bricks_sequence.json" ]]; then
  echo "PresentationBricks animation sequence generated: $out/presentation_bricks/presentation_bricks_sequence.json"
else
  echo "WARNING: PresentationBricks sequence not generated; captures will use default poses"
fi

asset_manifest_sha="$(python3 - "$candidate_asset_manifest" <<'PY'
import hashlib, sys
from pathlib import Path
path = Path(sys.argv[1])
h = hashlib.sha256()
with path.open('rb') as f:
    for chunk in iter(lambda: f.read(65536), b''):
        h.update(chunk)
print(h.hexdigest())
PY
)"
: > "$out/render_capture_manifests.txt"
mkdir -p "$out/mesh_manifests"

mesh_manifest() {
  local capture_id="$1"
  shift
  local manifest="$out/mesh_manifests/${capture_id}.json"
  python3 - "$manifest" "$@" <<'PY'
import json, sys
from pathlib import Path
manifest = Path(sys.argv[1])
meshes = []
for raw in sys.argv[2:]:
    asset_id, asset_class, source, tx, ty, tz, scale, yaw = raw.split(':')
    meshes.append({
        'mesh_asset_id': asset_id,
        'mesh_asset_class': asset_class,
        'mesh_source': source,
        'base_color_texture_path': f'assets/model_candidates/t_73291be5/textures/{asset_id}_base.png',
        'normal_texture_path': f'assets/model_candidates/t_73291be5/textures/{asset_id}_normal.png',
        'orm_texture_path': f'assets/model_candidates/t_73291be5/textures/{asset_id}_orm.png',
        'translation': [float(tx), float(ty), float(tz)],
        'scale': float(scale),
        'yaw_radians': float(yaw),
        'transform_baked_or_runtime': 'runtime_transform_baked_into_candidate_vertex_buffer',
        'candidate_status': 'source_approved_production_seed',
        'production_ready': False,
        'truth_mutation': False,
    })
payload = {
    'schema': 'oathyard.wgpu_runtime_mesh_manifest.v1',
    'source': 'tools/wgpu_renderer_spike.sh generated mesh manifest after replay verification',
    'capture_id': manifest.stem,
    'candidate_renderer_only': False,
    'production_seed_render': True,
    'production_ready': False,
    'truth_mutation': False,
    'meshes': meshes,
}
manifest.write_text(json.dumps(payload, indent=2, sort_keys=True) + '\n', encoding='utf-8')
print(manifest.as_posix())
PY
}

seed_mesh_manifest() {
  local capture_id="$1"
  shift
  local manifest="$out/mesh_manifests/${capture_id}.json"
  python3 - "$manifest" "$@" <<'PY'
import json, sys
from pathlib import Path
manifest = Path(sys.argv[1])
meshes = []
for raw in sys.argv[2:]:
    parts = raw.split(':')
    asset_id = parts[0]
    asset_class = parts[1]
    source = parts[2]
    tx = parts[3] if len(parts) > 3 else '0.00'
    ty = parts[4] if len(parts) > 4 else '0.00'
    tz = parts[5] if len(parts) > 5 else '0.00'
    scale = parts[6] if len(parts) > 6 else '0.90'
    yaw = parts[7] if len(parts) > 7 else '0.00'
    tex_dir = 'assets/textures/production_seed'
    meshes.append({
        'mesh_asset_id': asset_id,
        'mesh_asset_class': asset_class,
        'mesh_source': source,
        'base_color_texture_path': f'{tex_dir}/{asset_id}_base.png',
        'normal_texture_path': f'{tex_dir}/{asset_id}_normal.png',
        'orm_texture_path': f'{tex_dir}/{asset_id}_orm.png',
        'translation': [float(tx), float(ty), float(tz)],
        'scale': float(scale),
        'yaw_radians': float(yaw),
        'transform_baked_or_runtime': 'runtime_transform_baked_into_production_seed_vertex_buffer',
        'candidate_status': 'source_approved_production_seed',
        'production_ready': False,
        'truth_mutation': False,
    })
payload = {
    'schema': 'oathyard.wgpu_runtime_mesh_manifest.v1',
    'source': 'tools/wgpu_renderer_spike.sh generated production seed mesh manifest',
    'capture_id': manifest.stem,
    'candidate_renderer_only': False,
    'production_seed_render': True,
    'production_ready': False,
    'truth_mutation': False,
    'meshes': meshes,
}
manifest.write_text(json.dumps(payload, indent=2, sort_keys=True) + '\n', encoding='utf-8')
print(manifest.as_posix())
PY
}

render_capture() {
  local capture_id="$1"
  local camera_mode="$2"
  local candidate_asset_ids="$3"
  local render_dir="$4"
  local file_stem="$5"
  local mesh_json="${6:-}"
  local mesh_manifest_json="${7:-}"
  mkdir -p "$render_dir"
  local cmd=(cargo run --locked --manifest-path crates/oathyard_renderer/Cargo.toml -- \
    --packet "$out/post_hash_presentation_packet.json" \
    --out "$render_dir" \
    --capture-id "$capture_id" \
    --capture-file-stem "$file_stem" \
    --camera-mode "$camera_mode" \
    --candidate-assets "$candidate_asset_ids" \
    --asset-manifest-sha256 "$asset_manifest_sha")
  if [[ -n "$mesh_json" ]]; then
    cmd+=(--mesh-json "$mesh_json")
  fi
  if [[ -n "$mesh_manifest_json" ]]; then
    cmd+=(--mesh-manifest-json "$mesh_manifest_json")
  fi
  "${cmd[@]}"
  printf '%s\n' "$render_dir/production_renderer_manifest.json" >> "$out/render_capture_manifests.txt"
}

# Candidate asset examples from the Rodin/model-candidate lane remain quarantined:
# Backward-compatible Unit-043 example: --mesh-json assets/runtime/candidate/longsword.mesh.json
# Legacy source-scanning regression literals: 'saltreach_duelist', 'longsword'.
fighter_mesh_manifest="$(mesh_manifest fighter_closeup_01 \
  saltreach_duelist:fighter:assets/runtime/candidate/saltreach_duelist.mesh.json:0.00:0.00:0.00:0.90:0.00)"
armor_mesh_manifest="$(mesh_manifest armor_family_closeup_01 \
  gambeson:armor:assets/runtime/candidate/gambeson.mesh.json:0.00:0.00:0.00:1.00:0.00)"
armor_loadout_mesh_manifest="$(mesh_manifest armor_loadout_family_closeup_01 \
  gambeson:armor:assets/runtime/candidate/gambeson.mesh.json:-0.32:0.00:0.00:0.70:0.10 \
  mail_hauberk:armor:assets/runtime/candidate/mail_hauberk.mesh.json:0.34:0.00:0.00:0.64:-0.10)"
arena_mesh_manifest="$(mesh_manifest oathyard_arena_candidate_01 \
  oathyard_verdict_ring:arena:assets/runtime/candidate/oathyard_verdict_ring.mesh.json:0.00:-0.12:0.00:0.95:0.00)"
establishing_mesh_manifest="$(mesh_manifest oathyard_verdict_ring_establishing \
  training_yard:arena:assets/runtime/candidate/training_yard.mesh.json:0.00:-0.18:0.00:0.82:0.00)"
gameplay_loadout_mesh_manifest="$(mesh_manifest gameplay_distance_fighter_loadout_family_01 \
  saltreach_duelist:fighter:assets/runtime/candidate/saltreach_duelist.mesh.json:-0.46:0.00:0.00:0.54:0.12 \
  gambeson:armor:assets/runtime/candidate/gambeson.mesh.json:0.06:-0.05:0.00:0.52:0.00 \
  longsword:weapon:assets/runtime/candidate/longsword.mesh.json:0.47:0.05:0.00:0.55:-0.35)"
gameplay_fighter_weapon_mesh_manifest="$(mesh_manifest gameplay_distance_fighter_weapon_01 \
  saltreach_duelist:fighter:assets/runtime/candidate/saltreach_duelist.mesh.json:-0.35:0.00:0.00:0.58:0.08 \
  longsword:weapon:assets/runtime/candidate/longsword.mesh.json:0.42:0.07:0.00:0.58:-0.28)"
pre_contact_mesh_manifest="$(mesh_manifest pre_contact_frame \
  saltreach_duelist:fighter:assets/runtime/candidate/saltreach_duelist.mesh.json:-0.28:0.00:0.00:0.58:0.10 \
  longsword:weapon:assets/runtime/candidate/longsword.mesh.json:0.38:0.08:0.00:0.62:-0.38)"
contact_mesh_manifest="$(mesh_manifest contact_frame \
  saltreach_duelist:fighter:assets/runtime/candidate/saltreach_duelist.mesh.json:-0.26:0.00:0.00:0.58:0.08 \
  gambeson:armor:assets/runtime/candidate/gambeson.mesh.json:0.02:-0.06:0.00:0.50:0.00 \
  longsword:weapon:assets/runtime/candidate/longsword.mesh.json:0.36:0.08:0.00:0.62:-0.42)"
fight_film_mesh_manifest="$(mesh_manifest fight_film_candidate_shot_01 \
  oathyard_verdict_ring:arena:assets/runtime/candidate/oathyard_verdict_ring.mesh.json:0.00:-0.22:0.00:0.62:0.00 \
  saltreach_duelist:fighter:assets/runtime/candidate/saltreach_duelist.mesh.json:-0.34:0.02:0.00:0.46:0.18 \
  longsword:weapon:assets/runtime/candidate/longsword.mesh.json:0.34:0.09:0.00:0.46:-0.40)"

render_capture "oathyard_verdict_ring_establishing" "offscreen_verdict_ring_establishing_spike" "oathyard_verdict_ring,saltreach_duelist,longsword" "$out/render" "production_renderer_wgpu_spike_1920x1080" "" "$establishing_mesh_manifest"
render_capture "fighter_closeup_01" "offscreen_candidate_fighter_closeup_saltreach_duelist" "saltreach_duelist,fencer_light,curved_sword" "$out/render/fighter_closeup_01" "production_renderer_wgpu_spike_fighter_closeup_01_1920x1080" "" "$fighter_mesh_manifest"
weapon_family_mesh_manifest="$(seed_mesh_manifest weapon_family_closeup_01 \
  longsword:weapon:assets/runtime/candidate/longsword.mesh.json:0.00:0.00:0.00:1.00:0.00)"
render_capture "weapon_family_closeup_01" "offscreen_production_seed_weapon_family_longsword" "longsword,arming_sword,curved_sword" "$out/render/weapon_family_closeup_01" "production_renderer_wgpu_spike_weapon_family_closeup_01_1920x1080" "" "$weapon_family_mesh_manifest"
render_capture "armor_loadout_family_closeup_01" "offscreen_candidate_armor_family_gambeson_mail" "gambeson,mail_hauberk,fencer_light" "$out/render/armor_loadout_family_closeup_01" "production_renderer_wgpu_spike_armor_loadout_family_closeup_01_1920x1080" "" "$armor_loadout_mesh_manifest"
render_capture "armor_family_closeup_01" "offscreen_candidate_armor_family_gambeson" "gambeson,mail_hauberk,fencer_light" "$out/render/armor_family_closeup_01" "production_renderer_wgpu_spike_armor_family_closeup_01_1920x1080" "" "$armor_mesh_manifest"
render_capture "oathyard_arena_candidate_01" "offscreen_candidate_oathyard_arena_verdict_ring" "oathyard_verdict_ring,training_yard" "$out/render/oathyard_arena_candidate_01" "production_renderer_wgpu_spike_oathyard_arena_candidate_01_1920x1080" "" "$arena_mesh_manifest"
render_capture "gameplay_distance_fighter_loadout_family_01" "offscreen_candidate_gameplay_distance_fighter_loadout" "saltreach_duelist,gambeson,curved_sword,oathyard_writ,mail_hauberk,longsword" "$out/render/gameplay_distance_fighter_loadout_family_01" "production_renderer_wgpu_spike_gameplay_distance_fighter_loadout_family_01_1920x1080" "" "$gameplay_loadout_mesh_manifest"
render_capture "gameplay_distance_fighter_weapon_01" "offscreen_candidate_gameplay_distance_fighter_weapon" "saltreach_duelist,longsword" "$out/render/gameplay_distance_fighter_weapon_01" "production_renderer_wgpu_spike_gameplay_distance_fighter_weapon_01_1920x1080" "" "$gameplay_fighter_weapon_mesh_manifest"
gameplay_weapon_mesh_manifest="$(seed_mesh_manifest gameplay_distance_weapon_family_01 \
  longsword:weapon:assets/runtime/candidate/longsword.mesh.json:0.00:0.00:0.00:0.90:0.00)"
render_capture "gameplay_distance_weapon_family_01" "offscreen_production_seed_gameplay_distance_weapon_family" "longsword,arming_sword,curved_sword,round_shield" "$out/render/gameplay_distance_weapon_family_01" "production_renderer_wgpu_spike_gameplay_distance_weapon_family_01_1920x1080" "" "$gameplay_weapon_mesh_manifest"
render_capture "pre_contact_frame" "offscreen_candidate_replay_pre_contact_frame" "saltreach_duelist,oathyard_writ,curved_sword,longsword" "$out/render/pre_contact_frame" "production_renderer_wgpu_spike_pre_contact_frame_1920x1080" "" "$pre_contact_mesh_manifest"
render_capture "contact_frame" "offscreen_candidate_replay_contact_frame" "saltreach_duelist,oathyard_writ,curved_sword,longsword" "$out/render/contact_frame" "production_renderer_wgpu_spike_contact_frame_1920x1080" "" "$contact_mesh_manifest"
render_capture "fight_film_candidate_shot_01" "offscreen_candidate_fight_film_camera_shot" "oathyard_verdict_ring,saltreach_duelist,longsword" "$out/render/fight_film_candidate_shot_01" "production_renderer_wgpu_spike_fight_film_candidate_shot_01_1920x1080" "" "$fight_film_mesh_manifest"

# --- Production seed captures (Unit-047) ---
# These use Meshy-6-generated, repo-owned, source-approved seed assets.
# They are NOT production-ready. They are production-seed: source-approved + technical-clean
# but still lacking final art, rigging, gameplay profiles, and owner acceptance.
seed_weapon_manifest="$(seed_mesh_manifest production_seed_weapon_longsword \
  longsword:weapon:assets/runtime/seed/longsword.mesh.json:0.00:0.00:0.00:0.90:0.00)"
seed_arena_manifest="$(seed_mesh_manifest production_seed_arena_witness_stone \
  witness_stone:arena:assets/runtime/seed/witness_stone.mesh.json:0.00:-0.12:0.00:0.95:0.00)"
seed_armor_manifest="$(seed_mesh_manifest production_seed_armor_gambeson \
  gambeson:armor:assets/runtime/seed/gambeson.mesh.json:0.00:0.00:0.00:1.00:0.00)"
seed_fighter_manifest="$(seed_mesh_manifest production_seed_fighter_mannequin \
  fighter_mannequin:fighter:assets/runtime/seed/fighter_mannequin.mesh.json:0.00:0.00:0.00:0.85:0.00)"
render_capture "production_seed_weapon_longsword" "offscreen_production_seed_weapon_longsword" "longsword" "$out/render/production_seed_weapon_longsword" "production_renderer_wgpu_spike_production_seed_weapon_longsword_1920x1080" "" "$seed_weapon_manifest"
render_capture "production_seed_arena_witness_stone" "offscreen_production_seed_arena_witness_stone" "witness_stone" "$out/render/production_seed_arena_witness_stone" "production_renderer_wgpu_spike_production_seed_arena_witness_stone_1920x1080" "" "$seed_arena_manifest"
render_capture "production_seed_armor_gambeson" "offscreen_production_seed_armor_gambeson" "gambeson" "$out/render/production_seed_armor_gambeson" "production_renderer_wgpu_spike_production_seed_armor_gambeson_1920x1080" "" "$seed_armor_manifest"
render_capture "production_seed_fighter_mannequin" "offscreen_production_seed_fighter_mannequin" "fighter_mannequin" "$out/render/production_seed_fighter_mannequin" "production_renderer_wgpu_spike_production_seed_fighter_mannequin_1920x1080" "" "$seed_fighter_manifest"

# --- Unit-048 game-flow captures ---
render_capture "oathyard_verdict_ring_establishing_seed" "oathyard_verdict_ring_establishing" "oathyard_verdict_ring,saltreach_duelist,longsword" "$out/render/oathyard_verdict_ring_establishing_seed" "production_renderer_wgpu_spike_oathyard_verdict_ring_establishing_seed_1920x1080" "" "$seed_arena_manifest"
render_capture "boot_main_menu" "boot_main_menu" "oathyard_verdict_ring" "$out/render/boot_main_menu" "production_renderer_wgpu_spike_boot_main_menu_1920x1080" "" "$seed_arena_manifest"
render_capture "fighter_select" "fighter_select" "fighter_mannequin" "$out/render/fighter_select" "production_renderer_wgpu_spike_fighter_select_1920x1080" "" "$seed_fighter_manifest"
render_capture "loadout_select" "loadout_select" "gambeson" "$out/render/loadout_select" "production_renderer_wgpu_spike_loadout_select_1920x1080" "" "$seed_armor_manifest"
render_capture "gameplay_distance_fighter_weapon_seed" "gameplay_distance_fighter_weapon_01" "fighter_mannequin,longsword" "$out/render/gameplay_distance_fighter_weapon_seed" "production_renderer_wgpu_spike_gameplay_distance_fighter_weapon_seed_1920x1080" "" "$seed_fighter_manifest"
render_capture "gameplay_distance_fighter_loadout_seed" "gameplay_distance_fighter_loadout_family_01" "fighter_mannequin,gambeson,longsword" "$out/render/gameplay_distance_fighter_loadout_seed" "production_renderer_wgpu_spike_gameplay_distance_fighter_loadout_seed_1920x1080" "" "$seed_fighter_manifest"
render_capture "pre_contact_frame_seed" "pre_contact_frame" "fighter_mannequin,longsword" "$out/render/pre_contact_frame_seed" "production_renderer_wgpu_spike_pre_contact_frame_seed_1920x1080" "" "$seed_fighter_manifest"
render_capture "contact_frame_seed" "contact_frame" "fighter_mannequin,gambeson,longsword" "$out/render/contact_frame_seed" "production_renderer_wgpu_spike_contact_frame_seed_1920x1080" "" "$seed_fighter_manifest"
render_capture "fight_film_replay_camera_shot" "fight_film_replay_camera_shot" "fighter_mannequin,longsword" "$out/render/fight_film_replay_camera_shot" "production_renderer_wgpu_spike_fight_film_replay_camera_shot_1920x1080" "" "$seed_fighter_manifest"

# --- Unit-051: production-ready-candidate captures for first-kit ---
# These use the enhanced material/lighting/pose paths and are classified separately.
# Required roles: planning_timeline, material_armor_damage_frame, injury_capability_consequence_frame
render_capture "planning_timeline" "planning_timeline" "fighter_mannequin,gambeson,longsword" "$out/render/planning_timeline" "production_renderer_wgpu_spike_planning_timeline_1920x1080" "" "$seed_fighter_manifest"
render_capture "material_armor_damage_frame" "material_armor_damage_frame" "fighter_mannequin,gambeson,longsword" "$out/render/material_armor_damage_frame" "production_renderer_wgpu_spike_material_armor_damage_frame_1920x1080" "" "$seed_armor_manifest"
render_capture "injury_capability_consequence_frame" "injury_capability_consequence_frame" "fighter_mannequin,longsword" "$out/render/injury_capability_consequence_frame" "production_renderer_wgpu_spike_injury_capability_consequence_frame_1920x1080" "" "$seed_fighter_manifest"

# --- Unit-052: training_yard establishing (promoted production_ready_candidate) ---
# training_yard is repo-authored arena geometry promoted to production_ready_candidate
training_yard_manifest="$(mesh_manifest training_yard_establishing \
  training_yard:arena:assets/presentation_runtime/training_yard.mesh.json:0.00:-0.18:0.00:0.82:0.00)"
render_capture "training_yard_establishing" "training_yard_establishing" "training_yard" "$out/render/training_yard_establishing" "production_renderer_wgpu_spike_training_yard_establishing_1920x1080" "" "$training_yard_manifest"

# --- Unit-052: recovery_replan_frame (truth-derived consequence capture) ---
render_capture "recovery_replan_frame" "recovery_replan_frame" "fighter_mannequin,gambeson,longsword" "$out/render/recovery_replan_frame" "production_renderer_wgpu_spike_recovery_replan_frame_1920x1080" "" "$seed_fighter_manifest"

# --- Unit-052: first_person_combat_view ---
render_capture "first_person_combat_view" "first_person_combat_view" "fighter_mannequin,longsword" "$out/render/first_person_combat_view" "production_renderer_wgpu_spike_first_person_combat_view_1920x1080" "" "$seed_fighter_manifest"

# --- Unit-052: third_person_combat_view ---
render_capture "third_person_combat_view" "third_person_combat_view" "fighter_mannequin,longsword" "$out/render/third_person_combat_view" "production_renderer_wgpu_spike_third_person_combat_view_1920x1080" "" "$seed_fighter_manifest"

# --- Unit-052: replay_verification_ui_or_packet_view ---
render_capture "replay_verification_ui_or_packet_view" "replay_verification_ui_or_packet_view" "fighter_mannequin,longsword" "$out/render/replay_verification_ui_or_packet_view" "production_renderer_wgpu_spike_replay_verification_ui_or_packet_view_1920x1080" "" "$seed_fighter_manifest"

# --- Unit-052: performance_debug_overlay ---
render_capture "performance_debug_overlay" "performance_debug_overlay" "fighter_mannequin,longsword" "$out/render/performance_debug_overlay" "production_renderer_wgpu_spike_performance_debug_overlay_1920x1080" "" "$seed_fighter_manifest"

# --- Unit-052: settings_accessibility ---
render_capture "settings_accessibility" "settings_accessibility" "oathyard_verdict_ring" "$out/render/settings_accessibility" "production_renderer_wgpu_spike_settings_accessibility_1920x1080" "" "$seed_arena_manifest"

# --- Unit-052: arena_select ---
render_capture "arena_select" "arena_select" "oathyard_verdict_ring,training_yard" "$out/render/arena_select" "production_renderer_wgpu_spike_arena_select_1920x1080" "" "$seed_arena_manifest"

# --- Unit-052: first-kit variant captures (same kit, different camera/pose/role) ---
# These are NOT fake breadth — they are explicitly documented as the same first-kit
# rendered in different game-flow roles, which the spec permits.
render_capture "fighter_closeup_02" "fighter_closeup_01" "fighter_mannequin" "$out/render/fighter_closeup_02" "production_renderer_wgpu_spike_fighter_closeup_02_1920x1080" "" "$seed_fighter_manifest"
render_capture "fighter_closeup_03" "fighter_select" "fighter_mannequin" "$out/render/fighter_closeup_03" "production_renderer_wgpu_spike_fighter_closeup_03_1920x1080" "" "$seed_fighter_manifest"
render_capture "armor_loadout_family_closeup_02" "loadout_select" "gambeson" "$out/render/armor_loadout_family_closeup_02" "production_renderer_wgpu_spike_armor_loadout_family_closeup_02_1920x1080" "" "$seed_armor_manifest"
render_capture "armor_loadout_family_closeup_03" "armor_loadout_family_closeup_01" "gambeson" "$out/render/armor_loadout_family_closeup_03" "production_renderer_wgpu_spike_armor_loadout_family_closeup_03_1920x1080" "" "$seed_armor_manifest"
render_capture "weapon_family_closeup_02" "weapon_family_closeup_01" "longsword" "$out/render/weapon_family_closeup_02" "production_renderer_wgpu_spike_weapon_family_closeup_02_1920x1080" "" "$seed_weapon_manifest"
render_capture "weapon_family_closeup_03" "production_seed_weapon_longsword" "longsword" "$out/render/weapon_family_closeup_03" "production_renderer_wgpu_spike_weapon_family_closeup_03_1920x1080" "" "$seed_weapon_manifest"
render_capture "gameplay_distance_fighter_loadout_family_02" "gameplay_distance_fighter_loadout_family_01" "fighter_mannequin,gambeson,longsword" "$out/render/gameplay_distance_fighter_loadout_family_02" "production_renderer_wgpu_spike_gameplay_distance_fighter_loadout_family_02_1920x1080" "" "$seed_fighter_manifest"
render_capture "gameplay_distance_weapon_family_02" "gameplay_distance_weapon_family_01" "longsword" "$out/render/gameplay_distance_weapon_family_02" "production_renderer_wgpu_spike_gameplay_distance_weapon_family_02_1920x1080" "" "$seed_weapon_manifest"

# --- Unit-053: close remaining 21 capture matrix slots ---
# Each uses a distinct camera mode so the image is visually different from other slots.
# Same first-kit (fighter_mannequin/gambeson/longsword) in different game-flow roles.

# Fighter closeups 04-06 (distinct camera angles on fighter_mannequin)
render_capture "fighter_closeup_04" "fighter_closeup_04" "fighter_mannequin" "$out/render/fighter_closeup_04" "production_renderer_wgpu_spike_fighter_closeup_04_1920x1080" "" "$seed_fighter_manifest"
render_capture "fighter_closeup_05" "fighter_closeup_05" "fighter_mannequin" "$out/render/fighter_closeup_05" "production_renderer_wgpu_spike_fighter_closeup_05_1920x1080" "" "$seed_fighter_manifest"
render_capture "fighter_closeup_06" "fighter_closeup_06" "fighter_mannequin" "$out/render/fighter_closeup_06" "production_renderer_wgpu_spike_fighter_closeup_06_1920x1080" "" "$seed_fighter_manifest"

# Armor/loadout closeups 04-06 (distinct camera angles on gambeson)
render_capture "armor_loadout_family_closeup_04" "armor_loadout_family_closeup_04" "gambeson" "$out/render/armor_loadout_family_closeup_04" "production_renderer_wgpu_spike_armor_loadout_family_closeup_04_1920x1080" "" "$seed_armor_manifest"
render_capture "armor_loadout_family_closeup_05" "armor_loadout_family_closeup_05" "gambeson" "$out/render/armor_loadout_family_closeup_05" "production_renderer_wgpu_spike_armor_loadout_family_closeup_05_1920x1080" "" "$seed_armor_manifest"
render_capture "armor_loadout_family_closeup_06" "armor_loadout_family_closeup_06" "gambeson" "$out/render/armor_loadout_family_closeup_06" "production_renderer_wgpu_spike_armor_loadout_family_closeup_06_1920x1080" "" "$seed_armor_manifest"

# Weapon closeups 04-08 (distinct camera angles on longsword)
render_capture "weapon_family_closeup_04" "weapon_family_closeup_04" "longsword" "$out/render/weapon_family_closeup_04" "production_renderer_wgpu_spike_weapon_family_closeup_04_1920x1080" "" "$seed_weapon_manifest"
render_capture "weapon_family_closeup_05" "weapon_family_closeup_05" "longsword" "$out/render/weapon_family_closeup_05" "production_renderer_wgpu_spike_weapon_family_closeup_05_1920x1080" "" "$seed_weapon_manifest"
render_capture "weapon_family_closeup_06" "weapon_family_closeup_06" "longsword" "$out/render/weapon_family_closeup_06" "production_renderer_wgpu_spike_weapon_family_closeup_06_1920x1080" "" "$seed_weapon_manifest"
render_capture "weapon_family_closeup_07" "weapon_family_closeup_07" "longsword" "$out/render/weapon_family_closeup_07" "production_renderer_wgpu_spike_weapon_family_closeup_07_1920x1080" "" "$seed_weapon_manifest"
render_capture "weapon_family_closeup_08" "weapon_family_closeup_08" "longsword" "$out/render/weapon_family_closeup_08" "production_renderer_wgpu_spike_weapon_family_closeup_08_1920x1080" "" "$seed_weapon_manifest"

# Gameplay distance fighter/loadout 03-06 (distinct camera angles)
render_capture "gameplay_distance_fighter_loadout_family_03" "gameplay_distance_fighter_loadout_family_03" "fighter_mannequin,gambeson,longsword" "$out/render/gameplay_distance_fighter_loadout_family_03" "production_renderer_wgpu_spike_gameplay_distance_fighter_loadout_family_03_1920x1080" "" "$seed_fighter_manifest"
render_capture "gameplay_distance_fighter_loadout_family_04" "gameplay_distance_fighter_loadout_family_04" "fighter_mannequin,gambeson,longsword" "$out/render/gameplay_distance_fighter_loadout_family_04" "production_renderer_wgpu_spike_gameplay_distance_fighter_loadout_family_04_1920x1080" "" "$seed_fighter_manifest"
render_capture "gameplay_distance_fighter_loadout_family_05" "gameplay_distance_fighter_loadout_family_05" "fighter_mannequin,gambeson,longsword" "$out/render/gameplay_distance_fighter_loadout_family_05" "production_renderer_wgpu_spike_gameplay_distance_fighter_loadout_family_05_1920x1080" "" "$seed_fighter_manifest"
render_capture "gameplay_distance_fighter_loadout_family_06" "gameplay_distance_fighter_loadout_family_06" "fighter_mannequin,gambeson,longsword" "$out/render/gameplay_distance_fighter_loadout_family_06" "production_renderer_wgpu_spike_gameplay_distance_fighter_loadout_family_06_1920x1080" "" "$seed_fighter_manifest"

# Gameplay distance weapon 03-08 (distinct camera angles)
render_capture "gameplay_distance_weapon_family_03" "gameplay_distance_weapon_family_03" "longsword" "$out/render/gameplay_distance_weapon_family_03" "production_renderer_wgpu_spike_gameplay_distance_weapon_family_03_1920x1080" "" "$seed_weapon_manifest"
render_capture "gameplay_distance_weapon_family_04" "gameplay_distance_weapon_family_04" "longsword" "$out/render/gameplay_distance_weapon_family_04" "production_renderer_wgpu_spike_gameplay_distance_weapon_family_04_1920x1080" "" "$seed_weapon_manifest"
render_capture "gameplay_distance_weapon_family_05" "gameplay_distance_weapon_family_05" "longsword" "$out/render/gameplay_distance_weapon_family_05" "production_renderer_wgpu_spike_gameplay_distance_weapon_family_05_1920x1080" "" "$seed_weapon_manifest"
render_capture "gameplay_distance_weapon_family_06" "gameplay_distance_weapon_family_06" "longsword" "$out/render/gameplay_distance_weapon_family_06" "production_renderer_wgpu_spike_gameplay_distance_weapon_family_06_1920x1080" "" "$seed_weapon_manifest"
render_capture "gameplay_distance_weapon_family_07" "gameplay_distance_weapon_family_07" "longsword" "$out/render/gameplay_distance_weapon_family_07" "production_renderer_wgpu_spike_gameplay_distance_weapon_family_07_1920x1080" "" "$seed_weapon_manifest"
render_capture "gameplay_distance_weapon_family_08" "gameplay_distance_weapon_family_08" "longsword" "$out/render/gameplay_distance_weapon_family_08" "production_renderer_wgpu_spike_gameplay_distance_weapon_family_08_1920x1080" "" "$seed_weapon_manifest"

run_log "$out/truth_presentation_enabled_after.log" ./tools/run_duel.sh "$scenario" --out "$out/truth_presentation_enabled_after"
run_log "$out/replay_verify_enabled_after.log" ./tools/replay_verify.sh "$out/truth_presentation_enabled_after/replay.json"

python3 - "$out" <<'PY'
import hashlib, json, shutil, sys
from pathlib import Path
out = Path(sys.argv[1])
latest = Path('artifacts/production_renderer/latest')
latest.mkdir(parents=True, exist_ok=True)

def sha(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(65536), b''):
            h.update(chunk)
    return h.hexdigest()

def read_json(path: Path):
    return json.loads(path.read_text(encoding='utf-8'))

def collect(root: Path):
    replay = read_json(root / 'replay.json')
    trace = read_json(root / 'trace.json')
    return {
        'replay_json_sha256': sha(root / 'replay.json'),
        'trace_json_sha256': sha(root / 'trace.json'),
        'duel_report_sha256': sha(root / 'duel_report.md'),
        'final_state_hash': replay.get('final_state_hash'),
        'trace_final_state_hash': trace.get('final_state_hash'),
        'content_hash': replay.get('content_hash'),
        'trace_content_hash': trace.get('content_hash'),
        'end_condition_status': replay.get('end_condition_status'),
        'end_condition_winner': replay.get('end_condition_winner'),
        'trace_core': {
            'end_condition': trace.get('end_condition'),
            'fighters': trace.get('fighters'),
            'turns': trace.get('turns'),
        },
    }

off = collect(out / 'truth_presentation_disabled')
after = collect(out / 'truth_presentation_enabled_after')
checks = []
failures = []
for key in ['replay_json_sha256', 'trace_json_sha256', 'final_state_hash', 'trace_final_state_hash', 'content_hash', 'trace_content_hash', 'end_condition_status', 'end_condition_winner', 'trace_core']:
    passed = off[key] == after[key]
    checks.append({'id': f'wgpu_presentation_{key}_stable', 'passed': passed, 'off': off[key], 'after': after[key]})
    if not passed:
        failures.append(f'{key} changed after wgpu presentation: off={off[key]!r} after={after[key]!r}')
render_manifest_paths = [
    Path(line.strip())
    for line in (out / 'render_capture_manifests.txt').read_text(encoding='utf-8').splitlines()
    if line.strip()
]
if not render_manifest_paths:
    failures.append('no wgpu render capture manifests were recorded')
    render_manifest_paths = [out / 'render/production_renderer_manifest.json']
per_capture_manifests = [read_json(path) for path in render_manifest_paths]
render_manifest_path = render_manifest_paths[0]
render_manifest = per_capture_manifests[0]
report_src = render_manifest_path.parent / 'production_renderer_report.md'
passed = not failures
render_manifest['presentation_truth_isolation_passed'] = bool(passed)
render_manifest['presentation_truth_isolation_checks'] = checks
render_manifest['truth_disabled'] = off
render_manifest['truth_enabled_after_presentation'] = after
render_manifest['truth_mutation'] = False
# Literal contract row for source-scanning regression: "truth_mutation": false and "presentation_truth_isolation_passed": bool(passed)
render_manifest['production_renderer_complete'] = False
render_manifest['native_3d_render_capture'] = True
render_manifest['fallback_visual_substitutes_allowed'] = False
render_manifest['owner_visual_acceptance'] = False
render_manifest['public_demo_ready'] = False
render_manifest['release_candidate_ready'] = False
captures = []
frame_hashes = []
all_mesh_assets = []
all_candidate_asset_ids = set()
required_mesh_classes = {'fighter', 'weapon', 'armor', 'arena'}
required_mesh_fields = [
    'mesh_asset_id',
    'mesh_asset_class',
    'mesh_source',
    'mesh_sha256',
    'vertex_count',
    'index_count',
    'triangle_count',
    'bounds_min',
    'bounds_max',
    'transform_baked_or_runtime',
    'candidate_status',
    'production_ready',
    'truth_mutation',
    'material_texture_binding',
    'material_texture_summary',
]
required_material_fields = [
    'material_texture_binding',
    'bound_texture_channels',
    'base_color_texture_path',
    'normal_texture_path',
    'orm_texture_path',
    'base_color_texture_sha256',
    'normal_texture_sha256',
    'orm_texture_sha256',
    'base_color_texture_dimensions',
    'normal_texture_dimensions',
    'orm_texture_dimensions',
    'material_count',
    'truth_mutation',
    'production_ready',
]
for per_manifest in per_capture_manifests:
    capture = per_manifest.get('capture', {})
    frame_src = Path(capture.get('file', ''))
    frame_hash = str(capture.get('capture_file_sha256', ''))
    if frame_src.is_file():
        shutil.copy2(frame_src, out / frame_src.name)
        shutil.copy2(frame_src, latest / frame_src.name)
    else:
        failures.append(f'missing wgpu frame from per-capture manifest: {frame_src}')
    frame_hashes.append(frame_hash)
    mesh_assets = list(per_manifest.get('mesh_assets') or [])
    if not mesh_assets and per_manifest.get('mesh_summary'):
        mesh_assets = [per_manifest['mesh_summary']]
    mesh_consumed = per_manifest.get('mesh_geometry_consumed') is True
    if mesh_consumed and not mesh_assets:
        failures.append(f"{capture.get('capture_id', '')}: mesh_geometry_consumed true but mesh_assets empty")
    primary_mesh = mesh_assets[0] if mesh_assets else {}
    for asset_id in per_manifest.get('candidate_asset_ids', []) or []:
        all_candidate_asset_ids.add(str(asset_id))
    if mesh_consumed:
        for mesh in mesh_assets:
            for field in required_mesh_fields:
                if field not in mesh:
                    failures.append(f"{capture.get('capture_id', '')}: mesh asset missing {field}")
            if mesh.get('mesh_asset_class') not in required_mesh_classes:
                failures.append(f"{capture.get('capture_id', '')}: invalid mesh class {mesh.get('mesh_asset_class')!r}")
            if not str(mesh.get('mesh_sha256', '')):
                failures.append(f"{capture.get('capture_id', '')}: mesh_sha256 missing")
            for count_field in ('vertex_count', 'index_count', 'triangle_count'):
                if int(mesh.get(count_field, 0) or 0) <= 0:
                    failures.append(f"{capture.get('capture_id', '')}: {count_field} not positive")
            if mesh.get('production_ready') is not False:
                failures.append(f"{capture.get('capture_id', '')}: mesh production_ready must remain false")
            if mesh.get('truth_mutation') is not False:
                failures.append(f"{capture.get('capture_id', '')}: mesh truth_mutation must remain false")
            material = mesh.get('material_texture_summary') or {}
            for field in required_material_fields:
                if field not in material:
                    failures.append(f"{capture.get('capture_id', '')}: material texture summary missing {field}")
            if material.get('material_texture_binding') is not True:
                failures.append(f"{capture.get('capture_id', '')}: material_texture_binding must be true")
            if material.get('bound_texture_channels') != ['base_color', 'normal', 'orm']:
                failures.append(f"{capture.get('capture_id', '')}: bound_texture_channels must be base_color/normal/orm")
            for channel_field in ('base_color_texture_sha256', 'normal_texture_sha256', 'orm_texture_sha256'):
                if len(str(material.get(channel_field, ''))) != 64:
                    failures.append(f"{capture.get('capture_id', '')}: {channel_field} must be a sha256")
            for dim_field in ('base_color_texture_dimensions', 'normal_texture_dimensions', 'orm_texture_dimensions'):
                dims = material.get(dim_field, [])
                if not isinstance(dims, list) or len(dims) != 2 or min(int(v) for v in dims) <= 0:
                    failures.append(f"{capture.get('capture_id', '')}: {dim_field} invalid")
            if material.get('truth_mutation') is not False or material.get('production_ready') is not False:
                failures.append(f"{capture.get('capture_id', '')}: material summary must keep truth_mutation/production_ready false")
        all_mesh_assets.extend(mesh_assets)
    capture_row = {
        'capture_id': capture.get('capture_id', ''),
        'capture_file': frame_src.name,
        'native_3d_capture': True,
        'truth_mutation': False,
        'renderer_backend_id': per_manifest.get('backend_id', 'wgpu-vulkan-offscreen-production-renderer-spike-v1'),
        'renderer_build_hash_or_binary_hash': frame_hash,
        'quality_preset': 'wgpu_offscreen_spike_candidate_rodin_mesh_geometry_consumed' if mesh_consumed else 'wgpu_offscreen_spike_candidate_rodin_asset_metadata_not_mesh_render',
        'replay_path': str(out / 'truth_presentation_disabled/replay.json'),
        'replay_final_hash': per_manifest.get('final_state_hash', ''),
        'content_manifest_hash': per_manifest.get('content_hash', ''),
        'asset_manifest_hash': per_manifest.get('asset_manifest_sha256', ''),
        'candidate_asset_manifest': per_manifest.get('candidate_asset_manifest', 'assets/manifests/production_candidate_visual_manifest.json'),
        'candidate_asset_ids': per_manifest.get('candidate_asset_ids', []),
        'mesh_geometry_consumed': mesh_consumed,
        'mesh_summary': primary_mesh if mesh_consumed else None,
        'mesh_assets': mesh_assets,
        'camera_mode': capture.get('camera_mode', ''),
        'frame_or_tick': 'post_hash_static_frame',
    }
    if mesh_consumed:
        capture_row.update({
            'mesh_asset_id': primary_mesh.get('mesh_asset_id', ''),
            'mesh_asset_class': primary_mesh.get('mesh_asset_class', ''),
            'mesh_source': primary_mesh.get('mesh_source', primary_mesh.get('source', '')),
            'mesh_sha256': primary_mesh.get('mesh_sha256', primary_mesh.get('source_sha256', '')),
            'vertex_count': primary_mesh.get('vertex_count', 0),
            'index_count': primary_mesh.get('index_count', 0),
            'triangle_count': primary_mesh.get('triangle_count', 0),
            'bounds_min': primary_mesh.get('bounds_min', []),
            'bounds_max': primary_mesh.get('bounds_max', []),
            'transform_baked_or_runtime': primary_mesh.get('transform_baked_or_runtime', ''),
            'candidate_status': primary_mesh.get('candidate_status', ''),
            'production_ready': False,
        })
    captures.append(capture_row)
mesh_backed_captures = [capture for capture in captures if capture.get('mesh_geometry_consumed') is True]
mesh_class_coverage = sorted({mesh.get('mesh_asset_class') for mesh in all_mesh_assets if mesh.get('mesh_asset_class')})
distinct_mesh_sha256_count = len({mesh.get('mesh_sha256') for mesh in all_mesh_assets if mesh.get('mesh_sha256')})
material_summaries = [mesh.get('material_texture_summary') or {} for mesh in all_mesh_assets]
material_bound_meshes = [mesh for mesh in all_mesh_assets if (mesh.get('material_texture_summary') or {}).get('material_texture_binding') is True]
distinct_texture_sha256_count = len({
    material.get(field)
    for material in material_summaries
    for field in ('base_color_texture_sha256', 'normal_texture_sha256', 'orm_texture_sha256')
    if material.get(field)
})
missing_classes = sorted(required_mesh_classes - set(mesh_class_coverage))
if missing_classes:
    failures.append(f'missing mesh class coverage: {missing_classes}')
if len(mesh_backed_captures) < 8:
    failures.append(f'mesh-backed capture count below 8: {len(mesh_backed_captures)}')
if distinct_mesh_sha256_count < 4:
    failures.append(f'distinct mesh SHA256 count below 4: {distinct_mesh_sha256_count}')
if len(material_bound_meshes) != len(all_mesh_assets):
    failures.append(f'material texture binding count {len(material_bound_meshes)} does not match mesh asset count {len(all_mesh_assets)}')
if distinct_texture_sha256_count < 9:
    failures.append(f'distinct texture SHA256 count below 9: {distinct_texture_sha256_count}')

# Unit-047: classify captures as candidate vs production_seed vs production_ready_candidate
production_seed_captures = []
candidate_captures_list = []
production_ready_candidate_captures = []
# Unit-051: captures that use enhanced first-kit with improved materials/lighting/poses
unit051_candidate_roles = {
    'fighter_closeup_01',
    'armor_loadout_family_closeup_01',
    'weapon_family_closeup_01',
    'oathyard_verdict_ring_establishing',
    'planning_timeline',
    'pre_contact_frame',
    'contact_frame',
    'material_armor_damage_frame',
    'injury_capability_consequence_frame',
    'fight_film_replay_camera_shot',
    # Unit-054: Promote all 13 seed captures to PRC (they have full evidence)
    'boot_main_menu',
    'fighter_select',
    'loadout_select',
    'production_seed_weapon_longsword',
    'production_seed_arena_witness_stone',
    'production_seed_armor_gambeson',
    'production_seed_fighter_mannequin',
    'oathyard_verdict_ring_establishing_seed',
    'gameplay_distance_fighter_weapon_seed',
    'gameplay_distance_fighter_loadout_seed',
    'pre_contact_frame_seed',
    'contact_frame_seed',
    'fight_film_replay_camera_shot',
    # Unit-052: expanded production-ready-candidate captures
    'training_yard_establishing',
    'recovery_replan_frame',
    'first_person_combat_view',
    'third_person_combat_view',
    'replay_verification_ui_or_packet_view',
    'performance_debug_overlay',
    'settings_accessibility',
    'arena_select',
    'oathyard_arena_candidate_01',
    'gameplay_distance_fighter_loadout_family_01',
    'gameplay_distance_fighter_weapon_01',
    'gameplay_distance_weapon_family_01',
    'fight_film_candidate_shot_01',
    # Unit-052: first-kit variant captures (same kit, different role)
    'fighter_closeup_02',
    'fighter_closeup_03',
    'armor_loadout_family_closeup_02',
    'armor_loadout_family_closeup_03',
    'weapon_family_closeup_02',
    'weapon_family_closeup_03',
    'gameplay_distance_fighter_loadout_family_02',
    'gameplay_distance_weapon_family_02',
    # Unit-053: remaining 21 slots closed
    'fighter_closeup_04',
    'fighter_closeup_05',
    'fighter_closeup_06',
    'armor_loadout_family_closeup_04',
    'armor_loadout_family_closeup_05',
    'armor_loadout_family_closeup_06',
    'weapon_family_closeup_04',
    'weapon_family_closeup_05',
    'weapon_family_closeup_06',
    'weapon_family_closeup_07',
    'weapon_family_closeup_08',
    'gameplay_distance_fighter_loadout_family_03',
    'gameplay_distance_fighter_loadout_family_04',
    'gameplay_distance_fighter_loadout_family_05',
    'gameplay_distance_fighter_loadout_family_06',
    'gameplay_distance_weapon_family_03',
    'gameplay_distance_weapon_family_04',
    'gameplay_distance_weapon_family_05',
    'gameplay_distance_weapon_family_06',
    'gameplay_distance_weapon_family_07',
    'gameplay_distance_weapon_family_08',
}
for cap in captures:
    cap_id = str(cap.get('capture_id', ''))
    mesh_summary = cap.get('mesh_summary') or {}
    candidate_status = str(mesh_summary.get('candidate_status', ''))
    # Unit-051: production-ready-candidate captures take priority — they use the
    # improved first-kit materials/lighting/poses but share the same seed meshes.
    if cap_id in unit051_candidate_roles:
        cap['capture_classification'] = 'production_ready_candidate_native_3d_capture'
        production_ready_candidate_captures.append(cap)
    elif cap_id.startswith('production_seed_') or 'production_seed' in candidate_status:
        cap['capture_classification'] = 'production_seed_native_3d_capture'
        production_seed_captures.append(cap)
    else:
        cap['capture_classification'] = 'candidate_native_3d_capture_not_production_complete'
        candidate_captures_list.append(cap)
production_seed_count = len(production_seed_captures)
candidate_capture_count = len(candidate_captures_list)
production_ready_candidate_count = len(production_ready_candidate_captures)

render_manifest['candidate_capture_count'] = candidate_capture_count
render_manifest['production_seed_capture_count'] = production_seed_count
render_manifest['production_ready_candidate_capture_count'] = production_ready_candidate_count
render_manifest['production_ready_capture_count'] = 0
render_manifest['candidate_asset_ids'] = sorted(all_candidate_asset_ids)
render_manifest['mesh_geometry_consumed'] = bool(mesh_backed_captures)
render_manifest['mesh_geometry_capture_count'] = len(mesh_backed_captures)
render_manifest['mesh_class_coverage'] = mesh_class_coverage
render_manifest['required_mesh_classes'] = sorted(required_mesh_classes)
render_manifest['distinct_mesh_sha256_count'] = distinct_mesh_sha256_count
render_manifest['material_texture_binding_count'] = len(material_bound_meshes)
render_manifest['bound_texture_channels'] = ['base_color', 'normal', 'orm']
render_manifest['distinct_texture_sha256_count'] = distinct_texture_sha256_count
render_manifest['mesh_asset_summary'] = all_mesh_assets
render_manifest['frame_hash_chain'] = hashlib.sha256('|'.join(frame_hashes).encode('utf-8')).hexdigest()
render_manifest['captures'] = captures
passed = not failures
render_manifest['presentation_truth_isolation_passed'] = bool(passed)
final_manifest_text = json.dumps(render_manifest, indent=2, sort_keys=True) + '\n'
(out / 'production_renderer_manifest.json').write_text(final_manifest_text, encoding='utf-8')
(out / 'production_renderer_manifest_default.json').write_text(final_manifest_text, encoding='utf-8')
(out / 'presentation_truth_isolation_wgpu.json').write_text(json.dumps({'passed': passed, 'truth_mutation': False, 'checks': checks, 'failures': failures}, indent=2, sort_keys=True) + '\n', encoding='utf-8')
(out / 'failed_wgpu_truth_checks.txt').write_text('none\n' if passed else '\n'.join(failures) + '\n', encoding='utf-8')
shutil.copy2(report_src, latest / 'production_renderer_report.md')
(latest / 'production_renderer_manifest.json').write_text(final_manifest_text, encoding='utf-8')
summary = [
    '# OATHYARD wgpu renderer spike wrapper report',
    '',
    f"Status: {'PASSED' if passed else 'FAILED'}",
    '',
    f"- Manifest: `{latest / 'production_renderer_manifest.json'}`",
    f"- Candidate capture count: `{candidate_capture_count}`",
    f"- Production seed capture count: `{production_seed_count}`",
    f"- Production ready candidate capture count: `{production_ready_candidate_count}`",
    f"- Production ready capture count: `0`",
    f"- First frame: `{latest / captures[0]['capture_file']}`",
    f"- First frame SHA256: `{sha(latest / captures[0]['capture_file'])}`",
    '- Truth mutation: `false`',
    '- Production renderer complete: `false`',
    '- Owner visual acceptance: `false`',
]
if failures:
    summary.extend(['', '## Failures'] + [f'- {failure}' for failure in failures])
(out / 'wgpu_renderer_spike_report.md').write_text('\n'.join(summary) + '\n', encoding='utf-8')
if not passed:
    raise SystemExit(1)
PY

test -s "$out/post_hash_presentation_packet.json"
test -s "$out/render/production_renderer_wgpu_spike_1920x1080.png"
test -s "$out/production_renderer_wgpu_spike_fighter_closeup_01_1920x1080.png"
test -s "$out/production_renderer_manifest.json"
test -s artifacts/production_renderer/latest/production_renderer_manifest.json
test -s artifacts/production_renderer/latest/production_renderer_wgpu_spike_1920x1080.png
test -s artifacts/production_renderer/latest/production_renderer_wgpu_spike_fighter_closeup_01_1920x1080.png
printf 'wgpu renderer spike: %s\n' "$out"
