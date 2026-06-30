# OATHYARD

OATHYARD is a deterministic native-PC planned-time physical melee duel foundation.

This repository is a deterministic OATHYARD game foundation. It now contains verified local-duel systems and full-game-facing tools/assets, but public-demo-ready, release-candidate-ready, owner-final-accepted, legal clearance, and trademark clearance remain false unless their external gates are actually performed. The project is license-pending/unlicensed; see `LICENSE`.

## Design Knowledge Base

`docs/DESIGN_KB.md` consolidates the game identity, canon, mechanics, systems, content, art direction, architecture, roadmap, acceptance gates, and source map into one navigable reference.

## Requirements

- Rust/Cargo 1.96 or newer with the lockfile honored via `--locked`.
- `/bin/bash`, Python 3, and standard Unix tooling used by the repository scripts.
- Linux native display/runtime support for the local X11/XWayland smoke paths when running native presentation gates.
- No external store, legal, trademark, owner-acceptance, or public-demo gate is implied by local command success.

## Quick Start

```sh
cargo build --locked
./tools/build_assets.sh
cargo run --locked -- match --scenario examples/duels/basic_oathyard.duel --out artifacts/latest --best-of 5
./tools/replay_verify.sh artifacts/latest/replay.json
./tools/verify.sh
```

The generated `artifacts/`, `assets/`, and `target/` roots are local build/evidence caches. They are ignored and can be deleted; `./tools/build_assets.sh`, Cargo, and the verification/package scripts regenerate them from source inputs.

## Install / Package Smoke

```sh
./tools/package.sh
./tools/smoke_package.sh artifacts/package/oathyard-linux-x86_64.tar
```

The package is a local technical smoke artifact while `LICENSE` remains pending/unlicensed and public/store gates remain false. Do not treat the package tarball as a redistributable release unless the external gates in `ACCEPTANCE_MAP.md` have been performed.

## Repository Layout

- `src/` — Rust library and native executable source.
- `tests/` — Cargo integration tests.
- `tools/` — repo-native build, audit, verification, package, and smoke scripts.
- `docs/` — canon, scope, acceptance, roadmap, and decision records.
- `examples/duels/` — runnable duel scenarios.
- `assets_src/` — source asset definitions; generated runtime files are emitted under ignored `assets/`.
- `content/` — content manifest consumed by asset/runtime/package flows.
- `packaging/linux/` — Linux desktop/icon/AppStream-blocker metadata used by package gates.
- `spikes/` — retained historical experiments only; not production substrate and not package entrypoints.
- Ignored/generated roots: `artifacts/`, `assets/`, `target/`, and `tools/__pycache__/`.

## Baseline

Bootstrap history: the repository started near-empty with only `.gitignore`, `Cargo.toml`, and `LICENSE`; at that inspection time it had not yet been initialized as a Git repository. Current repository state should be checked with `git status --short --ignored`.

Available local stack at bootstrap:

- Rust: `rustc 1.96.0`, `cargo 1.96.0`
- Shell: `/bin/bash`
- C/C++ build tools: `gcc`, `clang`, `cmake`, `make`, `ninja`, `pkg-config`
- Native graphics helpers checked: `sdl2`, `glfw3`, and `vulkan` pkg-config metadata were unavailable; `x11`, `wayland-client`, `egl`, and `gl` metadata were present; Vulkan runtime/tooling command `vulkaninfo` was present

Before this bootstrap, no game system existed, so build, replay, artifact validity, truth audit, and deterministic-duel behavior could not be verified.

## Build

```sh
cargo build --locked
./tools/build.sh
```

## Test

```sh
cargo test --locked
./tools/test.sh
```

## Assets

```sh
./tools/build_assets.sh
./tools/validate_assets.sh
./tools/asset_budget_audit.sh artifacts/asset_budget/latest
./tools/render_asset_previews.sh artifacts/asset_previews/latest
```

