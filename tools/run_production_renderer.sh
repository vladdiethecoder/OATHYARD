#!/usr/bin/env bash
set -euo pipefail

# Unit-055: Production native 3D renderer wrapper.
# This is the production renderer path — it uses crates/oathyard_renderer/
# which lives outside spikes/ and carries production branding.
#
# The legacy spike path (tools/wgpu_renderer_spike.sh) remains as a
# compatibility wrapper that delegates to this tool.

scenario="${1:-examples/duels/basic_oathyard.duel}"
out="${2:-artifacts/production_renderer/latest}"

mkdir -p "$out" "$out/render"

echo "=== OATHYARD Production Native 3D Renderer ==="
echo "Scenario: $scenario"
echo "Output: $out"

# Step 1: Run deterministic truth engine
echo "--- Step 1: Truth engine ---"
./tools/run_duel.sh "$scenario" --out "$out/truth_presentation_disabled" > "$out/truth_engine.log" 2>&1
TRUTH_RC=$?
if [ $TRUTH_RC -ne 0 ]; then
    echo "ERROR: Truth engine failed (rc=$TRUTH_RC)"
    exit 1
fi

# Step 2: Verify replay
echo "--- Step 2: Replay verification ---"
./tools/replay_verify.sh "$out/truth_presentation_disabled/replay.json" > "$out/replay_verify.log" 2>&1
REPLAY_RC=$?
if [ $REPLAY_RC -ne 0 ]; then
    echo "ERROR: Replay verification failed (rc=$REPLAY_RC)"
    exit 1
fi

# Step 3: Generate post-hash presentation packet
echo "--- Step 3: Post-hash presentation packet ---"
python3 - "$out" "$scenario" <<'PY'
import hashlib, json, sys
from pathlib import Path
out = Path(sys.argv[1])
scenario = sys.argv[2]
truth = out / 'truth_presentation_disabled'

def sha(path: Path) -> str:
    h = hashlib.sha256()
    with path.open('rb') as f:
        for chunk in iter(lambda: f.read(65536), b''):
            h.update(chunk)
    return h.hexdigest()

def read_json(path: Path):
    return json.loads(path.read_text(encoding='utf-8'))

replay = read_json(truth / 'replay.json')
trace = read_json(truth / 'trace.json')
packet = {
    'schema': 'oathyard.post_hash_presentation_packet.v1',
    'source': 'tools/run_production_renderer.sh after tools/run_duel.sh and tools/replay_verify.sh',
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

# Step 4: Run the production renderer
echo "--- Step 4: Production renderer capture ---"
asset_manifest_sha="$(python3 -c "
import hashlib
from pathlib import Path
path = Path('assets/manifests/production_candidate_visual_manifest.json')
h = hashlib.sha256()
with path.open('rb') as f:
    for chunk in iter(lambda: f.read(65536), b''):
        h.update(chunk)
print(h.hexdigest())
")"

cargo run --locked --manifest-path crates/oathyard_renderer/Cargo.toml -- \
    --packet "$out/post_hash_presentation_packet.json" \
    --out "$out/render" \
    --capture-id "production_native_default" \
    --capture-file-stem "production_renderer_native_1920x1080" \
    --camera-mode "oathyard_verdict_ring_establishing" \
    --candidate-assets "oathyard_verdict_ring,fighter_mannequin,longsword" \
    --asset-manifest-sha256 "$asset_manifest_sha"

echo "=== Production renderer complete: $out ==="
