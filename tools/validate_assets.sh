#!/usr/bin/env bash
set -uo pipefail

out="${1:-assets}"
mkdir -p "$out"
structural_log="$out/asset_structural_validation.log"
python3 tools/asset_pipeline.py validate > "$structural_log" 2>&1
structural_rc=$?

python3 - "$out" "$structural_rc" <<'PY'
import json
import sys
from pathlib import Path

out = Path(sys.argv[1])
structural_rc = int(sys.argv[2])
root = Path.cwd()
local_failures = []
production_blockers = []

if structural_rc != 0:
    local_failures.append(
        f'low-level structural asset validation failed; see {out / "asset_structural_validation.log"}'
    )

prod_path = root / 'assets/production_visual_manifest.json'
candidate_path = root / 'assets/production_candidate_visual_manifest.json'
prod_data = {}
candidate_data = {}

if not prod_path.is_file():
    local_failures.append('missing production visual asset manifest: assets/production_visual_manifest.json')
else:
    try:
        prod_data = json.loads(prod_path.read_text(encoding='utf-8'))
    except Exception as exc:
        local_failures.append(f'production visual asset manifest invalid JSON: {exc}')
        prod_data = {}
    if prod_data.get('schema') != 'oathyard.production_visual_assets.v1':
        local_failures.append(
            f'production visual asset manifest has unexpected schema: {prod_data.get("schema")}'
        )
    if prod_data.get('production_candidate_manifest'):
        candidate_path = root / str(prod_data.get('production_candidate_manifest'))
    for flag in [
        'production_renderer_complete',
        'owner_visual_acceptance',
        'public_demo_ready',
        'release_candidate_ready',
    ]:
        if prod_data.get(flag) is True:
            local_failures.append(f'production visual asset manifest claims {flag} true without gate evidence')
    if prod_data.get('entries'):
        local_failures.append(
            'production visual asset manifest must not contain candidate-only entries before production gate passes'
        )

if not candidate_path.is_file():
    # Backward compatibility with the older combined manifest while still fail-closing production readiness.
    if prod_data.get('production_candidate_assets_complete') is True and prod_data.get('entries'):
        candidate_data = prod_data
    else:
        local_failures.append(f'missing production candidate visual asset manifest: {candidate_path}')
else:
    try:
        candidate_data = json.loads(candidate_path.read_text(encoding='utf-8'))
    except Exception as exc:
        local_failures.append(f'production candidate visual asset manifest invalid JSON: {exc}')
        candidate_data = {}

if candidate_data:
    if candidate_data.get('schema') not in {
        'oathyard.production_candidate_visual_assets.v1',
        'oathyard.production_visual_assets.v1',
    }:
        local_failures.append(
            f'production candidate manifest has unexpected schema: {candidate_data.get("schema")}'
        )
    if candidate_data.get('production_candidate_assets_complete') is not True:
        local_failures.append('production candidate visual asset manifest does not prove candidate completeness')

production_assets_complete = prod_data.get('production_assets_complete') is True
if not production_assets_complete:
    production_blockers.append('production visual asset manifest does not prove production_assets_complete true')
if candidate_data.get('candidate_run_id') == 't_73291be5':
    production_blockers.append(
        't_73291be5 model candidates are production-candidate evidence, not final high-fidelity DCC-authored production assets'
    )

required_categories = {'fighters', 'armor', 'weapons', 'arenas'}
entries = candidate_data.get('entries', []) if candidate_data else []
cats = {e.get('kind') for e in entries}
missing = sorted(required_categories - cats)
if missing:
    local_failures.append('production candidate visual asset manifest missing categories: ' + ','.join(missing))

