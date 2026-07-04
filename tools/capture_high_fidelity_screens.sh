#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/high_fidelity_screens/latest}"
mkdir -p "$out"

python3 - "$out" <<'PY'
import hashlib
import json
import os
import struct
import sys
from pathlib import Path
out = Path(sys.argv[1])
root = Path.cwd()
default_renderer_manifest = root / 'artifacts/production_renderer/latest/production_renderer_manifest.json'
renderer_manifest = Path(os.environ.get('OATHYARD_PRODUCTION_RENDERER_MANIFEST', default_renderer_manifest.as_posix()))
if not renderer_manifest.is_absolute():
    renderer_manifest = root / renderer_manifest
default_renderer_root = renderer_manifest.parent
renderer_root = Path(os.environ.get('OATHYARD_PRODUCTION_RENDERER_ROOT', default_renderer_root.as_posix()))
if not renderer_root.is_absolute():
    renderer_root = root / renderer_root
asset_manifest = root / 'assets/manifests/production_visual_manifest.json'
failures = []


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b''):
            h.update(chunk)
    return h.hexdigest()


def png_dimensions(path: Path) -> tuple[int, int]:
    with path.open('rb') as f:
        header = f.read(24)
    if not header.startswith(b'\x89PNG\r\n\x1a\n'):
        raise ValueError(f'not a PNG file: {path}')
    if len(header) < 24 or header[12:16] != b'IHDR':
        raise ValueError(f'PNG missing IHDR header: {path}')
    return struct.unpack('>II', header[16:24])


def display_path(path: Path) -> str:
    try:
        return path.resolve().relative_to(root.resolve()).as_posix()
    except Exception:
        return path.as_posix()


def renderer_capture_path(value: str) -> Path:
    path = Path(value)
    if not path.is_absolute():
        path = renderer_root / path
    resolved = path.resolve()
    renderer_root_resolved = renderer_root.resolve()
    try:
        resolved.relative_to(renderer_root_resolved)
    except ValueError as exc:
        raise ValueError(f'capture file escapes production renderer root: {value}') from exc
    return resolved


renderer_data = {}
renderer_manifest_valid = False
production_renderer_complete = False
renderer_readiness_claims_present = False
if not renderer_manifest.is_file():
    failures.append(f'missing production renderer manifest: {renderer_manifest}')
else:
    try:
        renderer_data = json.loads(renderer_manifest.read_text(encoding='utf-8'))
    except Exception as exc:
        failures.append(f'production renderer manifest unreadable: {exc}')
    else:
        renderer_manifest_valid = True
        production_renderer_complete = renderer_data.get('production_renderer_complete') is True
        if renderer_data.get('production_renderer_complete') is not True:
            failures.append('production renderer manifest keeps production_renderer_complete false')
        if renderer_data.get('native_3d_render_capture') is not True:
            failures.append('production renderer manifest lacks native_3d_render_capture true')
        if renderer_data.get('truth_mutation') is not False:
            failures.append('production renderer manifest lacks truth_mutation false')
        if renderer_data.get('fallback_visual_substitutes_allowed') is not False:
            failures.append('production renderer manifest lacks fallback_visual_substitutes_allowed false')
        for readiness_flag in ('owner_visual_acceptance', 'public_demo_ready', 'release_candidate_ready'):
            if renderer_data.get(readiness_flag) is True:
                renderer_readiness_claims_present = True
                failures.append(f'production renderer manifest prematurely claims {readiness_flag} true')
if not asset_manifest.is_file():
    failures.append(f'missing production visual asset manifest: {asset_manifest}')
