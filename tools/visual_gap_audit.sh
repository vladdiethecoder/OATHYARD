#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/visual_review/latest}"
mkdir -p "$out"

python3 - "$out" <<'PY'
import hashlib
import json
import os
import struct
import subprocess
import sys
from pathlib import Path

out = Path(sys.argv[1])
root = Path.cwd()

image_suffixes = {'.ppm', '.png', '.jpg', '.jpeg', '.svg'}
debug_tokens = [
    'native_combat', 'ppm', 'svg', 'raw-x11', 'raw_x11', 'software', 'debug',
    'line-art', 'line_art', 'capsule', 'silhouette', 'primitive', 'low-poly',
    'placeholder', 'roster_showcase', 'contact_sheet', 'visual_evidence'
]
production_required = {
    'renderer_manifest': root / 'artifacts/production_renderer/latest/production_renderer_manifest.json',
    'capture_manifest': root / 'artifacts/high_fidelity_screens/latest/high_fidelity_screen_manifest.json',
    'production_asset_manifest': root / 'assets/production_visual_manifest.json',
    'visual_benchmark_report': root / 'artifacts/visual_review/latest/visual_benchmark_report.md',
}
scan_roots = [root / 'evidence/capture', root / 'artifacts', root / 'assets/previews', root / 'assets/gltf']


def sha(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(65536), b''):
            h.update(chunk)
    return h.hexdigest()


def png_size(path: Path):
    try:
        data = path.read_bytes()[:24]
    except OSError:
        return None
    if data.startswith(b'\x89PNG\r\n\x1a\n') and len(data) >= 24:
        return struct.unpack('>II', data[16:24])
    return None


def ppm_size(path: Path):
    try:
        with path.open('rb') as f:
            magic = f.readline().strip()
            if magic not in {b'P6', b'P3'}:
                return None
            line = f.readline().strip()
            while line.startswith(b'#') or not line:
                line = f.readline().strip()
            w, h = [int(x) for x in line.split()[:2]]
            return w, h
    except Exception:
        return None


def svg_size(path: Path):
    text = path.read_text(encoding='utf-8', errors='ignore')[:1000]
    import re
    wm = re.search(r'width="?([0-9]+)', text)
    hm = re.search(r'height="?([0-9]+)', text)
    if wm and hm:
        return int(wm.group(1)), int(hm.group(1))
    return None

artifacts = []
for base in scan_roots:
    if not base.exists():
        continue
    for p in base.rglob('*'):
        if not p.is_file() or p.suffix.lower() not in image_suffixes:
            continue
        rel = p.relative_to(root).as_posix() if p.is_relative_to(root) else p.as_posix()
        lower = rel.lower()
        size = png_size(p) if p.suffix.lower() == '.png' else ppm_size(p) if p.suffix.lower() == '.ppm' else svg_size(p) if p.suffix.lower() == '.svg' else None
        hits = sorted({t for t in debug_tokens if t in lower})
        artifacts.append({
            'path': rel,
            'suffix': p.suffix.lower(),
            'width': size[0] if size else None,
            'height': size[1] if size else None,
            'sha256': sha(p),
            'debug_marker_hits': hits,
            'debug_or_baseline_evidence': bool(hits) or p.suffix.lower() in {'.ppm', '.svg'},
            'meets_1920x1080': bool(size and size[0] >= 1920 and size[1] >= 1080),
        })

production_files = {k: v.is_file() for k, v in production_required.items()}
failures = []
if not artifacts:
    failures.append('no current visual artifacts found to classify')
if artifacts and all(a['debug_or_baseline_evidence'] for a in artifacts):
    failures.append('best current visual evidence is debug/baseline output (PPM/SVG/software/native_combat/contact-sheet/low-poly)')
for key, present in production_files.items():
    if not present:
        failures.append(f'missing production evidence file: {production_required[key]}')

# If manifests exist, require them to prove production path and stay honest.
for key in ['renderer_manifest', 'capture_manifest', 'production_asset_manifest']:
    path = production_required[key]
    if not path.is_file():
        continue
    try:
        data = json.loads(path.read_text(encoding='utf-8'))
    except Exception as exc:
        failures.append(f'{path} is not valid JSON: {exc}')
        continue
    if data.get('production_renderer_complete') is not True:
        failures.append(f'{path} does not prove production_renderer_complete true')
    if data.get('owner_visual_acceptance') is True:
        failures.append(f'{path} claims owner_visual_acceptance before separate owner gate')
    if key == 'capture_manifest':
        captures = data.get('captures', [])
        if not captures or not all(int(c.get('width', 0)) >= 1920 and int(c.get('height', 0)) >= 1080 for c in captures):
            failures.append(f'{path} lacks complete 1920x1080+ capture list')

