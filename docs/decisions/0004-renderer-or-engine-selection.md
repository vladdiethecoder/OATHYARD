# 0004: Renderer or Engine Selection for Production 3D

Status: Accepted selection target; implementation incomplete.
Date: 2026-06-30
Supersedes/clarifies: `docs/decisions/0009-production-renderer-selection.md` where current toolchain evidence differs.

## Context

The current OATHYARD truth core is Rust and deterministic. Current visual evidence includes software/PPM/SVG/debug-local render paths and production-candidate 1920x1080 image evidence, but not a continuous high-fidelity native-PC 3D renderer or engine path.

Fresh local inspection on 2026-06-30 recorded:

- Rust: `rustc 1.96.0`, `cargo 1.96.0`.
- Blender: `/home/vdubrov/.local/bin/blender`, Blender 4.3.2 starts successfully.
- Python: `/usr/bin/python3`, Python 3.14.6.
- glTF/interchange tools: `gltf-transform`, `assimp`, `usdcat`, `usdchecker`, `usdview` available; `gltf-validator`, `toktx`, `ktx`, `pngquant`, `pngcrush`, `optipng` unavailable.
- Shader/graphics tools: `glslangValidator`, `spirv-val`, `vulkaninfo`, `glxinfo`, `xrandr` available.
- Image/video tools: ImageMagick `convert`/`magick`/`montage`, FFmpeg available.
- Engine binaries: `/usr/bin/godot` available; UnrealEditor/UnrealEditor-Cmd/UE4Editor unavailable; no `bevy` CLI binary expected.
- GPU: NVIDIA GeForce RTX 5090 visible through Vulkan/NVIDIA driver 595.80; AMD iGPU also visible; current `nvidia-smi` shows a llama-server using ~26 GiB VRAM, so production renderer captures may need GPU-memory coordination.

Official source checks on 2026-06-30:

- Bevy homepage describes a Rust engine with 3D renderer support for lights, shadows, cameras, meshes, textures, materials, glTF loading, skeletal animation, cross-platform support, and MIT/Apache-2.0 licensing.
- Godot license page states Godot is free/open-source under MIT, commercial redistribution is allowed with copyright/license notice, and game content remains the user's.
- Unreal licensing page states game developers can use Unreal with royalties after $1M USD gross product revenue and access to source/features under the EULA; adoption requires owner/legal/Epic account/toolchain gate.

## Decision

Use this selection order:

1. **Bevy/wgpu Rust-native renderer spike** — selected first implementation path because it preserves Rust/source-build workflow, can consume post-hash presentation packets, supports glTF/PBR/skinning/cameras/lights, and has permissive MIT/Apache-2.0 licensing.
2. **Godot native 3D bridge** — practical fallback because Godot is installed and MIT-licensed; requires a separate Rust-truth-to-Godot presentation packet bridge and package/capture ADR before runtime adoption.
3. **Custom Vulkan/wgpu/direct renderer** — fallback if engine dependencies or Bevy/Godot integration fail; highest implementation cost but maximum control.
4. **Unreal Engine** — not adopted until Unreal is installed and the EULA/royalty/source-build/asset pipeline/legal gates are accepted by owner; strongest rendering features but largest external gate.
5. **Raw X11/PPM/SVG/software renderers** — local verification backends only; never production high-fidelity presentation.
6. **Browser/HTML** — QA harness only; rejected for native product presentation.

No renderer/engine may become authoritative truth. Renderer, animation, camera, UI, VFX, audio, cloth, and post-processing consume truth state only after hash/replay verification.

## Minimum production renderer acceptance

A selected backend must prove:

- native PC execution;
- 1920x1080+ deterministic capture path;
- skinned characters and animation playback/retarget test;
- layered armor and high-quality weapon meshes;
- PBR/equivalent material response with normal/roughness/metallic/AO/detail/damage/blood/dirt/wear maps;
- dynamic lights, shadows, reflection/GI approximation, fog/dust/smoke/mist/wetness/weather hooks;
- OATHYARD arena loaded with lighting/camera anchors;
- replay/fight-film cameras;
- performance instrumentation;
- package inclusion/smoke evidence;
- presentation-on/off truth hashes are byte-identical.

## Current status

```text
renderer_engine_selected_for_spike: bevy_wgpu
production_renderer_implemented: false
production_renderer_complete: false
in_engine_visual_ready: false
high_fidelity_ready: false
owner_visual_acceptance: false
public_demo_ready: false
release_candidate_ready: false
```

The next non-degrading implementation unit is a Bevy/wgpu or Godot spike that reads a post-hash presentation packet and candidate/production asset manifest, loads at least one fighter/weapon/arena trio, emits a renderer manifest, captures 1920x1080 screenshots, and passes `./tools/presentation_truth_isolation.sh` without changing truth outputs.
