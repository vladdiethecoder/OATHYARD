# OATHYARD Full Game Roadmap

This roadmap is fail-closed. OATHYARD is not full-game complete until every acceptance gate in `docs/acceptance/FULL_GAME_ACCEPTANCE.md` passes and external gates remain explicitly false unless actually performed.

Detailed execution board: `docs/roadmap/PUBLISHABLE_KANBAN.md`.
M0-M21 canon/acceptance decomposition: `docs/roadmap/M0_M21_CANON_ACCEPTANCE_DECOMPOSITION.md`.
Acceptance map and active publishable goal: `ACCEPTANCE_MAP.md`.
High-fidelity production target: `docs/decisions/0007-high-fidelity-production-target.md`.

## High-Fidelity Target Reset

Current local deterministic/package gates can pass while the full high-fidelity game remains incomplete. The production target now requires a premium native-PC 3D renderer or approved engine integration, source-backed production-grade fighters/weapons/armor/arenas, PBR/equivalent materials, skeletal/skinned presentation, lighting/atmosphere, deterministic 1920x1080+ capture coverage, visual benchmark reporting, and owner visual acceptance.

Fresh baseline `artifacts/baseline/20260629T164832Z/` proves the existing local gate passes with final replay hash `f17c8f76b9dfae86`, but current pixel audits reject the visuals as prototype/placeholder-level. The next roadmap work must treat raw X11/PPM/low-poly evidence as verification scaffolding, not product visuals.

## M0-M6 Completed Foundation

- Rust/Cargo source build.
- Deterministic fixed-step truth at 120 Hz.
- Fixed-point/integer truth types.
- Canonical 16-joint body graph plus grip capability fields.
- Scripted duel parser, simultaneous commit/reveal, cost breakdowns, contact packets, injury/capability deltas, replay hashes, report, trace, fight-film manifest, and SVG timeline.
- Longer deterministic truth stress traces repeat bit-stably across planner pairings and verify contact ordering, turn-hash chains, replay equality, and capability-stop coverage.
- Truth edge audit verifies fixed-point/permille overflow policy, capability clamps, contact tie ordering, invalid-action cost response, and loud replay schema/hash failures.

## M7-M8 Asset And Content Expansion

- Add source-backed repo-owned assets under `assets_src/`.
- Build runtime assets under `assets/` from source.
- Validate provenance, rigs, physical profiles, deterministic runtime glTF files, previews, arena metadata, and no production placeholder markers.
- Expand deterministic content to at least eight physical weapon families, six armor/loadout families, six fighter traditions, OATHYARD verdict ring, and training yard.
- Verify all six default fighter/loadout families through native-software 3D roster showcase frames rendered from runtime glTF after content hashes.

## M9-M14 Playable Native Game Systems

- Deepen local game flow from the current replay-backed native menu/loadout/plan smoke with editable loadout/timeline state into persistent editable menus, mode selection, fighter/loadout selection, planning timeline, consequence screen, replay view, and settings/accessibility behavior.
- Preserve `docs/decisions/0003-native-input-model.md` as the production input command-boundary target while current controller evidence remains local schema, native command-flow, and interface evidence only.
- Verify keyboard, mouse-zone, default gamepad-command navigation, remappable input artifacts, and deterministic runtime settings persistence for accessibility/input/audio preferences; keep editable native settings UI depth and physical controller/Steam Deck hardware acceptance as separate gates.
- Add scripted/AI deterministic seats and automated match sweeps.
- Promote replay-verified fight-film camera manifests into richer native playback/capture with shot controls and readable player state.
- Add presentation states for hit, bind, guard, stagger, collapse, injury, and recovery as consumers of truth.
- Native graphics window is required for product-presentation completion; until a continuous player-facing 3D renderer is verified, deterministic 3D rendered artifacts are substitute evidence only.

## M15-M17 Ship-Local Quality

- Add measured performance summaries, asset budgets, package creation, package smoke launch, clean install/run verification, and final acceptance report.
- Validate packaged Linux `.desktop` and icon metadata locally; keep AppStream/metainfo blocked until owner-approved license/distribution terms exist.
- Record the local build/runtime environment for every verification cycle, including required tools, optional native graphics/DCC/audio tools, pkg-config libraries, and runtime surfaces; do not claim clean VM/container evidence until it is actually run.
- Audit source and packaged docs/manifests for readiness drift so external public/release/legal/store claims remain false until their gates are evidenced.
- Audit malformed scenarios, content manifests, replay files, replay export bundles so bad inputs fail loudly with specific errors instead of silent acceptance.
- Audit source, generated text artifacts/logs, and package content for credentials, private keys, service tokens, webhook secrets, and non-placeholder secret assignments.
- Keep runtime settings persistence presentation-only and outside replay hashes while expanding native settings UI, audio backend, and accessibility evidence; preserve `docs/decisions/0004-audio-runtime-target.md` and `./tools/audio_target_audit.sh` until a measured shipping audio backend ADR replaces the current local boundary.
- Keep `./tools/asset_budget_audit.sh` current as production assets grow; budget increases require evidence, not silent ceiling inflation.
- Keep public-demo-ready, release-candidate-ready, and owner-final-accepted false unless those gates are actually completed.
