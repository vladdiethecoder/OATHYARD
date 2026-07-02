#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/asset_atlas/latest}"
mkdir -p "$out"

python3 - "$out" <<'PY'
import hashlib
import json
import sys
from pathlib import Path
root = Path.cwd()
out = Path(sys.argv[1])
manifest_path = root / 'assets/runtime_manifest.json'
required_counts = {'fighters': 6, 'weapons': 8, 'armor': 6, 'arenas': 2}

def sha(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(65536), b''):
            h.update(chunk)
    return h.hexdigest()

def read_json(path: Path):
    return json.loads(path.read_text(encoding='utf-8'))

def gltf_metrics(path: Path):
    data = read_json(path)
    vertices = 0
    indices = 0
    z_depth = 0
    for mesh in data.get('meshes', []):
        for prim in mesh.get('primitives', []):
            pos = prim.get('attributes', {}).get('POSITION')
            if isinstance(pos, int):
                vertices += int(data['accessors'][pos]['count'])
            idx = prim.get('indices')
            if isinstance(idx, int):
                indices += int(data['accessors'][idx]['count'])
    accessors = data.get('accessors', [])
    if accessors:
        lo = accessors[0].get('min', [0, 0, 0])
        hi = accessors[0].get('max', [0, 0, 0])
        if len(lo) == 3 and len(hi) == 3:
            z_depth = int(round((hi[2] - lo[2]) * 1000.0))
    return {
        'vertex_count': vertices,
        'index_count': indices,
        'triangle_count': indices // 3,
        'material_count': len(data.get('materials', [])),
        'z_depth_milli': z_depth,
    }

failures = []
entries = []
manifest = read_json(manifest_path) if manifest_path.is_file() else {}
if manifest.get('schema') != 'oathyard.assets.v1':
    failures.append('runtime manifest missing or wrong schema')
kind_counts = {}
for item in manifest.get('entries', []):
    asset_id = item.get('id', '')
    kind = item.get('kind', '')
    kind_counts[kind] = kind_counts.get(kind, 0) + 1
    source = root / item.get('source', '')
    mesh = root / item.get('runtime_mesh', '')
    gltf = root / item.get('runtime_gltf', '')
    checks = {
        'source_exists': source.is_file(),
        'runtime_mesh_exists': mesh.is_file(),
        'runtime_gltf_exists': gltf.is_file(),
        'repo_owned_provenance': item.get('provenance') == 'repo_owned_original_text_asset',
    }
    metrics = gltf_metrics(gltf) if gltf.is_file() else {}
    checks['gltf_has_3d_z_depth'] = metrics.get('z_depth_milli', 0) > 0
    for name, passed in checks.items():
        if not passed:
            failures.append(f'{asset_id}: {name}')
    entries.append({
        'id': asset_id,
        'kind': kind,
        'source': item.get('source', ''),
        'runtime_mesh': item.get('runtime_mesh', ''),
        'runtime_gltf': item.get('runtime_gltf', ''),
        'source_sha256': sha(source) if source.is_file() else '',
        'runtime_mesh_sha256': sha(mesh) if mesh.is_file() else '',
        'runtime_gltf_sha256': sha(gltf) if gltf.is_file() else '',
        'gltf_metrics': metrics,
        'checks': checks,
        'passed': all(checks.values()),
    })
for kind, minimum in required_counts.items():
    if kind_counts.get(kind, 0) < minimum:
        failures.append(f'{kind}: count {kind_counts.get(kind, 0)} below required {minimum}')
passed = not failures and all(e['passed'] for e in entries)
payload = {
    'schema': 'oathyard.asset_3d_atlas.v1',
    'tool': 'tools/asset_visual_atlas.sh',
    'passed': passed,
    'source_manifest': 'assets/runtime_manifest.json',
    'entry_count': len(entries),
    'kind_counts': kind_counts,
    'required_counts': required_counts,
    'assets_with_3d_depth': sum(1 for e in entries if e['gltf_metrics'].get('z_depth_milli', 0) > 0),
    'native_3d_render_capture': False,
    'truth_mutation': False,
    'owner_visual_acceptance_claimed': False,
    'public_demo_ready': False,
    'release_candidate_ready': False,
    'failed_check_count': len(failures),
    'failures': failures,
    'entries': entries,
}
(out / 'asset_3d_atlas_manifest.json').write_text(json.dumps(payload, indent=2, sort_keys=True) + '\n', encoding='utf-8')
(out / 'failed_asset_visuals.txt').write_text('none\n' if not failures else '\n'.join(failures) + '\n', encoding='utf-8')
report = ['# OATHYARD Asset 3D Atlas', '', f"Status: {'PASSED' if passed else 'FAILED'}", '', f"- Entries: `{len(entries)}`", f"- Assets with 3D Z depth: `{payload['assets_with_3d_depth']}`", '- Native 3D render capture: `false`', '- Truth mutation: `false`', '- Owner visual acceptance claimed: `false`', '- Public demo ready: `false`', '- Release candidate ready: `false`', '', '## Entries']
for entry in entries:
    metrics = entry['gltf_metrics']
    report.append(f"- `{entry['id']}` `{entry['kind']}` triangles `{metrics.get('triangle_count', 0)}` z-depth `{metrics.get('z_depth_milli', 0)}` glTF `{entry['runtime_gltf']}`")
if failures:
    report.extend(['', '## Failures'] + [f'- {f}' for f in failures])
(out / 'asset_3d_atlas_report.md').write_text('\n'.join(report) + '\n', encoding='utf-8')
hashes = []
for entry in entries:
    for key in ['source', 'runtime_mesh', 'runtime_gltf']:
        digest = entry.get(f'{key}_sha256', '')
        if digest:
            hashes.append(f"{digest}  {entry[key]}")
(out / 'asset_3d_atlas_hashes.sha256').write_text('\n'.join(sorted(hashes)) + '\n', encoding='utf-8')
if not passed:
    raise SystemExit(1)
print(f'asset 3D atlas passed: {len(entries)} runtime assets indexed')
PY
