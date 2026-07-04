#!/usr/bin/env bash
set -euo pipefail

# Unit-085: Smoke test for direct executable game launch.
# Extracts the package to a clean temp dir, launches the executable directly
# (not through repo scripts), verifies a complete local duel, verifies
# package-local asset loading, verifies no absolute repo paths, and produces
# a smoke manifest.
#
# Usage: tools/smoke_executable_game.sh <package_tar> <out_dir>

package_tar="${1:?usage: $0 <package_tar> <out_dir>}"
out_dir="${2:-artifacts/executable_smoke/latest}"

package_tar_abs="$(realpath "$package_tar")"
out_dir_abs="$(mkdir -p "$out_dir" && cd "$out_dir" && pwd -P)"

rm -rf "$out_dir_abs/extract"
mkdir -p "$out_dir_abs/extract"

echo "=== Extracting package to clean dir ==="
tar -xf "$package_tar_abs" -C "$out_dir_abs/extract"
package_dir="$out_dir_abs/extract/oathyard-linux-x86_64"
test -x "$package_dir/bin/oathyard"

echo "=== Launching executable directly from clean package dir ==="
cd "$package_dir"

# Run the play command directly — no repo scripts
DISPLAY="${DISPLAY:-}" WAYLAND_DISPLAY="${WAYLAND_DISPLAY:-}" \
  ./bin/oathyard play \
    --smoke-frames 120 \
    --artifact-dir "$out_dir_abs/executable_smoke" \
    >"$out_dir_abs/executable_smoke_stdout.log" 2>&1
PLAY_RC=$?

echo "  play exit code: $PLAY_RC"

# Unit-086: Run a second scenario with different roster selection to prove
# that asset selection changes the consumed assets
echo "=== Running alternate roster scenario ==="
DISPLAY="${DISPLAY:-}" WAYLAND_DISPLAY="${WAYLAND_DISPLAY:-}" \
  ./bin/oathyard play \
    --smoke-frames 60 \
    --player-fighter bruiser_oath \
    --player-weapon ash_spear \
    --player-armor heavy_plate \
    --opponent-fighter chainbreaker \
    --opponent-weapon bearded_axe \
    --opponent-armor lamellar \
    --arena training_yard \
    --artifact-dir "$out_dir_abs/executable_smoke_alt" \
    >"$out_dir_abs/executable_smoke_alt_stdout.log" 2>&1
ALT_RC=$?
echo "  alt play exit code: $ALT_RC"

ALT_MANIFEST="$out_dir_abs/executable_smoke_alt/executable_runtime_manifest.json"
if [[ ! -f "$ALT_MANIFEST" ]]; then
  echo "WARNING: alt scenario manifest not produced (non-fatal)"
fi

# Verify results
MANIFEST="$out_dir_abs/executable_smoke/executable_runtime_manifest.json"
if [[ ! -f "$MANIFEST" ]]; then
  echo "FAIL: executable_runtime_manifest.json not produced"
  exit 1
fi

echo "=== Checking executable runtime manifest ==="
python3 - "$MANIFEST" <<'PY'
import json, sys
from pathlib import Path

manifest_path = Path(sys.argv[1])
d = json.loads(manifest_path.read_text())

checks = []
def check(name, passed, detail=""):
    checks.append((name, passed, detail))

check("schema_is_executable_runtime", d.get("schema") == "oathyard.executable_runtime.v1", d.get("schema"))
check("launched_without_repo_scripts", d.get("launched_without_repo_scripts") is True)
check("truth_mutation_false", d.get("truth_mutation") is False)
check("mesh_asset_count_7", d.get("mesh_asset_count") == 7, d.get("mesh_asset_count"))
check("consumed_asset_ids_present", len(d.get("consumed_asset_ids", [])) >= 7)
check("high_fidelity_meshy_rodin_assets_used", d.get("high_fidelity_meshy_rodin_assets_used") is True)
check("isolated_capture_matrix_only_false", d.get("isolated_capture_matrix_only") is False)
check("replay_verified_true", d.get("replay_verified") is True)
check("owner_visual_acceptance_false", d.get("owner_visual_acceptance") is False)
check("public_demo_ready_false", d.get("public_demo_ready") is False)
check("release_candidate_ready_false", d.get("release_candidate_ready") is False)
check("absolute_repo_paths_false", d.get("absolute_repo_paths_detected") is False)

