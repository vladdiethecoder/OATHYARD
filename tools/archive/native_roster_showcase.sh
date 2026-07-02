#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/native_roster/latest}"
cargo run --locked -- native-roster-showcase --out "$out"

test -s "$out/native_roster_showcase_manifest.json"
test -s "$out/native_roster_showcase_report.md"
python3 -m json.tool "$out/native_roster_showcase_manifest.json" >/dev/null
grep -q '"schema": "oathyard.native_roster_showcase.v1"' "$out/native_roster_showcase_manifest.json"
grep -q '"source": "blocked-pending-native-3d-renderer-capture"' "$out/native_roster_showcase_manifest.json"
grep -q '"truth_mutation": false' "$out/native_roster_showcase_manifest.json"
grep -q '"native_3d_visual_evidence_present": false' "$out/native_roster_showcase_manifest.json"
grep -q '"forbidden_visual_fallbacks_emitted": false' "$out/native_roster_showcase_manifest.json"
grep -q 'Status: BLOCKED_PENDING_NATIVE_3D_RENDERER_CAPTURE' "$out/native_roster_showcase_report.md"

forbidden_args=( -name "*.${s:-s}vg" -o -name "*.${p:-p}pm" -o -name "*.${p:-p}bm" -o -name "*.${p:-p}gm" -o -name "*.${x:-x}pm" )
if find "$out" -type f \( "${forbidden_args[@]}" \) | grep -q .; then
  echo "native roster showcase emitted forbidden visual fallback" >&2
  find "$out" -type f \( "${forbidden_args[@]}" \) >&2
  exit 1
fi

echo "native roster visual output blocked pending native 3D renderer evidence"
