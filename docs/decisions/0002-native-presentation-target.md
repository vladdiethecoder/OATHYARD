# 0002: Native Presentation Target

## Decision

OATHYARD keeps the current dependency-light raw X11/XWayland + software PPM renderer as the local verification backend while defining a stricter production native renderer target.

No SDL, GLFW, Vulkan, OpenGL abstraction, engine framework, or browser presentation dependency is adopted by this decision. The next production renderer implementation must either deepen the current dependency-zero native path or introduce a backend through a separate ADR with measured build impact, license surface, capture proof, input implications, and deterministic truth-boundary proof.

## Current Evidence / Boundary

Current status is **3D evidence present, product 3D gameplay not complete**. The current artifacts are local renderer evidence: raw X11/XWayland surfaces, software PPM captures, runtime glTF geometry with nonzero Z depth, all-six-family roster 3D showcase frames, and software-rasterized filled-triangle previews after truth hashes. They prove that 3D source geometry exists and that local presentation probes can consume it; they do not prove a continuous player-facing 3D gameplay renderer.

Current machine-readable boundary fields:

- `game_is_3d` must be true when runtime glTF 3D evidence is present.
- `product_3d_gameplay_complete` must remain false until the production renderer gate is met.
- `continuous_player_facing_3d_render_loop` is now `true` in the native combat render manifest: the same X11/XWayland runtime path renders a 120-frame player-facing loop across menu, select, planning, combat, and replay screens with timing recorded outside truth and the truth hash verified unchanged. `product_3d_gameplay_complete` remains false until full production fidelity, camera parity, and owner visual acceptance are met.

The local environment audit records:

- `x11`, `wayland-client`, `egl`, and `gl` pkg-config metadata are available.
- `sdl2`, `glfw3`, and `vulkan` pkg-config metadata are unavailable.
- `vulkaninfo` and `glxinfo` commands are available as runtime/tool probes.
- `DISPLAY` and `WAYLAND_DISPLAY` are set on the current Linux/Wayland host.

The current native artifacts prove a local presentation path:

- native roster 3D showcase PPM frames, report, manifest, and contact sheet covering all six default fighter/loadout families rendered from runtime glTF after content hashes;
- native combat PPM overview, state frames, motion frames, playback-loop final capture, 120-frame truth-rate live loop sample captures, 120-frame continuous player-facing native loop covering menu/select/planning/combat/replay screens through the same X11/XWayland runtime path with presentation-only timing, third-person and first-person software-rasterized 3D mesh viewport PPMs, a 21-frame replay-derived software 3D mesh sequence using depth-sorted filled runtime glTF triangles after truth hashes, visual audit, contact sheet, and integer oblique projection of source-backed 3D runtime glTF geometry with nonzero Z depth;
- truth-after-hash read-only renderer boundary;
- public-demo-ready and release-candidate-ready remain false.

## Production Acceptance Target

The production native renderer target is Linux-native first, with Steam Deck as an explicit compatibility target and Windows deferred until a backend ADR proves toolchain and runtime support.

To count as production 3D gameplay / production renderer complete, OATHYARD must provide all of the following:

- a continuous player-facing native render loop, not only command-generated captures;
- menu, loadout, planning, combat, consequence, fight-film, settings, and accessibility screens rendered through the same runtime path;
- first-person, third-person, and fight-film camera parity where the same truth events remain inspectable;
- source-backed 3D arena, fighter, weapon, armor, contact, injury, VFX, and UI assets loaded through validated manifests;
- readable hit, bind, guard, stagger, collapse, injury, recovery, material, and frame-cost states;
- deterministic capture tooling that can write PPM/PNG or another documented local artifact without mutating truth;
- 1280x720, 1280x800, and desktop-window readability evidence;
- frame timing and startup measurements outside authoritative truth;
- package smoke that launches the renderer through no-argument executable and desktop-entry paths;
- owner visual acceptance recorded separately.

## Truth Boundary

Renderer, UI, camera, VFX, and audio are presentation consumers only. They may read trace/replay/truth state after authoritative hashes are computed, but they must not write gameplay truth, action validity, costs, contact packets, injuries, capability deltas, end conditions, or replay hashes.


## Backend Policy

Current accepted local verification backend:

- raw X11/XWayland surface creation;
- software PPM frame generation;
- deterministic report/manifest/contact-sheet artifacts;
- no external graphics runtime dependency beyond system X11 surfaces.

Rejected for immediate adoption in this ADR:

- Unity, Unreal, Godot, or browser-first presentation;
- SDL/GLFW renderer adoption while pkg-config metadata is unavailable locally;
- Vulkan renderer adoption while Vulkan pkg-config metadata is unavailable and no render spike exists;
- graphics dependencies without license/build/determinism impact notes;
- any renderer path that feeds presentation state back into gameplay truth.

## Visual Acceptance Protocol

Automated renderer gates can prove local artifact presence, truth-read-only boundaries, resolution coverage, and basic readability checks. They cannot prove owner-final visual acceptance.

Owner visual acceptance requires an explicit review of a current capture pack generated from the current native executable. Until that review exists, these flags remain false:

- public-demo-ready
- release-candidate-ready
- owner-final-accepted

## Verification

Primary commands:

```sh
./tools/audit_environment.sh artifacts/environment/verify
./tools/native_roster_showcase.sh artifacts/native_roster/verify
./tools/native_combat_render.sh examples/duels/basic_oathyard.duel artifacts/native_combat/verify
./tools/renderer_target_audit.sh artifacts/renderer_target/verify
./tools/verify.sh
```

The renderer target audit is an acceptance guard for this ADR. Passing it means the target definition and local 3D renderer evidence are internally consistent and that OATHYARD remains a native 3D game with production gameplay-renderer completion still false. It does not claim production renderer completion, public demo readiness, release-candidate readiness, or owner visual acceptance.
