# OATHYARD M0-M21 Canon / Acceptance Decomposition

Status date: 2026-06-29

This file decomposes the existing grouped roadmap into M0-M21 pullable milestone surfaces. It is planning and acceptance glue only: it does not lower gates, does not claim source-code completion, and does not convert local verification evidence into product, owner, public-demo, release-candidate, legal, trademark, or store readiness.

## Source precedence preserved

1. `docs/design/GAME_CANON.md`
2. `docs/design/DEMO_SCOPE.md`
3. `ACCEPTANCE_MAP.md`
4. `AGENTS.md` / `CLAUDE.md`
5. PRDs/specs
6. Code comments

Controlling locks from those sources remain unchanged:

- Truth runs at fixed 120 Hz and uses deterministic integer/fixed-point gameplay state only.
- No hidden RNG, wall-clock gameplay truth, gameplay floats, unordered truth iteration, or presentation writes into truth.
- No HP, hit points, arbitrary damage numbers, armor points, DPS, crit chance, super meter, perks, unlock stats, or speed/damage bonuses as truth.
- Replay is authoritative evidence and must fail loudly on mismatch.
- Renderer, UI, audio, VFX, camera, settings, input, and fight-film systems consume truth after hashing and never mutate truth.
- Blocked native-renderer status, non-native diagram, non-native frame, low-poly glTF, diagnostic image rollups, non-native local raster captures, and metadata-only checks are local verification evidence only; they do not prove high-fidelity product presentation.
- Production renderer completion, owner visual/input/audio acceptance, public-demo readiness, release-candidate readiness, legal clearance, trademark clearance, and store readiness remain false until separately evidenced.

## Dependency spine

`M0-M1 canon/control -> M2-M6 deterministic foundation -> M7-M8 source-backed assets/content -> M9-M14 native playable/evidence surfaces -> M15-M17 local ship-quality gates -> M18-M21 production/external release gates`

Do not pull public/store release work before M18-M20 evidence exists. Local package success is not public/store readiness.

## Milestone gap / dependency table

