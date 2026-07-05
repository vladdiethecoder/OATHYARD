# OATHYARD — Agent Context File

## Project Identity
3D duel combat game. Custom Rust engine: wgpu, winit, WGSL. Deterministic truth architecture with cryptographic hash chains. YOMI-style simultaneous-reveal combat (13 intent actions, 169 matchup matrix).

## Tech Stack
- **Language**: Rust (edition 2021)
- **Rendering**: wgpu + WGSL shaders, custom `.mesh.json` format
- **Window**: winit
- **Assets**: Meshy AI AAA meshes (4K PBR), `.mesh.json` (positions/normals/texcoords/indices as nested arrays)
- **Git LFS**: tracks `*.obj`, `*.glb`, `*.gltf`

## Repository Layout
```
src/lib.rs           — Core game logic (types, engine, AI sweep, truth edge, native render)
src/bin/oathyard.rs  — CLI entry point (play, native-combat-render, roster-showcase)
src/local_game.rs    — Windowed playable game loop (winit + wgpu)
crates/oathyard_renderer/
  src/main.rs        — Native renderer binary (offscreen + windowed modes)
  src/verdict_ring.wgsl — All shaders (SDF scene, mesh PBR, tone mapping)
content/             — JSON content (input, combat, animation, visual_qa)
tools/               — Shell scripts for build/test/package/QA
tests/               — Integration tests (189 tests, truth hash contracts)
```

## Critical Constraints
- **Truth hash**: `f17c8f76b9dfae86` (basic duel) — must never change from visual-only edits
- **Readiness flags**: `owner_visual_acceptance`, `public_demo_ready`, `release_candidate_ready` — must remain `false`
- **truth_mutation**: Must be `false` in all presentation/visual manifests
- **No RNG/HP/DPS/crit/cooldowns** — deterministic only
- **No external AI animation services** — local procedural/skeletal only

## Build & Test
```bash
cargo build --locked
cargo test --locked           # 189 tests
cargo build --manifest-path crates/oathyard_renderer/Cargo.toml  # rebuild renderer
./tools/verify.sh
./tools/audit_truth.sh
./tools/check_renderer_staleness.sh  # checks if renderer binary is stale
```

## Renderer Architecture
- SDF procedural scene (floor, ring, stones, fighters) in WGSL raymarcher
- High-fidelity mesh rendering: per-mesh bind groups via `create_buffer_init` (NOT `queue.write_buffer` — doesn't work per-draw in render pass)
- CPU readback → composite → present pipeline for windowed UI overlay
- Tone mapping: Reinhard extended (Hable was causing black/white posterization)
- Arena: brighter floor tint, fighter contact blob shadows, golden ring boundary

## Asset Pipeline
- GLB → `.mesh.json` converter in `tools/generate_runtime_asset_sets.py`
- Always extracts: positions, normals, texcoords (TEXCOORD_0), indices
- AAA assets in `assets/presentation_runtime/` (gitignored, 50-60MB each)
- Manifests in `assets/manifests/` (committed)

## Conventions
- Unit-based development (Unit-XXX commits)
- Commit after each completed unit, push to `origin/main`
- Pre-commit hook: `cargo fmt --check` + `cargo check --locked` + staleness check
- Artifact cleanup: `./tools/cleanup_artifacts.sh dry-run`
- Large `.mesh.json` files are gitignored; manifests and code are committed
- `src/lib.rs` is 7000+ lines — be extremely careful when patching; use `patch` tool, never heredoc
