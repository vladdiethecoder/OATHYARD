# OATHYARD verdict-ring / training-yard identity audit

Status: actionable design audit for kanban task `t_b92fe668`; no implementation or readiness acceptance claim.
Date: 2026-06-30
Workspace: `/run/media/vdubrov/NVMe-Storage/OATHYARD`

## 0. Boundary

This document audits the current OATHYARD verdict-ring and training-yard identity against canon and art direction. It is written for downstream implementation card `t_48a6a7f8` and related world/arena remediation work.

It does not set or imply owner visual acceptance, production renderer completion, public-demo readiness, release-candidate readiness, legal clearance, trademark clearance, or store readiness. All those gates remain false until separately evidenced.

Presentation/world work remains after authoritative truth hashes. Renderer, UI, audio, VFX, camera, materials, post-processing, animation, and environment visuals must not mutate replay JSON, trace JSON, contact packets, action costs, injuries, capability deltas, content hashes, final hashes, winners, or end-state truth.

## 1. Controlling sources

Canon precedence reviewed:

1. `docs/design/GAME_CANON.md`
   - OATHYARD identity: deterministic native-PC 3D planned-time physical melee duel game (`GAME_CANON.md:5-15`).
   - Blocked native-renderer status, non-native diagram, non-native frame, low-poly glTF, and non-native local raster captures are local verification evidence only (`GAME_CANON.md:11-15`).
   - Public-demo and release-candidate readiness stay false without explicit owner/human gates (`GAME_CANON.md:165-170`).
2. `docs/design/DEMO_SCOPE.md`
   - Full-game target includes native menus, local match flow, fighter/loadout selection, deterministic seats, production asset manifests, packaging smoke, fight-film cameras, previews, audio/VFX, and quality gates (`DEMO_SCOPE.md:15-21`).
   - Current raw/headless/non-native diagram/non-native frame evidence is local verification only until a continuous high-fidelity native 3D renderer or accepted backend exists (`DEMO_SCOPE.md:29-35`).
3. `ACCEPTANCE_MAP.md`
   - High-fidelity production gate is not passed and requires high-fidelity arenas/training arena, lighting/atmosphere, deterministic 1920x1080+ capture coverage, visual benchmark report, and owner visual acceptance recorded separately (`ACCEPTANCE_MAP.md:32-40`).
   - Local package and public/store release gates are separate (`ACCEPTANCE_MAP.md:46-99`).
4. `docs/design/ART_DIRECTION_BRIEF.md`
   - Target look: grounded stylized realism; grim judicial-fantasy, tactile materials, readable combat shapes (`ART_DIRECTION_BRIEF.md:18-23`).
   - Main arena target: OATHYARD verdict ring, chalked stone, low rim, north judgment balcony, high cold oath-mark lighting. Training arena target: packed clay, measured practice lines, rope/work-lamp practicality (`ART_DIRECTION_BRIEF.md:50-52`).
   - Arena fixes: verdict ring must read as chalked stone ritual geometry with low rim, oath mark, judgment axis/balcony orientation, entry break, worn contact marks, and clear playable floor. Training yard must resolve into purposeful measured practice lines, start/target/footwork markers, rope/work-lamp/post purpose, and packed-clay/wood separation (`ART_DIRECTION_BRIEF.md:114-119`).
5. `docs/acceptance/VISUAL_FIDELITY_ACCEPTANCE_CRITERIA.md`
   - Required arena captures: verdict-ring and training-yard establishing/gameplay frames at 1920x1080+ (`VISUAL_FIDELITY_ACCEPTANCE_CRITERIA.md:92-153`, `:174-215`).
   - Environment identity thresholds require verdict ring to read as chalked-stone judicial ritual space and training yard to read as practice space with packed clay/stone/wood, measured lines or targets, rope/post/work-lamp/training props, and clear start/footwork markers (`VISUAL_FIDELITY_ACCEPTANCE_CRITERIA.md:199-201`).
   - Current 22-asset/292-vertex/492-triangle evidence is far below the high-fidelity floor and remains Tier 0 debug/local verification (`VISUAL_FIDELITY_ACCEPTANCE_CRITERIA.md:311-324`).

