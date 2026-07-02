#!/usr/bin/env bash
set -euo pipefail

scenario="${1:-examples/duels/basic_oathyard.duel}"
out="${2:-artifacts/audio_device/latest}"
cargo run --locked -- audio-device-smoke --scenario "$scenario" --out "$out"
test -s "$out/audio_device_smoke.json"
test -s "$out/audio_device_smoke_report.md"
grep -q '"status": "PASSED_LIVE_AUDIO_DEVICE_SMOKE"' "$out/audio_device_smoke.json"
grep -q '"live_audio_device_playback_smoke_claimed": true' "$out/audio_device_smoke.json"
grep -q '"integrated_runtime_mixer_claimed": false' "$out/audio_device_smoke.json"
grep -q 'Human audible acceptance claimed: `false`' "$out/audio_device_smoke_report.md"
echo "audio device smoke passed"