capture_groups = [
    ('boot_main_menu', 'Boot/main menu', 1),
    ('settings_accessibility', 'Settings/accessibility', 1),
    ('fighter_select', 'Fighter select', 1),
    ('loadout_select', 'Loadout select', 1),
    ('arena_select', 'Arena select when implemented', 1),
    ('oathyard_verdict_ring_establishing', 'OATHYARD verdict-ring establishing shot', 1),
    ('oathyard_arena_candidate_01', 'OATHYARD arena candidate renderer shot', 1),
    ('training_yard_establishing', 'Training-yard establishing shot', 1),
    ('fighter_closeup', 'Production fighter closeup', 6),
    ('armor_family_closeup_01', 'Candidate armor family closeup', 1),
    ('armor_loadout_family_closeup', 'Production armor/loadout family closeup', 6),
    ('weapon_family_closeup', 'Production weapon family closeup', 8),
    ('gameplay_distance_fighter_weapon_01', 'Candidate gameplay-distance fighter/weapon view', 1),
    ('gameplay_distance_fighter_loadout_family', 'Gameplay-distance fighter/loadout family view', 6),
    ('gameplay_distance_weapon_family', 'Gameplay-distance weapon family in-hand/shield context', 8),
    ('planning_timeline', 'Planning timeline', 1),
    ('pre_contact_frame', 'Pre-contact combat frame', 1),
    ('contact_frame', 'Contact frame', 1),
    ('material_armor_damage_frame', 'Material/armor damage frame', 1),
    ('injury_capability_consequence_frame', 'Injury/capability consequence frame', 1),
    ('recovery_replan_frame', 'Recovery/re-plan frame', 1),
    ('first_person_combat_view', 'First-person combat view', 1),
    ('third_person_combat_view', 'Third-person combat view', 1),
    ('fight_film_candidate_shot_01', 'Candidate fight-film camera shot', 1),
    ('fight_film_replay_camera_shot', 'Fight-film/replay camera shot', 1),
    ('replay_verification_ui_or_packet_view', 'Replay verification UI or packet view', 1),
    ('performance_debug_overlay', 'Performance/debug overlay separated from player HUD', 1),
]
capture_rows = []
for group_id, label, count in capture_groups:
    for index in range(1, count + 1):
        capture_id = group_id if count == 1 else f'{group_id}_{index:02d}'
        capture_rows.append({
            'capture_id': capture_id,
            'group_id': group_id,
            'label': label if count == 1 else f'{label} {index:02d}',
            'status': 'missing_native_3d_capture',
            'capture_file': '',
            'capture_file_sha256': '',
            'minimum_resolution_width': 1920,
            'minimum_resolution_height': 1080,
            'native_resolution_required': True,
            'upscaled_from_lower_resolution_allowed': False,
            'fallback_visual_substitutes_allowed': False,
            'renderer_backend_id': '',
            'renderer_build_hash_or_binary_hash': '',
            'quality_preset': '',
            'replay_path': '',
            'replay_final_hash': '',
            'content_manifest_hash': '',
            'asset_manifest_hash': '',
            'camera_mode': '',
            'frame_or_tick': '',
            'truth_mutation': False,
            'production_renderer_complete': False,
            'owner_visual_acceptance': False,
            'public_demo_ready': False,
            'release_candidate_ready': False,
        })
