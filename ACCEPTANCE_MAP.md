# OATHYARD Acceptance Map

Status date: 2026-06-29

This file is the acceptance bridge between canon, roadmap, local verification, and external publishing gates. It exists because `AGENTS.md` places `ACCEPTANCE_MAP.md` third in source precedence after `docs/design/GAME_CANON.md` and `docs/design/DEMO_SCOPE.md`.

## Active Goal

Make OATHYARD a complete high-fidelity native-PC 3D emergent melee simulation game without degrading canon, determinism, source-backed assets, accessibility evidence, packaging integrity, or readiness honesty.

The high-fidelity target is recorded in `docs/decisions/0007-high-fidelity-production-target.md`. The existing local publishable package gate is a lower technical gate: it may pass while the high-fidelity production renderer, production assets, owner visual acceptance, public-demo readiness, and release-candidate readiness remain false.

`publishable` is split into two gates:

1. **Local publishable package** — the source tree can build a native package, smoke-run it from a clean unpacked directory, verify deterministic replay evidence, validate assets, and produce a final evidence bundle.
2. **Public/store publishable release** — local publishable package plus owner-final acceptance, license/distribution decision, legal/trademark clearance, store credentials/forms/reviews, current store assets, and platform-specific release controls.

The local package may pass while public/store release remains blocked. Do not collapse these gates.

## Non-Negotiable Invariants

- Truth runs fixed 120 Hz.
- Gameplay truth is deterministic and integer/fixed-point only.
- No hidden RNG, wall-clock truth, gameplay floats, unordered truth iteration, or presentation writes into truth.
- No HP, hit points, arbitrary damage numbers, armor points, DPS, crit chance, super meter, perks, unlock stats, or speed/damage bonuses as truth.
- Replay is authoritative evidence and must fail loudly on mismatch.
- Renderer, UI, audio, VFX, camera, and fight-film systems consume truth after hashing and never mutate truth.
- Production assets must be repo-owned, source-backed, provenance-tagged, regenerable from `assets_src/`, and validated.
- Browser/HTML output, if ever introduced, is QA-only and not product presentation.
- Public-demo-ready and release-candidate-ready remain false unless explicit owner/human/external gates are actually satisfied.

## High-Fidelity Production Gate

This gate is currently **not passed**. It is separate from the local package gate.

To pass, current-run evidence must include a continuous player-facing high-fidelity native 3D renderer or approved engine integration, source-backed production assets, PBR/equivalent material response, skeletal/skinned fighters, layered armor/cloth/weapon detail, high-fidelity OATHYARD arena/training arena, lighting/atmosphere, animation and VFX driven only by hashed truth events, deterministic 1920x1080+ capture coverage, visual benchmark report, and owner visual acceptance recorded separately.

The concrete visual acceptance criteria for this gate are defined in `docs/acceptance/VISUAL_FIDELITY_ACCEPTANCE_CRITERIA.md`: atmosphere/lighting, melee animation/combat presentation, 3D model fidelity, post-processing, frame-rate, resolution, reference comparison sources, pass/fail checklists, and blocking vs. non-blocking visual issue taxonomy.

Current baseline evidence from `artifacts/final_acceptance/latest/baseline_20260629T181547Z/` shows the local technical gate passes, but pixel audits under `artifacts/baseline/20260629T164832Z/visual_inspection/` reject the visuals as prototype/placeholder-level. Current post-billhook local asset budget evidence is 22 runtime assets, 292 vertices, 492 triangles, and 8 weapon families. This satisfies the numeric weapon-family floor but remains far below production visual fidelity.

V0/V1 visual-fidelity escalation state: `./tools/visual_gap_audit.sh artifacts/visual_review/latest` classifies current visual status with JSON/Markdown gap evidence only; standalone rollup images are excluded from acceptance. `docs/decisions/0009-production-renderer-selection.md` selects Bevy/wgpu as the next renderer spike path and records that ADR 0008's raw OpenGL spike is superseded for V1 selection unless explicitly revived. `/usr/bin/blender` is currently blocked by a MaterialX ABI/symbol failure (`undefined symbol: _ZTVN17MaterialX_v1_39_46OutputE`), so the production source-asset pipeline is blocked until a working DCC/source-authoring route is installed and verified.