for e in entries:
    aid = e.get('id', '<unknown>')
    for field in [
        'source_file',
        'provenance_license',
        'authoring_process',
        'runtime_export',
        'content_hash',
        'preview_render',
        'in_engine_screenshot',
        'validation_result',
    ]:
        if not e.get(field):
            local_failures.append(f'{aid} missing {field}')
    source_file = str(e.get('source_file', ''))
    source_ext = Path(source_file).suffix.lower()
    if source_ext not in {'.blend', '.usd', '.usda', '.usdc', '.fbx'}:
        production_blockers.append(
            f'{aid} source_file {source_file} is not a DCC/interchange source (.blend/.usd/.usda/.usdc/.fbx)'
        )
    authoring = e.get('authoring_process', {})
    if authoring.get('external_dcc_validation_claimed') is not True:
        production_blockers.append(f'{aid} lacks external_dcc_validation_claimed true')
    if authoring.get('external_khronos_validation_claimed') is not True:
        production_blockers.append(f'{aid} lacks external_khronos_validation_claimed true')
    if 'procedural' in str(authoring.get('method', '')).lower():
        production_blockers.append(
            f'{aid} authoring process is procedural candidate generation, not final DCC production authoring'
        )
    if e.get('kind') == 'fighters':
        for field in [
            'rig',
            'skin_weights',
            'truth_joint_mapping',
            'cosmetic_bone_separation',
            'damage_masks',
            'armor_sockets',
        ]:
            if not e.get(field):
                local_failures.append(f'{aid} fighter missing {field}')
    if e.get('kind') == 'armor':
        for field in [
            'coverage_gap_maps',
            'straps_fasteners',
            'material_layers',
            'deformation_damage_states',
            'mass_inertia_profile',
            'collision_contact_regions',
        ]:
            if not e.get(field):
                local_failures.append(f'{aid} armor missing {field}')
    if e.get('kind') == 'weapons':
        for field in [
            'grip_frames',
            'edge_point_blunt_hook_features',
            'mass_distribution',
            'moment_of_inertia_profile',
            'contact_geometry',
            'material_durability_state',
        ]:
            if not e.get(field):
                local_failures.append(f'{aid} weapon missing {field}')
    if e.get('kind') == 'arenas':
        for field in [
            'verdict_ring',
            'witness_positions',
            'oath_witness_stone',
            'lighting_anchors',
            'camera_anchors',
            'collision_footing_metadata',
            'weather_atmosphere_hooks',
        ]:
            if not e.get(field):
                local_failures.append(f'{aid} arena missing {field}')

production_candidate_manifest_claimed_complete = (
    bool(entries) and candidate_data.get('production_candidate_assets_complete') is True
)
local_asset_gate_passed = not local_failures and production_candidate_manifest_claimed_complete
high_fidelity_production_gate_passed = production_assets_complete and not production_blockers
passed = local_asset_gate_passed
manifest = {
    'schema': 'oathyard.asset_validation.v3',
    'tool': 'tools/validate_assets.sh',
    'passed': passed,
    'local_asset_gate_passed': local_asset_gate_passed,
    'high_fidelity_production_gate_passed': high_fidelity_production_gate_passed,
    'structural_asset_validation_rc': structural_rc,
    'production_assets_complete': production_assets_complete and high_fidelity_production_gate_passed,
    'production_candidate_manifest_claimed_complete': production_candidate_manifest_claimed_complete,
    'production_manifest': prod_path.as_posix(),
    'production_candidate_manifest': candidate_path.as_posix(),
    'production_renderer_complete': False,
    'owner_visual_acceptance': False,
    'public_demo_ready': False,
    'release_candidate_ready': False,
    'failed_check_count': len(local_failures),
    'failures': local_failures,
    'production_blocker_count': len(production_blockers),
    'production_blockers': production_blockers,
}
(out / 'production_asset_validation_manifest.json').write_text(
    json.dumps(manifest, indent=2, sort_keys=True) + '\n', encoding='utf-8'
)
report = [
    '# OATHYARD Asset Validation',
    '',
    f"Status: {'PASSED' if passed else 'FAILED'}",
    '',
    f'- Structural validation rc: `{structural_rc}`',
    f'- Local asset gate passed: `{str(local_asset_gate_passed).lower()}`',
    f'- Production assets complete: `{str(production_assets_complete and high_fidelity_production_gate_passed).lower()}`',
    f'- Production-candidate manifest claimed complete: `{str(production_candidate_manifest_claimed_complete).lower()}`',
    f'- High-fidelity production gate passed: `{str(high_fidelity_production_gate_passed).lower()}`',
    '- Production renderer complete: `false`',
    '- Owner visual acceptance: `false`',
    '',
]
if local_failures:
    report += ['## Local failures', ''] + [f'- {f}' for f in local_failures] + ['']
if production_blockers:
    report += ['## Production blockers', ''] + [f'- {f}' for f in production_blockers]
(out / 'asset_validation_report_v2.md').write_text('\n'.join(report) + '\n', encoding='utf-8')
if not passed:
    raise SystemExit(1)
PY