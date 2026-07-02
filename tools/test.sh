#!/usr/bin/env bash
set -euo pipefail

cargo test --locked
./tools/test_visual_artifact_audit.sh
