# OATHYARD Publishable Completion Kanban

Setup timestamp: `2026-06-29T03:51:27Z`

This is the working board for getting OATHYARD to a publishable native-PC game without lowering canon, determinism, asset, legal, visual, accessibility, packaging, or verification standards. It is intentionally fail-closed: a card is not done because it is described here; it is done only when its evidence gate passes and the required artifact is read back.

## Active Goal

Push OATHYARD to a complete high-fidelity native-PC 3D game by driving every local gate to executable evidence, then reducing all remaining production-renderer, production-asset, owner/legal/platform, and store release blockers to exact actions.

High-fidelity production target: `docs/decisions/0007-high-fidelity-production-target.md`. The current local publishable package gate can pass while the high-fidelity product target remains false; do not collapse these gates.

M0-M21 canon/acceptance decomposition: `docs/roadmap/M0_M21_CANON_ACCEPTANCE_DECOMPOSITION.md`.

`publishable` has two explicit meanings here:

1. **Local publishable package candidate:** source build, tests, deterministic replay, asset validation, native presentation smoke, input artifacts, audio/VFX/captions, package build, package smoke, and final evidence report all pass locally.
2. **Public/store publishable release:** the local package candidate plus owner-final acceptance, license/distribution decision, legal/trademark clearance, store access/forms/reviews, current store assets, and platform-specific release controls.

Do not claim public demo readiness, release-candidate readiness, owner acceptance, legal clearance, trademark clearance, or store readiness until the corresponding external gate is actually performed.

## 0. Canon and non-degradation locks

Source precedence at setup:

1. `docs/design/GAME_CANON.md`
2. `docs/design/DEMO_SCOPE.md`
3. `ACCEPTANCE_MAP.md`
4. `AGENTS.md` / `CLAUDE.md`
5. PRDs/specs
6. Code comments

Hard locks from current sources:

- Truth is fixed 120 Hz, deterministic, integer/fixed-point only, with no hidden RNG, wall-clock truth, gameplay floats, unordered truth iteration, or render/UI/audio writes into truth (`docs/design/GAME_CANON.md:25-34`).
- Health is body/material/capability state; no HP, DPS, armor points, crits, super meter, arbitrary damage bonuses, or speed bonuses (`docs/design/GAME_CANON.md:69-83`).
- Native product target remains the goal; browser/HTML is QA-only and cannot be claimed as product presentation (`docs/design/GAME_CANON.md:100-104`, `docs/design/DEMO_SCOPE.md:26-28`).
- Public demo readiness, release-candidate readiness, owner-final acceptance, legal clearance, trademark clearance, and store readiness remain false until each external gate is actually completed (`docs/acceptance/FULL_GAME_ACCEPTANCE.md:48-57`).
- Full-game completion is not claimable while production renderer completeness beyond verified raw-X11 3D evidence, DCC/glTF validation, complete mouse/gamepad UI, physical controller/Steam Deck compliance, final loudness/platform audio certification, or owner visual/audio acceptance is missing (`artifacts/final/final_acceptance_report.md:28`).
- All production assets must be repo-owned, source-backed, provenance-tagged, and regenerable from `assets_src/`; copied/scraped/unlicensed/unverifiable/placeholder production assets are forbidden (`docs/asset_pipeline/ASSET_PIPELINE.md:3-7`).
- Current stack decision: Rust/Cargo, no external runtime dependencies for the first slice; no new dependency may enter without a decision record, license audit, determinism audit, and measured value (`docs/decisions/0001-stack-and-determinism.md:3-52`).

## 1. Board rules

### Columns

- `DONE / EVIDENCE-LOCKED`: verified by concrete artifact/command, but still subject to fresh rerun before readiness claims.
- `NOW`: highest-leverage implementation/research cards to pull next.
- `NEXT`: ready after current blockers/cards land.
- `RESEARCH`: active source-gathering or spike work; adoption requires a follow-up ADR/card.
- `BACKLOG`: required for publishable quality, not yet pullable.
- `BLOCKED / EXTERNAL`: blocked by missing credential, owner decision, legal/trademark action, platform access, missing local dependency, or human visual acceptance.

### WIP limits

- `NOW`: max 3 implementation cards + max 2 research cards.
- `RESEARCH`: each card must name the source class to fetch and the measurement that can falsify it.
- `BLOCKED`: every card must name the exact blocker class and the smallest unblock action.

### Done definition for any implementation card

A card can move to `DONE / EVIDENCE-LOCKED` only when all applicable evidence is present:

1. Canon review complete: no forbidden shortcut introduced.
2. Relevant focused test or artifact gate passes.
3. `./tools/verify.sh` passes for readiness-impacting changes, or the card explicitly states why a narrower gate is sufficient.
4. Generated replay-relevant artifacts are timestamp-free and path-free.
5. If visual/UI/audio: artifact is inspected, not just command-exit checked.
6. If dependency/tool adoption: ADR records source, license, footprint, deterministic boundaries, fallback, and removal plan.
7. If readiness/publishing: owner/legal/store gates are separately satisfied and recorded; no implicit claims.

### Fresh verification policy

Before reporting any readiness improvement, rerun fresh gates in a new timestamped log directory. Prior green logs are stale unless the task explicitly asks to parse a completed log.

## 2. Setup evidence snapshot

Observed locally during setup:

- Git is initialized locally on branch `main`; generated `target/`, `artifacts/`, and `assets/` directories are ignored. No baseline commit or remote has been created.
- `Cargo.toml` defines `oathyard` `0.1.0`, Rust 2021, one library at `src/lib.rs`, one binary at `src/bin/oathyard.rs`.
- `src/lib.rs` is still large and contains most truth, artifact writers, replay/fight-film/browser artifacts, truth stress/edge/negative-audit generation, X11 window/game-flow/combat rendering, and tests/support paths. Deterministic content tables and content hashing live in `src/content.rs`; replay export bundle writing/verification lives in `src/replay_bundle.rs`; input map/profile/glyph/checklist generation lives in `src/input_artifacts.rs`; accessibility artifact generation lives in `src/accessibility_artifacts.rs`; Linux joystick gamepad-smoke probing/reporting lives in `src/gamepad_smoke.rs`; runtime settings persistence artifacts live in `src/settings_artifacts.rs`; and trace-derived audio/VFX/mixer/audio-device-smoke generation lives in `src/audio_artifacts.rs`, as behavior-preserving source splits.
- Existing acceptance docs already require build, test, truth audit, environment audit, assets, asset budget audit, deterministic duels, replay verify, truth stress, truth edge audit, negative input audit, match sweep, capture, native combat render, native presentation target audit, native input target audit, audio/VFX, audio runtime target audit, package, no-argument package launch smoke, package smoke, and artifact validation (`docs/acceptance/FULL_GAME_ACCEPTANCE.md`, `tools/verify.sh`).
- Current content tables already contain at least six fighter traditions, six armor families, eight weapon profiles, and two arenas (`content/oathyard_content.manifest:15-42`, `src/content.rs`).
- Existing audio/VFX output is trace-derived procedural WAV/JSON/SRT artifacts, runtime audio mixer artifacts now prove deterministic routing/settings/loudness metrics in the native executable, runtime settings artifacts prove byte-exact presentation-only persistence for accessibility/input/audio preferences, audio-device smoke proves bounded local playback command success through the system backend, and audio runtime target audit records the local backend boundary. Shipping backend finalization, final loudness/platform audio certification, and human acceptance remain separate gates.
- Linux `.desktop` and scalable SVG icon metadata now validate locally and are packaged; AppStream/metainfo remains intentionally blocked while `LICENSE` is `PENDING / UNLICENSED`.
- Source/package readiness drift audit now checks docs/manifests and fails if public-demo, release-candidate, owner-final, legal, trademark, or store readiness flags drift true before external gates.
- Negative input audit now checks malformed scenarios, content manifests, replay files, replay export bundles, and replay export bundles for loud, specific failures.
- Source/package secrets audit now scans source, text artifacts/logs, and package text content for private keys, credentials, service tokens, webhook secrets, and non-placeholder secret assignments.
- Fresh final setup verification passes the full local gate (`./tools/verify.sh`) with final replay hash `f17c8f76b9dfae86`; controller profile/glyph/local Steam Deck checklist artifacts, native default controller-command navigation, runtime settings persistence, and Linux joystick-interface smoke are now included when `/dev/input/js*` is present, but publishability still remains blocked by the non-local/external and production-depth gates below.

