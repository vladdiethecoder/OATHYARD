# OATHYARD Frontier Autonomous Asset Production Plan

Status date: 2026-06-30
Scope: autonomous production pipeline design and first executable Blender lane. This is not production-art acceptance.

## Target bar

OATHYARD's target is a dark-fantasy judicial duel presentation with:

- For Honor-class combat readability: instant weapon class recognition, clean attack silhouettes, visible grip frames, contact features, armor coverage gaps, stance/body read, and camera-safe duel spacing.
- Elden Ring-class atmosphere: worn oath-stone, cold metal, weathered leather/cloth/mail/plate, restrained legal/faction accents, accumulated dirt/blood/rain/ash, fog/weather hooks, and high-contrast ritual arena composition.
- OATHYARD truth isolation: all art, renderer, animation, DCC, Rodin, Unreal, Godot, Blender, USD, glTF, VFX, audio, and ML systems consume replay/truth outputs only after authoritative hashes. They never decide contacts, injuries, capability deltas, costs, end states, or replay hashes.

"Perfect" is treated operationally as an iterative fail-closed gate: no generated or authored output is final until it survives every machine gate, native visual audit, and explicit owner/human visual acceptance. Agents may create and iterate candidates autonomously; they may not declare owner acceptance.

## Current evidence baseline

Inspected sources:

- `docs/asset_pipeline/ASSET_PIPELINE.md`: production assets require source/provenance/license, DCC/interchange/runtime export, manifest/hash coverage, previews, in-engine screenshot, collision/contact/material metadata, and validation. Fighters require rig/skin/truth-joint mapping/contact regions/damage masks/armor sockets. Weapons require grip frames/contact geometry/mass/MOI/material durability. Armor requires separate pieces/coverage gaps/straps/material layers/deformation states. Arenas require verdict ring, witness/oath stones, worn material zones, collision/footing, lighting/camera/weather/atmosphere hooks.
- `docs/research/FRONTIER_TECH_LEVERAGE.md`: generative 3D is `offline_research_authoring` only; generated meshes are concept/blockout candidates until source/provenance/license/human art pass/validation/in-engine evidence pass. Runtime high fidelity must be native 3D with PBR/equivalent materials, lights, shadows, atmosphere, cameras, and truth isolation.
- Hyper3D/Rodin docs: Rodin Gen-2.5 API supports image-to-3D and text-to-3D, up to 5 images, GLB/USDZ/FBX/OBJ/STL, PBR/Shaded/All materials, Raw/Quad mesh, quality/tier controls, `bbox_condition`, `TAPose`, `preview_render`, `hd_texture`, `texture_delight`, `texture_mode`, and async status/download. API requires bearer key; free test key exists in cloned Deemos skill docs, production/business use requires a real account/key/credits. Download URLs expire quickly.
- Fab listing `05142f43-5da0-4dc3-bb4d-502103953262`: `Rodin - Demo Plugin`, UE plugin, text/image-to-3D API, runtime import, 4 Blueprints, 1 C++ class, requires API key, UE-supported mesh/material preview/import. Listing claims Windows dev/target support and UE 5.1 and 5.3-5.7. It is demo/integration evidence, not OATHYARD production acceptance.
- Deemos addon docs: Blender addon requires Blender >=4.0 and Google Chrome >=116; Godot addon requires Godot >=4.4 and Chrome >=116; Unreal addon docs require UE 5.1-5.6; Maya addon requires Maya 2022-2026; all Rodin GUI addons rely on Rodin web/floating-window workflow. For autonomous batch work, API scripts are primary; GUI addons are optional import/inspection surfaces.

Local tool state:

- Blender usable via `/home/vdubrov/.local/bin/blender` -> `/home/vdubrov/.local/opt/blender-4.3.2-linux-x64/blender`; system `/usr/bin/blender` remains broken by a MaterialX symbol error and must not be hardcoded.
- Rodin Blender addon `a_Rodin` installed under `/home/vdubrov/.config/blender/4.3/scripts/addons/a_Rodin`; import and enable smoke passed.
- Google Chrome present: `/usr/bin/google-chrome`, version 148.x.
- OpenUSD tools present: `/usr/bin/usdcat`, `/usr/bin/usdview`.
- Assimp present: `/usr/bin/assimp`.
- GIMP/Krita/MeshLab/Godot packages were requested through `dnf`; verify commands must check binaries before depending on them.
- Deemos repos cloned under `external/rodin/` for local reference: `rodin3d-skills`, `rodin3d-bang-skills`, `blender-mcp-rodin-integration`, `RodinBridge_UnrealPlugin`.
- UnrealEditor source build is still running separately; do not stop it. Use Blender/Godot/current native debug renderer until UE binary exists.

