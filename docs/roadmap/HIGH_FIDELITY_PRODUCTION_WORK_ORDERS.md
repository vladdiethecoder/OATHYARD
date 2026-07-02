# OATHYARD High-Fidelity Production Work Orders

Status: work-order/spec package only; not implementation evidence.
Date: 2026-06-29
Kanban: t_5b0b0f71

## Source basis and boundary

Canon precedence preserved for this package:

1. `docs/design/GAME_CANON.md`
2. `docs/design/DEMO_SCOPE.md`
3. `ACCEPTANCE_MAP.md`
4. `docs/acceptance/FULL_GAME_ACCEPTANCE.md`
5. `docs/decisions/0007-high-fidelity-production-target.md`
6. `docs/decisions/0002-native-presentation-target.md`
7. `docs/design/ART_DIRECTION_BRIEF.md`
8. `AGENTS.md` / downstream implementation specs

Current accepted facts:

- OATHYARD is a deterministic native-PC 3D planned-time physical melee duel game.
- Frontier-tech leverage is now specified by `docs/research/FRONTIER_TECH_LEVERAGE.md`, `docs/decisions/0002-high-fidelity-production-target.md`, `docs/decisions/0003-truth-vs-presentation-layering.md`, and `docs/decisions/0004-renderer-and-asset-pipeline.md`.
- MotionBricks-style motion becomes internal `PresentationBricks` presentation only unless actual MotionBricks access/license/build/runtime verification is proven.
- Warp, Isaac Lab, Newton, MJWarp, PhysX, Chrono, and dense-contact/FEM/SPH/DEM/MPM/cloth/deformable solvers are offline research/reference tools by default, not live truth.
- glTF/GLB is the preferred runtime asset-delivery direction where feasible; OpenUSD or equivalent is the preferred source/interchange direction where feasible.
- Audio2Face-3D and generative 3D tools are presentation/authoring aids only and require provenance/license evidence before any production use.
- Truth runs fixed 120 Hz and remains deterministic integer/fixed-point gameplay state only.
- Renderer, UI, audio, VFX, camera, settings, and fight-film systems consume truth only after authoritative hashes and never mutate truth.
- There is no HP/stat shortcut path: no hit points, arbitrary damage numbers, armor points, DPS, crit chance, super meter, perks, unlock stats, or speed/damage bonuses as truth.
- Current blocked native-renderer status, non-native diagram, non-native frame, low-poly glTF, diagnostic image rollups, and non-native local raster captures are local verification evidence only.
- The current final-acceptance bundle may pass the local package-candidate gate while the high-fidelity native-PC 3D production visual gate remains failed/blocking.

Readiness flags remain false in this work package:

| External gate | Current flag | Evidence required before true |
| --- | --- | --- |
| Public demo readiness | `false` | Local package gate plus owner demo-scope acceptance, legal/trademark/license resolution, and approved public distribution path. |
| Release-candidate readiness | `false` | Clean release environment, owner acceptance, legal/trademark/license/store gates, package checksums, and platform review evidence. |
| Owner-final acceptance | `false` | Owner explicitly accepts current native visual/audio/input/accessibility/package evidence. |
| Legal clearance | `false` | Owner/legal clearance artifact. |
| Trademark clearance | `false` | Owner/legal trademark clearance artifact. |
| Store readiness | `false` | Store account/app access, current store checklist/assets, age/pricing/forms, build review evidence. |

Machine-readable boundary vocabulary for downstream cards:

```text
public_demo_ready: false
release_candidate_ready: false
owner_final_acceptance: false
legal_clearance: false
trademark_clearance: false
store_readiness: false
production_renderer_complete: false
owner_visual_acceptance: false
```

Do not flip any of these in source, manifests, package docs, or generated status reports until the actual gate evidence exists.

## Blocking evidence translated into work orders

The blocking visual audit under `artifacts/final_acceptance/latest/visual_fidelity_audit.md` rejects the current native-PC high-fidelity 3D visual gate because:

- renderer quality is prototype/local-debug: primitive low-poly geometry, flat untextured colors, no production anatomy, no detailed armor/weapons/arena, no PBR/equivalent material response, no shadows/GI/atmosphere, no VFX, no finished player-facing camera language;
- the current declared quality preset is `local_verification_raw_x11_ppm`;
- current visual coverage is 960x540 plus 1280x720/1280x800 local evidence, below the required 1920x1080+ capture target;
- stable interactive product frame timing is not proven;
- owner visual acceptance is absent;
- local package-candidate success must not be laundered into high-fidelity completion.