## 2. Current evidence inspected

Current source/manifest evidence:

- `assets_src/arenas/arenas.oysrc:1-7` now contains arena art metadata for `environment_profile`, `material_zones`, `material_maps`, `lighting_anchors`, `landmarks`, `floor_contact`, `composition_profile`, `scale_reference`, `silhouette_context`, `playable_space`, `atmosphere_hooks`, `capture_ids`, and `originality_notes`.
- `assets/runtime_manifest.json:632-702` contains `oathyard_verdict_ring` and `training_yard` runtime entries with material maps and presentation-only PBR profiles; owner/public/release flags remain false (`assets/runtime_manifest.json:711-713`).
- Runtime mesh/glTF inspection in this task found:
  - `oathyard_verdict_ring`: `assets/runtime/oathyard_verdict_ring.mesh.json` SHA-256 `abea152eef54dfb55f142cfbcc98a9ff6df564febbb178f478126f9859c13333`; `assets/gltf/oathyard_verdict_ring.gltf` SHA-256 `53a4c278d3d1ecfb473dfb11ecfa8cc1bef9703f7c964f71bc305f245dc14708`; 309 glTF vertices, 1176 indices, 6 materials, 3 images; scale reference `6200mm_ring_3600mm_clear_combat_core`.
  - `training_yard`: `assets/runtime/training_yard.mesh.json` SHA-256 `aa46f49b7ccda0a52e593e56e81c03e3a150d990b7b339b9e63c3f116008b319`; `assets/gltf/training_yard.gltf` SHA-256 `302a3d9e2a40eabfc969419da3c7abd7b49a3dd8ad8a6b2650d852bfc6736c25`; 253 glTF vertices, 1116 indices, 6 materials, 3 images; scale reference `4800mm_yard_3000mm_practice_core`.

Current historical image evidence (non-acceptance context only):

- Historical rollup image evidence existed for `t_185e9c4c`, but standalone rollups are no longer accepted by the 3D-only visual evidence policy.
- Individual 1920x1080 captures exist for both arenas in establishing, gameplay, and contact modes (`arena_environment_capture_manifest.json:5-270`).
- `arena_environment_capture_report.md:1-29` reports 6 captures, `truth_mutation:false`, owner visual acceptance not claimed, public-demo not ready, release-candidate not ready, and current blockers: non-native local raster PNG local verification evidence only, external Khronos/DCC/renderer acceptance not claimed.
- Vision inspection in this task:
  - Historical rollup review: verdict ring and training yard were visually distinct in palette and enclosure, but shared the same central circular/radial floor silhouette; fighters were tiny colored cubes/markers; context was minimal black void; combat center was readable but not final scale/art proof.
  - Verdict-ring gameplay capture: judicial-duel identity is suggested by a central disc/ring, opposing markers, symmetry, and flanking architecture, but scale is too zoomed out, fighters are minuscule, backdrop is dark/abstract, and debris/radial pattern can obscure action.
  - Training-yard gameplay capture: practice-yard identity is suggested by warm wooden frame, golden boundary, radial target motif, and opposing markers, but it still reads as low-poly blockout, has a huge circular backdrop, busy beams/verticals, and a third figure/outer scaffold that can distract from combat.

## 3. Related-card cross-reference

