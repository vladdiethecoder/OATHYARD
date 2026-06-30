#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/renderer_target/verify}"
env_json="${2:-artifacts/environment/verify/environment_audit.json}"
combat_json="${3:-artifacts/native_combat/verify/native_combat_render_manifest.json}"
roster_json="${4:-artifacts/native_roster/verify/native_roster_showcase_manifest.json}"
adr="${5:-docs/decisions/0002-native-presentation-target.md}"

python3 - "$out" "$env_json" "$combat_json" "$roster_json" "$adr" <<'PY'
import json
import sys
from pathlib import Path

out = Path(sys.argv[1])
env_path = Path(sys.argv[2])
combat_path = Path(sys.argv[3])
roster_path = Path(sys.argv[4])
adr_path = Path(sys.argv[5])
backend_path = combat_path.with_name("native_renderer_backend_manifest.json")
post_hash_input_path = combat_path.with_name("native_renderer_post_hash_input.json")
capture_hook_path = combat_path.with_name("native_renderer_capture_hook.json")
mutation_proof_path = combat_path.with_name("native_renderer_truth_mutation_proof.json")
out.mkdir(parents=True, exist_ok=True)
checks = []

def read_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))

def check(check_id: str, passed: bool, detail: str) -> None:
    checks.append({"id": check_id, "passed": bool(passed), "detail": detail})

try:
    env = read_json(env_path)
    combat = read_json(combat_path)
    roster = read_json(roster_path)
    backend = read_json(backend_path)
    post_hash_input = read_json(post_hash_input_path)
    capture_hook = read_json(capture_hook_path)
    mutation_proof = read_json(mutation_proof_path)
    adr_text = adr_path.read_text(encoding="utf-8")
except Exception as error:  # noqa: BLE001 - audit artifact records exact failure.
    env = {}
    combat = {}
    roster = {}
    backend = {}
    post_hash_input = {}
    capture_hook = {}
    mutation_proof = {}
    adr_text = ""
    check("inputs_readable", False, str(error))
else:
    check("inputs_readable", True, "3D renderer evidence artifacts loaded, including backend/input/capture/proof")

pkg_config = {entry.get("name"): entry for entry in env.get("pkg_config_libraries", [])}
native_graphics = env.get("native_graphics", {})
check("adr_present", adr_path.is_file() and "Native Presentation Target" in adr_text, str(adr_path))
check("environment_schema", env.get("schema") == "oathyard.environment_audit.v1", env_path.as_posix())
check("x11_pkg_config_available", bool(pkg_config.get("x11", {}).get("available")), "x11")
check("display_surface_available", bool(native_graphics.get("display_set") or native_graphics.get("wayland_display_set")), "DISPLAY or WAYLAND_DISPLAY")

