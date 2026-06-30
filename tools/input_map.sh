#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/input/latest}"
cargo run --locked -- input-map --out "$out"
test -s "$out/input_map.json"
test -s "$out/input_profile.json"
test -s "$out/steam_deck_checklist.md"
test -s "$out/input_remap_report.md"
python3 -m json.tool "$out/input_map.json" >/dev/null
python3 -m json.tool "$out/input_profile.json" >/dev/null
grep -q '"schema": "oathyard.input_profile.v1"' "$out/input_profile.json"
grep -q '"all_current_screens_reachable_with_default_controller": true' "$out/input_profile.json"
grep -q '"steam_deck_local_schema_check_passed": true' "$out/input_profile.json"
grep -q '"physical_gamepad_hardware_claimed": false' "$out/input_profile.json"
grep -q '"steam_deck_hardware_claimed": false' "$out/input_profile.json"
grep -q '"owner_input_acceptance_claimed": false' "$out/input_profile.json"
grep -q '"boundary": "presentation_command_only"' "$out/input_profile.json"
for screen in main_menu mode_select settings_accessibility fighter_select loadout_select observe plan commit_reveal resolve consequence replay_browser fight_film performance_debug_overlay; do
  grep -q "\"screen\": \"${screen}\"" "$out/input_profile.json"
done
for action in main_menu_start settings_accessibility fighter_select observe plan resolve replay_browser fight_film performance_debug_overlay; do
  grep -q "\"action\": \"${action}\"" "$out/input_map.json"
done
grep -q 'PASSED_LOCAL_INPUT_SCHEMA' "$out/steam_deck_checklist.md"
grep -q 'Steam Deck hardware claimed: `false`' "$out/steam_deck_checklist.md"
grep -q 'Owner input acceptance claimed: `false`' "$out/steam_deck_checklist.md"
