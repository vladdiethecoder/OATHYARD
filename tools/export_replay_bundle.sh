#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <replay.json> <out-dir>" >&2
  exit 2
fi

replay="$1"
out="$2"

cargo run --locked --bin oathyard -- export-bundle --replay "$replay" --out "$out"
