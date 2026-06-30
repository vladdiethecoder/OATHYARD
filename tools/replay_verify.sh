#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <replay.json-or-artifact-dir>" >&2
  exit 2
fi

target="$1"
if [[ -d "$target" ]]; then
  replay="$target/replay.json"
else
  replay="$target"
fi

if [[ ! -f "$replay" ]]; then
  echo "replay file not found: $replay" >&2
  exit 2
fi

cargo run --locked -- replay --replay "$replay"
