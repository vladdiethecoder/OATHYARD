#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/ai_planner_audit/latest}"
mkdir -p "$out"

./tools/ai_sweep.sh "$out/ai_sweep" > "$out/ai_sweep.log" 2>&1

python3 - "$out" <<'PY'
import json
import sys
from pathlib import Path

out = Path(sys.argv[1])
sweep_path = out / "ai_sweep/ai_sweep.json"
sweep = json.loads(sweep_path.read_text(encoding="utf-8"))
checks = []
failures = []

def check(check_id, passed, detail):
    checks.append({"id": check_id, "passed": bool(passed), "detail": detail})
    if not passed:
        failures.append(f"{check_id}: {detail}")

check("schema", sweep.get("schema") == "oathyard.ai_sweep.v1", str(sweep.get("schema")))
for key in [
    "all_pairings_stable",
    "all_replays_verified",
    "all_actions_legal",
    "all_truth_actions_valid",
]:
    check(key, sweep.get(key) is True, str(sweep.get(key)))
pairings = sweep.get("pairings", [])
for key in ["stable_committed_sequences", "stable_replay", "stable_trace"]:
    check(
        key,
        bool(pairings) and all(pair.get(key) is True for pair in pairings),
        f"{sum(1 for pair in pairings if pair.get(key) is True)}/{len(pairings)} pairings",
    )
for key in ["difficulty_changes_body_stats", "body_stat_mutation_by_ai"]:
    check(key, sweep.get(key) is False, str(sweep.get(key)))
check("outcome_authority_truth_only", sweep.get("outcome_authority") == "truth_replay_only", str(sweep.get("outcome_authority")))
check("policy_style_coverage", int(sweep.get("policy_style_count", 0)) >= 6, str(sweep.get("policy_style_count")))
check("distinct_action_labels", int(sweep.get("distinct_action_labels", 0)) >= 11, str(sweep.get("distinct_action_labels")))
check("unique_final_hashes", int(sweep.get("unique_final_hashes", 0)) >= 6, str(sweep.get("unique_final_hashes")))

passed = not failures
manifest = {
    "schema": "oathyard.ai_planner_audit.v1",
    "tool": "tools/ai_planner_audit.sh",
    "passed": passed,
    "source_ai_sweep": "ai_sweep/ai_sweep.json",
    "all_actions_legal": sweep.get("all_actions_legal"),
    "all_truth_actions_valid": sweep.get("all_truth_actions_valid"),
    "outcome_authority": sweep.get("outcome_authority"),
    "body_stat_mutation_by_ai": sweep.get("body_stat_mutation_by_ai"),
    "difficulty_changes_body_stats": sweep.get("difficulty_changes_body_stats"),
    "policy_style_count": sweep.get("policy_style_count"),
    "distinct_action_labels": sweep.get("distinct_action_labels"),
    "failed_check_count": len(failures),
    "checks": checks,
}
(out / "ai_planner_audit_manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
(out / "failed_ai_planner_checks.txt").write_text("none\n" if passed else "\n".join(failures) + "\n", encoding="utf-8")
report = [
    "# OATHYARD AI Planner Audit",
    "",
    f"Status: {'PASSED' if passed else 'FAILED'}",
    f"- All actions legal: `{sweep.get('all_actions_legal')}`",
    f"- Truth actions valid: `{sweep.get('all_truth_actions_valid')}`",
    f"- Outcome authority: `{sweep.get('outcome_authority')}`",
    f"- Body stat mutation by AI: `{sweep.get('body_stat_mutation_by_ai')}`",
    f"- Difficulty changes body stats: `{sweep.get('difficulty_changes_body_stats')}`",
    f"- Policy styles: `{sweep.get('policy_style_count')}`",
    f"- Distinct action labels: `{sweep.get('distinct_action_labels')}`",
    f"- Failed checks: `{len(failures)}`",
    "",
    "Scope: AI may emit legal planned actions and directional influence only; truth decides contacts, injuries, capability deltas, costs, end states, and hashes.",
]
if failures:
    report.extend(["", "## Failures"] + [f"- {f}" for f in failures])
(out / "ai_planner_audit_report.md").write_text("\n".join(report) + "\n", encoding="utf-8")
if not passed:
    raise SystemExit(1)
PY

echo "ai planner audit: $out"
