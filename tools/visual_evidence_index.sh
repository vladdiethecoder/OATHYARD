#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/visual_evidence/verify}"
mkdir -p "$out"

python3 - "$out" <<'PY'
import json
import sys
from pathlib import Path
out = Path(sys.argv[1])
manifest_path = Path('artifacts/native_combat/verify/native_combat_render_manifest.json')
items = []
if manifest_path.is_file():
    data = json.loads(manifest_path.read_text(encoding='utf-8'))
    items.append({
        'id': 'native_combat_manifest',
        'path': manifest_path.as_posix(),
        'schema': data.get('schema'),
        'native_3d_visual_evidence_present': data.get('native_3d_visual_evidence_present') is True,
        'truth_mutation': data.get('truth_mutation'),
    })
passed = all(item['native_3d_visual_evidence_present'] and item['truth_mutation'] is False for item in items)
manifest = {
    'schema': 'oathyard.visual_evidence_index.v2',
    'tool': 'tools/visual_evidence_index.sh',
    'passed': passed,
    'visual_evidence_status': 'requires_native_3d_renderer_capture',
    'forbidden_visual_fallbacks_allowed': False,
    'truth_mutation': False,
    'items': items,
}
(out / 'visual_evidence_manifest.json').write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n', encoding='utf-8')
report = ['# OATHYARD Visual Evidence Index', '', f"Status: {'PASSED' if passed else 'BLOCKED_PENDING_NATIVE_3D_RENDERER_CAPTURE'}", '', '- Required visual evidence: native 3D renderer capture with manifest-backed camera/asset metadata.', '- Forbidden fallback output allowed: `false`', '- Truth mutation: `false`', '', '## Items']
for item in items:
    report.append(f"- `{item['id']}` path `{item['path']}` schema `{item['schema']}` native_3d `{item['native_3d_visual_evidence_present']}`")
(out / 'visual_evidence_report.md').write_text('\n'.join(report) + '\n', encoding='utf-8')
(out / 'failed_visual_artifacts.txt').write_text('none\n' if passed else 'blocked_pending_native_3d_renderer_capture\n', encoding='utf-8')
if not passed:
    raise SystemExit(1)
PY

echo "visual evidence index: $out"