captures_by_id = {row['capture_id']: row for row in capture_rows}
can_ingest_native_captures = (
    renderer_manifest_valid
    and renderer_data.get('native_3d_render_capture') is True
    and renderer_data.get('truth_mutation') is False
    and renderer_data.get('fallback_visual_substitutes_allowed') is False
    and not renderer_readiness_claims_present
)
if can_ingest_native_captures:
    for capture in renderer_data.get('captures', []) or []:
        capture_id = str(capture.get('capture_id', ''))
        capture_classification = str(capture.get('capture_classification', ''))
        # Runtime asset-set candidate captures are validated below as candidate evidence;
        # they do not fill required high-fidelity production capture slots.
        if capture_classification == 'runtime_asset_set_candidate_native_3d_capture':
            continue
        # Production seed captures are tracked separately and do not map to required matrix slots
        if capture_id.startswith('production_seed_') or capture_classification == 'production_seed_native_3d_capture':
            if capture.get('native_3d_capture') is not True:
                failures.append(f'{capture_id}: production seed native_3d_capture is not true')
                continue
            if capture.get('truth_mutation') is not False:
                failures.append(f'{capture_id}: production seed truth_mutation is not false')
                continue
            capture_file = str(capture.get('capture_file', ''))
            if not capture_file:
                failures.append(f'{capture_id}: production seed capture_file missing')
                continue
            try:
                capture_path = renderer_capture_path(capture_file)
                width, height = png_dimensions(capture_path)
            except Exception as exc:
                failures.append(f'{capture_id}: invalid production seed PNG capture evidence: {exc}')
                continue
            if not capture_path.is_file():
                failures.append(f'{capture_id}: production seed capture file missing: {capture_path}')
                continue
            if capture_path.suffix.lower() != '.png' or not capture_path.name.startswith('production_renderer_'):
                failures.append(f'{capture_id}: production seed capture file is not a production_renderer_*.png native evidence file')
                continue
            if width < 1920 or height < 1080:
                failures.append(f'{capture_id}: production seed capture resolution {width}x{height} below required 1920x1080')
                continue
            # Count as production seed, do not fill required matrix slots
            continue
        row = captures_by_id.get(capture_id)
        if row is None:
            failures.append(f'production renderer manifest contains unknown capture_id: {capture_id or "<missing>"}')
            continue
        if capture.get('native_3d_capture') is not True:
            failures.append(f'{capture_id}: native_3d_capture is not true')
            continue
        if capture.get('truth_mutation') is not False:
            failures.append(f'{capture_id}: truth_mutation is not false')
            continue
        capture_file = str(capture.get('capture_file', ''))
        if not capture_file:
            failures.append(f'{capture_id}: capture_file missing')
            continue
        try:
            capture_path = renderer_capture_path(capture_file)
            width, height = png_dimensions(capture_path)
        except Exception as exc:
            failures.append(f'{capture_id}: invalid native PNG capture evidence: {exc}')
            continue
        if not capture_path.is_file():
            failures.append(f'{capture_id}: capture file missing: {capture_path}')
            continue
        if capture_path.suffix.lower() != '.png' or not capture_path.name.startswith('production_renderer_'):
            failures.append(f'{capture_id}: capture file is not a production_renderer_*.png native evidence file')
            continue
        if width < row['minimum_resolution_width'] or height < row['minimum_resolution_height']:
            failures.append(f'{capture_id}: capture resolution {width}x{height} below required {row["minimum_resolution_width"]}x{row["minimum_resolution_height"]}')
            continue
        row.update({
            'status': 'native_3d_capture_present' if production_renderer_complete else 'candidate_native_3d_capture_not_production_complete',
            'capture_file': display_path(capture_path),
            'capture_file_sha256': sha256(capture_path),
            'renderer_backend_id': str(capture.get('renderer_backend_id', '')),
            'renderer_build_hash_or_binary_hash': str(capture.get('renderer_build_hash_or_binary_hash', '')),
            'quality_preset': str(capture.get('quality_preset', '')),
            'replay_path': str(capture.get('replay_path', '')),
            'replay_final_hash': str(capture.get('replay_final_hash', '')),
            'content_manifest_hash': str(capture.get('content_manifest_hash', '')),
            'asset_manifest_hash': str(capture.get('asset_manifest_hash', '')),
            'camera_mode': str(capture.get('camera_mode', '')),
            'frame_or_tick': str(capture.get('frame_or_tick', '')),
            'truth_mutation': False,
            'production_renderer_complete': production_renderer_complete,
            'owner_visual_acceptance': False,
            'public_demo_ready': False,
            'release_candidate_ready': False,
        })
