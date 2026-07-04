#!/usr/bin/env bash
set -uo pipefail

out="${1:-artifacts/final_acceptance/latest}"
mkdir -p "$out/logs"
summary_tsv="$out/final_acceptance_steps.tsv"
: > "$summary_tsv"
echo -e "step\trc\tlog" >> "$summary_tsv"

run_step() {
  local name="$1"; shift
  local log="$out/logs/${name}.log"
  echo "[final_acceptance] $name: $*"
  "$@" > "$log" 2>&1
  local rc=$?
  echo -e "${name}\t${rc}\t${log}" >> "$summary_tsv"
  echo "$rc" > "$out/logs/${name}.rc"
  return 0
}

run_step research_audit ./tools/research_audit.sh "$out/research_audit"
run_step build ./tools/build.sh
run_step test ./tools/test.sh
run_step truth_audit ./tools/audit_truth.sh
run_step deterministic_duel_a ./tools/run_duel.sh examples/duels/basic_oathyard.duel --out "$out/verify_a"
run_step deterministic_duel_b ./tools/run_duel.sh examples/duels/basic_oathyard.duel --out "$out/verify_b"
run_step replay_verify ./tools/replay_verify.sh "$out/verify_a/replay.json"
run_step match_sweep ./tools/run_match_sweep.sh "$out/match_sweep"
run_step audit_generated_assets ./tools/audit_generated_assets.sh "$out/asset_audit"
run_step build_assets ./tools/build_assets.sh
run_step validate_assets ./tools/validate_assets.sh "$out/assets"
run_step render_asset_previews ./tools/render_asset_previews.sh "$out/asset_previews"
run_step ai_planner_audit ./tools/ai_planner_audit.sh "$out/ai_planner_audit"
run_step sim_reference_compare ./tools/sim_reference_compare.sh "$out/sim_reference_compare"
run_step presentation_truth_isolation ./tools/presentation_truth_isolation.sh examples/duels/basic_oathyard.duel "$out/presentation_truth_isolation"
run_step asset_provenance_audit ./tools/asset_provenance_audit.sh "$out/asset_provenance_audit"
run_step cross_platform_verify ./tools/cross_platform_verify.sh --out "$out/cross_platform_verify"
run_step performance_benchmark ./tools/perf_benchmark.sh "$out/perf"
run_step package ./tools/package.sh
run_step package_smoke ./tools/smoke_package.sh artifacts/package/oathyard-linux-x86_64.tar
run_step working_game_smoke ./tools/working_game_smoke.sh "$out/working_game_smoke"
run_step runtime_asset_sets ./tools/render_runtime_asset_sets.sh examples/duels/basic_oathyard.duel "$out/runtime_asset_sets"
run_step native_asset_capture_matrix ./tools/capture_native_asset_matrix.sh "$out/native_asset_capture_matrix"
run_step high_fidelity_capture_matrix ./tools/render_high_fidelity_capture_matrix.sh examples/duels/basic_oathyard.duel "$out/high_fidelity_capture_matrix"
run_step capture_high_fidelity_screens env OATHYARD_PRODUCTION_RENDERER_MANIFEST="$out/runtime_asset_sets/production_renderer_manifest.json" OATHYARD_UNIT082_CAPTURE_MATRIX_MANIFEST="$out/high_fidelity_capture_matrix/high_fidelity_capture_matrix_manifest.json" ./tools/capture_high_fidelity_screens.sh "$out/high_fidelity_screens"
run_step post_native_asset_audit env OATHYARD_UNIT083_NATIVE_ASSET_CAPTURE_MATRIX="$out/native_asset_capture_matrix/native_asset_capture_matrix_manifest.json" ./tools/audit_generated_assets.sh "$out/asset_audit_post_native"
run_step visual_gap_audit env OATHYARD_PRODUCTION_RENDERER_MANIFEST="$out/runtime_asset_sets/production_renderer_manifest.json" OATHYARD_UNIT082_CAPTURE_MATRIX_MANIFEST="$out/high_fidelity_capture_matrix/high_fidelity_capture_matrix_manifest.json" OATHYARD_UNIT083_NATIVE_ASSET_CAPTURE_MATRIX="$out/native_asset_capture_matrix/native_asset_capture_matrix_manifest.json" ./tools/visual_gap_audit.sh "$out/visual_gap"
run_step visual_qa env OATHYARD_PRODUCTION_RENDERER_MANIFEST="$out/runtime_asset_sets/production_renderer_manifest.json" OATHYARD_UNIT082_CAPTURE_MATRIX_MANIFEST="$out/high_fidelity_capture_matrix/high_fidelity_capture_matrix_manifest.json" OATHYARD_UNIT083_NATIVE_ASSET_CAPTURE_MATRIX="$out/native_asset_capture_matrix/native_asset_capture_matrix_manifest.json" ./tools/visual_qa.sh "$out/visual_qa"
run_step visual_benchmark env OATHYARD_PRODUCTION_RENDERER_MANIFEST="$out/runtime_asset_sets/production_renderer_manifest.json" OATHYARD_UNIT082_CAPTURE_MATRIX_MANIFEST="$out/high_fidelity_capture_matrix/high_fidelity_capture_matrix_manifest.json" OATHYARD_UNIT083_NATIVE_ASSET_CAPTURE_MATRIX="$out/native_asset_capture_matrix/native_asset_capture_matrix_manifest.json" ./tools/visual_benchmark.sh "$out/visual_review"

