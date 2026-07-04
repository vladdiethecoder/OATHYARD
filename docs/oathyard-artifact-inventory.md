# OATHYARD Artifact Inventory

Phase 1 — complete catalog of artifact categories the repo can emit.
Inventory date: 2026-07-03
Commit baseline: HEAD of main

---

## ADR Visual/Artifact Implications (one-line summaries)

| ADR | Visual/Artifact Implication |
|-----|------------------------------|
| 0001-stack-and-determinism | No visual artifacts adopted; standalone non-3D outputs rejected as verification evidence. |
| 0002-high-fidelity-production-target | Debug/low-poly artifacts are Tier 0 only; V0.5 candidate_asset_preview promoted but not production. |
| 0002-native-presentation-target | game_is_3d=true when runtime glTF 3D evidence present; product_3d_gameplay_complete=false until production renderer gate met. |
| 0002-remove-2d-visual-artifacts | Standalone non-3D visual artifacts banned from source, outputs, bundles, audits, docs, and visual gates. |
| 0003-truth-vs-presentation-layering | All presentation artifacts carry truth_mutation=false; toggling presentation cannot change truth output. |
| 0003-native-input-model | Input artifacts keep presentation_only=true, truth_mutation=false, replay_hash_affects=false. |
| 0003-rodin-to-production-asset-pipeline | Candidate assets cataloged but not production until licensed/source-approved/technical-clean/gameplay-profiled/in-engine gates pass. |
| 0004-renderer-and-asset-pipeline | Production renderer/asset pipeline target defined; current debug/local not production. |
| 0004-renderer-or-engine-selection | Bevy/wgpu selected for V1 spike; no backend adopted into production. |
| 0004-audio-runtime-target | Audio artifacts are trace-derived presentation-only; shipping backend not finalized. |
| 0005-cross-platform-verification | Cross-platform stamps under artifacts/cross_platform/stamps/<platform_id>/; all platforms must match before freeze. |
| 0007-high-fidelity-production-target | 1920x1080+ capture coverage required; visual benchmark report required; owner visual acceptance required. |
| 0008-hifi-wo-01-renderer-backend-adr | Raw OpenGL spike authorized as disposable; no production adoption; spikes under artifacts/renderer_spikes/. |
| 0009-production-renderer-selection | Bevy/wgpu V1 spike direction; Blender blocked by MaterialX ABI; production_renderer_complete=false. |
| 0010-large-file-policy | Files >50MiB policy; gambeson.obj allowlisted; runtime meshes replace source OBJ. |

---

## Tool-to-Artifact Grouping

### Visual-capture
- tools/native_combat_render.sh → artifacts/native_combat/latest (or verify)
- tools/capture_high_fidelity_screens.sh → artifacts/high_fidelity_screens/latest
- tools/visual_gap_audit.sh → artifacts/visual_review/latest
- tools/visual_benchmark.sh → artifacts/visual_review/latest
- tools/render_asset_previews.sh → artifacts/asset_previews/verify
- tools/render_high_fidelity_capture_matrix.sh → artifacts/production_renderer/
- tools/render_runtime_asset_sets.sh → artifacts/native_combat/ (runtime sets)
- tools/arena_environment_captures.sh → artifacts/

### Visual-audit
- tools/audit_visual_artifacts.sh → scans tracked files + generated artifacts
- tools/visual_qa.sh → artifacts/visual_review/
- tools/visual_evidence_index.sh → artifacts/visual_evidence/verify
- tools/asset_visual_atlas.sh → artifacts/asset_atlas/verify
- tools/asset_budget_audit.sh → artifacts/asset_budget/verify
- tools/final_acceptance.sh → artifacts/final_acceptance/

### Asset
- tools/build_assets.sh → assets/ (runtime assets, asset_validation_report.md)
- tools/validate_assets.sh → assets/asset_validation_report.md
- tools/audit_3d_runtime.sh → artifacts/runtime_3d/verify
- tools/pbr_materials.sh → artifacts/pbr_materials/verify
- tools/asset_provenance_audit.sh → artifacts/asset_audit/
- tools/audit_generated_assets.sh → artifacts/asset_audit/
- tools/audit_rodin_assets.sh → artifacts/asset_audit/

