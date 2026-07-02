# OATHYARD Asset Pipeline

## Policy

All production assets must be repo-owned, source-backed, provenance-tagged, and regenerable from `assets_src/`.

Copied, scraped, unlicensed, unverifiable, or placeholder production assets are forbidden. Debug-only primitives must be labeled debug-only and excluded from production manifests.

Production runtime meshes must be 3D. Flat glTF geometry with no Z depth is a validation failure, because the native combat renderer must project actual source-backed 3D runtime geometry instead of flat silhouettes.

## Directories

- `assets_src/`: source meshes, rigs, materials, audio specs, VFX specs, icons, and provenance.
- `assets/`: generated runtime assets and previews.
- `content/`: deterministic gameplay tables and manifests.
- `artifacts/`: generated validation reports, screenshots/renders, packages, traces, and logs.

## Build

```sh
./tools/build_assets.sh
```

The builder reads source assets and content manifests, then writes runtime manifests, mesh summaries, deterministic extruded 3D runtime `.gltf` files, previews, and provenance reports.

## Validate

```sh
./tools/validate_assets.sh
```

Validation checks:

- provenance for every production asset
- canonical truth-joint rig mapping for fighters
- physical/contact/material profiles for weapons and armor
- arena collision/navigation/camera/lighting metadata
- generated glTF files with embedded buffers, OATHYARD provenance extras, and non-authoritative truth metadata
- nonzero Z depth for every production runtime glTF asset
- generated previews
- no production placeholder markers

Additional visual/3D gates:

```sh
./tools/render_asset_previews.sh artifacts/asset_previews/latest
./tools/asset_visual_atlas.sh artifacts/asset_atlas/latest
./tools/audit_3d_runtime.sh artifacts/runtime_3d/latest assets/runtime_manifest.json artifacts/native_combat/verify/native_combat_render_manifest.json
```

The asset preview renderer emits source-backed local preview manifests/reports while keeping high-fidelity/owner-readiness claims false; accepted visual preview evidence requires native 3D renderer captures with metadata. The asset atlas checks source/runtime/preview/provenance coverage and rejects flat glTF geometry. The runtime 3D audit checks every runtime glTF asset for nonzero Z depth and confirms the native combat projection uses Z depth after truth hashes.

## Production source-to-runtime target

The current pipeline is a deterministic local regression lane, not production art completion. The high-fidelity production pipeline is defined by `docs/decisions/0004-renderer-and-asset-pipeline.md` and `docs/research/FRONTIER_TECH_LEVERAGE.md`.

Target source/interchange direction:

- OpenUSD or equivalent for production source/interchange when tooling is available.
- glTF/GLB or equivalent for runtime skinned mesh/material/animation delivery when tooling is available.
- Manifest and hash coverage for every source layer, runtime export, texture, material map, rig, and capture.

Every production asset must carry:

- source file;
- provenance/license record;
- author/toolchain/version record;
- runtime export;
- manifest entry;
- preview render;
- in-engine screenshot;
- collision/contact profile;
- material/physics profile where relevant;
- validation result.

Fighter assets additionally require high-fidelity mesh, rig, skin weights, canonical truth-joint mapping, cosmetic-only bones separated from truth, anatomy/contact regions, damage masks, armor attachment points, closeup render, gameplay screenshot, and no-clipping action-pose evidence.

Armor assets additionally require separate pieces, coverage/gap maps, straps/fasteners, material layers, deformation/damage states, mass/inertia profile, collision/contact regions, preview evidence, and gameplay evidence.

Weapon assets additionally require grip frames, edge/point/blunt/hook features, mass distribution, moment-of-inertia profile, contact geometry, durability/material state, preview evidence, and gameplay evidence.

Arena assets additionally require high-fidelity verdict ring, witness positions, oath/witness stone, weapon staging, worn stone/scuffs/cuts/blood wash/maintenance props, collision/footing metadata, lighting/camera anchors, weather/atmosphere hooks, preview evidence, and gameplay screenshots.

Do not promote generated, external, placeholder, or low-poly debug assets into production manifests without source/provenance/license, human art pass, validation, and in-engine visual evidence. Generative 3D tools are concept/blockout accelerators only until those gates pass.

## DCC Status

Blender is currently unavailable because it fails at startup with a MaterialX symbol lookup error. Until fixed, source assets are authored in deterministic text mesh/spec formats and validated by local scripts.

The local pipeline exports structurally validated 3D glTF 2.0 JSON files under `assets/gltf/`. This is not a claim that Blender round-trip, GLB packaging, GLB runtime packaging, OpenUSD source interchange, or the external Khronos validator has passed.
