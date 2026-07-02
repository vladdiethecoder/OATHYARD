#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/renderer_target/latest}"
mkdir -p "$out"

python3 - "$out" <<'PY'
import json
import sys
from pathlib import Path
out = Path(sys.argv[1])
root = Path.cwd()
source = root / 'artifacts/native_combat/verify/native_combat_render_manifest.json'
if not source.is_file():
    source = root / 'artifacts/native_combat/latest/native_combat_render_manifest.json'
checks = []

def check(name, passed, detail):
    checks.append({'id': name, 'passed': bool(passed), 'detail': str(detail)})

manifest = {}
if source.is_file():
    try:
        manifest = json.loads(source.read_text(encoding='utf-8'))
    except Exception as exc:
        check('native_status_manifest_parse', False, exc)
else:
    check('native_status_manifest_present', False, source)

if manifest:
    check('native_status_schema', manifest.get('schema') == 'oathyard.native_3d_visual_blocked.v1', manifest.get('schema'))
    check('native_status_truth_read_only', manifest.get('truth_mutation') is False, manifest.get('truth_mutation'))
    check('native_status_forbidden_fallbacks_absent', manifest.get('forbidden_visual_fallbacks_emitted') is False, manifest.get('forbidden_visual_fallbacks_emitted'))
    check('native_status_visual_evidence_blocked', manifest.get('native_3d_visual_evidence_present') is False, manifest.get('native_3d_visual_evidence_present'))
    check('readiness_flags_false', manifest.get('public_demo_ready') is False and manifest.get('release_candidate_ready') is False and manifest.get('owner_visual_acceptance') is False, 'external readiness flags')

failed = [item for item in checks if not item['passed']]
payload = {
    'schema': 'oathyard.native_presentation_target.v3',
    'tool': 'tools/renderer_target_audit.sh',
    'passed': not failed,
    'current_status': 'blocked_pending_native_3d_renderer_capture' if not failed else 'failed',
    'source_manifest': source.as_posix(),
    'native_3d_visual_evidence_required': True,
    'native_3d_visual_evidence_present': False,
    'production_renderer_complete': False,
    'owner_visual_acceptance': False,
    'public_demo_ready': False,
    'release_candidate_ready': False,
    'truth_mutation': False,
    'failed_check_count': len(failed),
    'check_count': len(checks),
    'checks': checks,
}
(out / 'native_presentation_target.json').write_text(json.dumps(payload, indent=2, sort_keys=True) + '\n', encoding='utf-8')
report = ['# OATHYARD Native Presentation Target Audit', '', f"Status: {'PASSED' if not failed else 'FAILED'}", '', '- Native 3D visual evidence required: `true`', '- Current visual status: `blocked_pending_native_3d_renderer_capture`', '- Production renderer complete: `false`', '- Owner visual acceptance: `false`', '- Public demo ready: `false`', '- Release candidate ready: `false`', '- Truth mutation: `false`', '', '## Checks']
for item in checks:
    report.append(f"- {'PASS' if item['passed'] else 'FAIL'} `{item['id']}`: {item['detail']}")
(out / 'native_presentation_target_report.md').write_text('\n'.join(report) + '\n', encoding='utf-8')
if failed:
    raise SystemExit(1)
PY

echo "renderer target audit passed: $out"