| Card | Prior work / status relevant to this audit | Use in this audit |
| --- | --- | --- |
| `t_1585bf34` | Root assets/world/arena dissatisfaction card. Owner-failure comment states the visual failure is broader than assets/world/arenas and includes renderer, integrated assets, camera/readability, UI/HUD, VFX/audio, lighting/post, benchmark/owner handoff. | Confirms this audit must not launder structural work into owner visual acceptance. |
| `t_7d209020` | Produced broad asset/world/arena canon audit with 17 findings. It found arena/world selection absent, source layout thin, verdict ring tokenized, training yard unvalidated, and current local visuals below standard. | Baseline gap taxonomy. This audit narrows to current verdict-ring/training-yard identity after later repairs. |
| `t_614cc73b` | Pixel-level visual fidelity audit. It found ultra-low-poly placeholder assets, flat/no material response, primitive arenas/world, text-driven combat feedback, debug HUD density, and missing training-yard scene coverage. | Baseline visual failure. Current captures improve training-yard coverage but still inherit blockout/non-native local raster and scale/readability limits. |
| `t_cb68c544` | Produced `docs/roadmap/HIFI_REMEDIATION_PHASE_PLAN.md`, mapping arena/world remediation to `t_6aea2f80`, renderer/capture work, lighting/post, camera/VFX/UI, benchmark packet, and owner handoff. | Downstream routing source. This audit feeds implementation child `t_48a6a7f8`; broad cards should not duplicate it. |
| `t_34a5af6b` | Legacy broad implementation catch-all; comments mark it aggregate/supersede-only and not broad implementation. | Do not use for vague implementation. Concrete repairs should be on child cards such as `t_48a6a7f8`. |
| `t_6aea2f80` | Implemented arena environment production-art metadata, nonzero-Z geometry, 6-zone PBR materials, validators, capture tooling; board-stage accepted structurally. Later owner-failure comment superseded any visual-acceptance interpretation. | Partial work that supplies source-backed arena metadata and captures. Still only structural/local evidence, not owner/high-fidelity acceptance. |
| `t_185e9c4c` | Owner-failure world/arena identity remediation parent. Latest handoff reports current source-backed identity repairs and captures but blocks on `./tools/verify.sh` because `validate_assets.sh` fails on missing `assets/production_visual_manifest.json`; creating fake production-asset evidence would violate scope. | Current implementation context and evidence root for this audit. |
| `t_48a6a7f8` | Developer child to implement concrete identity repairs using this audit. | Primary consumer. |

## 4. Summary verdict

Current world/arena identity is materially better than the earlier 18-vertex/32-triangle atlas-only baseline. The current evidence shows two arena identities rather than a single manifest token: a cold, open, circular verdict ring and a warm, framed training yard. Both now have source-backed metadata, material maps, nonzero-Z geometry, and 1920x1080 establishing/gameplay/contact captures.

The remaining issue is not asset existence. The issue is that the current pixels still read as non-native local rasterized low-poly environment visualization with shared radial-floor DNA, tiny placeholder fighters, black-void context, debug-like guide lines/borders, and limited environmental storytelling. It is adequate as a structured implementation seed. It is not yet an owner-reviewable high-fidelity arena identity packet.

## 5. Enumerated gaps and remediation directives

### ID-01 — Shared radial floor makes the two arenas read like variants of one stage

Area: identity, silhouette/context, originality
Severity: blocker for owner-facing world identity; major for current implementation slice

Observed gap:

- Historical review artifacts and individual gameplay captures showed both arenas using a dominant circular/radial floor/backdrop as the main visual signature.
- Verdict ring: the radial disc supports the oath/verdict concept.
- Training yard: the same large circular/radial motif makes the yard read as a warm reskin of the verdict ring or as a target-wheel stage, not a practical measured practice yard.

Canon/art conflict:

- Verdict ring target: chalked stone ritual geometry, low rim, judgment axis/balcony orientation, entry break, worn contact marks (`ART_DIRECTION_BRIEF.md:118`).
- Training yard target: packed clay, measured practice lines, rope/work-lamp practicality, named start/target/footwork markers, material separation (`ART_DIRECTION_BRIEF.md:119`).

Remediation directive:

- Keep the circular/radial oath geometry as a verdict-ring-specific identity element.
- Remove or subordinate the circular/radial motif from the training yard. Replace the training-yard dominant silhouette with a rectangular drill inset, lanes, footwork grids, start boxes, target marks, rope line, posts, weapon rack, water barrel, and maintenance/tool zone.
- Target state: in a native 3D renderer capture with labels removed, reviewers can identify verdict ring as circular judicial ritual and training yard as rectangular/practical drill yard.
- Implementation touchpoints: `assets_src/arenas/arenas.oysrc`, arena mesh generation in `tools/asset_pipeline.py`, and capture styling in `tools/arena_environment_captures.sh`.

Acceptance evidence:

- Updated native 3D arena capture manifest where the training yard no longer depends on a verdict-ring-like radial center.
- `arena_world_identity_manifest.json` preserves distinct `composition_profile`, `silhouette_context`, and `playable_space` for both arenas.
- Media/vision review explicitly answers “distinct environment, not reskin” without relying on labels.