These work orders convert that failure into implementation-sized units. They deliberately do not add runtime dependencies or code.

## Dependency spine

```text
HIFI-WO-00 canon/readiness contract
  -> HIFI-WO-01 continuous native 3D renderer/engine path
  -> HIFI-WO-02 1920x1080+ deterministic captures and performance proof
  -> HIFI-WO-03 source-backed high-detail fighters / armor/loadouts / 8 weapon families / arenas
  -> HIFI-WO-04 PBR/equivalent materials and damage/wear masks
  -> HIFI-WO-05 skeletal/skinned presentation bones separated from truth joints
  -> HIFI-WO-06 animation/VFX/camera/UI presentation integration
  -> HIFI-WO-07 visual benchmark criteria and owner visual acceptance packet
```

No public/store release work should be pulled before HIFI-WO-07 produces an owner-reviewable current capture pack and the owner actually accepts or rejects it.

## HIFI-WO-00: Canon/readiness contract guard

Assignee profile: `gamedesign` for spec updates; `developer` for audit wiring if source checks need expansion.

Deliverables:

- A short implementation checklist copied into every downstream renderer/asset/UI/audio card:
  - truth fixed 120 Hz;
  - renderer/UI/audio/VFX/camera consume truth after hashes only;
  - replay remains authoritative evidence and must fail loudly;
  - current local package gate remains separate from high-fidelity and public/store gates;
  - external readiness flags remain false unless separately evidenced.
- A fail-closed audit extension only if new manifests/status files are added by implementation cards.
- A final handoff line in every child card naming which canon files were reviewed.

Acceptance evidence:

- `./tools/audit_truth.sh` passes after code-impacting implementation cards.
- `./tools/audit_readiness.sh . artifacts/readiness/<card>` passes after readiness/status/doc changes.
- No downstream card uses blocked native-renderer status, non-native frame, low-poly glTF, diagnostic image rollups, or metadata-only checks as high-fidelity product evidence.

Forbidden shortcuts:

- No weakening tests, audits, expected outputs, canon, or readiness language.
- No moving false flags into prose that implies success.
- No implementation card may claim owner visual acceptance from automated inspection.

## HIFI-WO-01: continuous native 3D renderer/engine path

Assignee profile: `developer`; visual review support from `mediaqa`.

Purpose: provide a continuous player-facing high-fidelity-capable native 3D render loop or a separately accepted engine/backend ADR while preserving the deterministic truth boundary.

Entry evidence:

- Alternate renderer spike implementations have been pruned from the source tree; the only retained renderer implementation is the native software 3D path.
- Current local 3D captures remain technical evidence only and cannot satisfy high-fidelity production presentation or owner visual acceptance.

Exact deliverables:

1. Renderer/backend ADR before adoption:
   - backend candidate: dependency-zero native OpenGL path, Vulkan/direct-loader path, or other legally available backend/engine;
   - license, dependency footprint, build/package impact, removal plan, platform target, input/audio implications, capture method, and deterministic truth boundary;
   - package binary/tar delta against current local package baseline;
   - explicit statement that Unreal/Unity/Godot/browser-first adoption is not part of this slice unless a new owner-approved ADR overrides current constraints.
2. Continuous native player-facing render loop:
   - persistent window loop, not only command-generated still captures;
   - menu, fighter select, loadout select, planning, combat, consequence, fight-film, settings, and accessibility screens routed through the same runtime path;
   - first-person, third-person, and fight-film camera modes able to inspect the same verified truth events;
   - render frame timing measured outside authoritative truth;
   - input events kept in the presentation-command boundary until they author replayable committed inputs.
3. Truth-boundary proof:
   - renderer reads replay/trace/truth state only after final state hashes/content hashes are computed;
   - before/after replay JSON, final state hash, content hash, contact packets, injuries, capability deltas, action validity, and end-condition data remain unchanged by rendering;
   - audit/report field `truth_mutation: false` for each render/capture path.
