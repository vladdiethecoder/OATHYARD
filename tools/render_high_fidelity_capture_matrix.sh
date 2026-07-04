#!/usr/bin/env bash
set -euo pipefail

scenario="${1:-examples/duels/basic_oathyard.duel}"
out="${2:-artifacts/high_fidelity_capture_matrix/latest}"
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
renderer_bin="$repo_root/crates/oathyard_renderer/target/debug/oathyard-native-renderer"
slots_manifest="$repo_root/content/high_fidelity_capture_slots.json"

mkdir -p "$out" "$out/truth" "$out/rendered_slots" "$out/mesh_manifests" "$out/logs"
rm -rf "$out/rendered_slots" "$out/mesh_manifests/runtime_meshes"
mkdir -p "$out/rendered_slots" "$out/mesh_manifests"

if [[ ! -f "$slots_manifest" ]]; then
  echo "missing canonical capture slot manifest: $slots_manifest" >&2
  exit 1
fi
if [[ ! -x "$renderer_bin" ]]; then
  cargo build --locked --manifest-path "$repo_root/crates/oathyard_renderer/Cargo.toml"
fi

"$repo_root/tools/run_duel.sh" "$scenario" --out "$out/truth" > "$out/logs/truth_engine.log" 2>&1
"$repo_root/tools/replay_verify.sh" "$out/truth/replay.json" > "$out/logs/replay_verify.log" 2>&1

python3 - "$out" "$scenario" <<'PY'
import hashlib,json,sys
from pathlib import Path
out=Path(sys.argv[1]); scenario=sys.argv[2]; truth=out/'truth'
def sha(p):
    h=hashlib.sha256()
    with p.open('rb') as f:
        for c in iter(lambda:f.read(65536),b''): h.update(c)
    return h.hexdigest()
replay=json.loads((truth/'replay.json').read_text())
trace=json.loads((truth/'trace.json').read_text())
packet={
 'schema':'oathyard.post_hash_presentation_packet.v1',
 'source':'tools/render_high_fidelity_capture_matrix.sh after run_duel + replay_verify',
 'scenario_path':scenario,
 'scenario_id':trace.get('scenario_id') or replay.get('scenario_canonical') or 'unknown',
 'content_hash':trace.get('content_hash') or replay.get('content_hash'),
 'final_state_hash':trace.get('final_state_hash') or replay.get('final_state_hash'),
 'end_condition':trace.get('end_condition'),
 'end_condition_status':replay.get('end_condition_status'),
 'end_condition_winner':replay.get('end_condition_winner'),
 'replay_json':str(truth/'replay.json'),
 'trace_json':str(truth/'trace.json'),
 'replay_json_sha256':sha(truth/'replay.json'),
 'trace_json_sha256':sha(truth/'trace.json'),
 'duel_report_sha256':sha(truth/'duel_report.md'),
 'generated_after_replay_verify':True,
 'presentation_only':True,
 'truth_mutation':False,
 'renderer_consumption_layer':'runtime_presentation',
}
(out/'post_hash_presentation_packet.json').write_text(json.dumps(packet,indent=2,sort_keys=True)+'\n')
PY

index_path="$("$repo_root/tools/generate_runtime_asset_sets.py" "$out/mesh_manifests")"
asset_manifest_sha="$(python3 - <<'PY'
import hashlib
from pathlib import Path
path=Path('assets/manifests/production_candidate_visual_manifest.json')
h=hashlib.sha256()
with path.open('rb') as f:
    for c in iter(lambda:f.read(65536),b''): h.update(c)
print(h.hexdigest())
PY
)"

python3 - "$slots_manifest" "$index_path" <<'PY' > "$out/render_plan.tsv"
import json,sys
from pathlib import Path
slots=json.loads(Path(sys.argv[1]).read_text())['slots']
index=json.loads(Path(sys.argv[2]).read_text())
sets={r['asset_set_id']:r for r in index['sets']}
for slot in slots:
    row=sets[slot['asset_set_id']]
    print('\t'.join([slot['slot_id'], slot['camera'], row['manifest'], row['candidate_assets'], slot['asset_set_id'], slot['game_state_or_capture_role'], slot['required_ui_state']]))
PY

