#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/visual_review/latest}"
mkdir -p "$out"

python3 - "$out" <<'PY'
import json
import sys
from pathlib import Path

out = Path(sys.argv[1]); root = Path.cwd()

def first_existing(paths):
    for path in paths:
        if path.is_file():
            return path
    return None

gap_manifest = first_existing([
    out / 'visual_gap_audit.json',
    out.parent / 'visual_gap' / 'visual_gap_audit.json',
    root / 'artifacts' / 'visual_review' / 'latest' / 'visual_gap_audit.json',
])
capture_matrix = first_existing([
    out / 'high_fidelity_capture_matrix.json',
    out.parent / 'high_fidelity_screens' / 'high_fidelity_capture_matrix.json',
    root / 'artifacts' / 'high_fidelity_screens' / 'latest' / 'high_fidelity_capture_matrix.json',
])
capture_matrix_data = {}
if capture_matrix:
    try:
        capture_matrix_data = json.loads(capture_matrix.read_text(encoding='utf-8'))
    except Exception as exc:
        capture_matrix_data = {'load_error': str(exc)}
required_capture_slot_count = int(capture_matrix_data.get('required_capture_slot_count', 0) or 0)
missing_native_capture_count = int(capture_matrix_data.get('missing_native_capture_count', 0) or 0)
current_native_capture_count = int(capture_matrix_data.get('current_native_capture_count', 0) or 0)
runtime_asset_set_candidate_capture_count = int(capture_matrix_data.get('runtime_asset_set_candidate_capture_count', 0) or 0)
production_seed_native_capture_count = int(capture_matrix_data.get('production_seed_native_capture_count', 0) or 0)
sections = [
    ('current_oathyard_captures', 'FAIL', 'Current native-renderer evidence is absent or blocked; standalone fallback captures are excluded from the current visual benchmark surface.' + (f' Runtime asset-set progress: {runtime_asset_set_candidate_capture_count} coherent local Meshy/Rodin native 3D captures exist but are candidate evidence only.' if runtime_asset_set_candidate_capture_count > 0 else '') + (f' Production seed progress: {production_seed_native_capture_count} source-approved native 3D captures exist but are not production-ready.' if production_seed_native_capture_count > 0 else '')),
    ('elden_ring_atmosphere_world_detail_goals', 'FAIL', 'No production renderer, GI/reflection solution, high-quality shadows, cinematic atmosphere, weather/wetness, large-scale dark-fantasy environment richness, or material-depth evidence exists.'),
    ('for_honor_melee_readability_goals','FAIL','Current fighters/weapons are candidate/procedural structural assets; no source-approved high-fidelity DCC production characters, armor/weapon identity, visceral contact response, or best-of-five presentation packet exists.'),
    ('original_art_ip_safety_check', 'PASS', 'Docs/canon define benchmark references as quality bars only and prohibit copying third-party names, assets, silhouettes, UI, lore, animations, music, textures, or proprietary mechanics. Production asset provenance remains absent.'),
    ('material_quality_review', 'FAIL', 'Current PBR/equivalent material data is source-text/local-atlas evidence, not production material maps on high-fidelity meshes with albedo/normal/roughness/metallic/AO/detail/emissive/blood/dirt/damage evidence.'),
    ('lighting_atmosphere_review', 'FAIL', 'No production dynamic lighting, shadowing, GI/probe/baked hybrid, reflection, fog, dust, smoke, mist, weather, exposure/tone-map, or DoF evidence.'),
    ('character_armor_weapon_closeup_review', 'FAIL', 'No source-backed production rigged fighter closeups, layered armor closeups, or detailed weapon-family in-engine closeups.'),
    ('animation_pose_credibility_review', 'FAIL', 'PresentationBricks/debug motion evidence is post-truth and useful, but no production retargeted render skeleton, skin deformation, cloth/hair/armor secondary presentation, or closeup pose credibility packet exists.'),
    ('ui_readability_review', 'FAIL', 'Existing UI captures are debug/local product-flow evidence, below production capture matrix and not routed through a high-fidelity native 3D presentation stack.'),
    ('combat_contact_readability_review', 'FAIL', 'Contact/injury/capability events exist in truth/replay evidence, but high-fidelity readable contact frames, armor response, blood/material damage, and fight-film cinematic shots are absent.'),
    ('performance_notes', 'FAIL', 'No production renderer frame-time distribution, GPU timing, memory, loading, or package-size delta has been measured. Truth timing remains separate.'),
    ('remaining_visual_gaps', 'FAIL', 'Production renderer, production assets, capture matrix, in-engine closeups, benchmark packet, and owner/human review are missing.'),
]
blocking = [sid for sid, verdict, _ in sections if verdict == 'FAIL']
manifest = {
    'schema': 'oathyard.visual_benchmark.v2',
    'tool': 'tools/visual_benchmark.sh',
    'passed': False,
    'candidate_evidence_package': True,
    'current_fidelity_tier': 'Tier 0 / failing baseline',
    'source_visual_gap_audit': gap_manifest.as_posix() if gap_manifest else '',
    'source_capture_matrix': capture_matrix.as_posix() if capture_matrix else '',
    'required_capture_slot_count': required_capture_slot_count,
    'missing_native_capture_count': missing_native_capture_count,
    'current_native_capture_count': current_native_capture_count,
    'runtime_asset_set_candidate_capture_count': runtime_asset_set_candidate_capture_count,
    'production_seed_native_capture_count': production_seed_native_capture_count,
    'native_3d_visual_evidence_required': True,
    'fallback_visual_substitutes_allowed': False,
    'production_renderer_complete': False,
    'owner_visual_acceptance': False,
    'public_demo_ready': False,
    'release_candidate_ready': False,
    'sections': [{'id': sid, 'verdict': verdict, 'summary': summary} for sid, verdict, summary in sections],
    'blocking_sections': blocking,
}
(out / 'visual_benchmark_manifest.json').write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n', encoding='utf-8')
(out / 'failed_visual_benchmark_criteria.txt').write_text('\n'.join(blocking) + '\n', encoding='utf-8')
report = [
    '# OATHYARD Visual Benchmark Report', '',
    'Status: FAILED',
    'Evidence class: candidate evidence package only.', '',
    'Explicit baseline statement: current non-production render artifacts are failing baselines for the final visual target. Standalone fallback previews are excluded from the visual benchmark surface. Current captures do not satisfy production visual quality, high-fidelity 3D, native public demo readiness, owner visual acceptance, Elden Ring-class atmosphere, or For Honor-class melee presentation.', '',
    'Forbidden readiness statements remain false:', '',
    '- Production renderer complete: `false`',
    '- Owner visual acceptance: `false`',
    '- Public demo ready: `false`',
    '- Release candidate ready: `false`',
    '- Elden Ring quality achieved: `false`',
    '- For Honor quality achieved: `false`', '',
    'Capture matrix integration:', '',
    f'- Source capture matrix: `{capture_matrix.as_posix() if capture_matrix else "missing"}`',
    f'- Required high-fidelity capture slots: `{required_capture_slot_count}`',
    f'- Current native capture slots: `{current_native_capture_count}`',
    f'- Runtime asset-set candidate captures: `{runtime_asset_set_candidate_capture_count}`',
    f'- Production seed native capture slots: `{production_seed_native_capture_count}`',
    f'- Missing native capture slots: `{missing_native_capture_count}`',
    '- Fallback visual substitutes: `forbidden`', '',
    '## Benchmark sections', '',
    '| Section | Verdict | Review |', '| --- | --- | --- |',
]
for sid, verdict, summary in sections:
    report.append(f'| `{sid}` | `{verdict}` | {summary} |')