current_native_capture_count = sum(1 for row in capture_rows if row['status'] == 'native_3d_capture_present')
candidate_native_capture_count = sum(1 for row in capture_rows if row['status'] == 'candidate_native_3d_capture_not_production_complete')
runtime_asset_set_candidate_captures = []
if can_ingest_native_captures:
    for capture in renderer_data.get('captures', []) or []:
        if str(capture.get('capture_classification', '')) != 'runtime_asset_set_candidate_native_3d_capture':
            continue
        capture_id = str(capture.get('capture_id', ''))
        capture_file = str(capture.get('capture_file') or capture.get('file') or '')
        if capture.get('native_3d_capture') is not True:
            failures.append(f'{capture_id}: runtime asset-set native_3d_capture is not true')
            continue
        if capture.get('truth_mutation') is not False:
            failures.append(f'{capture_id}: runtime asset-set truth_mutation is not false')
            continue
        if capture.get('mesh_geometry_consumed') is not True or int(capture.get('mesh_asset_count', 0) or 0) <= 0:
            failures.append(f'{capture_id}: runtime asset-set capture lacks mesh consumption evidence')
            continue
        if not capture_file:
            failures.append(f'{capture_id}: runtime asset-set capture_file missing')
            continue
        try:
            capture_path = renderer_capture_path(capture_file)
            width, height = png_dimensions(capture_path)
        except Exception as exc:
            failures.append(f'{capture_id}: invalid runtime asset-set PNG capture evidence: {exc}')
            continue
        if not capture_path.is_file():
            failures.append(f'{capture_id}: runtime asset-set capture file missing: {capture_path}')
            continue
        if capture_path.suffix.lower() != '.png' or not capture_path.name.startswith('production_renderer_asset_set_'):
            failures.append(f'{capture_id}: runtime asset-set capture file is not a production_renderer_asset_set_*.png native evidence file')
            continue
        if width < 1920 or height < 1080:
            failures.append(f'{capture_id}: runtime asset-set capture resolution {width}x{height} below required 1920x1080')
            continue
        runtime_asset_set_candidate_captures.append({
            'capture_id': capture_id,
            'asset_set_id': str(capture.get('asset_set_id', '')),
            'capture_file': display_path(capture_path),
            'capture_file_sha256': sha256(capture_path),
            'renderer_backend_id': str(capture.get('renderer_backend_id', '')),
            'quality_preset': str(capture.get('quality_preset', '')),
            'camera_mode': str(capture.get('camera_mode', '')),
            'mesh_asset_count': int(capture.get('mesh_asset_count', 0) or 0),
            'mesh_asset_ids': capture.get('mesh_asset_ids', []),
            'truth_mutation': False,
            'production_renderer_complete': False,
            'owner_visual_acceptance': False,
            'public_demo_ready': False,
            'release_candidate_ready': False,
        })
runtime_asset_set_candidate_capture_count = len(runtime_asset_set_candidate_captures)
# Count production seed captures from the renderer manifest (tracked separately from required matrix slots)
production_seed_native_capture_count = 0
if can_ingest_native_captures:
    for capture in renderer_data.get('captures', []) or []:
        cid = str(capture.get('capture_id', ''))
        cc = str(capture.get('capture_classification', ''))
        if cid.startswith('production_seed_') or cc == 'production_seed_native_3d_capture':
            production_seed_native_capture_count += 1
missing_capture_count = len(capture_rows) - current_native_capture_count
if missing_capture_count:
    failures.append(f'missing native high-fidelity capture slots: {missing_capture_count}')
