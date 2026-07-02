# OATHYARD

OATHYARD is a deterministic native-PC planned-time physical melee duel foundation. The current repository verifies truth, replay, nonvisual artifacts, source-built runtime 3D asset structure, packaging smoke, and readiness boundaries. Public-demo-ready, release-candidate-ready, owner-final-accepted, legal clearance, and trademark clearance remain false unless those external gates are actually performed. The project is license-pending/unlicensed; see `LICENSE`.

## Core artifact contract

Normal verification keeps nonvisual evidence:

- `trace.json`
- `replay.json`
- `final_state_hash.txt`
- `duel_report.md`
- `fight_film_manifest.json`
- export-bundle manifest/report/hash files
- build, test, audit, package, environment, and readiness reports

Standalone two-dimensional diagrams, image rollups, frame dumps, debug panels, browser canvas output, and fallback captures are not accepted as visual evidence. Visual readiness is blocked until a native 3D renderer/engine capture path writes manifest-backed captures with renderer, asset, camera, replay/hash, and `truth_mutation=false` metadata.

## Quick start

```sh
cargo build --locked
cargo test --locked
./tools/build_assets.sh
./tools/run_duel.sh examples/duels/basic_oathyard.duel --out artifacts/latest
./tools/replay_verify.sh artifacts/latest/replay.json
./tools/verify.sh
```

`artifacts/`, `assets/`, and `target/` are local build/evidence caches. They are ignored and regenerated from source inputs.

## Repository layout

- `src/` — Rust truth, replay, artifact, and CLI source.
- `tests/` — Cargo integration tests.
- `tools/` — repo-native build, audit, verification, package, and smoke scripts.
- `docs/` — canon, scope, acceptance, roadmap, visual policy, and decision records.
- `examples/duels/` — runnable duel scenarios.
- `assets_src/` — source asset definitions; generated runtime files are emitted under ignored `assets/`.
- `content/` — content manifest consumed by asset/runtime/package flows.
- `packaging/linux/` — Linux desktop/AppStream-blocker metadata used by package gates.
- `spikes/` — retained experiments only; not production substrate and not package entrypoints.

## Build and test

```sh
./tools/build.sh
./tools/test.sh
cargo build --locked
cargo test --locked
```

## Assets

```sh
./tools/build_assets.sh
./tools/validate_assets.sh
./tools/asset_budget_audit.sh artifacts/asset_budget/latest
./tools/asset_visual_atlas.sh artifacts/asset_atlas/latest
```

The asset builder writes runtime mesh summaries, deterministic extruded 3D glTF files under `assets/gltf/`, PBR material manifests, provenance reports, and validation reports. Validation fails if production runtime glTF assets have no Z depth. Asset atlas evidence is structural/nonvisual until a native 3D renderer captures the assets.

## Scripted duel and replay

```sh
./tools/run_duel.sh examples/duels/basic_oathyard.duel --out artifacts/latest
./tools/replay_verify.sh artifacts/latest/replay.json
```

The run writes deterministic truth/replay/report artifacts only. `trace.json`, `replay.json`, and `duel_report.md` include deterministic non-HP end-condition evidence. `fight_film_manifest.json` is trace-derived shot metadata, not visual proof.

## Replay export bundle

```sh
./tools/export_replay_bundle.sh artifacts/latest/replay.json artifacts/export_bundle/latest
./tools/verify_replay_bundle.sh artifacts/export_bundle/latest
```

The export command verifies the replay, regenerates canonical trace/replay/report/manifest evidence, writes `export_bundle_manifest.json`, `export_bundle_report.md`, and `bundle_hashes.txt`, and excludes forbidden standalone visual proof files. Bundle verification replays the bundle and fails if any bundled file hash changes or forbidden visual files are present.

## Native combat visual status

```sh
./tools/native_combat_render.sh examples/duels/basic_oathyard.duel artifacts/native_combat/latest
```

Until a real native 3D renderer capture path exists, this command verifies replay/truth input and writes a blocked manifest/report (`oathyard.native_3d_visual_blocked.v1`) instead of generating fallback visual proof. The command must not mutate truth or substitute non-3D evidence for visual readiness.

## Visual evidence policy

```sh
./tools/audit_visual_artifacts.sh
./tools/capture_high_fidelity_screens.sh artifacts/high_fidelity_screens/latest
./tools/visual_gap_audit.sh artifacts/visual_review/latest
./tools/visual_benchmark.sh artifacts/visual_review/latest
```

Visual evidence requires native 3D renderer/engine captures with current-run metadata. If the renderer path is absent, the correct status is blocked, while nonvisual truth/replay/build/package evidence can still pass. See `docs/visual/THREE_D_ONLY_VISUAL_EVIDENCE.md` and `docs/decisions/0002-remove-2d-visual-artifacts.md`.

## Other focused gates

```sh
./tools/audit_truth.sh
./tools/audit_secrets.sh
./tools/audit_environment.sh
./tools/contact_matrix.sh artifacts/contact_matrix/latest
./tools/ai_duel.sh artifacts/ai/latest 6
./tools/ai_sweep.sh artifacts/ai_sweep/latest
./tools/truth_stress.sh artifacts/truth_stress/latest
./tools/truth_edge_audit.sh artifacts/truth_edge/latest
./tools/negative_audit.sh artifacts/negative_audit/latest
./tools/audio_vfx_render.sh examples/duels/basic_oathyard.duel artifacts/audio_vfx/latest
./tools/package.sh
./tools/smoke_package.sh artifacts/package/oathyard-linux-x86_64.tar
```

These gates preserve deterministic truth, replay loud-failure behavior, no-HP/no-stat canon, readiness false flags, and presentation/truth isolation.

## Full verification

```sh
./tools/verify.sh
```

`verify.sh` runs build/test, asset build/validation, deterministic duel twice, replay verification, replay export/verification, truth audits, negative audits, AI/truth stress checks, environment/native status checks, and `audit_visual_artifacts.sh`. It fails if normal verification generates forbidden standalone visual files or if visual readiness is claimed without native 3D renderer evidence.

## Publishable local gate

```sh
./tools/publishable_gate.sh
```

This wraps local verification into a timestamped evidence bundle under `artifacts/publishable/<UTC>/`. It can only produce a local package-candidate result; it does not claim owner-final acceptance, public demo readiness, release-candidate readiness, legal clearance, trademark clearance, or store readiness.
