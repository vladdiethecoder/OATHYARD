#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <scenario.duel> [--out <artifact-dir>]" >&2
  exit 2
fi

scenario="$1"
shift
out="artifacts/latest"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out)
      if [[ $# -lt 2 ]]; then
        echo "--out requires a directory" >&2
        exit 2
      fi
      out="$2"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

cargo run --locked -- run --scenario "$scenario" --out "$out"
