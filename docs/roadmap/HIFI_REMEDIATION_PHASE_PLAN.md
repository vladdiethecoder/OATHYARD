# OATHYARD high-fidelity remediation phase plan

Status: actionable remediation spine from kanban task `t_cb68c544`.
Timestamp: 2026-06-29T22:38:36Z UTC.
Scope: consolidate design-standard audit `t_7d209020` and visual-fidelity audit `t_614cc73b` into a prioritized execution plan using the existing HIFI child cards; no new broad implementation cards are required.

## Source basis

Controlling sources, in precedence order:

1. `docs/design/GAME_CANON.md`
2. `docs/design/DEMO_SCOPE.md`
3. `ACCEPTANCE_MAP.md`
4. `AGENTS.md`
5. `docs/roadmap/HIGH_FIDELITY_PRODUCTION_WORK_ORDERS.md`
6. Parent audit artifacts:
   - `/home/vdubrov/.hermes/kanban/boards/oathyard-full-game/workspaces/t_7d209020/OATHYARD_ASSET_WORLD_ARENA_CANON_AUDIT.md`
   - `/home/vdubrov/.hermes/kanban/boards/oathyard-full-game/workspaces/t_614cc73b/visual_audit_current/OATHYARD_visual_fidelity_audit_t_614cc73b.md`

Hard facts from the audits:

- Current visuals fail the full-game visual bar: ultra-low-poly assets, flat materials, sparse/flat arenas, text-driven combat feedback, and debug/HUD overlays dominate the evidence (`t_614cc73b` report lines 7-12, 62-271).
- Current runtime asset breadth exists but is placeholder-scale: 22 assets, 292 vertices, 492 triangles total; fighters are 14 vertices / 28 triangles and arenas are 18 vertices / 32 triangles (`t_7d209020` report lines 40-49, 54-75).
- Existing local gates prove deterministic source-backed 3D evidence only; raw X11/XWayland, SVG, PPM, low-poly glTF, and software-raster captures do not prove high-fidelity product presentation, owner acceptance, public-demo readiness, or release-candidate readiness (`GAME_CANON.md:9-15`, `DEMO_SCOPE.md:15-21`, `ACCEPTANCE_MAP.md:32-42`).
- Current scenario/match evidence does not expose arena/world selection in the duel grammar and current native render evidence hard-codes `oathyard_verdict_ring`, leaving `training_yard` unexercised as a scene (`t_7d209020` report lines 275-294, 351-369; `t_614cc73b` report lines 181-197).
- The candidate lane `t_73291be5` materially improves structural package quality but is not native product visual acceptance or owner acceptance; it currently also has a path/name mismatch follow-up (`t_f72f4c22`) that must close before asset-source acceptance can be clean.

## Non-negotiable boundaries for every phase

- Truth stays fixed 120 Hz and deterministic integer/fixed-point only.
- Renderer, UI, audio, VFX, camera, fight-film, settings, and accessibility consume truth/replay/trace data after authoritative hashes and never mutate contacts, injuries, capability deltas, action validity, end state, replay JSON, trace JSON, content hashes, or final hashes.
- No HP, hit points, arbitrary damage numbers, armor points, DPS, crit chance, super meter, perks, unlock stats, or speed/damage bonuses as truth.
- No Unity, Unreal, Godot, browser-first product renderer, copied/unlicensed assets, telemetry, network services, installers, or vendored graphics blobs in this remediation slice unless a later owner-approved ADR explicitly supersedes the current canon.
- Public-demo, release-candidate, owner-final, owner-visual, legal, trademark, and store readiness remain false until the corresponding external gates are actually evidenced.
- Raw X11/PPM/SVG/debug/contact-sheet evidence may support local verification but cannot be reported as high-fidelity product presentation.

## Effort scale

- S: <= 1 focused engineer-day.
- M: 1-3 focused engineer-days.
- L: 3-7 focused engineer-days.
- XL: 1-3 weeks or multi-profile work, usually asset/renderer/UI integration plus media QA.
- XXL: open-ended external/owner acceptance or full production-art pass.

## Current HIFI card register and state

The plan uses existing cards; do not create duplicate vague implementation cards for these surfaces.

