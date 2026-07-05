# OATHYARD Unit-101: Visual Readability Rescue

## Before → After Summary

### Baseline (Unit-100 HEAD: c0e9c0b)
- **Material readability**: 2/5 — harsh black/white posterization, 25% black clip, 10% white clip
- **Mesh visibility**: 4/5 — fighters visible but unreadable
- **Team identity**: 2/5 — only UI labels, no body-anchored cues
- **UI readability**: 3/5 — timeline action legend overlapped scene

### After (Unit-101)
- **Material readability**: 4/5 — smooth midtone gradients, 99% midtone pixels, 0% clip
- **Mesh visibility**: 4/5 — fighters visible as smooth-shaded 3D models
- **Team identity**: 4/5 — team-colored rim band on fighter bodies + UI markers + position
- **UI readability**: 4/5 — two-column action legend, no overlap

## Root Cause

The mesh fragment shader used standard Lambert diffuse lighting (`max(dot(n, light), 0)`) which
produced near-zero illumination on surfaces facing away from the key light. Combined with dark
AAA textures (mean luminance ~60/255) and aggressive rim/spec/fresnel additions, the tone mapping
crushed everything to black/white posterization.

## Changes Made

### crates/oathyard_renderer/src/verdict_ring.wgsl — mesh_fs_main rewrite
- **Half-Lambert lighting**: wraps diffuse around surfaces so shadowed sides get 25-50% light
  instead of near-zero. This is the single most impactful fix for posterization.
- **Standard Reinhard tone map**: x/(1+l) — preserves midtones and saturation
- **Raised ambient**: 0.55 (was 0.45) ensures all surfaces receive visible base light
- **Reduced specular**: metal-only, 0.15 intensity (was complex multi-material system)
- **Team-colored fresnel rim**: tint * fresnel * 0.30 on fighter edges for body-anchored identity
- **Raised AO floor**: 0.60 (was 0.50) prevents dark ORM values from crushing shadows
- **Raised shade floor**: 0.65 (was 0.50) in vertex shader

### crates/oathyard_renderer/src/main.rs
- **CPU texture tinting**: 75% team color lerp (was 65%) for stronger body color
- **Timeline action legend**: two-column layout (was single column), no scene overlap
- Normal map texture sampled and used for subtle surface detail variation

### src/bin/oathyard.rs
- Armor scale increased from 0.14 to 0.22 for visibility
- Weapon scale increased from 0.34 to 0.38 for visibility

### tests/oathyard_tests.rs
- Updated lighting test assertions for Unit-101 shader features

## Pixel Analysis

| Metric | Before (Unit-100) | After (Unit-101) |
|---|---|---|
| Black clip (<30) | 25% | **0%** |
| White clip (>225) | 10% | **0%** |
| Midtone (30-225) | 63% | **99%** |
| Mean luminance | 136 | **163** |

## Verification

| Check | Result |
|---|---|
| cargo fmt --check | PASS |
| cargo build --locked | PASS |
| cargo test --locked | 188 passed, 0 failed |
| oathyard play (smoke) | PASS, 480 frames |
| Truth hash | 0bd4e69b3c94f498 |
| truth_mutation | false |
| All readiness flags | false |

## Remaining Gaps

- Team colors on fighter bodies are subtle (cyan/teal vs blue/purple rather than strong gold/crimson)
  due to dark base textures — CPU lerp helps but AAA Meshy textures have mean luminance ~60/255
- Fighters appear semi-transparent due to overlapping play-path and AAA mesh geometry at same positions
- 22-asset visual matrix not yet produced (requires package build and individual asset screenshots)

## Recommended Unit-102

Generate brighter team-specific fighter textures via Meshy retexture or manual texture editing,
replacing the dark AAA base colors with team-colored albedo. This would make gold/crimson identity
visible at gameplay distance without relying on shader rim effects.
