# 0009: Production renderer selection for visual-fidelity escalation

Status: Accepted as V1 direction; implementation remains blocked/incomplete.
Date: 2026-06-29
Milestone: V1 — Renderer decision

## Decision

Use a Bevy/wgpu production-renderer spike as the next high-fidelity native-PC 3D renderer path, while preserving the existing deterministic Rust truth core as the authority and keeping renderer/UI/audio/VFX/camera presentation-only after truth hashing.

This does not mark production renderer completion. It selects the next legally usable, source-buildable, Rust-native path to test against OATHYARD's production visual target.

Machine-readable readiness remains false:

```text
production_renderer_complete: false
owner_visual_acceptance: false
public_demo_ready: false
release_candidate_ready: false
legal_clearance: false
trademark_clearance: false
store_readiness: false
```

## Current environment evidence

Measured on this host during the V1 inspection:

```text
GPU: NVIDIA GeForce RTX 5090, 32607 MiB, driver 595.80
Vulkan instance: 1.4.341
Vulkan NVIDIA device: NVIDIA GeForce RTX 5090, apiVersion 1.4.329, driverName NVIDIA, driverInfo 595.80
OpenGL display renderer: AMD Ryzen 9 7900X iGPU via Mesa 26.1.3 on :0
Rust: rustc 1.96.0, cargo 1.96.0
USD tools: /usr/bin/usdcat, /usr/bin/usdchecker
Shader tools: /usr/bin/glslangValidator, /usr/bin/spirv-val
Image tools: /usr/bin/convert, /usr/bin/montage
Installed engines: no godot, no UnrealEditor/unreal detected
Blender: /usr/bin/blender exists but fails startup with `undefined symbol: _ZTVN17MaterialX_v1_39_46OutputE`
Installed packages: blender-5.1.1-3.fc44.x86_64, materialx-1.39.5-1.fc44.x86_64, usd-26.03-4.fc44.x86_64
```

The Blender failure blocks the normal DCC/source-asset path until a self-contained Blender build, Flatpak Blender, matching MaterialX ABI, container/toolbox, or equivalent source-authoring route is installed and verified.

## Candidate evaluation

| Candidate | License / availability | Fit | Decision |
| --- | --- | --- | --- |
| Bevy/wgpu | Official Bevy docs state Bevy is free/open-source under MIT or Apache 2.0; current quick-start lists `bevy = "0.19"` as a Rust crate. | Rust-native, source-buildable with Cargo, wgpu backend can target Vulkan on RTX 5090, supports 3D/PBR/gltf/skinning ecosystem, clean truth isolation by consuming replay-derived presentation data. | Selected for V1 production-renderer spike. |
| Custom Vulkan/DX12 | Legally usable but would require direct loader/header integration, swapchain, asset, skinning, PBR, shadows, capture, and tooling from scratch. | Maximum control, highest implementation cost. | Fallback only if Bevy/wgpu fails measured spike criteria. |
| Raw X11/GLX/OpenGL | Existing ADR 0008 authorized a dependency-zero spike, but the referenced spike directory is absent and GLX currently reports the AMD iGPU display path, not the RTX 5090 production path. | Useful technical spike/debug route, not a credible high-fidelity production target by itself. | Superseded for V1 selection by Bevy/wgpu unless the user explicitly wants the raw OpenGL spike first. |
| Godot 4 | Official license page says Godot is MIT and game content remains the user's; engine not installed. | Strong editor/runtime option, but adds non-Rust engine pipeline and separate scripting/build/package integration. | Secondary option; reconsider if Bevy/wgpu cannot meet animation/asset pipeline needs. |
| Unreal Engine 5 | EULA-governed proprietary licensed technology; not installed. | Strongest out-of-box Nanite/Lumen-class rendering, but requires Epic account/download/EULA/royalty/licensing review, large toolchain, and C++/asset pipeline bridge. | Not adopted without explicit owner/legal/toolchain gate. |
| Fyrox | Rust engine option, less proven/mature for the target than Bevy. | Possible fallback. | Defer. |
| Browser/HTML | QA only per canon. | Cannot satisfy native product presentation. | Rejected. |

## Source-backed rationale

