# OATHYARD Agent Rules

## Design Knowledge Base

`docs/DESIGN_KB.md` is a consolidated design reference synthesizing identity, canon, mechanics, systems, content, art direction, architecture, roadmap, acceptance gates, and source map. Use it for orientation. It does not override the canon sources below.

## Canon Precedence

Read and preserve this order before changing behavior:

1. `docs/design/GAME_CANON.md`
2. `docs/design/DEMO_SCOPE.md`
3. `ACCEPTANCE_MAP.md`
4. `AGENTS.md` / `CLAUDE.md`
5. PRDs/specs
6. Code comments

## Verification Commands

Use these commands before reporting success:

```sh
./tools/build.sh
./tools/test.sh
cargo build --locked
cargo test --locked
./tools/verify.sh
```

For focused replay work:

```sh
./tools/run_duel.sh examples/duels/basic_oathyard.duel --out artifacts/latest
./tools/replay_verify.sh artifacts/latest/replay.json
./tools/export_replay_bundle.sh artifacts/latest/replay.json artifacts/export_bundle/latest
./tools/verify_replay_bundle.sh artifacts/export_bundle/latest
./tools/audit_truth.sh
./tools/audit_secrets.sh
./tools/audit_environment.sh
./tools/contact_matrix.sh
./tools/build_assets.sh
./tools/validate_assets.sh
./tools/render_asset_previews.sh
./tools/research_audit.sh
./tools/presentation_truth_isolation.sh
./tools/sim_reference_compare.sh
./tools/ai_planner_audit.sh
./tools/capture_high_fidelity_screens.sh
./tools/visual_benchmark.sh
./tools/final_acceptance.sh
./tools/asset_budget_audit.sh
./tools/asset_visual_atlas.sh
./tools/audit_3d_runtime.sh
./tools/audit_readiness.sh
./tools/run_match_sweep.sh
./tools/performance_benchmark.sh
./tools/perf_benchmark.sh
./tools/input_map.sh
./tools/accessibility.sh
./tools/runtime_settings.sh
./tools/desktop_metadata.sh
./tools/native_combat_render.sh
./tools/renderer_target_audit.sh
./tools/input_target_audit.sh
./tools/audio_target_audit.sh
./tools/audio_vfx_render.sh
./tools/ai_duel.sh
./tools/truth_stress.sh
./tools/truth_edge_audit.sh
./tools/negative_audit.sh
./tools/package.sh
./tools/smoke_package.sh
./tools/audit_visual_artifacts.sh
./tools/visual_gap_audit.sh
./tools/visual_evidence_index.sh
./tools/ai_sweep.sh
```

### Visual Inspection Protocol

Run visual gates in this order (each depends on prior verification):

1. Build + truth baseline: `build.sh`, `test.sh`, `cargo build --locked`, `cargo test --locked`
2. Asset pipeline: `build_assets.sh`, `validate_assets.sh`
3. Duel + replay: `run_duel.sh`, `replay_verify.sh`, `export_replay_bundle.sh`, `verify_replay_bundle.sh`
4. Visual gates: `native_combat_render.sh`, `capture_high_fidelity_screens.sh`, `visual_gap_audit.sh`, `visual_benchmark.sh`, `render_asset_previews.sh`, `asset_visual_atlas.sh`, `asset_budget_audit.sh`, `audit_3d_runtime.sh`
5. Visual artifact audit: `audit_visual_artifacts.sh`
6. Final acceptance: `final_acceptance.sh`

A capture counts as visual evidence only when ALL five conditions in `docs/visual/THREE_D_ONLY_VISUAL_EVIDENCE.md` (section "Allowed visual evidence") are satisfied verbatim. Do not paraphrase or interpret these conditions.

Every artifact directory produced by a visual gate must be classified into exactly one bucket:

- **VISUAL-EVIDENCE**: native 3D capture + complete manifest + `truth_mutation=false` + current-run (generated after replay/truth verification)
- **NONVISUAL**: JSON/MD/hash/log/manifest output; valid, preserved as-is
- **BLOCKED**: no native 3D capture exists; tool wrote blocked JSON/MD status
- **FORBIDDEN**: standalone 2D diagram, frame dump, proof packet, debug panel, browser canvas output, or fallback capture presented as visual evidence