| Milestone | Acceptance surface | Current state / gap | Primary dependency | Next executable card | Verification commands |
| --- | --- | --- | --- | --- | --- |
| M0 | Canon and forbidden-shortcut lock. | Source hierarchy and hard locks exist; every future card must re-check them. | None. | `M0-CANON-AUDIT-001` (`gamedesign`): before each readiness-impacting change, compare touched docs/code against canon/acceptance locks. | `./tools/audit_truth.sh`; `./tools/audit_readiness.sh . artifacts/readiness/source` |
| M1 | Acceptance map, roadmap, and pull-source discipline. | This decomposition now makes the M0-M21 split explicit; board must stay evidence-backed. | M0. | `M1-BOARD-DRIFT-001` (`gamedesign`): keep `ACCEPTANCE_MAP.md`, roadmap, and board cards synchronized without readiness inflation. | `./tools/audit_readiness.sh . artifacts/readiness/source`; `./tools/audit_secrets.sh . artifacts/secrets/source` |
| M2 | Native Rust source build and test harness. | Local build/test gates exist; no baseline commit/remote yet. | M0-M1. | `M2-BASELINE-VCS-001` (`developer`, owner-gated): create owner-approved baseline commit/remote policy without generated artifacts or secrets. | `./tools/build.sh`; `./tools/test.sh`; `cargo build --locked`; `cargo test --locked`; `git status --short --branch` |
| M3 | Fixed-step deterministic truth kernel and action/cost model. | Current evidence exists; broader property-style truth-helper sweeps remain useful. | M2. | `M3-TRUTH-SWEEP-001` (`developer`): extend deterministic helper sweeps without RNG/floats/HP/stat shortcuts. | `./tools/audit_truth.sh`; `./tools/truth_stress.sh artifacts/truth_stress/verify`; `./tools/truth_edge_audit.sh artifacts/truth_edge/verify` |
| M4 | Body/material/capability health model and contact solve. | Contact matrix, material/capability cause chains, and edge audits exist; broader solver stress can deepen coverage. | M3. | `M4-CONTACT-MATRIX-DEPTH-001` (`developer`): add exhaustive action x weapon x armor x target invariant coverage as fixtures/gates expand. | `./tools/contact_matrix.sh artifacts/contact_matrix/verify`; `./tools/run_match_sweep.sh`; `./tools/audit_truth.sh` |
| M5 | Replay, replay verification, fight-film manifests, and export bundle chain. | Replay and bundle gates exist; richer player-facing replay UX remains separate. | M3-M4. | `M5-REPLAY-UX-001` (`developer`): expose replay verification/hash failure state in native replay UX without bypassing replay verification. | `./tools/replay_verify.sh artifacts/latest/replay.json`; `./tools/export_replay_bundle.sh artifacts/latest/replay.json artifacts/export_bundle/verify`; `./tools/verify_replay_bundle.sh artifacts/export_bundle/verify` |
| M6 | Loud-failure audits: stress, edge, negative, corrupt replay/content/bundle inputs. | Current gates exist; expand only as parser/schema surfaces grow. | M3-M5. | `M6-NEGATIVE-SURFACE-001` (`developer`): add negative cases for any new parser/content/replay/package surface. | `./tools/negative_audit.sh artifacts/negative_audit/verify`; `./tools/truth_edge_audit.sh artifacts/truth_edge/verify`; `./tools/negative_audit.sh artifacts/negative_audit/verify` |
| M7 | Source-backed asset pipeline, manifests, provenance, runtime glTF, local validation. | Local text-spec/glTF pipeline exists; external DCC/Khronos validation remains blocked. | M0-M2. | `M7-ASSET-TOOLCHAIN-001` (`developer`): fix/install Blender/glTF tooling or write ADR preserving deterministic text-spec pipeline as intentional. | `./tools/build_assets.sh`; `./tools/validate_assets.sh`; `./tools/render_asset_previews.sh artifacts/asset_previews/verify`; `./tools/asset_visual_atlas.sh artifacts/asset_atlas/verify`; `./tools/audit_3d_runtime.sh artifacts/runtime_3d/verify assets/runtime_manifest.json artifacts/native_combat/verify/native_combat_render_manifest.json` |
| M8 | Content breadth and 3D runtime asset budgets. | Current generated runtime artifacts show six fighters / six armor families / eight weapon families / two arenas, for 22 runtime assets. This satisfies the minimum weapon-family count but not production fidelity; any stale 21-asset/7-weapon expectations are drift to fix without weakening gates. | M7. | `M8-CONTENT-BREADTH-001` (`developer` + `mediaqa`): add source-backed weapon/content breadth and budget evidence without placeholder production claims. | `./tools/build_assets.sh`; `./tools/validate_assets.sh`; `./tools/asset_budget_audit.sh artifacts/asset_budget/verify`; `./tools/native_roster_showcase.sh artifacts/native_roster/verify` |
| M11 | Native input boundary, remap schema, keyboard/mouse/default gamepad command coverage. | Local schema/command-flow evidence exists; physical controller ergonomics, Steam Deck hardware, and owner input acceptance remain false. | M9-M10. | `M11-PHYSICAL-INPUT-SMOKE-001` (`desktopcontrol` + `mediaqa`): run physical controller/Steam Deck smoke when hardware is present; otherwise keep blocker explicit. | `./tools/input_map.sh artifacts/input/verify`; `./tools/gamepad_smoke.sh artifacts/gamepad/verify`; `./tools/input_target_audit.sh artifacts/input_target/verify` |
| M13 | Deterministic AI/scripted seats and match sweeps. | Current AI duel/sweep and match sweep exist; broader property-style match variation can deepen evidence. | M3-M6, M8. | `M13-AI-MATCH-DEPTH-001` (`developer`): extend legal-action AI sweeps without allowing AI to decide contacts/injuries/hashes. | `./tools/ai_duel.sh artifacts/ai/verify 6`; `./tools/ai_sweep.sh artifacts/ai_sweep/verify`; `./tools/run_match_sweep.sh`; `./tools/replay_verify.sh artifacts/ai/verify/replay.json` |
| M15 | Native presentation renderer/readability local evidence. | Blocked native-renderer status/non-native frame/software 3D evidence exists and `game_is_3d` must stay true; production renderer completion remains false. | M7-M14. | `M15-PLAYER-FACING-RENDER-LOOP-001` (`developer` + `mediaqa`): build richer continuous player-facing 3D loop/captures or record backend ADR blocker; no high-fidelity claim until M18-M20. | `./tools/native_combat_render.sh examples/duels/basic_oathyard.duel artifacts/native_combat/verify`; `./tools/native_roster_showcase.sh artifacts/native_roster/verify`; `./tools/renderer_target_audit.sh artifacts/renderer_target/verify` |
| M16 | Audio/VFX, captions, mixer, settings, bounded local device smoke. | Trace-derived audio/VFX and local device smoke exist; shipping backend, platform loudness, and owner audio acceptance remain false. | M5, M12, M15. | `M16-AUDIO-BACKEND-ADR-001` (`developer` + `mediaqa`): choose or reject shipping audio backend with license/package/determinism smoke evidence. | `./tools/audio_vfx_render.sh examples/duels/basic_oathyard.duel artifacts/audio_vfx/verify`; `./tools/audio_mixer.sh examples/duels/basic_oathyard.duel artifacts/audio_mixer/verify`; `./tools/audio_device_smoke.sh examples/duels/basic_oathyard.duel artifacts/audio_device/verify`; `./tools/audio_target_audit.sh artifacts/audio_target/verify` |
| M17 | Performance, environment, local package, package smoke, reproducibility, desktop metadata. | Local package candidate gates exist; clean VM/container and public/store release remain separate. | M2-M16. | `M17-FRESH-PACKAGE-GATE-001` (`automation` + `developer`): run a fresh timestamped local package/publishable gate and reduce outputs. | `./tools/perf_benchmark.sh artifacts/perf/verify`; `./tools/audit_environment.sh artifacts/environment/verify`; `./tools/package.sh`; `./tools/smoke_package.sh artifacts/package/oathyard-linux-x86_64.tar`; `./tools/check_package_repro.sh artifacts/package_repro/verify`; `./tools/publishable_gate.sh` |
| M18 | High-fidelity production renderer/backend and 1920x1080+ capture coverage. | Not passed. Current local 3D captures remain prototype/debug-level evidence only, and alternate renderer spike implementations have been pruned from source. | M15-M17 and renderer/backend ADR. | `M18-HIFI-RENDERER-SPIKE-001` (`developer` + `mediaqa`): measure renderer/backend candidate, capture path, frame timing, license/dependency footprint, and truth boundary before adoption without retaining alternate renderer implementation source unless a later ADR explicitly accepts it. | `./tools/audit_environment.sh artifacts/environment/verify`; `./tools/renderer_target_audit.sh artifacts/renderer_target/verify`; current-run visual inspection of generated captures |
| M19 | Production assets, animation/material/VFX art depth, visual benchmark report. | Not passed. Current assets are low-poly/local structural evidence, below high-fidelity target. | M7-M8, M15, M18. | `M19-PRODUCTION-ASSET-PACK-001` (`mediaqa` + `developer`): produce source-backed production asset/capture pack and benchmark report; reject copied/unlicensed/placeholder assets. | `./tools/build_assets.sh`; `./tools/validate_assets.sh`; `./tools/asset_budget_audit.sh artifacts/asset_budget/verify`; `./tools/native_combat_render.sh examples/duels/basic_oathyard.duel artifacts/native_combat/verify`; vision/owner inspection of `artifacts/visual_review/latest/visual_benchmark_report.md` inputs |
| M20 | Owner/human visual, input, audio, accessibility, and demo-scope acceptance. | Not performed. Automated gates cannot mark owner acceptance true. | M18-M19 plus current capture/audio/input packs. | `M20-OWNER-ACCEPTANCE-PACK-001` (`mediaqa`): prepare current evidence pack and checklist; owner explicitly accepts or rejects each domain. | `./tools/visual_evidence_index.sh artifacts/visual_evidence/verify`; `./tools/input_target_audit.sh artifacts/input_target/verify`; `./tools/audio_target_audit.sh artifacts/audio_target/verify`; owner review artifact recorded separately |
| M21 | License/legal/trademark/store/public-demo/release-candidate gates. | Blocked/external. `LICENSE` remains pending/unlicensed; store credentials and legal/trademark clearance are absent. | M17-M20 and owner decisions. | `M21-RELEASE-BLOCKER-TRIAGE-001` (`gamedesign`, owner-gated): reduce every release blocker to a smallest owner/legal/store action without flipping readiness flags. | `./tools/audit_readiness.sh . artifacts/readiness/source`; `./tools/audit_secrets.sh . artifacts/secrets/source`; package audits after `./tools/package.sh`; external legal/store evidence only when actually performed |