## 3. Researched source register

Fetched during setup on `2026-06-29T03:51:27Z`. These are inputs to the board, not proof of OATHYARD compliance.

| Source | Type | Key finding to encode into cards |
| --- | --- | --- |
| Steamworks `Release Process` (`https://partner.steamgames.com/doc/store/releasing`) | Official platform docs | Steam has separate store-page and product-build/configuration checklists; store presence must be submitted before build review; approved titles require manual release; store page must be approved and coming soon for at least 2 weeks before release. |
| Steamworks `Uploading to Steam` (`https://partner.steamgames.com/doc/sdk/uploading`) | Official platform docs | SteamPipe supports efficient delivery, beta branches, rollback, update-size preview, depots/builds; package structure and patch efficiency need explicit planning. |
| Steamworks `Getting your game ready for Steam Deck and Steam Machine` (`https://partner.steamgames.com/doc/steamhardware/recommendations`) | Official platform docs | Deck/Machine verified expects all in-game functionality accessible from the default controller configuration; avoid hardware-locked graphics settings; cloud saves are recommended if saving exists. |
| Steamworks `Steam Deck and Steam Machine Compatibility Review` (`https://partner.steamgames.com/doc/steamhardware/compat`) | Official platform docs | Deck support requires physical controls/default config access to all content; recommended Deck resolutions are 1280x800 preferred or 1280x720; interface text must be readable at 12 in / 30 cm. |
| Steamworks `Graphical Assets - Overview` (`https://partner.steamgames.com/doc/store/assets`) | Official platform docs | Store capsule/library/screenshot assets have current templates and required dimensions; old dimensions are no longer accepted. |
| Steamworks `Steam Input` (`https://partner.steamgames.com/doc/features/steam_controller`) | Official platform docs | Steam Input action manifests/configs are a publishing-relevant route for gamepad glyphs and Deck controls; adoption requires source review and API boundary decision. |
| itch.io `butler manual` (`https://itch.io/docs/butler/`) | Official platform docs | Butler uploads game builds quickly/reliably, manages channels, patches, preview/diff flows; usable as a non-Steam release-channel research track. |
| Xbox Accessibility Guidelines V3.2 (`https://learn.microsoft.com/en-us/xbox/accessibility/guidelines`) | Official accessibility docs | Guidelines are best-practice guardrails, with goals, scoping questions, key target areas, implementation guidance, and example content; use as accessibility audit structure, not legal-compliance theater. |
| W3C WCAG 2.2 (`https://www.w3.org/TR/WCAG22/`) | Standard | Use perceivable/operable principles for UI text, contrast, keyboard/controller operability, and captions where applicable. |
| AppStream 1.0 (`https://www.freedesktop.org/software/appstream/docs/`) | Linux packaging metadata docs | Provides distro-agnostic software-center metadata; relevant for Linux desktop package polish if shipping outside Steam/itch. |
| Khronos glTF 2.0 specification (`https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html`) | Graphics asset standard | glTF is an API-neutral runtime asset delivery format connecting DCC tools and graphics applications; candidate for production asset interchange after validator/toolchain unblock. |
| Reproducible Builds docs (`https://reproducible-builds.org/docs/`) | Build integrity docs | Build reproducibility requires controlling variance: timestamps, locales, archive metadata, stable order, randomness, build paths, environment definition. |
| SDL3 wiki (`https://wiki.libsdl.org/SDL3/CategoryAPI`, `https://wiki.libsdl.org/SDL3/CategoryGamepad`) | Candidate runtime API docs | SDL3 offers cross-platform APIs and gamepad categories; not adopted. Needs license/footprint/determinism/reliability spike before any dependency decision. |
| OpenAL Soft (`https://openal-soft.org/`) | Candidate audio API docs | Candidate live audio runtime; not adopted. Needs license/packaging/latency/fallback spike before any dependency decision. |

Academic/live literature search status:

- arXiv queries for physics-based character control, motion matching, and contact-rich humanoid control hit HTTP 400/429/timeouts during setup.
- Semantic Scholar API queries for the same topics returned HTTP 429 during setup.
- No academic animation/control technique is adopted from memory. Cards `R-ANIM-001` and `R-PHYS-001` keep this as a pending source-verification/research track.

## 4. Current board

### DONE / EVIDENCE-LOCKED baseline

These are already represented by repo artifacts/docs, but must be freshly rerun before any readiness claim.

