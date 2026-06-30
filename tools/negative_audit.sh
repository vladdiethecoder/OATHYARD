#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/negative_audit/verify}"

./tools/build.sh >/dev/null
target/debug/oathyard negative-audit --out "$out"

python3 -m json.tool "$out/negative_input_audit.json" >/dev/null
grep -q '"schema": "oathyard.negative_input_audit.v1"' "$out/negative_input_audit.json"
grep -q '"truth_hz": 120' "$out/negative_input_audit.json"
grep -q '"case_count": 13' "$out/negative_input_audit.json"
grep -q '"all_failed_loudly": true' "$out/negative_input_audit.json"
grep -q '"all_cases_passed": true' "$out/negative_input_audit.json"
grep -q '"public_demo_ready": false' "$out/negative_input_audit.json"
grep -q '"release_candidate_ready": false' "$out/negative_input_audit.json"
grep -q '"id": "scenario_unknown_action_fails_loud"' "$out/negative_input_audit.json"
grep -q '"id": "scenario_unknown_weapon_fails_loud"' "$out/negative_input_audit.json"
grep -q '"id": "content_manifest_schema_fails_loud"' "$out/negative_input_audit.json"
grep -q '"id": "content_manifest_readiness_true_fails_loud"' "$out/negative_input_audit.json"
grep -q '"id": "content_manifest_missing_rows_fails_loud"' "$out/negative_input_audit.json"
grep -q '"id": "replay_mismatched_final_hash_fails_loud"' "$out/negative_input_audit.json"
grep -q '"id": "export_bundle_tamper_fails_loud"' "$out/negative_input_audit.json"
grep -q 'unknown action label' "$out/negative_input_audit.json"
grep -q 'content manifest schema mismatch' "$out/negative_input_audit.json"
grep -q 'public_demo_ready must remain false' "$out/negative_input_audit.json"
grep -q 'fighters count 0 below required 6' "$out/negative_input_audit.json"
grep -q 'final state hash mismatch' "$out/negative_input_audit.json"
grep -q 'export bundle hash mismatch' "$out/negative_input_audit.json"
grep -q 'Status: PASSED' "$out/negative_input_audit_report.md"
grep -q 'All failed loudly: `true`' "$out/negative_input_audit_report.md"

echo "negative input audit passed"