- Bevy official introduction describes Bevy as a Rust game engine with complete 2D/3D feature goals, data-driven ECS, modularity, and MIT/Apache-2.0 licensing.
- Bevy official getting-started docs list crate installation through Cargo and `bevy = "0.19"`.
- Godot official license page states Godot is MIT, free/open-source, and game content is not covered by the engine license.
- Unreal official EULA page states Unreal Engine use is governed by Epic's EULA and accepting/downloading creates contractual terms; this is an owner/legal gate before adoption.

## Truth isolation architecture

The renderer process/crate must consume a frozen presentation packet produced after authoritative truth hashing:

```text
truth sim / replay verify
  -> final_state_hash + content_hash + trace/replay artifact hashes
  -> presentation_packet.json
  -> Bevy/wgpu renderer reads packet + production asset manifest
  -> screenshots / frame timing / fight-film cameras / UI
```

Forbidden renderer writes:

- replay JSON;
- trace JSON;
- contact packets;
- injuries;
- capability deltas;
- frame costs;
- action validity;
- end state/winner;
- content/table hashes;
- authoritative replay hash.

Required V1 spike proof:

1. Run the same duel with production presentation disabled.
2. Verify replay.
3. Render/capture via Bevy/wgpu from post-hash presentation packet.
4. Run the same duel with production presentation enabled path present.
5. Compare replay JSON, trace JSON, final state hash, content hash, contact packets, injuries, capability deltas, action validity, and end condition. They must be byte-identical or the renderer path is rejected.

## Asset pipeline direction

Target source/interchange/runtime path:

```text
assets_src/production/** (.blend and/or OpenUSD .usda/.usd source when DCC works)
  -> exported .gltf/.glb or Bevy-compatible runtime assets
  -> assets/production_visual_manifest.json
  -> Bevy/wgpu renderer load test
  -> preview renders + in-engine screenshots + content hashes
```

Until Blender or equivalent DCC works, production asset generation is blocked. Current `assets_src/*.oysrc` -> low-poly glTF output stays a deterministic regression lane only and must not be promoted into `assets/production_visual_manifest.json`.

## Falsification criteria

Reject or downgrade Bevy/wgpu if the spike cannot prove all of:

- source build on this host through Cargo with exact dependency versions recorded;
- native PC windowed execution and screenshot capture;
- wgpu selects a hardware GPU path suitable for high-fidelity rendering, preferably the RTX 5090/Vulkan path;
- production presentation on/off truth hashes are identical;
- glTF/GLB production-asset load path works;
- PBR materials, dynamic lights, shadows, cameras, and performance instrumentation are available in the spike;
- package/build dependency impact is measured and reversible;
- no public-demo/release/owner acceptance flags are flipped.

If Bevy/wgpu fails from missing system libraries, GPU backend mismatch, unacceptable package impact, or insufficient rendering/animation support, fall back to a custom Vulkan/direct-loader spike or Godot/Unreal only after a new ADR.

## Blockers before V2/V3

1. Blender/DCC blocker:
   - command: `blender --factory-startup --version`
   - failure: `blender: symbol lookup error: blender: undefined symbol: _ZTVN17MaterialX_v1_39_46OutputE`
   - installed mismatch evidence: `blender-5.1.1-3.fc44.x86_64` with `materialx-1.39.5-1.fc44.x86_64` and `/usr/lib64/libMaterialX*.so.1.39.5`.
   - unblock: install official Blender tarball/Flatpak/container or matching MaterialX ABI and verify `blender --background --factory-startup --version` plus a headless render/export smoke.
2. Production renderer implementation absent:
   - no `artifacts/production_renderer/latest/production_renderer_manifest.json` exists.
   - no Bevy/wgpu spike crate/path exists yet.
3. Production asset manifest absent:
   - no `assets/production_visual_manifest.json` exists.

## Acceptance commands for the next V1 spike

```sh
cargo build --locked
./tools/run_duel.sh examples/duels/basic_oathyard.duel --out artifacts/production_renderer/v1_truth_off
./tools/replay_verify.sh artifacts/production_renderer/v1_truth_off/replay.json
# future Bevy/wgpu spike command writes artifacts/production_renderer/latest/production_renderer_manifest.json
./tools/presentation_truth_isolation.sh examples/duels/basic_oathyard.duel artifacts/presentation_truth_isolation/v1
./tools/capture_high_fidelity_screens.sh artifacts/high_fidelity_screens/v1
./tools/visual_benchmark.sh artifacts/visual_review/v1
./tools/audit_readiness.sh . artifacts/readiness/v1_renderer
```

Expected current result before implementation: high-fidelity gates fail closed.
