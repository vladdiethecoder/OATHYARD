#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/asset_previews/latest}"
mkdir -p "$out/previews"

./tools/build_assets.sh > "$out/build_assets.log" 2>&1
# Current validate_assets is production fail-closed; keep structural preview generation useful for V0/V0.5.
python3 tools/asset_pipeline.py validate > "$out/structural_validate_assets.log" 2>&1

python3 - "$out" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

ROOT = Path.cwd()
out = Path(sys.argv[1])
preview_out = out / 'previews'
preview_out.mkdir(parents=True, exist_ok=True)
manifest_path = ROOT / 'assets/runtime_manifest.json'
production_manifest_path = ROOT / 'assets/manifests/production_visual_manifest.json'
candidate_manifest_path = ROOT / 'assets/manifests/production_candidate_visual_manifest.json'
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
if production_manifest.get('production_candidate_manifest'):
    candidate_manifest_path = ROOT / str(production_manifest.get('production_candidate_manifest'))
candidate_manifest = json.loads(candidate_manifest_path.read_text(encoding='utf-8')) if candidate_manifest_path.is_file() else {}
candidate_entries = {str(item.get('id', '')): item for item in candidate_manifest.get('entries', [])}
production_entries = {str(item.get('id', '')): item for item in production_manifest.get('entries', [])}
production_blockers = []
if candidate_manifest.get('candidate_run_id') == 't_73291be5':
    production_blockers.append('t_73291be5 previews/captures are production-candidate evidence, not final high-fidelity production closeups')
if production_manifest.get('entries'):
    production_blockers.append('production_visual_manifest has entries before production asset gate')

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
    runtime_mesh = ROOT / str(item.get('runtime_mesh', ''))
    runtime_gltf = ROOT / str(item.get('runtime_gltf', ''))
    candidate_entry = candidate_entries.get(asset_id, {})
    production_entry = production_entries.get(asset_id, {})
    candidate_preview = ROOT / str(candidate_entry.get('preview_render', {}).get('path', ''))
    candidate_captures = candidate_entry.get('in_engine_screenshot', {}).get('captures', {})
    candidate_capture_paths = [ROOT / str(path) for path in candidate_captures.values()]
    production_preview = ROOT / str(production_entry.get('preview_render', {}).get('path', ''))
    production_captures = production_entry.get('in_engine_screenshot', {}).get('captures', {})
    production_capture_paths = [ROOT / str(path) for path in production_captures.values()]
    checks = {
        'source_exists': source.is_file(),
        'runtime_mesh_exists': runtime_mesh.is_file(),
        'runtime_gltf_exists': runtime_gltf.is_file(),
        'candidate_png_or_engine_closeup_exists': candidate_preview.is_file() or any(path.is_file() for path in candidate_capture_paths),
        'candidate_in_context_capture_exists': any(path.is_file() for path in candidate_capture_paths),
    }
    production_checks = {
        'final_dcc_high_fidelity_preview': bool(production_preview.is_file() or any(path.is_file() for path in production_capture_paths)),
    }
    for check, ok in checks.items():
        if not ok:
            failures.append(f'{asset_id}: {check}')
    for check, ok in production_checks.items():
        if not ok:
            production_blockers.append(f'{asset_id}: {check}')
    entries.append({
        'id': asset_id,
        'kind': kind,
        'source': item.get('source', ''),
        'runtime_gltf': item.get('runtime_gltf', ''),
        'structural_preview': '',
        'structural_preview_sha256': '',
        'candidate_preview': candidate_entry.get('preview_render', {}).get('path', ''),
        'candidate_captures': candidate_captures,
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
    'schema': 'oathyard.asset_previews.v3',
    'tool': 'tools/render_asset_previews.sh',
    'passed': passed,
    'local_preview_gate_passed': local_preview_gate_passed,
    'high_fidelity_production_preview_gate_passed': high_fidelity_production_preview_gate_passed,
    'current_preview_quality': 'production_candidate_png_capture_evidence_only_svg_structural_previews_excluded',
    'production_renderer_complete': False,
    'production_asset_previews_complete': False,
    'production_candidate_previews_present': any(bool(e.get('candidate_preview')) for e in entries),
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
report = ['# OATHYARD Asset Preview Render Report', '', f"Status: {'PASSED' if passed else 'FAILED'}", '', '- Current preview evidence uses production-candidate native-renderer closeup metadata only; standalone preview sheets are excluded from this audit surface.', f'- Local preview gate passed: `{str(local_preview_gate_passed).lower()}`', f'- High-fidelity production preview gate passed: `{str(high_fidelity_production_preview_gate_passed).lower()}`', '- Owner visual acceptance claimed: `false`', '- Production renderer complete: `false`', '', '## Preview files']
for e in entries:
    report.append(f"- `{'passed' if e['passed'] else 'failed'}` `{e['id']}` `{e['kind']}` candidate `{e['candidate_preview']}` production `{e['production_preview']}`")
if failures:
    report.extend(['', '## Failures'] + [f'- {f}' for f in failures])
if production_blockers:
    report.extend(['', '## Production blockers'] + [f'- {f}' for f in production_blockers])
(out / 'asset_preview_report.md').write_text('\n'.join(report) + '\n', encoding='utf-8')

if not passed:
    raise SystemExit(1)
PY

echo "asset previews rendered: $out"