| Card | Role in plan | Owner-profile suggestion | Current board state | Operational state | Work type |
| --- | --- | --- | --- | --- | --- |
| `t_cb68c544` | This remediation plan | `gamedesign` | running | unblocked; complete by writing this plan and handoff | design-doc update |
| `t_f72f4c22` | Fix model-candidate package path mismatch found by media QA | `developer` | running | unblocked active prerequisite for clean asset-source acceptance | code/config or spec correction |
| `t_a95fe445` | HIFI source-backed asset pack / production-target asset quality | `mediaqa` with `developer` support | running | active, with explicit dependency edge from `t_f72f4c22`; acceptance should wait for path resolution and direct visual review | asset + QA |
| `t_54eabe83` | HIFI rig/skin/deformation schema | `developer`; `mediaqa` deformation review | todo | dependency-blocked on `t_a95fe445` | code/config + asset integration |
| `t_62c8e2af` | HIFI PBR/equivalent materials and event-keyed damage/wear masks | `developer`; `mediaqa` material review | todo | dependency-blocked on `t_a95fe445` | code/config + asset/material |
| `t_c7afd55b` | Animation state machine over truth events | `developer`; `mediaqa` pose/contact review | todo | dependency-blocked on `t_54eabe83` | code/config + animation |
| `t_6aea2f80` | Production-target arena/environment art for verdict ring and training yard | `developer`; `mediaqa` environment review | todo | dependency-blocked on `t_a95fe445` | asset creation + config |
| `t_5738a1d3` | Actual native 3D renderer backend implementation | `developer` | todo | dependency-blocked on renderer epic/spike evidence `t_05a6f650` and loop task `t_f175e621` | code/config |
| `t_7365cf05` | 1920x1080+ deterministic capture matrix | `developer`; `mediaqa` inspection | todo | dependency-blocked on renderer loop/backend evidence | code/config + QA |
| `t_26ed8543` | Lighting/atmosphere | `developer`; `mediaqa` lighting review | todo | dependency-blocked on renderer backend and asset/material readiness | code/config + visual QA |
| `t_faf2fd4c` | Post-processing/readability pipeline | `developer`; `mediaqa` review | todo | dependency-blocked on `t_26ed8543` and renderer/capture | code/config |
| `t_0e005e0f` | Combat/replay/fight-film camera language | `developer`; `mediaqa` frame review | todo | dependency-blocked on renderer/backend and animation state machine | code/config + camera |
| `t_8aebf932` | Truth-driven impact VFX/particles | `developer`; `mediaqa` frame review | todo | dependency-blocked on materials, animation, renderer | code/config + VFX |
| `t_3bafc3e1` | Production-facing audio runtime integration | `developer`; `mediaqa` audio review | todo | dependency-blocked on integration epic and trace/replay event surfaces | code/config + audio assets |
| `t_be25d936` | Native HUD/UI/menu/match flow | `developer`; `desktopcontrol` GUI smoke; `mediaqa` UI review | todo | dependency-blocked on renderer backend/capture and integration surfaces | code/config + UI |
| `t_3eb8f26b` | Settings/accessibility surfaces | `developer`; `mediaqa` accessibility review | todo | dependency-blocked on `t_be25d936` | code/config + accessibility |
| `t_3cd9ed70` | Controller/remap ergonomics | `developer`; `desktopcontrol` when hardware/UI exercise needed | todo | dependency-blocked on `t_be25d936` and full-program input context | code/config + input QA |
| `t_609f0e85` | Deterministic AI opponent/player-seat polish | `developer` | todo | dependency-blocked on full-program context; not a visual blocker but needed for product loop | code/config |
| `t_da27c022` | Performance profiling/optimization | `developer` | todo | dependency-blocked on renderer backend and capture matrix | code/config + performance QA |
| `t_c77e54b6` | HIFI visual benchmark packet | `mediaqa`; `gamedesign` wording support | todo | hard dependency-blocked on all renderer/assets/materials/rig/camera/UI/capture/perf children | QA + design-doc |
| `t_58035fe0` | Owner acceptance review handoff | `mediaqa` for packet handoff only; owner signs | todo | hard dependency-blocked on `t_c77e54b6`; automated agents cannot sign acceptance | external review |
| `t_71790e5c` | VFX/audio/camera/UI integration epic | `developer` | todo | epic only; do not implement as monolith | coordination/epic |
| `t_05a6f650` | Renderer ADR/loop epic | `developer` | todo | epic/review gate; first advance `t_f175e621`, then `t_5738a1d3` | coordination/epic |
| `t_34a5af6b` | Original broad implementation catch-all | `developer` | todo | supersede/aggregate only; do not dispatch as vague implementation | coordination/legacy |

Known board hygiene issue:

- `t_5b0b0f71` is referenced by older comments/docs as a work-order package root, but `kanban_show t_5b0b0f71` currently returns task-not-found on the active board. Treat `docs/roadmap/HIGH_FIDELITY_PRODUCTION_WORK_ORDERS.md`, not that missing card id, as the work-order source.
- Some epic edges currently make implementation children depend on broad epic cards (`t_71790e5c`, `t_05a6f650`). Operationally, these epics must remain coordination/acceptance aggregators and should not be worked as monoliths. If board tooling permits, a board operator should invert or clear those epic-blocking edges; until then, this document is the execution spine.
- This plan run encoded two concrete dependency edges on the board: `t_f72f4c22 -> t_a95fe445` for the asset package path mismatch, and `t_5738a1d3 -> t_7365cf05` so the 1920x1080 capture matrix waits for the actual renderer backend implementation instead of only the renderer epic.

## Ordered dependency spine

This is the remediation order. Later phases must not claim completion from earlier-phase metadata or stale logs; each phase needs current-run evidence.

