#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <bundle-dir>" >&2
  exit 2
fi

bundle="$1"

cargo run --locked --bin oathyard -- verify-bundle --bundle "$bundle"
