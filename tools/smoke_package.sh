#!/usr/bin/env bash
set -euo pipefail

package_tar="${1:-artifacts/package/oathyard-linux-x86_64.tar}"
smoke_root="${2:-artifacts/package_smoke/latest}"
package_tar_abs="$(realpath "$package_tar")"
rm -rf "$smoke_root"
mkdir -p "$smoke_root"
smoke_root_abs="$(cd "$smoke_root" && pwd -P)"

tar -xf "$package_tar_abs" -C "$smoke_root_abs"
package_dir="$smoke_root_abs/oathyard-linux-x86_64"
test -x "$package_dir/bin/oathyard"

(
  cd "$package_dir"
  ./bin/oathyard run \
    --scenario examples/duels/basic_oathyard.duel \
    --out "$smoke_root_abs/run_artifacts" >/dev/null
  ./bin/oathyard replay \
    --replay "$smoke_root_abs/run_artifacts/replay.json" >/dev/null
  ./bin/oathyard export-bundle \
    --replay "$smoke_root_abs/run_artifacts/replay.json" \
    --out "$smoke_root_abs/export_bundle" >/dev/null
  ./bin/oathyard verify-bundle \
    --bundle "$smoke_root_abs/export_bundle" >/dev/null
)

(
  cd "$package_dir"
  ./bin/oathyard native-roster-showcase \
    --out "$smoke_root_abs/native_roster" >/dev/null
  ./bin/oathyard native-combat-render \
    --scenario examples/duels/basic_oathyard.duel \
    --out "$smoke_root_abs/native_combat" >/dev/null
  OATHYARD_LAUNCH_OUT="$smoke_root_abs/no_args_launch" \
  OATHYARD_LAUNCH_SCENARIO="$package_dir/examples/duels/basic_oathyard.duel" \
    ./bin/oathyard >/dev/null
)

test -s "$smoke_root_abs/run_artifacts/trace.json"
test -s "$smoke_root_abs/run_artifacts/replay.json"
test -s "$smoke_root_abs/run_artifacts/duel_report.md"
test -s "$smoke_root_abs/run_artifacts/fight_film_manifest.json"
test -s "$smoke_root_abs/export_bundle/export_bundle_manifest.json"
test -s "$smoke_root_abs/export_bundle/bundle_hashes.txt"
test -s "$smoke_root_abs/native_roster/native_roster_showcase_manifest.json"
test -s "$smoke_root_abs/native_combat/native_combat_render_manifest.json"
test -s "$smoke_root_abs/no_args_launch/native_combat_render_manifest.json"

grep -q '"renderer": "native-software-3d"' "$smoke_root_abs/native_combat/native_combat_render_manifest.json"
grep -q '"renderer": "native-software-3d"' "$smoke_root_abs/no_args_launch/native_combat_render_manifest.json"
grep -q '"native_3d_runtime_geometry": true' "$smoke_root_abs/native_combat/native_combat_render_manifest.json"
grep -q '"game_is_3d": true' "$smoke_root_abs/native_roster/native_roster_showcase_manifest.json"

echo "package smoke passed: $package_tar_abs"