### ID-02 — Verdict-ring judicial identity is too abstract; it needs court-of-violence hierarchy

Area: identity, composition, environmental storytelling
Severity: major

Observed gap:

- The verdict ring pixels show a compelling central disc and flanking architecture, but the “judicial” function is mostly inferred from symmetry, dark palette, and metadata.
- The north judgment balcony, witness markers, entry break, oath stones, chain posts, weapon/evidence staging, and verdict hierarchy are not yet strong enough as visual facts.

Canon/art conflict:

- OATHYARD target is “dark-fantasy judicial-duel art direction” (`GAME_CANON.md:9-15`).
- Art direction says gear/environments should feel like “evidence in a court of violence” and the verdict ring should carry judgment ritual, oath marks, low rim, balcony orientation, and worn contact marks (`ART_DIRECTION_BRIEF.md:18-23`, `:50-52`, `:118`).

Remediation directive:

- Build a clear north-south judgment axis:
  - north: raised balcony/dais with strong silhouette, oath-light aperture, witness rail;
  - south: entry break/gate aligned to the duel axis;
  - east/west: witness pillar and chain post as readable landmarks;
  - floor: chalk oath ring, scuffed center, radial cracks, worn contact stains localized to combat paths.
- Do not add non-diegetic text or readiness badges to explain the court identity. The architecture must communicate it.
- Keep all lighting/camera/weather hooks presentation-only.

Acceptance evidence:

- Establishing capture reads as a judicial ritual arena before seeing the filename.
- Gameplay capture keeps the court hierarchy visible without hiding feet, weapons, or contact state.
- The low rim and entry break are visible as spatial affordances, not only source tokens.

### ID-03 — Training yard reads as a stylized cage/arena, not yet a practical training ground

Area: identity, composition, readable combat space
Severity: major

Observed gap:

- The training-yard capture has warm wood, a golden rectangular border, beams, and a radial target motif. It suggests a practice/dojo stage, but not yet OATHYARD’s packed-clay yard with measured lines, work lamps, posts, rope practicality, and tool/weapon staging.
- The large circular backdrop competes with the practical lane/grid identity.

Canon/art conflict:

- Training target: packed clay, measured practice lines, rope/work-lamp practicality (`ART_DIRECTION_BRIEF.md:50-52`).
- Required training yard cues: clean circular boundary or rectangular practice inset, named markers for start/target/footwork, rope/work-lamp/post purpose, packed-clay/wood/material separation, markers that teach spacing rather than random grid (`ART_DIRECTION_BRIEF.md:119`).

Remediation directive:

- Make the training yard a working space:
  - clay floor with heel gouges and swept lane marks;
  - paired start marks and target lane sized for OATHYARD reach/footwork;
  - rope line with posts that marks spectator/tool boundary, not a cage wall;
  - weapon rack and water barrel as side props outside the clear combat core;
  - warm work lamps that illuminate lanes without turning into decorative fantasy torches.
- Use measured practice lines as functional spacing tools for `step`, `pivot`, `guard`, `parry`, `cut`, `thrust`, `brace`, `bash`, `hook_bind`, `grab`, `shove`, `kick`, and `recover` drills.

Acceptance evidence:

- Gameplay capture shows start boxes, lane, and outer tool zone with the two fighters clearly inside a practical practice core.
- A reviewer can infer “training yard” from floor/props/lanes without labels.
- Any third/background figure or target dummy must be obviously outside playable bounds or intentionally a training dummy, not an accidental distractor.

### COMP-01 — Composition is centered and symmetric but lacks authored negative-space hierarchy

Area: composition, readability
Severity: major

Observed gap:

- Both arenas center the action and use symmetry. This is stable for a review rollup, but too generic as product composition.
- The current negative space is mostly a black void or repeated beams/guide lines. It does not yet tell where players may move, where observers stand, where evidence/weapon staging lives, or why the camera is placed there.

Canon/art conflict:

- Environments should frame the duel and show material storytelling without hiding feet, weapon arcs, contact, recovery, or consequence states (`ART_DIRECTION_BRIEF.md:50-53`).
- Visual acceptance requires at least three depth planes: foreground playable surface, duel subject plane, and background architectural/environment plane (`VISUAL_FIDELITY_ACCEPTANCE_CRITERIA.md:196-201`).

Remediation directive:

- For verdict ring, use asymmetric narrative anchors while preserving duel symmetry: north balcony overhang, south entry gap, east witness pillar, west chain post, floor scars radiating from prior verdicts.
- For training yard, use practical side zones: one side weapon rack/water barrel/workbench; the other rope line/lamp posts/target board; keep central lane uncluttered.
- Replace generic black void with controlled background depth planes that support readability.

Acceptance evidence:

- Establishing captures show foreground/midground/background separation in both arenas.
- Gameplay captures retain a clean central combat corridor and expose side-context only as readable framing.

### COMP-02 — Contact-mode environment state is not visually distinct enough from gameplay mode

Area: composition, readable combat space, combat feedback handoff
Severity: major

Observed gap:

- Contact captures currently add red/orange marks or fragments, but the state still reads mostly as the same environment with colored overlays.
- This does not yet provide an environment-side visual language for contact, scuffs, disturbed clay, chalk dust, or consequence paths.

Canon/art conflict:

- Contact truth path must remain authoritative and replay-derived (`GAME_CANON.md:134-139`). Presentation may consume truth after hashes but must show contact outcomes clearly.
- Visual criteria require pre-contact/contact/post-contact/consequence frames where contact/no-contact and injury/capability consequence are unambiguous without metadata (`VISUAL_FIDELITY_ACCEPTANCE_CRITERIA.md:235-259`).

Remediation directive:

- Environment response should be trace-derived and local to the contact site:
  - verdict ring: chalk dust, fresh scuff, chip/crack highlight, wet stone smear, not random red wedges;
  - training yard: clay scrape, heel gouge, dust puff, rope shadow/marker disturbance, not decorative orange bars.
- Keep any damage/wear or floor disturbance keyed to replay/trace event ids after truth hash. No environment effect may decide contact.

Acceptance evidence:

- Contact frames can be recognized as contact frames even when labels are removed.
- `native_combat_render`/capture manifests keep `truth_mutation:false` and stable final hash.

### SCALE-01 — Placeholder fighters are too small and abstract to validate arena scale

Area: scale, readable combat space
Severity: blocker for owner-facing scale acceptance; major for implementation

Observed gap:

- Vision inspection of full-resolution captures found the fighters/markers minuscule relative to the arena. In the verdict ring, the ring feels enormous and the figures are hard to read. In the training yard, the structure dominates the figures.
- Current colored block/cube fighters do not prove human scale, weapon reach, guard state, foot placement, or playable edge affordances.

Canon/art conflict:

- OATHYARD combat must communicate action intent, weapon reach, guard state, armor coverage, injury/capability changes, and contact outcomes instantly (`ART_DIRECTION_BRIEF.md:18-23`).
- Arena/visual acceptance needs gameplay-distance views where fighter/loadout/weapon/arena read without labels (`VISUAL_FIDELITY_ACCEPTANCE_CRITERIA.md:92-153`, `:363-372`).

Remediation directive:

- Add a scale contract for arena presentation:
  - explicit meter grid or art-authored scale markers tied to `scale_reference` but not debug overlay;
  - human-height reference from real fighter silhouettes, not colored cubes;
  - visible reach bands for weapon lengths only when diegetic/training-appropriate;
  - camera distances that keep fighters large enough to identify stance, weapon, and edge/guard relation.
- Target: gameplay frame should show two fighters and their weapons large enough to parse guard/contact state while still showing bounds and key landmarks.

Acceptance evidence:

- Capture packet includes at least one gameplay frame per arena where a reviewer can identify fighter side, active weapon family, feet, approximate reach, and bounds without metadata.
- `scale_reference` metadata remains, but acceptance is based on pixels and reviewed captures.

### SCALE-02 — Playable core dimensions exist as metadata but not as diegetic visual affordances

Area: scale, readable bounds
Severity: major

Observed gap:

- Source says verdict ring has `6200mm_ring_3600mm_clear_combat_core`; training yard has `4800mm_yard_3000mm_practice_core` (`assets_src/arenas/arenas.oysrc:6-7`).
- Current pixels rely on visible ovals, golden rectangles, or guide lines/borders that read partly like debug overlays or image-rollup framing.

Canon/art conflict:

- Arena space must preserve feet, weapon arcs, contact, recovery, and consequence states (`ART_DIRECTION_BRIEF.md:50-53`).
- Raw/debug overlays cannot be product-presentation proof (`GAME_CANON.md:11-15`, `VISUAL_FIDELITY_ACCEPTANCE_CRITERIA.md:22-29`).

Remediation directive:

- Convert bounds into diegetic geometry/materials:
  - verdict ring: low rim, entry break, chalk ring, worn inner clear core, darker outer witness line;
  - training yard: rope boundary, clay-lane edges, paired start marks, target lane, outer tool zone.
- Debug guides may remain in QA overlays, but product captures need a clean/no-debug variant.

Acceptance evidence:

- Separate clean product capture and audit-overlay capture for each arena.
- In clean capture, bounds are readable from environment geometry and floor treatment alone.

### SC-01 — Skyline/backdrop/context is too sparse to carry world identity

Area: silhouette/context, environmental storytelling
Severity: major

Observed gap:

- Current captures largely isolate the stage in a dark/black void. There are some towers, beams, bars, and lantern-like shapes, but no coherent skyline, surrounding yard, witness area, or spatial relationship to OATHYARD’s judicial world.
- The result reads like an asset preview or fighting-game blockout rather than a lived-in verdict/training location.

Canon/art conflict:

- Production target requires high-fidelity arenas, lighting, atmosphere, material maps, and dark-fantasy judicial-duel art direction (`GAME_CANON.md:9-15`, `ACCEPTANCE_MAP.md:32-40`).
- Visual criteria require depth planes, atmosphere/depth, and environment identity without copying references (`VISUAL_FIDELITY_ACCEPTANCE_CRITERIA.md:174-215`).

Remediation directive:

- Add authored backdrop layers:
  - verdict ring: courthouse/yard wall silhouette, oath-lit balcony, witness rail, chain shadows, cold stone recesses, non-playable observer plane;
  - training yard: perimeter wall/fence, storage/tool racks, practice posts, work lamps, clay/wood transitions, overcast yard light.
- Keep background contrast low enough to not steal from combat silhouettes.

Acceptance evidence:

- Establishing captures show at least three depth planes in both arenas.
- Gameplay captures show backdrop context while keeping fighters and weapon arcs dominant.

### SC-02 — Vertical elements and radial patterns risk fighting readability

Area: silhouette/context, readable combat space
Severity: major

Observed gap:

- Verdict ring: radial spokes/debris and bright fragments can compete with tiny fighters near center.
- Training yard: vertical posts/beams, a golden border, outer scaffold, and radial target/backdrop create busy subdivisions. Vision inspection flagged possible obstruction/confusion from vertical beams and central radial pattern.

Canon/art conflict:

- Environments must frame the duel without hiding feet, weapon arcs, contact, recovery, or consequence states (`ART_DIRECTION_BRIEF.md:50-53`).
- Camera/arena acceptance requires preserving feet, hands/grips, weapon arcs, contact surface, armor gaps, and consequence UI/readouts (`VISUAL_FIDELITY_ACCEPTANCE_CRITERIA.md:253-259`).

Remediation directive:

- Establish a combat readability exclusion zone around the playable core:
  - no tall posts, lanterns, racks, chains, debris, or high-contrast floor pattern crossing the combat silhouettes at expected camera angles;
  - floor markings fade behind fighters and brighten only at edges/lanes;
  - landmark silhouettes stay outside weapon-arc lanes.
- Add a renderer/capture QA check that flags high-contrast vertical lines intersecting fighter bounding boxes in gameplay/contact captures.

Acceptance evidence:

- Reviewer can identify feet, body orientation, active weapon side, and contact/no-contact in both arenas without reading labels.
- Image rollup includes clean product view plus optional overlay/debug view.

### READ-01 — Arena select / scenario use is still a design-risk surface

