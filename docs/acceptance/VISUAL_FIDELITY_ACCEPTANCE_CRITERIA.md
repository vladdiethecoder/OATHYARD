# OATHYARD Visual Fidelity Acceptance Criteria

Status: acceptance criteria only; not implementation evidence; not owner acceptance.
Date: 2026-06-30T02:47:46Z
Kanban: t_a35eac12

## 0. Source hierarchy and hard boundary

This document translates the aspirational OATHYARD visual bars into testable gates for engineers and QA. It does not claim the bars are currently met.

Controlling sources, in precedence order:

1. `docs/design/GAME_CANON.md`
2. `docs/design/DEMO_SCOPE.md`
3. `ACCEPTANCE_MAP.md`
4. `AGENTS.md`
5. `docs/decisions/0007-high-fidelity-production-target.md`
6. `docs/design/ART_DIRECTION_BRIEF.md`
7. `docs/roadmap/HIGH_FIDELITY_PRODUCTION_WORK_ORDERS.md`
8. `docs/roadmap/HIFI_REMEDIATION_PHASE_PLAN.md`

Hard boundary:

- OATHYARD is a deterministic native-PC 3D planned-time physical melee duel game.
- Truth remains fixed 120 Hz, deterministic, integer/fixed-point, replayable, and hash-audited.
- Renderer, UI, audio, VFX, camera, fight-film, settings, accessibility, materials, post-processing, animation, and facial/cloth/cosmetic bones are presentation only unless a separate truth-promotion ADR passes the canon gates.
- Blocked native-renderer status, non-native diagram, non-native frame, non-native local raster captures, debug overlays, diagnostic image rollups, low-poly glTF, cubes/capsules/primitives, wireframes, and metadata-only checks are local verification evidence only. They cannot pass this document.
- Elden Ring and For Honor are quality/readability references only. OATHYARD must not copy their names, assets, silhouettes, factions, UI, animations, lore, characters, textures, music, sounds, or proprietary mechanics.
- Automated QA/media review can pass or fail the acceptance packet. Only the owner can set `owner_visual_acceptance:true`; legal/store/public-demo gates remain separate.

Machine-readable flags that must remain false until separately evidenced:

```text
production_renderer_complete: false
owner_visual_acceptance: false
public_demo_ready: false
release_candidate_ready: false
legal_clearance: false
trademark_clearance: false
store_readiness: false
```

## 1. Reference set and comparison rules

References are used to calibrate fidelity, atmosphere, readability, animation presentation, camera clarity, and performance expectations. They are not target content and are not a license to copy protected work.

### 1.1 Official / storefront reference sources

Reference IDs below should be used in visual benchmark packets and review sheets.

| Ref ID | Source | Use | URL |
| --- | --- | --- | --- |
| ER-OFFICIAL | Bandai Namco official Elden Ring site | Product identity/source provenance for Elden Ring reference | https://www.bandainamcoent.com/games/elden-ring |
| ER-TRAILER-2021 | ELDEN RING – Official Gameplay Trailer, Bandai Namco Entertainment America, uploaded 2021-06-10, 02:59 | Dark-fantasy atmosphere, lighting hierarchy, material richness, environment scale, boss/combat framing | https://www.youtube.com/watch?v=MUV5dqaumHE |
| ER-STEAM-APP | Steam app 1245620 storefront data, fetched 2026-06-30 | 1920x1080 screenshot references and PC requirement context | https://store.steampowered.com/app/1245620/ELDEN_RING/ |
| FH-OFFICIAL | Ubisoft For Honor official page, fetched 2026-06-30 | Current official mode descriptions, training/duel/brawl references, current media/news references | https://www.ubisoft.com/en-us/game/for-honor |
| FH-PC-SPECS | Ubisoft For Honor PC Specs and System Requirements Updated, 2017-02-13 | Official 1080p approximately 60 FPS high-preset reference point for melee game PC presentation | https://www.ubisoft.com/en-us/game/for-honor/news-updates/25Xa7RP2YnVsSSA0BUlSd4/for-honor-pc-specs-and-system-requirements-updated |
| FH-STEAM-APP | Steam app 304390 storefront data, fetched 2026-06-30 | 1920x1080 screenshot references and movie references | https://store.steampowered.com/app/304390/FOR_HONOR/ |

Selected image/video reference URLs from current storefront/API data:

| Ref ID | Category | URL |
| --- | --- | --- |
| ER-SS-00 | Elden Ring 1920x1080 screenshot | https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/1245620/ss_943bf6fe62352757d9070c1d33e50b92fe8539f1.1920x1080.jpg?t=1767883716 |
| ER-SS-01 | Elden Ring 1920x1080 screenshot | https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/1245620/ss_dcdac9e4b26ac0ee5248bfd2967d764fd00cdb42.1920x1080.jpg?t=1767883716 |
| ER-SS-02 | Elden Ring 1920x1080 screenshot | https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/1245620/ss_3c41384a24d86dddd58a8f61db77f9dc0bfda8b5.1920x1080.jpg?t=1767883716 |
| ER-SS-03 | Elden Ring 1920x1080 screenshot | https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/1245620/ss_e0316c76f8197405c1312d072b84331dd735d60b.1920x1080.jpg?t=1767883716 |
| ER-SS-04 | Elden Ring 1920x1080 screenshot | https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/1245620/ss_ef61b771ee6b269b1f0cb484233e07a0bfb5f81b.1920x1080.jpg?t=1767883716 |
| ER-SS-05 | Elden Ring 1920x1080 screenshot | https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/1245620/ss_b1b91299d7e4b94201ac840aa64de54d9f5cb7f3.1920x1080.jpg?t=1767883716 |
| FH-SS-00 | For Honor 1920x1080 screenshot | https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/304390/b44e9cec8aad36fbda39c5fc8ff31dd265aac213/ss_b44e9cec8aad36fbda39c5fc8ff31dd265aac213.1920x1080.jpg?t=1781283400 |
| FH-SS-01 | For Honor 1920x1080 screenshot | https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/304390/37c713982e81b3a94152f75ac4a645749e56787a/ss_37c713982e81b3a94152f75ac4a645749e56787a.1920x1080.jpg?t=1781283400 |
| FH-SS-02 | For Honor 1920x1080 screenshot | https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/304390/83da0e7bb47a3330b9802f043793ad50bde51f94/ss_83da0e7bb47a3330b9802f043793ad50bde51f94.1920x1080.jpg?t=1781283400 |
| FH-SS-03 | For Honor 1920x1080 screenshot | https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/304390/ss_4d71d0150ad7e55b8d3883dba083dc187a7ebfae.1920x1080.jpg?t=1781283400 |
| FH-SS-04 | For Honor 1920x1080 screenshot | https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/304390/ss_f999b373cb537dce1313fc5cb35f53ca1bcf6820.1920x1080.jpg?t=1781283400 |
| FH-SS-05 | For Honor 1920x1080 screenshot | https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/304390/15af9bb15d5e353136c58dbc654977ed9fb9aeef/ss_15af9bb15d5e353136c58dbc654977ed9fb9aeef.1920x1080.jpg?t=1781283400 |
| FH-Y10S2-GAMEPLAY | Ubisoft official current gameplay trailer entry on For Honor page, 2026-06-10 | Current melee presentation/media comparison | https://i.ytimg.com/vi/iFPcblUD7tg/maxresdefault.jpg |
| FH-WHAT-IS-FH | Steam For Honor movie reference, `What is For Honor` | Third-person melee/product framing comparison | https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/257012135/bcde451088eafac0f3d8d816248b9d49e3b71022/movie_600x337.jpg?t=1770752611 |

### 1.2 How to compare

Do not use pixel similarity or style-copy scoring. Use a paired evidence board:

1. Place OATHYARD captures beside reference images/clips by dimension: atmosphere/lighting, melee presentation, model/material detail, post-processing, UI/readability, and performance.
2. Score mechanisms, not surface content. Example: "OATHYARD has readable contact shadows and material response under arena lighting" is valid; "OATHYARD has the same golden tree / faction UI / attack indicator" is invalid.
3. Every OATHYARD visual claim must cite current OATHYARD capture IDs, hashes, renderer/backend ID, replay hash, asset manifest hash, and mediaqa/owner review row.
4. A reference comparison packet must include an originality/no-copying row. Failure of originality is blocking even if the image looks high quality.

## 2. Acceptance package prerequisites

A visual acceptance attempt is invalid unless all prerequisites below are present.

### 2.1 Required capture matrix

Current-run native production-renderer captures, not stale copied images, are required for:

- main menu;
- settings/accessibility;
- fighter select;
- loadout select;
- arena select when implemented;
- OATHYARD verdict-ring establishing shot;
- training-yard establishing shot;
- six fighter closeups;
- six armor/loadout family closeups;
- eight weapon family closeups;
- gameplay-distance view of every fighter/loadout family;
- gameplay-distance view of every weapon family in hand or shield-bearing context;
- planning timeline;
- pre-contact frame;
- contact frame;
- material/armor damage frame;
- injury/capability consequence frame;
- recovery/re-plan frame;
- first-person combat view;
- third-person combat view;
- fight-film/replay camera shot;
- replay verification UI or packet view;
- performance/debug overlay separated from player HUD.

### 2.2 Manifest fields

Every capture row must include:

```text
capture_id
source_command
git_identity_or_source_tree_stamp
renderer_backend_id
renderer_build_hash_or_binary_hash
quality_preset
resolution_width
resolution_height
native_resolution: true
upscaled_from_lower_resolution: false
capture_file_sha256
replay_path
replay_final_hash
content_manifest_hash
asset_manifest_hash
camera_mode
frame_or_tick
truth_mutation: false
production_renderer_complete: false until full gate passes
owner_visual_acceptance: false until owner explicitly accepts
```

### 2.3 Evidence minimums

- Images must be at least 1920x1080 native pixels. 1280x720, 1280x800, 960x540, upscaled captures, and cropped debug image rollups fail the high-fidelity gate.
- 2560x1440 captures are a stretch target where hardware/toolchain supports them; lack of 1440p is non-blocking only if 1080p native passes and public/demo requirements do not demand 1440p.
- Every capture must be generated after replay verification when it represents combat, fight-film, contact, injury, consequence, or replay UI.
- Every asset visible in product captures must be loaded from the production asset manifest, not a debug primitive fallback.
- Media QA must inspect actual image pixels or video frames. Metadata, manifests, file existence, OCR-only checks, and command exit code are insufficient.

