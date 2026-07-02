#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/perf/latest}"
python3 tools/performance_benchmark.py "$out"