```text
0. Current plan handoff
   t_cb68c544

1. Asset package path and source-pack acceptance
   t_f72f4c22 -> t_a95fe445

2. Production asset/world schema and arena coverage
   t_a95fe445 -> t_6aea2f80
   t_a95fe445 -> t_54eabe83
   t_a95fe445 -> t_62c8e2af

3. Renderer/backend and capture foundation
   t_f175e621 -> t_05a6f650 -> t_5738a1d3 -> t_7365cf05

4. Material, lighting, rig, animation foundations
   t_62c8e2af -> t_26ed8543 -> t_faf2fd4c
   t_54eabe83 -> t_c7afd55b

5. Combat presentation integration
   t_c7afd55b + t_62c8e2af + t_5738a1d3 -> t_0e005e0f
   t_c7afd55b + t_62c8e2af + t_5738a1d3 -> t_8aebf932
   replay/trace events + UI/settings surfaces -> t_3bafc3e1

6. Product UI/input/accessibility/performance
   t_5738a1d3 + t_7365cf05 -> t_be25d936
   t_be25d936 -> t_3eb8f26b
   t_be25d936 -> t_3cd9ed70
   t_0d8e1bbf context -> t_609f0e85
   t_5738a1d3 + t_7365cf05 -> t_da27c022

7. Verification packet and owner handoff
   all phase 1-6 children -> t_c77e54b6 -> t_58035fe0

8. Legacy broad catch-all handling
   t_34a5af6b should be closed/superseded or used only as an aggregate after the concrete HIFI children above have completed. It should not perform broad implementation.
```

## Phase 0 — plan and board-spine hygiene

Purpose: freeze this task's output as the canonical remediation routing artifact and prevent duplicate vague implementation work.

Cards:

- `t_cb68c544` — this plan.
- `t_34a5af6b` — legacy broad implementation card, aggregate/supersede only.
- `t_71790e5c` and `t_05a6f650` — epics/coordination only.

Effort: S.

Acceptance criteria:

- A written remediation plan exists in `docs/roadmap/HIFI_REMEDIATION_PHASE_PLAN.md`.
- The plan references existing HIFI child cards by id and maps every major audit finding to a concrete card.
- The plan states which cards are broad epics/legacy aggregates and must not be used as monolithic implementation work.
- No new vague implementation card is created.
- Readiness language remains fail-closed.

Verification commands:

```sh
python3 - <<'PY'
from pathlib import Path
p = Path('docs/roadmap/HIFI_REMEDIATION_PHASE_PLAN.md')
s = p.read_text(encoding='utf-8')
required = [
    't_f72f4c22', 't_a95fe445', 't_5738a1d3', 't_7365cf05',
    't_6aea2f80', 't_62c8e2af', 't_54eabe83', 't_c7afd55b',
    't_26ed8543', 't_faf2fd4c', 't_0e005e0f', 't_8aebf932',
    't_3bafc3e1', 't_be25d936', 't_3eb8f26b', 't_3cd9ed70',
    't_da27c022', 't_c77e54b6', 't_58035fe0', 't_34a5af6b',
    'supersede/aggregate only', 'owner_visual_acceptance',
    'renderer, UI, audio, VFX, camera, fight-film, settings, and accessibility consume truth/replay/trace data after authoritative hashes'
]
missing = [x for x in required if x not in s]
if missing:
    raise SystemExit('missing required plan terms: ' + ', '.join(missing))
print('HIFI remediation plan structural check passed')
PY
./tools/audit_readiness.sh . artifacts/readiness/hifi_remediation_phase_plan
```

## Phase 1 — unblock asset-source acceptance

Primary defects addressed:

- ASSET-BLOCKER-001 / A1: runtime assets are placeholder low-poly geometry.
- ASSET-MAJOR-007: candidate lane is not integrated/accepted current game asset lane.
- Path mismatch from `t_db892c74`: available audited paths are `assets/model_candidates/t_73291be5` and `artifacts/model_candidates/t_73291be5`, while one task variant expected `package_run_id=t73291be5` aliases.

Cards:

- `t_f72f4c22` — fix model-candidate package path mismatch. Owner: `developer`. Type: code/config or spec correction. State: running/unblocked.
- `t_a95fe445` — HIFI asset source pack. Owner: `mediaqa` with `developer` support. Type: asset-source QA + asset creation gate. State: running; acceptance should wait for `t_f72f4c22` resolution and current direct visual inspection.

Effort: M for path/spec correction; XL/XXL for actual production-quality asset art if the candidate package is rejected visually.

Acceptance criteria:

- `t_f72f4c22` either creates deterministic repo-owned alias/symlink/copy paths matching the expected path contract or updates the incorrect board/docs/spec reference without weakening media QA.
- `t_a95fe445` confirms the source-backed package covers six fighters, eight weapons, six armor/loadout families, and two arenas with provenance/source hashes/runtime hashes.
- Media QA inspects actual current images/contact sheets, not only manifests, and records named blockers for any asset below target.
- Any accepted package still keeps owner/public/release/native-DCC/external-validation readiness false unless separately evidenced.

Exact verification commands:

```sh
./tools/build_assets.sh
./tools/validate_assets.sh
./tools/asset_budget_audit.sh artifacts/asset_budget/t_a95fe445
./tools/render_asset_previews.sh artifacts/asset_previews/t_a95fe445
./tools/asset_visual_atlas.sh artifacts/asset_atlas/t_a95fe445
./tools/native_combat_render.sh examples/duels/basic_oathyard.duel artifacts/native_combat/t_a95fe445
./tools/audit_3d_runtime.sh artifacts/runtime_3d/t_a95fe445 assets/runtime_manifest.json artifacts/native_combat/t_a95fe445/native_combat_render_manifest.json
./tools/audit_readiness.sh . artifacts/readiness/t_a95fe445
./tools/audit_secrets.sh . artifacts/secrets/t_a95fe445
python3 tools/model_candidates/audit_model_candidate_lane.py --run-id t_73291be5 --require-vision-audit
```

Do not count phase 1 passed from structural glTF counts alone. The visual audit explicitly rejected local placeholder pixels; current-run pixel/media inspection must say what still fails.

## Phase 2 — production asset/world schema and arena coverage

Primary defects addressed:

- ASSET-MAJOR-003/004/005/006: fighter, weapon, armor, and source metadata are too thin for product art.
- WORLD-MAJOR-002/003/004: arena/world selection and environment schema are too shallow.
- ARENA-BLOCKER-001 / AR1: verdict ring is tokenized low-poly ring, not ritual arena.
- ARENA-MAJOR-002 / AR2: training yard exists in manifest but is not scene-validated.

Cards:

- `t_6aea2f80` — production-target arena/environment art for `oathyard_verdict_ring` and `training_yard`. Owner: `developer`; `mediaqa` review. Type: asset creation + config. State: dependency-blocked on `t_a95fe445`.
- `t_54eabe83` — rig/skin/deformation schema. Owner: `developer`; `mediaqa` review. Type: code/config + asset integration. State: dependency-blocked on `t_a95fe445`.
- `t_62c8e2af` — material schema and trace-driven damage/wear masks. Owner: `developer`; `mediaqa` review. Type: code/config + material assets. State: dependency-blocked on `t_a95fe445`.

Effort: XL for arena/environment art; L/XL for rig/material schema depending on whether the candidate lane is accepted as source.

Acceptance criteria:

- Arena source schema includes structured collision/footing zones, start positions, contact-safe floor masks, camera anchors, lighting anchors, weather/atmosphere hooks, witness/judgment/weapon-staging landmarks, and capture ids.
- Truth-affecting collision/footing data is hashed and replayed; lighting/camera/weather remains presentation-only.
- Both arenas have source-backed nonzero-Z runtime geometry, material zones/maps, lighting anchors, and gameplay-distance + closeup captures.
- Scenario/match/capture paths can exercise both `oathyard_verdict_ring` and `training_yard`; training yard cannot remain atlas-only.
- Fighter/weapon/armor specs include per-asset silhouette, material families, rig/topology notes, contact profile, no-clipping requirements, and QA blocker status.

Exact verification commands:

```sh
./tools/build_assets.sh
./tools/validate_assets.sh
./tools/asset_budget_audit.sh artifacts/asset_budget/t_6aea2f80
./tools/asset_visual_atlas.sh artifacts/asset_atlas/t_6aea2f80
./tools/native_combat_render.sh examples/duels/basic_oathyard.duel artifacts/native_combat/t_6aea2f80
./tools/audit_3d_runtime.sh artifacts/runtime_3d/t_6aea2f80 assets/runtime_manifest.json artifacts/native_combat/t_6aea2f80/native_combat_render_manifest.json
./tools/audit_truth.sh
./tools/audit_readiness.sh . artifacts/readiness/t_6aea2f80
```

Additional design-doc acceptance if schema docs change:

```sh
python3 - <<'PY'
from pathlib import Path
for path in ['docs/design/ART_DIRECTION_BRIEF.md', 'docs/asset_pipeline/ASSET_PIPELINE.md']:
    if Path(path).exists():
        print(path, 'bytes', Path(path).stat().st_size)
PY
```

## Phase 3 — renderer/backend and 1920x1080 capture foundation

Primary defects addressed:

- WORLD-BLOCKER-001 / W1: current world presentation is local/debug shell.
- Visual audit: current evidence is prototype/debug and not product presentation.
- HIFI-WO-01/HIFI-WO-02: need continuous native product loop and deterministic 1920x1080+ capture matrix.

Cards:

- `t_f175e621` — native 3D render loop without mutating game state. Owner: `developer`. Type: code/config. State: todo but dependency/review-blocked by `t_5649e61c` and current canonical verification freshness.
- `t_05a6f650` — renderer ADR/loop epic. Owner: `developer`. Type: coordination/epic; do not redo completed ADR/audit unless stale. State: todo; parent acceptance requires child evidence.
- `t_5738a1d3` — actual renderer backend implementation. Owner: `developer`. Type: code/config. State: dependency-blocked on renderer loop/epic evidence.
- `t_7365cf05` — 1920x1080+ deterministic capture matrix. Owner: `developer`; `mediaqa` review. Type: code/config + QA. State: dependency-blocked on renderer backend/loop evidence.