## 3. Overall scoring model

Each dimension is scored as:

- `0 / FAIL`: blocks high-fidelity visual acceptance.
- `1 / CONDITIONAL`: not blocking for internal iteration, but blocks owner/public-demo readiness unless waived by owner for a named limited demo scope.
- `2 / PASS`: meets this document for the current packet.

High-fidelity visual acceptance requires:

- all hard gates in sections 2, 10, and 11 pass;
- every required dimension in sections 4-9 scores `2 / PASS`;
- no blocking issue from section 10 is open;
- `visual_benchmark_report.md` names exact current captures and reference IDs;
- mediaqa image/frame review is complete;
- owner acceptance is recorded separately if and only if the owner actually accepts.

An automated `2 / PASS` across all criteria still does not by itself set `owner_visual_acceptance:true`.

## 4. Atmosphere / lighting acceptance

Aspirational bar translated: "Elden Ring-class atmosphere" means OATHYARD achieves a premium dark-fantasy judicial-duel mood with rich depth, shadow, environmental storytelling, and readable combat silhouettes. It does not mean copying Elden Ring landscapes, icons, bosses, lore, colors, or silhouettes.

Primary references: ER-TRAILER-2021, ER-SS-00 through ER-SS-05.

### 4.1 Measurable thresholds

Hard pass requires all of the following:

1. Capture coverage
   - Verdict ring establishing shot at 1920x1080+.
   - Training yard establishing shot at 1920x1080+.
   - Gameplay-distance combat frame in each arena at 1920x1080+.
   - Closeup material/lighting sheet for metal, cloth, leather, wood, stone, flesh, and blood/wetness/wear families.
2. Lighting model
   - At least one authored key light or sun/moon equivalent, one fill/bounce/ambient term, and one grounding/contact-shadow/AO/equivalent term are visible in product captures.
   - Player feet, weapons, and low arena props cast or receive visible contact shadows/AO/equivalent grounding in gameplay-distance captures.
   - Fighters remain separable from background in all required combat states. Central playable-area subject/background luminance contrast must be visually adequate and should be measured by a reducer; if the subject silhouette cannot be segmented by a media reviewer without labels, fail.
3. Exposure and dynamic range
   - No central gameplay capture may be a dark void or whiteout. As an initial machine guard, reject if more than 8% of central 70% image pixels are clipped near-black (`luma <= 3/255`) or more than 3% are clipped near-white (`luma >= 252/255`), unless a mediaqa reviewer marks the clipped region as intentional non-playable sky/light source and gameplay silhouettes remain readable.
   - UI text and combat silhouettes must remain readable under the same exposure; separate debug relighting is not acceptance.
4. Atmosphere/depth
   - Verdict ring and training yard must show at least three depth planes: foreground playable surface, duel subject plane, and background architectural/environment plane.
   - Fog, dust, weather, volumetric-like atmosphere, or a documented equivalent must add depth without hiding feet, weapon arcs, contact surfaces, armor gaps, or consequence states.
5. Environment identity
   - Verdict ring reads as chalked stone judicial ritual space: low rim, oath/verdict marks, entry break or judgment axis, worn contact marks, and readable floor space.
   - Training yard reads as practice space: packed clay/stone/wood material, measured lines or targets, rope/post/work-lamp/training props, and clear start/footwork markers.
6. Originality
   - No copied Elden Ring environment landmarks, architecture silhouettes, bosses, UI, symbols, textures, or color compositions that create substantial similarity.

### 4.2 Pass checklist

- [ ] Both arenas have 1920x1080+ establishing captures.
- [ ] Both arenas have gameplay-distance combat captures.
- [ ] Lighting has visible key/fill/grounding components or documented equivalents.
- [ ] Contact shadows/AO/equivalent grounding exists under feet and weapons.
- [ ] Atmosphere creates depth but does not obscure combat evidence.
- [ ] Verdict ring and training yard are visually distinct without labels.
- [ ] No black-void/whiteout exposure failure in central playable region.
- [ ] No reference copying.

### 4.3 Fail examples

- Flat gray/black background with primitive fighters.
- Non-native frame/non-native diagram/X11/debug capture presented as mood evidence.
- One flat floor disk or noisy decals instead of authored arena identity.
- Dark-fantasy mood achieved by crushing detail into black.
- Fog/dust hides feet, weapon arcs, or contact.
- Reference-inspired content crosses into copied Elden Ring visual identity.

## 5. Melee animation and combat presentation acceptance

Aspirational bar translated: "For Honor-class melee presentation" means OATHYARD has third-person melee clarity, physical-feeling weapons, readable intent, contact, guard/bind states, and consequence transitions, while preserving OATHYARD's planned-time deterministic truth. It does not mean copying For Honor UI, guard widgets, factions, executions, movesets, animations, camera, or combat mechanics.

Primary references: FH-OFFICIAL, FH-PC-SPECS, FH-SS-00 through FH-SS-05, FH-Y10S2-GAMEPLAY, FH-WHAT-IS-FH.

### 5.1 Measurable thresholds

Hard pass requires all of the following:

