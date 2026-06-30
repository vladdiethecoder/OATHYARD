#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/truth_stress/latest}"

cargo run --locked -- truth-stress --out "$out"

test -s "$out/truth_stress.json"
test -s "$out/truth_stress_report.md"
test -s "$out/reach_vs_mail/run_a/replay.json"
test -s "$out/reach_vs_mail/run_b/replay.json"
test -s "$out/hook_vs_plate/run_a/trace.json"
test -s "$out/maul_vs_fencer/run_b/replay.json"
test -s "$out/spear_vs_maul_pressure/run_a/duel_report.md"

python3 -m json.tool "$out/truth_stress.json" >/dev/null
python3 - <<PY
import json
from pathlib import Path

root = Path("$out")
stress = json.loads((root / "truth_stress.json").read_text())
assert stress["schema"] == "oathyard.truth_stress.v1"
assert stress["truth_hz"] == 120
assert stress["hidden_rng"] is False
assert stress["wall_clock"] is False
assert stress["gameplay_floats"] is False
assert stress["runs_per_pairing"] == 2
assert stress["pairing_count"] == 6
assert stress["stress_turn_count"] == 24
assert stress["minimum_total_contacts_required"] == 72
assert stress["minimum_capability_reactions_required"] == 150
assert stress["minimum_capability_stops_required"] == 4
assert stress["minimum_distinct_final_hashes_required"] == 5
assert stress["minimum_recovery_slowdown_required"] == 32
assert stress["maximum_min_balance_required"] == 100
assert stress["maximum_min_grip_r_required"] == 100
assert stress["maximum_min_torque_required"] == 100
assert stress["total_contacts"] >= stress["minimum_total_contacts_required"]
assert stress["capability_reaction_count"] >= stress["minimum_capability_reactions_required"]
assert stress["capability_stop_count"] >= stress["minimum_capability_stops_required"]
assert stress["distinct_final_hash_count"] >= stress["minimum_distinct_final_hashes_required"]
assert stress["max_recovery_slowdown_frames"] >= stress["minimum_recovery_slowdown_required"]
assert stress["min_balance_permille"] <= stress["maximum_min_balance_required"]
assert stress["min_grip_r_permille"] <= stress["maximum_min_grip_r_required"]
assert stress["min_torque_permille"] <= stress["maximum_min_torque_required"]
assert stress["stress_thresholds_passed"] is True
assert stress["all_stress_cases_stable"] is True
assert stress["all_contact_packets_ordered"] is True
assert stress["all_turn_hash_chains_stable"] is True
assert stress["contact_order_rule"] == "frame_then_attacker_then_defender_then_action_then_target_then_direction"
assert stress["outcome_authority"] == "truth_replay_only"
assert stress["public_demo_ready"] is False
assert stress["release_candidate_ready"] is False
for pairing in stress["pairings"]:
    assert pairing["turn_count"] == 24
    assert pairing["stable_committed_sequences"] is True
    assert pairing["stable_replay"] is True
    assert pairing["stable_trace"] is True
    assert pairing["stable_turn_hash_chain"] is True
    assert pairing["contact_order_ok"] is True
    assert pairing["repeat_contact_order_ok"] is True
    assert pairing["replay_verified"] is True
    assert pairing["all_truth_actions_valid"] is True
    assert pairing["passed"] is True
    replay_a = json.loads((root / pairing["id"] / "run_a" / "replay.json").read_text())
    replay_b = json.loads((root / pairing["id"] / "run_b" / "replay.json").read_text())
    assert replay_a == replay_b
PY

grep -q 'Status: PASSED' "$out/truth_stress_report.md"
grep -q 'Stress turn count: `24`' "$out/truth_stress_report.md"
grep -q 'Adversarial thresholds passed: `true`' "$out/truth_stress_report.md"
grep -q '## Adversarial Thresholds' "$out/truth_stress_report.md"
grep -q 'Contact order rule: `frame_then_attacker_then_defender_then_action_then_target_then_direction`' "$out/truth_stress_report.md"
grep -q 'Capability-stop end conditions:' "$out/truth_stress_report.md"
grep -q 'Hidden RNG: `false`' "$out/truth_stress_report.md"
grep -q 'Wall clock: `false`' "$out/truth_stress_report.md"
grep -q 'Gameplay floats: `false`' "$out/truth_stress_report.md"

echo "truth stress passed"