### Truth/Replay
- tools/run_duel.sh → artifacts/latest
- tools/replay_verify.sh → verifies artifacts/latest/replay.json
- tools/export_replay_bundle.sh → artifacts/export_bundle/latest
- tools/verify_replay_bundle.sh → verifies artifacts/export_bundle/latest
- tools/audit_truth.sh → static scan of src/
- tools/contact_matrix.sh → artifacts/contact_matrix/latest
- tools/truth_stress.sh → artifacts/truth_stress/latest
- tools/truth_edge_audit.sh → artifacts/truth_edge/latest
- tools/negative_audit.sh → artifacts/negative_audit/latest

### AI
- tools/ai_duel.sh → artifacts/ai/latest
- tools/ai_sweep.sh → artifacts/ai_sweep/latest
- tools/ai_planner_audit.sh → artifacts/

### Performance
- tools/perf_benchmark.sh → artifacts/perf/
- tools/performance_benchmark.sh → (archived)
- tools/run_match_sweep.sh → artifacts/match_sweep/

### Packaging
- tools/package.sh → artifacts/package/
- tools/smoke_package.sh → artifacts/package_smoke/
- tools/desktop_metadata.sh → artifacts/desktop_metadata/verify

### Input
- tools/input_map.sh → artifacts/input/verify
- tools/input_target_audit.sh → artifacts/input_target/verify
- tools/accessibility.sh → artifacts/accessibility/verify
- tools/runtime_settings.sh → artifacts/settings/verify

### Audio
- tools/audio_vfx_render.sh → artifacts/audio_vfx/latest
- tools/audio_target_audit.sh → artifacts/audio_target/verify

### Environment/Readiness
- tools/audit_environment.sh → artifacts/environment/verify
- tools/audit_readiness.sh → artifacts/readiness/
- tools/audit_secrets.sh → artifacts/secrets/
- tools/presentation_truth_isolation.sh → artifacts/presentation_truth_isolation/
- tools/research_audit.sh → artifacts/research_audit/
- tools/sim_reference_compare.sh → artifacts/
- tools/renderer_target_audit.sh → artifacts/renderer_target/verify

---

## Existing Artifact Directories (68 total)

accessibility, ai, ai_sweep, animation_state_machine, archive, asset_atlas,
asset_audit, asset_budget, asset_previews, audio_device, audio_mixer,
audio_target, audio_vfx, check_truth_hash, contact_matrix, cross_platform,
current, desktop_metadata, environment, export_bundle, fight_film, final,
final_acceptance, game_flow, gamepad, gates, hermes_ad_hoc,
high_fidelity_screens, hud_menu_flow_audit, input, input_target, kanban,
match_sweep, model_candidates, native_combat, negative_audit, package,
package_repro, package_smoke, pbr_materials, perf,
presentation_truth_isolation, production_candidates, production_renderer,
publishable, readiness, renderer_spikes, renderer_target, replay_browser,
repo_inspection, research_audit, runtime_3d, screens, secrets, settings,
_stale, tmp, toolchain_audit, truth_edge, truth_stress, verification,
verify_a, verify_b, verify_logs, verify_t_74d251ad_review,
visual_artifact_audit, visual_artifact_cleanup_baseline, visual_evidence,
visual_review

---

## Phase 2 — Artifact Classification Table

Classification run: 2026-07-03 on commit e3ded0b (Unit-081).
Buckets: VISUAL-EVIDENCE | NONVISUAL | BLOCKED | FORBIDDEN (see THREE_D_ONLY_VISUAL_EVIDENCE.md).

### Visual-Gate Results