1. Action coverage
   - Presentation states or clips cover every canonical action label from `GAME_CANON.md`: `step`, `pivot`, `guard`, `parry`, `cut`, `thrust`, `brace`, `bash`, `hook_bind`, `grab`, `shove`, `kick`, `recover`.
   - Required consequence states include bind/strain, stagger, collapse-risk or collapse, injury/capability change, grip loss, near miss/re-plan, and recovery.
2. Truth-to-presentation alignment
   - Combat captures map each visible attack/contact/consequence to replay id, tick/frame, trace event id, action label, weapon id, armor/material ids, and camera mode.
   - Visual contact may not appear before the authoritative contact/trace event.
   - At 60 FPS output, the visible contact/consequence should appear within 2 rendered frames of the intended presentation mapping from the verified replay tick. Larger offsets require a named latency/design explanation and mediaqa acceptance; hidden or premature contact fails.
3. Sequence evidence
   - For at least six representative exchanges, packet includes pre-contact, contact, post-contact/consequence, and recovery/re-plan frames.
   - Required exchange set must include: cut vs guard, thrust vs armor/gap, parry, bash/shove, hook_bind, kick/grab or close-body action, and one near miss.
4. Readability test
   - A reviewer must identify action label family, attacker/defender, active weapon family, guard/bind/contact state, and consequence state from frames without reading metadata.
   - Initial acceptance threshold: at least 90% correct across the required review sheet, and 100% correct for contact/no-contact and injury/capability consequence frames. Any reviewer confusion between contact and non-contact is blocking.
5. Motion quality
   - Planted-foot sliding in locked/brace/guard frames must be visually absent; engineering should measure foot drift and target less than 5 cm world-space or less than 2% of character height during planted intervals.
   - Grip alignment error should be below 3 cm for held weapons in idle/guard/strike frames, or below the visible tolerance accepted by mediaqa for stylized hands.
   - Armor/weapon/body clipping deeper than 2 cm in hero/combat camera views, or any clipping that changes contact readability, is blocking.
   - Weapon arcs, edges, points, hooks, blunt faces, and shields must remain visible through the action, not only in static closeups.
6. Camera readability
   - First-person, third-person, planning, consequence, and fight-film cameras preserve feet, hands/grips, weapon arcs, contact surface, armor gaps, and consequence UI/readouts.
   - Camera never hides the determinism evidence needed by the benchmark packet.
   - Camera motion has reduced-motion alternative and does not rely on blur to hide missing animation.
7. No truth contamination
   - Animation, IK, ragdoll-like presentation, cloth/strap motion, VFX, camera, and audio do not decide contacts, injuries, action costs, capability deltas, winners, hashes, or replay data.

### 5.2 Pass checklist

- [ ] Every canon action label has a presentation state/clip or documented equivalent.
- [ ] Every visible contact/consequence maps to verified replay/trace event ids.
- [ ] No visible contact occurs before truth contact.
- [ ] Required exchange sheet exists: cut, thrust, parry, bash/shove, hook_bind, kick/grab, near miss.
- [ ] Reviewers can identify action/contact/consequence without metadata.
- [ ] Foot, grip, armor, and weapon clipping tolerances pass.
- [ ] Cameras preserve feet/weapon/contact/armor gaps.
- [ ] Presentation systems leave replay/truth hashes unchanged.

### 5.3 Fail examples

- Text labels or HUD logs carry the combat while bodies remain static.
- Canned hit animation pre-decides outcome before truth contact.
- Weapon appears to hit but trace says miss, or trace says contact but frame hides it.
- Camera crops feet/weapon arcs or hides armor gaps during contact.
- For Honor guard/UI/animation language is copied instead of independently designed.

## 6. 3D model fidelity acceptance

Aspirational bar translated: "production visual quality" means high-detail, source-backed, readable, riggable, materially rich OATHYARD models that survive closeup, gameplay, and combat-context review. It does not mean high vertex count alone.

Primary references: ART_DIRECTION_BRIEF, ER-SS-00 through ER-SS-05 for atmosphere/material scale, FH-SS-00 through FH-SS-05 for melee character/weapon readability.

### 6.1 Coverage floor

The packet cannot pass unless it includes production-lane assets for:

- six fighter traditions: `saltreach_duelist`, `oathyard_writ`, `chainbreaker`, `reed_sentinel`, `gate_shield`, `bruiser_oath`;
- six armor/loadout families: `gambeson`, `mail_hauberk`, `heavy_plate`, `lamellar`, `fencer_light`, `bruiser_padded_plate`;
- eight weapon families: `curved_sword`, `longsword`, `bearded_axe`, `ash_spear`, `round_shield`, `iron_maul`, `arming_sword`, `billhook`;
- two arenas: `oathyard_verdict_ring`, `training_yard`.

### 6.2 Source/provenance thresholds

Every production asset must provide:

- authoring source under `assets_src/` or documented successor source directory;
- repo-owned or licensed provenance record;
- author/tool/version record;
- source hash and runtime export hash;
- runtime manifest entry;
- validation result;
- isolated preview, gameplay-distance capture, and in-engine/product-renderer capture;
- material ids/maps/equivalent fields;
- collision/contact/material metadata where relevant;
- readiness/owner flags remaining false until accepted by the proper gate.

Generated or external assets are draft candidates only until they pass the same source/provenance/art/validation/visual gates.

### 6.3 Geometry/detail thresholds

