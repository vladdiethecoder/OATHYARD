# OATHYARD weapon diversity concept extraction

Source image: `assets_src/reference/concepts/hustle_honor_weapon_diversity_concept_sheet.png`
Source SHA-256: `e6ab4dcf8a7e9026a273e5a7fba1d2620f54e0c20e020719cff2b8a10b41a9f6`
Extracted machine spec: `assets_src/reference/concepts/weapon_diversity_concept_spec.json`

This is a user-supplied concept-control signal for generating local Blender weapon candidates. It is not a production art acceptance packet and does not alter authoritative gameplay truth.

## Boundary

- Presentation-only candidate generation.
- No canonical `assets/gltf/` overwrite.
- No readiness, owner acceptance, legal/trademark, release, or public-demo claim.
- Generated weapons live under `assets/production_candidates/<run_id>/weapons/<weapon_id>/` until native renderer and physical-fidelity gates accept them.
- The current base male USDZ files under `assets/model_candidates/base_male/` are recorded as future scale/attachment references only.

## Extracted roster

| # | ID | Name | Archetype | Tags |
| --- | --- | --- | --- | --- |
| 1 | `colossal_war_hammer` | Colossal War Hammer | Ultra Heavy / Brutal / Smash | slow, heavy_hit, armor_break |
| 2 | `ardent_sword` | Ardent Sword | Balanced / Versatile / Cut & Thrust | balanced, reliable, all_rounder |
| 3 | `longsword` | Longsword | Balanced / Control / Duels | control, parry, riposte |
| 4 | `greatsword_zweihander` | Greatsword (Zweihander) | Two-Handed / Power / Reach | heavy_reach, wide_slash, armor_pressure |
| 5 | `greatblade_nodachi` | Greatblade (Nodachi) | Heavy / Wide Arc / Reach | slower, devastating, long_reach |
| 6 | `spear` | Spear | Reach / Thrust / Control | long_reach, poke, keep_out |
| 7 | `glaive` | Glaive | Polearm / Sweep / Control | wide_sweeps, crowd_control |
| 8 | `halberd_poleaxe` | Halberd (Poleaxe) | Polearm / Versatile / Pierce | puncture, sweep, hook |
| 9 | `battle_axe` | Battle Axe | Heavy / Chop / Armor Break | power, chop, armor_break |
| 10 | `dane_axe` | Dane Axe | Aggressive / Cleave / Pressure | fast_chop, bleed, relentless |
| 11 | `mace` | Mace | Blunt / Control / Disrupt | blunt, concuss, disrupt |
| 12 | `flanged_maul` | Flanged Maul | Heavy Blunt / Crush / Break Guard | crush, armor_damage, stagger |
| 13 | `chain_flail` | Chain Flail | Unpredictable / Whip / Crowd Control | unpredictable, wrap, disarm |
| 14 | `dual_daggers` | Dual Daggers | Fast / Agility / Bleed | quick, mobile, finisher |
| 15 | `saber_scimitar` | Saber (Scimitar) | Agile / Slash / Flow | fast, flow, bleed |
| 16 | `bo_staff` | Bo Staff | Balanced / Flow / Control | sweep, parry, flow |
| 17 | `shield_one_handed` | Shield & One-Handed | Defense / Counter / Control | block, counter, hold_line |
| 18 | `hand_cannon` | Hand Cannon | Ballistic / Ranged / Burst | pierce, burst, finish |
| 19 | `rotary_revolver` | Rotary Revolver | Ballistic / Rapid Fire / Control | rapid, reloadless, suppress |
| 20 | `hooked_chain` | Hooked Chain | Mobility / Disrupt / Control | pull, disrupt, reposition |
| 21 | `hollow_fist_gloves` | Hollow Fist Gloves | Unarmed / Pressure / Combo | fast, combo, internal_damage |
| 22 | `arcane_focus` | Arcane Focus | Esoteric / Magic / Area Control | elemental, area, setup |

## Generator requirements

Every candidate should emit:

- `.source.blend`
- `.candidate.glb`
- `.preview.png`
- `candidate_manifest.json`
- `visual_audit.json` after inspection

Required candidate manifest properties:

- source concept image and hash
- extracted concept archetype/tags/damage/weight
- Blender source file
- runtime export file
- preview file
- source/export/preview hashes
- mesh metrics
- material list
- contact/readability features
- truth boundary flags
- not-claimed list
- blockers

## Current acceptance boundary

A generated candidate can pass only `machine_candidate_readability` or fail with blockers. It cannot set:

- `production_assets_complete`
- `owner_visual_acceptance`
- `public_demo_ready`
- `release_candidate_ready`
- `native_in_engine_runtime_capture`
