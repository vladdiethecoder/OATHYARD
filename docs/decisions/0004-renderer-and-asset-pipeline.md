# 0004: Renderer and Asset Pipeline Target

Status: Accepted target; current implementation remains debug/local verification.
Date: 2026-06-29
Related: `docs/asset_pipeline/ASSET_PIPELINE.md`, `docs/research/FRONTIER_TECH_LEVERAGE.md`, `docs/decisions/0002-high-fidelity-production-target.md`

## Context

The current renderer path is raw X11/XWayland drawing plus software PPM rasterization. It is useful as deterministic local evidence, but it is not a production renderer and it does not prove premium 3D product presentation.

The asset pipeline currently generates repo-owned low-poly local glTF/runtime assets from deterministic text sources. That is a good regression and determinism lane, but it is not a production art pipeline.

## Decision

OATHYARD needs two separate but connected production lanes:

1. A high-fidelity runtime presentation renderer.
2. A source-to-runtime asset pipeline with provenance, validation, previews, in-engine evidence, and fail-closed acceptance flags.

Neither lane may mutate authoritative truth.

## Renderer target

The renderer or approved engine integration must support:

- continuous native player-facing 3D loop;
- 1920x1080 minimum capture path, 2560x1440 where hardware permits;
- skinned high-fidelity fighters;
- layered armor/clothing;
- high-quality weapons;
- OATHYARD verdict-ring and training arenas;
- PBR or equivalently convincing material response;
- normal/roughness/metallic/AO/detail/damage/blood/dirt/wear masks or documented equivalents;
- dynamic lights;
- shadows;
- GI/reflection solution or documented approximation;
- atmosphere/fog/dust/weather;
- cinematic replay/fight-film camera shots;
- performance instrumentation for render frame time, load time, memory, and package delta;
- deterministic capture manifests with file hashes;
- strict truth isolation.

Unreal Nanite/Lumen-class features are the quality reference, not an adopted dependency. Unreal, Unity, Godot, proprietary vendor SDKs, RTX/DLSS/FSR/XeSS, path tracing, or neural rendering require a separate backend/license/dependency ADR before use.

## Asset pipeline target

Use OpenUSD or equivalent for source/interchange where feasible. Use glTF/GLB or equivalent for runtime skinned mesh/material delivery where feasible.

Every production asset must have:

- source file;
- provenance/license record;
- author/toolchain/version record;
- runtime export;
- manifest entry;
- source and runtime hashes;
- preview render;
- in-engine screenshot;
- collision/contact profile;
- material/physics profile where relevant;
- validation result;
- readiness flags remaining false unless the correct acceptance authority passed.

### Fighter assets

Fighter assets require:

- high-fidelity mesh;
- rig;
- skin weights;
- canonical truth-joint mapping;
- cosmetic-only bones separated from truth;
- anatomy/contact regions;
- damage masks;
- armor attachment points;
- closeup render;
- gameplay screenshot;
- no-clipping action-pose evidence.

### Armor assets

Armor assets require:

- separate pieces;
- coverage/gap maps;
- straps/fasteners;
- material layers;
- deformation/damage states;
- mass/inertia profile;
- collision/contact regions;
- closeup preview;
- gameplay evidence.

### Weapon assets

Weapon assets require:

- grip frames;
- edge/point/blunt/hook features;
- mass distribution;
- moment-of-inertia profile;
- contact geometry;
- durability/material state;
- preview evidence;
- gameplay evidence.

### Arena assets

OATHYARD arena assets require:

- high-fidelity verdict ring;
- witness positions;
- oath/witness stone;
- weapon staging;
- worn stone, scuffs, cuts, blood wash, and maintenance props;
- collision/footing metadata;
- lighting/camera anchors;
- weather/atmosphere hooks;
- establishing shot;
- gameplay screenshot.

## Runtime manifest requirements

Runtime presentation manifests must record:

- renderer/backend id;
- asset manifest hash;
- source asset hashes;
- runtime export hashes;
- replay path and final hash where applicable;
- capture resolution;
- camera mode/state;
- capture command;
- presentation layer toggles;
- `truth_mutation:false`;
- `production_renderer_complete:false` until the production gate passes;
- `owner_visual_acceptance:false` until owner review passes;
- `public_demo_ready:false` and `release_candidate_ready:false` until external gates pass.

## Visual benchmark gate

`artifacts/visual_review/latest/visual_benchmark_report.md` must compare current captures against:

- silhouette readability;
- anatomy quality;
- armor/cloth/weapon detail;
- material richness;
- lighting/atmosphere;
- contact readability;
- injury/capability readability;
- UI readability;
- camera composition;
- originality/no copying;
- performance;
- truth boundary.

The report may say `candidate evidence package`. It may not say owner accepted, public-demo ready, release-candidate ready, Elden Ring quality achieved, or For Honor quality achieved unless those gates actually passed.

## Verification

Required commands:

```sh
./tools/build_assets.sh
./tools/validate_assets.sh
./tools/render_asset_previews.sh
./tools/capture_high_fidelity_screens.sh
./tools/visual_benchmark.sh
./tools/presentation_truth_isolation.sh
./tools/audit_readiness.sh . artifacts/readiness/source
```

`capture_high_fidelity_screens.sh` and `visual_benchmark.sh` are intentionally fail-closed until production renderer/assets/captures exist. Their failure is current evidence, not a reason to weaken gates.
