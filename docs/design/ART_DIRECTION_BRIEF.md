# OATHYARD Art-Direction Brief for Game Models

Status: production art target, not visual-completion evidence.
Date: 2026-06-29

## Source basis

Canon precedence for this brief is `docs/design/GAME_CANON.md`, `docs/design/DEMO_SCOPE.md`, `ACCEPTANCE_MAP.md`, then ADRs and asset-source files. No separate mood-board file was found in the repository; the current source anchors are the canon/ADR language, `assets_src/` text assets, and the visual gap report.

Key controlling constraints:

- OATHYARD is a deterministic native-PC 3D planned-time physical melee duel game.
- The target is premium current-generation high-fidelity native 3D with dark-fantasy judicial-duel art direction and melee readability benchmarked against Elden Ring and For Honor for quality/readability only. Do not copy their names, assets, silhouettes, factions, UI, animation, lore, characters, textures, music, or proprietary mechanics.
- Current raw X11/XWayland, SVG, PPM, low-poly glTF, software-raster captures, diagnostic contact sheets, and sandbox-regenerated model candidates are local verification or QA evidence only. They prove pipeline/data-flow boundaries or expose blockers; they are not visual target evidence and are not the production model style.
- Production assets must be repo-owned, source-backed, provenance-tagged, regenerable from `assets_src/`, non-flat 3D geometry, and suitable for rigging/skinning without mutating deterministic gameplay truth.
- Current QA blocker source for the next model pass: `/home/vdubrov/.hermes/kanban/boards/oathyard-full-game/workspaces/t_3388ac99/qa_results/OATHYARD_MODEL_VISUAL_ANIMATION_QA_REPORT.md`.

## Target look

Use **grounded stylized realism**: high-detail, tactile, grim judicial-fantasy models with intentionally readable combat shapes. This is not cute low-poly, not flat cel-shading, and not generic photoreal medieval kit. Models should feel like ceremonial duelists whose gear is also evidence in a court of violence: oaths, verdict marks, chalked rings, worn mail, strapped plate, ash wood, leather ties, blood/dirt/wetness, and visible consequences of physical contact.

The camera and gameplay need instant recognition of action intent, weapon reach, guard state, armor coverage, injury/capability changes, and contact outcomes. Therefore stylization should simplify only where it improves melee readability; material and silhouette detail should remain production-grade.

## Style anchors for 3D artists

1. **Silhouette language: readable duel intent before ornament.**
   - Every fighter must read from the gameplay camera by stance, weapon mass/reach, and armor bulk.
   - Preserve the existing six tradition silhouettes from `assets_src/fighters/traditions.oysrc`: lean-forward Saltreach Duelist, balanced Oathyard Writ, wide-hooking Chainbreaker, long-reed-guard Reed Sentinel, shielded-low Gate Shield, and heavy-maul Bruiser Oath.
   - Push asymmetry only when it communicates function: dominant weapon side, shield shoulder, cloak/scabbard side, maul weight, spear line, hook profile, damaged armor side.
   - Avoid noisy heroic fantasy spikes/wings that obscure weapon arcs, gaps, or readable contact surfaces.

2. **Palette and value: cold oath stone plus faction accents.**
   - Base world palette: cold chalk stone, iron black, tarnished steel, ash wood, buff leather, quilted linen, dried blood, dark wet earth, and muted oath-red/chalk-white markings.
   - Fighter accents come from current source visuals: inked salt sash, black oath tabard, split-chain badge, ash pole wrap, chalked round shield, iron oath mask.
   - Keep playable state readability above realism: weapon edge, shield face, exposed gaps, active guard, injury, and capability loss must remain visible under arena lighting.
   - Saturation should be restrained; color should mark legal/duel identity and gameplay affordances, not arcade stats.

3. **Surface treatment: tactile physical materials.**
   - Required material families: flesh/tendon/bone/cloth, quilted linen padding, riveted mail, tempered plate, lamellar iron/leather, ash wood, stone, buff leather/textile, padded plate mix.
   - Production models need PBR or equivalent material response: roughness/metalness/normal/AO or documented analogs; edge wear, dents, seams, straps, buckles, dirt, sweat, blood, wetness, stitch lines, mail rings, plate bevels, and leather compression.
   - Weapon contact profiles must be visible in geometry: edge, blunt, pierce, hook, grip, reach line. A billhook must visibly hook/bind; a maul must visibly crush; a spear must visibly brace and pierce.
   - Damage/wear masks are presentation-only and must be driven by hashed truth events after gameplay hashes, never by renderer-side truth mutation.

4. **Proportions and anatomy: grounded humans, combat-expressive gear.**
   - Human bodies should be credible at close-up: face/head/hand silhouettes where visible, believable shoulders/hips/knees, weight-bearing feet, and riggable deformation zones.
   - Stylization may exaggerate armor/weapon read by 5–15% where it clarifies stance, reach, or mass, but not enough to imply impossible physics or hide truth-relevant contact surfaces.
   - Separate truth joints from cosmetic bones. Canon truth joints remain root/spine/head/shoulders/elbows/wrists/hips/knees/ankles plus grip frames; cosmetic cloak/scabbard/strap bones must stay presentation-only.
   - Model armor as layered pieces with gaps: gambeson stitching, mail hauberk rings/coif, heavy-plate hinges, lamellar tiles, fencer trim, padded-plate blocks. Gaps and coverage must be legible.