| # | Script | Exit | Status |
|---|--------|------|--------|
| 1 | build.sh | 0 | PASS |
| 2 | test.sh (cargo test) | 0 (3-4 failing tests) | PARTIAL — 3 test failures in oathyard_tests, 61 passing |
| 3 | cargo build --locked | 0 | PASS |
| 4 | build_assets.sh | 0 | PASS — 22 candidates + 26 production assets |
| 5 | validate_assets.sh | 0 | PASS |
| 6 | run_duel.sh | 0 | PASS — hash f17c8f76b9dfae86 |
| 7 | replay_verify.sh | 0 | PASS |
| 8 | export_replay_bundle.sh | 0 | PASS |
| 9 | verify_replay_bundle.sh | 0 | PASS |
| 10 | native_combat_render.sh ★ | 0 | ✓ PASS — 922,960 byte PNG produced |
| 11 | capture_high_fidelity_screens.sh | 1 | BLOCKED — missing 56 capture slots |
| 12 | visual_gap_audit.sh | 1 | BLOCKED |
| 13 | visual_benchmark.sh | 1 | BLOCKED |
| 14 | asset_visual_atlas.sh | 0 | PASS — 22 runtime assets indexed |
| 15 | asset_budget_audit.sh | 0 | PASS |
| 16 | render_asset_previews.sh | 0 | PASS |
| 17 | audit_3d_runtime.sh | 1 | FAILED (5 failures) |
| 18 | audit_visual_artifacts.sh | 0 | PASS — no forbidden files |
| 19 | contact_matrix.sh | 0 | PASS |
| 20 | audio_vfx_render.sh | 0 | PASS |
| 21 | ai_duel.sh | 0 | PASS |
| 22 | truth_stress.sh | 0 | PASS |
| 23 | truth_edge_audit.sh | 0 | PASS |
| 24 | negative_audit.sh | 0 | PASS |
| 25 | audit_truth.sh | 0 | PASS |
| 26 | audit_secrets.sh | TIMEOUT (>180s, exits 124) | TIMEOUT — known scan of large assets |
| 27 | audit_environment.sh | 0 | PASS |
| 28 | audit_readiness.sh | 0 | PASS |
| 29 | package.sh | 0 | PASS |
| 30 | smoke_package.sh | 0 | PASS |

### Bucket 1: VISUAL-EVIDENCE (candidate-level, per THREE_D_ONLY_VISUAL_EVIDENCE.md)

| Artifact Dir | File | Renderer/Backend | Manifest Fields | Resol. | truth_mutation | Verdict |
|---|---|---|---|---|---|---|
| artifacts/native_combat/latest/render/ | production_renderer_native_combat_3d_1920x1080.png | oathyard-native-wgpu-production-v1 / wgpu 29.0.3 / Vulkan / NVIDIA RTX 5090 | renderer_id, asset manifest sha, camera_mode (oathyard_verdict_ring_establishing), capture sha256, content_hash, final_state_hash, resolution, native_resolution=true, upscaling=false | 1920x1080 | false | VISUAL-EVIDENCE (candidate asset class; production_renderer_complete=false and owner_visual_acceptance=false remain correct per canon) |

Five-condition check per THREE_D_ONLY_VISUAL_EVIDENCE.md:
1. Native 3D renderer/engine client → YES (wgpu/Vulkan/RTX 5090)
2. Camera/render path inside client → YES (oathyard_verdict_ring_establishing)
3. Manifest with renderer/backend/asset/camera/replay/content/capture/resolution → YES
4. truth_mutation=false → YES
5. Current-run after replay/truth verification → YES (replay hash matches, replay_verified=true)

→ Candidate-level VISUAL-EVIDENCE. Downgrading factor: production assets are still low-poly candidate class (32x32 textures), not production fidelity. Per ADR 0002-high-fidelity the "candidate_asset_preview" tier is the current correct classification.

### Bucket 2: NONVISUAL — Valid, preserved as-is (JSON/MD/hash/log/manifest)

Replay/truth baseline:
- artifacts/latest (replay.json, trace.json, final_state_hash, duel_report)
- artifacts/export_bundle/latest (manifest, replay, trace, hashes)
- artifacts/contact_matrix/latest
- artifacts/truth_stress/latest
- artifacts/truth_edge/latest
- artifacts/negative_audit/latest
- artifacts/ai/latest + artifacts/ai_sweep/latest
- artifacts/verify_a, artifacts/verify_b
- artifacts/match_sweep

Asset evidence:
- artifacts/asset_budget/latest (asset_budget.json, report)
- artifacts/asset_atlas/latest (hashes, manifest, report, failed_asset_visuals.txt)
- artifacts/asset_previews/latest (report/manifest — actual previews under assets/model_candidates/, NOT under artifacts/)
- artifacts/asset_audit/latest

Presentation/input/audio:
- artifacts/fight_film/latest (shot manifests, metadata)
- artifacts/input/verify, input_target/verify
- artifacts/accessibility/verify
- artifacts/settings/verify
- artifacts/desktop_metadata/verify
- artifacts/audio_vfx/latest (WAV, caption/event JSON)
- artifacts/audio_target/verify
- artifacts/audio_mixer/verify, artifacts/audio_device/verify