`artifacts/visual_review/latest/visual_benchmark_report.md` is the current visual gap report. It is a candidate evidence package, not owner acceptance.

Frontier-tech leverage is governed by `docs/research/FRONTIER_TECH_LEVERAGE.md`, `docs/decisions/0002-high-fidelity-production-target.md`, `docs/decisions/0003-truth-vs-presentation-layering.md`, and `docs/decisions/0004-renderer-and-asset-pipeline.md`. MotionBricks-style motion, Warp, Isaac Lab, Newton, MJWarp, PhysX, Chrono, Nanite/Lumen-class renderer targets, glTF/GLB, OpenUSD, Audio2Face-3D, and generative 3D tools must be classified as offline research/authoring or runtime presentation unless a separate deterministic truth ADR promotes them. Current verification commands are `./tools/research_audit.sh`, `./tools/presentation_truth_isolation.sh`, `./tools/sim_reference_compare.sh`, `./tools/ai_planner_audit.sh`, `./tools/capture_high_fidelity_screens.sh`, `./tools/visual_benchmark.sh`, and `./tools/final_acceptance.sh`; the high-fidelity screen and visual benchmark gates are expected to fail until production renderer/assets/captures exist.

## Local Publishable Package Gate

`./tools/verify.sh` is the current local gate. It must pass and include:

`./tools/publishable_gate.sh` wraps that gate and records a timestamped local package-candidate evidence bundle under `artifacts/publishable/<UTC>/`.