Unit-081 status (commit e3ded0b): `./tools/native_combat_render.sh` currently produces a valid 1920x1080 PNG with complete manifest (renderer_id, camera_mode, replay hash, truth_mutation=false). The commit message "failure" reflects earlier incomplete state; the current tooling is functional. Verify by checking exit code 0 and presence of `production_renderer_native_combat_3d_1920x1080.png` under `artifacts/native_combat/latest/render/`.

### Artifact Catalog

Current as of commit e3ded0b. Re-verify before relying.

| Artifact Directory | Classification | Notes |
|---|---|---|
| artifacts/latest | NONVISUAL | replay.json, trace.json, final_state_hash.txt |
| artifacts/export_bundle/latest | NONVISUAL | bundle with manifest, replay, hashes |
| artifacts/native_combat/latest/render/ | VISUAL-EVIDENCE | 1920x1080 PNG from wgpu/Vulkan/RTX 5090, manifest-backed |
| artifacts/high_fidelity_screens/latest | BLOCKED | missing 56 capture slots |
| artifacts/visual_review/latest | BLOCKED | visual_benchmark failed, 0 native slots |
| artifacts/runtime_3d | BLOCKED | audit failed with 5 failures |
| artifacts/production_renderer/latest | BLOCKED | no manifest produced |
| artifacts/contact_matrix/latest | NONVISUAL | contact packets |
| artifacts/truth_stress/latest | NONVISUAL | stress test results |
| artifacts/truth_edge/latest | NONVISUAL | edge case audit |
| artifacts/negative_audit/latest | NONVISUAL | negative input audit |
| artifacts/ai/latest | NONVISUAL | AI duel artifacts |
| artifacts/ai_sweep/latest | NONVISUAL | AI sweep results |
| artifacts/match_sweep | NONVISUAL | match sweep summary |
| artifacts/verify_a, artifacts/verify_b | NONVISUAL | determinism verification |
| artifacts/asset_budget/latest | NONVISUAL | budget report |
| artifacts/asset_atlas/latest | NONVISUAL | atlas hashes + manifest |
| artifacts/asset_previews/latest | NONVISUAL | manifest only (PNGs under assets/model_candidates/) |
| artifacts/fight_film/latest | NONVISUAL | shot manifests |
| artifacts/input/verify | NONVISUAL | input map artifacts |
| artifacts/accessibility/verify | NONVISUAL | accessibility report |
| artifacts/settings/verify | NONVISUAL | runtime settings |
| artifacts/audio_vfx/latest | NONVISUAL | WAV + caption JSON |
| artifacts/environment/verify | NONVISUAL | environment audit |
| artifacts/readiness | NONVISUAL | readiness audit |
| artifacts/package | NONVISUAL | package tar |
| artifacts/package_smoke | NONVISUAL | smoke test results |
| artifacts/visual_artifact_audit/latest | NONVISUAL | audit report |
| artifacts/renderer_target/verify | NONVISUAL | renderer target audit |

**FORBIDDEN bucket: empty** — `audit_visual_artifacts.sh` exit 0 confirms no forbidden 2D substitutes in tracked files.

## Determinism Rules

