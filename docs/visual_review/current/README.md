# OATHYARD Unit-102: Visual Composition and Roster Matrix

## Before → After Summary

### Baseline (Unit-101 HEAD: bbb7aca)
- **Duplicate geometry**: play-path fighters misclassified as "weapon" by `infer_mesh_asset_class`, retained alongside AAA fighters — 9 meshes, 2 fighter bodies per side
- **Full-body tint**: 75% team color lerp destroyed PBR material detail
- **Timeline**: action legend still cramped at bottom

### After (Unit-102)
- **Single fighter body per side**: `infer_mesh_asset_class` fixed to use `contains()` — 7 meshes, exactly 1 AAA fighter per side
- **Material detail preserved**: 30% team color lerp (was 75%), identity via fresnel rim band (0.45 intensity)
- **Timeline**: two-column action legend with clear separation

## Root Cause

`infer_mesh_asset_class` used exact string matching (`match asset_id { "saltreach_duelist" => "fighter" }`)
but play-path manifest generates prefixed IDs (`player_saltreach_duelist`). Prefixed IDs fell through
to `_ => "weapon"`, so play-path fighters were retained as "weapon" class alongside AAA fighters,
creating duplicate geometry at the same position.

## Changes Made

### crates/oathyard_renderer/src/main.rs
- **Fixed `infer_mesh_asset_class`**: changed from exact match to `contains()` for all 22 asset names
- **Reduced CPU texture tint**: 30% team color (was 75%) to preserve PBR material detail
- Armor/weapon transforms: offset to reduce body overlap

### crates/oathyard_renderer/src/verdict_ring.wgsl
- **Stronger team rim band**: 0.45 intensity (was 0.30) to compensate for reduced body tint

### src/bin/oathyard.rs
- Armor scale: 0.14 → 0.22, weapon scale: 0.34 → 0.38
- Armor/weapon translations offset from fighter body center

### tests/oathyard_tests.rs
- Updated team rim band assertion

## 22-Asset Roster Visual Matrix

All 22 source-approved assets have runtime mesh files and are consumed:

| Kind | Count | Mesh Files | Vertices Range |
|---|---|---|---|
| Fighter | 6 | 6/6 | 13,262–13,598 |
| Weapon | 8 | 8/8 | 472–1,870 |
| Armor | 6 | 6/6 | 672–1,422 |
| Arena | 2 | 2/2 | 1,850–2,398 |

Full matrix: `artifacts/verification/2026-07-05T_unit102_visual_composition_roster_matrix/roster_asset_visual_matrix.json`

## Verification

| Check | Result |
|---|---|
| cargo fmt --check | PASS |
| cargo build --locked | PASS |
| cargo test --locked | 188 passed, 0 failed |
| oathyard play (smoke) | PASS, 480 frames |
| mesh_asset_count | 7 (was 9) |
| Truth hash | 0bd4e69b3c94f498 |
| truth_mutation | false |
| All readiness flags | false |

## Remaining Gaps

- AAA Meshy fighter meshes have complex layered geometry (572K vertices) that at low resolution
  can appear as overlapping forms — this is an asset quality limitation, not a rendering bug
- Team rim band provides body-anchored identity but is subtle at gameplay distance
- Full per-asset screenshots require a dedicated capture matrix tool (not yet built)

## Recommended Unit-103

Build a native executable asset capture matrix tool that renders each of the 22 roster assets
individually through the real `oathyard play` path, producing per-asset screenshots and visual
scores for the full roster evidence.
