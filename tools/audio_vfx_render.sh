#!/usr/bin/env bash
set -euo pipefail

scenario="${1:-examples/duels/basic_oathyard.duel}"
out="${2:-artifacts/audio_vfx/latest}"
cargo run --locked -- audio-vfx-render --scenario "$scenario" --out "$out"
test -s "$out/audio_mix.wav"
test -s "$out/audio_events.json"
test -s "$out/vfx_manifest.json"
test -s "$out/audio_vfx_timing_loudness.json"
test -s "$out/impact_vfx_contact_sheet.ppm"
test -s "$out/captions.srt"
test -s "$out/audio_vfx_report.md"
grep -q 'Status: PASSED' "$out/audio_vfx_report.md"
grep -q 'trace-derived-only' "$out/audio_events.json"
grep -q '"truth_mutation": false' "$out/audio_events.json"
grep -q '"owner_audio_acceptance_claimed": false' "$out/audio_events.json"
grep -q 'presentation_only' "$out/vfx_manifest.json"
grep -q '"owner_visual_acceptance": false' "$out/vfx_manifest.json"
grep -q '"timing_source": "truth_frame_120hz_after_hash"' "$out/audio_vfx_timing_loudness.json"
head -c 2 "$out/impact_vfx_contact_sheet.ppm" | grep -q 'P6'
head -c 4 "$out/audio_mix.wav" | grep -q 'RIFF'
echo "audio/vfx render passed"