5. **Arena and motion framing: judgment ritual, not arena noise.**
   - Main arena target: OATHYARD verdict ring, chalked stone, low rim, north judgment balcony, high cold oath-mark lighting. Training arena target: packed clay, measured practice lines, rope/work-lamp practicality.
   - Environments should frame the duel and show material storytelling without hiding feet, weapon arcs, contact, recovery, or consequence states.
   - Animation presentation should look physically committed: weight shifts, bracing, recovery cost, binding strain, armor drag, stagger/collapse. Canned hits are forbidden as truth; animation must present truth events.

## Immediate production target for model work

For the next model-production pass, create source-backed high-detail 3D assets for the six fighter/loadout families, eight weapon families, six armor families, OATHYARD verdict ring, and training yard. Each asset should ship with source files, provenance notes, runtime export, material definitions, preview/closeup renders, rig/topology notes, per-asset QA fix notes, and explicit truth-boundary notes. Poly and texture budgets should be set by the technical model spec, but art should not regress to the current local-evidence baseline.

The next pass must resolve the QA blockers below before any "model set accepted" language is used. A generator or modeler should treat these notes as required silhouette/material targets, not optional polish.

Do not label any model set as high-fidelity complete, owner accepted, public-demo ready, or release-candidate ready until current-run native captures, visual benchmark inspection, and owner visual acceptance exist.

## QA-driven model revision targets

Source blocker report: `/home/vdubrov/.hermes/kanban/boards/oathyard-full-game/workspaces/t_3388ac99/qa_results/OATHYARD_MODEL_VISUAL_ANIMATION_QA_REPORT.md`.

These targets revise the art/model direction for the next source-backed generation or DCC pass. They do not accept the sandbox candidate package. They define what a developer/model generator must encode in source metadata, mesh generation, material assignment, and visual QA captures.

### Cross-asset silhouette and material rules

- Primary identity must survive a gameplay-distance contact sheet: each fighter, weapon, armor family, and arena must be identifiable by outline before reading labels or metadata.
- Preserve functional negative space: arms must separate from shoulders/straps, weapon heads must separate from shafts, armor pieces must separate from cloth layers, and arena landmarks must not merge into floor noise.
- Do not differentiate families by flat recolor alone. Use geometry, edge highlights, normals, straps, rows, lacing, rivets, bosses, fullers, hooks, bevels, dents, stitched channels, and material masks.
- Random red/dark scatter is not material identity. Blood, grit, soot, chalk, and wear must be localized to contact/wear zones and must not obscure the silhouette or make artifacts look like generator noise.
- Metal treatment must be unified: iron/steel surfaces need readable edge/rim highlights, bevels, roughness variation, and worn contact faces. Flat black metal is allowed only when explicitly authored as blackened iron and still must carry highlights.
- Fighters require authored idle/walk/attack presentation clips or pose sets, a real skeleton hierarchy, non-identity bind pose, and blended shoulder/elbow/hip/knee deformation proof before animation-readiness can be claimed.
- Wearable armor must either be skinned to the fighter rig or have explicit sockets/attachment offsets plus captured idle/walk/attack no-clipping proof. Static armor previews alone are not wearable acceptance.

### Fighter/loadout fixes

| Asset | Required silhouette/material target for next pass |
| --- | --- |
| `saltreach_duelist` | Preserve the lean-forward duelist stance and inked salt sash, but add the same real rig hierarchy, bind pose, blended deformation, and idle/walk/attack proof required for every fighter. Do not add shoulder bulk that hides the light, forward-reading profile. |
| `oathyard_writ` | Preserve the balanced black-oath-tabard identity, with clear tabard-over-cloth layering and readable waist/shoulder separation. Add real rig/deformation/motion proof; avoid becoming the default generic body. |
| `chainbreaker` | Narrow the over-wide shoulder block and carve visible neck/arm negative space. The split-chain badge and hooked/wide stance should read through asymmetrical chain/strap placement, not through a single broad shoulder slab. Straps must clear arms in guard/cut poses. |
| `reed_sentinel` | Preserve the long-reed-guard reach line and ash pole wrap. Increase value contrast between pole/cloth/body where needed, and prove two-hand spear/pole grip alignment through idle/walk/attack or guard/thrust poses. |
| `gate_shield` | Separate shoulder, shield strap, and shield disc silhouettes: visible shield rim/boss/inner grip, strap path across torso or forearm, and clean air gaps around upper arm. The shielded-low profile must not merge into a muddy shoulder mass. |
| `bruiser_oath` | Reduce the oversized shoulder/proportion breakdown while keeping heavy-maul mass and iron-oath-mask identity. Use low, dense torso/forearm mass, wide belts, mask, and maul weight rather than balloon shoulders. Must pass arm-swing clearance without clipping. |

