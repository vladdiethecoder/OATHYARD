#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/truth_stress/latest}"

# Create output directory
mkdir -p "$out"

# Run the main command
cargo run --locked -- truth-stress --out "$out"

# Verify generated files with proper quoting
test -s "$out/truth_stress.json"
test -s "$out/truth_stress_report.md"

echo "Fixed truth stress completed successfully"
