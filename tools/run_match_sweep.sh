#!/usr/bin/env bash
set -euo pipefail

root="${1:-artifacts/match_sweep}"
mkdir -p "$root"

scenarios=(
  examples/duels/basic_oathyard.duel
  examples/duels/axe_vs_spear.duel
  examples/duels/shield_vs_maul.duel
  examples/duels/duelist_vs_writ.duel
)

for scenario in "${scenarios[@]}"; do
  name="$(basename "$scenario" .duel)"
  out="$root/$name"
  cargo run --locked -- match --scenario "$scenario" --out "$out" --best-of 5
done

./tools/ai_sweep.sh "$root/ai_sweep"
./tools/truth_stress.sh "$root/truth_stress"

python3 - "$root" <<'PY'
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
scenario_paths = [
    Path("examples/duels/basic_oathyard.duel"),
    Path("examples/duels/axe_vs_spear.duel"),
    Path("examples/duels/shield_vs_maul.duel"),
    Path("examples/duels/duelist_vs_writ.duel"),
]

scripted_matches = []
scripted_hashes = set()
scripted_round_count = 0
for scenario_path in scenario_paths:
    name = scenario_path.stem
    match_dir = root / name
    data = json.loads((match_dir / "match_summary.json").read_text())
    rounds = data["rounds"]
    hashes = [round_entry["final_state_hash"] for round_entry in rounds]
    statuses = [round_entry["end_condition_status"] for round_entry in rounds]
    winners = [round_entry["end_condition_winner"] for round_entry in rounds]
    scripted_hashes.update(hashes)
    scripted_round_count += len(rounds)
    scripted_matches.append(
        {
            "scenario": name,
            "scenario_file": str(scenario_path),
            "best_of": data["best_of"],
            "rounds_played": len(rounds),
            "seat_0_wins": data["seat_0_wins"],
            "seat_1_wins": data["seat_1_wins"],
            "match_winner": data["match_winner"],
            "round_hash_stable": len(set(hashes)) == 1,
            "round_status_stable": len(set(statuses)) == 1,
            "round_winner_token_stable": len(set(winners)) == 1,
            "final_round_hash": hashes[-1],
            "final_round_status": statuses[-1],
            "final_round_winner": winners[-1],
        }
    )

ai = json.loads((root / "ai_sweep" / "ai_sweep.json").read_text())
truth = json.loads((root / "truth_stress" / "truth_stress.json").read_text())

ai_capability_stops = sum(
    1 for pairing in ai["pairings"] if pairing["end_condition_winner"] != "none"
)
scripted_all_stable = all(
    match["round_hash_stable"]
    and match["round_status_stable"]
    and match["round_winner_token_stable"]
    for match in scripted_matches
)
combined_capability_stop_outcomes = ai_capability_stops + truth["capability_stop_count"]
overall_passed = (
    len(scripted_matches) >= 4
    and scripted_round_count >= 12
    and scripted_all_stable
    and len(scripted_hashes) >= 4
    and ai["all_pairings_stable"] is True
    and ai["all_replays_verified"] is True
    and ai["all_actions_legal"] is True
    and ai["all_truth_actions_valid"] is True
    and ai["total_contacts"] >= 30
    and ai["capability_reaction_count"] >= 50
    and ai["distinct_action_labels"] >= 10
    and ai["unique_final_hashes"] >= 5
    and ai_capability_stops >= 2
    and truth["stress_thresholds_passed"] is True
    and truth["stress_turn_count"] >= 24
    and truth["all_stress_cases_stable"] is True
    and truth["all_contact_packets_ordered"] is True
    and truth["all_turn_hash_chains_stable"] is True
    and combined_capability_stop_outcomes >= 6
)