capture_matrix = {
    'schema': 'oathyard.high_fidelity_capture_matrix.v1',
    'tool': 'tools/capture_high_fidelity_screens.sh',
    'production_renderer_manifest': display_path(renderer_manifest),
    'required_capture_group_count': len(capture_groups),
    'required_capture_slot_count': len(capture_rows),
    'current_native_capture_count': current_native_capture_count,
    'candidate_native_capture_count': candidate_native_capture_count,
    'runtime_asset_set_candidate_capture_count': runtime_asset_set_candidate_capture_count,
    'production_seed_native_capture_count': production_seed_native_capture_count,
    'production_ready_native_capture_count': current_native_capture_count,
    'missing_native_capture_count': missing_capture_count,
    'minimum_resolution_width': 1920,
    'minimum_resolution_height': 1080,
    'native_3d_visual_evidence_required': True,
    'fallback_visual_substitutes_allowed': False,
    'production_renderer_complete': production_renderer_complete,
    'owner_visual_acceptance': False,
    'public_demo_ready': False,
    'release_candidate_ready': False,
    'truth_mutation': False,
    'capture_groups': [
        {'group_id': group_id, 'label': label, 'required_slots': count}
        for group_id, label, count in capture_groups
    ],
    'captures': capture_rows,
    'runtime_asset_set_candidate_captures': runtime_asset_set_candidate_captures,
}
(out / 'high_fidelity_capture_matrix.json').write_text(json.dumps(capture_matrix, indent=2, sort_keys=True) + '\n', encoding='utf-8')
matrix_report = [
    '# OATHYARD High-Fidelity Capture Matrix',
    '',
    'Status: BLOCKED_PENDING_NATIVE_3D_RENDERER_CAPTURE' if missing_capture_count else 'Status: PASSED',
    '',
    f"- Required capture groups: `{len(capture_groups)}`",
    f"- Required capture slots: `{len(capture_rows)}`",
    f"- Current native capture count: `{current_native_capture_count}`",
    f"- Candidate native capture count: `{candidate_native_capture_count}`",
    f"- Runtime asset-set candidate capture count: `{runtime_asset_set_candidate_capture_count}`",
    f"- Production seed native capture count: `{production_seed_native_capture_count}`",
    f"- Missing native capture count: `{missing_capture_count}`",
    '- Minimum native resolution: `1920x1080`',
    '- Fallback visual substitutes: `forbidden`',
    f"- Production renderer complete: `{str(production_renderer_complete).lower()}`",
    '- Owner visual acceptance: `false`',
    '- Public demo ready: `false`',
    '- Release candidate ready: `false`',
    '',
    '## Required captures',
]
for row in capture_rows:
    matrix_report.append(f"- `{row['capture_id']}` — {row['label']} — `{row['status']}`")
(out / 'high_fidelity_capture_matrix.md').write_text('\n'.join(matrix_report) + '\n', encoding='utf-8')
manifest = {
    'schema': 'oathyard.high_fidelity_screen_capture.v3',
    'tool': 'tools/capture_high_fidelity_screens.sh',
    'passed': not failures,
    'capture_count': current_native_capture_count,
    'candidate_native_capture_count': candidate_native_capture_count,
    'runtime_asset_set_candidate_capture_count': runtime_asset_set_candidate_capture_count,
    'production_seed_native_capture_count': production_seed_native_capture_count,
    'required_capture_matrix': (out / 'high_fidelity_capture_matrix.json').as_posix(),
    'production_renderer_manifest': display_path(renderer_manifest),
    'required_capture_group_count': len(capture_groups),
    'required_capture_slot_count': len(capture_rows),
    'missing_native_capture_count': missing_capture_count,
    'native_3d_visual_evidence_required': True,
    'production_renderer_complete': production_renderer_complete,
    'owner_visual_acceptance': False,
    'public_demo_ready': False,
    'release_candidate_ready': False,
    'truth_mutation': False,
    'failed_check_count': len(failures),
    'failures': failures,
}
(out / 'high_fidelity_screen_manifest.json').write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n', encoding='utf-8')
(out / 'failed_high_fidelity_screens.txt').write_text('none\n' if not failures else '\n'.join(failures) + '\n', encoding='utf-8')
report = ['# OATHYARD High-Fidelity Screen Capture Gate', '', f"Status: {'PASSED' if not failures else 'BLOCKED_PENDING_NATIVE_3D_RENDERER_CAPTURE'}", '', '- Required evidence: native 3D renderer captures with renderer/asset/camera metadata.', '- Fallback generated visuals: `forbidden`', f'- Production renderer complete: `{str(production_renderer_complete).lower()}`', '- Owner visual acceptance: `false`', '- Public demo ready: `false`', '- Release candidate ready: `false`']
if failures:
    report.extend(['', '## Failures'] + [f'- {f}' for f in failures])
(out / 'high_fidelity_screen_report.md').write_text('\n'.join(report) + '\n', encoding='utf-8')
if failures:
    raise SystemExit(1)
PY
