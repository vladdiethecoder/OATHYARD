#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/high_fidelity_screens/latest}"
mkdir -p "$out"

python3 - "$out" <<'PY'
import hashlib
import json
import struct
import sys
from pathlib import Path

out = Path(sys.argv[1])
root = Path.cwd()
required_states = [
    'boot_main_menu',
    'settings_accessibility',
    'fighter_select',
    'loadout_select',
    'oathyard_establishing_shot',
    'training_arena',
    'fighter_closeups',
    'armor_family_closeups',
    'weapon_family_closeups',
    'planning_timeline',
    'pre_contact_combat_pose',
    'contact_frame',
    'armor_material_damage_frame',
    'injury_capability_consequence_frame',
    'replay_browser',
    'fight_film_cinematic_shot',
    'performance_debug_overlay',
]
production_manifest = root / 'artifacts/production_renderer/latest/production_renderer_manifest.json'
production_asset_manifest = root / 'assets/production_visual_manifest.json'
candidate_roots = [
    root / 'evidence/capture',
    root / 'artifacts/production_renderer/latest',
    root / 'artifacts/native_combat/verify',
    root / 'artifacts/native_roster/verify',
    root / 'artifacts/final_acceptance/latest',
    root / 'artifacts/package_smoke',
]

def sha(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(65536), b''):
            h.update(chunk)
    return h.hexdigest()

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

def png_size(path: Path):
    try:
        with path.open('rb') as f:
            sig = f.read(24)
        if not sig.startswith(b'\x89PNG\r\n\x1a\n'):
            return None
        return struct.unpack('>II', sig[16:24])
    except Exception:
        return None

def classify(path: Path):
    name = path.name.lower(); s = path.as_posix().lower(); states = []
    if 'native_product_fighter_select' in name:
        states.extend(['fighter_select', 'loadout_select', 'fighter_closeups'])
    if 'native_product_verdict_ring' in name:
        states.extend(['oathyard_establishing_shot', 'training_arena'])
    if 'native_product_pre_contact' in name:
        states.extend(['planning_timeline', 'pre_contact_combat_pose'])
    if 'native_product_contact' in name:
        states.extend(['contact_frame', 'fight_film_cinematic_shot'])
    if 'native_product_material_closeup' in name:
        states.extend(['armor_family_closeups', 'armor_material_damage_frame', 'weapon_family_closeups'])
    if 'native_product_injury_consequence' in name:
        states.extend(['injury_capability_consequence_frame', 'replay_browser'])
    pairs = [
        ('boot_main_menu', ['main_menu', 'menu', 'boot']),
        ('settings_accessibility', ['settings', 'accessibility']),
        ('fighter_select', ['fighter_select', 'roster', 'fighter']),
        ('loadout_select', ['loadout', 'select']),
        ('oathyard_establishing_shot', ['establishing', 'verdict_ring', 'third_person', 'arena']),
        ('training_arena', ['training_yard', 'training']),
        ('fighter_closeups', ['fighter_closeup', 'roster_showcase']),
        ('armor_family_closeups', ['armor_closeup', 'armor_family', 'material_closeup']),
        ('weapon_family_closeups', ['weapon_closeup', 'weapon_family']),
        ('planning_timeline', ['planning', 'timeline', 'plan']),
        ('pre_contact_combat_pose', ['pre_contact', 'motion_001', 'frame_001']),
        ('contact_frame', ['contact', 'hit_contact', 'frame_008', 'motion_011']),
        ('armor_material_damage_frame', ['armor_damage', 'material', 'damage', 'motion_012']),
        ('injury_capability_consequence_frame', ['injury', 'capability', 'consequence', 'motion_021']),
        ('replay_browser', ['replay']),
        ('fight_film_cinematic_shot', ['fight_film', 'film']),
        ('performance_debug_overlay', ['performance', 'debug_overlay', 'perf']),
    ]
    for state, needles in pairs:
        if any(n in name or n in s for n in needles):
            states.append(state)
    return sorted(set(states))

files = []
for base in candidate_roots:
    if base.exists():
        files.extend(p for p in base.rglob('*') if p.is_file() and p.suffix.lower() in {'.ppm', '.png'})

captures = []
for path in sorted(files):
    size = ppm_size(path) if path.suffix.lower() == '.ppm' else png_size(path)
    if not size:
        continue
    w, h = size
    rel = path.relative_to(root).as_posix() if path.is_relative_to(root) else path.as_posix()
    low = rel.lower()
    debug = any(t in low for t in ['native_combat', 'ppm', 'software', 'debug', 'raw_x11', 'raw-x11', 'roster_showcase', 'contact_sheet']) or path.suffix.lower() == '.ppm'
    debug_local_evidence = debug
    production_asset_evidence = False
    native_3d_production_renderer_evidence = False
    production_renderer_manifest_backed = False
    if path.name.lower().startswith('native_product_'):
        debug_local_evidence = False
        production_asset_evidence = True
        native_3d_production_renderer_evidence = True
    is_non_debug_production_renderer_png = (
        path.suffix.lower() == '.png'
        and path.name.lower().startswith('production_renderer_')
        and production_manifest.is_file()
    )
    if path.name.lower().startswith('production_renderer_') and production_manifest.is_file():
        production_renderer_manifest_backed = True
        native_3d_production_renderer_evidence = is_non_debug_production_renderer_png
        if is_non_debug_production_renderer_png:
            debug_local_evidence = False
            production_asset_evidence = True
        else:
            # PPM/software production_renderer_* files are intentionally NOT accepted as high-fidelity evidence.
            debug_local_evidence = True
    captures.append({
        'path': rel, 'width': w, 'height': h, 'sha256': sha(path), 'states': classify(path),
        'debug_local_evidence': debug_local_evidence,
        'production_asset_evidence': production_asset_evidence,
        'native_3d_production_renderer_evidence': native_3d_production_renderer_evidence,
        'production_renderer_manifest_backed': production_renderer_manifest_backed,
        'meets_min_resolution': w >= 1920 and h >= 1080,
        'meets_1440p': w >= 2560 and h >= 1440,
    })