### Weapon fixes

| Asset | Required silhouette/material target for next pass |
| --- | --- |
| `curved_sword` | Make the curved single-edge identity unmistakable: a continuous crescent-like cutting line, separated spine/edge highlights, readable guard/pommel, and enough blade length/thickness that it cannot read as a stubby dagger or wedge. |
| `longsword` | Distinguish from `arming_sword` by two-handed scale: longer straight double-edge blade, fuller or center ridge, longer grip, larger cruciform guard, and slimmer reach-forward profile. It must read as 1220 mm reach at contact-sheet scale. |
| `bearded_axe` | Keep the strong bearded hook silhouette, but replace flat-black head/outlier triangle artifacts with unified worn steel/blackened-iron material: bevel highlight on cutting edge, darker poll/socket, clear beard edge, and no stray gray geometry that reads as shadow trash. |
| `ash_spear` | Strengthen the long-thin read without losing elegance: thicker/value-separated ash shaft, leaf head wide enough to see, metal socket/collar, two grip wraps, and reduced scatter near the point so the pierce/bracing line remains clean. |
| `round_shield` | Preserve round readability but clarify construction: raised boss, rim thickness, wood plank or hide face, rear strap/grip cues, chalk/red legal marking, and edge wear localized to the rim. |
| `iron_maul` | Keep the blunt-crush block silhouette while adding material truth: chamfered head faces, edge highlights, handle socket/collar, worn impact planes, and blackened-iron roughness variation consistent with `bearded_axe`. |
| `arming_sword` | Distinguish from `longsword` as a one-handed 910 mm weapon: shorter blade, simpler guard, compact pommel, one-hand grip, and a broader/readable tip or fuller treatment that does not mimic the longsword silhouette. |
| `billhook` | Enlarge and separate the hook profile so it cannot read as spear/glaive: visible forward hook/beak, back spike or cutting bill, socket collar, long shaft with two grip zones, and strong negative space inside the hook. |

### Armor fixes

| Asset | Required silhouette/material target for next pass |
| --- | --- |
| `gambeson` | Add quilted linen identity: stitched vertical/diagonal channels, padded sleeve/skirt volume, cord ties, softened cloth edges, and deformation loops at shoulders/elbows/hips. Remove stray visual noise that is not stitching or wear. |
| `mail_hauberk` | Must no longer be a dark recolor of gambeson. Add ring/mail cues via normal/height/dither pattern, coif or scarf drape, belt weight, metallic glints on ring rows, and a heavier hanging silhouette distinct from quilted cloth. |
| `heavy_plate` | Add rigid plate segmentation: breastplate center ridge, pauldrons, vambraces, greaves, tassets, rivets, leather hinges/straps, bevel highlights, and articulated gaps at shoulder/elbow/hip/knee so it reads as metal plates, not a gray tunic. |
| `lamellar` | Add layered tile rows and lacing: overlapping horizontal/chevron plates, side laces, tassets, bracer rows, alternating edge highlights, and visible plate overlap. It must read as lamellar construction, not flat brown cloth/leather. |
| `fencer_light` | Make the agile/fencer identity distinct from gambeson: fitted short jack, narrow waist, open or lower-bulk shoulders, forearm guard, buckles, pale duelist trim, and less skirt/torso mass for a quick-stepping silhouette. |
| `bruiser_padded_plate` | Fix the format outlier: no detached top square or blob. Integrate mask/gorget/neck guard into the body design, expose padded blocks plus reinforced plates, separate wide belts/cuffs, and lift black values with readable edge/detail highlights. |

### Arena fixes

| Asset | Required silhouette/material target for next pass |
| --- | --- |
| `oathyard_verdict_ring` | Reduce random red/dark scatter and keep the central mark from dominating as a flat black disk. The ring should read as chalked stone ritual geometry: low rim, oath mark, judgment axis/balcony orientation, entry break, worn contact marks, and playable floor space clear around fighters' feet. Scatter must become authored decals/wear, not noise. |
| `training_yard` | Resolve the grid into purposeful measured practice lines: clean circular boundary or rectangular practice inset, intentional clipped edge treatment, named markers for start/target/footwork points, rope/work-lamp or post purpose, and packed-clay/wood/material separation. Markers must teach spacing rather than look random. |

### Acceptance for the next model/art pass

A revised model source package should not be accepted until it provides:

- updated source metadata carrying these per-asset silhouette/material notes or equivalent fields;
- regenerated previews/contact sheets plus close-up crops for fighters, weapons, armor, and arenas;
- visual inspection confirming the listed identity blockers are gone;
- fighter motion/deformation evidence for idle, walk, and attack or mapped OATHYARD action poses;
- armor skin/socket no-clipping proof if armor is wearable;
- local structural glTF/package validation, with external Khronos/DCC/owner/public readiness still explicitly unclaimed unless separately evidenced.
