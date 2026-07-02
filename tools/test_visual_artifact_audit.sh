#!/usr/bin/env bash
set -euo pipefail

tmp="$(mktemp -d)"
fixture_dir="artifacts/native_combat/verify"
fixture="$fixture_dir/audit_forbidden_rollup_fixture.txt"
cleanup() {
  rm -f "$fixture"
  rm -rf "$tmp"
}
trap cleanup EXIT

mkdir -p "$fixture_dir"
bad_term="visual "'composite'
printf '%s\n' "$bad_term fallback should fail closed" > "$fixture"

set +e
./tools/audit_visual_artifacts.sh "$tmp/negative" >"$tmp/negative.log" 2>&1
rc=$?
set -e
if [[ "$rc" -eq 0 ]]; then
  echo "expected audit to reject forbidden generated fixture" >&2
  cat "$tmp/negative.log" >&2
  exit 1
fi
python3 - "$tmp/negative/visual_artifact_audit.json" <<'PY'
import json
import sys
payload = json.load(open(sys.argv[1], encoding='utf-8'))
assert payload['passed'] is False, payload
assert payload['forbidden_visual_artifact_count'] >= 1, payload
assert any('audit_forbidden_rollup_fixture.txt' in item for item in payload['violations']), payload['violations']
PY

rm -f "$fixture"
./tools/audit_visual_artifacts.sh "$tmp/positive" >"$tmp/positive.log" 2>&1
python3 - "$tmp/positive/visual_artifact_audit.json" <<'PY'
import json
import sys
payload = json.load(open(sys.argv[1], encoding='utf-8'))
assert payload['passed'] is True, payload
assert payload['forbidden_visual_artifact_count'] == 0, payload
PY

echo "visual artifact audit negative/positive tests passed"
