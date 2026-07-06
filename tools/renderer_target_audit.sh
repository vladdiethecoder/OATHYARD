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

schema = manifest.get('schema', '')
evidence_present = manifest.get('native_3d_visual_evidence_present', False)

if manifest:
    # Accept either promoted or blocked schema.
    valid_schema = schema in ('oathyard.native_combat_render.v1', 'oathyard.native_3d_visual_blocked.v1')
    check('native_status_schema', valid_schema, schema)
    check('native_status_truth_read_only', manifest.get('truth_mutation') is False, manifest.get('truth_mutation'))
    check('native_status_forbidden_fallbacks_absent', manifest.get('forbidden_visual_fallbacks_emitted') is False, manifest.get('forbidden_visual_fallbacks_emitted'))
    check('native_status_source_after_hash', manifest.get('source') == 'truth-after-hash-duel-result', manifest.get('source'))
    check('readiness_flags_false', manifest.get('public_demo_ready') is False and manifest.get('release_candidate_ready') is False and manifest.get('owner_visual_acceptance') is False, 'external readiness flags')

    if evidence_present:
        check('native_status_visual_evidence_promoted', evidence_present is True, evidence_present)
        check('native_status_capture_status', manifest.get('visual_evidence_status') == 'native_3d_renderer_capture_present', manifest.get('visual_evidence_status'))
        # Verify real capture PNG exists
        render_dir = root / 'artifacts/native_combat/verify/render'
        capture_pngs = list(render_dir.glob('production_renderer_*.png')) if render_dir.is_dir() else []
        if not capture_pngs:
            render_dir = root / 'artifacts/native_combat/latest/render'
            capture_pngs = list(render_dir.glob('production_renderer_*.png')) if render_dir.is_dir() else []
        check('native_status_capture_file_present', len(capture_pngs) > 0, f'{len(capture_pngs)} capture(s)')
        if capture_pngs:
            size = capture_pngs[0].stat().st_size
            check('native_status_capture_nontrivial_size', size > 50000, f'{size} bytes')

        # Unit-071: Require mesh_geometry_consumed=true for promoted combat evidence.
        # The production renderer manifest must prove actual rigged mesh consumption.
        prod_manifest_path = render_dir / 'production_renderer_manifest.json'
        if prod_manifest_path.is_file():
            try:
                prod = json.loads(prod_manifest_path.read_text(encoding='utf-8'))
                mesh_consumed = prod.get('mesh_geometry_consumed', False)
                mesh_count = prod.get('mesh_asset_count', 0)
                mesh_summary = prod.get('mesh_summary') or {}
                mesh_assets = prod.get('mesh_assets', []) if isinstance(prod.get('mesh_assets', []), list) else []
                vertex_count = mesh_summary.get('vertex_count', 0) if isinstance(mesh_summary, dict) else 0
                if mesh_assets:
                    vertex_count = sum(int(m.get('vertex_count', 0) or 0) for m in mesh_assets)
                check('native_status_mesh_geometry_consumed', mesh_consumed is True, f'mesh_geometry_consumed={mesh_consumed}')
                check('native_status_mesh_asset_count_positive', mesh_count > 0, f'mesh_asset_count={mesh_count}')
                check('native_status_mesh_vertex_count_substantial', vertex_count >= 1000, f'vertex_count={vertex_count}')
                mesh_ids = [str(m.get('mesh_asset_id', '?')) for m in mesh_assets]
                duelist_consumed = any(('saltreach' in mid) or ('duelist' in mid) for mid in mesh_ids)
                check('native_status_saltreach_duelist_consumed', duelist_consumed, f'mesh_ids={mesh_ids}')
            except Exception as exc:
                check('native_status_prod_manifest_parse', False, str(exc))
        else:
            check('native_status_prod_manifest_present', False, str(prod_manifest_path))
    else:
        check('native_status_visual_evidence_blocked', evidence_present is False, evidence_present)

failed = [item for item in checks if not item['passed']]

if evidence_present:
    current_status = 'native_3d_renderer_capture_present'
else:
    current_status = 'blocked_pending_native_3d_renderer_capture'

payload = {
    'schema': 'oathyard.native_presentation_target.v3',
    'tool': 'tools/renderer_target_audit.sh',
    'passed': not failed,
    'current_status': current_status,
    'source_manifest': source.as_posix(),
    'native_3d_visual_evidence_required': True,
    'native_3d_visual_evidence_present': evidence_present,
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

status_line = 'PASSED' if not failed else 'FAILED'
report = [
    '# OATHYARD Native Presentation Target Audit', '',
    f'Status: {status_line}', '',
    '- Native 3D visual evidence required: `true`',
    f'- Current visual status: `{current_status}`',
    f'- Native 3D visual evidence present: `{evidence_present}`',
    '- Production renderer complete: `false`',
    '- Owner visual acceptance: `false`',
    '- Public demo ready: `false`',
    '- Release candidate ready: `false`',
    '- Truth mutation: `false`', '',
    '## Checks'
]
for item in checks:
    report.append(f"- {'PASS' if item['passed'] else 'FAIL'} `{item['id']}`: {item['detail']}")
(out / 'native_presentation_target_report.md').write_text('\n'.join(report) + '\n', encoding='utf-8')
if failed:
    raise SystemExit(1)
PY

echo "renderer target audit passed: $out"
