#!/usr/bin/env bash
set -euo pipefail

stamp="$(date -u +%Y%m%dT%H%M%SZ)"
run_dir="artifacts/publishable/$stamp"
mkdir -p "$run_dir"

{
  echo "started_utc=$stamp"
  echo "workdir=$(pwd)"
  echo "gate=publishable-local-package-candidate"
} > "$run_dir/metadata.env"

git status --short --branch > "$run_dir/git_status.txt" 2>&1 || true

set +e
./tools/verify.sh 2>&1 | tee "$run_dir/verify.log"
verify_rc=${PIPESTATUS[0]}
set -e
printf '%s\n' "$verify_rc" > "$run_dir/verify.rc"

set +e
./tools/visual_evidence_index.sh "$run_dir/visual_evidence" 2>&1 | tee "$run_dir/visual_evidence.log"
visual_rc=${PIPESTATUS[0]}
set -e
printf '%s\n' "$visual_rc" > "$run_dir/visual_evidence.rc"

if [[ -f artifacts/verify_a/final_state_hash.txt ]]; then
  cp artifacts/verify_a/final_state_hash.txt "$run_dir/final_state_hash.txt"
fi
if [[ -f artifacts/final/deterministic_run_hashes.txt ]]; then
  cp artifacts/final/deterministic_run_hashes.txt "$run_dir/deterministic_run_hashes.txt"
fi
if [[ -f artifacts/final/final_acceptance_report.md ]]; then
  cp artifacts/final/final_acceptance_report.md "$run_dir/final_acceptance_report.md"
fi

{
  for path in \
    ACCEPTANCE_MAP.md \
    docs/roadmap/PUBLISHABLE_KANBAN.md \
    docs/acceptance/FULL_GAME_ACCEPTANCE.md \
    docs/decisions/0002-native-presentation-target.md \
    docs/decisions/0003-native-input-model.md \
    docs/decisions/0004-audio-runtime-target.md \
    artifacts/verify_a/final_state_hash.txt \
    artifacts/verify_a/replay.json \
    artifacts/verify_a/trace.json \
    artifacts/truth_stress/verify/truth_stress.json \
    artifacts/truth_stress/verify/truth_stress_report.md \
    artifacts/truth_edge/verify/truth_edge_audit.json \
    artifacts/truth_edge/verify/truth_edge_audit_report.md \
    artifacts/negative_audit/verify/negative_input_audit.json \
    artifacts/negative_audit/verify/negative_input_audit_report.md \
    artifacts/desktop_metadata/verify/desktop_metadata.json \
    artifacts/desktop_metadata/verify/desktop_metadata_report.md \
    artifacts/readiness/source/readiness_audit.json \
    artifacts/readiness/source/readiness_audit_report.md \
    artifacts/readiness/package/readiness_audit.json \
    artifacts/readiness/package/readiness_audit_report.md \
    artifacts/secrets/source/secrets_audit.json \
    artifacts/secrets/source/secrets_audit_report.md \
    artifacts/secrets/package/secrets_audit.json \
    artifacts/secrets/package/secrets_audit_report.md \
    artifacts/environment/verify/environment_audit.json \
    artifacts/environment/verify/environment_audit_report.md \
    artifacts/asset_budget/verify/asset_budget.json \
    artifacts/asset_budget/verify/asset_budget_report.md \
    artifacts/asset_atlas/verify/asset_visual_atlas_manifest.json \
    artifacts/asset_atlas/verify/asset_visual_atlas_report.md \
    artifacts/asset_atlas/verify/asset_visual_atlas.svg \
    artifacts/asset_atlas/verify/asset_visual_atlas_hashes.sha256 \
    artifacts/asset_atlas/verify/failed_asset_visuals.txt \
    artifacts/runtime_3d/verify/runtime_3d_audit.json \
    artifacts/runtime_3d/verify/runtime_3d_audit_report.md \
    artifacts/renderer_target/verify/native_presentation_target.json \
    artifacts/renderer_target/verify/native_presentation_target_report.md \
    artifacts/visual_evidence/verify/visual_evidence_manifest.json \
    artifacts/visual_evidence/verify/visual_evidence_report.md \
    artifacts/visual_evidence/verify/visual_evidence_contact_sheet.svg \
    artifacts/visual_evidence/verify/visual_evidence_hashes.sha256 \
    artifacts/visual_evidence/verify/failed_visual_artifacts.txt \
    "$run_dir/visual_evidence/visual_evidence_manifest.json" \
    "$run_dir/visual_evidence/visual_evidence_report.md" \
    "$run_dir/visual_evidence/visual_evidence_contact_sheet.svg" \
    "$run_dir/visual_evidence/visual_evidence_hashes.sha256" \
    "$run_dir/visual_evidence/failed_visual_artifacts.txt" \
    artifacts/input_target/verify/native_input_target.json \
    artifacts/input_target/verify/native_input_target_report.md \
    artifacts/export_bundle/verify/export_bundle_manifest.json \
    artifacts/export_bundle/verify/export_bundle_report.md \
    artifacts/export_bundle/verify/bundle_hashes.txt \
    packaging/linux/io.oathyard.OATHYARD.desktop \
    packaging/linux/io.oathyard.OATHYARD.svg \
    artifacts/input/verify/input_profile.json \
    artifacts/input/verify/steam_deck_checklist.md \
    artifacts/settings/verify/runtime_settings.saved.json \
    artifacts/settings/verify/runtime_settings.loaded.json \
    artifacts/settings/verify/runtime_settings_report.md \
    artifacts/audio_mixer/verify/runtime_audio_mix.wav \
    artifacts/audio_mixer/verify/audio_mixer_settings.json \
    artifacts/audio_mixer/verify/audio_mixer_channels.json \
    artifacts/audio_mixer/verify/audio_mixer_loudness.json \
    artifacts/audio_mixer/verify/audio_mixer_report.md \
    artifacts/audio_device/verify/audio_device_smoke.json \
    artifacts/audio_target/verify/audio_runtime_target.json \
    artifacts/audio_target/verify/audio_runtime_target_report.md \
    artifacts/package_smoke/package_smoke.json \
    artifacts/package_smoke/package_smoke_report.md \
    artifacts/package_smoke/environment_audit/environment_audit.json \
    artifacts/package_smoke/environment_audit/environment_audit_report.md \
    artifacts/package/oathyard-linux-x86_64.tar \
    artifacts/package/oathyard-linux-x86_64.tar.sha256 \
    artifacts/package/oathyard-linux-x86_64/package_checksums.sha256 \
    artifacts/package_repro/verify/package_repro_report.md; do
    if [[ -f "$path" ]]; then
      sha256sum "$path"
    fi
  done
} > "$run_dir/hashes.sha256"

