# Production Asset Acceptance

Status: Active fail-closed checklist.
Date: 2026-06-30

## Universal production asset requirements

Every production asset must have:

- source file;
- provenance/license metadata;
- runtime export;
- manifest entry;
- content hash;
- preview render;
- in-engine/native screenshot;
- validation result;
- material profile;
- physics/contact profile where relevant;
- package inclusion test;
- truth-boundary classification.

Candidate-only, placeholder, unlicensed, unrigged, untextured, non-loaded, or debug-only assets must not be listed as production-ready.

## Fighter assets

Required:

- high-fidelity body mesh with closeup-capable head/face/hand quality;
- rig and skin weights;
- canonical truth-joint mapping for the 16 truth joints plus grip frames;
- cosmetic-only bones separated from truth;
- animation/retarget test;
- anatomy/contact regions;
- damage masks;
- armor attachment sockets;
- material maps;
- scale/orientation validation;
- first-person and third-person visibility checks;
- in-engine closeup screenshot;
- gameplay screenshot.

## Weapon assets

Required:

- grip frames;
- edge/point/blunt/hook feature markers;
- mass distribution;
- moment-of-inertia profile;
- collision/contact geometry;
- material/durability profile;
- UV/material maps;
- multiple-angle preview renders;
- in-engine loadout screenshot;
- gameplay contact screenshot.

## Armor assets

Required:

- separate mesh pieces;
- coverage/gap maps;
- straps/fasteners;
- attachment points;
- material layers;
- deformation/damage states;
- mass/inertia profile;
- collision/contact regions;
- clipping checks against fighters;
- preview closeups;
- in-game evidence.

## Arena assets

Required:

- judicial duel ground;
- verdict ring;
- witness positions;
- oath/witness stone;
- weapon staging;
- worn stone, cuts, scuffs, blood wash;
- maintenance props, banners/markers/lore props;
- lighting/camera anchors;
- collision and footing metadata;
- atmosphere hooks for fog/dust/wetness/weather;
- establishing, gameplay, contact, replay, and fight-film captures.

## Verification

The acceptance packet must include outputs from:

```sh
./tools/audit_rodin_assets.sh
./tools/validate_assets.sh
./tools/render_asset_previews.sh
./tools/capture_high_fidelity_screens.sh
./tools/presentation_truth_isolation.sh
./tools/visual_benchmark.sh
```

Owner visual acceptance, public-demo readiness, release-candidate readiness, legal clearance, trademark clearance, and store readiness remain false unless separately and explicitly evidenced.
