# OATHYARD Visual Review — Native Roster Asset Capture Matrix (Unit-103)

## What This Is

Curated visual evidence from the Unit-103 native executable roster asset capture matrix. Every source-approved roster asset is rendered individually through:

```bash
./bin/oathyard play --capture-roster-matrix <output-dir>
```

This is executable-path visual evidence, not a filesystem-only audit and not a static preview image set.

## Readiness Boundary

- `truth_mutation`: **false**
- `production_asset_ready`: **false**
- `owner_visual_accepted`: **false**
- `public_demo_ready`: **false**
- `release_candidate_ready`: **false**

## Asset Counts

- Fighters: 6
- Weapons: 8
- Armor: 6
- Arenas: 2
- Total: 22

## What Changed In This Evidence Path

1. Native executable capture mode exists on `oathyard play` and can be run from a packaged executable.
2. Each asset receives a fresh native PNG screenshot and SHA256.
3. Each matrix row records geometry consumption, mesh counts, material binding, runtime path, source manifest path, and false readiness flags.
4. Contact sheets and per-kind contact sheets are generated for reviewer inspection.
5. Visual scores now include pixel-based metrics derived from actual screenshots: foreground coverage, edge contrast, silhouette variance, and composite readability. These metrics are evidence aids, not owner acceptance.

## Known Remaining Visual Risks

- Fighter identities remain visually similar; distinct role silhouettes/accessories are still needed for future units.
- Some low-poly armor types, especially mail/lamellar, need stronger material texture cues.
- Grey weapons on grey/tan arena backgrounds still have limited contrast, although framing is now much more readable than the earlier tiny-weapon captures.
- These issues do not mutate deterministic truth and do not promote readiness.

## Files

- `manifest.json` — machine-readable 22-asset matrix
- `visual_scores.md` — score summary
- `asset_matrix_contact_sheet.png` — all assets
- `fighters_contact_sheet.png` — fighters
- `weapons_contact_sheet.png` — weapons
- `armors_contact_sheet.png` — armor
- `arenas_contact_sheet.png` — arenas
- `thumbnails/` — representative per-asset screenshots
