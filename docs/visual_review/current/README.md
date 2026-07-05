# OATHYARD Unit-100: Native Render Stack Repair

## Before → After Summary

### Baseline (Unit-099 HEAD: 1321c09)
- **Mesh visibility**: 1/5 — SDF arena visible but real Meshy/Rodin fighter meshes absent
- **Arena**: 4/5 — floor, ring, boundary posts visible
- **Team identity**: 2/5 — only UI labels
- **UI**: 3/5 — timeline legend overlapped scene, boot menu ghosted

### After (Unit-100)
- **Mesh visibility**: 4/5 — real AAA Meshy fighter meshes visible with arena
- **Arena**: 5/5 — floor, ring, boundary posts all clearly visible
- **Team identity**: 4/5 — gold triangle/crimson diamond markers + floor position + shape distinguish teams
- **UI**: 4/5 — timeline legend in panel, boot menu clean

## Root Cause

TWO critical bugs from the Unit-099 SDF integration:

1. **SDF depth overwrite**: The SDF pipeline had `depth_write_enabled=true` and `depth_compare=Less`.
   The SDF fullscreen triangle wrote near-0 depth for every pixel, causing all mesh fragments
   to fail the depth test and be discarded.

2. **Per-mesh material uniform pitfall**: `queue.write_buffer` was called inside the render pass
   between draw calls, which does NOT update per-draw in wgpu.

## Changes Made

### crates/oathyard_renderer/src/main.rs
- SDF pipeline: depth_write=false, depth_compare=Always (background layer)
- Mesh pipeline: depth_write=true, depth_compare=Less, blend=REPLACE
- Per-mesh bind group 0 with dedicated material uniform buffer
- CPU-side team color tinting (lerp blend) baked into base color textures
- Vertex colors set to team tint for fighter body meshes
- Fixed boot menu ghosting (skip redundant state label)
- Fixed timeline action legend (moved to bottom panel)
- Combat cameras adjusted for fighter visibility

### crates/oathyard_renderer/src/verdict_ring.wgsl
- Fighter mesh uses texture directly (team color baked into texture at load time)
- Gentle Reinhard tone map for mesh path
- Reduced mesh ambient to preserve team color saturation
- Mesh fragment alpha changed to 1.0 (opaque)

## Draw Order / Depth / Composition

```
Render pass (depth cleared to 1.0):
  1. SDF pipeline (depth_write=false, depth_compare=Always, blend=ALPHA)
     → Arena background: floor, ring, boundary posts, witness stones
     → Writes color to all pixels, does NOT write depth
  2. Mesh pipeline (depth_write=true, depth_compare=Less, blend=REPLACE)
     → Fighter/weapon/armor/arena meshes
     → All fragments pass (depth < 1.0), completely overwrite SDF color
  3. CPU UI composite (post-readback)
     → UI panels/text drawn over composited RGBA buffer
```

## Verification

| Check | Result |
|---|---|
| cargo fmt --check | PASS |
| cargo build --locked | PASS |
| cargo test --locked | 188 passed, 0 failed |
| oathyard play (smoke) | PASS, 480 frames |
| Truth hash | 0bd4e69b3c94f498 |
| truth_mutation | false |
| native_windowed_execution | true |
| mesh_geometry_consumed | true |
| mesh_asset_count | 9 |

## Remaining Gaps

- Team tint (gold/crimson) baked into textures but AAA mesh textures are very dark,
  making team colors subtle at gameplay distance. UI markers (gold triangle, crimson
  diamond) provide primary team identity.
- Material readability at 3/5 — AAA mesh normal data creates noisy shading on 1M+ tri meshes
- Fighter bodies appear as dark silhouettes with white highlights due to texture brightness

## Recommended Unit-101

PBR material refinement: brighten/retexturing AAA fighter textures for better visibility,
or generate new fighter textures with team colors as the base color rather than multiplying
dark textures by team tints.