These are initial high-fidelity floors, not quality ceilings. Passing them does not guarantee visual acceptance; failing them blocks unless an owner-approved renderer/asset ADR documents an equivalent technique such as displacement, virtualized geometry, or shader-authored high-frequency detail with direct closeup proof.

| Asset class | Initial technical floor | Visual requirement |
| --- | --- | --- |
| Fighter body LOD0 | >= 40k visible triangles per unclothed/underclothed fighter source, or equivalent sculpt/displacement detail with in-engine proof | Credible head, hand, shoulder, hip, knee, foot forms; readable stance and tradition silhouette at gameplay distance |
| Fighter with loadout/armor LOD0 | >= 75k visible triangles combined fighter+wearables in closeup, or equivalent | Layered silhouette, no merged arm/strap/weapon blobs, readable stance/weapon/armor bulk |
| Armor family | >= 25k visible triangles per wearable family or equivalent material/geometry detail | Coverage/gaps/straps/buckles/mail/plate/lamellar/quilted channels readable in closeup and gameplay distance |
| Weapon family | >= 5k visible triangles for swords/spears; >= 8k for axe/shield/maul/billhook or equivalent | Edge/blunt/pierce/hook/grip/reach/mass identity readable; no slab/post/block/stub read |
| Arena visible scene | >= 250k visible triangles/instances or equivalent virtualized/instanced/displacement environment detail | Verdict ring and training yard have authored landmarks, floor material, props, lighting anchors, playable clear space |
| LOD behavior | LOD0 closeup, LOD1 gameplay-distance, lower LODs only when they preserve silhouette/action readability | No visible LOD pop exceeding 2% of screen height for hero subjects in combat cameras |

A lower-poly stylized choice can only pass if mediaqa and owner agree it meets production visual quality at the target resolution. Current 22-asset/292-vertex/492-triangle evidence is far below this floor and remains Tier 0 debug-local verification.

### 6.4 Material thresholds

Required material families:

- flesh/tendon/bone/skin variation;
- cloth/quilted linen;
- mail/ring pattern;
- tempered plate/blackened iron/steel;
- lamellar iron/leather;
- ash wood;
- chalk stone;
- buff leather/textile;
- dirt, soot, blood, wetness, scratches, dents, wear.

Required channels or documented equivalents:

- base color/albedo;
- roughness;
- metallic where applicable;
- normal or height detail;
- AO/cavity/curvature or equivalent grounding;
- material ID masks;
- localized damage/wear/blood/wetness masks keyed to truth-after-hash events.

Initial texture/detail floors:

- hero fighter/armor/weapon closeup material sets should provide 2k-class or better texture/detail density, or procedural/equivalent detail that resolves at 1920x1080 closeup without blur/noise.
- arena hero surfaces should provide 4k-class tiling/detail density or procedural/equivalent material detail for close camera surfaces.
- gameplay-distance assets must remain readable without labels; recolor-only differentiation fails.

### 6.5 Rig/deformation thresholds

- Every fighter has real skeleton hierarchy, non-identity bind pose, truth-joint mapping, and cosmetic-only bone separation.
- Wearable armor is skinned or socketed with attachment offsets.
- Required pose/no-clipping evidence covers idle, walk/step, pivot, guard, parry, cut, thrust, brace, bash, hook_bind, grab/shove/kick where applicable, recover, stagger/collapse/injury consequence.
- Shoulder/elbow/hip/knee deformation must not visibly collapse or candy-wrapper in required action views.

### 6.6 Pass checklist

- [ ] All six fighters, six armor families, eight weapon families, and two arenas are present in production-lane packet.
- [ ] Every asset has source, provenance/license, hashes, runtime export, manifest entry, validation, preview, and in-engine capture.
- [ ] Geometry/detail floors or documented equivalent proof pass.
- [ ] Material families and channels/equivalents pass.
- [ ] Gameplay-distance silhouettes identify fighter, weapon, armor, and arena without labels.
- [ ] Closeups show real detail; no flat recolor-only families.
- [ ] Rig/deformation/no-clipping evidence exists for fighters and wearables.
- [ ] Generated/external asset candidates have not been laundered into production acceptance.

## 7. Post-processing and renderer presentation acceptance

Aspirational bar translated: production visual quality includes antialiasing, tone mapping, exposure, shadow quality, temporal stability, legibility-preserving effects, and artifact-free presentation. Post-processing must support combat readability, not hide missing art.

### 7.1 Measurable thresholds

Hard pass requires:

1. Antialiasing / image stability
   - Product captures show no severe jagged silhouette stair-stepping on fighters/weapons/arena rims at 1920x1080.
   - Temporal antialiasing or equivalent must not create ghost trails that confuse weapon arcs, contact point, guard state, or UI text.
   - Thin weapons such as spear/billhook/sword edges remain visible at gameplay distance without shimmer that changes apparent reach.
2. Shadows / AO / grounding
   - Contact shadows/AO/equivalent grounding visible for feet, shields, weapons near floor, and low props.
   - Shadow acne, peter-panning, detached feet, and floating bodies are blocking when visible in required captures.
3. Tone mapping / color grading
   - Dark-fantasy grade preserves material identity: blackened iron still has edge detail; dark cloth is not a blob; chalk/stone is not whiteout.
   - Injury/blood/wetness material cues remain visible but do not obscure silhouette/action evidence.
