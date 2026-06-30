#!/usr/bin/env bash
set -euo pipefail

python3 tools/asset_pipeline.py build

if [[ -f assets/presentation_manifest.json ]]; then
  python3 tools/production_visual_manifest.py
fi

python3 - <<'PY'
import json
from pathlib import Path
manifest = {
    'schema': 'oathyard.production_asset_gap.v1',
    'tool': 'tools/build_assets.sh',
    'generated_runtime_assets': True,
    'production_visual_assets_complete': False,
    'production_candidate_visual_assets_complete': Path('assets/production_candidate_visual_manifest.json').is_file(),
    'production_renderer_complete': False,
    'owner_visual_acceptance': False,
    'public_demo_ready': False,
    'release_candidate_ready': False,
    'notes': [
        'Default build_assets regenerates only deterministic low-poly/source-text runtime assets.',
        'The t_73291be5 model-candidate integration is intentionally not run by default because it is a production-candidate lane, can hang under concurrent sessions, and cannot satisfy final high-fidelity DCC-authored asset gates.',
        'Production visual assets remain blocked until DCC/interchange source files, external validation, final material/rig/capture evidence, and owner review exist.'
    ],
}
Path('assets/production_asset_gap.json').write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n', encoding='utf-8')
Path('assets/production_asset_gap_report.md').write_text('\n'.join([
    '# OATHYARD Production Asset Gap', '',
    'Status: FAILING BASELINE / FINAL PRODUCTION ASSETS BLOCKED', '',
    '- Runtime regression assets regenerated: `true`',
    '- Production visual assets complete: `false`',
    '- Production renderer complete: `false`',
    '- Owner visual acceptance: `false`',
    '- t_73291be5 integration default-run: `false`',
]) + '\n', encoding='utf-8')
PY
