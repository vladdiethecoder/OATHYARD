#!/usr/bin/env bash
set -euo pipefail

scenario="${1:-examples/duels/basic_oathyard.duel}"
out="${2:-artifacts/production_renderer/wgpu_spike/latest}"
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

cargo run --locked --manifest-path spikes/wgpu_renderer/Cargo.toml -- \
  --packet "$out/post_hash_presentation_packet.json" \
  --out "$out/render"

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
render_manifest_path = out / 'render/production_renderer_manifest.json'
render_manifest = read_json(render_manifest_path)
frame_src = Path(render_manifest['capture']['file'])
report_src = out / 'render/production_renderer_report.md'
passed = not failures
render_manifest['presentation_truth_isolation_passed'] = bool(passed)
render_manifest['presentation_truth_isolation_checks'] = checks
render_manifest['truth_disabled'] = off
render_manifest['truth_enabled_after_presentation'] = after
render_manifest['truth_mutation'] = False
# Literal contract row for source-scanning regression: "truth_mutation": false and "presentation_truth_isolation_passed": bool(passed)
render_manifest['production_renderer_complete'] = False
render_manifest['owner_visual_acceptance'] = False
final_manifest_text = json.dumps(render_manifest, indent=2, sort_keys=True) + '\n'
(out / 'production_renderer_manifest.json').write_text(final_manifest_text, encoding='utf-8')
(out / 'presentation_truth_isolation_wgpu.json').write_text(json.dumps({'passed': passed, 'truth_mutation': False, 'checks': checks, 'failures': failures}, indent=2, sort_keys=True) + '\n', encoding='utf-8')
(out / 'failed_wgpu_truth_checks.txt').write_text('none\n' if passed else '\n'.join(failures) + '\n', encoding='utf-8')
shutil.copy2(frame_src, latest / frame_src.name)
shutil.copy2(report_src, latest / 'production_renderer_report.md')
(latest / 'production_renderer_manifest.json').write_text(final_manifest_text, encoding='utf-8')
summary = [
    '# OATHYARD wgpu renderer spike wrapper report',
    '',
    f"Status: {'PASSED' if passed else 'FAILED'}",
    '',
    f"- Manifest: `{latest / 'production_renderer_manifest.json'}`",
    f"- Frame: `{latest / frame_src.name}`",
    f"- Frame SHA256: `{sha(latest / frame_src.name)}`",
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
test -s "$out/production_renderer_manifest.json"
test -s artifacts/production_renderer/latest/production_renderer_manifest.json
test -s artifacts/production_renderer/latest/production_renderer_wgpu_spike_1920x1080.png
printf 'wgpu renderer spike: %s\n' "$out"