4. Motion blur / depth of field
   - Motion blur and depth of field are off or minimal for benchmark capture unless also accompanied by clean capture variants.
   - Effects cannot hide clipping, bad deformation, missing contact, or weak weapon arc readability.
5. Bloom / glow / VFX compositing
   - Bloom/emissive effects do not wash out weapons, UI text, or contact/injury evidence.
   - VFX are depth-buffered or otherwise integrated into the world; flat UI glyphs/proxy bars cannot masquerade as production VFX.
6. UI preservation
   - Post-processing does not blur or low-contrast critical UI, accessibility settings, captions, frame-cost deltas, replay verification text, or performance overlay.
7. Truth isolation
   - Renderer/post-processing toggles leave replay JSON, trace JSON, contact packets, action costs, capability deltas, end condition, content hash, and final hash byte-identical.

### 7.2 Pass checklist

- [ ] Antialiasing or equivalent makes silhouettes stable at 1920x1080.
- [ ] Thin weapons remain readable without shimmer or disappearing edges.
- [ ] Shadows/AO/equivalent ground subjects and props.
- [ ] Tone mapping preserves material detail in dark and bright regions.
- [ ] Clean variants exist for any blurred/DOF/film-grain benchmark shots.
- [ ] VFX are world-integrated, not flat proxy glyphs.
- [ ] UI/captions/settings/replay verification remain legible.
- [ ] Presentation toggles do not mutate truth/replay data.

## 8. Resolution targets

### 8.1 Required targets

| Target | Required for | Acceptance |
| --- | --- | --- |
| 1920x1080 native | All required high-fidelity visual acceptance captures | Hard requirement. No upscaling from lower resolution. |
| 1920x1200 or 1280x800 | Steam Deck / handheld-oriented later evidence | Separate hardware/input gate; does not replace 1920x1080 visual acceptance. |
| 2560x1440 native | Stretch evidence where hardware/toolchain supports it | Non-blocking for internal gate unless owner/public-demo scope requires it. |
| 3840x2160 native or supersampled stills | Marketing/store/key-art candidate | Not required for first visual acceptance, but store asset work requires separate gate. |

### 8.2 Resolution pass criteria

- Capture files report actual pixel dimensions matching target.
- Manifest records resolution and `upscaled_from_lower_resolution:false`.
- UI scale remains readable at 1920x1080 with default settings and at large-text accessibility setting.
- No capture is cropped so tightly that required combat evidence is missing.
- Reference comparison sheets maintain original aspect ratio; no stretching or non-uniform scaling.

### 8.3 Fail examples

- 1280x720 frame resized to 1920x1080.
- Asset closeup at 1920x1080 but gameplay/contact frame at 1280x720.
- Debug image rollup composed at 1920x1080 from lower-resolution tiles and presented as product capture.
- UI readable only because of debug overlay zoom, not in real product frame.

## 9. Frame-rate and performance targets

For Honor's official PC spec page lists a recommended target of 1080p at approximately 60 FPS on High preset. OATHYARD's first visual acceptance target should meet or exceed 1920x1080 high-fidelity presentation at stable 60 FPS on the current target development PC class, while preserving deterministic 120 Hz truth.

No final minimum hardware spec is defined here. This document sets the visual-acceptance packet threshold; public system requirements require a later packaging/platform gate.

### 9.1 Required measurements

Performance report must separate:

- simulation truth step time at fixed 120 Hz;
- render frame time;
- capture/export time, clearly labeled as non-interactive when applicable;
- startup/load time;
- memory usage estimate;
- package size delta;
- renderer backend ID and quality preset;
- resolution;
- host CPU/GPU/driver/runtime facts.

### 9.2 Interactive frame-time thresholds

For the initial high-fidelity visual acceptance packet at 1920x1080:

| Metric | Pass threshold | Blocking threshold |
| --- | --- | --- |
| Median render frame time | <= 16.7 ms | > 20 ms |
| 95th percentile render frame time | <= 20 ms | > 25 ms in combat/planning/fight-film product views |
| 99th percentile render frame time | <= 33.3 ms | > 50 ms or visible hitching during melee/contact views |
| Single-frame stall | No stall > 100 ms during a 5-minute representative run after loading | Any repeated >100 ms stalls in product loop, or one stall that hides contact/consequence evidence |
| Truth step | Fixed 120 Hz truth remains deterministic; render timing never feeds truth | Any render/UI/audio/VFX/camera timing affects contacts, costs, capabilities, end state, or hashes |
| Startup to first interactive menu | <= 10 s target on target dev machine after cold start | > 20 s without loading screen/progress explanation for demo scope |

If the renderer is not yet interactive and only produces offline captures, the packet may not claim interactive performance. Offline capture throughput can be reported only as artifact generation speed.

### 9.3 Performance pass checklist

- [ ] 1920x1080 high-fidelity run has frame-time distribution, not just nominal FPS.
- [ ] Performance report separates simulation, rendering, and capture/export time.
- [ ] Truth replay outputs are byte-identical with renderer/performance overlay enabled/disabled.
- [ ] No combat-critical hitch hides contact, parry, bind, injury/capability, or UI consequence evidence.
- [ ] Host hardware/driver/toolchain facts are recorded.
- [ ] No public minimum/recommended PC spec is claimed from this internal packet alone.

