# 0008: HIFI-WO-01 renderer/backend ADR and continuous-loop spike boundary

Status: Accepted for spike execution only; no production renderer/backend adoption.
Date: 2026-06-29T20:34:41Z
Kanban: t_05a6f650
Work order: HIFI-WO-01 from `docs/roadmap/HIGH_FIDELITY_PRODUCTION_WORK_ORDERS.md`

## Decision

OATHYARD will execute HIFI-WO-01 with a dependency-zero blocked native-renderer status/GLX/OpenGL continuous player-facing native 3D loop spike under `spikes/renderer/002-raw-opengl-player-loop/`.

This ADR does not adopt OpenGL, Vulkan, SDL, GLFW, Unity, Unreal, Godot, a browser renderer, or any other backend into production runtime or packaging. It authorizes a disposable spike to measure whether the raw native path can route player-facing screens, cameras, replay-derived truth data, deterministic 1920x1080+ capture, input/audio boundaries, and package impact without adding project dependencies or mutating authoritative truth.

The accepted production renderer remains incomplete. These flags remain false unless a later current-run gate and owner/external evidence explicitly satisfy them:

```text
production_renderer_complete: false
owner_visual_acceptance: false
owner_final_acceptance: false
public_demo_ready: false
release_candidate_ready: false
legal_clearance: false
trademark_clearance: false
store_readiness: false
```

## Canon and acceptance basis reviewed

- `docs/design/GAME_CANON.md`: renderer/UI/audio/camera consume fixed 120 Hz deterministic truth only after hashes; no presentation writes into gameplay truth.
- `docs/design/DEMO_SCOPE.md`: blocked native-renderer status/non-native diagram/non-native frame/low-poly evidence is local verification only and cannot satisfy high-fidelity product presentation.
- `ACCEPTANCE_MAP.md`: local package gate and high-fidelity production gate are separate; external readiness flags remain false.
- `docs/decisions/0007-high-fidelity-production-target.md`: high-fidelity target requires continuous native 3D renderer or legally available engine integration plus PBR/equivalent assets, 1920x1080+ captures, benchmark review, and owner visual acceptance.
- `docs/decisions/0002-native-presentation-target.md`: current blocked native-renderer status/non-native frame path is local evidence; backend adoption requires measured ADR and truth-boundary proof.
- `docs/decisions/0005-renderer-toolchain-spike-path.md`: raw OpenGL is the smallest measured next spike because `x11`, `egl`, and `gl` metadata are available while `vulkan`, `sdl2`, and `glfw3` pkg-config metadata are absent.
- `docs/decisions/0006-raw-opengl-native-loop-spike-result.md`: previous raw OpenGL loop validated context/capture feasibility only; visuals remained debug/diagnostic and were not product evidence.

## Backend candidates considered

| Candidate | Current measurement | Decision for this slice |
| --- | --- | --- |
| Dependency-zero blocked native-renderer status/GLX/OpenGL spike | `pkg-config --modversion x11 gl egl` reported `1.8.13`, `1.2`, `1.5`; `glxinfo -B` reported direct rendering on display `:0` and OpenGL/Mesa renderer metadata; previous spike rendered 22 glTF assets and 1920x1080 non-native frame. | Use only as disposable HIFI-WO-01 spike. No production/package adoption. |
| Vulkan/direct-loader path | `vulkaninfo --summary` previously showed runtime-capable GPUs, but `pkg-config --modversion vulkan` currently fails. | Blocked for adoption until a separate direct-loader/header spike records license/build/package/capture impact. |
| SDL2 / GLFW | `pkg-config --modversion sdl2 glfw3` currently fails. | Do not adopt; missing local metadata and adds unmeasured window/input dependency. |
| Unity / Unreal / Godot | Forbidden for this slice by work order and AGENTS rules unless owner approves a new ADR. | Rejected for this slice. |
| Browser/HTML renderer | Browser output may be QA-only and cannot be native product presentation. | Rejected as product path. |

## Measured dependency and license impact

Current tool measurements from this task run:

```text
rustc 1.96.0 (ac68faa20 2026-05-25)
cargo 1.96.0 (30a34c682 2026-05-25)
cc (GCC) 16.1.1 20260515 (Red Hat 16.1.1-2)
pkg-config x11: 1.8.13
pkg-config gl: 1.2
pkg-config egl: 1.5
pkg-config vulkan: missing
pkg-config sdl2: missing
pkg-config glfw3: missing
glxinfo -B: direct rendering yes on display :0
```

Project dependency impact:

- Cargo dependency delta: none; `Cargo.toml` still has no `[dependencies]` section.
- Vendored dependency delta: none.
- Engine/framework delta: none.
- Spike compile link surface: host system `x11` and `gl` through `pkg-config`, plus C math library.
- License impact for source tree: no third-party source copied; direct system library link only in disposable artifact under `artifacts/renderer_spikes/`.
- Adoption/removal plan: delete `spikes/renderer/002-raw-opengl-player-loop/` and generated `artifacts/renderer_spikes/hifi_wo_01_player_loop*` without touching Cargo, package scripts, truth code, or runtime assets.

## Build and package impact boundary

Pre-spike package observation before adding this ADR/spike:

```text
artifacts/package/oathyard-linux-x86_64.tar 13363200 bytes
artifacts/package/oathyard-linux-x86_64/bin/oathyard 12905896 bytes
```

Expected impact before verification:

- Runtime binary impact: zero, because the spike is not linked into `src/bin/oathyard.rs` or the Cargo build.
- Package content impact: this ADR is copied into `docs/decisions` by `tools/package.sh`; package tar may grow by the ADR/doc delta if the package is regenerated.
- Spike binary impact: outside package only, under the selected spike output directory.
- No `tools/package.sh` adoption of OpenGL, Vulkan, SDL, GLFW, engines, browser assets, network services, installers, or telemetry.

A post-spike package measurement must record current tar/binary size and show whether any size delta is documentation-only.

## Continuous loop spike contract

The authorized spike must provide:

- a persistent native X11/GLX window loop for at least 240 presentation frames;
- a player-facing route through these screen/state labels: `main_menu`, `fighter_select`, `loadout_select`, `planning_timeline`, `combat_exchange`, `consequence_readout`, `replay_browser`, `settings_accessibility`;
- camera-mode metadata for `third_person_verdict_ring`, `first_person_guard_line`, and `fight_film_orbit` over the same verified replay/content hashes;
- current runtime glTF geometry loaded from `assets/runtime_manifest.json` with nonzero-Z assets;
- replay verification before the render/capture path and again after capture;
- at least one current-run 1920x1080+ non-native frame capture, not an upscale;
- frame timing measured outside authoritative truth;
- manifest fields proving `truth_mutation: false`, `project_dependency_adopted: false`, `production_renderer_complete: false`, `owner_visual_acceptance_claimed: false`, `public_demo_ready: false`, and `release_candidate_ready: false`.

## Truth boundary schema

Presentation input consumed by the spike is read-only and must be represented as:

```text
renderer_backend_id: raw-x11-glx-opengl-player-loop-spike
replay_file: path to replay verified before and after capture
replay_schema_seen: oathyard.replay.v1
replay_final_state_hash: final hash parsed from replay
content_hash: content hash parsed from replay
runtime_manifest: assets/runtime_manifest.json
screen_route: ordered presentation screen labels
camera_modes: ordered presentation camera labels
capture_settings: file, width, height, bytes, hash in sidecar evidence
truth_mutation: false
presentation_only: true
```

The spike may read replay JSON, content hash, runtime glTF vertices/indices, and static presentation route metadata. It must not write replay JSON, trace JSON, action costs, contact packets, injuries, capability deltas, end conditions, content tables, or any gameplay hash source.

## Input impact

Input remains presentation-command boundary only in this slice:

- no physical controller, Steam Deck hardware, or owner input acceptance claim;
- no input dependency adoption such as SDL/GLFW/winit;
- any player-facing navigation labels in the spike are visual route metadata, not authoritative committed actions;
- committed gameplay inputs remain authored by replay/scenario paths and verified by replay tools.

## Audio impact

Audio remains unchanged:

- no audio backend adoption;
- no mixer, device, platform loudness, or owner audio acceptance claim;
- any future audio/VFX synchronization must consume hashed truth events after replay verification, matching `docs/decisions/0004-audio-runtime-target.md`.

## Capture impact

The spike capture path writes local non-native frame files from the current OpenGL framebuffer. non-native frame captures remain local evidence only. They do not prove high-fidelity product presentation, owner visual acceptance, public demo readiness, release-candidate readiness, legal clearance, trademark clearance, or store readiness.

The 1920x1080 capture requirement for HIFI-WO-01 is a current-run technical proof requirement; it is not high-fidelity visual acceptance by itself.

## Acceptance commands for this ADR

Run from repo root with the user's Rust toolchain selected when needed:

```sh
export RUSTUP_HOME=/home/vdubrov/.rustup
export CARGO_HOME=/home/vdubrov/.cargo
spikes/renderer/002-raw-opengl-player-loop/run.sh \
  artifacts/renderer_spikes/hifi_wo_01_player_loop \
  artifacts/renderer_spikes/hifi_wo_01_duel
./tools/replay_verify.sh artifacts/renderer_spikes/hifi_wo_01_duel/replay.json
./tools/renderer_target_audit.sh artifacts/renderer_target/hifi_wo_01
./tools/audit_readiness.sh . artifacts/readiness/hifi_wo_01
./tools/package.sh
```

Renderer target audit may continue to report `continuous_player_facing_3d_render_loop: false` and `production_renderer_complete: false` because this ADR authorizes a spike, not production adoption.

## Falsification criteria

Reject raw OpenGL as the next backend path if the spike:

- cannot create a native window/context on the current host;
- cannot render current runtime glTF geometry after replay verification;
- cannot produce a real 1920x1080+ capture;
- mutates replay/truth artifacts or changes final/content hashes;
- requires a new Cargo crate, vendored blob, SDL/GLFW/framework/engine/browser path, network service, telemetry, or package-script adoption;
- produces only metadata without inspected pixels;
- causes readiness audits to accept false public/release/owner/legal/store claims.

## Current conclusion

Proceed with the HIFI-WO-01 raw OpenGL player-loop spike and evidence collection. Do not adopt any renderer backend into production until a later ADR has current-run visual quality evidence, build/package/input/audio/truth measurements, and an owner-approved acceptance path.