4. Renderer data interface schema:
   - explicit, stable schema for per-frame presentation input: replay id, content hash, final hash, tick/frame, camera mode, truth poses/events after hash, asset ids, material ids, damage/wear masks, UI state, capture settings;
   - no unordered iteration in any replay-relevant presentation export.

Acceptance evidence:

- Focused renderer spike/build log with exact command and tool versions.
- Current-run captures from the continuous loop, including at least one 1920x1080+ frame after replay verification.
- `./tools/renderer_target_audit.sh artifacts/renderer_target/<card>` passes and still records production renderer completion false until all HIFI-WO gates pass.
- `./tools/replay_verify.sh <renderer-input-replay>` passes before and after capture.
- `./tools/audit_readiness.sh . artifacts/readiness/<card>` passes if ADR/docs/status files change.

## HIFI-WO-02: 1920x1080+ deterministic captures and performance proof

Assignee profile: `developer`; visual artifact inspection by `mediaqa`; GUI/manual smoke by `desktopcontrol` when live windows must be exercised.

Purpose: replace sub-1080p local evidence with deterministic high-resolution capture coverage for every required product state, without using capture as truth.

Exact deliverables:

1. Deterministic capture matrix at 1920x1080+:
   - main menu;
   - settings/accessibility;
   - fighter select;
   - loadout select;
   - OATHYARD establishing shot;
   - six fighter closeups;
   - six armor/loadout family closeups;
   - eight weapon family closeups;
   - OATHYARD verdict ring and training arena;
   - planning timeline;
   - pre-contact frame;
   - contact frame;
   - armor/material damage frame;
   - injury/capability consequence frame;
   - replay verification UI;
   - fight-film camera shot;
   - performance/debug overlay.
2. Optional stretch captures at 2560x1440 when host/toolchain supports it.
3. Capture manifest schema:
   - capture id, command, git/worktree identity if available, replay path, replay final hash, content hash, asset manifest hash, renderer/backend id, resolution, camera mode, frame/tick, sha256, truth_mutation false, owner_visual_acceptance false.
4. Timing/performance report:
   - simulation step time separate from render frame time;
   - frame time distribution, not only nominal FPS;
   - startup/load time;
   - memory estimate;
   - package size delta;
   - explicit note that artifact generation throughput is not interactive product FPS.
5. Pixel/visual review packet:
   - native 3D capture manifest/index for all captures;
   - failed-artifact triage file;
   - direct image inspection notes by `mediaqa` or owner.

Acceptance evidence:

- Capture command exits 0 and writes all required files.
- Manifest hashes match files on disk.
- At least one replay input is verified before capture.
- Current captures are actually 1920x1080 or larger for every required state.
- `./tools/visual_evidence_index.sh artifacts/visual_evidence/<card>` or successor reducer indexes the new captures without owner acceptance claims.
- `./tools/audit_readiness.sh . artifacts/readiness/<card>` passes after docs/status changes.

Forbidden shortcuts:

- No upscaled 960x540/1280x720 evidence may satisfy the 1920x1080+ requirement.
- No metadata-only check may replace pixel inspection.
- No capture may be generated from stale replay/camera data without replay verification.

## HIFI-WO-03: source-backed high-detail fighters, armor/loadouts, 8 weapon families, and arenas

Assignee profile: `mediaqa` for visual criteria and acceptance review; `developer` for source/runtime asset pipeline and validators.

Purpose: replace current low-poly verification assets with source-backed, repo-owned, production-target assets that keep gameplay truth physical and deterministic.

Required content floor:

| Category | Exact coverage required | Production deliverables |
| --- | --- | --- |
| Source-backed high-detail fighters | Six fighter traditions: `saltreach_duelist`, `oathyard_writ`, `chainbreaker`, `reed_sentinel`, `gate_shield`, `bruiser_oath`. | Source files, provenance/license notes, high-detail mesh, rigging notes, material zones, closeups, gameplay-distance captures, current QA-fix checklist. |
| Armor/loadouts | Six armor/loadout families: `gambeson`, `mail_hauberk`, `heavy_plate`, `lamellar`, `fencer_light`, `bruiser_padded_plate`. | Layered wearable meshes or socketed armor pieces, coverage/gap notes, mass/inertia truth mapping notes, no-clipping proof, closeups. |
| 8 weapon families | `curved_sword`, `longsword`, `bearded_axe`, `ash_spear`, `round_shield`, `iron_maul`, `arming_sword`, `billhook`. | Edge/blunt/pierce/hook geometry, grip points, reach/mass/inertia truth mapping notes, material/detail closeups, gameplay-distance silhouette proof. |
| Arenas | `oathyard_verdict_ring`, `training_yard`. | High-fidelity duel-readable environment geometry, lighting anchors, material zones, foot/weapon readability proof, establishing shots. |