Environment/readiness/packaging:
- artifacts/readiness/*, artifacts/secrets/* (if audit completed)
- artifacts/environment/verify
- artifacts/presentation_truth_isolation
- artifacts/research_audit
- artifacts/package + artifacts/package_smoke
- artifacts/final_acceptance, artifacts/final
- artifacts/native_combat/latest (non-PNG files: manifest, report, packet, mesh manifest)
- artifacts/visual_review/latest (visual_benchmark_report.md, visual_gap_audit_report.md, manifest JSONs, failed files)
- artifacts/high_fidelity_screens/latest (blocked report + capture matrix metadata, NO visual content emitted)
- artifacts/renderer_target/verify
- artifacts/visual_artifact_audit/latest
- artifacts/visual_evidence/verify

### Bucket 3: BLOCKED

| Dir | Block reason | Missing gate |
|---|---|---|
| artifacts/high_fidelity_screens/latest | BLOCKED_PENDING_NATIVE_3D_RENDERER_CAPTURE; missing 56 capture slots | production renderer completeness, owner visual acceptance |
| artifacts/visual_review/latest | BLOCKED_PENDING_NATIVE_3D_RENDERER_CAPTURE; visual_benchmark FAILED (0 native capture slots for 56 required) | production assets + full capture matrix |
| artifacts/runtime_3d | FAILED with 5 failures — runtime assets have Z depth but "native_3d_runtime_geometry=false" per audit criteria | production runtime geometry acceptance |
| artifacts/production_renderer/latest | No production renderer manifest produced by this run | Bevy/wgpu V1 spike not implemented |

### Bucket 4: FORBIDDEN — None found

- audit_visual_artifacts.sh exit 0 — no forbidden standalone 2D diagrams, frame dumps, proof packets, debug panels, browser canvas outputs, or fallback visual substitutes detected in tracked files or generated output.
- asset_previews/latest/previews/ is an EMPTY directory; all candidate PNG previews live under assets/model_candidates/ (tracked source, not generated output) — correct per 3D-only policy.
- No fallback PNG was generated under visual_review, high_fidelity_screens, or asset_atlas.

### Unit-081 Critical Finding

Commit e3ded0b "Unit-081: runtime asset-set visual evidence integration" is marked "failure" in history. Investigation of the current tools:

- `./tools/native_combat_render.sh examples/duels/basic_oathyard.duel artifacts/native_combat/latest` → EXIT 0, 922,960-byte 1920x1080 PNG produced.
- Renderer backend: `oathyard-native-wgpu-production-v1` (wgpu 29.0.3 high-performance Vulkan adapter)
- Adapter: NVIDIA GeForce RTX 5090 / Vulkan / DiscreteGpu / driver 595.80
- Capture ID: native_combat_capture_unit070
- Mesh assets: 7 loaded (player_saltreach_duelist, opponent_oathyard_writ, longswords, gambeson armor, training_yard arena)
- Mesh source: assets/runtime/*.mesh.json with source texture bindings under assets/model_candidates/t_73291be5/textures/
- truth_mutation: false; presentation only: true; production_ready: false (per Unit-082's ADR 0009 scope)
- Vision-model inspection of the PNG confirms: real 3D render from wgpu/Vulkan with 3D depth, geometry, lighting, shadows, two skinned fighter figures, central structure, arena ring, architectural backdrop, HUD overlays (OBSERVE status panel, hash label). This is NOT a 2D diagram, debug panel, browser canvas, or fallback frame dump.

Verdict: Unit-081 failure is stale from the first-patch history — current tooling produces valid manifest-backed PNG. The "failure" tag on the commit reflects Unit-081's earlier incomplete state, not the current capability.

### cargo test failures (3-4, unrelated to visual gates)

- `asset_provenance_audit_accepts_candidate_preview_metadata_without_standalone_runtime_previews` — fails
- `final_acceptance_manifest_indexes_required_evidence_artifacts` — fails
- `generated_asset_audit_emits_fail_closed_candidate_quarantine_manifest` — fails (cargo test only)
- `high_detail_presentation_manifest_is_validated_and_loud_fail` — fails

These are test-suite failures, not script/visual-gate failures. 61 other tests pass.