# Contact sheet from current failing baseline, for V0 evidence. Use PNG/JPG only to avoid external SVG/PPM conversion surprises.
sheet_candidates = [root / a['path'] for a in artifacts if a['suffix'] in {'.png', '.jpg', '.jpeg'}]
sheet_path = out / 'v0_current_visual_baseline_contact_sheet.png'
if sheet_candidates and not sheet_path.exists():
    cmd = ['montage'] + [str(p) for p in sheet_candidates[:24]] + ['-thumbnail', '320x180', '-background', '#15120f', '-fill', '#f3e8cf', '-label', '%t', '-tile', '3x', '-geometry', '+8+28', str(sheet_path)]
    try:
        subprocess.run(cmd, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    except Exception as exc:
        failures.append(f'could not create baseline contact sheet with montage: {exc}')

passed = not failures
manifest = {
    'schema': 'oathyard.visual_gap_audit.v1',
    'tool': 'tools/visual_gap_audit.sh',
    'passed': passed,
    'current_fidelity_tier': 'Tier 0 / failing baseline' if failures else 'production evidence present',
    'production_renderer_complete': False,
    'owner_visual_acceptance': False,
    'public_demo_ready': False,
    'release_candidate_ready': False,
    'artifact_count': len(artifacts),
    'artifacts_1920x1080_or_larger': sum(1 for a in artifacts if a['meets_1920x1080']),
    'debug_or_baseline_artifact_count': sum(1 for a in artifacts if a['debug_or_baseline_evidence']),
    'production_files_present': production_files,
    'contact_sheet': sheet_path.as_posix() if sheet_path.is_file() else '',
    'failed_check_count': len(failures),
    'failures': failures,
    'artifacts': artifacts[:300],
}
(out / 'visual_gap_audit.json').write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n', encoding='utf-8')
(out / 'failed_visual_gap_checks.txt').write_text('none\n' if passed else '\n'.join(failures) + '\n', encoding='utf-8')
report = [
    '# OATHYARD V0 Baseline Visual Gap Audit', '',
    f"Status: {'PASSED' if passed else 'FAILED'}", '',
    'The current PPM/SVG/raw-X11/software/native_combat/low-poly/primitive evidence is a failing baseline for the production visual target.', '',
    f"- Artifact count inspected: `{len(artifacts)}`",
    f"- 1920x1080+ baseline artifacts found: `{manifest['artifacts_1920x1080_or_larger']}`",
    f"- Debug/baseline-class artifacts: `{manifest['debug_or_baseline_artifact_count']}`",
    f"- Contact sheet: `{manifest['contact_sheet'] or 'not generated'}`",
    '- Production renderer complete: `false`',
    '- Owner visual acceptance: `false`',
    '- Public demo ready: `false`',
    '- Release candidate ready: `false`', '',
    '## Required production evidence files', '',
]
for key, path in production_required.items():
    report.append(f"- `{key}`: `{'present' if production_files[key] else 'missing'}` `{path}`")
if failures:
    report.extend(['', '## Failures', ''])
    report.extend(f'- {f}' for f in failures)
report.extend(['', '## Current artifact sample', ''])
for a in artifacts[:40]:
    report.append(f"- `{a['path']}` `{a['suffix']}` `{a['width']}x{a['height']}` debug=`{a['debug_or_baseline_evidence']}` sha256=`{a['sha256'][:16]}`")
(out / 'visual_gap_audit_report.md').write_text('\n'.join(report) + '\n', encoding='utf-8')
gap_list = [
    '# OATHYARD Visual Gap List',
    '',
    f"Status: {'PASSED' if passed else 'FAILED'}",
    '',
    'The current visual packet remains candidate/debug evidence unless every item below is resolved.',
    '',
]
if failures:
    gap_list.extend(f'- {f}' for f in failures)
else:
    gap_list.append('- none')
(out / 'visual_gap_list.md').write_text('\n'.join(gap_list) + '\n', encoding='utf-8')
if not passed:
    raise SystemExit(1)
PY

echo "visual gap audit: $out"