check("combat_schema", combat.get("schema") == "oathyard.native_combat_render.v1", combat_path.as_posix())
check("combat_renderer_is_3d", combat.get("renderer") == "native-software-3d", str(combat.get("renderer", "")))
check("combat_truth_after_hash_source", combat.get("source") == "truth-after-hash-duel-result", str(combat.get("source", "")))
check("combat_truth_mutation_false", combat.get("truth_mutation") is False, "native combat")
check("combat_presentation_only", combat.get("presentation_only") is True, "native combat")
check("combat_game_is_3d_true", combat.get("game_is_3d") is True, str(combat.get("game_is_3d")))
check("combat_runtime_gltf_geometry_projected", combat.get("runtime_gltf_geometry_projected") is True, "native combat")
check("combat_native_3d_runtime_geometry", combat.get("native_3d_runtime_geometry") is True, "native combat")
check("combat_projection_uses_z_depth", combat.get("projection_uses_z_depth") is True, "native combat")
check("combat_state_frames", int(combat.get("state_frame_count", 0)) >= 12, str(combat.get("state_frame_count")))
check("combat_continuous_player_loop", combat.get("continuous_player_facing_3d_render_loop") is True, str(combat.get("continuous_player_facing_3d_render_loop")))
check("combat_high_res_visual_floor", combat.get("high_res_capture_debug_overlay_minimized") is True and "source-backed shaded presentation-floor" in str(combat.get("high_res_capture_visual_floor", "")), str(combat.get("high_res_capture_visual_floor", "")))
product_captures = combat.get("product_mode_clean_captures", [])
check("combat_product_mode_clean_capture_count", int(combat.get("product_mode_clean_capture_count", 0)) >= 6 and len(product_captures) >= 6, str(combat.get("product_mode_clean_capture_count", 0)))
check("combat_product_mode_clean_capture_source", bool(product_captures) and all(item.get("source") == "software-product-renderer-clean-frame-after-truth-hash" and item.get("debug_overlay") is False and item.get("production_asset_evidence") is True for item in product_captures), "clean product-mode framebuffer source")
check("combat_product_mode_required_roles", {"fighter_select_loadout", "verdict_ring_establishing", "pre_contact_spacing", "combat_contact_readability", "material_impact_closeup", "injury_consequence_readability"}.issubset({item.get("capture_role") for item in product_captures}), str(sorted({item.get("capture_role") for item in product_captures})))
check("combat_high_res_scope_honest", combat.get("owner_visual_acceptance") is False and combat.get("product_3d_gameplay_complete") is False, "owner/product acceptance remains false")
player_loop = combat.get("player_facing_loop", {})
check("combat_player_loop_screens", int(player_loop.get("screen_count", 0)) >= 5, str(player_loop.get("screen_count", 0)))
check("combat_player_loop_timing_outside_truth", player_loop.get("timing_recorded_outside_truth") is True, "player loop timing outside truth")
check("combat_player_loop_truth_unchanged", player_loop.get("truth_hash_before_loop", "") == player_loop.get("truth_hash_after_loop", "") and player_loop.get("truth_hash_before_loop", "") != "", "player loop truth hash unchanged")
check("combat_motion_frames", int(combat.get("motion_frame_count", 0)) >= 21, str(combat.get("motion_frame_count")))
viewports = combat.get("software_3d_viewports", [])
check("combat_software_3d_viewports", len(viewports) >= 2, str(len(viewports)))
check("combat_depth_sorted_mesh_viewports", bool(viewports) and all(v.get("projection_model") == "integer_depth_sorted_mesh_raster" and v.get("depth_sorted") is True for v in viewports), "depth-sorted 3D viewports")
sequence = combat.get("software_3d_sequence", {})
check("combat_replay_3d_sequence", int(sequence.get("frame_count", 0)) >= 21 and sequence.get("projection_model") == "integer_depth_sorted_mesh_raster", str(sequence.get("frame_count", 0)))

check("backend_schema", backend.get("schema") == "oathyard.renderer_backend.v1", backend_path.as_posix())
check("backend_decision_recorded", backend.get("backend_id") == "native-x11-software-3d-depth-raster" and backend.get("project_dependency_adopted") is False, str(backend.get("backend_id", "")))
check("backend_continuous_loop", backend.get("continuous_render_loop") is True and int(backend.get("continuous_render_loop_frames", 0)) >= 120, str(backend.get("continuous_render_loop_frames", 0)))
check("backend_submission_and_binding", backend.get("mesh_submission") is True and backend.get("texture_binding") is True and backend.get("material_binding") is True, "mesh/texture/material")
check("backend_capture_hook_integrated", backend.get("capture_hook_integrated") is True and backend.get("capture_hook_artifact") == "native_renderer_capture_hook.json", str(backend.get("capture_hook_artifact", "")))
check("backend_truth_mutation_proof", backend.get("mutation_proof_all_equal") is True and backend.get("mutation_proof_artifact") == "native_renderer_truth_mutation_proof.json", str(backend.get("mutation_proof_all_equal")))
check("backend_post_hash_input_artifact", backend.get("post_hash_input_artifact") == "native_renderer_post_hash_input.json", str(backend.get("post_hash_input_artifact", "")))
check("backend_camera_metadata", len(backend.get("camera_metadata", [])) >= 5 and all(item.get("presentation_only") is True and item.get("truth_mutation") is False for item in backend.get("camera_metadata", [])), str(len(backend.get("camera_metadata", []))))

check("post_hash_input_schema", post_hash_input.get("schema") == "oathyard.renderer_post_hash_input.v1", post_hash_input_path.as_posix())
check("post_hash_input_truth_read_only", post_hash_input.get("post_hash_only") is True and post_hash_input.get("presentation_only") is True and post_hash_input.get("truth_mutation") is False, "post hash input flags")
frame_schema = set(post_hash_input.get("frame_schema", []))
check("post_hash_input_frame_schema_explicit", {"loop_frame_index", "truth_frame", "scheduled_ms", "screen", "camera_mode", "asset_ids", "material_ids", "event_ids", "damage_wear_mask", "capture_file"}.issubset(frame_schema), str(sorted(frame_schema)))
screen_inputs = post_hash_input.get("screen_inputs", [])
check("post_hash_input_screen_inputs", len(screen_inputs) >= 5 and all(item.get("camera_mode") for item in screen_inputs), str(len(screen_inputs)))
check("post_hash_input_hashes_present", bool(post_hash_input.get("replay_json_hash")) and bool(post_hash_input.get("trace_json_hash")) and bool(post_hash_input.get("final_state_hash")), "replay/trace/final hashes")