| ID | Card | Evidence at setup | Regression gate |
| --- | --- | --- | --- |
| D-BASE-001 | Deterministic local duel foundation exists. | `docs/design/DEMO_SCOPE.md:3-12`; `src/lib.rs`; `tools/run_duel.sh`; `tools/replay_verify.sh`. | `./tools/run_duel.sh examples/duels/basic_oathyard.duel --out artifacts/latest && ./tools/replay_verify.sh artifacts/latest/replay.json`; trace/replay/report include deterministic non-HP end-condition evidence. |
| D-BASE-002 | Canonical body/material/capability health model is documented. | `docs/design/GAME_CANON.md:36-90`; `src/lib.rs:888-1179`; `tools/audit_truth.sh` scans all Rust source files under `src/`. | Truth audit + replay hash comparison; no HP/stat terms in truth audit. |
| D-BASE-003 | Source-backed text/glTF asset pipeline exists. | `assets_src/*/*.oysrc`; `assets/gltf/*.gltf`; `docs/asset_pipeline/ASSET_PIPELINE.md`; `tools/asset_pipeline.py`; `tools/render_asset_previews.sh`; `tools/asset_visual_atlas.sh`; `tools/audit_3d_runtime.sh`. | `./tools/build_assets.sh && ./tools/validate_assets.sh`; inspect `assets/gltf_validation_report.md`; `./tools/render_asset_previews.sh artifacts/asset_previews/verify`, `./tools/asset_visual_atlas.sh artifacts/asset_atlas/verify`, and `./tools/audit_3d_runtime.sh artifacts/runtime_3d/verify` must prove every runtime glTF asset has nonzero Z depth and every source-backed preview is indexed while high-fidelity/owner claims remain false. |
| D-BASE-004 | Local package tar, checksum, contents-checksum, and smoke path exist. | `tools/package.sh`; `tools/smoke_package.sh`; package report in `artifacts/final/final_acceptance_report.md`. | `./tools/package.sh && sha256sum -c artifacts/package/oathyard-linux-x86_64.tar.sha256 && ./tools/smoke_package.sh artifacts/package/oathyard-linux-x86_64.tar`. |
| D-BASE-006 | Audio/VFX trace-derived artifact path, runtime mixer artifacts, and local audio-device smoke path exist. | `src/audio_artifacts.rs`; `./tools/audio_vfx_render.sh`; `./tools/audio_mixer.sh`; `./tools/audio_device_smoke.sh`. | `./tools/audio_vfx_render.sh examples/duels/basic_oathyard.duel artifacts/audio_vfx/latest`; `./tools/audio_mixer.sh examples/duels/basic_oathyard.duel artifacts/audio_mixer/latest`; `./tools/audio_device_smoke.sh examples/duels/basic_oathyard.duel artifacts/audio_device/latest`; inspect WAV/captions/JSON/report/loudness metrics. |
| D-BASE-007 | Input-map, controller profile, local Steam Deck checklist, and Linux joystick-interface smoke artifact paths exist for keyboard/mouse/gamepad-readiness documentation. | `src/input_artifacts.rs`; `src/gamepad_smoke.rs`; `src/lib.rs`; `src/bin/oathyard.rs`; `artifacts/input/verify`; `artifacts/gamepad/verify`. | `cargo run --locked -- input-map --out artifacts/input/verify`; `./tools/input_map.sh artifacts/input/verify`; `./tools/gamepad_smoke.sh artifacts/gamepad/verify`; inspect `input_map.json`, `input_profile.json`, `steam_deck_checklist.md`, `input_remap_report.md`, `gamepad_smoke.json`, and `gamepad_smoke_report.md`. |
| D-BASE-008 | Measured local performance and budget benchmark exists outside truth. | `tools/performance_benchmark.py`; `tools/performance_benchmark.sh`; `tools/perf_benchmark.sh`; `artifacts/perf/verify/performance_summary.json`. | `./tools/perf_benchmark.sh artifacts/perf/verify`; inspect timing samples, native playback-loop frame timing, asset bytes, package bytes, and truth-boundary notes. |
| D-BASE-009 | Contact/action/loadout matrix coverage and deterministic material/capability invariants exist. | `tools/contact_matrix.sh`; `artifacts/contact_matrix/verify/contact_matrix.json`; `artifacts/contact_matrix/verify/contact_matrix_report.md`; `src/lib.rs` contact matrix generator. | `./tools/contact_matrix.sh artifacts/contact_matrix/verify`; verify `invariants_passed`, all shipped action x weapon x armor x target contacts, representative material result classes, torque/grip/action invalidation deltas, and readable cause-chain invariant text. |
| D-BASE-010 | Local Git repository is initialized with generated outputs ignored. | `git status --short --branch` returns `## No commits yet on main`; `.gitignore` ignores `/target/`, `/artifacts/`, `/assets/`. | Baseline commit and remote remain separate owner-approved steps; do not commit/push without explicit instruction. |
| D-BASE-011 | Timestamped local publishable-gate evidence runner exists. | `tools/publishable_gate.sh`; `README.md`; `ACCEPTANCE_MAP.md`. | `./tools/publishable_gate.sh` creates `artifacts/publishable/<UTC>/summary.md`, `verify.log`, `hashes.sha256`, `verify.rc`, and blocker notes. |
| D-BASE-012 | Deterministic AI/scripted-seat duel exists. | `tools/ai_duel.sh`; `artifacts/ai/verify/ai_plan.json`; `src/lib.rs` observe/replan AI planner. | `./tools/ai_duel.sh artifacts/ai/verify 6 && ./tools/replay_verify.sh artifacts/ai/verify/replay.json`; inspect `ai_plan_report.md`. |
| D-BASE-014 | Local package reproducibility gate exists. | `tools/check_package_repro.sh`; `artifacts/package_repro/verify/package_repro_report.md`. | `./tools/check_package_repro.sh artifacts/package_repro/verify`; report must say `Status: PASSED` and `Byte comparison: identical`. |
| D-BASE-015 | Accessibility/settings artifact gate exists. | `src/accessibility_artifacts.rs`; `src/settings_artifacts.rs`; `tools/accessibility.sh`; `tools/runtime_settings.sh`; `artifacts/accessibility/verify/accessibility_settings.json`; `artifacts/settings/verify/runtime_settings.saved.json`; `artifacts/settings/verify/runtime_settings.loaded.json`; `artifacts/settings/verify/runtime_settings_report.md`. | `./tools/accessibility.sh artifacts/accessibility/verify`; `./tools/runtime_settings.sh artifacts/settings/verify`; reports must say `Status: PASSED`, truth mutation `none`, captions/visual equivalents enabled, runtime settings saved/loaded byte-exact, replay hash unaffected, and hardware gamepad smoke not claimed. |
| D-BASE-019 | Deterministic AI planner sweep covers multiple physical pairings and policy styles. | `tools/ai_sweep.sh`; `artifacts/ai_sweep/verify/ai_sweep.json`; `artifacts/ai_sweep/verify/ai_sweep_report.md`; `src/lib.rs` AI sweep generator. | `./tools/ai_sweep.sh artifacts/ai_sweep/verify`; report must say `Status: PASSED`, six pairings, two runs per pairing, stable committed sequences/replay/trace hashes, replay verification, six policy styles, eleven action labels, stable end-condition status/winner, capability-stop evidence, capability reactions, and no body-stat mutation by AI. |
| D-BASE-020 | Truth stress gate covers longer repeated deterministic replay traces. | `tools/truth_stress.sh`; `artifacts/truth_stress/verify/truth_stress.json`; `artifacts/truth_stress/verify/truth_stress_report.md`; `src/lib.rs` truth stress generator. | `./tools/truth_stress.sh artifacts/truth_stress/verify`; report must say `Status: PASSED`, 24-turn traces, six pairings, two runs per pairing, contact-order stability, turn-hash-chain stability, replay equality, action validity, capability reactions, capability-stop coverage, distinct final hashes, and adversarial capability-extrema thresholds. |
| D-BASE-021 | Truth edge audit covers fixed-point overflow policy and replay compatibility failures. | `tools/truth_edge_audit.sh`; `artifacts/truth_edge/verify/truth_edge_audit.json`; `artifacts/truth_edge/verify/truth_edge_audit_report.md`; `src/lib.rs` truth edge generator. | `./tools/truth_edge_audit.sh artifacts/truth_edge/verify`; report must say `Status: PASSED`, overflow policy `i128_intermediate_then_saturate_or_clamp`, capability lower/upper clamps, contact tie-breaker signature, current replay schema verifies, and unsupported/missing/mismatched replay fixtures fail loudly. |
| D-BASE-023 | Linux desktop entry/icon metadata validates and is packaged locally. | `packaging/linux/io.oathyard.OATHYARD.desktop`; `packaging/linux/io.oathyard.OATHYARD.svg`; `tools/desktop_metadata.sh`; `tools/package.sh`; `tools/smoke_package.sh`. | `./tools/desktop_metadata.sh artifacts/desktop_metadata/verify`; package smoke validates packaged `.desktop`/icon files and launches through the packaged `.desktop` `Exec=oathyard` path. AppStream/metainfo remains blocked by `LICENSE` pending/unlicensed status. |
| D-BASE-024 | Replay export bundle verifies trace/report/capture/hash evidence after unpack/copy. | `src/lib.rs`; `src/bin/oathyard.rs`; `tools/export_replay_bundle.sh`; `tools/verify_replay_bundle.sh`; `tools/smoke_package.sh`. | `./tools/export_replay_bundle.sh artifacts/verify_a/replay.json artifacts/export_bundle/verify && ./tools/verify_replay_bundle.sh artifacts/export_bundle/verify`; package smoke runs packaged `export-bundle` and `verify-bundle`; tampered bundle file fails hash verification. |
| D-BASE-025 | Source/package readiness drift audit guards external gate honesty. | `tools/audit_readiness.sh`; `tools/verify.sh`; `tools/smoke_package.sh`; `ACCEPTANCE_MAP.md`. | `./tools/audit_readiness.sh . artifacts/readiness/source` and package audit after `./tools/package.sh`; audit report says `Status: PASSED`, package/source modes pass, and machine-readable readiness flags remain false. |
| D-BASE-026 | Negative input audit guards parser/replay/content/bundle failure behavior. | `tools/negative_audit.sh`; `src/lib.rs` negative input generator; `src/bin/oathyard.rs`; `tests/oathyard_tests.rs`; `tools/smoke_package.sh`. | `./tools/negative_audit.sh artifacts/negative_audit/verify`; report must say `Status: PASSED`, 13 cases, all failed loudly, and malformed scenarios, content manifests, replay files, and tampered export bundles fail with specific errors. |
| D-BASE-027 | Source/package secrets audit guards local credential hygiene. | `tools/audit_secrets.sh`; `tools/verify.sh`; `tools/smoke_package.sh`; `tools/publishable_gate.sh`; `AGENTS.md`. | `./tools/audit_secrets.sh . artifacts/secrets/source` and package audit after `./tools/package.sh`; reports must say `Status: PASSED`, findings `0`, and package/source modes pass. |
| D-BASE-028 | Clean package smoke starts from an empty generated smoke root and writes package-smoke evidence. | `tools/smoke_package.sh`; `tools/verify.sh`; `artifacts/package_smoke/package_smoke.json`; `artifacts/package_smoke/package_smoke_report.md`. | `./tools/smoke_package.sh artifacts/package/oathyard-linux-x86_64.tar`; report must say `Status: PASSED`, clean smoke root `true`, no-argument launch `true`, desktop Exec launch `true`, replay verified `true`, contents checksum verified `true`, readiness audit `true`, and secrets audit `true`. |
| D-BASE-029 | Local build/runtime environment audit records host toolchain and native surface evidence. | `tools/audit_environment.sh`; `tools/verify.sh`; `tools/smoke_package.sh`; `tools/publishable_gate.sh`; `artifacts/environment/verify/environment_audit.json`. | `./tools/audit_environment.sh artifacts/environment/verify`; report must say `Status: PASSED`, required tools present, optional tool/pkg-config availability recorded, clean VM/container claimed `false`, and public/release readiness flags `false`. Package smoke also writes `artifacts/package_smoke/environment_audit/`. |
| D-BASE-031 | Native input model ADR/audit defines command boundary and controller-readiness evidence without claiming hardware acceptance. | `docs/decisions/0003-native-input-model.md`; `tools/input_target_audit.sh`; `tools/verify.sh`; `artifacts/input_target/verify/native_input_target.json`. | `./tools/input_target_audit.sh artifacts/input_target/verify`; report must say `Status: PASSED`, command boundary `presentation_command_only_until_replayable_committed_inputs`, keyboard/mouse/gamepad schema ready `true`, physical gamepad hardware `false`, Steam Deck hardware `false`, owner input acceptance `false`, replay hash affects `false`, and truth mutation `none`. |
| D-BASE-032 | Audio runtime target ADR/audit defines trace-derived mixer/device boundary without claiming shipping backend or acceptance. | `docs/decisions/0004-audio-runtime-target.md`; `tools/audio_target_audit.sh`; `tools/verify.sh`; `artifacts/audio_target/verify/audio_runtime_target.json`. | `./tools/audio_target_audit.sh artifacts/audio_target/verify`; report must say `Status: PASSED`, event source `verified_trace_replay_after_hash`, runtime mixer artifact `true`, live audio-device smoke `true`, shipping backend finalized `false`, platform loudness acceptance `false`, owner audio acceptance `false`, replay hash affects `false`, and truth mutation `none`. |
| D-BASE-033 | Asset budget audit records runtime asset size/count budgets and fails on local regressions. | `tools/asset_budget_audit.sh`; `tools/verify.sh`; `artifacts/asset_budget/verify/asset_budget.json`; `artifacts/asset_budget/verify/asset_budget_report.md`. | `./tools/asset_budget_audit.sh artifacts/asset_budget/verify`; report must say `Status: PASSED`, entries `22`, fighters `6`, weapons `8`, armor `6`, arenas `2`, audio events `6`, VFX events `6`, vertices `292`, triangles `492`, external Khronos validation `false`, owner visual acceptance `false`, and readiness flags `false`. |
| D-BASE-035 | Runtime 3D asset audit and visual atlas fail on flat or missing production visuals. | `tools/asset_visual_atlas.sh`; `tools/audit_3d_runtime.sh`; `tools/verify.sh`; `artifacts/asset_atlas/verify`; `artifacts/runtime_3d/verify`. | `./tools/asset_visual_atlas.sh artifacts/asset_atlas/verify` and `./tools/audit_3d_runtime.sh artifacts/runtime_3d/verify assets/runtime_manifest.json artifacts/native_combat/verify/native_combat_render_manifest.json`; reports must say `Status: PASSED`, assets with nonzero Z depth `22`, source/runtime/preview/provenance coverage true, production placeholder markers false, native 3D runtime geometry true, projection uses Z depth true, owner visual acceptance false, and readiness flags false. |
| D-BASE-034 | Visual evidence reducer indexes source-run and package-smoke visual artifacts. | `tools/visual_evidence_index.sh`; `tools/verify.sh`; `tools/publishable_gate.sh`; `artifacts/visual_evidence/verify/visual_evidence_manifest.json`. | `./tools/visual_evidence_index.sh artifacts/visual_evidence/verify`; report must say `Status: PASSED`, failed artifact reduction `none`, owner visual acceptance `false`, truth mutation `none`, and contact sheet/hash manifest present. `./tools/publishable_gate.sh` also writes a timestamp-local visual report/contact sheet and failed-artifact list. |