Exact deliverables for each asset:

1. Authoring source under `assets_src/` or a documented successor source directory.
2. Provenance record:
   - repo-owned or licensed source statement;
   - author/tool chain;
   - no copied/scraped/unlicensed source;
   - source hash;
   - generated/runtime export hash.
3. Runtime export:
   - glTF or successor runtime format with nonzero Z depth;
   - source-to-runtime manifest entry;
   - local structural validation;
   - external Khronos/DCC validation only if tools actually run.
4. Preview/capture set:
   - isolated asset closeup;
   - gameplay-distance view;
   - in-combat or in-selection context view;
   - 1920x1080+ where HIFI-WO-02 is available.
5. QA criteria from `docs/design/ART_DIRECTION_BRIEF.md`:
   - primary silhouette survives gameplay-distance native 3D capture;
   - materials differentiated by geometry/normals/masks, not recolor alone;
   - functional negative space preserved;
   - fighter/loadout identity follows the per-asset revision targets.

Acceptance evidence:

- `./tools/build_assets.sh` and `./tools/validate_assets.sh` or successor asset gates pass.
- `./tools/asset_budget_audit.sh artifacts/asset_budget/<card>` records new budgets without silently inflating ceilings.
- `./tools/asset_visual_atlas.sh artifacts/asset_atlas/<card>` and `./tools/audit_3d_runtime.sh artifacts/runtime_3d/<card>` prove nonzero-Z runtime geometry and source/runtime/preview/provenance coverage.
- `mediaqa` inspects actual captures and either accepts the asset for the next gate or records named blockers.

Forbidden shortcuts:

- No production placeholder primitives, cubes, capsules, flat planes, copied assets, or low-poly debug silhouettes can be accepted as production assets.
- No stale 22-asset/292-vertex local budget may be treated as a production fidelity target; it is a baseline floor only.
- No asset acceptance from manifest metadata without pixel/visual inspection.

## HIFI-WO-04: PBR/equivalent materials and damage/wear masks

Assignee profile: `developer` for material pipeline; `mediaqa` for material visual audit.

Purpose: add convincing material response to fighters, armor, weapons, arenas, and combat consequences while keeping all damage/wear presentation driven by truth-after-hash events.

Exact deliverables:

1. Material schema:
   - material id;
   - base color/albedo or equivalent;
   - normal/height detail or equivalent geometry-backed surface detail;
   - roughness;
   - metallic where applicable;
   - ambient occlusion/curvature/cavity or documented equivalent;
   - emissive only for justified VFX/UI, not material cheating;
   - damage/wear mask channels keyed by truth event ids after hash.
2. Required material families:
   - flesh/tendon/bone/cloth;
   - quilted linen padding;
   - riveted mail;
   - tempered plate;
   - lamellar iron/leather;
   - ash wood;
   - chalk stone;
   - buff leather/textile;
   - dirt, blood, wetness, soot, and wear as localized masks.
3. Lighting/material proof captures:
   - neutral material turntable or closeup;
   - in-arena lit context;
   - pre/post contact material change driven by replay trace;
   - metal/leather/cloth/stone native 3D comparison capture set.
4. Truth-boundary proof:
   - material masks read contact/injury/material-solve events after hash;
   - material changes do not modify action costs, contacts, injuries, capability deltas, or replay hashes.

Acceptance evidence:

- Material manifest validates every runtime asset has the required material ids/maps/equivalent fields.
- Captures show visually distinct metal, cloth, leather, wood, stone, blood/wetness/wear families.
- `./tools/replay_verify.sh <input-replay>` passes after material/VFX capture generation.
- `./tools/audit_truth.sh` passes if code changes are made.
- `./tools/audit_readiness.sh . artifacts/readiness/<card>` passes after material docs/status changes.

Forbidden shortcuts:

- Flat recolor alone is not PBR/equivalent materials.
- Damage/wear cannot be random, time-based, or renderer-decided truth.
- Local texture presence is not material acceptance without rendered inspection.

