#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/native_asset_capture_matrix/latest}"
scenario="${2:-examples/duels/basic_oathyard.duel}"

python3 tools/unit083_native_asset_matrix.py capture "$out" --scenario "$scenario"