| Area | Required evidence | Current command/artifact |
| --- | --- | --- |
| Source build | Native Rust executable builds from source. | `./tools/build.sh`, `cargo build --locked` |
| Rust tests | Canon/determinism/content/artifact tests pass. | `./tools/test.sh`, `cargo test --locked` |
| Truth audit | Static audit scans every Rust source file under `src/` and rejects forbidden truth shortcuts. | `./tools/audit_truth.sh` |
| Readiness drift audit | Source and package docs/manifests agree that public-demo, release-candidate, owner-final, legal, trademark, and store readiness remain false. | `./tools/audit_readiness.sh . artifacts/readiness/source`, `./tools/audit_readiness.sh artifacts/package/oathyard-linux-x86_64 artifacts/readiness/package` |
| Secrets audit | Source, text artifacts/logs, and package content contain no private keys, credentials, service tokens, webhook secrets, or non-placeholder secret assignments. | `./tools/audit_secrets.sh . artifacts/secrets/source`, `./tools/audit_secrets.sh artifacts/package/oathyard-linux-x86_64 artifacts/secrets/package` |
| Environment audit | Current host build/runtime surface records required gate tools, optional graphics/DCC/audio tools, pkg-config libraries, native runtime surfaces, and false readiness flags without claiming a clean VM/container. | `./tools/audit_environment.sh artifacts/environment/verify`, `artifacts/environment/verify/environment_audit_report.md` |
| Assets | Runtime assets are generated from source and validated. | `./tools/build_assets.sh`, `./tools/validate_assets.sh`, `assets/asset_validation_report.md` |
| Runtime glTF | Local structural glTF export/validation passes, and production runtime glTF assets must have nonzero Z depth for native 3D presentation; external Khronos validation remains separate. | `assets/gltf_validation_report.md`, `assets/gltf/*.gltf` |
| PBR material evidence | Source-backed PBR/equivalent material profiles cover weapons, armor, arenas, and fighters with albedo, roughness/metallic, normal/height, edge wear, dirt, blood/wetness, cloth grain, steel scratches, leather strain, stone dust, stitching, hair/skin variation, and material IDs; replay-derived material events remain presentation-only JSON evidence and prove material maps do not affect truth hashes. | `assets_src/materials/pbr_materials.oysrc`, `assets/materials/pbr_surface_manifest.json`, `./tools/pbr_materials.sh examples/duels/basic_oathyard.duel artifacts/pbr_materials/verify`, `artifacts/pbr_materials/verify/pbr_material_manifest.json`, `artifacts/pbr_materials/verify/pbr_material_report.md` |
| Asset budgets/previews | Runtime mesh/glTF/preview metadata, source bytes, audio events, VFX events, vertices, indices, triangles, material counts, primitive counts, and the current 3D baseline are measured against local regression budgets while external validation and owner acceptance stay false. Source-backed asset preview status is recorded in local manifest/report evidence; accepted visual preview evidence still requires native 3D renderer captures with metadata. | `./tools/asset_budget_audit.sh artifacts/asset_budget/verify`, `./tools/render_asset_previews.sh artifacts/asset_previews/verify`, `artifacts/asset_budget/verify/asset_budget_report.md`, `artifacts/asset_previews/verify/asset_preview_report.md` |
| Runtime 3D audit | Every runtime glTF asset has nonzero Z depth and native combat projection consumes Z depth after truth hashes are computed. | `./tools/asset_visual_atlas.sh artifacts/asset_atlas/verify`, `./tools/audit_3d_runtime.sh artifacts/runtime_3d/verify`, `artifacts/runtime_3d/verify/runtime_3d_audit_report.md` |
| Determinism | Two local duel runs produce identical replay-relevant artifacts, and same-turn contact packets declare and obey deterministic frame ordering. | `artifacts/verify_a`, `artifacts/verify_b`, `cmp` checks and AI-sweep trace ordering checks in `tools/verify.sh` |
| Replay | Replay verifies final state hash and fails loud on mismatch. | `./tools/replay_verify.sh artifacts/verify_a/replay.json` |
| AI/scripted seats | Deterministic observe/replan AI emits legal committed actions only, then replay verifies truth outputs; AI sweep repeats multiple physical pairings and policy styles and compares committed sequences, replay JSON, trace JSON, final hashes, end-condition status/winner, capability-stop outcomes, and capability reactions. | `./tools/ai_duel.sh artifacts/ai/verify 6`, `./tools/replay_verify.sh artifacts/ai/verify/replay.json`, `./tools/ai_sweep.sh artifacts/ai_sweep/verify` |
| Truth stress | Longer 24-turn deterministic planner traces repeat bit-stably across six physical pairings, with replay equality, turn-hash-chain equality, contact ordering, action validity, capability reactions, capability-stop coverage, distinct final hashes, and adversarial capability-extrema thresholds verified. | `./tools/truth_stress.sh artifacts/truth_stress/verify`, `artifacts/truth_stress/verify/truth_stress_report.md` |
| Truth edge audit | Fixed-point/permille overflow behavior, capability clamps, invalid-action cost response, contact tie ordering, and replay schema compatibility failures are verified with generated JSON/report evidence. | `./tools/truth_edge_audit.sh artifacts/truth_edge/verify`, `artifacts/truth_edge/verify/truth_edge_audit_report.md` |
| Negative input audit | Malformed scenarios, content manifests, replay files, and replay export bundles fail loudly with specific errors rather than silent acceptance. | `./tools/negative_audit.sh artifacts/negative_audit/verify`, `artifacts/negative_audit/verify/negative_input_audit_report.md` |
| Match sweep | Multiple scenario/loadout sweeps run with machine-readable scripted-match stability, deterministic AI pairing/policy coverage, and adversarial truth-stress rollup. | `./tools/run_match_sweep.sh`, `artifacts/match_sweep/match_sweep_summary.json`, `artifacts/match_sweep/match_sweep_summary.md` |
| Input | Keyboard/mouse-zone/remap artifact exists with controller profile, glyph preview, local Steam Deck checklist, and native default gamepad-command navigation evidence; Linux joystick-interface smoke is locally verified when present, while physical controller ergonomics, Steam Deck hardware compliance, and owner input acceptance remain unclaimed. | `./tools/input_map.sh artifacts/input/verify`, `./tools/gamepad_smoke.sh artifacts/gamepad/verify` |
| Native input target | ADR and audit define the production input command boundary, validate current keyboard/mouse/default-controller command evidence, and keep physical controller, Steam Deck hardware, and owner input acceptance false. | `docs/decisions/0003-native-input-model.md`, `./tools/input_target_audit.sh artifacts/input_target/verify` |
| Accessibility | Text scale, contrast, captions, visual equivalents, remapping, reduced motion, and reduced flash settings exist as presentation-only artifacts. | `./tools/accessibility.sh artifacts/accessibility/verify`, `artifacts/accessibility/verify/accessibility_report.md` |
| Runtime settings | Accessibility/input/audio settings persist through a native save/load roundtrip with byte-identical saved and loaded JSON, explicit presentation-only truth boundary, and false public/release flags. | `./tools/runtime_settings.sh artifacts/settings/verify`, `artifacts/settings/verify/runtime_settings_report.md` |
| Native roster 3D showcase | All six default fighter/loadout families are tracked as blocked-pending-native-3D-renderer evidence from source-backed runtime glTF after content hashes; the tool writes manifest/report evidence only and emits no fallback visual files. | `./tools/native_roster_showcase.sh artifacts/native_roster/verify`, `artifacts/native_roster/verify/native_roster_showcase_manifest.json`, `artifacts/native_roster/verify/native_roster_showcase_report.md` |
| Native combat render | Native combat status consumes replay/duel result after hashes, verifies truth-read-only boundaries, records blocked-pending-native-3D-renderer status, and refuses fallback visual files until a manifest-backed native 3D renderer/camera path exists. | `./tools/native_combat_render.sh examples/duels/basic_oathyard.duel artifacts/native_combat/verify`, `artifacts/native_combat/verify/native_combat_render_manifest.json`, `artifacts/native_combat/verify/native_combat_render_report.md` |
| Visual evidence reducer | Source-run and package-smoke visual evidence status is reduced into deterministic manifest/report/hash/failed-artifact text evidence; owner visual acceptance remains false and missing native 3D captures stay blocked rather than substituted. | `./tools/visual_evidence_index.sh artifacts/visual_evidence/verify`, `artifacts/visual_evidence/verify/visual_evidence_manifest.json`, `artifacts/visual_evidence/verify/failed_visual_artifacts.txt` |
| Native presentation target | ADR and audit define the production renderer acceptance target, validate the current blocked native 3D evidence status, require manifest-backed native renderer captures for visual evidence, and explicitly keep production 3D gameplay completion, production renderer completion, and owner visual acceptance false. | `docs/decisions/0002-native-presentation-target.md`, `./tools/renderer_target_audit.sh artifacts/renderer_target/verify` |
| Audio/VFX | Trace-derived audio/VFX/captions are generated, deterministic runtime mixer settings/routing/loudness artifacts are produced, and bounded local audio-device playback smoke passes through the system backend. Final loudness approval and human audio acceptance remain unclaimed. | `./tools/audio_vfx_render.sh examples/duels/basic_oathyard.duel artifacts/audio_vfx/verify`, `./tools/audio_mixer.sh examples/duels/basic_oathyard.duel artifacts/audio_mixer/verify`, `./tools/audio_device_smoke.sh examples/duels/basic_oathyard.duel artifacts/audio_device/verify` |
| Audio runtime target | ADR and audit define the production audio runtime boundary, validate current trace-derived mixer/device/caption evidence, and keep shipping backend finalization, platform loudness acceptance, and owner audio acceptance false. | `docs/decisions/0004-audio-runtime-target.md`, `./tools/audio_target_audit.sh artifacts/audio_target/verify` |
| Desktop metadata | Linux `.desktop` entry and scalable icon validate locally; AppStream/metainfo remains blocked by pending license/distribution decision. | `./tools/desktop_metadata.sh artifacts/desktop_metadata/verify`, `artifacts/desktop_metadata/verify/desktop_metadata_report.md`, `packaging/linux/APPSTREAM_BLOCKED.md` |
| Package | Package tar, tar checksum, and package contents checksum manifest are generated from a clean package root. | `./tools/package.sh`, `artifacts/package/oathyard-linux-x86_64.tar`, `artifacts/package/oathyard-linux-x86_64.tar.sha256`, `artifacts/package/oathyard-linux-x86_64/package_checksums.sha256` |
| Package smoke | Clean unpack smoke verifies tar checksum, package contents checksums, no-argument native launch, packaged `.desktop` Exec launch, and CLI/game-flow/render/audio paths. | `./tools/smoke_package.sh artifacts/package/oathyard-linux-x86_64.tar`, `artifacts/package_smoke/package_smoke.json`, `artifacts/package_smoke/package_smoke_report.md` |
| Package reproducibility | Two consecutive local package builds produce byte-identical tarballs and package content checksum manifests. | `./tools/check_package_repro.sh artifacts/package_repro/verify`, `artifacts/package_repro/verify/package_repro_report.md` |
| Final evidence | Final acceptance report lists verified and blocked gates honestly. | `artifacts/final/final_acceptance_report.md` |