### NOW

| ID | Workstream | Card | Why now | Acceptance evidence |
| --- | --- | --- | --- | --- |
| N-PM-001 | Program control | Keep this board plus `ACCEPTANCE_MAP.md` as the single pull source for publishable completion. | Prevents ad hoc scope drift and false readiness. | `docs/roadmap/PUBLISHABLE_KANBAN.md` and `ACCEPTANCE_MAP.md` reviewed; package includes both. |
| N-BLOCK-001 | Version control | Create an owner-approved baseline commit and remote/issue tracker policy. | Git is initialized, but publishable provenance/review still needs committed baseline and optional remote/project tracker. | Owner explicitly approves commit/remote; `git status --short --branch` is clean after commit; generated artifacts remain ignored; no secrets. |
| N-ARCH-001 | Architecture | Continue splitting `src/lib.rs` into truth/content/replay/artifacts/presentation/platform modules without changing behavior. | Behavior-preserving extractions moved deterministic content tables/content hashing to `src/content.rs`, replay export bundle writing/verification to `src/replay_bundle.rs`, input artifacts to `src/input_artifacts.rs`, accessibility artifacts to `src/accessibility_artifacts.rs`, gamepad smoke to `src/gamepad_smoke.rs`, runtime settings to `src/settings_artifacts.rs`, and trace-derived audio/VFX/mixer/device-smoke generation to `src/audio_artifacts.rs`, but the remaining 15059-line `src/lib.rs` still raises change-risk for publishable-quality renderer/truth/platform work. | Each module extraction is behavior-preserving: focused artifact hashes unchanged where applicable, `cargo test --locked`, `./tools/verify.sh`; pre/post deterministic replay artifacts identical except expected non-replay logs. Latest replay-bundle extraction evidence: `artifacts/arch_bundle_before` and `artifacts/arch_bundle_after` trace/replay/report/export manifest/bundle hash/fight-film frame files are byte-identical; final hash `f17c8f76b9dfae86`, content hash `084fd68b7013d91b`. |
| N-RENDER-001 | Native presentation | Define production native renderer acceptance target before implementation: platform(s), backend candidate list, frame timing, capture method, visual bar. | Covered locally by `D-BASE-030`; current raw X11 3D evidence is still not production renderer completion. | `docs/decisions/0002-native-presentation-target.md` plus `./tools/renderer_target_audit.sh artifacts/renderer_target/verify`; next pull is implementation depth in `X-RENDER-001`. |
| N-INPUT-001 | UI/input | Define complete input model: keyboard, mouse, gamepad, Steam Deck controls, remapping, glyphs, and deterministic command boundary. | Covered locally by `D-BASE-031`; current native flow has keyboard and X11 mouse zones, default gamepad-command navigation across all current screens with glyph evidence, an input-map artifact, controller profile/glyph/local Steam Deck checklist, Linux joystick-interface smoke, and persisted hold/toggle settings, but physical controller/Steam Deck hardware ergonomics and full native remapping UI are not publishable-complete. | `docs/decisions/0003-native-input-model.md` plus `./tools/input_target_audit.sh artifacts/input_target/verify`; next pull is implementation depth in `X-UI-001`, `X-UI-002`, and `B-UI-007`. |
| N-LEGAL-001 | Legal/release | Resolve license status or keep all publishing cards blocked. | `LICENSE` is `PENDING / UNLICENSED`; distribution/publishing is blocked. | Owner-chosen license/commercial distribution policy recorded; package/readme/store cards updated; no legal clearance claim until reviewed. |

