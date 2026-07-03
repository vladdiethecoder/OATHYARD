#!/usr/bin/env bash
set -euo pipefail

scenario="${1:-examples/duels/basic_oathyard.duel}"
out="${2:-artifacts/native_combat/latest}"

mkdir -p "$out"

cargo run --locked -- native-combat-render --scenario "$scenario" --out "$out"

test -s "$out/native_combat_render_report.md"
test -s "$out/native_combat_render_manifest.json"
test -s "$out/native_combat_visual_audit.md"
test -s "$out/native_capture_input_replay.json"
python3 -m json.tool "$out/native_combat_render_manifest.json" >/dev/null

# Unit-070: Require promoted schema for current-run evidence.
# The promoted schema indicates real native 3D renderer capture was produced.
if grep -q '"native_3d_visual_evidence_present":true' "$out/native_combat_render_manifest.json"; then
  PROMOTED=1
elif grep -q '"native_3d_visual_evidence_present":false' "$out/native_combat_render_manifest.json"; then
  BLOCKED=1
else
  echo "ERROR: neither promoted nor blocked evidence flag found in manifest" >&2
  exit 1
fi

grep -q '"source":"truth-after-hash-duel-result"' "$out/native_combat_render_manifest.json"
grep -q '"truth_mutation":false' "$out/native_combat_render_manifest.json"
grep -q '"forbidden_visual_fallbacks_emitted":false' "$out/native_combat_render_manifest.json"

if [[ "${BLOCKED:-0}" -gt 0 ]]; then
  grep -q '"native_3d_visual_evidence_present":false' "$out/native_combat_render_manifest.json"
  echo "WARNING: native combat visual output is BLOCKED — renderer did not produce capture evidence" >&2
fi

if [[ "${PROMOTED:-0}" -gt 0 ]]; then
  grep -q '"native_3d_visual_evidence_present":true' "$out/native_combat_render_manifest.json"
  grep -q '"visual_evidence_status":"native_3d_renderer_capture_present"' "$out/native_combat_render_manifest.json"
  # Verify a real PNG capture exists and is non-trivial in size
  capture_png=$(find "$out/render" -name "production_renderer_*.png" -type f | head -1)
  if [[ -z "$capture_png" ]]; then
    echo "ERROR: promoted manifest but no production_renderer_*.png capture found" >&2
    exit 1
  fi
  capture_size=$(stat -c%s "$capture_png" 2>/dev/null || stat -f%z "$capture_png" 2>/dev/null)
  if [[ "$capture_size" -lt 50000 ]]; then
    echo "ERROR: capture PNG suspiciously small ($capture_size bytes): $capture_png" >&2
    exit 1
  fi
  echo "native combat 3D capture promoted: $out"
  echo "  capture: $capture_png ($capture_size bytes)"
fi

# Verify no forbidden visual fallback artifacts were emitted
forbidden_args=( -name "*.svg" -o -name "*.ppm" -o -name "*.pbm" -o -name "*.pgm" -o -name "*.xpm" )
if find "$out" -type f \( "${forbidden_args[@]}" \) | grep -q .; then
  echo "native combat render emitted forbidden visual fallback" >&2
  find "$out" -type f \( "${forbidden_args[@]}" \) >&2
  exit 1
fi