python3 - "$out" "$summary_tsv" <<'PY'
import json, sys
from pathlib import Path
out = Path(sys.argv[1]); summary_tsv = Path(sys.argv[2])
rows=[]
for line in summary_tsv.read_text(encoding='utf-8').splitlines()[1:]:
    if line.strip():
        name, rc, log = line.split('\t', 2); rows.append({'step': name, 'rc': int(rc), 'log': log})
failed=[r for r in rows if r['rc'] != 0]
passed=not failed
report=['# OATHYARD Final Acceptance Gate','',f"Status: {'PASSED' if passed else 'FAILED'}",'', 'Readiness boundary:', '', '- Production renderer complete: `false` unless current production renderer evidence proves it.', '- Owner visual acceptance: `false` until owner explicitly accepts.', '- Public demo ready: `false` until owner/legal/store/demo-scope gates pass.', '- Release candidate ready: `false` until local, clean-release, owner, legal/trademark/license, and store gates pass.', '', '## Step results','', '| Step | RC | Log |','| --- | ---: | --- |']
for row in rows: report.append(f"| `{row['step']}` | `{row['rc']}` | `{row['log']}` |")
if failed:
    report.extend(['','## Failed steps','']+[f"- `{r['step']}` rc `{r['rc']}` log `{r['log']}`" for r in failed])
