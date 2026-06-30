#!/usr/bin/env bash
set -euo pipefail

scenario="${1:-examples/duels/basic_oathyard.duel}"
out="${2:-artifacts/native_combat/latest}"

mkdir -p "$out"

cargo run --locked -- native-combat-render --scenario "$scenario" --out "$out"

test -s "$out/native_combat_render_report.md"
test -s "$out/native_combat_render_manifest.json"
test -s "$out/native_combat_visual_audit.md"
test -s "$out/native_combat_contact_sheet.svg"
test -s "$out/native_renderer_post_hash_input.json"
test -s "$out/native_renderer_capture_hook.json"
test -s "$out/native_renderer_backend_manifest.json"
test -s "$out/native_renderer_truth_mutation_proof.json"
test -s "$out/native_renderer_truth_writeback_audit.md"
test -s "$out/native_production_renderer_manifest.json"
test -s "$out/native_production_renderer_report.md"

test -s "$out/native_combat_render.ppm"
test -s "$out/native_combat_render_1280x720.ppm"
test -s "$out/native_combat_render_1280x800.ppm"
test -s "$out/native_combat_render_1920x1080.ppm"

test -s "$out/native_combat_3d_first_person.ppm"
test -s "$out/native_combat_3d_third_person.ppm"
test -s "$out/native_combat_3d_planning.ppm"
test -s "$out/native_combat_3d_consequence.ppm"
test -s "$out/native_combat_3d_fight_film.ppm"
test -s "$out/native_combat_3d_asset_closeup.ppm"

test -s "$out/native_product_fighter_select_1920x1080.ppm"
test -s "$out/native_product_verdict_ring_1920x1080.ppm"
test -s "$out/native_product_pre_contact_1920x1080.ppm"
test -s "$out/native_product_contact_1920x1080.ppm"
test -s "$out/native_product_material_closeup_1920x1080.ppm"
test -s "$out/native_product_injury_consequence_1920x1080.ppm"
test -s "$out/native_product_fight_film_1920x1080.ppm"

OATHYARD_NATIVE_COMBAT_OUT="$out" python3 - <<'PY'
import json
import os
from pathlib import Path

out = Path(os.environ["OATHYARD_NATIVE_COMBAT_OUT"])

required_modes = {
    "first_person_guard_line",
    "third_person_verdict_ring",
    "planning_tactical_reach",
    "consequence_aftermath_dwell",
    "fight_film_orbit",
    "asset_closeup_weapon_armor",
}

required_clean_files = {
    "native_product_fighter_select_1920x1080.ppm",
    "native_product_verdict_ring_1920x1080.ppm",
    "native_product_pre_contact_1920x1080.ppm",
    "native_product_contact_1920x1080.ppm",
    "native_product_material_closeup_1920x1080.ppm",
    "native_product_injury_consequence_1920x1080.ppm",
    "native_product_fight_film_1920x1080.ppm",
}

manifest = json.loads((out / "native_combat_render_manifest.json").read_text(encoding="utf-8"))
mutation_proof = json.loads((out / "native_renderer_truth_mutation_proof.json").read_text(encoding="utf-8"))
backend = json.loads((out / "native_renderer_backend_manifest.json").read_text(encoding="utf-8"))
production = json.loads((out / "native_production_renderer_manifest.json").read_text(encoding="utf-8"))

assert manifest["renderer"] == "native-software-3d"
assert manifest["source"] == "truth-after-hash-duel-result"
assert manifest["verified_replay_trace_input"]["replay_verified"] is True
assert manifest["truth_mutation"] is False
assert manifest["truth_writeback_guard"]["runtime_assertion"] == "native_renderer_assert_truth_unchanged"
assert manifest["product_mode_clean_capture_count"] >= len(required_clean_files)

camera_modes = {entry["camera"] for entry in manifest["camera_metadata"]}
assert required_modes.issubset(camera_modes), sorted(required_modes - camera_modes)

clean = manifest["product_mode_clean_captures"]
clean_files = {cap["file"] for cap in clean}
clean_modes = {cap["camera"] for cap in clean}
assert required_clean_files.issubset(clean_files), sorted(required_clean_files - clean_files)
assert required_modes.issubset(clean_modes), sorted(required_modes - clean_modes)
for cap in clean:
    assert cap["capture_after_truth_hash"] is True
    assert cap["truth_mutation"] is False
    assert cap["presentation_only"] is True
    assert cap["debug_overlay"] is False
    assert cap["production_asset_evidence"] is True
    assert cap["camera_metadata"]["camera_mode"] == cap["camera"]
    assert cap["camera_metadata"]["replay_source_hash"] == cap["replay_source_hash"]
    assert cap["camera_metadata"]["timestamp_ms"] == cap["timestamp_ms"]
    assert cap["frame_hash"]

for capture in manifest["resolution_captures"]:
    assert capture["capture_after_truth_hash"] is True
    assert capture["truth_mutation"] is False
    assert capture["presentation_only"] is True

assert mutation_proof["all_equal"] is True
assert mutation_proof["truth_mutation"] is False
assert mutation_proof["changed_fields"] == []

assert backend["truth_mutation"] is False
assert backend["mutation_proof_all_equal"] is True

assert production["truth_mutation"] is False
assert production["source"] == "current-run-replay-and-trace-after-truth-hash"
assert production["frame_hash_chain"]
assert any(
    capture["stream"] == "production_renderer_state_capture"
    for capture in production["captures"]
)
PY

echo "native combat render verified: $out"