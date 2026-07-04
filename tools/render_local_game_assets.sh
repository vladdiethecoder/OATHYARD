#!/usr/bin/env bash
set -euo pipefail

# Unit-083: render the actual local-game loadout through the native production
# renderer using the source-approved Meshy/Rodin runtime meshes. This keeps the
# local game truth path authoritative while proving the game-facing loadout is
# not limited to isolated capture-matrix evidence.

if [[ $# -lt 1 ]]; then
  echo "usage: $0 <local-game-out-dir> [render-out-dir]" >&2
  exit 2
fi

local_game_out="$1"
out="${2:-$local_game_out/native_asset_runtime}"
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
renderer_bin="$repo_root/crates/oathyard_renderer/target/debug/oathyard-native-renderer"

mkdir -p "$out" "$out/mesh_manifests" "$out/render"

if [[ ! -f "$local_game_out/game_flow_manifest.json" ]]; then
  echo "ERROR: local game manifest missing: $local_game_out/game_flow_manifest.json" >&2
  exit 1
fi
if [[ ! -f "$local_game_out/replay.json" || ! -f "$local_game_out/trace.json" ]]; then
  echo "ERROR: local game replay/trace missing under $local_game_out" >&2
  exit 1
fi
if [[ ! -x "$renderer_bin" ]]; then
  cargo build --manifest-path "$repo_root/crates/oathyard_renderer/Cargo.toml"
fi

python3 - "$repo_root" "$local_game_out" "$out" <<'PY'
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

repo = Path(sys.argv[1])
local_game_out = Path(sys.argv[2])
out = Path(sys.argv[3])
flow_path = local_game_out / "game_flow_manifest.json"
flow = json.loads(flow_path.read_text(encoding="utf-8"))

spec = importlib.util.spec_from_file_location("generate_runtime_asset_sets", repo / "tools/generate_runtime_asset_sets.py")
generator = importlib.util.module_from_spec(spec)
assert spec and spec.loader
spec.loader.exec_module(generator)

required_keys = [
    "player_fighter",
    "player_weapon",
    "player_armor",
    "opponent_fighter",
    "opponent_weapon",
    "opponent_armor",
    "arena_id",
]
missing = [key for key in required_keys if not flow.get(key)]
if missing:
    raise SystemExit(f"local game manifest missing asset keys: {missing}")

player = {
    "fighter": flow["player_fighter"],
    "armor": flow["player_armor"],
    "weapon": flow["player_weapon"],
}
opponent = {
    "fighter": flow["opponent_fighter"],
    "armor": flow["opponent_armor"],
    "weapon": flow["opponent_weapon"],
}
arena = flow["arena_id"]
mesh_dir = out / "mesh_manifests" / "runtime_meshes"
mesh_dir.mkdir(parents=True, exist_ok=True)
manifest = generator.build_manifest(
    {
        "set_id": "local_game_current_loadout",
        "description": "Current oathyard play-local loadout rendered with source-approved Meshy/Rodin assets.",
        "player": player,
        "opponent": opponent,
        "arena": arena,
    },
    mesh_dir,
)
manifest["schema"] = "oathyard.wgpu_runtime_mesh_manifest.v1"
manifest["source"] = "tools/render_local_game_assets.sh from oathyard play-local game_flow_manifest.json"
manifest["capture_id"] = "unit083_local_game_generated_asset_consumption"
manifest["local_game_flow_manifest"] = flow_path.as_posix()
manifest["local_playable_game_ready"] = bool(flow.get("local_playable_game_ready"))
manifest["owner_visual_acceptance"] = False
manifest["public_demo_ready"] = False
manifest["release_candidate_ready"] = False
manifest["truth_mutation"] = False
mesh_manifest_path = out / "mesh_manifests" / "local_game_current_loadout.mesh_manifest.json"
mesh_manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")

packet = {
    "schema": "oathyard.post_hash_presentation_packet.v1",
    "source": "tools/render_local_game_assets.sh from verified local game replay/trace artifacts",
    "generated_after_replay_verify": bool(flow.get("replay_verified")) and bool(flow.get("replay_verified_final_hash_matches")),
    "scenario_id": flow.get("scenario_id", "unknown"),
    "content_hash": flow.get("content_hash", "unknown"),
    "final_state_hash": flow.get("final_state_hash", "unknown"),
    "replay_json_sha256": flow.get("replay_json_sha256", ""),
    "trace_json_sha256": flow.get("trace_json_sha256", ""),
    "local_game_flow_manifest": flow_path.as_posix(),
    "truth_mutation": False,
    "owner_visual_acceptance": False,
    "public_demo_ready": False,
    "release_candidate_ready": False,
}
if not packet["generated_after_replay_verify"]:
    raise SystemExit("local game replay verification flags are not true; refusing render")
packet_path = out / "post_hash_presentation_packet.json"
packet_path.write_text(json.dumps(packet, indent=2, sort_keys=True) + "\n", encoding="utf-8")

asset_ids = []
source_asset_ids = []
for mesh in manifest["meshes"]:
    asset_ids.append(mesh["mesh_asset_id"])
    source = str(mesh["mesh_asset_id"]).removeprefix("player_").removeprefix("opponent_")
    source_asset_ids.append(source)

(out / "local_game_asset_runtime_args.json").write_text(json.dumps({
    "packet_path": packet_path.as_posix(),
    "mesh_manifest_path": mesh_manifest_path.as_posix(),
    "candidate_assets": ",".join(source_asset_ids),
    "expected_mesh_asset_ids": asset_ids,
    "truth_mutation": False,
    "owner_visual_acceptance": False,
    "public_demo_ready": False,
    "release_candidate_ready": False,
}, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(packet_path.as_posix())
print(mesh_manifest_path.as_posix())
print(",".join(source_asset_ids))
PY

mapfile -t args < <(python3 - "$out/local_game_asset_runtime_args.json" <<'PY'
import json, sys
from pathlib import Path
args = json.loads(Path(sys.argv[1]).read_text())
print(args["packet_path"])
print(args["mesh_manifest_path"])
print(args["candidate_assets"])
PY
)
packet_path="${args[0]}"
mesh_manifest_path="${args[1]}"
candidate_assets="${args[2]}"

"$renderer_bin" \
  --packet "$packet_path" \
  --out "$out/render" \
  --capture-id unit083_local_game_generated_asset_consumption \
  --capture-file-stem production_renderer_unit083_local_game_generated_asset_consumption_1920x1080 \
  --camera-mode gameplay_distance_fighter_loadout_family_01 \
  --candidate-assets "$candidate_assets" \
  --asset-manifest-sha256 "$(sha256sum assets/manifests/production_candidate_visual_manifest.json | awk '{print $1}')" \
  --mesh-manifest-json "$mesh_manifest_path" \
  > "$out/render/renderer.log" 2>&1

python3 - "$local_game_out" "$out" <<'PY'
import hashlib
import json
import sys
from pathlib import Path
local_game_out = Path(sys.argv[1])
out = Path(sys.argv[2])
flow_path = local_game_out / "game_flow_manifest.json"
flow = json.loads(flow_path.read_text(encoding="utf-8"))
args = json.loads((out / "local_game_asset_runtime_args.json").read_text(encoding="utf-8"))
renderer_manifest_path = out / "render" / "production_renderer_manifest.json"
renderer = json.loads(renderer_manifest_path.read_text(encoding="utf-8"))
png = Path(renderer["capture"]["file"])
if not png.is_file():
    raise SystemExit(f"renderer PNG missing: {png}")
mesh_assets = renderer.get("mesh_assets", [])
mesh_ids = [str(mesh.get("mesh_asset_id", "")) for mesh in mesh_assets]
expected = args["expected_mesh_asset_ids"]
missing = [asset for asset in expected if asset not in mesh_ids]
if missing:
    raise SystemExit(f"renderer did not consume expected local game mesh assets: {missing}; consumed={mesh_ids}")
required_source_assets = [
    flow["player_fighter"], flow["player_weapon"], flow["player_armor"],
    flow["opponent_fighter"], flow["opponent_weapon"], flow["opponent_armor"], flow["arena_id"],
]
source_consumed = sorted(set(mid.removeprefix("player_").removeprefix("opponent_") for mid in mesh_ids))
missing_source = [asset for asset in required_source_assets if asset not in source_consumed]
if missing_source:
    raise SystemExit(f"renderer did not consume expected local game source assets: {missing_source}; consumed={source_consumed}")
payload = {
    "schema": "oathyard.local_game_asset_consumption.v1",
    "source": "tools/render_local_game_assets.sh",
    "local_game_flow_manifest": flow_path.as_posix(),
    "renderer_manifest": renderer_manifest_path.as_posix(),
    "png_path": png.as_posix(),
    "png_sha256": hashlib.sha256(png.read_bytes()).hexdigest(),
    "renderer_backend_id": renderer.get("backend_id"),
    "mesh_geometry_consumed": renderer.get("mesh_geometry_consumed") is True,
    "mesh_asset_count": renderer.get("mesh_asset_count", 0),
    "mesh_asset_ids": mesh_ids,
    "source_asset_ids_consumed": source_consumed,
    "local_game_asset_ids": {
        "player_fighter": flow["player_fighter"],
        "player_weapon": flow["player_weapon"],
        "player_armor": flow["player_armor"],
        "opponent_fighter": flow["opponent_fighter"],
        "opponent_weapon": flow["opponent_weapon"],
        "opponent_armor": flow["opponent_armor"],
        "arena_id": flow["arena_id"],
    },
    "uses_source_approved_meshy_rodin_assets": True,
    "isolated_capture_matrix_only": False,
    "truth_mutation": False,
    "production_ready": False,
    "owner_visual_acceptance": False,
    "public_demo_ready": False,
    "release_candidate_ready": False,
    "blockers": ["owner_visual_acceptance_false", "production_ready_false"],
}
if payload["mesh_asset_count"] < len(expected):
    raise SystemExit(f"mesh_asset_count {payload['mesh_asset_count']} < expected {len(expected)}")
(out / "local_game_asset_consumption_manifest.json").write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
report = [
    "# Local Game Meshy/Rodin Asset Consumption",
    "",
    f"Status: `PASSED`",
    f"Renderer backend: `{payload['renderer_backend_id']}`",
    f"PNG: `{payload['png_path']}`",
    f"Mesh assets consumed: `{payload['mesh_asset_count']}`",
    f"Source assets consumed: `{', '.join(source_consumed)}`",
    "Truth mutation: `false`",
    "Production ready: `false`",
    "Owner visual acceptance: `false`",
]
(out / "local_game_asset_consumption_report.md").write_text("\n".join(report) + "\n", encoding="utf-8")
print(json.dumps({"mesh_asset_count": payload["mesh_asset_count"], "source_assets_consumed": source_consumed, "manifest": (out / "local_game_asset_consumption_manifest.json").as_posix()}, sort_keys=True))
PY

cp "$out/local_game_asset_consumption_manifest.json" "$local_game_out/local_game_asset_consumption_manifest.json"
echo "local game asset consumption passed: $out/local_game_asset_consumption_manifest.json"
