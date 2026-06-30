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
```

## Determinism Rules

- Truth runs at fixed 120 Hz.
- Authoritative simulation uses fixed-point/integer truth only.
- No hidden RNG, wall-clock time, gameplay floats, unordered truth iteration, or render/UI/audio writes into gameplay truth.
- Replay is authoritative evidence and must fail loudly on mismatch.
- Replay browser/indexing must verify through the same replay path and surface corrupt replays loudly.
- Truth edge audits must preserve fixed-point/permille overflow policy, capability clamps, deterministic contact ordering, and replay schema loud-failure behavior.
- Negative input audits must keep malformed scenarios, content manifests, replay files, and replay export bundles failing loudly with specific errors.
- Fight-film/camera artifacts and PPM frame renders must be generated from verified replay/trace data only and must remain presentation-only.
- Native presentation target audits must keep raw X11/PPM as local verification backend evidence only until a production renderer backend and owner visual acceptance are actually verified.
- High-fidelity production target is governed by `docs/decisions/0007-high-fidelity-production-target.md`, `docs/decisions/0002-high-fidelity-production-target.md`, `docs/decisions/0003-truth-vs-presentation-layering.md`, `docs/decisions/0004-renderer-and-asset-pipeline.md`, and `docs/research/FRONTIER_TECH_LEVERAGE.md`; raw X11, SVG, PPM, low-poly glTF, diagnostic contact sheets, cubes, capsules, or debug renders must never be reported as high-fidelity product presentation.
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
- Do not regress production runtime visuals to 2D/flat geometry or claim a browser/HTML artifact as native 3D product presentation.
- Do not claim Elden Ring/For Honor-class ambition has been met without current-run native high-fidelity captures, pixel inspection, visual benchmark report, and owner visual acceptance.
- AI/scripted seats may emit only legal planned actions and directional influence; truth must decide contacts, injuries, capability changes, and hashes.
- MotionBricks-style systems, Warp, Isaac Lab, Newton, MJWarp, PhysX, Chrono, Audio2Face, generative 3D, renderer physics, VFX, audio, and camera systems must stay outside runtime authoritative truth unless a separate deterministic replay/hash ADR promotes them with current evidence.
- Do not weaken tests, expected outputs, audits, or canon to make verification pass.
- Do not claim native public demo readiness, release-candidate readiness, owner visual acceptance, legal clearance, or trademark clearance.
- Do not commit or package credentials, API keys, tokens, private keys, webhooks, or store secrets.
- Do not introduce Unity, Unreal, Godot, browser-first frameworks, vendored blobs, network services, telemetry, installers, or release packaging for this slice.
- Do not claim full-game completion while native rendering, assets, UI, audio/VFX, AI, replay/fight-film, packaging, or quality gates are missing or blocked.

## Final Report Requirements

Report exact commands run and whether they passed. Include artifact paths, final hash, replay result, determinism evidence, loadout/injury variation evidence, any skipped checks, and whether canon/forbidden-shortcut constraints were reviewed.
