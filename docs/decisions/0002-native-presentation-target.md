# 0002: Native Presentation Target

## Decision

OATHYARD keeps the current dependency-light truth/replay/status verification backend while defining a stricter production native renderer target.

No SDL, GLFW, Vulkan, OpenGL abstraction, engine framework, or browser presentation dependency is adopted by this decision. The next production renderer implementation must either deepen the current dependency-zero native path or introduce a backend through a separate ADR with measured build impact, license surface, capture proof, input implications, and deterministic truth-boundary proof.

## Current Evidence / Boundary

Current status is **3D source geometry present, product 3D gameplay renderer not complete**. The current artifacts are nonvisual status evidence plus runtime glTF geometry with nonzero Z depth. They prove that 3D source geometry exists and that presentation probes can consume truth after hashes; they do not prove a continuous player-facing 3D gameplay renderer or accepted visual evidence.

Current machine-readable boundary fields:

- `game_is_3d` must be true when runtime glTF 3D evidence is present.
- `product_3d_gameplay_complete` must remain false until the production renderer gate is met.
- `continuous_player_facing_3d_render_loop` must remain false until a native 3D renderer/camera path emits manifest-backed captures across menu, select, planning, combat, and replay screens with timing recorded outside truth and the truth hash verified unchanged. `product_3d_gameplay_complete` remains false until full production fidelity, camera parity, and owner visual acceptance are met.

The local environment audit records:

- `x11`, `wayland-client`, `egl`, and `gl` pkg-config metadata are available.
- `sdl2`, `glfw3`, and `vulkan` pkg-config metadata are unavailable.
- `vulkaninfo` and `glxinfo` commands are available as runtime/tool probes.
- `DISPLAY` and `WAYLAND_DISPLAY` are set on the current Linux/Wayland host.

The current native artifacts prove only a local presentation boundary, not accepted visual evidence:

- native roster status manifest/report covering all six default fighter/loadout families from runtime glTF after content hashes;
- native combat status manifest/report consuming replay data after hashes and blocking visual output until a native 3D renderer/camera path exists;
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
- deterministic capture tooling that can write manifest-backed native 3D renderer/engine captures without mutating truth;
- 1280x720, 1280x800, and desktop-window readability evidence;
- frame timing and startup measurements outside authoritative truth;
- package smoke that launches the renderer through no-argument executable and desktop-entry paths;
- owner visual acceptance recorded separately.

## Truth Boundary

Renderer, UI, camera, VFX, and audio are presentation consumers only. They may read trace/replay/truth state after authoritative hashes are computed, but they must not write gameplay truth, action validity, costs, contact packets, injuries, capability deltas, end conditions, or replay hashes.


## Backend Policy

Current accepted local verification backend:

- deterministic truth/replay/status reports and manifests;
- native 3D renderer capture remains blocked until the backend emits captures with renderer/asset/camera/replay metadata and `truth_mutation=false`;
- fallback non-3D visual output is not accepted by audits, bundles, or visual verification.

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
