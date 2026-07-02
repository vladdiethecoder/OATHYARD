#!/usr/bin/env bash
set -euo pipefail

# Unit-050: Native 3D game-flow planning loop driver
# Runs the deterministic truth engine, generates PresentationBricks,
# then renders native 3D captures for each game-flow phase.
# All rendering is presentation-only; truth is never mutated.

scenario="${1:-examples/duels/basic_oathyard.duel}"
out="${2:-artifacts/game_flow/latest}"
renderer_spine="${3:-}"

mkdir -p "$out" "$out/captures" "$out/logs"

echo "=== Unit-050: Native 3D Game-Flow Planning Loop ==="
echo "Scenario: $scenario"
echo "Output: $out"

# Step 1: Run deterministic truth engine
echo "--- Step 1: Truth engine ---"
./tools/run_duel.sh "$scenario" --out "$out/truth" > "$out/logs/truth_engine.log" 2>&1
TRUTH_RC=$?
if [ $TRUTH_RC -ne 0 ]; then
    echo "ERROR: Truth engine failed (rc=$TRUTH_RC)"
    exit 1
fi

# Verify replay
echo "--- Step 2: Replay verification ---"
./tools/replay_verify.sh "$out/truth/replay.json" > "$out/logs/replay_verify.log" 2>&1
REPLAY_RC=$?
if [ $REPLAY_RC -ne 0 ]; then
    echo "ERROR: Replay verification failed (rc=$REPLAY_RC)"
    exit 1
fi