Area: readable combat space, full-game flow
Severity: major until implemented/verified through product flow

Observed gap:

- Current capture tooling can render both arenas, but previous audits found scenario grammar and native combat paths hard-coded to `oathyard_verdict_ring` and lacking full arena/world selection (`t_7d209020`, `t_614cc73b`).
- Current source/capture work improves evidence, but this audit has not verified a native arena-select product flow.

Canon/art conflict:

- Full-game target includes native menus, local match flow, fighter/loadout selection, and production asset manifests (`DEMO_SCOPE.md:15-21`).
- Visual criteria require arena select when implemented and both arena establishing captures (`VISUAL_FIDELITY_ACCEPTANCE_CRITERIA.md:92-103`).

Remediation directive:

- Downstream implementation should ensure `training_yard` and `oathyard_verdict_ring` can be selected or otherwise deliberately exercised through match/capture/product flow, not only atlas/capture scripts.
- Include arena id in the capture manifest, content/asset hashes, replay source where combat is represented, and truth-mutation boundary.

Acceptance evidence:

- A current run demonstrates both arenas through the same player-facing or capture path expected for the product loop.
- If true arena selection is deferred, report it explicitly as a blocker rather than treating scripted capture as product flow.

### ORIG-01 — Originality is mostly metadata plus one radial motif; needs OATHYARD-specific visual grammar

Area: originality, identity
Severity: major

Observed gap:

- Source metadata correctly says repo-owned/no copied IP, and no direct copied reference content was observed in the inspected captures.
- However, current originality is carried mainly by abstract radial geometry, palette contrast, and declared landmark names. Training-yard cage/wood-frame language and verdict-ring pillars can still read as generic fighting-game/fantasy blockout.

Canon/art conflict:

- Elden Ring and For Honor are quality/readability references only; OATHYARD must remain original and not copy names, assets, silhouettes, factions, UI, animations, lore, textures, music, or mechanics (`GAME_CANON.md:9-12`, `VISUAL_FIDELITY_ACCEPTANCE_CRITERIA.md:43-87`).
- OATHYARD’s own target vocabulary includes oaths, verdict marks, chalked rings, worn mail, strapped plate, ash wood, leather ties, blood/dirt/wetness, and visible consequence of physical contact (`ART_DIRECTION_BRIEF.md:18-23`).

Remediation directive:

- Define an OATHYARD-specific arena motif set:
  - verdict marks are chalk/legal glyphs tied to oath ritual, not generic runes;
  - witness-chain markers and judgment balcony have distinctive geometry;
  - training markers are practical drill inscriptions, not magical circles;
  - material wear follows combat/use zones, not random decoration.
- Add `originality_notes` fields that name the motif source and what must not be copied, but make the pixels carry the identity.

Acceptance evidence:

- Visual benchmark row “originality/no-copying” passes by inspection with current capture IDs.
- Removed labels still leave a recognizable OATHYARD court/practice identity, not generic medieval/fighting arena.

### ORIG-02 — Low-poly non-native local raster style risks being mistaken for final style unless explicitly quarantined

Area: originality, readiness boundary
Severity: blocker for acceptance claims; major for implementation planning

Observed gap:

- Current captures are useful and more detailed than earlier baseline, but still visually read as low-poly/non-native local raster visualization with flat-shaded shapes and placeholder fighters.
- Without explicit quarantine, downstream work may incorrectly treat the look as a chosen stylized art direction rather than a local verification backend.

Canon/art conflict:

- Current blocked native-renderer status, non-native diagram, non-native frame, low-poly glTF, and non-native local raster captures are local verification evidence only (`GAME_CANON.md:11-15`, `DEMO_SCOPE.md:29-35`).
- High-fidelity gate requires production renderer/assets/materials/lighting/captures and owner acceptance separately (`ACCEPTANCE_MAP.md:32-40`).

Remediation directive:

- Add wording in downstream manifests/reports that the current arena capture style is `local_verification_software_raster`, not final production style.
- Keep clean distinction between:
  - structural identity acceptance for implementation routing;
  - product visual acceptance after renderer/assets/materials/owner review.
