#!/usr/bin/env bash
set -euo pipefail

scenario="${1:-examples/duels/basic_oathyard.duel}"
out="${2:-artifacts/runtime_asset_sets/latest}"
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
renderer_bin="$repo_root/crates/oathyard_renderer/target/debug/oathyard-native-renderer"

mkdir -p "$out" "$out/truth" "$out/rendered_sets" "$out/mesh_manifests"

echo "=== OATHYARD coherent runtime asset sets ==="
echo "Scenario: $scenario"
echo "Output: $out"

if [[ ! -x "$renderer_bin" ]]; then
  cargo build --manifest-path "$repo_root/crates/oathyard_renderer/Cargo.toml"
fi

echo "--- Step 1: truth/replay packet ---"
"$repo_root/tools/run_duel.sh" "$scenario" --out "$out/truth" > "$out/truth_engine.log" 2>&1
"$repo_root/tools/replay_verify.sh" "$out/truth/replay.json" > "$out/replay_verify.log" 2>&1

python3 - "$out" "$scenario" <<'PY'
import hashlib, json, sys
from pathlib import Path
out = Path(sys.argv[1])
scenario = sys.argv[2]
truth = out / 'truth'

def sha(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(65536), b''):
            h.update(chunk)
    return h.hexdigest()

replay = json.loads((truth / 'replay.json').read_text(encoding='utf-8'))
trace = json.loads((truth / 'trace.json').read_text(encoding='utf-8'))
packet = {
    'schema': 'oathyard.post_hash_presentation_packet.v1',
    'source': 'tools/render_runtime_asset_sets.sh after run_duel + replay_verify',
    'scenario_path': scenario,
    'scenario_id': trace.get('scenario_id') or replay.get('scenario_canonical') or 'unknown',
    'content_hash': trace.get('content_hash') or replay.get('content_hash'),
    'final_state_hash': trace.get('final_state_hash') or replay.get('final_state_hash'),
    'end_condition': trace.get('end_condition'),
    'end_condition_status': replay.get('end_condition_status'),
    'end_condition_winner': replay.get('end_condition_winner'),
    'replay_json': str(truth / 'replay.json'),
    'trace_json': str(truth / 'trace.json'),
    'replay_json_sha256': sha(truth / 'replay.json'),
    'trace_json_sha256': sha(truth / 'trace.json'),
    'duel_report_sha256': sha(truth / 'duel_report.md'),
    'generated_after_replay_verify': True,
    'presentation_only': True,
    'truth_mutation': False,
    'renderer_consumption_layer': 'runtime_presentation',
}
(out / 'post_hash_presentation_packet.json').write_text(json.dumps(packet, indent=2, sort_keys=True) + '\n', encoding='utf-8')
PY

echo "--- Step 2: generate coherent mesh manifests ---"
index_path="$("$repo_root/tools/generate_runtime_asset_sets.py" "$out/mesh_manifests")"

asset_manifest_sha="$(python3 - <<'PY'
import hashlib
from pathlib import Path
path = Path('assets/manifests/production_candidate_visual_manifest.json')
h = hashlib.sha256()
with path.open('rb') as f:
    for chunk in iter(lambda: f.read(65536), b''):
        h.update(chunk)
print(h.hexdigest())
PY
)"

echo "--- Step 3: render sets ---"
python3 - "$index_path" <<'PY' > "$out/render_plan.tsv"
import json, sys
from pathlib import Path
index = json.loads(Path(sys.argv[1]).read_text(encoding='utf-8'))
for row in index['sets']:
    print(f"{row['asset_set_id']}\t{row['manifest']}\t{row['candidate_assets']}")
PY

while IFS=$'\t' read -r set_id manifest candidate_assets; do
  set_out="$out/rendered_sets/$set_id"
  mkdir -p "$set_out"
  "$renderer_bin" \
    --packet "$out/post_hash_presentation_packet.json" \
    --out "$set_out" \
    --capture-id "unit081_asset_set_${set_id}" \
    --capture-file-stem "production_renderer_asset_set_${set_id}_1920x1080" \
    --camera-mode "pre_contact_frame" \
    --candidate-assets "$candidate_assets" \
    --asset-manifest-sha256 "$asset_manifest_sha" \
    --mesh-manifest-json "$manifest" \
    > "$set_out/renderer.log" 2>&1
  test -s "$set_out/production_renderer_asset_set_${set_id}_1920x1080.png"