The asset builder writes runtime mesh summaries, deterministic extruded 3D `.gltf` files under `assets/gltf/`, SVG previews, provenance reports, and validation reports. Validation fails if production runtime glTF assets have no Z depth.
The asset budget audit writes `asset_budget.json` and `asset_budget_report.md`, measuring runtime glTF bytes, mesh bytes, preview bytes, vertices, indices, triangles, material counts, source bytes, audio event count, and VFX event count. The current post-billhook local 3D baseline is 22 runtime assets, 292 vertices, and 492 triangles. It is a local regression gate and does not claim external Khronos validation or owner visual acceptance.
The asset preview renderer writes `asset_preview_manifest.json`, `asset_preview_report.md`, copied source-backed preview SVGs, and an asset-preview contact sheet for six fighters, eight weapons, six armor families, and two arenas. Current previews are local structural SVG evidence, not high-fidelity product presentation or owner visual acceptance.

```sh
./tools/asset_visual_atlas.sh artifacts/asset_atlas/latest
./tools/audit_3d_runtime.sh artifacts/runtime_3d/latest assets/runtime_manifest.json artifacts/native_combat/verify/native_combat_render_manifest.json
```

The asset atlas indexes every runtime source/mesh/glTF/preview and fails on missing provenance, placeholder markers, or flat glTF geometry. The runtime 3D audit proves every runtime asset has nonzero Z depth and the native combat renderer uses `integer_oblique_depth_projection` after truth hashes are computed.

## Run A Scripted Duel

```sh
./tools/run_duel.sh examples/duels/basic_oathyard.duel --out artifacts/latest
```

The run writes:

- `artifacts/latest/trace.json`
- `artifacts/latest/replay.json`
- `artifacts/latest/final_state_hash.txt`
- `artifacts/latest/duel_report.md`
- `artifacts/latest/fight_film_manifest.json`

`trace.json`, `replay.json`, and `duel_report.md` include deterministic non-HP end-condition evidence. The basic scripted duel remains `unresolved_after_script`; longer AI/capability stress runs can resolve to a physical capability stop such as torso rotation lock or stance collapse.

`trace.json` also declares the simultaneous contact order rule: `frame_then_attacker_then_defender_then_action_then_target_then_direction`. Verification gates fail if same-turn contact packets are emitted out of frame order.

## Replay Verify

```sh
./tools/replay_verify.sh artifacts/latest/replay.json
```

## Fight-Film Camera Artifacts

```sh
```

This verifies the replay first, then writes `fight_film_camera_manifest.json`, `fight_film_report.md`, and `fight_film_contact_sheet.svg`. Camera shots are trace-derived presentation artifacts and do not mutate gameplay truth.

## Replay Export Bundle

```sh
./tools/export_replay_bundle.sh artifacts/latest/replay.json artifacts/export_bundle/latest
./tools/verify_replay_bundle.sh artifacts/export_bundle/latest
```

The export command verifies the replay, regenerates canonical trace/replay/report/timeline artifacts, regenerates replay-verified fight-film camera/frame artifacts, and writes `export_bundle_manifest.json`, `export_bundle_report.md`, and `bundle_hashes.txt`. The verify command replays the bundle and fails if any bundled file hash is changed.

## Scripted Local Match

```sh
cargo run --locked -- match --scenario examples/duels/basic_oathyard.duel --out artifacts/match/latest --best-of 5
```


```sh
```



```sh
cargo run --locked --
```

For automated smoke without manual input:

```sh
```

```sh
```


Manual native flow:

```sh
```

## Native Roster 3D Showcase

```sh
./tools/native_roster_showcase.sh artifacts/native_roster/latest
```

This renders deterministic native-software 3D PPM frames for all six default fighter/loadout families from runtime glTF after content hashes. It writes `native_roster_showcase_manifest.json`, `native_roster_showcase_report.md`, `native_roster_showcase_contact_sheet.svg`, and six PPM frames covering `saltreach_duelist`, `oathyard_writ`, `chainbreaker`, `reed_sentinel`, `gate_shield`, and `bruiser_oath`. The artifacts are source-backed 3D presentation evidence and remain truth-read-only; owner visual acceptance and production renderer completion are not claimed.