## 10. Blocking vs non-blocking visual issues for demo readiness

This section defines visual blockers for native public-demo readiness. Public-demo readiness also requires owner, legal, trademark, license, package, store/distribution, and demo-scope gates; passing visuals alone is not enough.

### 10.1 Blocking issues

Any item below blocks high-fidelity visual acceptance and native public-demo readiness:

1. Evidence/source blockers
   - Required capture missing.
   - Capture below 1920x1080, upscaled from lower resolution, stale, unhashable, or not from current executable/assets.
   - Capture is blocked native-renderer status/non-native frame/non-native diagram/debug/non-native local raster/low-poly/primitive evidence instead of production renderer output.
   - Manifest lacks replay hash, content/asset hashes, renderer/backend id, command, resolution, or capture file hash.
   - Media QA did not inspect actual pixels/frames.
2. Readiness honesty blockers
   - `owner_visual_acceptance`, `public_demo_ready`, `release_candidate_ready`, `legal_clearance`, `trademark_clearance`, or `store_readiness` is true without external evidence.
   - Automated review is presented as owner acceptance.
   - Local package gate is presented as high-fidelity/product visual acceptance.
3. Renderer/post blockers
   - No continuous native player-facing render loop or legally approved engine path for demo context.
   - Severe jaggies, flicker, ghosting, black voids, whiteouts, shadow acne/peter-panning, floating subjects, or unstable exposure in required product views.
   - Post-processing hides missing animation, clipping, contact, or unreadable materials.
4. Asset blockers
   - Placeholder primitives/cubes/capsules/flat planes/low-poly text-generated blockouts visible in product captures.
   - Missing production-lane coverage for any required fighter, armor, weapon, or arena.
   - Asset source/provenance/license/hash records missing.
   - Weapon/armor/fighter silhouette cannot be identified without labels at gameplay distance.
   - Materials are flat recolor-only or read as noise/black blobs.
   - Armor/weapon/body clipping changes combat readability.
5. Combat presentation blockers
   - Contact/no-contact is visually ambiguous.
   - Visible contact contradicts replay/trace event data.
   - Canon action intent, guard/bind state, injury/capability consequence, or recovery cannot be identified in required frames.
   - Camera hides feet, weapon arcs, contact surfaces, armor gaps, or consequence evidence.
   - Animation/VFX/audio/camera decides or mutates truth outputs.
6. Performance blockers
   - Interactive product loop cannot sustain the frame-time thresholds in section 9 for required combat/planning/fight-film views.
   - Renderer stalls repeatedly during melee or hides contact/consequence evidence.
   - Performance report collapses artifact-generation time into interactive FPS.
7. Originality/IP blockers
   - Copied or substantially similar Elden Ring/For Honor assets, silhouettes, UI, animations, lore, music, textures, names, factions, or proprietary mechanics.
   - Unlicensed asset source or unknown provenance in production packet.

### 10.2 Non-blocking issues

Items below can be logged as non-blocking only when every hard gate and all combat-critical readability criteria pass:

- Minor texture seam visible only in extreme closeup and not in gameplay/contact/fight-film views.
- Minor LOD pop outside combat focus that does not change silhouette/action/contact readability.
- Small isolated prop clipping outside playable/combat area.
- Optional 2560x1440/4K capture missing while all 1920x1080 required captures pass and owner scope does not require higher resolution.
- Subtle material tuning issue that does not affect material family identification, contact consequence, or art direction.
- One non-critical decoration/prop below target if not visible in required demo paths and logged as a follow-up.
- Minor UI alignment issue outside core menu/settings/planning/consequence/replay/performance surfaces.

Non-blocking does not mean ignored. Every non-blocking issue must have a follow-up id, capture id, severity, owner, and acceptance note.

## 11. Dimension pass/fail checklists

### 11.1 Atmosphere / lighting

Pass requires:

- [ ] ER reference comparison uses ER-OFFICIAL/ER-TRAILER/ER-SS only as quality references, not copied content.
- [ ] Both arenas have 1920x1080+ establishing and gameplay captures.
- [ ] Lighting includes visible key/fill/grounding or documented equivalent.
- [ ] Shadows/AO/grounding visible under players and props.
- [ ] Atmosphere/depth exists and preserves combat readability.
- [ ] Exposure and contrast avoid black-void/whiteout failure.
- [ ] Verdict ring and training yard have distinct OATHYARD identity.

Fail if any are missing or if mood is created by hiding detail.

### 11.2 Melee animation / combat presentation

Pass requires:

- [ ] All canon action labels have presentation states/clips/equivalents.
- [ ] Six representative exchange sequences include pre/contact/post/recovery frames.
- [ ] Contact/consequence frames map to replay/trace event ids.
- [ ] Blind readability threshold passes: 90% overall, 100% contact/no-contact and injury/capability.
- [ ] Foot/grip/clipping tolerances pass.
- [ ] First-person, third-person, planning, consequence, and fight-film cameras preserve evidence.
- [ ] No copied For Honor UI/combat language.

Fail if a trace/contact contradiction appears or if text logs carry the fight instead of bodies, weapons, materials, and camera.

### 11.3 3D model fidelity

Pass requires:

- [ ] Six fighters, six armor/loadouts, eight weapons, and two arenas are covered.
- [ ] Every asset has source/provenance/hash/runtime export/validation/captures.
- [ ] Geometry/detail floors or documented equivalents pass.
- [ ] Materials use PBR/equivalent channels and family identities.
- [ ] Gameplay-distance silhouette identification passes without labels.
- [ ] Closeups show real authored detail, not generator noise or flat recolor.
- [ ] Rig/deformation/no-clipping evidence passes.

Fail if any visible production asset is a placeholder primitive, low-poly debug asset, unsupported candidate, or unlicensed/unknown provenance asset.

### 11.4 Post-processing

Pass requires:

- [ ] Stable antialiasing/image quality at 1920x1080.
- [ ] No temporal ghosting that changes weapon/contact read.
- [ ] Contact shadows/AO/grounding pass.
- [ ] Tone mapping preserves dark material detail and bright UI/text detail.
- [ ] Clean variants exist for blurred/DOF/filmic shots.
- [ ] VFX are world-integrated and trace-driven.
- [ ] UI/captions/accessibility remain legible.
- [ ] Post-process toggles do not mutate truth/replay outputs.

Fail if post hides missing work or creates combat ambiguity.

### 11.5 Frame-rate / resolution

Pass requires:

- [ ] Every required capture is native 1920x1080+ and hash-verified.
- [ ] No upscaled 720p/800p/960p capture is counted.
- [ ] Frame-time distribution passes section 9 thresholds for interactive product loop.
- [ ] Simulation 120 Hz truth and render timing are separated.
- [ ] No combat-critical hitch.
- [ ] Host/toolchain facts are recorded.

Fail if offline capture throughput is claimed as interactive FPS or if render timing affects truth.

## 12. Exact QA workflow

Recommended command shape after the production renderer/assets exist:

```sh
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
root="artifacts/visual_acceptance/${stamp}"

./tools/run_duel.sh examples/duels/basic_oathyard.duel --out "${root}/duel"
./tools/replay_verify.sh "${root}/duel/replay.json"

./tools/presentation_truth_isolation.sh "${root}/truth_isolation"
./tools/build_assets.sh
./tools/validate_assets.sh
./tools/render_asset_previews.sh "${root}/asset_previews"
./tools/capture_high_fidelity_screens.sh "${root}/captures"
./tools/visual_evidence_index.sh "${root}/visual_evidence"
./tools/visual_benchmark.sh "${root}/visual_benchmark"
./tools/audit_readiness.sh . "artifacts/readiness/visual_acceptance_${stamp}"
```

If any tool is intentionally fail-closed before production renderer/assets exist, report the failure as current evidence. Do not weaken the gate.

### 12.1 Required mediaqa review packet

Packet root should contain:

```text
visual_acceptance_manifest.json
visual_reference_manifest.json
capture_manifest.json
visual_benchmark_report.md
native_3d_visual_capture_manifest.json
per_capture_reviews/*.md or .json
failed_visual_artifacts.txt
owner_review_checklist.md
readiness_boundary.md
```

### 12.2 Review prompts / rows

Each image/frame review row should answer:

- What capture id, replay hash, asset hash, renderer id, resolution, and camera mode is this?
- Which reference IDs are relevant for quality/readability comparison?
- Is the OATHYARD content original and not copied?
- Can the reviewer identify fighter, weapon, armor, arena, action, guard/contact state, and consequence without metadata?
- Are material families visually distinct?
- Are lighting, atmosphere, shadows, and post-processing supporting rather than hiding evidence?
- Are any blockers from section 10 present?
- Is the finding `PASS`, `CONDITIONAL`, or `FAIL`?

## 13. Owner visual acceptance checklist

The owner review artifact must be explicit and separate from automated QA.

Owner checklist rows:

- [ ] I reviewed current native 1920x1080+ captures, not stale/debug evidence.
- [ ] I reviewed atmosphere/lighting.
- [ ] I reviewed melee animation/combat presentation.
- [ ] I reviewed 3D model fidelity and materials.
- [ ] I reviewed post-processing/image stability.
- [ ] I reviewed frame-rate/resolution/performance evidence.
- [ ] I reviewed UI/readability/accessibility where included.
- [ ] I reviewed originality/no-copying and provenance notes.
- [ ] I accept the current visual packet for the named scope, or I reject it with blocker notes.

Until a real owner response artifact exists, the correct state is:

```text
owner_visual_acceptance: false
native_public_demo_ready: false
public_demo_ready: false
release_candidate_ready: false
```

## 14. Summary pass standard

A packet may be called `high-fidelity visual criteria passed for mediaqa review` only when:

1. all required current-run native 1920x1080+ captures exist and hashes verify;
2. all visible production assets are source-backed, provenance-recorded, validated, and rendered in product context;
3. atmosphere/lighting, melee presentation, model fidelity, post-processing, frame-rate, and resolution all score `2 / PASS`;
4. no blocking issue remains;
5. mediaqa inspected actual pixels/frames;
6. readiness flags remain honest.

A packet may be called `owner visually accepted` only when the owner explicitly accepts the current packet in a durable artifact.

A packet may be called `native public demo ready` only when high-fidelity visuals, owner demo-scope acceptance, local package gate, legal/trademark/license gates, and approved distribution path all pass. Visual acceptance alone is not public-demo readiness.