done < "$out/render_plan.tsv"

python3 - "$out" <<'PY'
import hashlib, json, sys
from pathlib import Path
out = Path(sys.argv[1])
rows = []
renderer_captures = []
packet = json.loads((out / 'post_hash_presentation_packet.json').read_text(encoding='utf-8'))
for png in sorted((out / 'rendered_sets').glob('*/production_renderer_asset_set_*_1920x1080.png')):
    h = hashlib.sha256(png.read_bytes()).hexdigest()
    production_manifest = json.loads((png.parent / 'production_renderer_manifest.json').read_text(encoding='utf-8'))
    capture = production_manifest.get('capture', {})
    mesh_assets = production_manifest.get('mesh_assets', [])
    mesh_asset_ids = [m.get('mesh_asset_id', '') for m in mesh_assets]
    row = {
        'asset_set_id': png.parent.name,
        'capture_file': png.as_posix(),
        'capture_file_sha256': h,
        'renderer_manifest': (png.parent / 'production_renderer_manifest.json').as_posix(),
        'mesh_asset_count': production_manifest.get('mesh_asset_count', 0),
        'mesh_asset_ids': mesh_asset_ids,
    }
    rows.append(row)
    renderer_captures.append({
        'capture_id': production_manifest.get('capture', {}).get('capture_id', f"unit081_asset_set_{png.parent.name}"),
        'capture_classification': 'runtime_asset_set_candidate_native_3d_capture',
        'asset_set_id': png.parent.name,
        'capture_file': png.relative_to(out).as_posix(),
        'file': png.relative_to(out).as_posix(),
        'native_3d_capture': True,
        'truth_mutation': False,
        'renderer_backend_id': production_manifest.get('backend_id', 'oathyard-native-wgpu-production-v1'),
        'renderer_build_hash_or_binary_hash': production_manifest.get('frame_hash_chain', h),
        'quality_preset': 'unit081_runtime_asset_set_candidate_not_production_ready',
        'replay_path': packet.get('replay_json', ''),
        'replay_final_hash': packet.get('final_state_hash', ''),
        'content_manifest_hash': packet.get('content_hash', ''),
        'asset_manifest_hash': production_manifest.get('asset_manifest_sha256', ''),
        'camera_mode': capture.get('camera_mode', 'pre_contact_frame'),
        'frame_or_tick': 'post_hash_static_frame',
        'mesh_geometry_consumed': production_manifest.get('mesh_geometry_consumed') is True,
        'mesh_asset_count': production_manifest.get('mesh_asset_count', 0),
        'mesh_asset_ids': mesh_asset_ids,
    })
manifest = {
    'schema': 'oathyard.runtime_asset_sets.render_manifest.v1',
    'source': 'tools/render_runtime_asset_sets.sh',
    'truth_mutation': False,
    'production_ready': False,
    'rendered_set_count': len(rows),
    'captures': rows,
}
(out / 'runtime_asset_sets_render_manifest.json').write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n', encoding='utf-8')
renderer_manifest = {
    'schema': 'oathyard.production_renderer_manifest.v1',
    'source': 'tools/render_runtime_asset_sets.sh aggregate runtime asset-set candidate evidence',
    'native_3d_render_capture': True,
    'fallback_visual_substitutes_allowed': False,
    'truth_mutation': False,
    'production_renderer_complete': False,
    'owner_visual_acceptance': False,
    'public_demo_ready': False,
    'release_candidate_ready': False,
    'runtime_asset_set_candidate_capture_count': len(renderer_captures),
    'runtime_asset_sets_render_manifest': (out / 'runtime_asset_sets_render_manifest.json').as_posix(),
    'captures': renderer_captures,
}
(out / 'production_renderer_manifest.json').write_text(json.dumps(renderer_manifest, indent=2, sort_keys=True) + '\n', encoding='utf-8')
print(json.dumps(manifest, indent=2, sort_keys=True))
PY

echo "=== Runtime asset-set render complete: $out ==="