check("capture_hook_schema", capture_hook.get("schema") == "oathyard.renderer_capture_hook.v1", capture_hook_path.as_posix())
captures = capture_hook.get("captures", [])
check("capture_hook_high_res", int(capture_hook.get("high_resolution_capture_count", 0)) >= 1 and any(int(item.get("width", 0)) >= 1920 and int(item.get("height", 0)) >= 1080 for item in captures), str(capture_hook.get("high_resolution_capture_count", 0)))
check("capture_hook_truth_read_only", bool(captures) and all(item.get("capture_after_truth_hash") is True and item.get("presentation_only") is True for item in captures), str(len(captures)))
check("capture_hook_hashes_present", bool(capture_hook.get("hook_hash")) and all(item.get("frame_hash") for item in captures), "hook/frame hashes")

check("mutation_proof_schema", mutation_proof.get("schema") == "oathyard.renderer_truth_mutation_proof.v1", mutation_proof_path.as_posix())
check("mutation_proof_all_equal", mutation_proof.get("all_equal") is True and mutation_proof.get("truth_mutation") is False, str(mutation_proof.get("all_equal")))
before = mutation_proof.get("before", {})
after = mutation_proof.get("after", {})
check("mutation_proof_hashes_unchanged", bool(before) and before == after, "before == after")

check("roster_schema", roster.get("schema") == "oathyard.native_roster_showcase.v1", roster_path.as_posix())
check("roster_game_is_3d_true", roster.get("game_is_3d") is True, str(roster.get("game_is_3d")))
check("roster_truth_mutation_false", roster.get("truth_mutation") is False, "native roster")
check("roster_depth_sorted_frames", roster.get("all_frames_depth_sorted") is True, "native roster")
check("roster_shaded_triangles", roster.get("all_frames_shaded_triangles") is True, "native roster")
roster_frames = roster.get("frames", [])
check("roster_projection_model", bool(roster_frames) and all(frame.get("projection_model") == "integer_depth_sorted_mesh_raster" for frame in roster_frames), str(len(roster_frames)))

removed_tokens = [
    "native_" + "game_flow",
    "native_" + "replay_" + "browser",
    "native_" + "window_smoke",
    "native_" + "fight_film_player",
    "raw-" + "x11-ppm",
    "native-" + "software-ppm",
    "spikes/" + "renderer",
    "renderer_" + "spikes",
    "raw_" + "opengl",
    "raw " + "OpenGL",
    "raw-x11-glx-" + "opengl",
    "raw X11/GLX/" + "OpenGL",
]
source_roots = [Path("src"), Path("tools"), Path("docs"), Path("spikes")]
source_hits = []
for root in source_roots:
    if not root.exists():
        continue
    for path in root.rglob("*"):
        if not path.is_file() or path.suffix not in {".rs", ".sh", ".md"}:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        # Exempt decision/roadmap/research prose that legitimately references
        # retired candidates as blocked context; this check targets renderer
        # implementation/script remnants, not fail-closed historical notes.
        is_design_doc = (
            str(path).startswith("docs/decisions/")
            or str(path).startswith("docs/roadmap/")
            or str(path).startswith("docs/research/")
        )
        for token in removed_tokens:
            if token in text and path != Path("tools/renderer_target_audit.sh") and not is_design_doc:
                source_hits.append(f"{path}:{token}")
check("removed_non3d_renderer_sources_absent", not source_hits, "; ".join(source_hits[:20]) if source_hits else "none")

failed = [item for item in checks if not item["passed"]]
manifest = {
    "schema": "oathyard.native_presentation_target.v2",
    "tool": "tools/renderer_target_audit.sh",
    "source": "3d-renderer-only-audit",
    "passed": not failed,
    "failed_check_count": len(failed),
    "check_count": len(checks),
    "checks": checks,
}
(out / "native_presentation_target.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
report = [
    "# OATHYARD Native Presentation Target Audit",
    "",
    f"Status: {'PASSED' if not failed else 'FAILED'}",
    "- Policy: only the native 3D renderer path remains as a renderer implementation.",
    f"- Combat renderer: `{combat.get('renderer', '')}`",
    f"- Checks: `{len(checks)}`",
    f"- Failed checks: `{len(failed)}`",
    "",
    "## Checks",
]
for item in checks:
    report.append(f"- {'PASS' if item['passed'] else 'FAIL'} `{item['id']}`: {item['detail']}")
(out / "native_presentation_target_report.md").write_text("\n".join(report) + "\n", encoding="utf-8")
if failed:
    raise SystemExit(1)
PY

echo "renderer target audit passed: $out"
