# OATHYARD Full Game Acceptance

## Current Status

Partial. The deterministic duel foundation is verified. Full-game completion is not yet claimed.

The high-fidelity production target is now recorded in `docs/decisions/0007-high-fidelity-production-target.md`. Current local raw-X11/software-PPM/low-poly glTF evidence is verification scaffolding only; it does not satisfy the high-fidelity native-PC 3D visual target.

Known blockers or constraints discovered locally:

- SDL2 and GLFW pkg-config metadata are unavailable.
- Vulkan runtime/tooling command `vulkaninfo` is present, but Vulkan pkg-config metadata is unavailable and no Vulkan renderer is implemented or claimed.
- Blender is installed but fails to start with a MaterialX symbol lookup error.
- `gltf-validator`, `gltfpack`, `toktx`, `sox`, ALSA development metadata, PulseAudio development metadata, and OpenAL metadata are unavailable. Local structural glTF export/validation and nonzero-Z runtime 3D audits exist, but external Khronos validation is not claimed.
- Git is initialized locally on branch `main`, with generated `target/`, `artifacts/`, and `assets/` ignored. A baseline commit, remote, and issue tracker are not yet created.
- Local host build/runtime environment audit exists; a separate clean OS user, VM, or container run is not claimed.

## Required Gates

`./tools/verify.sh` must pass and include:

- build
- tests
- source/package readiness drift audit
- source/package secrets audit
- local build/runtime environment audit
- truth audit over every Rust source file under `src/`
- asset build
- asset validation
- asset preview render manifest/contact sheet for fighters, weapons, armor, and arenas
- high-fidelity production target ADR and visual benchmark gap report
- local structural glTF runtime export and validation
- asset visual atlas and runtime 3D audit requiring source-backed runtime visuals, nonzero-Z glTF geometry, and native Z-depth projection
- asset budget regression audit for runtime mesh/glTF/preview/audio/VFX counts
- deterministic duel twice
- replay verification
- contact/action/loadout matrix coverage with deterministic material/capability invariants
- deterministic AI/scripted-seat duel and replay verification
- deterministic AI planner sweep across multiple physical pairings with repeated-run sequence/hash verification
- truth stress sweep across 24-turn repeated replay traces with contact-order, turn-hash-chain stability, and adversarial solver thresholds
- truth edge audit for fixed-point/permille overflow policy, capability clamps, deterministic contact tie ordering, and replay schema compatibility failures
- negative input audit for malformed scenarios, content manifests, replay files, replay export bundles
- match sweep with machine-readable scripted-match, deterministic-AI, and adversarial-truth-stress rollup
- screenshot/render capture
- measured performance and asset/package budget benchmark
- native input map/remapping artifacts with controller profile, glyph preview, and local Steam Deck checklist
- Linux joystick-interface gamepad smoke artifact
- native input target ADR/audit, with physical controller hardware, Steam Deck hardware, and owner input acceptance still false
- accessibility settings artifacts for text scale, contrast, captions, visual equivalents, remapping, reduced motion, and reduced flash
- runtime settings persistence roundtrip for accessibility, input, and audio preferences, with byte-identical saved/loaded artifacts and no truth or replay-hash mutation
- native roster 3D showcase for all six default fighter/loadout families from runtime glTF after content hashes
- native combat overview and state-sequence capture
- visual evidence reducer with source-run/package-smoke visual manifest, contact-sheet rollup, hashes, and reduced failed-artifact list
- native presentation target ADR/audit, with production renderer completion and owner visual acceptance still false
- trace-derived audio/VFX render and captions
- bounded live audio-device playback smoke through the local backend
- audio runtime target ADR/audit, with shipping backend finalization, platform loudness acceptance, and owner audio acceptance still false
- Linux desktop entry and icon validation, with AppStream/metainfo blocked while license/distribution remains undecided
- package build
- package smoke test
- artifact validation

## Full-Game Completion Criteria

Full-game complete requires:

- Native executable launches and runs the local game flow.
- Native roster 3D showcase captures all six default fighter/loadout families as source-backed runtime glTF 3D PPM frames after content hashes, including fighter/weapon/armor identifiers, depth-sorted filled triangle evidence, contact sheet, report, and false owner/public/release readiness flags.
- Keyboard and mouse-zone input paths are verified; native default gamepad-command navigation reaches every current screen and records glyph coverage; controller profile, glyph preview, and local Steam Deck checklist artifacts are generated; Linux joystick-interface smoke proves local native input visibility when present, but physical controller ergonomics, Steam Deck hardware compliance, and owner input acceptance require explicit external/hardware evidence.
- Native input target audit proves the current command boundary is presentation-only until replayable committed inputs, validates remapping/controller-schema/native-controller-command/runtime-settings evidence, and keeps physical controller hardware, Steam Deck hardware, and owner input acceptance false.
- Native combat render captures replay/duel-derived overview, source-backed weapon/armor silhouettes reconstructed from canonical scenario loadouts, active weapon/armor/arena runtime mesh/glTF/preview references, integer-projected generated extruded 3D glTF triangle geometry with Z-depth oblique projection for active weapons/armor/arena, 21 replay-derived motion frames, a 42-frame native X11 playback-loop final capture, a 120-frame truth-rate native live loop with five PPM sample captures and deterministic loop hash, third-person and first-person software-rasterized 3D mesh viewport PPMs, a 21-frame replay-derived software 3D mesh sequence using depth-sorted filled runtime glTF triangles after truth hashes, 1280x720/1280x800 resolution evidence, 12 state-sequence frames from truth-after-hash data, and an automated visual audit/contact sheet covering observe/plan, guard/bind, parry, weapon arc, hit/contact, armor/material solve, injury/capability, grip loss, stagger/collapse-risk, near miss/replan, recovery, and final hash evidence.
- Visual evidence reducer indexes source-run and package-smoke visual artifacts into a deterministic manifest/report/contact sheet, records artifact hashes, and writes `failed_visual_artifacts.txt` so missing or failing visual evidence is immediately triaged without claiming owner visual acceptance.
- At least six fighter traditions, eight weapon families, six armor/loadout families, OATHYARD verdict ring, and training arena are present in content and assets.
- Assets are source-backed, repo-owned, provenance-tagged, built into runtime assets, and loaded or validated by tooling.
- Runtime production glTF assets are generated from source with nonzero Z depth and local structural validation; external validator/DCC round-trip evidence is required before claiming DCC readiness.
- No production placeholder primitives or copied assets are accepted.
- Replay includes schema, initial state, committed inputs, content hashes, asset hashes, state hashes, and deterministic non-HP end-condition status/winner evidence.
- Trace artifacts declare the simultaneous contact ordering rule, and automated gates verify same-turn contact packets are emitted in deterministic frame order.
- Replay browser artifacts list saved replays, verification status, content hashes, final hashes, and fail loudly on corrupt fixtures.
- Contact/action/loadout matrix artifacts prove shipped weapons, armor, attack labels, and target regions produce deterministic material/anatomy/capability cause chains.
- Deterministic AI/scripted seats emit legal planned actions and directional influence only; AI sweep proves repeated committed sequences, replay JSON, trace JSON, final hashes, end-condition status/winner, capability reactions, policy-style variation, and replay verification across multiple physical pairings.
- Truth stress artifacts prove longer repeated replay traces keep committed sequences, replay JSON, trace JSON, turn-hash chains, contact ordering, action validity, capability-stop outcomes, distinct final hashes, and adversarial capability-extrema thresholds deterministic.
- Truth edge audit artifacts prove fixed-point/permille overflow behavior, capability clamps, invalid-action cost response, deterministic contact tie ordering, and loud replay schema/hash failure behavior.
- Negative input audit artifacts prove malformed scenarios, invalid content manifests, unsupported/incomplete/mismatched replay files, tampered replay export bundles fail loudly with specific errors.
- Asset budget audit measures runtime mesh/glTF/preview bytes, source bytes, audio/VFX event counts, vertices, indices, triangles, material counts, and primitive counts, and fails if local budget ceilings or minimum coverage counts regress.
- Asset visual atlas and runtime 3D audit prove every production runtime glTF asset has nonzero Z depth, no source/runtime/preview/provenance gaps, and active native combat projection consumes Z depth after truth hashes.
- Renderer, UI, audio, VFX, camera, and fight-film systems consume truth after hashing and never mutate truth.
- Source and packaged documentation/manifests are audited so public-demo, release-candidate, owner-final, legal, trademark, and store readiness cannot drift true before external gates are complete.
- Source, generated text artifacts/logs, and package text content are audited so credentials, private keys, service tokens, webhook secrets, and non-placeholder secret assignments are not committed or packaged.
- Accessibility settings for text scale, contrast, captions, visual equivalents, remapping, reduced motion, and reduced flash are present and presentation-only.
- Runtime settings for accessibility, input alternatives, and audio gains/mute persist through deterministic save/load artifacts, remain presentation-only, and do not enter replay hashes.
- Audio/VFX artifacts are generated from trace events only and include captions or visual equivalents for critical audio.
- Runtime audio mixer artifacts prove deterministic routing, volume/mute settings, captions, and integer loudness metrics as presentation-only outputs.
- Audio-device smoke proves bounded local backend playback of trace-derived audio; final loudness approval, platform audio certification, and human audio acceptance remain separate.
- Audio runtime target audit records the current backend boundary and must keep shipping backend finalization, platform loudness acceptance, owner audio acceptance, public demo readiness, and release-candidate readiness false until those gates are actually performed.
- Package artifact can be unpacked and smoke-run from a clean directory, with machine-readable package-smoke evidence recording the clean smoke root, checksum verification, no-argument launch, `.desktop` launch, replay verification, readiness audit, and secrets audit.
- Package smoke records the host environment audit used for the clean unpack smoke, while clean VM/container evidence remains a separate external gate.
- Package artifact includes validated Linux `.desktop` and icon metadata, and package smoke launches through the packaged `.desktop` `Exec=oathyard` path; AppStream/metainfo remains external-blocked until a license/distribution decision exists.

## External Gates

These remain false unless separately performed:

- public demo readiness
- release-candidate readiness
- owner-final acceptance
- legal clearance
- trademark clearance
- store readiness
