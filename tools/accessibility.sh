#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/accessibility/latest}"

cargo run --locked -- accessibility --out "$out"