## Asset inventory target

Minimum first production lane, matching existing OATHYARD asset sources/model-candidate inventory:

- Fighters: 6 (`saltreach_duelist`, `oathyard_writ`, `chainbreaker`, `reed_sentinel`, `gate_shield`, `bruiser_oath`).
- Weapons: 8 (`curved_sword`, `longsword`, `bearded_axe`, `ash_spear`, `round_shield`, `iron_maul`, `arming_sword`, `billhook`).
- Armor sets: 6 (`bruiser_padded_plate`, `fencer_light`, `lamellar`, `heavy_plate`, `mail_hauberk`, `gambeson`).
- Arenas: 2 (`training_yard`, `oathyard_verdict_ring`).

Each asset must have a production source packet under a new non-canonical candidate lane, not overwrite `assets/gltf/` or existing regression assets until acceptance passes.

Recommended candidate layout:

```text
assets_src/production_candidates/<run_id>/<category>/<asset_id>/
  source.blend
  source.usda or source.usdc when available
  source_manifest.json
  art_brief.md
  license_provenance.json
  contact_material_contract.json
assets/production_candidates/<run_id>/<category>/<asset_id>/
  <asset_id>.glb
  textures/*.png
  preview/*.png
  visual_audit.json
artifacts/production_candidates/<run_id>/<category>/<asset_id>/
  build.log
  blender_report.json
  validation_report.md
  native_capture_manifest.json
```

## Fail-closed production DAG

Every asset follows the same gate sequence. A candidate may move forward only when the prior gate produces machine-readable evidence.

1. Canon and mechanical contract
   - Read `GAME_CANON.md`, `DEMO_SCOPE.md`, `ACCEPTANCE_MAP.md`, `ASSET_PIPELINE.md`, source `.oysrc`, and existing model-source JSON.
   - Write an asset contract: category, dimensions, silhouette landmarks, contact surfaces, material families, deformation/damage hooks, rig/truth-joint hooks, collision/footing/camera hooks, and non-authoritative truth boundary.

2. Reference/control package
   - Generate or author front/side/back/detail reference images and dimension cards.
   - Prefer multiview/control images over text-only prompting.
   - Store prompt, seed, model/API version, source image hashes, and license/provenance.

3. AI/blockout generation lane
   - Primary scriptable API: Hyper3D Rodin Gen-2.5 via `external/rodin/rodin3d-skills`.
   - Recommended draft settings: `Gen-2.5-Medium`/`High`, `geometry_file_format=glb`, `material=PBR`, `mesh_mode=Raw`, `texture_mode=high`, `preview_render=true`, `bbox_condition` from contract, seed recorded.
   - Extreme-High/HighPack/paid-credit use requires explicit credential/credit approval. Free key/test-key generations remain prototype evidence only.
   - Rodin GUI addons are optional import/inspection bridges, not the autonomous batch controller.
   - If Rodin unavailable, use Blender procedural/authored generation directly.

4. Blender DCC refinement lane
   - Import draft GLB/FBX/OBJ or generate source geometry from contract.
   - Retopo/decimate to category budgets; preserve silhouette landmarks.
   - UV unwrap and assign material zones.
   - Add high-frequency details: bevels, fullers, rivets, straps, edge wear, chipped stone, cloth seams, mail patterns, scuffs, blood/wetness masks.
   - Bake or author PBR channels: base color, normal, ORM, curvature/AO/wear, material IDs.
   - Save `.blend` as source-of-truth candidate.

5. Physical-fidelity metadata lane
   - Weapons: grip frames, edge/point/blunt/hook regions, mass distribution, MOI, durability/material state, contact geometry.
   - Armor: coverage/gap maps, straps/fasteners, material layers, deformation/damage states, mass/inertia, collision/contact regions.
   - Fighters: high-fidelity mesh, rig, skin weights, canonical 16 truth-joint map, cosmetic bones separated from truth, anatomy/contact regions, damage masks, armor attachment points, no-clipping action-pose evidence.
   - Arenas: verdict ring, witness/oath stones, weapon staging, worn stone/scuffs/cuts/blood/maintenance props, collision/footing, lighting/camera/weather/atmosphere hooks.

6. Runtime export lane
   - Export GLB/glTF 2.0 with PBR material bindings.
   - Hash all source, export, texture, manifest, and preview artifacts.
   - Do not mutate `assets/gltf/` canonical regression outputs until production candidate acceptance is explicitly approved.

7. Native visual evidence lane
   - Render preview/contact sheets in Blender immediately.
   - When available, render through Godot or Unreal/native renderer as runtime-presentation evidence.
   - Use OATHYARD native capture tools when renderer integration exists.
   - Required perspectives: full-body/whole-asset, contact/action pose, material closeup, silhouette thumbnail, gameplay camera, and arena establishing/gameplay/contact views.