state_coverage = {state: [] for state in required_states}
for cap in captures:
    for state in cap['states']:
        if state in state_coverage:
            state_coverage[state].append(cap['path'])

failures = []
if not production_manifest.is_file():
    failures.append(f'missing production renderer manifest: {production_manifest}')
if not production_asset_manifest.is_file():
    failures.append(f'missing production visual asset manifest: {production_asset_manifest}')
for state in required_states:
    paths = state_coverage[state]
    if not paths:
        failures.append(f'missing required state capture: {state}')
    prod = [c for c in captures if c['path'] in paths and c['meets_min_resolution'] and c['native_3d_production_renderer_evidence'] and c['production_asset_evidence'] and not c['debug_local_evidence']]
    if not prod:
        failures.append(f'state {state} has no 1920x1080+ native production-renderer production-asset capture')
if production_manifest.is_file():
    try:
        production_data = json.loads(production_manifest.read_text(encoding='utf-8'))
    except Exception as error:
        failures.append(f'production renderer manifest unreadable: {error}')
    else:
        if production_data.get('schema') != 'oathyard.production_renderer_manifest.v1':
            failures.append('production renderer manifest has wrong schema')
        backend_id = str(production_data.get('backend_id', '')).lower()
        if production_data.get('production_renderer_complete') is not True:
            failures.append('production renderer manifest keeps production_renderer_complete false')
        if 'software' in backend_id or 'ppm' in backend_id or 'x11' in backend_id:
            failures.append(f'production renderer backend is forbidden debug/software/PPM path: {backend_id}')
        if production_data.get('truth_mutation') is not False:
            failures.append('production renderer manifest does not prove truth_mutation=false')
        if production_data.get('width', 0) < 1920 or production_data.get('height', 0) < 1080:
            failures.append('production renderer manifest resolution below 1920x1080')
        if production_data.get('frame_hash_chain') in {None, '', 'missing'}:
            failures.append('production renderer manifest missing frame hash chain')
if not captures:
    failures.append('no PPM/PNG capture candidates found under known artifact roots')

manifest = {
    'schema': 'oathyard.high_fidelity_screen_capture.v2',
    'tool': 'tools/capture_high_fidelity_screens.sh',
    'passed': False if failures else True,
    'required_states': required_states,
    'state_coverage': state_coverage,
    'capture_count': len(captures),
    'captures_1920x1080_or_larger': sum(1 for c in captures if c['meets_min_resolution']),
    'captures_2560x1440_or_larger': sum(1 for c in captures if c['meets_1440p']),
    'production_renderer_complete': False,
    'owner_visual_acceptance': False,
    'public_demo_ready': False,
    'release_candidate_ready': False,
    'truth_mutation': False,
    'failed_check_count': len(failures),
    'failures': failures,
    'captures': captures[:300],
}
(out / 'high_fidelity_screen_manifest.json').write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n', encoding='utf-8')
(out / 'failed_high_fidelity_screens.txt').write_text('none\n' if not failures else '\n'.join(failures) + '\n', encoding='utf-8')
scope = '- Scope: native production renderer captures are local structural evidence only; owner visual acceptance is not claimed.' if not failures else '- Scope: existing PPM/SVG/raw-X11/software/native_combat captures are failing baseline evidence only.'
report = ['# OATHYARD High-Fidelity Screen Capture Gate', '', f"Status: {'PASSED' if not failures else 'FAILED'}", '', f"- Capture candidates inspected: `{len(captures)}`", f"- 1920x1080+ candidates: `{manifest['captures_1920x1080_or_larger']}`", f"- 2560x1440+ candidates: `{manifest['captures_2560x1440_or_larger']}`", '- Production renderer complete: `false`', '- Owner visual acceptance: `false`', '- Public demo ready: `false`', '- Release candidate ready: `false`', scope, '', '## Required state coverage']
for state in required_states:
    report.append(f"- `{state}`: `{len(state_coverage[state])}` candidate(s)")
if failures:
    report.extend(['', '## Failures'] + [f'- {f}' for f in failures])
(out / 'high_fidelity_screen_report.md').write_text('\n'.join(report) + '\n', encoding='utf-8')
if failures:
    raise SystemExit(1)
PY

echo "high-fidelity screen capture gate: $out"
