#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/gamepad/latest}"
cargo run --locked -- gamepad-smoke --out "$out"