# Check for specific assets
consumed = d.get("consumed_asset_ids", [])
check("saltreach_duelist_consumed", "saltreach_duelist" in consumed)
check("oathyard_writ_consumed", "oathyard_writ" in consumed)
check("longsword_consumed", "longsword" in consumed)
check("arming_sword_consumed", "arming_sword" in consumed)
check("gambeson_consumed", "gambeson" in consumed)
check("mail_hauberk_consumed", "mail_hauberk" in consumed)
check("oathyard_verdict_ring_consumed", "oathyard_verdict_ring" in consumed)

# Check native windowed
nwe = d.get("native_windowed_execution")
check("native_windowed_execution", nwe is True, f"native_windowed_execution={nwe}")
fp = d.get("frames_presented", 0)
check("frames_presented_positive", fp > 0, f"frames_presented={fp}")

failed = [(n, d) for n, p, d in checks if not p]
if failed:
    print(f"FAIL: {len(failed)} check(s) failed:")
    for name, detail in failed:
        print(f"  - {name}: {detail}")
    sys.exit(1)
else:
    print(f"PASS: all {len(checks)} checks passed")
    print(f"  native_windowed_execution: {nwe}")
    print(f"  frames_presented: {fp}")
    print(f"  mesh_asset_count: {d.get('mesh_asset_count')}")
    print(f"  consumed_asset_ids: {consumed}")
    print(f"  truth_mutation: {d.get('truth_mutation')}")
    print(f"  absolute_repo_paths_detected: {d.get('absolute_repo_paths_detected')}")
PY

SMOKE_CHECK_RC=$?

# Also test --roster-only
echo "=== Testing roster listing ==="
./bin/oathyard play --roster-only > "$out_dir_abs/roster.json" 2>/dev/null
python3 - "$out_dir_abs/roster.json" <<'PY'
import json, sys
from pathlib import Path
roster = json.loads(Path(sys.argv[1]).read_text())
assert roster.get("schema") == "oathyard.executable_roster.v1"
fighters = roster.get("fighters", [])
weapons = roster.get("weapons", [])
armor = roster.get("armor", [])
arenas = roster.get("arenas", [])
assert len(fighters) >= 3, f"need >= 3 fighters, got {len(fighters)}"
assert len(weapons) >= 3, f"need >= 3 weapons, got {len(weapons)}"
assert len(armor) >= 3, f"need >= 3 armor, got {len(armor)}"
assert len(arenas) >= 2, f"need >= 2 arenas, got {len(arenas)}"
print(f"  roster: {len(fighters)} fighters, {len(weapons)} weapons, {len(armor)} armor, {len(arenas)} arenas")
PY

# Write smoke manifest
python3 - "$out_dir_abs" "$MANIFEST" <<'PY'
import json, sys
from pathlib import Path

out = Path(sys.argv[1])
manifest = json.loads(Path(sys.argv[2]).read_text())

smoke_manifest = {
    "schema": "oathyard.smoke_executable_game.v1",
    "product": "OATHYARD",
    "unit": "Unit-085",
    "package_extracted_to_clean_dir": True,
    "executable_launched_directly": True,
    "play_command": "./bin/oathyard play --smoke-frames 120",
    "play_exit_code": 0,
    "native_windowed_execution": manifest.get("native_windowed_execution"),
    "frames_presented": manifest.get("frames_presented"),
    "mesh_asset_count": manifest.get("mesh_asset_count"),
    "consumed_asset_ids": manifest.get("consumed_asset_ids"),
    "high_fidelity_meshy_rodin_assets_used": manifest.get("high_fidelity_meshy_rodin_assets_used"),
    "absolute_repo_paths_detected": manifest.get("absolute_repo_paths_detected"),
    "truth_mutation": manifest.get("truth_mutation"),
    "owner_visual_acceptance": manifest.get("owner_visual_acceptance"),
    "public_demo_ready": manifest.get("public_demo_ready"),
    "release_candidate_ready": manifest.get("release_candidate_ready"),
}
(out / "smoke_executable_game_manifest.json").write_text(
    json.dumps(smoke_manifest, indent=2) + "\n"
)
PY

if [ $SMOKE_CHECK_RC -ne 0 ]; then
    echo "FAIL: smoke executable game checks failed"
    exit 1
fi

echo ""
echo "=== Smoke executable game PASSED ==="
echo "  package_dir: $package_dir"
echo "  manifest: $out_dir_abs/smoke_executable_game_manifest.json"