## HIFI-WO-05: skeletal/skinned presentation bones separated from truth joints

Assignee profile: `developer`; visual/deformation audit by `mediaqa`.

Purpose: present fighters, armor, cloth, straps, and weapon attachments with credible deformation while preserving canon truth joints as the only authoritative gameplay body graph.

Truth joints remain:

`root`, `spine_lower`, `spine_upper`, `neck_head`, `shoulder_r`, `elbow_r`, `wrist_r`, `shoulder_l`, `elbow_l`, `wrist_l`, `hip_r`, `knee_r`, `ankle_r`, `hip_l`, `knee_l`, `ankle_l`, plus `grip_r` and `grip_l` frames.

Exact deliverables:

1. Rig separation schema:
   - truth_joint_id mapping for every presentation anchor;
   - presentation-only bones for face/head detail, hands/fingers, cloak, skirt, scabbard, straps, armor plates, cloth folds, weapon secondary motion;
   - explicit `presentation_only: true` for cosmetic bones;
   - no presentation bone writes back to truth state.
2. Skinning proof:
   - non-identity bind pose;
   - vertex weights or documented equivalent;
   - shoulder/elbow/hip/knee deformation captures;
   - armor socket or skinning attachment offsets;
   - no-clipping proof for idle, walk, guard, cut, thrust, brace, bash, hook_bind, grab, shove, kick, recover where applicable.
3. Runtime pose input schema:
   - truth pose/event after hash;
   - presentation pose layer;
   - animation clip id;
   - additive reaction id;
   - cosmetic bone transforms;
   - deterministic capture id.
4. Verification captures:
   - idle/walk/guard/action pose sheet;
   - armor no-clipping sheet;
   - weapon grip alignment sheet;
   - injury/capability pose consequence sheet.

Acceptance evidence:

- Rig/skin manifest validates every fighter has a skeleton hierarchy and every wearable armor has socket/skin data.
- Captures visibly prove deformation and no-clipping in action poses.
- Replay hashes are identical before and after animation presentation capture.
- `./tools/audit_truth.sh` passes after rig/animation code changes.

Forbidden shortcuts:

- Canned animation may present truth events but may not pre-decide hit/contact/injury results.
- No gameplay capability, frame cost, or action validity may be derived from presentation bones.
- Static armor preview is not wearable armor acceptance.

## HIFI-WO-06: animation/VFX/camera/UI presentation integration

Assignee profile: `developer`; `mediaqa` for visual/audio review; `desktopcontrol` for GUI/manual smoke when needed.

Purpose: make combat legible and product-facing across animation, VFX, camera, UI, captions, and audio while preserving replay/truth authority.

Exact deliverables:

1. Animation presentation:
   - authored or generated pose/clip set for observe, plan, step, pivot, guard, parry, cut, thrust, brace, bash, hook_bind, grab, shove, kick, recover;
   - bind/guard/stagger/collapse/injury/recovery reactions driven by truth events;
   - readable weight shift, recovery cost, armor drag, binding strain, grip loss, and capability loss.
2. VFX presentation:
   - sparks, dust, cloth/armor movement, blood/wetness, debris, weapon trails, impact flashes, shock or pressure cues only when trace events justify them;
   - no UI glyph/proxy bars masquerading as production VFX;
   - VFX event manifest keyed to verified replay/trace events.
3. Audio/caption integration:
   - UI, impact, armor/material, capability, ambience, and settings-preview audio event categories;
   - captions or visual equivalents for combat-critical audio;
   - gains/mute/settings remain presentation-only and do not affect replay hashes;
   - shipping backend still requires a separate ADR and owner audio acceptance.
4. Camera language:
   - first-person, third-person, fight-film, asset closeup, planning and consequence cameras;
   - camera must preserve feet, weapon arcs, contact surfaces, armor gaps, injury/capability evidence, UI readability, and motion comfort settings;
   - no camera framing that hides determinism evidence in benchmark captures.
5. UI language:
   - base/current frame cost and physical delta reasons visible on consequence screens;
   - text scale, contrast, reduced motion, reduced flash, remapping, captions, and audio settings visible in native UI.

Acceptance evidence:

- Current-run captures at 1920x1080+ for every UI/camera state once HIFI-WO-02 is available.
- Replay verifies before fight-film/camera/VFX capture.
- Audio/VFX artifacts derive only from trace/replay events and captions exist for critical cues.
- `./tools/audio_target_audit.sh artifacts/audio_target/<card>` and `./tools/input_target_audit.sh artifacts/input_target/<card>` remain green when those surfaces change.
- `mediaqa` records frame-by-frame inspection notes and named blockers.

Forbidden shortcuts:

- No arbitrary damage numbers, health bars, super meters, DPS, crits, or stat perks in UI truth.
- No VFX/audio event may decide contact, injury, or capability outcomes.
- No browser/HTML artifact may be claimed as native product presentation.

## HIFI-WO-07: visual benchmark criteria and owner visual acceptance packet

Assignee profile: `mediaqa`; `gamedesign` for acceptance wording; `developer` only if packet tooling must change.

Purpose: turn the high-fidelity target into a fail-closed owner-review packet without converting automated checks into owner acceptance.

Visual benchmark criteria:

| Criterion | Required pass evidence | Failure examples |
| --- | --- | --- |
| Silhouette readability | Fighter tradition, armor/loadout, weapon family, arena, guard state, contact state readable at gameplay distance. | Low-poly blockouts, merged arms/straps/weapons, recolor-only families. |
| Material richness | PBR/equivalent metal/cloth/leather/wood/stone/flesh/blood/wetness/wear response visible under in-game lighting. | Flat colors, no normals/roughness, random scatter/noise, black blobs. |
| Anatomy and deformation | Credible head/hand/shoulder/hip/knee/foot forms and skinned action poses. | Static mannequins, bad shoulder blocks, no rig hierarchy, clipping armor. |
| Weapon/armor detail | Edge/blunt/pierce/hook/grip/reach and armor coverage/gaps/straps/buckles/mail/plate readable. | Weapon reads as slab/post; armor reads as gray tunic. |
| Arena identity | OATHYARD verdict ring and training yard frame the duel without hiding feet/contact. | Blank floor, noisy decals, flat disk, no judgment/training landmarks. |
| Lighting/atmosphere | Shadows, depth, fog/dust/weather/atmosphere or documented equivalent support dark-fantasy judicial duel mood. | No lighting, no shadows, no atmosphere, debug dark background only. |
| Animation/contact readability | Pre-contact, contact, bind/guard, material solve, injury/capability, recovery readable across frames. | Canned hit results, hidden contact, unreadable consequence. |
| UI legibility | Product UI readable at 1920x1080+, including settings/accessibility and consequence deltas. | Debug labels only, tiny text, missing physical reason for cost changes. |
| Originality/no-copying | References Elden Ring/For Honor only for quality/readability bar; OATHYARD remains original. | Copied names, assets, silhouettes, factions, UI, animations, lore, textures, music, mechanics. |
| Performance evidence | Render frame timing, startup, memory, package size measured outside truth. | Nominal FPS only, artifact throughput misreported as interactive FPS. |
| Truth boundary | Renderer/UI/audio/VFX/camera truth mutation false and replay hashes stable. | Presentation writes action costs, contacts, injuries, capability deltas, or replay hashes. |

Owner visual acceptance packet deliverables:

1. Packet root: `artifacts/owner_visual_acceptance/<UTC-or-card-id>/`.
2. `owner_visual_acceptance_manifest.json`:
   - capture list and hashes;
   - replay paths and replay final hashes;
   - content/asset manifest hashes;
   - renderer/backend id;
   - resolution list including 1920x1080+;
   - benchmark criteria version;
   - `production_renderer_complete: false` until accepted by the correct gate;
   - `owner_visual_acceptance: false` until owner marks acceptance.
3. `visual_benchmark_report.md` comparing the current packet to the criteria above.
4. Native 3D renderer capture manifest and per-capture image paths.
5. `failed_visual_artifacts.txt` with missing/failing captures; empty only when every packet artifact exists and hashes verify.
6. `owner_review_checklist.md` with explicit accept/reject rows for visual, input, audio, accessibility, demo scope, and blocker notes.
7. Owner response artifact recorded separately after human review; automated agents may prepare the packet but cannot sign it.

Acceptance evidence:

- Packet generated from current native executable and current assets, not stale copied images.
- Every source replay verifies before capture.
- All capture hashes verify after packet assembly.
- `mediaqa` inspects actual images and records pass/fail per criterion.
- Owner explicitly accepts or rejects. Until then `owner_visual_acceptance: false` remains correct.
- `./tools/audit_readiness.sh . artifacts/readiness/<card>` passes after packet/status/docs changes.

Forbidden shortcuts:

- Do not call the packet owner accepted before owner review.
- Do not call the game public-demo ready, release-candidate ready, legal cleared, trademark cleared, or store ready from this packet alone.
- Do not benchmark against Elden Ring/For Honor by copying any protected asset, identity, animation, UI, lore, music, or proprietary mechanic.

## Downstream implementation card list

These are the implementation cards that should be created or pulled after this work-order package. Dependencies should be encoded in the board, not only prose.

| Proposed card | Profile | Depends on | Acceptance summary |
| --- | --- | --- | --- |
| `HIFI-RENDERER-ADR-AND-LOOP-001` | `developer` | HIFI-WO-00 | Renderer/backend ADR plus continuous player-facing native 3D loop spike; truth mutation false; no readiness flags flipped. |
| `HIFI-CAPTURE-MATRIX-1920-001` | `developer` | Renderer loop spike | 1920x1080+ deterministic capture matrix and timing report for all required states; manifest hashes verify. |
| `HIFI-ASSET-SOURCE-PACK-001` | `mediaqa` + `developer` | Work-order spec | Source-backed high-detail fighters, armor/loadouts, 8 weapon families, and arenas with provenance and visual QA criteria. |
| `HIFI-MATERIAL-PBR-MASKS-001` | `developer` + `mediaqa` | Asset source pack | PBR/equivalent material schema, material IDs, maps/equivalents, and trace-driven damage/wear masks. |
| `HIFI-RIG-SKIN-ANIMATION-001` | `developer` + `mediaqa` | Asset source pack | Skeletal/skinned presentation bones separated from truth joints; deformation/no-clipping/action pose proof. |
| `HIFI-VFX-AUDIO-CAMERA-UI-001` | `developer` + `mediaqa` + `desktopcontrol` | Renderer/capture/rig/material gates | Animation/VFX/camera/UI/audio/caption integration from trace/replay events with product-facing captures. |
| `HIFI-VISUAL-BENCHMARK-PACKET-001` | `mediaqa` | Current renderer/assets/capture packet | Benchmark report, image inspection, owner visual acceptance packet, failed-artifact triage, readiness flags false until owner review. |
| `HIFI-OWNER-ACCEPTANCE-REVIEW-001` | `mediaqa` | Visual benchmark packet | Owner review requested/recorded; accepts or rejects visual/input/audio/accessibility/demo scope explicitly. |

## Verification plan for this spec package

This document is a design/spec artifact only, so verification is structural and readiness-boundary focused:

```sh
python3 - <<'PY'
from pathlib import Path
p = Path('docs/roadmap/HIGH_FIDELITY_PRODUCTION_WORK_ORDERS.md')
s = p.read_text(encoding='utf-8')
required = [
    'continuous native 3D renderer/engine path',
    'FRONTIER_TECH_LEVERAGE.md',
    'PresentationBricks',
    'offline research/reference tools',
    'glTF/GLB',
    'OpenUSD',
    '1920x1080+ deterministic captures',
    'source-backed high-detail fighters',
    'armor/loadouts',
    '8 weapon families',
    'arenas',
    'PBR/equivalent materials',
    'skeletal/skinned presentation bones separated from truth joints',
    'animation/VFX/camera/UI',
    'visual benchmark criteria',
    'owner visual acceptance packet',
    'Public demo readiness | `false`',
    'Release-candidate readiness | `false`',
    'Owner-final acceptance | `false`',
    'Legal clearance | `false`',
    'Trademark clearance | `false`',
    'Store readiness | `false`',
]
missing = [term for term in required if term not in s]
if missing:
    raise SystemExit('missing required terms: ' + ', '.join(missing))
print('structural high-fidelity work-order check passed')
PY
./tools/research_audit.sh artifacts/research_audit/high_fidelity_work_orders
./tools/audit_readiness.sh . artifacts/readiness/high_fidelity_work_orders
```

Passing these commands proves only that the spec names required categories and preserves readiness boundaries. It does not prove implementation, visual quality, or owner acceptance.