### NEXT

| ID | Workstream | Card | Pull after | Acceptance evidence |
| --- | --- | --- | --- | --- |
| X-TRUTH-001 | Truth/sim | Extend current contact matrix invariants into broader solver stress thresholds and longer replay traces. | `N-ARCH-001` or stable test harness. | Current matrix covers action x weapon x armor x target with material/capability invariants, trace artifacts declare/verify same-turn contact frame ordering, truth stress runs repeated 24-turn traces with turn-hash-chain stability and explicit adversarial solver thresholds, and truth edge audit covers fixed-point overflow policy plus replay compatibility loud failures. Next evidence broadens property-style/exhaustive truth helper sweeps. |
| X-TRUTH-002 | Truth/sim | Extend current deterministic AI planner sweep into longer automated match sweeps with adversarial end-condition and capability-state stress. | `D-BASE-019`, `D-BASE-020`, `D-BASE-021`. | Current match sweep now writes `match_sweep_summary.json` with scripted best-of-five round stability, deterministic AI pairing/policy coverage, and adversarial truth-stress rollup; current AI sweep covers six physical pairings and six policy styles with repeated sequence/hash verification; truth stress adds 24-turn repeated traces, thresholded contact/capability/capability-stop/hash-diversity/capability-extrema checks, and no body-stat mutation by AI. Next evidence adds broader property-style/exhaustive match variations or longer player-facing match flows. |
| X-RENDER-001 | Renderer | Deepen trace-derived combat frames from current labeled state sequence toward production readability: arena, fighters, weapon arcs, contact, bind/guard/stagger/collapse/injury/recovery states. | `N-RENDER-001`, `X-TRUTH-001`, `D-BASE-005`. | Current native sequence covers observe/plan, guard/bind, parry, weapon arc, hit/contact, armor/material solve, injury/capability, grip loss, stance-collapse risk, near miss/replan, recovery, final hash proof, visual audit, contact sheet, 21 replay-driven motion frames, runtime mesh/glTF/preview refs, integer-projected generated 3D glTF geometry with nonzero Z depth for active weapons/armor/arena, a UI-authored game-flow software-3D evidence preview, a 42-frame X11 playback loop, a 120-frame truth-rate native live loop with five sampled PPM captures, first-person/third-person software-rasterized mesh viewports, and a 21-frame replay-derived software 3D mesh sequence with depth-sorted filled runtime glTF triangles; remaining evidence needs production renderer completeness and a richer player-facing render loop. |
| X-AUDIO-001 | Runtime audio | Promote deterministic mixer artifacts and bounded audio-device smoke into shipping audio behavior: package-stable backend implementation, loopback/platform checks, and owner acceptance. | `R-AUDIO-001`, `D-BASE-006`, `D-BASE-032`. | Current evidence proves runtime mixer routing/settings/loudness artifacts, byte-exact persisted audio preference settings, `pw-play` playback of trace-derived WAV, and a local audio runtime target audit. Remaining evidence needs a shipping backend ADR/implementation, loopback/platform checks, and owner audio acceptance. |
| X-ASSET-001 | Asset production | Unblock external DCC/Khronos glTF validation or document text-spec plus local glTF pipeline as intentionally shipping; no placeholders. | `R-ASSET-001`. | glTF/text runtime validation report; provenance report; previews; owner visual acceptance of production assets. |
| X-PERF-001 | Performance | Expand measured benchmark into clean-target hardware runs with memory/startup distributions once renderer/input/audio milestones stabilize. | Renderer/input/audio milestones. | `artifacts/perf/<timestamp>/performance_summary.md` with frame timing distributions, memory, startup time, package size, and hardware profile. |
| X-PACK-001 | Packaging | Extend the now-checksummed reproducible package with install/run scripts and store/AppStream metadata once final renderer/input/audio/license decisions settle. | `X-UI-001`, `X-RENDER-001`, `X-AUDIO-001`, `D-BASE-013`, `D-BASE-023`. | Clean unpack smoke plus local desktop metadata validation; package checksums and two-build reproducibility remain green; no path/timestamp drift in replay-relevant artifacts. Store/AppStream metadata remains blocked until license/distribution gates are resolved. |
| X-QA-001 | QA | Extend the timestamped publishable-gate runner with visual contact sheets and failed-artifact reduction. | Core ship systems. | Current evidence: `D-BASE-034` creates an automated visual evidence manifest/report/contact sheet, source/package artifact hashes, and `failed_visual_artifacts.txt` from one command. Remaining evidence for public/store release is human/owner visual verdict and any external visual-quality review; those are not claimed locally. |

