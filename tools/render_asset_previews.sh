#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/asset_previews/latest}"
mkdir -p "$out/previews"

./tools/build_assets.sh > "$out/build_assets.log" 2>&1
# Current validate_assets is production fail-closed; keep structural preview generation useful for V0 by invoking the structural validator directly.
python3 tools/asset_pipeline.py validate > "$out/structural_validate_assets.log" 2>&1

python3 - "$out" <<'PY'
import hashlib
import html
import json
import shutil
import sys
from pathlib import Path

ROOT = Path.cwd()
out = Path(sys.argv[1])
preview_out = out / 'previews'
preview_out.mkdir(parents=True, exist_ok=True)
manifest_path = ROOT / 'assets/runtime_manifest.json'
production_manifest_path = ROOT / 'assets/production_visual_manifest.json'
required_counts = {'fighters': 6, 'weapons': 8, 'armor': 6, 'arenas': 2}
failures = []

def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(65536), b''):
            h.update(chunk)
    return h.hexdigest()

if not manifest_path.is_file():
    raise SystemExit(f'missing runtime manifest: {manifest_path}')
manifest = json.loads(manifest_path.read_text(encoding='utf-8'))
production_manifest = json.loads(production_manifest_path.read_text(encoding='utf-8')) if production_manifest_path.is_file() else {}
production_entries = {str(item.get('id', '')): item for item in production_manifest.get('entries', [])}
production_blockers = []
if production_manifest.get('candidate_run_id') == 't_73291be5':
    production_blockers.append('t_73291be5 previews/captures are production-candidate evidence, not final high-fidelity production closeups')
entries = []
kind_counts = {k: 0 for k in required_counts}
for item in manifest.get('entries', []):
    asset_id = str(item.get('id', ''))
    kind = str(item.get('kind', ''))
    if kind not in required_counts:
        failures.append(f'{asset_id}: unexpected kind {kind}')
        continue
    kind_counts[kind] += 1
    source = ROOT / str(item.get('source', ''))
    preview = ROOT / str(item.get('preview', ''))
    runtime_mesh = ROOT / str(item.get('runtime_mesh', ''))
    runtime_gltf = ROOT / str(item.get('runtime_gltf', ''))
    production_entry = production_entries.get(asset_id, {})
    production_preview = ROOT / str(production_entry.get('preview_render', {}).get('path', ''))
    production_captures = production_entry.get('in_engine_screenshot', {}).get('captures', {})
    production_capture_paths = [ROOT / str(path) for path in production_captures.values()]
    copied_name = f'{kind}_{asset_id}.svg'
    copied = preview_out / copied_name
    if preview.is_file():
        shutil.copyfile(preview, copied)
    checks = {
        'source_exists': source.is_file(),
        'runtime_mesh_exists': runtime_mesh.is_file(),
        'runtime_gltf_exists': runtime_gltf.is_file(),
        'structural_svg_preview_exists': copied.is_file(),
        'production_png_or_engine_closeup_exists': production_preview.is_file() or any(path.is_file() for path in production_capture_paths),
        'in_engine_screenshot_exists': any(path.is_file() for path in production_capture_paths),
    }
    production_checks = {
        'final_dcc_high_fidelity_preview': False,
    }
    for check, passed in checks.items():
        if not passed:
            failures.append(f'{asset_id}: {check}')
    for check, passed in production_checks.items():
        if not passed:
            production_blockers.append(f'{asset_id}: {check}')
    entries.append({
        'id': asset_id,
        'kind': kind,
        'source': item.get('source', ''),
        'runtime_gltf': item.get('runtime_gltf', ''),
        'structural_preview': f'previews/{copied_name}' if copied.is_file() else '',
        'structural_preview_sha256': sha256_file(copied) if copied.is_file() else '',
        'production_preview': production_entry.get('preview_render', {}).get('path', ''),
        'production_captures': production_captures,
        'checks': checks,
        'production_checks': production_checks,
        'passed': all(checks.values()),
    })
for kind, required in required_counts.items():
    if kind_counts[kind] < required:
        failures.append(f'{kind}: count {kind_counts[kind]} below required {required}')
