#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/truth_edge/verify}"

./tools/build.sh >/dev/null
target/debug/oathyard truth-edge-audit --out "$out"

python3 -m json.tool "$out/truth_edge_audit.json" >/dev/null
grep -q '"schema": "oathyard.truth_edge_audit.v1"' "$out/truth_edge_audit.json"
grep -q '"truth_hz": 120' "$out/truth_edge_audit.json"
grep -q '"fixed_point_scale": 1000' "$out/truth_edge_audit.json"
grep -q '"overflow_policy": "i128_intermediate_then_saturate_or_clamp"' "$out/truth_edge_audit.json"
grep -q '"replay_schema": "oathyard.replay.v1"' "$out/truth_edge_audit.json"
grep -q '"hidden_rng": false' "$out/truth_edge_audit.json"
grep -q '"wall_clock": false' "$out/truth_edge_audit.json"
grep -q '"gameplay_floats": false' "$out/truth_edge_audit.json"
grep -q '"unordered_truth_iteration": false' "$out/truth_edge_audit.json"
grep -q '"all_edge_cases_passed": true' "$out/truth_edge_audit.json"
grep -q '"id": "permille_positive_overflow_saturates"' "$out/truth_edge_audit.json"
grep -q '"id": "fixed_ratio_zero_denominator_saturates"' "$out/truth_edge_audit.json"
grep -q '"id": "capability_lower_clamp_and_validity"' "$out/truth_edge_audit.json"
grep -q '"id": "contact_tie_breaker_signature"' "$out/truth_edge_audit.json"
grep -q '"id": "unsupported_schema_fails_loud"' "$out/truth_edge_audit.json"
grep -q '"id": "missing_required_field_fails_loud"' "$out/truth_edge_audit.json"
grep -q '"id": "mismatched_final_hash_fails_loud"' "$out/truth_edge_audit.json"
grep -q '"message": "verification error: replay schema mismatch:' "$out/truth_edge_audit.json"
grep -q '"message": "verification error: replay missing scenario_canonical"' "$out/truth_edge_audit.json"
grep -q '"message": "verification error: final state hash mismatch:' "$out/truth_edge_audit.json"
grep -q 'Status: PASSED' "$out/truth_edge_audit_report.md"
grep -q 'Overflow policy: `i128_intermediate_then_saturate_or_clamp`' "$out/truth_edge_audit_report.md"
grep -q 'replay `unsupported_schema_fails_loud`' "$out/truth_edge_audit_report.md"

echo "truth edge audit passed"