### RESEARCH

Research cards remain non-adopted until their source notes and local spike results exist.

| ID | Topic | Source targets | Falsifying measurement | Output |
| --- | --- | --- | --- | --- |
| R-RENDER-001 | Native renderer/input backend choice. | Raw X11/Wayland/EGL/OpenGL docs; SDL3 docs; possible Rust direct bindings; Steam Deck docs. | If candidate cannot provide deterministic capture, controller support, packaged runtime stability, and low dependency/legal footprint, reject it. | ADR with measured startup, frame capture, event model, build impact, license surface, removal plan. |
| R-INPUT-001 | Steam Deck / gamepad / glyph path. | Steam Deck compatibility and Steam Input docs. | If all content cannot be accessed from default controller config without settings changes, not acceptable. | Input spec + automated checklist. |
| R-AUDIO-001 | Live audio runtime. | OpenAL Soft, SDL audio, native ALSA/Pulse/PipeWire docs. | If candidate requires fragile system packages or cannot run clean smoke in package, reject/keep captions-only until solved. | Audio ADR + playback smoke artifact. |
| R-ASSET-001 | Production asset interchange. | Khronos glTF 2.0, glTF validator, Blender issue/logs, package availability. | If DCC/glTF path cannot be validated reproducibly, keep deterministic text source pipeline and do not claim DCC/glTF readiness. | Asset ADR + validator report. |
| R-ACCESS-001 | Accessibility baseline. | Xbox Accessibility Guidelines V3.2, WCAG 2.2, Game Accessibility Guidelines as secondary. | If an accessibility claim lacks captured UI/audio evidence, reject claim. | Accessibility checklist with exact game settings/screens. |
| R-STORE-001 | Steam release process and assets. | Steamworks release, SteamPipe, graphical assets, pricing/localization docs. | If store checklist item depends on credentials/legal assets not available, mark external-blocked instead of simulating. | Store-readiness checklist with owner-owned artifacts. |
| R-ITCH-001 | itch.io release channel. | Butler docs: push, channels, preview/diff. | If channel upload requires credentials not available, block before publishing; package locally only. | itch release runbook; dry-run/preview evidence if possible. |
| R-REPRO-001 | Reproducible package discipline. | Reproducible Builds docs, Cargo lock/build env, tar metadata. | If two clean builds differ without understood reason, release package fails. | Reproducibility report with diffoscope or equivalent byte diff. |
| R-ANIM-001 | Combat animation/motion readability. | arXiv/ACM/SIGGRAPH/industry talks for motion matching, physics-based character control, hit reaction readability. | If technique cannot preserve truth-after-hash boundary or is too data-heavy/unlicensed, reject. | Annotated research memo + minimal local spike using repo-owned data only. |
| R-PHYS-001 | Contact-rich physical melee modeling. | HEMA/biomechanics references, rigid-body/contact solver literature, game postmortems. | If model reintroduces HP/arbitrary damage or non-deterministic floats into truth, reject. | Mechanistic model memo + integer/fixed-point acceptance tests. |
| R-CAMERA-001 | Fight-film/camera language. | Sports broadcast/film grammar, fighting game replay readability, accessibility for motion/camera shake. | If camera hides contact/capability evidence, reject. | Camera manifest spec + visual audit rubric. |
| R-PRICING-001 | Commercial release packaging. | Steam pricing, regional pricing, itch pricing docs, owner business decision. | If owner has not selected price/region/demo policy, keep store readiness false. | Release business checklist, not committed in source if sensitive. |

### BACKLOG by workstream

#### Program / governance

| ID | Card | Done evidence |
| --- | --- | --- |
| B-PM-001 | Maintain `docs/roadmap/PUBLISHABLE_KANBAN.md` as board state; update card moves only with evidence. | Board changed with command/artifact references. |
| B-PM-002 | Add source-of-truth drift audit that checks canon, acceptance, roadmap, README, package docs agree on readiness flags. | Covered by `D-BASE-025`; remaining work is extending the audit to future issue tracker/release checklist once those exist. |
| B-PM-003 | Create issue labels/milestones after version control exists: `canon`, `truth`, `renderer`, `ui`, `audio`, `asset`, `qa`, `release`, `blocked-external`. | Issue tracker exists with no secrets and mirrors this board. |
| B-PM-004 | Add decision log for every dependency/platform/store choice. | `docs/decisions/NNNN-*.md` contains source/pro/con/measurement/licensing/determinism. |

#### Truth / deterministic gameplay

| ID | Card | Done evidence |
| --- | --- | --- |
| B-TRUTH-001 | Expand action semantics beyond current labels only where canon supports it; no raw puppeteering. | Tests show action labels map to physical causes/costs. |
| B-TRUTH-002 | Enforce stable ordering and overflow policy across all truth iteration/math. | Current edge audit proves fixed-point/permille overflow policy, capability clamps, and contact tie ordering; remaining work is broader property-style randomized-free exhaustive sweeps across every truth helper. |
| B-TRUTH-003 | Build loadout/injury variation sweeps across six traditions/families. | Match sweep report proves varied costs/injuries/capabilities. |
| B-TRUTH-004 | Add replay schema version migration/compatibility tests. | Current edge audit proves current schema verification plus unsupported/missing/mismatched replay loud failures; remaining work is explicit old-schema migration once a second supported replay schema exists. |
| B-TRUTH-005 | Define save-game model if any progression/settings are persisted; separate from replay truth. | Save fixtures; no wall-clock truth; Steam Cloud candidate mapped. |

#### Native renderer / presentation

