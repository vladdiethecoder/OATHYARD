#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/visual_artifact_audit/latest}"
mkdir -p "$out"

python3 - "$out" <<'PY'
import json
import subprocess
import sys
from pathlib import Path
out = Path(sys.argv[1])
violations = []
extensions = ['s'+'vg', 'p'+'pm', 'p'+'bm', 'p'+'gm', 'x'+'pm']
terms = [
    'time'+'line.'+'s'+'vg',
    '<'+'s'+'vg',
    'fight_film_contact_'+'sheet',
    'contact '+'sheet',
    'contact-'+'sheet',
    'contact_'+'sheet',
    'proof '+'sheet',
    'visual '+'composite',
    'visual-'+'composite',
    'raw '+'frame',
    'raw-'+'fra'+'me',
    'legacy '+'raw-'+'fra'+'me',
    'legacy '+'vector-diagram',
    'legacy '+'local capture',
    'legacy '+'local raster',
    'software-legacy '+'raw-'+'fra'+'me',
    'native-software-'+'p'+'pm',
    'p'+'pm_capture',
    'time'+'line_capture',
    'headless '+'visual',
    'software '+'render',
    'software '+'fallback',
    'raw '+'x11',
    'raw-x11-'+'p'+'pm',
    'browser/'+'canvas',
]
exclude_roots = {'.git', 'target'}
tracked = subprocess.run(['git', 'ls-files'], check=True, stdout=subprocess.PIPE, text=True).stdout.splitlines()
for rel in tracked:
    parts = set(Path(rel).parts)
    if parts & exclude_roots:
        continue
    suffix = Path(rel).suffix.lower().lstrip('.')
    if suffix in extensions:
        violations.append(f'tracked forbidden visual artifact file: {rel}')
    try:
        text = Path(rel).read_text(encoding='utf-8', errors='replace')
    except Exception:
        continue
    for line_no, line in enumerate(text.splitlines(), 1):
        lower = line.lower()
        for term in terms:
            if term in lower:
                violations.append(f'tracked forbidden visual reference: {rel}:{line_no}:{line}')

generated_roots = [
    Path('artifacts/verify_a'),
    Path('artifacts/verify_b'),
    Path('artifacts/export_bundle/verify'),
    Path('artifacts/native_combat/verify'),
    Path('artifacts/pbr_materials/verify'),
]
for root in generated_roots:
    if not root.exists():
        continue
    for path in sorted(p for p in root.rglob('*') if p.is_file()):
        suffix = path.suffix.lower().lstrip('.')
        if suffix in extensions:
            violations.append(f'generated forbidden visual artifact file: {path.as_posix()}')
        try:
            text = path.read_text(encoding='utf-8', errors='replace')
        except Exception:
            continue
        for line_no, line in enumerate(text.splitlines(), 1):
            lower = line.lower()
            for term in terms:
                if term in lower:
                    violations.append(f'generated forbidden visual reference: {path.as_posix()}:{line_no}:{line}')

payload = {
    'schema': 'oathyard.visual_artifact_audit.v1',
    'tool': 'tools/audit_visual_artifacts.sh',
    'passed': not violations,
    'forbidden_visual_artifact_count': len(violations),
    'native_3d_visual_evidence_required': True,
    'truth_mutation': False,
    'violations': violations,
}
(out / 'visual_artifact_audit.json').write_text(json.dumps(payload, indent=2, sort_keys=True) + '\n', encoding='utf-8')
report = ['# OATHYARD Visual Artifact Audit', '', f"Status: {'PASSED' if not violations else 'FAILED'}", '', f"- Forbidden artifact/reference count: `{len(violations)}`", '- Native 3D visual evidence policy: `required`', '- Truth mutation: `false`']
if violations:
    report.extend(['', '## Violations'] + [f'- {v}' for v in violations])
(out / 'visual_artifact_audit_report.md').write_text('\n'.join(report) + '\n', encoding='utf-8')
if violations:
    print('\n'.join(report), file=sys.stderr)
    raise SystemExit(1)
print(f'visual artifact audit passed: {out}')
PY
