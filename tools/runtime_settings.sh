#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/settings/latest}"

cargo run --locked -- runtime-settings --out "$out"

test -s "$out/runtime_settings.default.json"
test -s "$out/runtime_settings.saved.json"
test -s "$out/runtime_settings.loaded.json"
test -s "$out/runtime_settings_report.md"

python3 -m json.tool "$out/runtime_settings.default.json" >/dev/null
python3 -m json.tool "$out/runtime_settings.saved.json" >/dev/null
python3 -m json.tool "$out/runtime_settings.loaded.json" >/dev/null
cmp "$out/runtime_settings.saved.json" "$out/runtime_settings.loaded.json"

grep -q '"schema": "oathyard.runtime_settings.v1"' "$out/runtime_settings.saved.json"
grep -q '"presentation_only": true' "$out/runtime_settings.saved.json"
grep -q '"truth_mutation": false' "$out/runtime_settings.saved.json"
grep -q '"replay_hash_affects": false' "$out/runtime_settings.saved.json"
grep -q '"uses_wall_clock": false' "$out/runtime_settings.saved.json"
grep -q '"hidden_rng": false' "$out/runtime_settings.saved.json"
grep -q '"text_scale_permille": 1400' "$out/runtime_settings.saved.json"
grep -q '"master_gain_permille": 720' "$out/runtime_settings.saved.json"
grep -q '"hold_to_commit": true' "$out/runtime_settings.saved.json"
grep -q '"toggle_guard": true' "$out/runtime_settings.saved.json"
grep -q '"public_demo_ready": false' "$out/runtime_settings.saved.json"
grep -q '"release_candidate_ready": false' "$out/runtime_settings.saved.json"
grep -q 'Status: PASSED' "$out/runtime_settings_report.md"
grep -q 'Roundtrip byte exact: `true`' "$out/runtime_settings_report.md"
grep -q 'Truth mutation: `none`' "$out/runtime_settings_report.md"
grep -q 'Replay hash affects: `false`' "$out/runtime_settings_report.md"

echo "runtime settings persistence passed"