- Do not create a fake `production_visual_manifest.json` or equivalent production evidence unless real production-asset/owner evidence exists.

Acceptance evidence:

- `audit_readiness.sh` passes after any doc/manifest/status edits.
- Generated reports include false readiness flags and explicit blockers for owner/product renderer acceptance.

## 6. Developer-ready remediation sequence for `t_48a6a7f8`

1. Preserve truth boundary first.
   - Confirm changed paths are presentation/world/capture/asset-pipeline only unless a scenario/arena-selection truth change is explicitly required and hash/replay-covered.
   - Run `./tools/audit_truth.sh` after any code change.
2. Split arena shape language.
   - Verdict ring keeps circular/radial oath identity and gains stronger judgment axis/balcony/entry/witness/chain hierarchy.
   - Training yard loses the verdict-like dominant radial disc and gains rectangular lane/practice geometry, start boxes, rope/work-lamp/tool-zone vocabulary.
3. Add diegetic bounds and scale markers.
   - Replace debug ovals/golden rectangles as product evidence with low rim, chalk rings, rope/lanes/start marks, and visible scale relation to fighter silhouettes.
4. Add context/backdrop planes.
   - Verdict: court wall/balcony/witness plane.
   - Training: yard perimeter/tool/rack/lamp plane.
   - Keep central combat negative space clean.
5. Add clean-vs-debug capture split.
   - Product-like clean capture without construction guides.
   - Audit overlay capture may keep guides, hashes, and labels.
6. Regenerate and inspect current evidence.
   - Required: establishing/gameplay/contact for both arenas.
   - Required: direct image inspection, not manifest-only acceptance.
7. Keep readiness fail-closed.
   - Owner visual acceptance and all external release gates remain false.
   - If `./tools/verify.sh` remains blocked by missing production visual manifest, report the blocker exactly instead of fabricating production evidence.

## 7. Suggested verification commands after implementation changes

Focused arena/world commands:

```sh
python3 -m py_compile tools/asset_pipeline.py
bash -n tools/asset_budget_audit.sh tools/asset_visual_atlas.sh tools/arena_environment_captures.sh tools/native_combat_render.sh
./tools/build_assets.sh
./tools/validate_assets.sh <artifact-root>/validate_assets
./tools/asset_budget_audit.sh <artifact-root>/asset_budget
./tools/asset_visual_atlas.sh <artifact-root>/asset_atlas
./tools/arena_environment_captures.sh <artifact-root>/arena_captures
./tools/native_combat_render.sh examples/duels/basic_oathyard.duel <artifact-root>/native_combat
./tools/audit_3d_runtime.sh <artifact-root>/audit_3d_runtime assets/runtime_manifest.json <artifact-root>/native_combat/native_combat_render_manifest.json
./tools/audit_truth.sh
./tools/audit_readiness.sh . <artifact-root>/readiness
./tools/audit_secrets.sh . <artifact-root>/secrets
```

Full repo gates after code changes, if not blocked by known production-manifest gate:

```sh
./tools/build.sh
./tools/test.sh
cargo build --locked
cargo test --locked
./tools/verify.sh
```

If `./tools/verify.sh` fails because `./tools/validate_assets.sh` requires `assets/production_visual_manifest.json`, classify it as a missing production visual asset manifest / external production-asset evidence blocker. Do not fake that manifest from local non-native local raster captures.

## 8. Audit conclusion

The current verdict-ring/training-yard lane has crossed from “manifest-only arena tokens” into structured source-backed identity evidence: both arenas have source metadata, runtime glTF/mesh assets, material-map references, and 1920x1080 captures. The verdict ring and training yard now separate by cold/circular/judicial versus warm/framed/practice cues.

The remaining gaps are specific and implementable: split the shared radial floor language, make verdict-ring judicial hierarchy visual, make training-yard practicality visual, prove human scale with actual fighter silhouettes, replace debug-like bounds with diegetic bounds, add backdrop/context depth, keep combat readability clear of posts/radial clutter, and quarantine low-poly/non-native local raster evidence as local verification only.

This audit should unblock `t_48a6a7f8` as a concrete implementation task. It should not unblock owner visual acceptance or any public/release readiness gate.
