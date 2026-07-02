#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/visual_review/latest}"
mkdir -p "$out"

python3 - "$out" <<'PY'
import json
import sys
from pathlib import Path
out = Path(sys.argv[1])
root = Path.cwd()
required = {
    'renderer_manifest': root / 'artifacts/production_renderer/latest/production_renderer_manifest.json',
    'capture_manifest': root / 'artifacts/high_fidelity_screens/latest/high_fidelity_screen_manifest.json',
    'production_asset_manifest': root / 'assets/manifests/production_visual_manifest.json',
}
failures = []
for key, path in required.items():
    if not path.is_file():
        failures.append(f'missing production evidence file: {path}')
        continue
    try:
        data = json.loads(path.read_text(encoding='utf-8'))
    except Exception as exc:
        failures.append(f'{path} is not valid JSON: {exc}')
        continue
    if data.get('truth_mutation') is not False:
        failures.append(f'{path} does not prove truth_mutation false')
    if data.get('production_renderer_complete') is not True and key != 'capture_manifest':
        failures.append(f'{path} does not prove production_renderer_complete true')
passed = not failures
manifest = {
    'schema': 'oathyard.visual_gap_audit.v2',
    'tool': 'tools/visual_gap_audit.sh',
    'passed': passed,
    'current_fidelity_tier': 'blocked_pending_native_3d_renderer_capture' if failures else 'native_3d_renderer_evidence_present',
    'native_3d_visual_evidence_required': True,
    'production_renderer_complete': False,
    'visual_qa_integrated': True,
    'owner_visual_acceptance': False,
    'public_demo_ready': False,
    'release_candidate_ready': False,
    'truth_mutation': False,
    'failed_check_count': len(failures),
    'failures': failures,
}
(out / 'visual_gap_audit.json').write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n', encoding='utf-8')
(out / 'failed_visual_gap_checks.txt').write_text('none\n' if passed else '\n'.join(failures) + '\n', encoding='utf-8')
report = ['# OATHYARD Visual Gap Audit', '', f"Status: {'PASSED' if passed else 'BLOCKED_PENDING_NATIVE_3D_RENDERER_CAPTURE'}", '', '- Required evidence: native 3D renderer captures only.', '- Standalone generated visual substitutes: `forbidden`', '- Production renderer complete: `false`', '- Owner visual acceptance: `false`', '- Public demo ready: `false`', '- Release candidate ready: `false`']
if failures:
    report.extend(['', '## Failures'] + [f'- {f}' for f in failures])
(out / 'visual_gap_audit_report.md').write_text('\n'.join(report) + '\n', encoding='utf-8')
(out / 'visual_gap_list.md').write_text('\n'.join(report) + '\n', encoding='utf-8')
if not passed:
    raise SystemExit(1)
PY
