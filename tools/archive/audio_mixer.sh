#!/usr/bin/env bash
set -euo pipefail

scenario="${1:-examples/duels/basic_oathyard.duel}"
out="${2:-artifacts/audio_mixer/latest}"
cargo run --locked -- audio-mixer --scenario "$scenario" --out "$out"
test -s "$out/runtime_audio_mix.wav"
test -s "$out/audio_mixer_settings.json"
test -s "$out/audio_mixer_channels.json"
test -s "$out/audio_mixer_loudness.json"
test -s "$out/audio_mixer_report.md"
test -s "$out/captions.srt"
grep -q '"schema": "oathyard.audio_mixer.v1"' "$out/audio_mixer_settings.json"
grep -q '"integrated_runtime_mixer_claimed": true' "$out/audio_mixer_settings.json"
grep -q '"human_audible_acceptance_claimed": false' "$out/audio_mixer_settings.json"
grep -q '"truth_mutation": false' "$out/audio_mixer_settings.json"
grep -q '"schema": "oathyard.audio_mixer_channels.v1"' "$out/audio_mixer_channels.json"
grep -q '"bus": "impact"' "$out/audio_mixer_channels.json"
grep -q '"schema": "oathyard.audio_mixer_loudness.v1"' "$out/audio_mixer_loudness.json"
grep -q '"peak_permille":' "$out/audio_mixer_loudness.json"
grep -q '"limited_sample_count":' "$out/audio_mixer_loudness.json"
grep -q 'Status: PASSED' "$out/audio_mixer_report.md"
grep -q 'Integrated runtime mixer claimed: `true`' "$out/audio_mixer_report.md"
grep -q 'Human audible acceptance claimed: `false`' "$out/audio_mixer_report.md"
head -c 4 "$out/runtime_audio_mix.wav" | grep -q 'RIFF'
python3 -m json.tool "$out/audio_mixer_settings.json" >/dev/null
python3 -m json.tool "$out/audio_mixer_channels.json" >/dev/null
python3 -m json.tool "$out/audio_mixer_loudness.json" >/dev/null
echo "audio mixer passed"