Effort: L for smallest real backend loop if the raw native path remains viable; XL if renderer backend has to move beyond the current spike.

Acceptance criteria:

- Renderer/backend has actual continuous player-facing native loop, not still-frame-only artifact generation.
- One backend can display menu/select/planning/combat/replay states or has a concrete incremental path to those states; frame timing is measured outside truth.
- Renderer consumes replay/trace/presentation input through an explicit schema after truth hashes.
- Replay JSON, trace JSON, contacts, injuries, capability deltas, action validity, end state, content hash, and final hash are unchanged by rendering.
- Capture matrix includes every required HIFI-WO-02 state at >= 1920x1080 without upscaling.
- Capture manifest includes command, replay path/hash, content/asset hashes, backend id, resolution, camera mode, frame/tick, sha256, `truth_mutation=false`, and `owner_visual_acceptance=false`.

Exact verification commands:

```sh
./tools/run_duel.sh examples/duels/basic_oathyard.duel --out artifacts/hifi_renderer/t_f175e621/duel
./tools/replay_verify.sh artifacts/hifi_renderer/t_f175e621/duel/replay.json
./tools/renderer_target_audit.sh artifacts/renderer_target/t_5738a1d3
./tools/capture_high_fidelity_screens.sh artifacts/hifi_captures/t_7365cf05
./tools/visual_evidence_index.sh artifacts/visual_evidence/t_7365cf05
./tools/audit_truth.sh
./tools/audit_readiness.sh . artifacts/readiness/t_5738a1d3
./tools/audit_secrets.sh . artifacts/secrets/t_5738a1d3
```

Full repo gates after renderer code changes:

```sh
./tools/build.sh
./tools/test.sh
cargo build --locked
cargo test --locked
./tools/verify.sh
```

## Phase 4 — materials, lighting, rig, animation, and post-processing foundations

Primary defects addressed:

- ASSET-BLOCKER-002 / A2: material/texture response absent.
- W2: lighting/shadowing missing or non-production.
- Combat readability lacks pose/contact consequence readability.

Cards:

- `t_62c8e2af` — PBR/equivalent material schema and trace-driven damage/wear masks. Owner: `developer`; `mediaqa` review. Type: code/config + material assets. State: dependency-blocked on `t_a95fe445`.
- `t_26ed8543` — lighting/atmosphere. Owner: `developer`; `mediaqa` review. Type: code/config + visual QA. State: dependency-blocked on renderer and material/asset readiness.
- `t_faf2fd4c` — post-processing pipeline. Owner: `developer`; `mediaqa` review. Type: code/config. State: dependency-blocked on lighting and renderer/capture.
- `t_54eabe83` — rig/skin/deformation. Owner: `developer`; `mediaqa` review. Type: code/config + asset integration. State: dependency-blocked on source pack.
- `t_c7afd55b` — animation state machine. Owner: `developer`; `mediaqa` review. Type: code/config + animation. State: dependency-blocked on `t_54eabe83`.

Effort: XL for full material/lighting/rig/animation foundation.

Acceptance criteria:

- Materials visibly distinguish flesh, cloth/quilted linen, mail, plate, lamellar/leather, wood, chalk stone, dirt, blood/wetness, and wear under in-game lighting.
- Damage/wear masks are keyed to replay/trace event ids after hash; no random/time-based renderer-decided material truth.
- Lighting includes directional/key/fill or documented equivalent, contact shadows/AO/equivalent grounding, mood presets for verdict ring/training yard, fog/dust/atmosphere hooks, and timing impact outside truth.
- Rig schema maps canon truth joints to presentation anchors while cosmetic bones are presentation-only.
- Animation state labels cover observe, plan, step, pivot, guard, parry, cut, thrust, brace, bash, hook_bind, grab, shove, kick, recover plus bind/stagger/collapse/injury/recovery reactions.
- Replay hashes remain stable with animation/material/post-processing enabled/disabled.

Exact verification commands:

```sh
./tools/replay_verify.sh artifacts/hifi_renderer/t_f175e621/duel/replay.json
./tools/audit_truth.sh
./tools/accessibility.sh artifacts/accessibility/t_faf2fd4c
./tools/runtime_settings.sh artifacts/settings/t_faf2fd4c
./tools/renderer_target_audit.sh artifacts/renderer_target/t_26ed8543
./tools/audit_readiness.sh . artifacts/readiness/t_26ed8543
```

Material/visual acceptance must include current rendered comparison sheets; texture file presence or recolor alone fails.

## Phase 5 — combat presentation: cameras, VFX, audio, and readable consequence

Primary defects addressed:

- C1: contact and material outcomes are carried by text/line art instead of visible feedback.
- Fight-film/camera language not product-facing.
- Audio/VFX/captions are not integrated as production-facing runtime presentation.

Cards:

