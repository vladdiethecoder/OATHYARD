#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/contact_matrix/latest}"
cargo run --locked -- contact-matrix --out "$out"
python3 -m json.tool "$out/contact_matrix.json" >/dev/null
grep -q '"schema": "oathyard.contact_matrix.v1"' "$out/contact_matrix.json"
grep -q '"combinations": 1344' "$out/contact_matrix.json"
grep -q '"contacts": 1344' "$out/contact_matrix.json"
grep -q '"invalid_actions": 0' "$out/contact_matrix.json"
grep -q '"invariants_passed": true' "$out/contact_matrix.json"
grep -q '"id": "all_material_result_classes_present"' "$out/contact_matrix.json"
grep -q '"id": "mail_cut_blunt_transfer_slows_recovery"' "$out/contact_matrix.json"
grep -q '"id": "weapon_arm_gap_penetration_compromises_grip"' "$out/contact_matrix.json"
grep -q '"id": "hook_bind_reduces_torque_and_grip"' "$out/contact_matrix.json"
grep -q '"id": "blunt_transfer_breaks_stance"' "$out/contact_matrix.json"
grep -q '"id": "deflection_still_applies_posture_shock"' "$out/contact_matrix.json"
grep -q '"id": "low_coverage_transfers_capability_loss"' "$out/contact_matrix.json"
grep -q '"id": "physical_costs_vary_from_base"' "$out/contact_matrix.json"
grep -q '"torque_delta":' "$out/contact_matrix.json"
grep -q '"invalidates_thrust": true' "$out/contact_matrix.json"
grep -q 'Status: PASSED' "$out/contact_matrix_report.md"
grep -q '## Invariants' "$out/contact_matrix_report.md"
grep -q 'weapon-arm gap penetration applies severe right-grip loss' "$out/contact_matrix_report.md"
