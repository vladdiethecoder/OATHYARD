#!/usr/bin/env bash
set -euo pipefail

final_dir="artifacts/final"
mkdir -p "$final_dir"

log_run() {
  local log="$1"
  shift
  "$@" 2>&1 | tee "$log"
}

require_file() {
  if [[ ! -f "$1" ]]; then
    echo "missing required file: $1" >&2
    exit 1
  fi
}

safe_empty_generated_dir() {
  local dir="$1"
  case "$dir" in
    artifacts/verify_a|artifacts/verify_b|artifacts/export_bundle/verify|artifacts/native_combat/verify|artifacts/pbr_materials/verify) ;;
    *)
      echo "refusing to clean unexpected generated directory: $dir" >&2
      exit 1
      ;;
  esac
  if [[ -d "$dir" ]]; then
    find "$dir" -mindepth 1 -depth -delete
  fi
}

for file in \
  README.md \
  AGENTS.md \
  ACCEPTANCE_MAP.md \
  docs/design/GAME_CANON.md \
  docs/design/DEMO_SCOPE.md \
  docs/decisions/0002-native-presentation-target.md \
  examples/duels/basic_oathyard.duel \
  examples/duels/axe_vs_spear.duel \
  src/lib.rs \
  src/bin/oathyard.rs \
  tools/build.sh \
  tools/test.sh \
  tools/run_duel.sh \
  tools/replay_verify.sh \
  tools/export_replay_bundle.sh \
  tools/verify_replay_bundle.sh \
  tools/native_combat_render.sh \
  tools/pbr_materials.sh \
  tools/audit_3d_runtime.sh \
  tools/renderer_target_audit.sh \
  tools/audit_environment.sh \
  tools/audit_truth.sh \
  tools/audit_visual_artifacts.sh \
  tools/test_visual_artifact_audit.sh \
  tools/audit_secrets.sh \
  tools/contact_matrix.sh \
  tools/build_assets.sh \
  tools/validate_assets.sh \
  tools/negative_audit.sh \
  tools/ai_duel.sh \
  tools/ai_sweep.sh \
  tools/truth_stress.sh \
  tools/truth_edge_audit.sh \
  tools/asset_provenance_audit.sh \
  tools/cross_platform_verify.sh \
  tools/cross_platform_hash_exchange.sh; do
  require_file "$file"
done

log_run "$final_dir/build.log" ./tools/build.sh
log_run "$final_dir/test.log" ./tools/test.sh
log_run "$final_dir/asset_build.log" ./tools/build_assets.sh
log_run "$final_dir/asset_validation.log" ./tools/validate_assets.sh

for generated_dir in \
  artifacts/verify_a \
  artifacts/verify_b \
  artifacts/export_bundle/verify \
  artifacts/native_combat/verify \
  artifacts/pbr_materials/verify; do
  safe_empty_generated_dir "$generated_dir"
  mkdir -p "$generated_dir"
done

log_run "$final_dir/deterministic_run_a.log" ./tools/run_duel.sh examples/duels/basic_oathyard.duel --out artifacts/verify_a
log_run "$final_dir/deterministic_run_b.log" ./tools/run_duel.sh examples/duels/basic_oathyard.duel --out artifacts/verify_b
cmp artifacts/verify_a/final_state_hash.txt artifacts/verify_b/final_state_hash.txt
cmp artifacts/verify_a/replay.json artifacts/verify_b/replay.json
cmp artifacts/verify_a/trace.json artifacts/verify_b/trace.json
cmp artifacts/verify_a/fight_film_manifest.json artifacts/verify_b/fight_film_manifest.json

for artifact in trace.json replay.json final_state_hash.txt duel_report.md fight_film_manifest.json; do
  test -s "artifacts/verify_a/$artifact"
done

grep -q '"schema": "oathyard.trace.v1"' artifacts/verify_a/trace.json
grep -q '"schema": "oathyard.replay.v1"' artifacts/verify_a/replay.json
grep -q '"schema": "oathyard.fight_film_manifest.v1"' artifacts/verify_a/fight_film_manifest.json
python3 -m json.tool artifacts/verify_a/trace.json >/dev/null
python3 -m json.tool artifacts/verify_a/replay.json >/dev/null
python3 -m json.tool artifacts/verify_a/fight_film_manifest.json >/dev/null