while IFS=$'\t' read -r slot_id camera_mode manifest candidate_assets asset_set role ui_state; do
  slot_out="$out/rendered_slots/$slot_id"
  mkdir -p "$slot_out"
  "$renderer_bin" \
    --packet "$out/post_hash_presentation_packet.json" \
    --out "$slot_out" \
    --capture-id "$slot_id" \
    --capture-file-stem "production_renderer_${slot_id}_1920x1080" \
    --camera-mode "$camera_mode" \
    --candidate-assets "$candidate_assets" \
    --asset-manifest-sha256 "$asset_manifest_sha" \
    --mesh-manifest-json "$manifest" \
    > "$slot_out/renderer.log" 2>&1
  test -s "$slot_out/production_renderer_${slot_id}_1920x1080.png"
done < "$out/render_plan.tsv"

python3 - "$out" "$slots_manifest" <<'PY'
import hashlib,json,struct,sys
from pathlib import Path
out=Path(sys.argv[1]); slots_manifest=Path(sys.argv[2])
slots=json.loads(slots_manifest.read_text())['slots']
packet=json.loads((out/'post_hash_presentation_packet.json').read_text())
def sha(p):
    h=hashlib.sha256()
    with p.open('rb') as f:
        for c in iter(lambda:f.read(65536),b''): h.update(c)
    return h.hexdigest()
def dims(p):
    b=p.read_bytes()[:24]
    if not b.startswith(b'\x89PNG\r\n\x1a\n') or b[12:16] != b'IHDR': raise ValueError('not png')
    return list(struct.unpack('>II', b[16:24]))
captures=[]; slot_rows=[]; failures=[]
for slot in slots:
    sid=slot['slot_id']; png=out/'rendered_slots'/sid/f'production_renderer_{sid}_1920x1080.png'; manifest_path=png.parent/'production_renderer_manifest.json'
    try:
        prod=json.loads(manifest_path.read_text())
        resolution=dims(png)
    except Exception as exc:
        failures.append(f'{sid}: render output invalid: {exc}')
        prod={}; resolution=[0,0]
    mesh_assets=prod.get('mesh_assets', []) if isinstance(prod, dict) else []
    mesh_ids=[m.get('mesh_asset_id','') for m in mesh_assets if isinstance(m,dict)]
    mesh_classes=[m.get('mesh_asset_class','') for m in mesh_assets if isinstance(m,dict)]
    material_summaries=[m.get('material_texture_summary',{}) for m in mesh_assets if isinstance(m,dict)]
    lighting=prod.get('visual_features',{}) if isinstance(prod.get('visual_features',{}),dict) else {}
    row={
      **slot,
      'png_path': png.relative_to(out).as_posix(),
      'absolute_png_path': png.resolve().as_posix(),
      'sha256': sha(png) if png.is_file() else '',
      'resolution': resolution,
      'renderer_backend': prod.get('backend_id','oathyard-native-wgpu-production-v1'),
      'renderer_manifest': manifest_path.relative_to(out).as_posix(),
      'mesh_geometry_consumed': prod.get('mesh_geometry_consumed') is True,
      'mesh_asset_count': prod.get('mesh_asset_count',0),
      'mesh_asset_ids': mesh_ids,
      'mesh_asset_classes': mesh_classes,
      'material_texture_status': 'present' if all(m.get('material_texture_binding') is True for m in material_summaries) and material_summaries else 'missing',
      'material_texture_summaries': material_summaries,
      'lighting_status': 'present' if lighting.get('dynamic_key_fill_rim_lighting') and lighting.get('contact_shadows_ao_equivalent') else 'missing',
      'lighting': lighting,
      'truth_hash': packet.get('final_state_hash',''),
      'replay_path': packet.get('replay_json',''),
      'replay_hash': packet.get('replay_json_sha256',''),
      'truth_mutation': False,
      'native_3d_visual_evidence_present': png.is_file(),
      'production_visual_candidate': True,
      'production_visual_seed': False,
      'high_fidelity_production': False,
      'production_renderer_complete': False,
      'owner_visual_acceptance': False,
      'public_demo_ready': False,
      'release_candidate_ready': False,
      'legal_clearance': False,
      'trademark_clearance': False,
      'store_readiness': False,
      'status': 'production_candidate_current_run_native_3d_capture' if png.is_file() else 'missing_current_run_capture',
      'blockers': ['production_renderer_complete_false_until_visual_qa_benchmark_renderer_target_pass','owner_visual_acceptance_false','public_demo_ready_false','release_candidate_ready_false'],
    }
    slot_rows.append(row)
    captures.append({
      'capture_id': sid,
      'capture_classification':'production_visual_capture_matrix_current_run',
      'current_run_capture_matrix_slot': True,
      'slot_metadata': row,
      'capture_file': row['png_path'],
      'file': row['png_path'],
      'native_3d_capture': png.is_file(),
      'truth_mutation': False,
      'renderer_backend_id': row['renderer_backend'],
      'renderer_build_hash_or_binary_hash': prod.get('frame_hash_chain',row['sha256']),
      'quality_preset':'unit082_current_run_production_candidate_not_owner_accepted',
      'replay_path': row['replay_path'],
      'replay_final_hash': row['truth_hash'],
      'content_manifest_hash': packet.get('content_hash',''),
      'asset_manifest_hash': prod.get('asset_manifest_sha256',''),
      'camera_mode': slot['camera'],
      'frame_or_tick':'post_hash_static_frame',
      'mesh_geometry_consumed': row['mesh_geometry_consumed'],
      'mesh_asset_count': row['mesh_asset_count'],
      'mesh_asset_ids': row['mesh_asset_ids'],
      'material_texture_status': row['material_texture_status'],
      'lighting_status': row['lighting_status'],
      'ui_state': slot['required_ui_state'],
      'production_renderer_complete': False,
      'owner_visual_acceptance': False,
      'public_demo_ready': False,
      'release_candidate_ready': False,
    })
