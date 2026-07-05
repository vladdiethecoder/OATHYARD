# OATHYARD Visual Review — Roster Asset Capture Matrix (Unit-103)

## What This Is

This directory contains curated visual evidence from the Unit-103 native
executable roster asset capture matrix. Every roster asset has been rendered
individually through the native `oathyard play --capture-roster-matrix` path
using the real wgpu/Vulkan renderer with source-approved Meshy/Rodin assets.

**This is evidence only.** It does not promote production asset readiness,
owner visual acceptance, public demo readiness, or release candidate readiness.

## Readiness Boundary

- `truth_mutation`: **false**
- `production_asset_ready`: **false**
- `owner_visual_accepted`: **false**
- `public_demo_ready`: **false**
- `release_candidate_ready`: **false**

## Asset Counts

- **Fighters**: 6 (saltreach_duelist, oathyard_writ, bruiser_oath, chainbreaker, gate_shield, reed_sentinel)
- **Weapons**: 8 (longsword, arming_sword, ash_spear, bearded_axe, billhook, curved_sword, iron_maul, round_shield)
- **Armor**: 6 (gambeson, mail_hauberk, bruiser_padded_plate, fencer_light, heavy_plate, lamellar)
- **Arenas**: 2 (oathyard_verdict_ring, training_yard)
- **Total**: 22

## Capture Method

```bash
./bin/oathyard play --capture-roster-matrix <output-dir>
```

Each asset was rendered individually through the native wgpu/Vulkan offscreen
renderer. No SDF placeholders, no static pre-existing images, no proxy geometry.

## Contact Sheets

- `asset_matrix_contact_sheet.png` — all 22 assets
- `fighters_contact_sheet.png` — 6 fighters
- `weapons_contact_sheet.png` — 8 weapons
- `armors_contact_sheet.png` — 6 armor pieces
- `arenas_contact_sheet.png` — 2 arenas

## Per-Asset Thumbnails

Representative screenshots in `thumbnails/` (one per kind).

## Known Visual Issues

- Weapons have low contrast (grey-on-grey) against the arena background
- Some fighters appear washed out due to flat lighting/material presentation
- Fog/lighting makes fine detail hard to read at this capture distance

These are presentation quality issues, not capture failures. All 22 assets
have valid geometry consumption (mesh_geometry_consumed=true) and native
screenshots with valid SHA256 hashes.

## See Also

- `manifest.json` — full machine-readable matrix
- `visual_scores.md` — per-asset pass/warn/fail score table