grep -q 'base_cost_frames' artifacts/verify_a/trace.json
grep -q 'current_cost_frames' artifacts/verify_a/trace.json
grep -q 'cause_chain' artifacts/verify_a/trace.json
grep -q '"contact_order_rule": "frame_then_attacker_then_defender_then_action_then_target_then_direction"' artifacts/verify_a/trace.json
grep -q '## End Condition' artifacts/verify_a/duel_report.md

log_run "$final_dir/replay_verification.log" ./tools/replay_verify.sh artifacts/verify_a/replay.json
log_run "$final_dir/export_bundle.log" ./tools/export_replay_bundle.sh artifacts/verify_a/replay.json artifacts/export_bundle/verify
log_run "$final_dir/export_bundle_verify.log" ./tools/verify_replay_bundle.sh artifacts/export_bundle/verify
log_run "$final_dir/truth_audit.log" ./tools/audit_truth.sh
log_run "$final_dir/contact_matrix.log" ./tools/contact_matrix.sh artifacts/contact_matrix/verify
log_run "$final_dir/ai_duel.log" ./tools/ai_duel.sh artifacts/ai/verify 6
log_run "$final_dir/ai_sweep.log" ./tools/ai_sweep.sh artifacts/ai_sweep/verify
log_run "$final_dir/truth_stress.log" ./tools/truth_stress.sh artifacts/truth_stress/verify
log_run "$final_dir/truth_edge_audit.log" ./tools/truth_edge_audit.sh artifacts/truth_edge/verify
log_run "$final_dir/negative_input_audit.log" ./tools/negative_audit.sh artifacts/negative_audit/verify

log_run "$final_dir/environment_audit.log" ./tools/audit_environment.sh artifacts/environment/verify
log_run "$final_dir/native_combat_render.log" ./tools/native_combat_render.sh examples/duels/basic_oathyard.duel artifacts/native_combat/verify
log_run "$final_dir/pbr_materials.log" ./tools/pbr_materials.sh examples/duels/basic_oathyard.duel artifacts/pbr_materials/verify
log_run "$final_dir/renderer_target_audit.log" ./tools/renderer_target_audit.sh artifacts/renderer_target/verify

# Unit-070: Require promoted native combat render schema for current-run evidence.
# The promoted schema indicates real native 3D renderer capture was produced.
if ! grep -q '"native_3d_visual_evidence_present":true' artifacts/native_combat/verify/native_combat_render_manifest.json; then
  echo "ERROR: native combat render did not produce promoted evidence (native_3d_visual_evidence_present:true)" >&2
  exit 1
fi
grep -q '"source":"truth-after-hash-duel-result"' artifacts/native_combat/verify/native_combat_render_manifest.json
grep -q '"forbidden_visual_fallbacks_emitted":false' artifacts/native_combat/verify/native_combat_render_manifest.json
grep -q '"visual_evidence_status":"native_3d_renderer_capture_present"' artifacts/native_combat/verify/native_combat_render_manifest.json
grep -q '"all_required_channels_covered": true' artifacts/pbr_materials/verify/pbr_material_manifest.json
grep -q '"flat_recolor_rejected": true' artifacts/pbr_materials/verify/pbr_material_manifest.json
grep -q 'Status: PASSED' artifacts/renderer_target/verify/native_presentation_target_report.md

log_run "$final_dir/visual_artifact_audit.log" ./tools/audit_visual_artifacts.sh

log_run "$final_dir/cross_platform_verify.log" ./tools/cross_platform_verify.sh --out artifacts/cross_platform/verify
test -s "artifacts/cross_platform/verify/cross_platform_matrix.json"
grep -q '"schema": "oathyard.cross_platform_matrix.v1"' artifacts/cross_platform/verify/cross_platform_matrix.json
grep -q '"hashes_match": true' artifacts/cross_platform/verify/cross_platform_matrix.json

echo "verify passed: truth/replay gates with promoted native 3D visual evidence"