## Input Map

```sh
./tools/input_map.sh artifacts/input/latest
```

This writes `input_map.json`, `input_profile.json`, `steam_deck_checklist.md`, and `input_remap_report.md` for keyboard bindings, native mouse zones, gamepad-ready command bindings, and a local Steam Deck input checklist. These artifacts are schema/local evidence only; they do not claim physical controller ergonomics, Steam Deck hardware compliance, or owner input acceptance.

## Gamepad Smoke

```sh
./tools/gamepad_smoke.sh artifacts/gamepad/latest
```

This probes the Linux `/dev/input/js*` joystick interface from the native executable and writes `gamepad_smoke.json` plus `gamepad_smoke_report.md`. It proves only that a joystick-class input device is visible and readable on this machine; physical controller ergonomics, Steam Deck compliance, glyph correctness, and owner acceptance remain unclaimed.

## Native Input Target Audit

```sh
./tools/input_target_audit.sh artifacts/input_target/latest
```


## Accessibility Settings

```sh
./tools/accessibility.sh artifacts/accessibility/latest
```

This writes `accessibility_settings.json` and `accessibility_report.md` for text scale, high contrast, captions, visual equivalents, remapping, reduced motion, and reduced flash. These settings are presentation/input only and do not mutate gameplay truth.

## Runtime Settings Persistence

```sh
./tools/runtime_settings.sh artifacts/settings/latest
```

This writes `runtime_settings.default.json`, `runtime_settings.saved.json`, `runtime_settings.loaded.json`, and `runtime_settings_report.md`. The smoke saves a deterministic accessibility/input/audio profile, reads it back through the native executable, proves the saved and loaded JSON are byte-identical, and records that the settings are presentation-only, do not mutate truth, and do not affect replay hashes.

## Readiness Drift Audit

```sh
./tools/audit_readiness.sh . artifacts/readiness/source
./tools/audit_readiness.sh artifacts/package/oathyard-linux-x86_64 artifacts/readiness/package
```

This checks source/package docs and manifests for false readiness claims. It fails if public-demo, release-candidate, owner-final, legal, trademark, or store readiness flags drift true before their external gates are completed.

## Secrets Audit

```sh
./tools/audit_secrets.sh . artifacts/secrets/source
./tools/audit_secrets.sh artifacts/package/oathyard-linux-x86_64 artifacts/secrets/package
```

This scans source, text artifacts/logs, and packaged text content for common credentials, private keys, service tokens, and secret assignments. It writes `secrets_audit.json` and `secrets_audit_report.md`, and fails if any finding is present.

## Environment Audit

```sh
./tools/audit_environment.sh artifacts/environment/latest
```

This records the local build/runtime environment: required verification tools, optional compiler/graphics/DCC/audio tools, pkg-config libraries, native runtime surfaces, and false readiness flags. It is host evidence only and does not claim a clean VM/container, public demo readiness, release-candidate readiness, legal clearance, trademark clearance, store readiness, or owner acceptance.

## Native Combat Render

```sh
./tools/native_combat_render.sh examples/duels/basic_oathyard.duel artifacts/native_combat/latest
```

This runs the scenario through authoritative truth, then captures a native X11/XWayland overview plus a deterministic trace-derived 12-frame combat state sequence: observe/plan, guard/bind, parry window, weapon arc, hit/contact, armor/material solve, injury/capability, grip loss, stance-collapse risk, near miss/replan, recovery, and final hash proof. It reconstructs source-backed silhouettes from the canonical scenario after hashing, verifies the reconstructed initial state hash, verifies runtime mesh/glTF/preview references for the active weapons, armor, and OATHYARD arena, and projects extruded 3D glTF triangle geometry into the native frames using integer presentation coordinates and Z-depth oblique projection. It also writes a 21-frame replay-derived motion sequence sampled from committed turn hashes, renders a 42-frame X11 playback loop before capturing `native_combat_playback_final.ppm`, renders a 120-frame truth-rate native live loop with five inspectable PPM sample captures plus a deterministic loop hash, writes two software-rasterized 3D mesh viewport PPMs for third-person and first-person cameras, and writes a 21-frame software-rasterized 3D replay mesh sequence (`native_combat_3d_motion_001.ppm` through `native_combat_3d_motion_021.ppm`) using depth-sorted filled runtime glTF triangles after truth hashes. It captures 1280x720 and 1280x800 overview frames for resolution support evidence. It writes `native_combat_render_manifest.json`, `native_combat_visual_audit.md`, and `native_combat_contact_sheet.svg` with PPM frame hashes, runtime asset geometry evidence, visual-audit evidence, 3D Z-depth projection evidence, software mesh viewport/sequence evidence, live-loop evidence, and a compact contact-sheet summary while keeping the renderer truth-read-only.

