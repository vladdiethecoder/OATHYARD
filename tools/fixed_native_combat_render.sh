#!/usr/bin/env bash
set -euo pipefail

source="${1:-examples/duels/basic_oathyard.duel}"
out="${2:-artifacts/native_combat/latest}"

# Create output directory
mkdir -p "$out"

# Run the main render command
cargo run --locked -- native-combat-render --scenario "$source" --out "$out"

# Verify generated files with proper quoting
test -s "$out/native_combat_render_report.md"
test -s "$out/native_combat_render.ppm"
test -s "$out/native_combat_render_1280x720.ppm"
test -s "$out/native_combat_render_1280x800.ppm"
test -s "$out/native_combat_render_1920x1080.ppm"
test -s "$out/native_product_fighter_select_1920x1080.ppm"
test -s "$out/native_product_verdict_ring_1920x1080.ppm"
test -s "$out/native_product_pre_contact_1920x1080.ppm"
test -s "$out/native_product_contact_1920x1080.ppm"
test -s "$out/native_product_material_closeup_1920x1080.ppm"
test -s "$out/native_product_injury_consequence_1920x1080.ppm"

echo "Fixed native combat render completed successfully"
