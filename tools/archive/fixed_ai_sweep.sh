#!/usr/bin/env bash
set -euo pipefail

# Fix the quoted path issue by using proper variable handling
out="${1:-artifacts/ai_sweep/latest}"

# Create output directory
mkdir -p "$out"

# Run the main command
cargo run --locked -- ai-sweep --out "$out"

# Verify generated files with proper quoting
test -s "$out/ai_sweep.json"
test -s "$out/ai_sweep_report.md"

echo "Fixed AI sweep completed successfully"
