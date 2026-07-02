#!/usr/bin/env bash
set -euo pipefail

# Unit-057: Launch a deterministic local working-game path.
# This is the user-facing entry point for the scripted "playable game".
# Internally it runs the same truth/replay pipeline used by production
# renderer captures — it is NOT a detached screenshot script.
#
# Usage: ./tools/play_local_game.sh <output-dir>
#   Example: ./tools/play_local_game.sh artifacts/verification/<ts>_unit057/playable_game

if [[ $# -lt 1 ]]; then
    echo "usage: $0 <output-dir>" >&2
    exit 2
fi

out="$1"
mkdir -p "$out"

echo "=== OATHYARD Unit-057: Working Local Game ==="
echo "Output: $out"

# Step 1: ensure the binary is built.
if [[ ! -x ./target/debug/oathyard ]]; then
    echo "Building oathyard binary..."
    cargo build --locked
fi

# Step 2: run the scripted local game path.
# The local_game module drives:
#   Boot -> MainMenu -> ModeSelect -> FighterSelect -> LoadoutSelect ->
#   ArenaSelect -> MatchIntro ->
#   (Observe -> Plan -> CommitReveal -> Resolve -> Consequence -> Replan)*
#   -> MatchResult -> ReplayBrowser -> FightFilmView -> Settings -> Quit
# Truth is never mutated; replay is verified against final hash.
./target/debug/oathyard play-local --out "$out" | tee "$out/play_local_stdout.log"

# Step 3: require artifacts.
required=(
    game_flow_manifest.json
    scripted_input_manifest.json
    replay.json
    trace.json
    final_state_hash.txt
    fight_film_manifest.json
    fight_film_view_manifest.json
    planning_ui_data_report.md
    consequence_cause_chain_report.md
    replay_verification_report.md
    duel_report.md
)
missing=0
for f in "${required[@]}"; do
    if [[ ! -f "$out/$f" ]]; then
        echo "ERROR: missing required artifact: $out/$f" >&2
        missing=1
    fi
done
if [[ $missing -ne 0 ]]; then
    exit 1
fi

# Step 4: replay verify via the same path used by replay_verify.sh.
./target/debug/oathyard replay --replay "$out/replay.json" > "$out/replay_verify_stdout.log" 2>&1

# Step 5: truth-mutation check — every manifest must report truth_mutation=false.
truth_mutation_check() {
    local path="$1"
    local found
    found=$(grep -E '"truth_mutation"\s*:\s*false' "$path" | head -1 || true)
    if [[ -z "$found" ]]; then
        echo "ERROR: $path did not assert truth_mutation=false" >&2
        return 1
    fi
}

truth_mutation_check "$out/game_flow_manifest.json"
truth_mutation_check "$out/fight_film_view_manifest.json"
truth_mutation_check "$out/scripted_input_manifest.json"

# Step 6: required states visited.
required_states=(boot main_menu mode_select fighter_select loadout_select arena_select match_intro observe plan commit_reveal resolve consequence replan match_result replay_browser fight_film_view settings quit)
for state in "${required_states[@]}"; do
    count=$(grep -o "\"state\": \"$state\"" "$out/game_flow_manifest.json" | wc -l)
    if [[ $count -eq 0 ]]; then
        echo "ERROR: required game state '$state' not present in game_flow_manifest.json" >&2
        exit 1
    fi
done

# Step 7: plan cycles >= 2.
plan_cycles=$(python3 -c "
import json
with open('$out/game_flow_manifest.json') as f:
    data = json.load(f)
print(data.get('plan_cycles', 0))
")
if [[ "$plan_cycles" -lt 2 ]]; then
    echo "ERROR: plan_cycles=$plan_cycles < required 2" >&2
    exit 1
fi

# Step 8: local_playable_game_ready must be true for the smoke to pass.
ready=$(python3 -c "
import json
with open('$out/game_flow_manifest.json') as f:
    data = json.load(f)
print(str(data.get('local_playable_game_ready', False)).lower())
")
if [[ "$ready" != "true" ]]; then
    echo "ERROR: local_playable_game_ready is not true in game_flow_manifest.json" >&2
    exit 1
fi

echo "=== OATHYARD Unit-057: Working local game play PASSED ==="
echo "plan_cycles=$plan_cycles final_hash=$(cat $out/final_state_hash.txt)"
echo "local_playable_game_ready=$ready"
exit 0
