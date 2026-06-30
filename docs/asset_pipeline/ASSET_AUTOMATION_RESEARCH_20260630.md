# OATHYARD asset automation research intake — 2026-06-30

This file records the delegated research results and the YOMIH-inspired Codex image-generation correction from the current asset-production session. It is evidence/control context only; it does not promote any asset to production, gameplay truth, public-demo readiness, or owner acceptance.

## Canon boundary

- Classification: `offline_research_authoring` / `runtime_presentation` only.
- Truth authoritative: `false`.
- Generated images, Rodin outputs, MotionBricks motions, and DCC/engine imports must not alter replay JSON, contact packets, action legality, injury/capability state, or final hashes unless a separate deterministic truth ADR passes all canon gates.
- Do not copy YOMI/YOMIH names, characters, sprites, UI, exact weapons, or protected identity. The usable inspiration is mechanical: frame-by-frame planning, prediction/hit-frame/parry/initiative clarity, flashy combo construction, extreme role variety, mobility/grappler/zoner/rushdown tension, and modded-style breadth.

## YOMIH source facts used for the addendum

Primary/current sources checked:

- Steam store page for `Your Only Move Is HUSTLE`, app `2212330`, developer/publisher Ivy Sly, released 2023-02-02. Steam describes it as mastering technique, executing flashy combos, outsmarting opponents frame-by-frame, slowing down the clock, fine-tuning fighting style, and a turn-based combat simulator.
- Ivy Sly page `https://ivysly.com/games/your-only-move-is-hustle.html`, dated 2023-02-02. Ivy describes it as an online turn-based fighting game and superpowered fight scene simulator.
- Ivy Sly patch notes `https://ivysly.com/hustle/patchnotes/1.9.0.html`. Relevant mechanical facts: prediction UI, combo-vs-neutral distinction, initiative state, parry timing indicator, hit-frame display, modded character support, Mutant as extreme rushdown/mobility with Juke, Ninja tools including grappling hook/shuriken/caltrops, and Robot mechanical move/control changes.

## Codex image generation outputs

### Original catalog

- Root: `assets_src/reference/concepts/codex_weapon_catalog_20260630T184451Z/`
- Pages: 4
- Vision-audited approximate visible slots: 230
- Manifest: `assets_src/reference/concepts/codex_weapon_catalog_20260630T184451Z/codex_weapon_catalog_manifest.json`
- Audit: `assets_src/reference/concepts/codex_weapon_catalog_20260630T184451Z/audit/visual_audit.md`

### YOMIH-inspired addendum

- Root: `assets_src/reference/concepts/codex_weapon_catalog_yomih_20260630T185753Z/`
- Pages: 3
- Vision-audited approximate visible slots: 358
- Manifest: `assets_src/reference/concepts/codex_weapon_catalog_yomih_20260630T185753Z/codex_weapon_catalog_yomih_manifest.json`
- Audit: `assets_src/reference/concepts/codex_weapon_catalog_yomih_20260630T185753Z/audit/visual_audit.md`

Hostile audit summary:

- Page A: high object count but fails as a move-set catalog without curation; too many dagger/wrist-launcher/claw near-duplicates and not enough visible stance/timing/function separation.
- Page B: good loadout-object breadth but contains pseudo-label/text artifacts, disembodied hand/claw forms, and near-duplicate shuriken/fans/chains/cages/crystals.
- Page C: stronger curated prompt direction but too few visible slots and still mostly static prop shapes.

Next use: crop/curate individual one-object/one-function candidates into a smaller Rodin input set. Do not use full generated sheets as Rodin single-image inputs.

### Physical-scale V2 catalog

Generated after user rejection that prior outputs contained physically impossible/ridiculous weapons and lacked For Honor / Elden Ring-like size breadth.

- Root: `assets_src/reference/concepts/codex_weapon_catalog_physical_scale_20260630T191159Z/`
- Pages: 5
- Vision-audited approximate visible items: 173
- Manifest: `assets_src/reference/concepts/codex_weapon_catalog_physical_scale_20260630T191159Z/codex_weapon_catalog_physical_scale_manifest.json`
- Audit: `assets_src/reference/concepts/codex_weapon_catalog_physical_scale_20260630T191159Z/audit/visual_audit.md`
- Size-band spec: `assets_src/reference/concepts/codex_weapon_catalog_physical_scale_20260630T191159Z/physical_size_band_spec.json`

Hostile audit summary:

- Overall verdict: `pass_as_replacement_reference_pool_fail_as_direct_Rodin_input`.
- Page 1 scale ladder passes as the strongest size-reference sheet: small knives, one-handed weapons, two-handed/heavy weapons, and polearms are visibly different sizes.
- Page 2 light/one-handed passes with curation; it has plausible construction but near-duplicate daggers, axes, hooks, and paired sets.
- Page 3 two-handed/heavy is a conditional fail for heaviest slots: several heavy/colossal heads push cartoon-slab or bad-weight-distribution territory and must be rejected or redrawn.
- Page 4 reach/polearms/control passes as a polearm pool but fails as a size-variety reference because many polearms are normalized to similar cell height.
- Page 5 defensive/hybrid/ritual is conditional fail until curated: pseudo-firearms, fragile ritual heads, and non-weapon props can confuse Rodin.

