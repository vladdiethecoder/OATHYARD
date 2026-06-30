#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/native_roster/latest}"
cargo run --locked -- native-roster-showcase --out "$out"

test -s "$out/native_roster_showcase_manifest.json"
test -s "$out/native_roster_showcase_report.md"
test -s "$out/native_roster_showcase_contact_sheet.svg"
for frame in \
  01_saltreach_duelist \
  02_oathyard_writ \
  03_chainbreaker \
  04_reed_sentinel \
  05_gate_shield \
  06_bruiser_oath; do
  test -s "$out/native_roster_showcase_${frame}.ppm"
done

python3 -m json.tool "$out/native_roster_showcase_manifest.json" >/dev/null
grep -q '"schema": "oathyard.native_roster_showcase.v1"' "$out/native_roster_showcase_manifest.json"
grep -q '"source": "fighter-tradition-default-loadouts-runtime-gltf-after-content-hash"' "$out/native_roster_showcase_manifest.json"
grep -q '"fighter_tradition_count": 6' "$out/native_roster_showcase_manifest.json"
grep -q '"frame_count": 6' "$out/native_roster_showcase_manifest.json"
grep -q '"all_default_loadouts_rendered": true' "$out/native_roster_showcase_manifest.json"
grep -q '"all_frames_depth_sorted": true' "$out/native_roster_showcase_manifest.json"
grep -q '"all_frames_shaded_triangles": true' "$out/native_roster_showcase_manifest.json"
grep -q '"projection_model": "integer_depth_sorted_mesh_raster"' "$out/native_roster_showcase_manifest.json"
grep -q '"truth_mutation": false' "$out/native_roster_showcase_manifest.json"
grep -q '"game_is_3d": true' "$out/native_roster_showcase_manifest.json"
grep -q '"product_3d_gameplay_complete": false' "$out/native_roster_showcase_manifest.json"
grep -q '"continuous_player_facing_3d_render_loop": false' "$out/native_roster_showcase_manifest.json"
grep -q '"owner_visual_acceptance_claimed": false' "$out/native_roster_showcase_manifest.json"
grep -q '"public_demo_ready": false' "$out/native_roster_showcase_manifest.json"
grep -q '"release_candidate_ready": false' "$out/native_roster_showcase_manifest.json"
for id in saltreach_duelist oathyard_writ chainbreaker reed_sentinel gate_shield bruiser_oath; do
  grep -q "\"fighter_id\": \"$id\"" "$out/native_roster_showcase_manifest.json"
done
for id in curved_sword longsword bearded_axe ash_spear round_shield iron_maul; do
  grep -q "\"weapon_id\": \"$id\"" "$out/native_roster_showcase_manifest.json"
done
for id in fencer_light mail_hauberk lamellar gambeson heavy_plate bruiser_padded_plate; do
  grep -q "\"armor_id\": \"$id\"" "$out/native_roster_showcase_manifest.json"
done

grep -q 'Status: PASSED' "$out/native_roster_showcase_report.md"
grep -q 'Fighter traditions rendered: `6`' "$out/native_roster_showcase_report.md"
grep -q 'All default loadouts rendered: `true`' "$out/native_roster_showcase_report.md"
grep -q 'Projection model: `integer_depth_sorted_mesh_raster`' "$out/native_roster_showcase_report.md"
grep -q 'Owner visual acceptance claimed: `false`' "$out/native_roster_showcase_report.md"
grep -q '<svg' "$out/native_roster_showcase_contact_sheet.svg"
grep -q 'OATHYARD native roster 3D showcase contact sheet' "$out/native_roster_showcase_contact_sheet.svg"
grep -q 'all six default loadout families rendered from runtime glTF after content hash' "$out/native_roster_showcase_contact_sheet.svg"
grep -q 'owner visual acceptance not claimed' "$out/native_roster_showcase_contact_sheet.svg"

python3 - "$out" <<'PY'
import json
import sys
from pathlib import Path

out = Path(sys.argv[1])
manifest = json.loads((out / "native_roster_showcase_manifest.json").read_text(encoding="utf-8"))
frames = manifest["frames"]
assert len(frames) == 6
assert len({frame["fighter_id"] for frame in frames}) == 6
assert len({frame["weapon_id"] for frame in frames}) == 6
assert len({frame["armor_id"] for frame in frames}) == 6
assert all(frame["width"] == 960 and frame["height"] == 540 for frame in frames)
assert all(frame["triangle_count"] >= 90 for frame in frames)
assert all(frame["shaded_triangle_count"] >= 80 for frame in frames)
assert all(frame["non_background_pixels"] > 100000 for frame in frames)
assert all(frame["source"] == "default-loadout-runtime-gltf-after-content-hash" for frame in frames)
assert all(frame["depth_sorted"] is True for frame in frames)
for frame in frames:
    data = (out / frame["file"]).read_bytes()
    assert data.startswith(b"P6\n960 540\n255\n")
    assert len(data) == 1555215
PY

echo "native roster showcase passed"