| ID | Card | Done evidence |
| --- | --- | --- |
| B-RENDER-001 | Move from one-frame X11 line art to continuous native render loop with deterministic replay-driven frames. | Current native combat render writes 21 sampled replay-derived motion frames, a 42-frame X11 playback-loop final capture, a 120-frame truth-rate native live loop with five sampled PPM captures and loop hash, two software-rasterized 3D mesh viewport captures, a 21-frame replay-derived software 3D mesh sequence, and external benchmark timing per playback/live-loop/viewport/sequence frame path; remaining work is a richer player-facing runtime loop with persistent controls and owner visual acceptance. |
| B-RENDER-002 | Implement production arena, fighters, weapons, armor silhouettes from source assets. | Current native combat captures use source-backed weapon reach/mass and armor coverage from canonical scenario loadouts, verifies runtime mesh/glTF/preview refs for active weapons/armor/arena, projects generated 3D glTF triangle geometry with nonzero Z depth into native frames, and writes filled depth-sorted software mesh viewport and replay-sequence captures; remaining evidence requires production renderer completeness and source preview parity. |
| B-RENDER-003 | Render readable contact states: hit, bind, guard, stagger, collapse, injury, recovery. | Current native combat sequence labels hit/contact, guard/bind, parry, weapon arc, armor/material solve, injury/capability, grip loss, recovery, near miss, and stagger/collapse-risk motion frames, with source-backed silhouettes, visual audit, and contact sheet; remaining evidence requires production-quality visual treatment. |

#### UI / UX / input / accessibility

| ID | Card | Done evidence |
| --- | --- | --- |
| B-UI-002 | Fighter/loadout/arena selection modifies deterministic inputs/content hashes visibly. | Native flow selects `chain_reed`, records the edit log, writes a UI scenario canonical hash, and generates replay artifacts for the selected loadout; remaining work is richer in-window selection controls and owner UI acceptance. |
| B-UI-003 | Planning timeline authoring UI for compact action labels and directional influence. | Native flow edits the timeline variant to `bind_pressure`, serializes committed compact action labels and directional influence, and replay-verifies the authored duel; remaining work is persistent editable timeline controls with manual/video proof. |
| B-UI-004 | Consequence screen shows base/current frame cost and every physical delta reason. | Capture and trace/report agree exactly. |
| B-UI-005 | Promote replay verification into full runtime replay UI. | UI can verify replay files, display hashes, and fail corrupt replay loudly; CLI and native artifact gates remain green. |
| B-UI-006 | Extend settings/accessibility beyond artifact gate: hold/toggle alternatives, runtime persistence, audio level/mute, and native settings UI. | Runtime settings persistence now proves byte-exact save/load for text scale, hold/toggle alternatives, and audio gains outside truth; remaining work is native settings UI editing, settings-extreme captures, manual/input smoke, and owner accessibility acceptance. |
| B-UI-007 | Mouse and gamepad support, including all major navigation/actions. | Current local evidence can read a Linux joystick-class device, generates controller profile/glyph/local Steam Deck checklist artifacts, and proves default controller-command navigation plus glyph coverage across current native screens; remaining done evidence needs physical controller/Steam Deck runtime smoke, active-device glyph switching, and owner input acceptance. |

#### Assets / content / art direction

| ID | Card | Done evidence |
| --- | --- | --- |
| B-ASSET-001 | Fix or bypass Blender MaterialX failure with evidence. | `blender --version`/startup log fixed, or ADR documents text-spec pipeline as shipping path. |
| B-ASSET-002 | Install/validate `gltf-validator`, `gltfpack`, `toktx` or reject each with blocker evidence. | Tool version logs + validation reports, or blocker cards. |
| B-ASSET-003 | Replace every debug/placeholder-looking production visual with owner-approved repo-owned source asset. | Provenance report + owner visual acceptance notes. |
| B-ASSET-004 | Add asset budgets: polygon/vertex/material/texture/audio/VFX counts and memory estimates. | Covered locally by `D-BASE-033`; remaining production work is raising asset fidelity and validating larger budgets when real production assets replace generated local meshes. |
| B-ASSET-005 | Generate store-quality screenshots/key art from actual native captures, not fabricated mockups. | Store asset folder contains source/capture/provenance and dimensions match current Steam templates. |

#### Audio / VFX

| ID | Card | Done evidence |
| --- | --- | --- |
| B-AUDIO-001 | Decide integrated live playback backend and package path. | Current package smoke exercises deterministic `audio-mixer` artifacts, bounded `audio-device-smoke`, and `audio_target_audit`; remaining done evidence needs ADR/implementation for shipping backend integration plus owner/loopback acceptance. |
| B-AUDIO-002 | Keep all critical audio accompanied by captions/visual equivalents. | Captions SRT + UI captions setting + visual audit. |
| B-AUDIO-003 | Expand trace-derived sound/VFX events for bind, deflect, armor shift, injury, collapse, recovery. | Event manifest and captured playback/captions from replay. |
| B-AUDIO-004 | Volume/mute/settings persistence. | Runtime settings smoke now persists audio gains/mute fields with no truth mutation; remaining work is native settings UI editing, loopback/platform checks, and owner audio acceptance. |

#### Replay / fight film / evidence

| ID | Card | Done evidence |
| --- | --- | --- |
| B-REPLAY-001 | Replay verify UX. | UI can verify replay files, display hashes, and fail corrupt replay. |
| B-REPLAY-002 | Fight-film camera manifests, PPM frames, native player capture, and native shot stepping generated only from trace. | `D-BASE-016`, `D-BASE-017`; next step is richer player-state presentation. |
| B-REPLAY-003 | Build visual diff/contact-sheet tooling for captures. | Before/after contact sheet plus frame annotations. |
| B-REPLAY-004 | Add export bundle: replay + trace + report + captures + hash manifest. | Covered by `D-BASE-024`; remaining work is richer player-facing export UX and optional archive packaging after renderer/UI depth improves. |

#### Packaging / build / release engineering

| ID | Card | Done evidence |
| --- | --- | --- |
| B-PACK-001 | Create clean build environment definition. | Locally covered by `D-BASE-029`: host audit records rustc/cargo/pkg-config/system libraries and runtime surfaces. Remaining publishable-hardening work is a separate clean OS user/VM/container run when that environment is available. |
| B-PACK-002 | Maintain reproducible package tar/hash gate. | `./tools/check_package_repro.sh artifacts/package_repro/verify` remains green after packaging changes. |
| B-PACK-003 | Desktop integration metadata if shipping Linux desktop outside Steam. | `.desktop` and icon validation are covered by `D-BASE-023`; remaining AppStream/metainfo validation is blocked until owner license/distribution terms exist. No installer claim unless built. |
| B-PACK-004 | SteamPipe depot structure plan. | Steam build scripts/manifests staged outside secrets; dry-run/local structure check. |
| B-PACK-005 | itch butler channel plan. | Butler dry-run/preview where credentials exist; otherwise external-blocked. |
| B-PACK-006 | Crash/error reporting without telemetry/network service by default. | Local logs/artifact path; no network/telemetry introduced. |

#### QA / performance / owner gates