## Native Presentation Target Audit

```sh
./tools/renderer_target_audit.sh artifacts/renderer_target/latest
```



```sh
```


## Audio/VFX Render

```sh
./tools/audio_vfx_render.sh examples/duels/basic_oathyard.duel artifacts/audio_vfx/latest
```

## Runtime Audio Mixer

```sh
./tools/audio_mixer.sh examples/duels/basic_oathyard.duel artifacts/audio_mixer/latest
```

This regenerates trace-derived presentation events through the native executable's deterministic runtime mixer path, writes `runtime_audio_mix.wav`, `audio_mixer_settings.json`, `audio_mixer_channels.json`, `audio_mixer_loudness.json`, `captions.srt`, and `audio_mixer_report.md`. It proves mixer routing, volume/mute settings, captions, and integer loudness metrics as presentation-only artifacts. It does not claim owner audio acceptance, platform audio certification, spatial mix acceptance, or final loudness approval.

## Audio Device Smoke

```sh
./tools/audio_device_smoke.sh examples/duels/basic_oathyard.duel artifacts/audio_device/latest
```

This regenerates trace-derived procedural audio, plays `audio_mix.wav` through a bounded local backend attempt such as `pw-play`, and writes `audio_device_smoke.json` plus `audio_device_smoke_report.md`. It proves local playback command success only; platform audio certification, spatial mix acceptance, final loudness approval, and human audio acceptance remain unclaimed.

## Audio Runtime Target Audit

```sh
./tools/audio_target_audit.sh artifacts/audio_target/latest
```

This checks `docs/decisions/0004-audio-runtime-target.md` against the current trace-derived audio events, deterministic mixer artifacts, local audio-device smoke, captions, and persisted runtime audio settings. It keeps shipping backend finalization, platform loudness acceptance, owner audio acceptance, public demo readiness, and release-candidate readiness false.

## Performance Benchmark

```sh
./tools/perf_benchmark.sh artifacts/perf/latest
```

This writes measured command timings and asset/package byte budgets. Timing is QA evidence only; it is not replay input and never enters gameplay truth.

## Contact Matrix

```sh
./tools/contact_matrix.sh artifacts/contact_matrix/latest
```

This runs deterministic truth scenarios across every shipped weapon, armor, attack label, and target region, then writes contact/material/anatomy/capability cause-chain coverage.

## Deterministic AI Duel

```sh
./tools/ai_duel.sh artifacts/ai/latest 6
./tools/replay_verify.sh artifacts/ai/latest/replay.json
./tools/ai_sweep.sh artifacts/ai_sweep/latest
```

This writes an observe/replan AI plan, generated scenario, trace, replay, duel report, timeline, and fight-film manifest. The AI emits legal lane actions and directional influence only; contact, injury, capability deltas, end conditions, and hashes remain authoritative truth/replay output.

The AI sweep runs multiple physical fighter/loadout pairings and planning policy styles twice each, then compares committed sequences, replay JSON, trace JSON, final hashes, end-condition status/winner, capability reactions, and replay verification evidence. Difficulty/body stats are not mutated by the planner.

## Truth Stress

```sh
./tools/truth_stress.sh artifacts/truth_stress/latest
```