- Truth runs at fixed 120 Hz.
- Authoritative simulation uses fixed-point/integer truth only.
- No hidden RNG, wall-clock time, gameplay floats, unordered truth iteration, or render/UI/audio writes into gameplay truth.
- Replay is authoritative evidence and must fail loudly on mismatch.
- Replay browser/indexing must verify through the same replay path and surface corrupt replays loudly.
- Truth edge audits must preserve fixed-point/permille overflow policy, capability clamps, deterministic contact ordering, and replay schema loud-failure behavior.
- Negative input audits must keep malformed scenarios, content manifests, replay files, and replay export bundles failing loudly with specific errors.
- Fight-film/camera artifacts must be generated from verified replay/trace data only and must remain presentation-only.
- Native presentation target audits must keep current visual status blocked until a production renderer backend and owner visual acceptance are actually verified.
- Visual evidence is 3D-only: accepted visual evidence must be captured inside a native 3D renderer/engine client with renderer, asset, camera, replay/hash metadata and `truth_mutation=false`.
- Standalone two-dimensional diagrams, frame dumps, proof packets, debug panels, browser canvas output, and fallback visual substitutes must not be generated by normal audits or counted as visual evidence. If native 3D capture is absent, visual status is blocked rather than substituted.
- High-fidelity production target is governed by `docs/decisions/0007-high-fidelity-production-target.md`, `docs/decisions/0002-high-fidelity-production-target.md`, `docs/decisions/0003-truth-vs-presentation-layering.md`, `docs/decisions/0004-renderer-and-asset-pipeline.md`, and `docs/research/FRONTIER_TECH_LEVERAGE.md`; non-native visual substitutes, low-poly glTF, cubes, capsules, debug renders, or metadata-only outputs must never be reported as high-fidelity product presentation.
- Production runtime assets must remain 3D: source-built glTF geometry must have nonzero Z depth, asset audits must fail on flat production glTFs, and native combat projection must consume Z depth after truth hashes.
- Native input target audits must keep keyboard/mouse/controller schema evidence separate from physical controller, Steam Deck hardware, and owner input acceptance claims.
- Audio runtime target audits must keep deterministic mixer/local playback evidence separate from shipping backend finalization, platform loudness acceptance, and owner audio acceptance claims.
- Accessibility/settings work must remain presentation/input-only and must not alter truth, action costs, contact packets, injuries, capabilities, or hashes.
- Linux desktop metadata may package a `.desktop` file and icon, but AppStream/store metadata must stay blocked while the project is pending/unlicensed and external release gates are false.
- Readiness drift audits must fail if source/package docs or manifests claim public-demo, release-candidate, owner-final, legal, trademark, or store readiness before external gates are completed.
- Secrets audits must fail on private keys, credentials, service tokens, webhook secrets, or non-placeholder secret assignments in source, text artifacts/logs, and package content.
- Environment audits must record required local build/test/package tools, optional graphics/DCC/audio tooling, pkg-config library availability, and native runtime surfaces without claiming a clean VM/container or external release readiness.
- Generated replay-relevant artifacts must be timestamp-free and path-free.

## Forbidden Shortcuts

- Do not add HP, hit points, arbitrary damage numbers, armor points, DPS, crit chance, super meter, perks, unlock stats, or speed/damage bonuses as truth.
- Do not hard-code only the demonstrated trace while pretending to have a general system.
- Do not use canned animation or pre-decided hit results as gameplay truth.
- Do not regress production runtime visuals to flat/non-3D geometry or claim a browser/HTML artifact as native 3D product presentation.
- Do not claim Elden Ring/For Honor-class ambition has been met without current-run native high-fidelity captures, pixel inspection, visual benchmark report, and owner visual acceptance.
- AI/scripted seats may emit only legal planned actions and directional influence; truth must decide contacts, injuries, capability changes, and hashes.
- MotionBricks-style systems, Warp, Isaac Lab, Newton, MJWarp, PhysX, Chrono, Audio2Face, generative 3D, renderer physics, VFX, audio, and camera systems must stay outside runtime authoritative truth unless a separate deterministic replay/hash ADR promotes them with current evidence. PresentationBricks (the internal MotionBricks-inspired layer) may use indeterministic motion generation provided truth_mutation remains false and the canonical truth hash is preserved.
- Do not weaken tests, expected outputs, audits, or canon to make verification pass.
- Do not claim native public demo readiness, release-candidate readiness, owner visual acceptance, legal clearance, or trademark clearance.
- Do not commit or package credentials, API keys, tokens, private keys, webhooks, or store secrets.
- Do not introduce Unity, Unreal, Godot, browser-first frameworks, vendored blobs, network services, telemetry, installers, or release packaging for this slice.
- Do not claim full-game completion while native rendering, assets, UI, audio/VFX, AI, replay/fight-film, packaging, or quality gates are missing or blocked.

## Final Report Requirements

Report exact commands run and whether they passed. Include artifact paths, final hash, replay result, determinism evidence, loadout/injury variation evidence, any skipped checks, and whether canon/forbidden-shortcut constraints were reviewed.
