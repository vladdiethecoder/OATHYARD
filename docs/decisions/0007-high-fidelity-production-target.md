# 0007: High-Fidelity Production Target

Status: Accepted as production target; not accepted as completion evidence.

Date: 2026-06-29

## Context

OATHYARD's current verified local gate proves deterministic truth, replay, source-backed low-poly 3D runtime glTF assets, packaging, and broad audit tooling. That evidence is valuable, but it is not a high-fidelity native-PC product renderer, not a production asset set, and not visual acceptance.

The production target is now a complete native-PC 3D planned-time physical melee duel game with visual fidelity and melee readability ambition benchmarked against Elden Ring and For Honor. Those titles are references for quality bar only: dark-fantasy atmosphere, rich material response, complex arenas, readable melee intent, physical-feeling weapons, duel clarity, training/readability tooling, and best-of-five presentation. OATHYARD must remain original and must not copy names, assets, silhouettes, factions, UI, animations, lore, characters, textures, music, or proprietary mechanics from either reference.

## Decision

OATHYARD's production visual target is high-fidelity native 3D. Completion requires a continuous player-facing native 3D renderer or legally available high-fidelity engine integration that preserves deterministic truth isolation.

The current status-manifest path remains a local verification backend only. It can prove determinism, replay-to-presentation data flow, and source-backed nonzero-Z geometry. It cannot satisfy production visual completion, premium high-fidelity rendering, owner visual acceptance, public-demo readiness, or release-candidate readiness.

Frontier-tech leverage and fail-closed layering are specified in `docs/research/FRONTIER_TECH_LEVERAGE.md`, `docs/decisions/0002-high-fidelity-production-target.md`, `docs/decisions/0003-truth-vs-presentation-layering.md`, and `docs/decisions/0004-renderer-and-asset-pipeline.md`. Those files classify MotionBricks-style presentation motion, external solvers, renderer targets, asset interchange, and facial/generative tools as offline research/authoring or runtime presentation unless a separate deterministic truth ADR promotes them.

A production renderer/engine path must provide, at minimum:

- continuous native player-facing render loop;
- PBR or equivalently convincing material response;
- high-detail human/creature fighters with credible anatomy, deformation, face/head/hand silhouettes where visible;
- skeletal/skinned meshes with cosmetic-only bones separated from truth joints;
- layered armor, cloth, leather, mail, plate, straps, buckles, seams, dents, scratches, dirt, blood, wetness, and wear;
- readable weapon edge/blunt/pierce/hook geometry;
- high-fidelity OATHYARD verdict-ring arena and training arena;
- dynamic lighting, shadowing, atmosphere/fog/dust/weather, reflections/probes or documented GI approximation;
- damage/wear masks and material IDs;
- animation/presentation driven only by hashed truth poses/events;
- deterministic capture tooling for menu, selection, planning, contact, consequence, replay, fight-film, settings, asset closeups, and performance overlay;
- performance instrumentation for simulation step time, render frame time, memory, loading, and package size;
- truth boundary proof that renderer/UI/audio/VFX/camera never mutate authoritative gameplay state.

If Unreal Engine source, another licensed engine, Vulkan/DX12/GL, SDL/GLFW/winit, or any other backend is adopted, a follow-up renderer/engine ADR must record license, dependency footprint, platform target, build/package impact, capture method, input/audio implications, deterministic truth boundary, and measured spike results before it is treated as implementation substrate.

## Current Baseline Evidence

Fresh baseline run:

- `artifacts/baseline/20260629T164832Z/baseline_summary.txt`
- `./tools/build.sh`: passed
- `./tools/test.sh`: passed
- `cargo build --locked`: passed
- `cargo test --locked`: passed
- `./tools/verify.sh`: passed
- final replay hash: `f17c8f76b9dfae86`
- package SHA-256: `36f664eae58ad3ec47f2229e8f207b2697cd2f4b16f9626e3a254ede063965a5  oathyard-linux-x86_64.tar`

Current local graphics/tool evidence:

- Rust/Cargo: `rustc 1.96.0`, `cargo 1.96.0`.
- C/C++ tools present: GCC 16.1.1, Clang 22.1.8, CMake 4.3.2, Ninja 1.13.0, GNU Make 4.4.1.
- Native surface metadata present: `x11`, `wayland-client`, `egl`, `gl`, `xrandr`, `xi`, `xcursor`, `xkbcommon` pkg-config metadata.
- Vulkan runtime present: Vulkan instance 1.4.341, NVIDIA GeForce RTX 5090 visible, AMD integrated GPU visible; Vulkan pkg-config metadata unavailable.
- SDL2/GLFW pkg-config metadata unavailable.
- Blender binary exists but fails startup with `undefined symbol: _ZTVN17MaterialX_v1_39_46OutputE`.
- `gltf-validator`, `gltfpack`, `toktx`, `ktx`, `basisu`, and `sox` are unavailable.
- Image/audio tooling present: ImageMagick 7.1.2-13, FFmpeg 8.1.2, `aplay`, `pw-play`, `paplay`.

Current asset/runtime evidence:

- 22 runtime assets: 6 fighters, 8 weapons, 6 armor/loadout families, 2 arenas.
- Asset budget: 292 vertices, 492 triangles, 22 materials, 22 primitives.
- Production target requires at least 8 weapon families; the numeric family floor is now met by adding the repo-owned billhook polearm/hook profile, but production asset fidelity remains far below target.
- Current local glTF assets have nonzero Z depth and pass local structural validation, but they are low-poly text-generated evidence, not production sculpt/retopo/PBR/skinned assets.

Current pixel-level visual audit evidence:

- `artifacts/baseline/20260629T164832Z/visual_inspection/roster_gemma4_visual_audit.txt`
- `artifacts/baseline/20260629T164832Z/visual_inspection/combat_gemma4_visual_audit.txt`
- `artifacts/baseline/20260629T164832Z/visual_inspection/game_flow_gemma4_visual_audit.txt`
- `artifacts/baseline/20260629T164832Z/visual_inspection/native_combat_3d_third_person_gemma4_visual_audit.txt`
- `artifacts/baseline/20260629T164832Z/visual_inspection/native_roster_showcase_01_saltreach_duelist_gemma4_visual_audit.txt`

Those audits inspected current local visual substitutes and rejected them as prototype/placeholder-level: primitive geometry, flat colors, no production anatomy, no PBR texture detail, minimal or nonexistent lighting, and no high-fidelity arena/material richness.

## Acceptance Gates

High-fidelity visual completion remains false until current-run evidence includes:

- 1920x1080 captures for main menu, settings/accessibility, fighter select, loadout select, OATHYARD establishing shot, each fighter closeup, each armor family closeup, each weapon family closeup, planning timeline, pre-contact frame, contact frame, armor/material damage frame, injury/capability consequence frame, fight-film camera shot, training arena, and performance/debug overlay;
- 2560x1440 captures where hardware/toolchain supports it;
- source-backed production assets with source files, provenance, license status, runtime exports, manifest entries, validation, previews, and in-engine load tests;
- `artifacts/visual_review/latest/visual_benchmark_report.md` comparing captures against silhouette readability, material richness, armor/weapon detail, anatomy/skin/cloth quality, lighting/atmosphere, animation pose credibility, contact readability, UI legibility, combat readability, and originality/no-copying;
- actual image inspection by a vision-capable reviewer or owner, not metadata-only checks;
- owner visual acceptance recorded separately.

## Consequences

- Existing first-slice/bootstrap docs are historical baseline only when they conflict with this target.
- Current local package gates may pass while high-fidelity production completion remains false.
- Blocked native-renderer status silhouettes, debug renders, cubes, capsules, low-poly text-generated meshes, and metadata-only outputs cannot be called high fidelity.
- Production renderer, production assets, DCC/glTF validation, animation, audio/VFX, UI, and packaging must continue to preserve deterministic truth isolation and false external readiness flags until their gates are actually complete.
- Public-demo-ready, release-candidate-ready, owner-final-accepted, legal clearance, trademark clearance, and store readiness remain false.