local_preview_gate_passed = not failures
high_fidelity_production_preview_gate_passed = (
    production_manifest.get('production_assets_complete') is True
    and not production_blockers
    and all(all(e.get('production_checks', {}).values()) for e in entries)
)
passed = local_preview_gate_passed
manifest_out = {
    'schema': 'oathyard.asset_previews.v2',
    'tool': 'tools/render_asset_previews.sh',
    'passed': passed,
    'local_preview_gate_passed': local_preview_gate_passed,
    'high_fidelity_production_preview_gate_passed': high_fidelity_production_preview_gate_passed,
    'current_preview_quality': 'structural_svg_plus_production_candidate_png_capture_evidence',
    'production_renderer_complete': False,
    'production_asset_previews_complete': False,
    'production_candidate_previews_present': any(bool(e.get('production_preview')) for e in entries),
    'owner_visual_acceptance_claimed': False,
    'public_demo_ready': False,
    'release_candidate_ready': False,
    'entry_count': len(entries),
    'kind_counts': kind_counts,
    'failed_check_count': len(failures),
    'failures': failures,
    'production_preview_blocker_count': len(production_blockers),
    'production_preview_blockers': production_blockers,
    'entries': entries,
}
(out / 'asset_preview_manifest.json').write_text(json.dumps(manifest_out, indent=2, sort_keys=True) + '\n', encoding='utf-8')
(out / 'failed_asset_previews.txt').write_text('none\n' if passed else '\n'.join(failures) + '\n', encoding='utf-8')
report = ['# OATHYARD Asset Preview Render Report', '', f"Status: {'PASSED' if passed else 'FAILED'}", '', '- Current previews include structural SVG baseline evidence plus production-candidate PNG closeups/captures from the t_73291be5 presentation lane.', f'- Local preview gate passed: `{str(local_preview_gate_passed).lower()}`', f'- High-fidelity production preview gate passed: `{str(high_fidelity_production_preview_gate_passed).lower()}`', '- Owner visual acceptance claimed: `false`', '- Production renderer complete: `false`', '', '## Preview files']
for e in entries:
    report.append(f"- `{'passed' if e['passed'] else 'failed'}` `{e['id']}` `{e['kind']}` structural `{e['structural_preview']}` production `{e['production_preview']}` sha `{e['structural_preview_sha256'][:16]}`")
if failures:
    report.extend(['', '## Failures'] + [f'- {f}' for f in failures])
if production_blockers:
    report.extend(['', '## Production blockers'] + [f'- {f}' for f in production_blockers])
(out / 'asset_preview_report.md').write_text('\n'.join(report) + '\n', encoding='utf-8')
cols = 2; cell_w = 520; cell_h = 92; rows = (len(entries)+cols-1)//cols
svg = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{cols*cell_w}" height="{max(1, rows)*cell_h+96}" viewBox="0 0 {cols*cell_w} {max(1, rows)*cell_h+96}">', '<rect width="100%" height="100%" fill="#15120f"/>', '<text x="24" y="32" fill="#f3e8cf" font-family="monospace" font-size="20">OATHYARD asset preview/capture evidence contact sheet</text>', '<text x="24" y="56" fill="#c9b99b" font-family="monospace" font-size="13">Structural SVG previews plus production-candidate PNG capture paths; owner/release readiness false.</text>']
for i, e in enumerate(entries):
    x = (i % cols) * cell_w + 24; y = (i // cols) * cell_h + 88
    svg.append(f'<rect x="{x}" y="{y}" width="{cell_w-36}" height="74" fill="#263526" stroke="#6f5d3f"/>')
    svg.append(f'<text x="{x+12}" y="{y+24}" fill="#f7efd9" font-family="monospace" font-size="15">{html.escape(e["kind"])} / {html.escape(e["id"])}</text>')
    svg.append(f'<text x="{x+12}" y="{y+46}" fill="#d5c6a3" font-family="monospace" font-size="12">baseline preview: {html.escape(e["structural_preview"])}</text>')
    svg.append(f'<text x="{x+12}" y="{y+64}" fill="#d5c6a3" font-family="monospace" font-size="12">production: {html.escape(e["production_preview"])}</text>')
svg.append('</svg>')
(out / 'asset_preview_contact_sheet.svg').write_text('\n'.join(svg) + '\n', encoding='utf-8')
if not passed:
    raise SystemExit(1)
PY

echo "asset previews rendered: $out"