report.extend(['', '## Next fixes', '',
    '1. Use the now-working Blender 4.3.2 or OpenUSD/Godot/Bevy path to create real DCC/interchange production source assets; candidate JSON/glTF remains insufficient.',
    '2. Execute the V1 renderer/backend spike or superseding ADR with measured native 3D renderer evidence and truth isolation.',
    '3. Populate `assets/manifests/production_visual_manifest.json` only with real source-approved production assets; keep candidate-only assets in `assets/manifests/production_candidate_visual_manifest.json`.',
    '4. Generate the complete 1920x1080+ capture matrix with native production renderer and production assets loaded.',
    '5. Re-run `./tools/presentation_truth_isolation.sh`, `./tools/validate_assets.sh`, `./tools/capture_high_fidelity_screens.sh`, and this benchmark.',
    '6. Submit the packet for owner/human visual review; until then owner visual acceptance remains pending/false.',
])
(out / 'visual_benchmark_report.md').write_text('\n'.join(report) + '\n', encoding='utf-8')
gap_lines = [
    '# OATHYARD Visual Gap List',
    '',
    'Status: FAILED',
    '',
    'Blocking high-fidelity visual gaps:',
    '',
    f'- Required high-fidelity capture slots: `{required_capture_slot_count}`',
    f'- Current native capture slots: `{current_native_capture_count}`',
    f'- Runtime asset-set candidate captures: `{runtime_asset_set_candidate_capture_count}`',
    f'- Production seed native capture slots: `{production_seed_native_capture_count}`',
    f'- Missing native capture slots: `{missing_native_capture_count}`',
    '- Fallback visual substitutes: `forbidden`',
    '',
]
for sid, verdict, summary in sections:
    if verdict == 'FAIL':
        gap_lines.append(f'- `{sid}`: {summary}')
gap_lines.extend(['', 'Readiness flags remain false:', '', '- Public demo ready: `false`', '- Release candidate ready: `false`', '- Owner visual accepted: `false`'])
(out / 'visual_gap_list.md').write_text('\n'.join(gap_lines) + '\n', encoding='utf-8')
raise SystemExit(1)
PY

echo "visual benchmark: $out"