summary = {
    "schema": "oathyard.match_sweep.v1",
    "product": "OATHYARD",
    "public_demo_ready": False,
    "release_candidate_ready": False,
    "scripted_match_count": len(scripted_matches),
    "scripted_round_count": scripted_round_count,
    "scripted_unique_final_hashes": len(scripted_hashes),
    "scripted_all_rounds_stable": scripted_all_stable,
    "combined_capability_stop_outcomes": combined_capability_stop_outcomes,
    "ai_sweep": {
        "pairing_count": ai["pairing_count"],
        "runs_per_pairing": ai["runs_per_pairing"],
        "total_contacts": ai["total_contacts"],
        "capability_reaction_count": ai["capability_reaction_count"],
        "distinct_action_labels": ai["distinct_action_labels"],
        "policy_style_count": ai["policy_style_count"],
        "unique_final_hashes": ai["unique_final_hashes"],
        "capability_stop_outcomes": ai_capability_stops,
        "all_pairings_stable": ai["all_pairings_stable"],
        "all_replays_verified": ai["all_replays_verified"],
        "all_actions_legal": ai["all_actions_legal"],
        "all_truth_actions_valid": ai["all_truth_actions_valid"],
        "difficulty_changes_body_stats": ai["difficulty_changes_body_stats"],
        "body_stat_mutation_by_ai": ai["body_stat_mutation_by_ai"],
        "outcome_authority": ai["outcome_authority"],
    },
    "truth_stress": {
        "stress_turn_count": truth["stress_turn_count"],
        "total_contacts": truth["total_contacts"],
        "capability_reaction_count": truth["capability_reaction_count"],
        "capability_stop_count": truth["capability_stop_count"],
        "distinct_final_hash_count": truth["distinct_final_hash_count"],
        "max_recovery_slowdown_frames": truth["max_recovery_slowdown_frames"],
        "min_balance_permille": truth["min_balance_permille"],
        "min_grip_r_permille": truth["min_grip_r_permille"],
        "min_torque_permille": truth["min_torque_permille"],
        "stress_thresholds_passed": truth["stress_thresholds_passed"],
        "all_stress_cases_stable": truth["all_stress_cases_stable"],
        "all_contact_packets_ordered": truth["all_contact_packets_ordered"],
        "all_turn_hash_chains_stable": truth["all_turn_hash_chains_stable"],
    },
    "scripted_matches": scripted_matches,
    "overall_passed": overall_passed,
}

(root / "match_sweep_summary.json").write_text(json.dumps(summary, indent=2) + "\n")

lines = [
    "# OATHYARD Match Sweep Summary",
    "",
    f"Status: {'PASSED' if overall_passed else 'FAILED'}",
    f"- Public demo ready: `{str(summary['public_demo_ready']).lower()}`",
    f"- Release candidate ready: `{str(summary['release_candidate_ready']).lower()}`",
    f"- Scripted matches: `{summary['scripted_match_count']}`",
    f"- Scripted rounds: `{summary['scripted_round_count']}`",
    f"- Scripted unique final hashes: `{summary['scripted_unique_final_hashes']}`",
    f"- Scripted repeated rounds stable: `{str(scripted_all_stable).lower()}`",
    f"- Combined capability-stop outcomes: `{combined_capability_stop_outcomes}`",
    "",
    "## Scripted Matches",
    "",
]
for match in scripted_matches:
    lines.append(
        "- {scenario}: passed, final round hash `{final_round_hash}`, outcome `{final_round_status} {final_round_winner}`, rounds `{rounds_played}`, stable `{stable}`".format(
            scenario=match["scenario"],
            final_round_hash=match["final_round_hash"],
            final_round_status=match["final_round_status"],
            final_round_winner=match["final_round_winner"],
            rounds_played=match["rounds_played"],
            stable=str(match["round_hash_stable"]).lower(),
        )
    )
lines.extend(
    [
        "",
        "## Deterministic AI And Truth Stress",
        "",
        (
            "- deterministic_ai_sweep: passed, pairings `{pairing_count}`, runs per pairing `{runs_per_pairing}`, "
            "contacts `{total_contacts}`, capability reactions `{capability_reaction_count}`, distinct actions `{distinct_action_labels}`, "
            "unique hashes `{unique_final_hashes}`, capability-stop outcomes `{capability_stop_outcomes}`"
        ).format(**summary["ai_sweep"]),
        (
            "- adversarial_truth_stress: passed, turns `{stress_turn_count}`, contacts `{total_contacts}`, "
            "capability reactions `{capability_reaction_count}`, capability-stop outcomes `{capability_stop_count}`, "
            "distinct hashes `{distinct_final_hash_count}`, thresholds `{thresholds}`"
        ).format(
            thresholds=str(summary["truth_stress"]["stress_thresholds_passed"]).lower(),
            **summary["truth_stress"],
        ),
        "",
    ]
)
(root / "match_sweep_summary.md").write_text("\n".join(lines))

if not overall_passed:
    raise SystemExit("match sweep did not meet deterministic/adversarial evidence thresholds")
PY

python3 -m json.tool "$root/match_sweep_summary.json" >/dev/null
grep -q 'Status: PASSED' "$root/match_sweep_summary.md"
grep -q 'deterministic_ai_sweep: passed' "$root/match_sweep_summary.md"
grep -q 'adversarial_truth_stress: passed' "$root/match_sweep_summary.md"

echo "match sweep passed"