cat > "$run_dir/remaining_external_blockers.md" <<'BLOCKERS'
# Remaining External / Human Gates

These remain false even when the local publishable package gate passes:

- owner-final acceptance
- public demo readiness
- release-candidate readiness
- legal clearance
- trademark clearance
- store readiness
- Steam/itch account, forms, review, upload, pricing, age-rating, and release controls
- physical controller ergonomics and Steam Deck compliance
- final loudness/platform audio certification and owner audio acceptance
- external DCC/Khronos glTF validation unless separately run
BLOCKERS

status="FAILED"
if [[ "$verify_rc" -eq 0 && "$visual_rc" -eq 0 ]]; then
  status="PASSED_LOCAL_PACKAGE_CANDIDATE"
fi

cat > "$run_dir/summary.md" <<SUMMARY
# OATHYARD Publishable Gate Summary

Status: $status

- Started UTC: \`$stamp\`
- Verify rc: \`$verify_rc\`
- Visual evidence rc: \`$visual_rc\`
- Verify log: \`$run_dir/verify.log\`
- Visual evidence log: \`$run_dir/visual_evidence.log\`
- Visual evidence report: \`$run_dir/visual_evidence/visual_evidence_report.md\`
- Visual evidence contact sheet: \`$run_dir/visual_evidence/visual_evidence_contact_sheet.svg\`
- Failed visual artifact reduction: \`$run_dir/visual_evidence/failed_visual_artifacts.txt\`
- Hash manifest: \`$run_dir/hashes.sha256\`
- Git status: \`$run_dir/git_status.txt\`
- Final state hash: \`$run_dir/final_state_hash.txt\`
- Final acceptance report: \`$run_dir/final_acceptance_report.md\`
- Remaining blockers: \`$run_dir/remaining_external_blockers.md\`

This gate can only produce a local package-candidate result. It does not claim owner-final acceptance, public demo readiness, release-candidate readiness, legal clearance, trademark clearance, or store readiness.
SUMMARY

cat "$run_dir/summary.md"
if [[ "$verify_rc" -ne 0 ]]; then
  exit "$verify_rc"
fi
exit "$visual_rc"