This runs the same deterministic planner through six longer 24-turn physical pairings twice each, writes replay/trace/report artifacts for both runs, and checks committed sequences, replay JSON, trace JSON, turn-hash chains, contact ordering, replay verification, action validity, contact count, capability reactions, capability-stop coverage, distinct final hashes, and adversarial capability-extrema thresholds. It remains a truth/replay stress gate only; AI still authors legal planned actions and never decides contacts, injuries, outcomes, or hidden future results.

## Truth Edge Audit

```sh
./tools/truth_edge_audit.sh artifacts/truth_edge/latest
```

This writes `truth_edge_audit.json` and `truth_edge_audit_report.md`. It proves the current fixed-point/permille overflow policy, capability clamp behavior, invalid-action cost behavior, deterministic contact tie ordering, and replay schema compatibility failures. Current replay schema replays must verify; unsupported schema, missing required replay fields, and mismatched final hashes must fail loudly.

## Negative Input Audit

```sh
./tools/negative_audit.sh artifacts/negative_audit/latest
```

This writes `negative_input_audit.json` and `negative_input_audit_report.md`. It verifies malformed scenarios, invalid content manifests, unsupported/incomplete/mismatched replay files, and tampered replay export bundles all fail loudly with specific errors instead of being silently accepted.

You can also pass an artifact directory:

```sh
./tools/replay_verify.sh artifacts/latest
```

## Visual Evidence Index

```sh
./tools/visual_evidence_index.sh artifacts/visual_evidence/latest
```

This reduces source-run and package-smoke visual evidence into `visual_evidence_manifest.json`, `visual_evidence_report.md`, `visual_evidence_contact_sheet.svg`, `visual_evidence_hashes.sha256`, and `failed_visual_artifacts.txt`. It is an automated artifact inspection gate, not owner visual acceptance.

## Full Verification

```sh
./tools/verify.sh
```

This runs the strongest local check:

- docs/canon/acceptance-map presence check
- source/package readiness drift audit
- source/package secrets audit
- local build/runtime environment audit
- build
- unit and integration tests
- asset build and validation
- local structural glTF runtime export and validation
- asset budget regression audit for runtime mesh/glTF/preview/audio/VFX counts
- asset visual atlas and runtime 3D audit proving nonzero Z-depth glTF geometry and Z-depth native projection
- deterministic duel twice
- replay verification
- replay export bundle with trace, replay, report, captures, and canonical hash manifest
- contact/action/loadout matrix coverage with deterministic material/capability invariants
- deterministic AI/scripted-seat duel with replay verification
- deterministic AI planner sweep across multiple physical pairings with repeated-run sequence/hash verification
- truth stress sweep across 24-turn repeated replay traces with contact-order, turn-hash-chain stability, and adversarial solver thresholds
- truth edge audit for fixed-point overflow policy, capability clamps, deterministic contact tie ordering, and replay schema compatibility failures
- negative input audit for malformed scenarios, content manifests, replay files, and replay export bundles
- match sweep with machine-readable scripted-match, deterministic-AI, and adversarial-truth-stress rollup
- screenshot/render capture
- measured performance and asset/package budget benchmark
- Linux desktop entry/icon metadata validation, with AppStream intentionally blocked while the project is pending/unlicensed
- local package checksum generation, package smoke test, and package reproducibility check
- no-argument packaged native launch smoke through `OATHYARD_LAUNCH_SMOKE=1`
- native roster 3D showcase for all six default fighter/loadout families from runtime glTF after content hash
- input map/remapping artifacts with gamepad-ready schema, controller glyph preview, and local Steam Deck input checklist
- Linux joystick-interface gamepad smoke artifact
- native input target ADR/audit with physical controller and Steam Deck hardware acceptance still false
- accessibility settings artifacts for text scale, high contrast, captions, visual equivalents, reduced motion, and reduced flash
- runtime settings persistence roundtrip for accessibility/input/audio preferences, with byte-identical saved/loaded artifacts and no truth or replay-hash mutation
- native X11 combat overview and state sequence captured to PPM from hashed duel output
- native X11 combat projection uses extruded 3D glTF runtime geometry with nonzero Z depth
- native X11 combat live render loop with 120 replay-derived frames, five PPM samples, and deterministic loop hash
- native software-rasterized third-person and first-person 3D combat viewport PPMs using depth-sorted filled runtime glTF triangles
- visual evidence index with source-run/package-smoke contact-sheet rollup and reduced failed-artifact list
- native presentation target ADR/audit with production renderer completion still false
- trace-derived procedural WAV, captions, and VFX manifest
- live audio-device playback smoke through the local backend, with human audio acceptance still false
- audio runtime target ADR/audit with shipping backend, platform loudness, and owner audio acceptance still false
- replay-relevant artifact comparison
- truth-path static audit
- artifact validity check