- `t_0e005e0f` — combat/replay/fight-film camera language. Owner: `developer`; `mediaqa` frame inspection. Type: code/config + camera QA. State: dependency-blocked on renderer and animation.
- `t_8aebf932` — truth-driven impact VFX/particle system. Owner: `developer`; `mediaqa` frame inspection. Type: code/config + VFX. State: dependency-blocked on materials, animation, renderer.
- `t_3bafc3e1` — audio engine integration. Owner: `developer`; `mediaqa` review. Type: code/config + audio assets. State: dependency-blocked on trace/replay event surfaces and integration epic.

Effort: L/XL, depending on renderer/material/animation readiness.

Acceptance criteria:

- Camera modes include first-person, third-person, planning, consequence, asset-closeup, replay-browser/fight-film, and preserve feet, weapon arcs, contact surfaces, armor gaps, injury/capability evidence, UI readability, and motion comfort settings.
- VFX event manifest maps every emitted effect to replay id/tick/event id/material ids; VFX never decides contact/injury/capability.
- Frames show pre-contact/contact/post-contact consequence, weapon trails/arcs, material response, body reaction, dust/sparks/blood/wetness/debris where justified.
- Audio events derive only from trace/replay events after hash; captions/visual equivalents exist for combat-critical audio cues; mute/gain/settings remain presentation-only.
- Replay hashes stable with VFX/audio/camera/fight-film enabled vs disabled.

Exact verification commands:

```sh
./tools/replay_verify.sh artifacts/hifi_renderer/t_f175e621/duel/replay.json
./tools/audio_vfx_render.sh examples/duels/basic_oathyard.duel artifacts/audio_vfx/t_3bafc3e1
./tools/audio_mixer.sh examples/duels/basic_oathyard.duel artifacts/audio_mixer/t_3bafc3e1
./tools/audio_target_audit.sh artifacts/audio_target/t_3bafc3e1
./tools/accessibility.sh artifacts/accessibility/t_8aebf932
./tools/audit_truth.sh
./tools/audit_readiness.sh . artifacts/readiness/t_3bafc3e1
```

Media QA must inspect actual frames/audio artifacts. Text labels, debug overlays, or trace logs alone cannot close this phase.

## Phase 6 — product UI, input, accessibility, AI seat polish, and performance

Primary defects addressed:

- U1/U2: debug/HUD text density and weak accessibility differentiation.
- Full-game target needs native menu/match flow, settings, input, replay/fight-film entry, performance/debug overlay, and deterministic AI/scripted seats.

Cards:

- `t_be25d936` — native product UI/HUD/menu/match flow. Owner: `developer`; `desktopcontrol` GUI smoke; `mediaqa` UI review. Type: code/config + UI. State: dependency-blocked on renderer/capture and integration surfaces.
- `t_3eb8f26b` — settings/accessibility surfaces. Owner: `developer`; `mediaqa` accessibility review. Type: code/config + accessibility. State: dependency-blocked on `t_be25d936`.
- `t_3cd9ed70` — controller/remap ergonomics. Owner: `developer`; `desktopcontrol` for live GUI/hardware where available. Type: code/config + input QA. State: dependency-blocked on `t_be25d936`.
- `t_609f0e85` — deterministic AI opponent/player-seat polish. Owner: `developer`. Type: code/config. State: todo; product-loop supporting card, not a visual blocker.
- `t_da27c022` — performance profiling/optimization. Owner: `developer`. Type: code/config + performance QA. State: dependency-blocked on renderer and capture matrix.

Effort: XL for UI/settings/input/performance together; M/L for individual input or settings surfaces after UI route exists.

Acceptance criteria:

- Native flow navigates main menu, settings/accessibility, fighter select, loadout select, arena select, observe/plan/commit/resolve/consequence screens, replay browser, fight-film entry, and performance/debug overlay.
- UI shows base frame cost, current frame cost, and physical delta reasons without HP/DPS/crit/super/stat shortcuts.
- Internal audit overlays are separated from player HUD; player-facing captures have readable typography, hierarchy, contrast, safe margins, and minimal debug/provenance text.
- Settings persist through native save/load for quality preset, resolution/window mode when backend supports it, input remap, text scale, contrast, colorblind-safe cues, captions, reduced motion, reduced flash, audio mute/gain/submix choices.
- Controller/input evidence distinguishes local interface smoke from unclaimed physical controller/Steam Deck/owner input acceptance.
- AI emits only legal planned action labels and directional influence through the normal commit path; truth still decides legality/contact/injury/capability/hash.
- Performance report separates simulation step time from render frame time and includes frame-time distribution, startup time, memory, package size delta, and measured before/after for any optimization.

Exact verification commands:

```sh
./tools/input_map.sh artifacts/input/t_3cd9ed70
./tools/gamepad_smoke.sh artifacts/gamepad/t_3cd9ed70 || true
./tools/input_target_audit.sh artifacts/input_target/t_3cd9ed70
./tools/accessibility.sh artifacts/accessibility/t_3eb8f26b
./tools/runtime_settings.sh artifacts/settings/t_3eb8f26b
./tools/audio_target_audit.sh artifacts/audio_target/t_3eb8f26b
./tools/ai_duel.sh artifacts/ai/t_609f0e85 6
./tools/replay_verify.sh artifacts/ai/t_609f0e85/replay.json
./tools/ai_sweep.sh artifacts/ai_sweep/t_609f0e85
./tools/ai_planner_audit.sh artifacts/ai_planner/t_609f0e85
./tools/performance_benchmark.sh artifacts/performance/t_da27c022
./tools/perf_benchmark.sh artifacts/perf/t_da27c022
./tools/audit_truth.sh
./tools/audit_readiness.sh . artifacts/readiness/t_be25d936
```