Next use: curate first Rodin batch from `small_offhand`, `one_handed`, and `hand_and_half_two_handed` bands with clean one-object crops. Deprioritize flexible chains/nets/whips and the heaviest cartoon-slab designs until authored modeling or stronger source references exist.

## MotionBricks compatibility conclusion

Fail-closed classification from delegated research:

- Compatible as-is with Rodin GLB/FBX characters: `false`.
- Categorically incompatible: not proven.
- Compatible only after a custom rig/retarget/export pipeline: `true`.
- High-fidelity OATHYARD melee readiness: `unknown / unproven`.

Primary blockers:

1. Public MotionBricks preview is G1/MuJoCo-focused, not a general game-character GLB/FBX import path.
2. Current public representation uses `G1Skeleton34`, 418-dimensional per-frame features, and MuJoCo qpos/joint trajectories.
3. No primary source showed FBX/glTF animation clip export, Blender support, Unity support, or a released general UE plugin for arbitrary characters.
4. SOMA Retargeter bridges SOMA BVH to Unitree G1 CSV, not Rodin mesh rigs to OATHYARD humanoids.
5. UE5 demos exist in NVIDIA project/paper sources, but public preview does not expose a complete OATHYARD-ready game-engine pipeline.
6. High-fidelity melee combat is unproven: sources show object interaction / sword pickup, not full combat exchanges, weapon contact timing, parries, hit reactions, or weapon/object synchronized output.

Required bridge before use:

```text
Rodin mesh generation/repair
-> clean humanoid rig
-> MotionBricks/SOMA/G1 motion-to-game-skeleton retargeting
-> engine animation clip export
-> contact, grip, root motion, foot sliding, parry/hit-frame, and melee timing validation
-> native OATHYARD presentation-only truth-isolation proof
```

## Rodin / generated asset staging path

Smallest safe integration path for Rodin outputs:

1. Keep raw Rodin downloads under `artifacts/production_candidates/<rodin_run_id>/rodin/<asset_id>/`.
2. Stage reviewed candidates under `assets/production_candidates/<run_id>/weapons/<weapon_id>/`, not under canonical `assets/gltf/` or truth tables.
3. Candidate manifests must include source concept hash, generation prompt, export hash, mesh/material metrics, preview/audit paths, truth-boundary flags, and not-claimed list.
4. Classify as `offline_research_authoring` or `runtime_presentation` until deterministic truth gates exist.
5. Only after DCC cleanup/retopo/UV/PBR/native capture should anything approach `assets_src/production/**` or `assets/production_visual_manifest.json`.

Do not directly copy generated GLB/FBX/glTF into:

- `assets/gltf/`
- `assets/runtime/`
- `content/oathyard_content.manifest`
- gameplay weapon/material/contact/anatomy/capability truth tables

## Rodin web UI no-credit probe constraints

Known safe selectors / controls from artifact audit:

- Upload selector: `#newRodin-upload input[type=file][accept*='jpg']`
- Generate button class: `[class*='newrodin_generateBtn']` / observed `newrodin_generateBtn__cTxM7`
- Direction classes/text: `newrodin_imgBtn__1u9gq`, `newrodin_oripopup__OgHAA`, `newrodin_oriitem__cHPKh`, labels `Front`, `Front Left`, `Front Right`, `Left`, `Right`, `Back`, `Back Left`, `Back Right`, `UP`, `Down`, `Unknown`
- ControlNet text/classes: `Bounding Box ControlNet`, `Voxel ControlNet`, `PointCloud ControlNet`, `ModelPresets_boundingBox__wtrrU`

No-credit probe rules:

1. Upload only separated single-view test images, not a full multiview collage.
2. Never click main Generate, modal Generate, result Confirm, Subscribe, Top-Up, payment, OAuth, or permission UI during selector probing.
3. Treat any `Confirm` as unsafe unless proven to be per-image only.
4. Capture screenshot + DOM after upload, direction popup open/selection, multiview affordance, and ControlNet selection.
5. Stop before any action displaying or consuming `0.5 Credits`.

Known morphing failure: submitting a full character or weapon concept sheet as one undifferentiated image can fuse/morph multiple views. Use separate per-view files and explicit directions, or verified Rodin multiview region tooling.

## Verification commands run for this intake

Focused verification already run on the generated catalogs:

```sh
python3 -m json.tool assets_src/reference/concepts/codex_weapon_catalog_20260630T184451Z/codex_weapon_catalog_manifest.json
python3 -m json.tool assets_src/reference/concepts/codex_weapon_catalog_20260630T184451Z/audit/visual_audit.json
python3 -m json.tool assets_src/reference/concepts/codex_weapon_catalog_yomih_20260630T185753Z/codex_weapon_catalog_yomih_manifest.json
python3 -m json.tool assets_src/reference/concepts/codex_weapon_catalog_yomih_20260630T185753Z/audit/visual_audit.json
python3 - <<'PY'
from pathlib import Path
from PIL import Image
for p in Path('assets_src/reference/concepts').glob('codex_weapon_catalog*/**/*.png'):
    im = Image.open(p)
    im.verify()
print('catalog PNGs verify')
PY
```

Full repo gates remain pending under the active task list.