# Extract final state hash
FINAL_HASH=$(python3 -c "
import json
with open('$out/truth/replay.json') as f:
    r = json.load(f)
print(r.get('final_state_hash', ''))
")
echo "Final state hash: $FINAL_HASH"

# Step 3: Generate PresentationBricks animation sequence
echo "--- Step 3: PresentationBricks ---"
./target/debug/oathyard presentation-bricks --scenario "$scenario" --out "$out/presentation_bricks" > "$out/logs/presentation_bricks.log" 2>&1
PB_RC=$?
if [ $PB_RC -ne 0 ]; then
    echo "WARNING: PresentationBricks failed (rc=$PB_RC); using default poses"
fi

# Step 4: Render wgpu spine (if not pre-rendered)
if [ -n "$renderer_spine" ] && [ -d "$renderer_spine" ]; then
    echo "--- Step 4: Using pre-rendered spine from $renderer_spine ---"
    mkdir -p "$out/renderer_spine"
    cp -r "$renderer_spine"/* "$out/renderer_spine/" 2>/dev/null || true
else
    echo "--- Step 4: Rendering wgpu spine ---"
    ./tools/wgpu_renderer_spike.sh "$scenario" "$out/renderer_spine" > "$out/logs/renderer_spine.log" 2>&1
    RENDERER_RC=$?
    if [ $RENDERER_RC -ne 0 ]; then
        echo "ERROR: Renderer spine failed (rc=$RENDERER_RC)"
        exit 1
    fi
fi

# Step 5: Generate game-flow manifest from trace data
echo "--- Step 5: Game-flow manifest ---"
python3 - "$out" "$scenario" <<'PYEOF'
import json, hashlib, os, sys
from pathlib import Path

out = Path(sys.argv[1])
scenario = sys.argv[2]

def sha256_file(path):
    h = hashlib.sha256()
    with open(path, 'rb') as f:
        for chunk in iter(lambda: f.read(65536), b''):
            h.update(chunk)
    return h.hexdigest()

# Load trace data
trace = json.loads((out / 'truth' / 'trace.json').read_text())
replay = json.loads((out / 'truth' / 'replay.json').read_text())

# Load PresentationBricks sequence if available
pb_sequence = []
pb_path = out / 'presentation_bricks' / 'presentation_bricks_sequence.json'
if pb_path.exists():
    pb_data = json.loads(pb_path.read_text())
    pb_sequence = pb_data.get('frames', [])

# Load renderer manifest
renderer_manifest = {}
rm_path = out / 'renderer_spine' / 'production_renderer_manifest.json'
if rm_path.exists():
    renderer_manifest = json.loads(rm_path.read_text())

# Map renderer captures by capture_id
renderer_captures = {}
for cap in renderer_manifest.get('captures', []):
    renderer_captures[cap.get('capture_id', '')] = cap

# Build game-flow states from trace turns
game_flow_states = []
phases = ["OBSERVE", "PLAN", "COMMIT_REVEAL", "RESOLVE", "CONSEQUENCE", "REPLAN"]

# Menu states (pre-game)
game_flow_states.append({
    "state": "boot_main_menu",
    "phase": "MENU",
    "turn": -1,
    "description": "Boot/main menu state with arena establishing view and UI panels",
    "renderer_capture_id": "boot_main_menu",
    "ui_role": "title_screen",
    "truth_hash": "",
    "truth_mutation": False,
})
game_flow_states.append({
    "state": "fighter_select",
    "phase": "MENU",
    "turn": -1,
    "description": "Fighter selection state with mannequin display and UI panels",
    "renderer_capture_id": "fighter_select",
    "ui_role": "fighter_select_panel",
    "truth_hash": "",
    "truth_mutation": False,
})
game_flow_states.append({
    "state": "loadout_select",
    "phase": "MENU",
    "turn": -1,
    "description": "Loadout selection state with armor/weapon display and UI panels",
    "renderer_capture_id": "loadout_select",
    "ui_role": "loadout_select_panel",
    "truth_hash": "",
    "truth_mutation": False,
})

# Arena establishing
game_flow_states.append({
    "state": "arena_establishing",
    "phase": "ESTABLISHING",
    "turn": 0,
    "description": "Arena establishing shot before first turn",
    "renderer_capture_id": "oathyard_verdict_ring_establishing_seed",
    "ui_role": "establishing",
    "truth_hash": replay.get("final_state_hash", ""),
    "truth_mutation": False,
})

# Per-turn game-flow states
for turn_trace in trace.get("turns", []):
    turn_idx = turn_trace.get("turn", 0)
    commits = turn_trace.get("commits", [])
    costs = turn_trace.get("costs", [])
    contacts = turn_trace.get("contacts", [])
    state_hash = turn_trace.get("state_hash", "")

    # OBSERVE phase
    game_flow_states.append({
        "state": f"turn_{turn_idx}_observe",
        "phase": "OBSERVE",
        "turn": turn_idx,
        "description": f"Turn {turn_idx}: Observe phase - fighters assess positioning",
        "renderer_capture_id": "gameplay_distance_fighter_loadout_seed" if turn_idx == 0 else "oathyard_arena_candidate_01",
        "ui_role": "observe_timeline",
        "truth_hash": state_hash,
        "commits": [c.get("label", "") for c in commits],
        "cost_count": len(costs),
        "contact_count": len(contacts),
        "truth_mutation": False,
    })

    # PLAN phase
    game_flow_states.append({
        "state": f"turn_{turn_idx}_plan",
        "phase": "PLAN",
        "turn": turn_idx,
        "description": f"Turn {turn_idx}: Planning phase - action selection and directional influence",
        "renderer_capture_id": "gameplay_distance_fighter_weapon_seed" if turn_idx == 0 else "gameplay_distance_fighter_weapon_01",
        "ui_role": "plan_action_select",
        "truth_hash": state_hash,
        "commits": [{"label": c.get("label", ""), "direction": c.get("direction", ""), "target": c.get("target", "")} for c in commits],
        "cost_preview": [{"seat": c.get("seat", 0), "action": c.get("action", ""), "cost": c.get("cost", 0)} for c in costs[:4]],
        "truth_mutation": False,
    })

    # COMMIT_REVEAL phase
    game_flow_states.append({
        "state": f"turn_{turn_idx}_commit_reveal",
        "phase": "COMMIT_REVEAL",
        "turn": turn_idx,
        "description": f"Turn {turn_idx}: Commit/reveal - both fighters' actions revealed simultaneously",
        "renderer_capture_id": "pre_contact_frame_seed" if turn_idx == 0 else "pre_contact_frame",
        "ui_role": "commit_reveal",
        "truth_hash": state_hash,
        "committed_actions": [{"seat": c.get("seat", 0), "label": c.get("label", ""), "direction": c.get("direction", "")} for c in commits],
        "truth_mutation": False,
    })

    # RESOLVE phase
    resolve_cap = "contact_frame_seed" if turn_idx == 0 else "contact_frame"
    game_flow_states.append({
        "state": f"turn_{turn_idx}_resolve",
        "phase": "RESOLVE",
        "turn": turn_idx,
        "description": f"Turn {turn_idx}: Contact resolution - physical contact resolved deterministically",
        "renderer_capture_id": resolve_cap,
        "ui_role": "resolve_contact",
        "truth_hash": state_hash,
        "contact_packets": len(contacts),
        "contacts": [{"attacker": c.get("attacker", ""), "defender": c.get("defender", ""), "result": c.get("result", "")} for c in contacts[:3]],
        "truth_mutation": False,
    })

    # CONSEQUENCE phase
    game_flow_states.append({
        "state": f"turn_{turn_idx}_consequence",
        "phase": "CONSEQUENCE",
        "turn": turn_idx,
        "description": f"Turn {turn_idx}: Consequence - injury/capability changes from contact",
        "renderer_capture_id": "fight_film_replay_camera_shot" if turn_idx == 0 else "fight_film_candidate_shot_01",
        "ui_role": "consequence_cause_chain",
        "truth_hash": state_hash,
        "cost_total": len(costs),
        "capability_changes": [{"seat": c.get("seat", 0), "action": c.get("action", ""), "balance_delta": c.get("balance_delta", 0)} for c in costs if c.get("balance_delta", 0) != 0][:4],
        "truth_mutation": False,
    })

    # REPLAN phase
    game_flow_states.append({
        "state": f"turn_{turn_idx}_replan",
        "phase": "REPLAN",
        "turn": turn_idx,
        "description": f"Turn {turn_idx}: Replan - assess new state and prepare next turn",
        "renderer_capture_id": "oathyard_arena_candidate_01",
        "ui_role": "replan_assessment",
        "truth_hash": state_hash,
        "truth_mutation": False,
    })

# Post-game: replay/fight-film
game_flow_states.append({
    "state": "replay_fight_film",
    "phase": "REPLAY",
    "turn": -2,
    "description": "Post-game replay and fight-film review",
    "renderer_capture_id": "fight_film_replay_camera_shot",
    "ui_role": "replay_timeline",
    "truth_hash": replay.get("final_state_hash", ""),
    "truth_mutation": False,
})

# Build manifest
manifest = {
    "schema": "oathyard.native_3d_game_flow_manifest.v1",
    "scenario": scenario,
    "scenario_id": trace.get("scenario_id", ""),
    "final_state_hash": replay.get("final_state_hash", ""),
    "content_hash": replay.get("content_hash", ""),
    "end_condition": trace.get("end_condition", ""),
    "end_condition_status": replay.get("end_condition_status", ""),
    "end_condition_winner": replay.get("end_condition_winner", ""),
    "truth_mutation": False,
    "presentation_only": True,
    "production_renderer_complete": False,
    "owner_visual_acceptance": False,
    "public_demo_ready": False,
    "release_candidate_ready": False,
    "game_flow_states": game_flow_states,
    "state_count": len(game_flow_states),
    "phase_coverage": phases,
    "presentation_bricks_frames": len(pb_sequence),
    "replay_path": str(out / "truth" / "replay.json"),
    "trace_path": str(out / "truth" / "trace.json"),
    "renderer_manifest_path": str(rm_path),
    "renderer_manifest_sha256": sha256_file(rm_path) if rm_path.exists() else "",
    "replay_sha256": sha256_file(out / "truth" / "replay.json"),
    "trace_sha256": sha256_file(out / "truth" / "trace.json"),
}

# Count production-seed captures
seed_captures = sum(1 for s in game_flow_states if renderer_captures.get(s.get("renderer_capture_id", ""), {}).get("capture_classification") == "production_seed_native_3d_capture")
manifest["production_seed_capture_count"] = seed_captures
manifest["production_ready_capture_count"] = 0

# Write manifest
manifest_path = out / "native_3d_game_flow_manifest.json"
manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")

# Write report
report_lines = [
    "# Unit-050: Native 3D Game-Flow Planning Loop",
    "",
    f"Scenario: `{scenario}`",
    f"Final state hash: `{replay.get('final_state_hash', '')}`",
    f"Content hash: `{replay.get('content_hash', '')}`",
    f"End condition: `{trace.get('end_condition', '')}`",
    f"Game-flow states: `{len(game_flow_states)}`",
    f"PresentationBricks frames: `{len(pb_sequence)}`",
    f"Production seed captures: `{seed_captures}`",
    f"Production ready captures: `0`",
    f"Truth mutation: `false`",
    "",
    "## Game-Flow States",
    "",
    "| State | Phase | Turn | UI Role | Truth Hash |",
    "|-------|-------|------|---------|------------|",
]
for s in game_flow_states:
    report_lines.append(f"| `{s['state']}` | `{s['phase']}` | `{s['turn']}` | `{s.get('ui_role', '')}` | `{s.get('truth_hash', '')[:16]}...` |")

report_lines.extend([
    "",
    "## Phase Coverage",
    "",
    f"Phases: `{', '.join(phases)}`",
    "",
    "## Planning UI Evidence",
    "",
    "The following states show planning/consequence UI content derived from trace data:",
    "",
])
for s in game_flow_states:
    if s["phase"] in ("PLAN", "COMMIT_REVEAL", "RESOLVE", "CONSEQUENCE"):
        ui_data = []
        if "commits" in s:
            ui_data.append(f"actions={[c.get('label','') if isinstance(c, dict) else c for c in s.get('commits', [])]}")
        if "cost_preview" in s:
            ui_data.append(f"costs={len(s['cost_preview'])}")
        if "contacts" in s:
            ui_data.append(f"contacts={len(s['contacts'])}")
        if "capability_changes" in s:
            ui_data.append(f"capability_changes={len(s['capability_changes'])}")
        report_lines.append(f"- `{s['state']}`: {'; '.join(ui_data)}")

report_lines.extend([
    "",
    "## Readiness Flags",
    "",
    "- `production_renderer_complete`: false",
    "- `owner_visual_acceptance`: false",
    "- `public_demo_ready`: false",
    "- `release_candidate_ready`: false",
    "- `truth_mutation`: false",
])

(out / "native_3d_game_flow_report.md").write_text("\n".join(report_lines) + "\n")

print(f"Game-flow manifest: {manifest_path}")
print(f"Game-flow states: {len(game_flow_states)}")
print(f"Production seed captures: {seed_captures}")
PYEOF

echo "--- Step 6: Summary ---"
python3 -c "
import json
m = json.load(open('$out/native_3d_game_flow_manifest.json'))
print(f'States: {m[\"state_count\"]}')
print(f'Final hash: {m[\"final_state_hash\"]}')
print(f'Truth mutation: {m[\"truth_mutation\"]}')
print(f'Seed captures: {m[\"production_seed_capture_count\"]}')
print(f'PB frames: {m[\"presentation_bricks_frames\"]}')
"

echo "=== Unit-050 game-flow complete: $out ==="