Use `|| true` only for optional physical gamepad smoke when no device interface is present; the resulting report must state that physical controller/Steam Deck/owner input acceptance is unclaimed.

## Phase 7 — visual benchmark packet and owner handoff

Primary defects addressed:

- No current owner-reviewable HIFI packet exists.
- Owner visual acceptance is absent and cannot be automated.

Cards:

- `t_c77e54b6` — HIFI visual benchmark packet. Owner: `mediaqa`; `gamedesign` wording support. Type: QA + design-doc/update. State: hard dependency-blocked on all prior HIFI children.
- `t_58035fe0` — owner acceptance review handoff. Owner: `mediaqa` for packet presentation; actual owner signs or rejects. Type: external review. State: hard dependency-blocked on `t_c77e54b6`.

Effort: M/L for packet assembly if phases 1-6 are green; XXL/open-ended for owner rejection loops and external gates.

Acceptance criteria:

- Packet is generated from current native executable/assets, not stale copied images.
- Every source replay verifies before capture.
- Capture hashes verify after packet assembly.
- Report evaluates silhouette readability, material richness, anatomy/deformation, weapon/armor detail, arena identity, lighting/atmosphere, animation/contact readability, UI legibility, originality/no-copying, performance evidence, and truth boundary.
- `owner_visual_acceptance=false`, `production_renderer_complete=false`, `public_demo_ready=false`, `release_candidate_ready=false`, `legal_clearance=false`, `trademark_clearance=false`, and `store_readiness=false` remain false until actual external/owner evidence exists.
- Owner response is recorded separately. Automated media QA can prepare evidence and recommendations but cannot sign owner acceptance.

Exact verification commands:

```sh
./tools/replay_verify.sh <each-packet-replay.json>
./tools/visual_benchmark.sh artifacts/visual_benchmark/t_c77e54b6
./tools/final_acceptance.sh artifacts/final_acceptance/t_c77e54b6
./tools/visual_evidence_index.sh artifacts/visual_evidence/t_c77e54b6
./tools/audit_readiness.sh . artifacts/readiness/t_c77e54b6
./tools/audit_secrets.sh . artifacts/secrets/t_c77e54b6
```

## Legacy broad-card policy

`t_34a5af6b` is not an implementation plan. It is the old broad catch-all card and should be treated as follows:

- Do not dispatch it as a broad implementation task.
- Do not use it to duplicate child cards already listed in this plan.
- After concrete HIFI cards complete, either close it as superseded/aggregated with links to this plan and the completed child cards, or keep it as a parent aggregator that verifies no blocker/major items remain.
- If kept as an aggregate, its only acceptance should be: all referenced concrete cards completed, all relevant verification commands passed, media QA packet exists, deferred minor items are logged as named follow-ups, and readiness flags remain honest.

` t_71790e5c ` and `t_05a6f650` are also epics/coordination gates, not monolithic implementation tasks. Their child cards carry the implementation work.

## Mapping from audit findings to concrete cards

