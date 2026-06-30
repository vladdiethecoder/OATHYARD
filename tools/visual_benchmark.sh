#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/visual_review/latest}"
mkdir -p "$out"

python3 - "$out" <<'PY'
import json
import sys
from pathlib import Path

out = Path(sys.argv[1]); root = Path.cwd()
gap_manifest = out / 'visual_gap_audit.json'
contact_sheet = out / 'v0_current_visual_baseline_contact_sheet.png'
sections = [
    ('current_oathyard_captures', 'FAIL', 'Current capture sheet is made from PPM/SVG/raw-X11/software/native_combat/debug-local artifacts. These are useful deterministic evidence and explicitly failing visual baseline.'),
    ('elden_ring_atmosphere_world_detail_goals', 'FAIL', 'No production renderer, GI/reflection solution, high-quality shadows, cinematic atmosphere, weather/wetness, large-scale dark-fantasy environment richness, or material-depth evidence exists.'),
    ('for_honor_melee_readability_goals', 'FAIL', 'Current fighters/weapons are low-poly structural silhouettes; no high-fidelity skinned characters, armor/weapon identity, visceral contact response, or best-of-five presentation packet exists.'),
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
    'baseline_contact_sheet': contact_sheet.as_posix() if contact_sheet.is_file() else '',
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
    'Explicit baseline statement: current PPM/SVG/raw-X11/software/debug/native_combat/primitive/low-poly renders are failing baselines for the final visual target. They do not satisfy production visual quality, high-fidelity 3D, native public demo readiness, owner visual acceptance, Elden Ring-class atmosphere, or For Honor-class melee presentation.', '',
    'Forbidden readiness statements remain false:', '',
    '- Production renderer complete: `false`',
    '- Owner visual acceptance: `false`',
    '- Public demo ready: `false`',
    '- Release candidate ready: `false`',
    '- Elden Ring quality achieved: `false`',
    '- For Honor quality achieved: `false`', '',
    '## Side-by-side contact sheet of current OATHYARD captures', '',
    f"- Contact sheet: `{manifest['baseline_contact_sheet'] or 'missing; run ./tools/visual_gap_audit.sh first'}`", '',
    '## Benchmark sections', '',
    '| Section | Verdict | Review |', '| --- | --- | --- |',
]
for sid, verdict, summary in sections:
    report.append(f'| `{sid}` | `{verdict}` | {summary} |')
report.extend(['', '## Next fixes', '',
    '1. Unblock a functional DCC/source-asset toolchain: current `/usr/bin/blender` fails with a MaterialX symbol lookup error.',
    '2. Execute the V1 renderer/backend spike or superseding ADR with measured native 3D renderer evidence and truth isolation.',
    '3. Create `assets/production_visual_manifest.json` with real source files, license/provenance, rigs/skin weights/material maps/runtime exports/previews/in-engine screenshots.',
    '4. Generate the complete 1920x1080+ capture matrix with native production renderer and production assets loaded.',
    '5. Re-run `./tools/presentation_truth_isolation.sh`, `./tools/validate_assets.sh`, `./tools/capture_high_fidelity_screens.sh`, and this benchmark.',
    '6. Submit the packet for owner/human visual review; until then owner visual acceptance remains pending/false.',
])
(out / 'visual_benchmark_report.md').write_text('\n'.join(report) + '\n', encoding='utf-8')
raise SystemExit(1)
PY

echo "visual benchmark: $out"
