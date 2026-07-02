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
grep -q '"schema": "oathyard.native_3d_visual_blocked.v1"' "$out/native_combat_render_manifest.json"
grep -q '"source": "truth-after-hash-duel-result"' "$out/native_combat_render_manifest.json"
grep -q '"truth_mutation": false' "$out/native_combat_render_manifest.json"
grep -q '"native_3d_visual_evidence_present": false' "$out/native_combat_render_manifest.json"
grep -q '"forbidden_visual_fallbacks_emitted": false' "$out/native_combat_render_manifest.json"
grep -q 'Status: BLOCKED_PENDING_NATIVE_3D_RENDERER_CAPTURE' "$out/native_combat_render_report.md"

forbidden_args=( -name "*.${s:-s}vg" -o -name "*.${p:-p}pm" -o -name "*.${p:-p}bm" -o -name "*.${p:-p}gm" -o -name "*.${x:-x}pm" )
if find "$out" -type f \( "${forbidden_args[@]}" \) | grep -q .; then
  echo "native combat render emitted forbidden visual fallback" >&2
  find "$out" -type f \( "${forbidden_args[@]}" \) >&2
  exit 1
fi

echo "native combat visual output blocked pending native 3D renderer evidence: $out"
