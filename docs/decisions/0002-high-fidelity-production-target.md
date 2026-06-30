# 0002: High-Fidelity Production Target - Frontier Research Amendment

Status: Accepted as target amendment; not completion evidence.
Date: 2026-06-29
Related canonical ADR: `docs/decisions/0007-high-fidelity-production-target.md`
Related research register: `docs/research/FRONTIER_TECH_LEVERAGE.md`

## Context

OATHYARD currently has deterministic truth, replay verification, source-backed low-poly local 3D runtime assets, raw X11/XWayland and software-PPM evidence, and package/audit tooling. The uploaded and repo-local PPM captures are useful debug proof, but they are not production visual fidelity and must not be described as Elden Ring / For Honor-class presentation.

The frontier research pass adds a sharper production target:

- high-fidelity native-PC 3D presentation;
- production asset source/interchange/runtime pipeline;
- MotionBricks-inspired presentation motion without truth authority;
- external simulation tools as offline references only;
- fail-closed visual benchmark and capture gates.

2026-06-30 update: the replacement-batch/Rodin-style turntable/contact-sheet evidence is promoted only to `V0.5 candidate_asset_preview`. It improves geometry and silhouette visibility over the earlier PPM/SVG/debug proof, but it remains candidate evidence until provenance, license/export terms, source/DCC files, material/rig/contact profiles, native-engine loading, 1920x1080 gameplay captures, benchmark review, and owner visual acceptance are complete.

Current toolchain inspection supersedes the older Blender blocker recorded in earlier snapshots: `/home/vdubrov/.local/bin/blender` reports Blender 4.3.2 successfully. `/usr/bin/godot`, USD tools, `assimp`, `gltf-transform`, shader tools, ImageMagick, FFmpeg, Vulkan, and an RTX 5090 are available. Unreal is not installed. glTF Validator and KTX/Basis texture-compressor tools remain unavailable locally.

## Decision

OATHYARD targets a complete high-fidelity native-PC 3D emergent melee simulation game. Current raw X11, PPM, SVG, low-poly glTF, software-raster, and diagnostic contact-sheet artifacts remain `Tier 0 / debug-local verification` until a production renderer and asset pipeline produce 1920x1080+ current-run captures with production assets and owner visual acceptance.

The target quality bar may be benchmarked against Elden Ring and For Honor for fidelity/readability only. OATHYARD must remain original and must not copy their assets, silhouettes, factions, UI, animations, lore, music, textures, names, or proprietary mechanics.

## Required target capabilities

A production high-fidelity path must provide:

- continuous player-facing native 3D render loop or separately approved engine integration;
- high-detail skinned fighters with credible anatomy and deformation;
- layered armor/cloth/leather/mail/plate with readable straps, buckles, seams, dents, dirt, blood, wetness, and wear;
- high-quality weapons with readable edge, blunt, pierce, hook, grip, mass, and reach features;
- high-fidelity OATHYARD verdict-ring arena and training arena;
- PBR or equivalently convincing material response;
- dynamic lights, cast/contact shadows, GI/reflection solution or documented approximation, atmosphere/fog/dust/weather, and production camera language;
- animation, VFX, audio, UI, fight-film, and camera driven only by hashed truth events/replay traces;
- deterministic capture tooling for all required product states at 1920x1080 minimum;
- performance instrumentation separated from truth timing;
- visual benchmark report and owner visual acceptance packet.

## Explicit non-evidence

These artifacts can prove dataflow, determinism, or debugging only. They cannot satisfy high-fidelity product presentation:

- PPM line art;
- SVG timelines/contact sheets;
- debug overlays/readout labels;
- wireframes;
- cubes, capsules, primitive silhouettes;
- untextured or flat-color low-poly meshes;
- software-raster integer-depth previews;
- upscaled 960x540 or 1280x720 debug captures;
- screenshots without loaded production assets;
- automated metadata-only checks.

## Frontier leverage policy

Frontier systems are useful only when placed around the deterministic core:

- MotionBricks-style systems become `PresentationBricks`, a presentation layer only.
- Warp, Isaac Lab, Newton, MJWarp, PhysX, Chrono, FEM/SPH/DEM/MPM/cloth/deformable solvers are offline research/reference tools unless separately promoted by a deterministic-truth ADR.
- Nanite/Lumen/RTX/path-tracing/upscaling are renderer target references or runtime-presentation candidates, not truth mechanisms.
- glTF/GLB is a runtime presentation asset delivery direction.
- OpenUSD is a source/interchange authoring direction.
- Audio2Face-3D is facial/cinematic presentation only.
- Generative 3D tools are concept/blockout accelerators only until source/provenance/art/validation gates pass.

## Acceptance checks

Current target acceptance requires all of these before any high-fidelity completion claim:

- `./tools/research_audit.sh` passes.
- `./tools/presentation_truth_isolation.sh` proves presentation toggles do not alter replay JSON, trace JSON, final hash, contact packets, cost breakdowns, capability deltas, or end conditions.
- `./tools/build_assets.sh`, `./tools/validate_assets.sh`, and `./tools/render_asset_previews.sh` pass for production assets, not only local low-poly regression assets.
- `./tools/capture_high_fidelity_screens.sh` produces complete 1920x1080+ capture coverage and exits 0.
- `./tools/visual_benchmark.sh` produces `artifacts/visual_review/latest/visual_benchmark_report.md` and exits 0.
- `./tools/sim_reference_compare.sh` records any external reference solver use without mutating truth.
- `./tools/ai_planner_audit.sh` proves AI emits legal planned actions/directional influence only.
- Owner visual acceptance is separately recorded.

## Current status

Current status is deliberately fail-closed:

```text
current_fidelity_tier: V0.5 candidate_asset_preview plus Tier 0 debug-local verification
production_renderer_complete: false
owner_visual_acceptance: false
public_demo_ready: false
release_candidate_ready: false
```

The target is accepted. Completion is not accepted.