(out/'final_acceptance_report.md').write_text('\n'.join(report)+'\n', encoding='utf-8')
artifact_specs = [
    ('final_acceptance_steps', summary_tsv),
    ('final_acceptance_report', out / 'final_acceptance_report.md'),
    ('deterministic_duel_a_replay', out / 'verify_a/replay.json'),
    ('deterministic_duel_a_final_hash', out / 'verify_a/final_state_hash.txt'),
    ('deterministic_duel_b_replay', out / 'verify_b/replay.json'),
    ('replay_verify_log', out / 'logs/replay_verify.log'),
    ('match_sweep_summary_json', out / 'match_sweep/match_sweep_summary.json'),
    ('match_sweep_summary_md', out / 'match_sweep/match_sweep_summary.md'),
    ('generated_asset_audit_json', out / 'asset_audit/generated_asset_audit.json'),
    ('generated_asset_audit_md', out / 'asset_audit/generated_asset_audit.md'),
    ('generated_asset_quarantine_manifest_json', out / 'asset_audit/generated_asset_quarantine_manifest.json'),
    ('generated_asset_quarantine_report_md', out / 'asset_audit/generated_asset_quarantine_report.md'),
    ('generated_asset_production_unblock_matrix_json', out / 'asset_audit/generated_asset_production_unblock_matrix.json'),
    ('generated_asset_production_unblock_matrix_md', out / 'asset_audit/generated_asset_production_unblock_matrix.md'),
    ('unit083_native_asset_capture_matrix_manifest', out / 'native_asset_capture_matrix/native_asset_capture_matrix_manifest.json'),
    ('unit083_native_asset_capture_matrix_report', out / 'native_asset_capture_matrix/native_asset_capture_matrix_report.md'),
    ('working_game_smoke_manifest', out / 'working_game_smoke/game_flow_manifest.json'),
    ('working_game_local_asset_consumption_manifest', out / 'working_game_smoke/local_game_asset_consumption_manifest.json'),
    ('working_game_local_asset_consumption_png', out / 'working_game_smoke/native_asset_runtime/render/production_renderer_unit083_local_game_generated_asset_consumption_1920x1080.png'),
    ('post_native_generated_asset_audit_json', out / 'asset_audit_post_native/generated_asset_audit.json'),
    ('post_native_generated_asset_unblock_matrix_json', out / 'asset_audit_post_native/generated_asset_production_unblock_matrix.json'),
    ('unit082_high_fidelity_capture_matrix_manifest', out / 'high_fidelity_capture_matrix/high_fidelity_capture_matrix_manifest.json'),
    ('unit082_high_fidelity_capture_slot_table', out / 'high_fidelity_capture_matrix/high_fidelity_capture_slot_table.md'),
    ('high_fidelity_capture_matrix_json', out / 'high_fidelity_screens/high_fidelity_capture_matrix.json'),
    ('high_fidelity_capture_matrix_md', out / 'high_fidelity_screens/high_fidelity_capture_matrix.md'),
    ('runtime_asset_sets_manifest', out / 'runtime_asset_sets/runtime_asset_sets_render_manifest.json'),
    ('runtime_asset_sets_renderer_manifest', out / 'runtime_asset_sets/production_renderer_manifest.json'),
    ('visual_benchmark_report', out / 'visual_review/visual_benchmark_report.md'),
    ('visual_qa_report', out / 'visual_qa/visual_qa_report.json'),
    ('visual_gap_list', out / 'visual_gap/visual_gap_list.md'),
    ('package_tar', Path('artifacts/package/oathyard-linux-x86_64.tar')),
]
artifact_index = [
    {'id': artifact_id, 'path': path.as_posix(), 'exists': path.is_file()}
    for artifact_id, path in artifact_specs
]
artifact_index_missing_count = sum(1 for artifact in artifact_index if not artifact['exists'])
manifest={'schema':'oathyard.final_acceptance.v2','tool':'tools/final_acceptance.sh','passed':passed,'production_renderer_complete':False,'owner_visual_acceptance':False,'public_demo_ready':False,'release_candidate_ready':False,'step_count':len(rows),'failed_step_count':len(failed),'steps':rows,'artifact_index':artifact_index,'artifact_index_missing_count':artifact_index_missing_count}
(out/'final_acceptance_manifest.json').write_text(json.dumps(manifest, indent=2, sort_keys=True)+'\n', encoding='utf-8')
report.extend(['','## Evidence artifact index','', '| Artifact | Exists | Path |','| --- | ---: | --- |'])
for artifact in artifact_index:
    report.append(f"| `{artifact['id']}` | `{str(artifact['exists']).lower()}` | `{artifact['path']}` |")
(out/'final_acceptance_report.md').write_text('\n'.join(report)+'\n', encoding='utf-8')
if failed: raise SystemExit(1)
PY
rc=$?
echo "final acceptance: $out rc=$rc"
exit "$rc"