summary={
 'schema':'oathyard.unit082.high_fidelity_capture_matrix_render.v1',
 'source':'tools/render_high_fidelity_capture_matrix.sh',
 'slot_manifest': slots_manifest.as_posix(),
 'truth_mutation': False,
 'native_3d_render_capture': True,
 'fallback_visual_substitutes_allowed': False,
 'production_renderer_complete': False,
 'owner_visual_acceptance': False,
 'public_demo_ready': False,
 'release_candidate_ready': False,
 'current_run_capture_matrix_slot_count': len(slot_rows),
 'valid_png_slot_count': sum(1 for r in slot_rows if r['native_3d_visual_evidence_present'] and r['resolution']==[1920,1080]),
 'high_fidelity_production_slot_count': 0,
 'production_candidate_slot_count': len(slot_rows),
 'failures': failures,
 'slots': slot_rows,
 'captures': captures,
}
(out/'high_fidelity_capture_matrix_manifest.json').write_text(json.dumps(summary,indent=2,sort_keys=True)+'\n')
(out/'production_renderer_manifest.json').write_text(json.dumps({k:summary[k] for k in ['schema','source','truth_mutation','native_3d_render_capture','fallback_visual_substitutes_allowed','production_renderer_complete','owner_visual_acceptance','public_demo_ready','release_candidate_ready','current_run_capture_matrix_slot_count','valid_png_slot_count','high_fidelity_production_slot_count','production_candidate_slot_count','failures','captures']},indent=2,sort_keys=True)+'\n')
md=['# Unit-082 High-Fidelity Capture Matrix Render','',f'- slots: `{len(slot_rows)}`',f'- valid_png_slot_count: `{summary["valid_png_slot_count"]}`','- high_fidelity_production_slot_count: `0`','- production_renderer_complete: `false`','', '| Slot | Status | PNG | SHA256 |','| --- | --- | --- | --- |']
for r in slot_rows:
    md.append(f"| `{r['slot_id']}` | `{r['status']}` | `{r['png_path']}` | `{r['sha256']}` |")
(out/'high_fidelity_capture_slot_table.md').write_text('\n'.join(md)+'\n')
print(json.dumps({'slots':len(slot_rows),'valid_png':summary['valid_png_slot_count'],'out':out.as_posix()},sort_keys=True))
if failures:
    raise SystemExit(1)
PY

echo "high-fidelity capture matrix rendered: $out"
