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
run_step visual_gap_audit ./tools/visual_gap_audit.sh "$out/visual_gap"
run_step capture_high_fidelity_screens ./tools/capture_high_fidelity_screens.sh "$out/high_fidelity_screens"
run_step visual_benchmark ./tools/visual_benchmark.sh "$out/visual_review"

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
manifest={'schema':'oathyard.final_acceptance.v2','tool':'tools/final_acceptance.sh','passed':passed,'production_renderer_complete':False,'owner_visual_acceptance':False,'public_demo_ready':False,'release_candidate_ready':False,'step_count':len(rows),'failed_step_count':len(failed),'steps':rows}
(out/'final_acceptance_manifest.json').write_text(json.dumps(manifest, indent=2, sort_keys=True)+'\n', encoding='utf-8')
report=['# OATHYARD Final Acceptance Gate','',f"Status: {'PASSED' if passed else 'FAILED'}",'', 'Readiness boundary:', '', '- Production renderer complete: `false` unless current production renderer evidence proves it.', '- Owner visual acceptance: `false` until owner explicitly accepts.', '- Public demo ready: `false` until owner/legal/store/demo-scope gates pass.', '- Release candidate ready: `false` until local, clean-release, owner, legal/trademark/license, and store gates pass.', '', '## Step results','', '| Step | RC | Log |','| --- | ---: | --- |']
for row in rows: report.append(f"| `{row['step']}` | `{row['rc']}` | `{row['log']}` |")
if failed:
    report.extend(['','## Failed steps','']+[f"- `{r['step']}` rc `{r['rc']}` log `{r['log']}`" for r in failed])
(out/'final_acceptance_report.md').write_text('\n'.join(report)+'\n', encoding='utf-8')
if failed: raise SystemExit(1)
PY
rc=$?
echo "final acceptance: $out rc=$rc"
exit "$rc"