| ID | Card | Done evidence |
| --- | --- | --- |
| B-QA-001 | Full verifier logs under timestamped `artifacts/final/<timestamp>/`. | Build/test/verify logs stored; summary gives hashes and paths. |
| B-QA-002 | Native visual acceptance pack. | Contact sheets/video/screens with owner checklist; owner marks accepted or rejected. |
| B-QA-003 | Performance gates. | Startup time, frame timing, input latency proxy, memory, package size; thresholds recorded. |
| B-QA-004 | Fuzz/negative tests for parsers/replay/content manifests. | Covered by `D-BASE-026`; remaining work is broader generated-case/property-style negative sweeps once the parser surface expands. |
| B-QA-005 | Clean-machine smoke. | Covered locally by `D-BASE-028`; remaining work is a separate clean OS user/VM/hardware run once that environment is available. |
| B-QA-006 | Security/secrets audit. | Covered locally by `D-BASE-027`; remaining store-token handling requires actual store/tool integration and must load secrets only from env/secret store, never source or package content. |

#### Store / legal / publishing

| ID | Card | Done evidence |
| --- | --- | --- |
| B-STORE-001 | Resolve license/distribution rights. | License or proprietary distribution terms chosen by owner; package/docs updated. |
| B-STORE-002 | Trademark/name clearance. | External legal/trademark review artifact; readiness flag remains false until done. |
| B-STORE-003 | Steam app setup and checklist. | Store page checklist and build checklist completed in Steamworks; no credentials committed. |
| B-STORE-004 | Steam coming-soon/store review. | Store presence submitted/approved; coming soon date recorded; 2-week gate tracked. |
| B-STORE-005 | Steam build review. | Build uploaded through SteamPipe and reviewed; app release remains manual. |
| B-STORE-006 | Store assets and trailer/demo captures. | Assets generated from real gameplay/captures, dimensions verified against current templates. |
| B-STORE-007 | Pricing/localization/age-rating decisions. | Owner decisions and platform forms completed; not inferred by agent. |
| B-STORE-008 | Public demo readiness gate. | Owner explicitly marks public demo ready after full local gates + store/legal gates; code flag updated only then. |
| B-STORE-009 | Release-candidate readiness gate. | Full RC checklist passes; owner signs off; legal/trademark/store readiness true. |

### BLOCKED / EXTERNAL

| ID | Blocker | Class | Evidence | Smallest unblock action |
| --- | --- | --- | --- | --- |
| E-GIT-001 | No owner-approved baseline commit or remote/issue tracker exists yet. | VCS/provenance. | Git is initialized locally on `main`, but all source files are untracked and no remote exists. | Owner approves first commit and remote/project tracker policy; then create baseline commit without generated artifacts or secrets. |
| E-LICENSE-001 | License is pending/unlicensed. | Legal/distribution. | `LICENSE:1-6` says no license granted; README says license-pending/unlicensed. | Owner chooses license/commercial distribution policy; legal review as needed. |
| E-TRADEMARK-001 | Trademark/name clearance not performed. | External legal. | Acceptance external gates remain false. | Owner/legal performs clearance; record result. |
| E-STORE-001 | Store credentials/app access absent. | Missing credential/external service. | Publishing is out of local scope without owner gate. | Owner provides Steamworks/itch access or asks for local-only release packaging. |
| E-OWNER-001 | Owner visual acceptance not performed. | Human-required action. | Final report explicitly does not claim owner-final acceptance. | Produce visual pack; owner accepts/rejects. |
| E-DCC-001 | Blender startup broken; glTF tooling absent. | Missing dependency/incompatible version. | Acceptance docs list Blender MaterialX failure and missing glTF tools. | Fix/install tools or approve deterministic text-spec asset pipeline as shipping path. |
| E-PLATFORM-001 | Current native presentation is Linux raw X11 only. | Incompatible platform breadth. | Non-Linux native paths return `BLOCKED`; current package is `oathyard-linux-x86_64`. | Decide target breadth: Linux-only first, Steam Deck, Windows, or cross-platform backend research. |
| E-AUDIO-001 | Shipping audio runtime is not production-complete. | Missing subsystem depth and human/platform acceptance. | Current path writes procedural WAV/captions/manifests, deterministic mixer settings/routing/loudness artifacts, byte-exact persisted runtime audio settings, bounded local playback smoke via system backend, and an audio runtime target audit keeping shipping/backend/owner claims false. | Approve shipping backend ADR/implementation, loopback/platform checks, and owner audio acceptance. |

## 5. Pull order to completion

Do not pull store/release cards before the local game can stand as a product. Recommended strict order:

1. Program integrity: `N-BLOCK-001`, `N-ARCH-001`, board discipline, timestamped logs.
2. Target decisions: renderer/input/audio/asset ADRs from `RESEARCH`.
3. Truth depth: contact/action/loadout/AI sweeps.
4. Native product loop: menu, selection, planning, resolve, consequence, replay.
5. Production presentation: renderer, camera, UI readability, input/gamepad, accessibility.
6. Production assets/audio/VFX: owner-approved, source-backed, runtime validated, captions.
7. Packaging/repro/perf/clean smoke.
8. Owner visual acceptance.
9. Legal/trademark/license.
10. Store-specific assets/forms/reviews.
11. Public demo / release-candidate flags only after all above evidence exists.

## 6. Standard evidence commands

Run focused gates during card work, then full gates before readiness claims.

```sh
./tools/build.sh
./tools/test.sh
cargo build --locked
cargo test --locked
./tools/build_assets.sh
./tools/validate_assets.sh
./tools/render_asset_previews.sh artifacts/asset_previews/latest
./tools/run_duel.sh examples/duels/basic_oathyard.duel --out artifacts/latest
./tools/replay_verify.sh artifacts/latest/replay.json
./tools/audit_truth.sh
./tools/audit_secrets.sh . artifacts/secrets/latest
./tools/negative_audit.sh artifacts/negative_audit/latest
./tools/run_match_sweep.sh
./tools/perf_benchmark.sh artifacts/perf/latest
./tools/gamepad_smoke.sh artifacts/gamepad/latest
./tools/accessibility.sh artifacts/accessibility/latest
./tools/native_combat_render.sh examples/duels/basic_oathyard.duel artifacts/native_combat/latest
./tools/audio_vfx_render.sh examples/duels/basic_oathyard.duel artifacts/audio_vfx/latest
./tools/audio_device_smoke.sh examples/duels/basic_oathyard.duel artifacts/audio_device/latest
./tools/audio_target_audit.sh artifacts/audio_target/latest
./tools/package.sh
./tools/smoke_package.sh artifacts/package/oathyard-linux-x86_64.tar
./tools/verify.sh
```

Readiness-impacting final runs should write to a fresh timestamped directory and include:

- exact command log;
- final replay hash;
- content hash;
- package SHA-256;
- deterministic A/B artifact comparison result;
- visual contact sheet path;
- skipped checks, if any, with blocker classification;
- explicit readiness flags.

## 7. Card template for future additions

```md
| ID | Column | Workstream | Card | Entry condition | Done evidence | Research/source refs | Blocker class |
| --- | --- | --- | --- | --- | --- | --- | --- |
| B-XXX-000 | BACKLOG | <truth/render/ui/audio/assets/package/store> | <task> | <what must be true before pull> | <command/artifact/owner gate> | <primary sources> | <only if blocked> |
```

Any future card that proposes a new dependency, external service, asset source, store claim, or readiness flag must include a primary-source reference and a local falsification/measurement path before implementation.
