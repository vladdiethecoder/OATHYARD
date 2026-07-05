# OATHYARD Unit-099 Visual Review: Arena, Team Identity, Material, UI

## Before → After Summary

### Baseline (Unit-098 HEAD: 777248e)
- **Arena**: 1/5 — grey void, no visible floor, ring, or arena boundaries
- **Team identity**: 2/5 — fighters indistinguishable, UI labels carried all identity
- **Material readability**: 2/5 — posterized/crushed textures, black/white noise
- **UI readability**: 4/5 — mostly readable, "OATHYARD" clipped, font artifacts in "SIMULTANEOUS REVEAL"

### After (Unit-099)
- **Arena**: 4/5 — visible textured floor, white ring boundary, four iron boundary posts, witness stones
- **Team identity**: 4/5 — gold triangle/crimson diamond markers clearly distinguish player/opponent
- **Material readability**: 3/5 — gamma-only tone mapping preserves saturation, higher ambient reduces crush
- **UI readability**: 4/5 — "OATHYARD" fully visible, "COMMIT REVEAL" clean text, no debug artifacts

## Root Cause

The windowed renderer (`oathyard play --interactive`) was missing the SDF raymarch pass entirely.
Only the mesh pipeline was rendering — the SDF arena layer (floor, ring, witness stones, boundary posts)
was never drawn. The "grey void" was the render pass clear color (0.35, 0.32, 0.28).

## Changes Made

### crates/oathyard_renderer/src/verdict_ring.wgsl
- Added 4 iron boundary posts to scene_sdf (material 8.0)
- Brightened floor material tint from (0.38,0.34,0.26) to (0.52,0.45,0.35)
- Brightened ring material to metallic gold (0.0 type with 0.85,0.68,0.28 tint)
- Doubled ring torus minor radius from 0.035 to 0.065 for visibility
- Reduced fog density from 0.012 to 0.004 (max 0.12 vs 0.20)
- Darkened void background to near-black for floor contrast
- Boosted SDF ambient from 0.38 to 0.65
- Raised SDF ground occlusion floor from 0.55 to 0.72
- Raised SDF AO floor from 0.16 to 0.28
- Mesh: boosted ambient from 0.55 to 0.85
- Mesh: raised AO floor from 0.45 to 0.55
- Mesh: stronger team tint blend from 0.75 to 0.92
- Mesh: replaced Reinhard tone map with gamma-only (preserves saturation)
- Mesh: raised shade floor from 0.22 to 0.50

### crates/oathyard_renderer/src/main.rs
- Added sdf_pipeline to WindowedApp struct
- Created SDF pipeline in windowed setup (vs_main/fs_main with depth stencil)
- Added depth buffer to windowed offscreen render target
- Added SDF fullscreen triangle draw before mesh draw in render loop
- Added depth stencil to windowed mesh pipeline
- Fixed "SIMULTANEOUS REVEAL" → "COMMIT REVEAL" in UI text
- Fixed "OATHYARD" bottom-right text clipping (x offset -90 → -170)

### src/bin/oathyard.rs
- Bumped fighter mesh scale from 0.72 to 0.95 for better visibility

## Verification

| Check | Result |
|---|---|
| cargo fmt --check | PASS |
| cargo build --locked | PASS |
| cargo test --locked | 188 passed, 0 failed |
| oathyard play (smoke) | PASS, 480 frames |
| Truth hash | 0bd4e69b3c94f498 |
| truth_mutation | false |
| owner_visual_acceptance | false |
| public_demo_ready | false |
| release_candidate_ready | false |

## Screenshots

- Before: `screenshots/before/` — 17 gameplay states from Unit-098 HEAD
- After: `screenshots/after/` — 17 gameplay states after Unit-099 fixes
