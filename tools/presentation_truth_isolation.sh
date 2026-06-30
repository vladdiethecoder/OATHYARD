#!/usr/bin/env bash
set -euo pipefail

scenario="${1:-examples/duels/basic_oathyard.duel}"
out="${2:-artifacts/presentation_truth_isolation/latest}"
mkdir -p "$out" "$out/presentation/native_combat" "$out/presentation/presentation_bricks" "$out/presentation/wgpu_renderer_spike"

run_log() { local log="$1"; shift; "$@" > "$log" 2>&1; }
run_log "$out/truth_presentation_disabled.log" ./tools/run_duel.sh "$scenario" --out "$out/truth_presentation_disabled"
run_log "$out/native_combat_presentation.log" ./tools/native_combat_render.sh "$scenario" "$out/presentation/native_combat"
run_log "$out/presentation_bricks.log" cargo run --locked -- presentation-bricks --scenario "$scenario" --out "$out/presentation/presentation_bricks"
if [[ "${OATHYARD_SKIP_WGPU_RENDERER_SPIKE:-0}" != "1" ]]; then
  run_log "$out/wgpu_renderer_spike.log" ./tools/wgpu_renderer_spike.sh "$scenario" "$out/presentation/wgpu_renderer_spike"
fi
run_log "$out/truth_presentation_enabled_after.log" ./tools/run_duel.sh "$scenario" --out "$out/truth_presentation_enabled_after"

python3 - "$out" "$scenario" <<'PY'
import hashlib, json, sys
from pathlib import Path
out = Path(sys.argv[1]); scenario = sys.argv[2]

def sha(path: Path) -> str:
    h=hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(65536), b''): h.update(chunk)
    return h.hexdigest()

def read_json(path: Path): return json.loads(path.read_text(encoding='utf-8'))
def pick_trace(data): return {'final_state_hash':data.get('final_state_hash'),'content_hash':data.get('content_hash'),'end_condition':data.get('end_condition'),'turns':data.get('turns')}
def collect(path: Path):
    trace_path=path/'trace.json'; replay_path=path/'replay.json'; final_path=path/'final_state_hash.txt'; report_path=path/'duel_report.md'
    trace=read_json(trace_path); replay=read_json(replay_path)
    return {'trace_sha256':sha(trace_path),'replay_sha256':sha(replay_path),'report_sha256':sha(report_path) if report_path.is_file() else '', 'final_hash_text':final_path.read_text(encoding='utf-8').strip(), 'trace_core':pick_trace(trace), 'replay_final_hash':replay.get('final_state_hash'), 'replay_content_hash':replay.get('content_hash'), 'replay_end_condition_status':replay.get('end_condition_status'), 'replay_end_condition_winner':replay.get('end_condition_winner')}

off=collect(out/'truth_presentation_disabled'); after=collect(out/'truth_presentation_enabled_after')
checks=[]; failures=[]
def check(cid, passed, detail):
    checks.append({'id':cid,'passed':bool(passed),'detail':detail})
    if not passed: failures.append(f'{cid}: {detail}')
for field in ['trace_sha256','replay_sha256','final_hash_text','trace_core','replay_final_hash','replay_content_hash','replay_end_condition_status','replay_end_condition_winner']:
    check(f'truth_{field}_stable_with_presentation_enabled_disabled', off[field] == after[field], f'off={off[field]} after={after[field]}')
combat_manifest_path=out/'presentation/native_combat/native_combat_render_manifest.json'
bricks_manifest_path=out/'presentation/presentation_bricks/presentation_bricks_manifest.json'
combat=read_json(combat_manifest_path); bricks=read_json(bricks_manifest_path)
check('native_combat_truth_mutation_false', combat.get('truth_mutation') is False, str(combat.get('truth_mutation')))
check('native_combat_presentation_only', combat.get('presentation_only') is True, str(combat.get('presentation_only')))
check('native_combat_source_after_hash', combat.get('source') == 'truth-after-hash-duel-result', str(combat.get('source')))
check('presentation_bricks_runtime_presentation', bricks.get('layer') == 'runtime_presentation', str(bricks.get('layer')))
check('presentation_bricks_truth_mutation_false', bricks.get('truth_mutation') is False, str(bricks.get('truth_mutation')))
check('presentation_bricks_presentation_only', bricks.get('presentation_only') is True, str(bricks.get('presentation_only')))
# New production presentation toggle requirement: fail closed until a production renderer manifest exists and declares an on/off truth-isolation result.
prod_manifest_path = Path('artifacts/production_renderer/latest/production_renderer_manifest.json')
if prod_manifest_path.is_file():
    prod = read_json(prod_manifest_path)
    check('production_presentation_on_off_hash_identity', prod.get('presentation_truth_isolation_passed') is True and prod.get('truth_mutation') is False, json.dumps({k: prod.get(k) for k in ['presentation_truth_isolation_passed','truth_mutation']}))
else:
    check('production_presentation_on_off_hash_identity', False, f'missing production renderer manifest: {prod_manifest_path}')

passed = not failures
manifest={'schema':'oathyard.presentation_truth_isolation.v3','tool':'tools/presentation_truth_isolation.sh','scenario':scenario,'passed':passed,'truth_disabled':off,'truth_enabled_after_presentation':after,'presentation_tools_exercised':['tools/native_combat_render.sh','oathyard presentation-bricks','tools/wgpu_renderer_spike.sh'],'production_renderer_manifest_required':prod_manifest_path.as_posix(),'truth_mutation':False if passed else None,'failed_check_count':len(failures),'checks':checks}
(out/'presentation_truth_isolation_manifest.json').write_text(json.dumps(manifest, indent=2, sort_keys=True)+'\n', encoding='utf-8')
(out/'failed_presentation_truth_checks.txt').write_text('none\n' if passed else '\n'.join(failures)+'\n', encoding='utf-8')
report=['# OATHYARD Presentation Truth Isolation Report','',f"Status: {'PASSED' if passed else 'FAILED'}",'',f'- Scenario: `{scenario}`',f"- Presentation-disabled final hash: `{off['final_hash_text']}`",f"- Presentation-enabled-after final hash: `{after['final_hash_text']}`",f"- Replay SHA stable: `{off['replay_sha256'] == after['replay_sha256']}`",f"- Trace SHA stable: `{off['trace_sha256'] == after['trace_sha256']}`",'- Production presentation on/off proof required: `artifacts/production_renderer/latest/production_renderer_manifest.json`','- Current debug-local presentation truth isolation is useful but cannot satisfy production renderer isolation by itself.']
if failures: report.extend(['','## Failures']+[f'- {f}' for f in failures])
(out/'presentation_truth_isolation_report.md').write_text('\n'.join(report)+'\n', encoding='utf-8')
if not passed: raise SystemExit(1)
PY

echo "presentation truth isolation: $out"
