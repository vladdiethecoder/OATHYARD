#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/input_target/verify}"
input_map_json="${2:-artifacts/input/verify/input_map.json}"
input_profile_json="${3:-artifacts/input/verify/input_profile.json}"
gamepad_json="${4:-artifacts/gamepad/verify/gamepad_smoke.json}"
settings_json="${5:-artifacts/settings/verify/runtime_settings.saved.json}"
adr="docs/decisions/0003-native-input-model.md"

python3 - "$out" "$input_map_json" "$input_profile_json" "$gamepad_json" "$settings_json" "$adr" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

out = Path(sys.argv[1])
input_map_path = Path(sys.argv[2])
input_profile_path = Path(sys.argv[3])
gamepad_path = Path(sys.argv[4])
settings_path = Path(sys.argv[5])
adr_path = Path(sys.argv[6])
out.mkdir(parents=True, exist_ok=True)

def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))

def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()

checks = []
def check(check_id: str, passed: bool, detail: str) -> None:
    checks.append({"id": check_id, "passed": bool(passed), "detail": detail})

try:
    input_map = read_json(input_map_path)
    input_profile = read_json(input_profile_path)
    gamepad = read_json(gamepad_path)
    settings = read_json(settings_path)
    adr_text = adr_path.read_text(encoding="utf-8")
except Exception as error:  # noqa: BLE001
    input_map = {}
    input_profile = {}
    gamepad = {}
    settings = {}
    adr_text = ""
    check("inputs_readable", False, str(error))
else:
    check("inputs_readable", True, "ADR and input evidence artifacts loaded")

required_actions = {
    "next_screen",
    "previous_screen",
    "main_menu_start",
    "mode_select",
    "settings_accessibility",
    "fighter_select",
    "loadout_select",
    "observe",
    "plan",
    "commit_reveal",
    "resolve",
    "consequence_readout",
    "replay_browser",
    "fight_film",
    "performance_debug_overlay",
    "quit",
}
required_screens = [
    "main_menu",
    "mode_select",
    "settings_accessibility",
    "fighter_select",
    "loadout_select",
    "observe",
    "plan",
    "commit_reveal",
    "resolve",
    "consequence",
    "replay_browser",
    "fight_film",
    "performance_debug_overlay",
]
map_actions = {entry.get("action") for entry in input_map.get("actions", [])}
profile_commands = input_profile.get("commands", [])
profile_actions = {entry.get("action") for entry in profile_commands}
profile_screens = {entry.get("screen") for entry in input_profile.get("screens", [])}

check("adr_present", adr_path.is_file() and "Native Input Model" in adr_text, str(adr_path))
check("input_map_schema", input_map.get("schema") == "oathyard.input_map.v1", input_map_path.as_posix())
check("input_map_required_actions", required_actions.issubset(map_actions), ",".join(sorted(str(a) for a in map_actions)))
check("input_map_remappable", input_map.get("remappable") is True, "input_map")
check("input_map_truth_mutation_false", input_map.get("truth_mutation") is False, "input_map")
check("input_profile_schema", input_profile.get("schema") == "oathyard.input_profile.v1", input_profile_path.as_posix())
check("input_profile_required_actions", required_actions.issubset(profile_actions), ",".join(sorted(str(a) for a in profile_actions)))
check("input_profile_remappable", input_profile.get("remappable") is True, "input_profile")
check("input_profile_truth_mutation_false", input_profile.get("truth_mutation") is False, "input_profile")
check("keyboard_mouse_gamepad_parity", input_profile.get("keyboard_mouse_gamepad_parity") is True, "input_profile")
for screen in required_screens:
    check(f"profile_screen_{screen}", screen in profile_screens, screen)
check("profile_all_current_screens_reachable", input_profile.get("all_current_screens_reachable_with_default_controller") is True, "input_profile")
check("gamepad_schema", gamepad.get("schema") == "oathyard.gamepad_smoke.v1", gamepad_path.as_posix())
check("gamepad_presentation_only", gamepad.get("presentation_only") is True, "gamepad")
check("gamepad_truth_mutation_false", gamepad.get("truth_mutation") is False, "gamepad")
check("physical_gamepad_not_claimed", input_profile.get("physical_gamepad_hardware_claimed") is False and gamepad.get("physical_gamepad_hardware_claimed") is False, "false")
check("steam_deck_hardware_not_claimed", input_profile.get("steam_deck_hardware_claimed") is False and gamepad.get("steam_deck_hardware_claimed") is False, "false")
check("settings_schema", settings.get("schema") == "oathyard.runtime_settings.v1", settings_path.as_posix())
check("settings_truth_mutation_false", settings.get("truth_mutation") is False, "settings")
check("settings_replay_hash_unaffected", settings.get("replay_hash_affects") is False, "settings")
check("settings_hold_to_commit", settings.get("hold_to_commit") is True, "settings")

passed = all(item["passed"] for item in checks)
artifact_hashes = {
    "adr": sha256(adr_path) if adr_path.is_file() else "",
    "input_map": sha256(input_map_path) if input_map_path.is_file() else "",
    "input_profile": sha256(input_profile_path) if input_profile_path.is_file() else "",
    "gamepad_smoke": sha256(gamepad_path) if gamepad_path.is_file() else "",
    "runtime_settings": sha256(settings_path) if settings_path.is_file() else "",
}
report = {
    "schema": "oathyard.native_input_target_audit.v2",
    "product": "OATHYARD",
    "adr": adr_path.as_posix(),
    "command_boundary": "presentation_command_only_until_replayable_committed_inputs",
    "keyboard_mouse_gamepad_schema_ready": True,
    "physical_gamepad_hardware_claimed": False,
    "steam_deck_hardware_claimed": False,
    "owner_input_acceptance_claimed": False,
    "covered_actions": sorted(required_actions),
    "covered_screens": required_screens,
    "presentation_only": True,
    "truth_mutation": False,
    "replay_hash_affects": False,
    "public_demo_ready": False,
    "release_candidate_ready": False,
    "checks": checks,
    "artifact_hashes": artifact_hashes,
    "passed": passed,
}
(out / "native_input_target.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
lines = [
    "# OATHYARD Native Input Target Audit",
    "",
    f"Status: {'PASSED' if passed else 'FAILED'}",
    "- Command boundary: `presentation_command_only_until_replayable_committed_inputs`",
    "- Keyboard/mouse/gamepad schema ready: `true`",
    "- Physical gamepad hardware claimed: `false`",
    "- Steam Deck hardware claimed: `false`",
    "- Owner input acceptance claimed: `false`",
    "- Truth mutation: `none`",
    "",
    "## Checks",
]
for item in checks:
    lines.append(f"- {'PASS' if item['passed'] else 'FAIL'} `{item['id']}`: {item['detail']}")
(out / "native_input_target_report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
if not passed:
    raise SystemExit(1)
PY

echo "input target audit passed: $out"
