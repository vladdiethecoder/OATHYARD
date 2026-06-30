#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/ai/latest}"
turns="${2:-6}"

cargo run --locked -- ai-duel --out "$out" --turns "$turns"