## Exact pull list / current status

These are the highest-leverage executable cards after this M1 decomposition. Assignee names are existing Hermes profiles observed locally: `developer`, `mediaqa`, `desktopcontrol`, `automation`, `gamedesign`, `localmodels`.

| Pull order | Card | Profile | Entry condition | Done evidence |
| --- | --- | --- | --- | --- |
| 1 | `M1-BOARD-DRIFT-001` | `gamedesign` | Any doc/roadmap/card update. | `ACCEPTANCE_MAP.md`, roadmap, and this file agree on false readiness gates; `./tools/audit_readiness.sh . artifacts/readiness/source` passes. |
| 2 | `M17-VERIFY-ASSET-COUNT-DRIFT-001` | `developer` | Completed by `t_c61f8b81` / `t_6c90f64a`. | `./tools/verify.sh` passes with 22 runtime assets, eight weapon families, 292 vertices, 492 triangles, and false public/owner/release flags. Evidence: `artifacts/final_acceptance/latest/post_billhook_verify_20260629T183424Z/summary.txt`. |
| 3 | `M17-FRESH-PACKAGE-GATE-001` | `automation` + `developer` | Before any readiness improvement is reported. | Fresh timestamped logs from `./tools/publishable_gate.sh`, package smoke, final replay hash, package SHA-256, skipped-check/blocker list. |
| 4 | `M15-PLAYER-FACING-RENDER-LOOP-001` | `developer` + `mediaqa` | M9-M14 local gates green. | Continuous native player-facing 3D loop/capture evidence, `./tools/renderer_target_audit.sh artifacts/renderer_target/verify` green, production renderer completion still false unless M18 acceptance is met. |
| 5 | `M18-HIFI-RENDERER-SPIKE-001` | `developer` + `mediaqa` | Candidate backend or dependency-zero renderer path chosen for measurement. | ADR with license/build/package/capture/input/audio/truth-boundary measurements; no dependency adopted without measured value and removal plan. |
| 6 | `M7-ASSET-TOOLCHAIN-001` | `developer` | Asset production path needed for M19. | Blender/glTF tools fixed with version logs, or ADR explicitly preserves deterministic text-spec pipeline; asset gates remain green. |
| 7 | `M19-PRODUCTION-ASSET-PACK-001` | `mediaqa` + `developer` | Renderer/capture path can show production assets. | Source-backed high-fidelity asset/capture pack, provenance, benchmark report, no copied/unlicensed/placeholder assets. |
| 8 | `M11-PHYSICAL-INPUT-SMOKE-001` | `desktopcontrol` + `mediaqa` | Physical controller/Steam Deck hardware available. | Hardware smoke evidence or explicit hardware-unavailable blocker; schema-only evidence not promoted to hardware acceptance. |
| 9 | `M16-AUDIO-BACKEND-ADR-001` | `developer` + `mediaqa` | Shipping audio needs runtime backend. | ADR/spike proves package-stable playback path, loopback/platform measurement plan, captions, settings persistence, and false owner/platform acceptance until reviewed. |
| 10 | `M20-OWNER-ACCEPTANCE-PACK-001` | `mediaqa` | M18-M19 current capture pack exists. | Owner signs visual/input/audio/accessibility/demo-scope acceptance or rejects with notes; automated gates alone do not pass this card. |
| 11 | `M21-RELEASE-BLOCKER-TRIAGE-001` | `gamedesign` | Owner wants public/store path after M20. | License/legal/trademark/store blockers have exact owner-side actions; public-demo/release-candidate/store readiness flags remain false until evidence exists. |

## Current verification drift watchpoint

The runtime asset floor is now eight weapon families and 22 source-backed runtime assets. If generated audits report 22 assets while a verifier still expects older 21-asset / 7-weapon literals, update the source verifier contract or derive it from the current manifest; do not edit generated artifacts to satisfy stale expectations.

## Standard verification bundle

Focused card work should run the relevant milestone commands above. Any readiness-impacting report should additionally run fresh gates in a timestamped evidence directory and include:

```sh
./tools/build.sh
./tools/test.sh
cargo build --locked
cargo test --locked
./tools/audit_truth.sh
./tools/audit_readiness.sh . artifacts/readiness/source
./tools/audit_secrets.sh . artifacts/secrets/source
./tools/verify.sh
```

Final readiness-sensitive evidence must also name the final replay hash, content hash, package SHA-256, deterministic A/B comparison result, visual image-rollup path, skipped checks, blocker classes, and current false external readiness flags.
