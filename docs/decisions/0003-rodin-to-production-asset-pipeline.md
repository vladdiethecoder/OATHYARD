# 0003: Rodin-to-Production Asset Pipeline

Status: Accepted pipeline target; current Rodin/replacement-batch assets remain candidate evidence only.
Date: 2026-07-02

## Context

OATHYARD now has a better visual baseline than the earlier debug proof: the replacement-batch turntable/image-rollup asset evidence shows that higher-detail fighter, armor, weapon, and arena silhouettes are available for review. That evidence is V0.5 candidate evidence. It is not production art, not final visual fidelity, not public-demo readiness, not owner acceptance, and not proof of Elden Ring / For Honor-class presentation.

The current checked-in candidate lane is source-backed procedural/model-candidate evidence under:

- `assets_src/model_candidates/t_73291be5/`
- `assets/model_candidates/t_73291be5/`
- `artifacts/model_candidates/t_73291be5/`
- `assets/presentation_manifest.json`

No local audit has found a completed raw Rodin export directory with Rodin task IDs, download receipts, subscription/plan evidence, terms snapshot, and source prompt/image bundle. Any Rodin or AI-generated asset without those records is `candidate-only / license-pending`.

## Decision

Create a fail-closed source-to-runtime asset pipeline with explicit candidate, source-approved, production-ready, rejected, and debug-only states.

Directory lanes:

```text
assets_src/rodin_candidates/   # raw Rodin/API/DCC candidate receipts, prompts, images, exports, terms snapshot
assets_src/production/         # source-approved production DCC/OpenUSD/Blend sources only
assets_src/rejected/           # rejected/quarantined generated/imported assets and reasons
assets_runtime/                # future cooked/runtime renderer outputs; generated, not truth-authoritative
content/assets/                # asset-index metadata consumed by runtime presentation only
content/loadouts/              # loadout presentation manifests; truth loadout tables remain deterministic
content/materials/             # material profile metadata; presentation-only unless promoted by truth ADR
content/physics_profiles/      # contact/material/footing metadata with truth-boundary classification
artifacts/asset_audit/latest/  # current audit packet
artifacts/asset_previews/latest/ # current candidate/production preview packet
artifacts/visual_review/latest/ # current high-fidelity benchmark/gap packet
```

Candidate assets may be cataloged and previewed. They must not be stored as production assets or marked production-ready until every production gate below passes.

## State machine

| State | Meaning | May enter production manifest? |
| --- | --- | --- |
| `candidate` | Generated/imported/model-candidate evidence with incomplete source/license/validation/renderer evidence. | No. Candidate manifest only. |
| `license-pending` | Candidate/source/runtime evidence exists, but project license/commercial-use/terms/receipt evidence is unresolved. | No. |
| `source-approved` | Source, prompt/image provenance, license/export terms, and IP/originality risk have been reviewed and approved for project use. | Not as runtime production until technical gates pass. |
| `technical-clean` | Machine validation, DCC/interchange import, glTF/topology/UV/normal/tangent/material-channel evidence passes, but product gates remain. | No; this is a facet, not a production grant. |
| `gameplay-profiled` | Rig/contact/truth-boundary profiles pass where applicable, but production capture/package/owner/legal gates remain. | No; this is a facet, not a production grant. |
| `in-engine-candidate` | Native or candidate renderer evidence exists with `truth_mutation=false`, but it is not production-renderer acceptance. | No. |
| `production-ready` | Source-approved plus DCC/interchange source, cleaned topology/UV/material/rig/contact profiles, runtime export, native in-engine proof, package inclusion, and truth-isolation evidence. | Yes. |
| `rejected` | Wrong category, IP risk, unverifiable terms, bad topology/material/rig, or visual/technical failure. | No; move to `assets_src/rejected/`. |
| `debug-only` | Useful local verification primitive/procedural asset that should never ship as production art. | No. |

## Rodin/export provenance requirements

Every Rodin/generated/imported asset must record:

- asset id and intended category;
- source prompt and/or source image path/hash;
- tool name, integration, model/tier, version, API endpoint, and date;
- Rodin task UUID/job UUID/subscription key reference where applicable;
- export files and hashes;
- subscription/plan at generation time without secrets;
- terms/export-rights snapshot URL, retrieval date, and hash;
- license status and commercial-use status;
- third-party protected-IP risk review;
- rejection/quarantine reason if terms or source cannot be verified.

Current Hyper3D/Rodin evidence from 2026-06-30 research:

- Hyper3D API docs say Rodin API requires a Business subscription and bearer API key; generation is asynchronous through `/api/v2/rodin`, status, and download endpoints.
- Gen-2.5 docs list GLB/USDZ/FBX/OBJ/STL output, PBR material output, texture modes, TApose, quality/face-count controls, and HighPack/HD texture options.
- Hyper3D pricing states Creator includes unlimited export/any use and Business includes API access; pricing FAQ says paid plans include broader export/usage rights and should be reviewed before regulated/high-risk use.
- The searched Terms URL returned 404 during current extraction, so local shipping rights cannot be inferred without a saved terms snapshot or owner/legal review.

Therefore, local Rodin/generative outputs remain `candidate-only / license-pending` until the exact generation-time terms and account plan are recorded.

## Technical production gates

Every production asset requires:

1. Source file in `.blend`, `.usd`, `.usda`, `.usdc`, `.fbx`, or another ADR-approved interchange/source format.
2. Runtime export in glTF/GLB or renderer-native format with content hash.
3. glTF Validator result or documented equivalent structural validation.
4. Mesh metrics: polygon count, vertices, bounds, nonzero Z depth, normals/tangents, UVs, texture coordinate sets, materials, textures, and known topology/manifold status.
5. PBR/equivalent material channels: base color, roughness/metallic/ORM, normal, AO, detail/wear/dirt/blood/wetness where relevant.
6. Fighter rigs: skin, weights, animation/retarget test, canonical 16 truth-joint mapping, grip frames, cosmetic-only bones separated from truth.
7. Weapon profiles: grip frames, edge/point/blunt/hook feature markers, mass distribution, moment of inertia, collision/contact geometry, durability/material profile.
8. Armor profiles: pieces, coverage/gap maps, straps/fasteners, material layers, deformation/damage states, mass/inertia, collision/contact regions, clipping checks.
9. Arena profile: verdict ring, witness positions, oath/witness stone, weapon staging, worn stone/cuts/scuffs/blood wash, maintenance props, banners/markers/lore props, lighting/camera anchors, collision/footing metadata, fog/dust/wetness/weather hooks.
10. In-engine/native screenshot evidence at 1920x1080+ in gameplay/replay/fight-film contexts.
11. Package inclusion test.
12. Presentation truth isolation: renderer/animation/VFX/audio/camera consume hashed truth only and cannot mutate replay/trace/contact/cost/injury/capability/end-state hashes.

## Verification commands

```sh
./tools/audit_generated_assets.sh
./tools/build_assets.sh
./tools/validate_assets.sh
./tools/render_asset_previews.sh
./tools/capture_high_fidelity_screens.sh
./tools/visual_gap_audit.sh
./tools/presentation_truth_isolation.sh
./tools/visual_benchmark.sh
./tools/final_acceptance.sh
```

Expected current result: candidate audits produce reports, while production/high-fidelity gates fail closed until licensed/source-approved production assets and a native production renderer exist.

Unit-046 audit hardening requires every generated/model-candidate audit packet to include JSON/MD/CSV records, blocked-evidence reports, state summaries, quarantine manifests, and production-unblock matrices. The current state remains fail-closed: 22 candidate assets audited, 22 quarantined, 22 candidate-only/license-pending, 0 production-ready, and 0 native production-renderer captures. Channel-complete candidate textures are reported separately from production material quality so base/normal/ORM presence cannot launder 32x32 candidate textures into production acceptance.