## Public / Store Release Gate

These gates remain false until separately performed and evidenced:

| Gate | Current status | Required evidence to mark true |
| --- | --- | --- |
| Owner-final acceptance | `false` | Owner reviews native visual/audio/input/package evidence and explicitly accepts. |
| Public demo readiness | `false` | Local package gate passes, owner accepts demo scope, legal/trademark/license gates are resolved, and demo store/package path is approved. |
| Release-candidate readiness | `false` | Local package gate passes from a clean release environment, owner accepts, external legal/trademark/license/store gates pass, package checksums recorded. |
| Legal clearance | `false` | Owner/legal clearance artifact. |
| Trademark clearance | `false` | Owner/legal trademark clearance artifact. |
| Store readiness | `false` | Store platform account/app access, store checklist, assets, pricing, age rating, and build review evidence. |
| Steam release | `false` | Steam store page approved, coming-soon timing satisfied where applicable, build reviewed, release manually triggered by owner. |
| itch.io release | `false` | Butler/auth/channel upload or owner-approved manual upload evidence. |

## Current Known Blockers

- Git has been initialized locally on branch `main`, generated output directories are ignored, and no obvious secret filenames were found. Source provenance is still incomplete until an owner-approved baseline commit and any desired remote/issue tracker are created.
- `LICENSE` is pending/unlicensed; distribution rights are not resolved.
- Trademark/legal/store readiness are external gates and cannot be inferred locally.
- External DCC/Khronos glTF validation is not claimed; local structural glTF validation and nonzero-Z 3D runtime audits exist.
- Linux joystick-interface smoke is local evidence only; physical controller ergonomics and Steam Deck compliance are not claimed.
- Runtime settings persistence, runtime audio mixer artifacts, bounded live audio-device playback smoke, and audio runtime target audit exist; shipping backend finalization, final loudness/platform audio certification, and human audio acceptance are not claimed.
- Linux desktop `.desktop` and icon validation exists, but AppStream/metainfo generation is blocked while `LICENSE` remains pending/unlicensed.
- Source/package readiness drift audit exists and fails on machine-readable readiness flags set true before external gates.
- Negative input audit exists and fails if malformed scenarios, content manifests, replay files, replay export bundles, or replay export bundles are silently accepted.
- Source/package secrets audit exists and fails if credentials, tokens, private keys, webhook secrets, or non-placeholder secret assignments appear.
- Local environment audit exists and records host build/runtime facts; separate clean OS user/VM/container evidence is still not claimed.
- Native presentation target ADR/audit exists and requires nonzero-Z combat asset projection evidence, UI-authored game-flow software-3D evidence, all-six-family native roster 3D showcase evidence, and depth-sorted software mesh viewport captures. `game_is_3d` must remain true for current 3D evidence, while production 3D gameplay completion, production renderer completion, and owner visual acceptance remain false.
- Native input target ADR/audit exists and validates default controller-command navigation through all current native screens, but physical controller ergonomics, Steam Deck hardware compliance, and owner input acceptance are still not claimed.
- Owner visual acceptance is not performed.

## Status Language Rules

Use these exact meanings:

- `verified locally`: command/artifact was generated in this workspace and inspected.
- `local package gate passed`: `./tools/verify.sh` passed on the current tree.
- `publishable package candidate`: local package gate passed, but public/store gates may still be false.
- `public demo ready`: only after owner/legal/store/demo-scope gates are true.
- `release candidate ready`: only after local package, clean release environment, owner, legal/trademark/license, and store gates are true.

Never use `done`, `publishable`, `ready`, `released`, or `cleared` without naming which gate passed and which gates remain false.
