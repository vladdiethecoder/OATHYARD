#!/usr/bin/env bash
set -euo pipefail

scenario="${1:-examples/duels/basic_oathyard.duel}"
out="${2:-artifacts/pbr_materials/verify}"

./tools/build.sh >/dev/null
target/debug/oathyard pbr-materials --scenario "$scenario" --out "$out"

test -s "$out/pbr_material_manifest.json"
test -s "$out/pbr_material_report.md"
test -s "$out/pbr_material_surface_atlas.ppm"
test -s "$out/pbr_material_response_sheet.ppm"
grep -q '"schema": "oathyard.pbr_material_artifacts.v1"' "$out/pbr_material_manifest.json"
grep -q '"all_required_channels_covered": true' "$out/pbr_material_manifest.json"
grep -q '"flat_recolor_rejected": true' "$out/pbr_material_manifest.json"
grep -q '"material_maps_affect_replay_hash": false' "$out/pbr_material_manifest.json"
grep -q 'Status: PASSED' "$out/pbr_material_report.md"