## Publishable Local Gate

```sh
./tools/publishable_gate.sh
```

This wraps `./tools/verify.sh`, writes a timestamped evidence bundle under `artifacts/publishable/<UTC>/`, records final/package hashes, and keeps public/store release blockers explicit.
It also writes a timestamp-local visual evidence report/contact sheet and `failed_visual_artifacts.txt` so failed or missing visual evidence is reduced to one triage file.

## Linux Desktop Metadata

```sh
./tools/desktop_metadata.sh artifacts/desktop_metadata/latest
```


## Package Verification

```sh
./tools/package.sh
(cd artifacts/package && sha256sum -c oathyard-linux-x86_64.tar.sha256)
./tools/smoke_package.sh artifacts/package/oathyard-linux-x86_64.tar
./tools/check_package_repro.sh artifacts/package_repro/latest
```

This verifies the local package tarball checksum, package contents checksum manifest, packaged Linux desktop entry/icon files, clean unpacked package smoke, no-argument launch, `.desktop` Exec launch, packaged replay export/verify, and byte-for-byte reproducibility across two local package builds. Package smoke writes `artifacts/package_smoke/package_smoke.json` and `artifacts/package_smoke/package_smoke_report.md` with clean-root and launch evidence.
Package smoke also writes `artifacts/package_smoke/environment_audit/` to record the host tools/runtime surface used for the clean unpack smoke without claiming a clean VM/container.

## Acceptance Map

`ACCEPTANCE_MAP.md` defines the active publishable goal and keeps local package readiness separate from public/store release readiness. A local package can pass while owner acceptance, legal clearance, trademark clearance, and store readiness remain false.

## Native Presentation Status

The executable is native Rust and OATHYARD's product target is 3D. Current gates produce deterministic visual artifacts, reports, source-built 3D glTF runtime assets, all-six-family native roster 3D showcase frames, replay-derived native combat 3D frames, and packageable local runtime content. SDL2, GLFW, and Vulkan pkg-config metadata were unavailable during the latest audit; Vulkan runtime/tooling command `vulkaninfo` is present, but no Vulkan renderer is implemented or claimed. Blender currently fails at startup, so production renderer completion beyond the verified raw-X11 3D evidence, Blender round-trip, external Khronos glTF validation, and DCC-generated GLB assets are not claimed. The X11 shell verifies keyboard, mouse-zone, and default gamepad-command navigation evidence, `input-map` writes a controller profile/glyph/local Steam Deck checklist, `gamepad-smoke` verifies the Linux joystick interface when present, `runtime-settings` verifies deterministic presentation-only settings persistence, `audio-mixer` verifies deterministic runtime mixer artifacts, `audio-device-smoke` verifies bounded local playback through the system audio stack, and `audio_target_audit` keeps the shipping backend/loudness/owner acceptance boundary explicit. Physical controller ergonomics, Steam Deck hardware compliance, final audio acceptance, and owner audio/visual/input acceptance remain unclaimed. Browser/HTML output is not used as product presentation.

## Canon

Canon precedence:

1. `docs/design/GAME_CANON.md`
2. `docs/design/DEMO_SCOPE.md`
3. `ACCEPTANCE_MAP.md`
4. `AGENTS.md` / `CLAUDE.md`
5. PRDs/specs
6. Code comments