8. Hostile visual audit lane
   - Inspect actual PNG/contact-sheet pixels with vision tooling.
   - Reject if silhouette/category is ambiguous, material reads flat/plastic, scale/contact features are unclear, or atmosphere is generic/borrowed.
   - Machine metrics do not override visual rejection.

9. Acceptance lane
   - Owner/human visual acceptance is the final gate.
   - Agent may report candidates, evidence, blockers, and iteration deltas only.
   - Readiness/public/release flags stay false unless current-run acceptance artifacts prove them.

## Category-specific quality bars

### Weapons

- Silhouette readable at 128px thumbnail and gameplay camera distance.
- Contact families visibly separated: edge, point, blunt, hook, guard, shaft, grip.
- Grip frames aligned to truth/planner semantics; two-hand vs one-hand visible.
- PBR: steel scratches, edge bevels, roughness variation, leather wrap strain, wood grain, ash/iron corrosion.
- Physics metadata: length, mass distribution, MOI, contact regions, durability state.

### Armor

- Layered pieces, not body-painted texture.
- Coverage/gap map readable: plate/mail/leather/cloth gaps matter for contact/injury presentation.
- Straps, buckles, fasteners, seams, deformation/damage variants.
- Material separation at distance: mail vs plate vs gambeson vs leather.
- No truth mutation: armor visuals consume truth contact/capability output; they do not define gameplay armor points.

### Fighters

- Truth skeleton map plus render skeleton separation.
- Anatomy/contact regions and damage masks.
- Armor sockets and weapon attachment points.
- Action-pose no-clipping proof for guard, attack, bind, stumble, fall.
- Distinct tradition identity without copying protected reference IP.

### Arenas

- Ritual/legal duel composition: verdict ring, oath/witness stone, witness positions, entry breaks, weapon staging.
- Gameplay readability: footing zones, arena boundary, camera anchors, combat-safe contrast behind fighters.
- Atmosphere: weathered stone, ash/fog/rain hooks, old cuts/scuffs/blood wash, maintenance props.
- Collision/footing metadata separated from visual-only detail.

## Automation architecture

- `tools/blender_autonomous_pipeline.py`: headless Blender script for contract-driven candidate generation/import, material assignment, preview render, GLB export, `.blend` save, and manifest output.
- `tools/rodin_asset_batch.py` (future): script around `external/rodin/rodin3d-skills` that submits/polls/downloads immediately, records prompt/seed/tier/key-type without logging secrets, and writes fail-closed metadata.
- `tools/production_asset_candidate_audit.py` (future): validates candidate directory shape, hashes, PBR channels, source/provenance, truth-boundary booleans, and category metadata.
- Kanban workers split by category: weapons first vertical slice, then armor/fighters/arenas/material library, then renderer import/capture.
- Cron hygiene remains non-destructive; production candidate artifacts are evidence and must be archived, not deleted.

## Renderer path while Unreal builds

- Immediate: Blender viewport/render/contact sheets for DCC evidence.
- Near-term fallback: Godot if installed and import/capture smoke passes, but Godot evidence is still runtime-presentation candidate evidence only.
- UE path: RodinBridge Unreal plugin is cloned but docs/listing target UE 5.1-5.6/5.7 while local source is UE 5.8. Plugin compatibility is unverified until UnrealEditor binary exists and compile/import smoke passes.
- OATHYARD native debug renderer remains insufficient for high-fidelity claims until production renderer/asset import is implemented and visual benchmark passes.

## Blockers / external requirements

- Production Rodin automation needs a valid Hyper3D API key/business subscription/credits. Free test key is only prototype-lane evidence.
- Paid/proprietary DCCs (Maya, 3ds Max, Cinema4D, Houdini, Substance, ZBrush) cannot be installed or activated without licenses/account installers. Their Deemos addon docs are learned as integration targets, not local proof.
- UnrealEditor is still compiling; UE import/capture proof is blocked until the binary exists.
- Final owner/human visual acceptance cannot be autonomously granted.

## First executable vertical slice

Use `longsword` because it has a clear existing contract: straight double edge, 1220 mm, edge/pierce/cross contact, two-hand grip, steel read.

Acceptance for the slice:

- Source `.blend` created from contract.
- GLB exported under `assets/production_candidates/<run>/weapons/longsword/`.
- Preview PNG rendered.
- Manifest records source, runtime export, hashes, materials, truth boundary, not-claimed fields, and blockers.
- Existing regression asset lane still builds/validates.
- No production/readiness/owner acceptance flags are promoted.