| Audit finding | Remediation card(s) | Primary acceptance evidence |
| --- | --- | --- |
| A1 / ASSET-BLOCKER-001 low-poly assets | `t_f72f4c22`, `t_a95fe445` | Source-backed package path fixed; media QA visual inspection; asset gates; nonzero-Z runtime exports; no production acceptance from placeholder lane |
| A2 / ASSET-BLOCKER-002 missing materials | `t_62c8e2af`, `t_26ed8543`, `t_faf2fd4c` | Material manifest; rendered material comparison sheets; replay-stable event-keyed damage/wear; lighting/post effects improve readability without hiding evidence |
| A3 visual differentiation/accessibility | `t_a95fe445`, `t_be25d936`, `t_3eb8f26b` | Fighter/loadout identifiable without labels; non-hue cues; accessibility reports and UI capture matrix |
| ASSET-MAJOR-003 fighters | `t_a95fe445`, `t_54eabe83`, `t_c7afd55b` | Rig/skin manifest; pose/no-clipping sheets; mediaqa deformation review |
| ASSET-MAJOR-004 weapons | `t_a95fe445`, `t_62c8e2af`, `t_8aebf932` | Edge/blunt/pierce/hook geometry visible; grip/reach/contact captures; material/VFX event manifests |
| ASSET-MAJOR-005 armor | `t_a95fe445`, `t_54eabe83`, `t_62c8e2af` | Coverage/gap maps; straps/layers/materials; wearable no-clipping proof |
| ASSET-MAJOR-006 thin source metadata | `t_a95fe445`, `t_6aea2f80` | Structured per-asset specs with provenance/source/runtime hashes and QA blockers |
| ASSET-MAJOR-007 candidate lane not integrated | `t_f72f4c22`, `t_a95fe445`, `t_5738a1d3`, `t_7365cf05` | Correct path contract; renderer/capture integration; visual QA packet |
| WORLD-BLOCKER-001 local/debug world shell | `t_5738a1d3`, `t_7365cf05`, `t_6aea2f80`, `t_26ed8543` | Continuous native loop; current 1920x1080+ environment captures; lighting/atmosphere |
| WORLD-MAJOR-002 arena/world selection absent | `t_6aea2f80`, `t_be25d936`, `t_7365cf05` | Arena select/match setup/capture paths for verdict ring and training yard; replay/content hashes stable |
| WORLD-MAJOR-003 shallow environment structure | `t_6aea2f80`, `t_26ed8543` | Structured environment source with landmarks, staging, material zones, lighting/weather hooks |
| WORLD-MAJOR-004 arena identity not proven | `t_6aea2f80`, `t_7365cf05`, `t_c77e54b6` | Establishing/gameplay/contact/closeup frames for both arenas; mediaqa inspection |
| C1 text-driven combat feedback | `t_c7afd55b`, `t_0e005e0f`, `t_8aebf932`, `t_3bafc3e1` | Pose/contact/VFX/audio/camera frames and manifests keyed to replay/trace events after hash |
| U1 debug HUD density | `t_be25d936`, `t_3eb8f26b`, `t_7365cf05` | Player HUD separated from audit overlays; 1920x1080 capture matrix; OCR/contrast/readability review |
| U2 weak selection/accessibility cues | `t_a95fe445`, `t_be25d936`, `t_3eb8f26b` | Silhouette/icon/badge/non-hue cue proof; accessibility/runtime settings audits |
| Owner visual acceptance absent | `t_c77e54b6`, `t_58035fe0` | Owner-review packet and explicit owner accept/reject artifact; no automated owner signature |

## Final gate bundle after all phases

Run this only after all concrete HIFI implementation/QA cards above have completed or intentionally blocked with named blockers:

```sh
./tools/build.sh
./tools/test.sh
cargo build --locked
cargo test --locked
./tools/verify.sh
./tools/run_duel.sh examples/duels/basic_oathyard.duel --out artifacts/latest
./tools/replay_verify.sh artifacts/latest/replay.json
./tools/export_replay_bundle.sh artifacts/latest/replay.json artifacts/export_bundle/latest
./tools/verify_replay_bundle.sh artifacts/export_bundle/latest
./tools/audit_truth.sh
./tools/audit_secrets.sh
./tools/audit_environment.sh artifacts/environment/final_hifi
./tools/contact_matrix.sh
./tools/build_assets.sh
./tools/validate_assets.sh
./tools/render_asset_previews.sh artifacts/asset_previews/final_hifi
./tools/asset_budget_audit.sh artifacts/asset_budget/final_hifi
./tools/asset_visual_atlas.sh artifacts/asset_atlas/final_hifi
./tools/audit_3d_runtime.sh artifacts/runtime_3d/final_hifi assets/runtime_manifest.json artifacts/native_combat/final_hifi/native_combat_render_manifest.json
./tools/renderer_target_audit.sh artifacts/renderer_target/final_hifi
./tools/input_target_audit.sh artifacts/input_target/final_hifi
./tools/audio_target_audit.sh artifacts/audio_target/final_hifi
./tools/audio_vfx_render.sh examples/duels/basic_oathyard.duel artifacts/audio_vfx/final_hifi
./tools/ai_duel.sh artifacts/ai/final_hifi 6
./tools/truth_stress.sh artifacts/truth_stress/final_hifi
./tools/truth_edge_audit.sh artifacts/truth_edge/final_hifi
./tools/negative_audit.sh artifacts/negative_audit/final_hifi
./tools/run_match_sweep.sh
./tools/performance_benchmark.sh artifacts/performance/final_hifi
./tools/perf_benchmark.sh artifacts/perf/final_hifi
./tools/input_map.sh artifacts/input/final_hifi
./tools/accessibility.sh artifacts/accessibility/final_hifi
./tools/runtime_settings.sh artifacts/settings/final_hifi
./tools/desktop_metadata.sh artifacts/desktop_metadata/final_hifi
./tools/native_combat_render.sh examples/duels/basic_oathyard.duel artifacts/native_combat/final_hifi
./tools/capture_high_fidelity_screens.sh artifacts/high_fidelity_screens/final_hifi
./tools/visual_benchmark.sh artifacts/visual_benchmark/final_hifi
./tools/final_acceptance.sh artifacts/final_acceptance/final_hifi
./tools/audit_readiness.sh . artifacts/readiness/final_hifi_source
./tools/package.sh
./tools/smoke_package.sh artifacts/package/oathyard-linux-x86_64.tar
```

Passing the local final bundle still does not imply public demo, release candidate, owner-final, legal, trademark, store, or owner visual acceptance. Those remain separate external gates.
