# OATHYARD Design Knowledge Base

Status: design reference synthesis from canonical sources. Not implementation evidence, not readiness evidence, not owner acceptance.

KB date: 2026-06-30
Task: t_5796ea5e / Gap 087

This KB consolidates OATHYARD's design identity, canon, mechanics, systems, content, art direction, architecture, roadmap, acceptance gates, and source map into one navigable reference. It exists so agents and contributors can orient without reading 50+ scattered docs first. Every claim here is sourced; nothing is invented. When this KB conflicts with a cited source, the source wins.

For authoritative rules, always defer to the source files listed in Section 12 (Source Map), in precedence order.

---

## 1. Identity

OATHYARD is a deterministic native-PC 3D planned-time physical melee duel game.

- Genre: planned-time physical melee duel.
- Platform: native PC (Linux-first, cross-platform target).
- Stack: Rust/Cargo, no external runtime dependencies for the truth core.
- Art target: premium high-fidelity dark-fantasy judicial-duel, benchmarked against Elden Ring and For Honor for quality/readability only. Original — no copying of names, assets, silhouettes, factions, UI, animations, lore, characters, textures, music, or proprietary mechanics.
- Core innovation: deterministic replayable physical melee truth. No HP, no stats, no RNG. Body/material/capability state IS the health model.

Source: `docs/design/GAME_CANON.md:7-11`, `docs/decisions/0007-high-fidelity-production-target.md:11-15`

---

## 2. Canon Precedence

Read and preserve this order before changing behavior:

1. `docs/design/GAME_CANON.md`
2. `docs/design/DEMO_SCOPE.md`
3. `ACCEPTANCE_MAP.md`
4. `AGENTS.md` / `CLAUDE.md`
5. PRDs/specs
6. Code comments

Source: `AGENTS.md:3-11`

---

## 3. Core Loop

`OBSERVE -> PLAN -> COMMIT-REVEAL -> RESOLVE -> CONSEQUENCE -> RE-PLAN`

Players author compact physical action labels plus directional influence, not raw joint puppeteering. This is "planned-time" melee: both sides commit, then truth resolves simultaneously.

Canonical action labels:
- step, pivot, guard, parry, cut, thrust, brace, bash, hook_bind, grab, shove, kick, recover

Source: `docs/design/GAME_CANON.md:17-39`

---

## 4. Deterministic Truth

This is the project's central technical value.

- Fixed 120 Hz simulation tick.
- Integer/fixed-point truth only. No gameplay floats.
- No hidden RNG. No wall-clock time in truth. No unordered iteration affecting state.
- Render, UI, and audio never write into gameplay truth.
- Replay is authoritative evidence and fails loudly on mismatch.
- Truth runs first; presentation systems consume truth after hashing.

### Contact truth path

`Timeline input -> sequence resolver -> physics/contact packet -> armor/material solve -> anatomy solve -> injury events -> capability deltas -> updated action validity/frame costs -> replay hash`

### Dynamic frame cost model

`Current Frame Cost = Base x Body x Equipment x State x Momentum x Injury`

UI and logs show base cost, current cost, and the physical reason for each delta.

Sources: `docs/design/GAME_CANON.md:41-50,126-148`, `AGENTS.md:70-90`

---

## 5. Health Model (No HP)

There is no HP. No hit points, arbitrary damage numbers, armor points, DPS, crit chance, super meter, perks, unlock stats, or speed/damage bonuses.

The body/material/capability state IS the health model:
- Injuries are physical consequences of contact, material solve, and anatomy solve.
- Capability deltas affect what actions remain legal and their frame costs.
- A fighter is defeated through capability-stop conditions (collapse, stagger, grip loss, etc.), not a depleting number.

Source: `docs/design/GAME_CANON.md:134-148`, `AGENTS.md:92-105`

---

## 6. Body Graph

16 truth joints plus grip frames:

| Id | Joint | Id | Joint |
|----|-------|----|-------|
| 0 | root | 8 | elbow_l |
| 1 | spine_lower | 9 | wrist_l |
| 2 | spine_upper | 10 | hip_r |
| 3 | neck_head | 11 | knee_r |
| 4 | shoulder_r | 12 | ankle_r |
| 5 | elbow_r | 13 | hip_l |
| 6 | wrist_r | 14 | knee_l |
| 7 | shoulder_l | 15 | ankle_l |

Also includes `grip_r` and `grip_l` frames.

Cosmetic bones (cloak, scabbard, straps) are presentation-only and separate from truth joints.

Source: `docs/design/GAME_CANON.md:101-124`, `docs/design/ART_DIRECTION_BRIEF.md:47`

---

## 7. Weapons and Armor

Weapons are physical. Defined by: length, mass distribution, moment of inertia, edge/blunt/pierce/hook profile, grip points, reach, alignment, follow-through.

Armor is physical. Defined by: pieces, mass, inertia, coverage, gaps, deformation, deflection, blunt transfer, binding, damage, detachment.

Minimum content breadth: 6 fighter traditions, 8 weapon families, 6 armor/loadout families, OATHYARD verdict ring, training arena.

Current asset baseline: 22 runtime assets (6 fighters, 8 weapons, 6 armor/loadout families, 2 arenas). This satisfies numeric floors but remains far below production visual fidelity.

Sources: `docs/design/GAME_CANON.md:149-156`, `ACCEPTANCE_MAP.md:40`, `docs/roadmap/M0_M21_CANON_ACCEPTANCE_DECOMPOSITION.md:44`

---

## 8. Fighter Traditions (Six)

From `assets_src/fighters/traditions.oysrc`:
1. Saltreach Duelist — lean-forward
2. Oathyard Writ — balanced
3. Chainbreaker — wide-hooking
4. Reed Sentinel — long-reed-guard
5. Gate Shield — shielded-low
6. Bruiser Oath — heavy-maul

Source: `docs/design/ART_DIRECTION_BRIEF.md:28`

---

## 9. Truth vs Presentation Layering

Every system is classified as exactly one layer before adoption:

### Layer 1: offline_research_authoring
May generate candidate motions, reference traces, solver comparisons, AI training data, material parameters, concept art, source drafts, benchmark captures, falsification fixtures. May not decide live contacts, injuries, costs, capabilities, end states, or hashes.

Examples: Warp, Isaac Lab, Newton, MJWarp, PhysX, Chrono, generative 3D proposals.

### Layer 2: runtime_presentation
May consume hashed truth/replay state to produce renderer poses, PresentationBricks motion, retargeted skeletons, cloth/armor secondary motion, facial animation, audio/VFX, camera/fight-film, UI/captions/accessibility, material wear masks, runtime glTF display. Must carry `truth_mutation:false`. May not mutate any truth field.

Examples: PresentationBricks (internal MotionBricks-inspired layer), Nanite/Lumen-class renderer, glTF runtime assets, Audio2Face-3D.

### Layer 3: runtime_authoritative_truth
Forbidden by default. A frontier system may enter this layer only after a separate ADR proves:
1. Frozen (content-addressed weights/checkpoints/source/seeds/params/versions)
2. Deterministic (bit-identical reproduction)
3. Hashed (content-addressable, verified at load/replay)
4. Replayable (full reproduction from recorded state/inputs/hashes)
5. Cross-platform verified (zero tolerance on replay-relevant bytes)

No current frontier system has passed these gates.

Sources: `docs/design/GAME_CANON.md:52-99`, `docs/decisions/0003-truth-vs-presentation-layering.md:11-80`

---

## 10. Physical Fidelity Architecture

Current truth is deterministic scalar scaffolding. The missing piece is an OATHYARD-owned reduced biomechanics/tissue/material truth model inside the deterministic envelope.

The bridge from high-fidelity reference simulation to game truth:

```
offline reference authoring
  -> frozen reference fixture (solver/version/scene/seed/hash)
  -> reduction script (explicit integer rounding)
  -> OATHYARD reduced tables/curves/fixtures (content hashes)
  -> 120 Hz deterministic truth solve
  -> replay/trace/content/table/state hashes
  -> read-only presentation deformation packets
```

Only OATHYARD reduced tables and the OATHYARD deterministic solver may become authoritative combat truth. External solver states, engine physics, renderer deformation, neural inference, or GPU results must not decide live contact/injury/capability/hash.

Source: `docs/design/PHYSICAL_FIDELITY_ARCHITECTURE.md:1-81`

---

## 11. Art Direction

Target look: grounded stylized realism. High-detail, tactile, grim judicial-fantasy models with intentionally readable combat shapes. Not cute low-poly, not flat cel-shading, not generic photoreal medieval kit.

Style anchors:
1. Silhouette language: readable duel intent before ornament.
2. Palette: cold oath stone (chalk stone, iron black, tarnished steel, ash wood, buff leather) plus faction accents (inked salt sash, black oath tabard, split-chain badge, ash pole wrap, chalked shield, iron oath mask).
3. Surface treatment: tactile physical materials with PBR (roughness/metalness/normal/AO), edge wear, dents, seams, blood/dirt/wetness.
4. Proportions: grounded humans, combat-expressive gear. Stylization may exaggerate read by 5-15% only.
5. Arena: judgment ritual framing — OATHYARD verdict ring (chalked stone, north judgment balcony, cold oath-mark lighting) and training arena (packed clay, measured practice lines).

Source: `docs/design/ART_DIRECTION_BRIEF.md:18-57`

---

## 12. Production Stack and Environment

- Language: Rust (rustc 1.96.0, cargo 1.96.0)
- Truth core: no external runtime dependencies
- Presentation target: Bevy/wgpu selected for V1 production-renderer spike (ADR 0009)
- GPU: NVIDIA GeForce RTX 5090 (32607 MiB, driver 595.80), Vulkan 1.4.341
- Native surface metadata present: x11, wayland-client, egl, gl, xrandr, xi, xcursor, xkbcommon
- Missing pkg-config: sdl2, glfw3, vulkan
- Known blocker: Blender fails startup with MaterialX ABI/symbol error (`undefined symbol: _ZTVN17MaterialX_v1_39_46OutputE`), blocking DCC/source-asset pipeline
- Missing tools: gltf-validator, gltfpack, toktx, ktx, basisu, sox, ALSA/PulseAudio dev metadata, OpenAL metadata

Source: `docs/decisions/0001-stack-and-determinism.md`, `docs/decisions/0009-production-renderer-selection.md:26-43`, `docs/acceptance/FULL_GAME_ACCEPTANCE.md:9-16`

---

## 13. Roadmap (M0-M21)

Dependency spine:
`M0-M1 canon/control -> M2-M6 deterministic foundation -> M7-M8 source-backed assets/content -> M9-M14 native playable/evidence surfaces -> M15-M17 local ship-quality gates -> M18-M21 production/external release gates`

| Milestone | Acceptance surface | Current state |
|-----------|-------------------|---------------|
| M0 | Canon and forbidden-shortcut lock | Hard locks exist |
| M1 | Acceptance map, roadmap, pull discipline | Decomposition explicit |
| M2 | Native Rust build/test harness | Local gates exist; no baseline commit/remote |
| M3 | Fixed-step deterministic truth kernel | Evidence exists |
| M4 | Body/material/capability health model | Contact matrix exists |
| M5 | Replay, fight-film, export bundle | Gates exist |
| M6 | Loud-failure audits (stress/edge/negative) | Gates exist |
| M7 | Source-backed asset pipeline | Local text-spec/glTF pipeline; DCC/Khronos blocked |
| M8 | Content breadth and 3D budgets | 22 runtime assets; below production fidelity |
| M9-M10 | Native game flow, menus, loadout selection | Replay-backed smoke exists |
| M11 | Native input boundary | Local schema/command-flow; physical controller false |
| M12 | Accessibility, runtime settings | Presentation-only artifacts exist |
| M13 | Deterministic AI/scripted seats | AI duel/sweep exists |
| M14 | Fight-film cameras, presentation states | Shot-manifest JSON exists |
| M15 | Native presentation renderer | Raw X11/PPM evidence; production renderer false |
| M16 | Audio/VFX, mixer, device smoke | Trace-derived evidence; shipping backend false |
| M17 | Performance, package, package smoke | Local package candidate exists |
| M18 | High-fidelity production renderer | NOT PASSED |
| M19 | Production assets, visual benchmark | NOT PASSED |
| M20 | Owner/human acceptance | NOT PERFORMED |
| M21 | License/legal/trademark/store gates | BLOCKED/EXTERNAL |

Sources: `docs/roadmap/M0_M21_CANON_ACCEPTANCE_DECOMPOSITION.md`, `docs/roadmap/FULL_GAME_ROADMAP.md`

---

## 14. Acceptance Gates

### Local Publishable Package Gate
`./tools/verify.sh` is the local gate. It must pass and include: build, tests, truth audit, readiness drift audit, secrets audit, environment audit, asset build/validation/previews, runtime glTF with nonzero Z, PBR material evidence, asset budgets, runtime 3D audit, determinism (two identical runs), replay verification, AI/scripted seats, truth stress, truth edge audit, negative input audit, match sweep, input map, native input target audit, accessibility, runtime settings, native roster 3D showcase, native combat render, visual evidence reducer, native presentation target audit, audio/VFX, audio runtime target audit, desktop metadata, package, package smoke, package reproducibility, final evidence report.

### High-Fidelity Production Gate
Currently NOT PASSED. Requires: continuous player-facing native 3D renderer, source-backed production assets, PBR materials, skeletal/skinned fighters, layered armor/cloth/weapon detail, arena with lighting/atmosphere, animation driven by hashed truth, 1920x1080+ deterministic capture coverage, visual benchmark report, owner visual acceptance.

### Public/Store Release Gate
All false until separately performed:
- Owner-final acceptance, public demo readiness, release-candidate readiness, legal clearance, trademark clearance, store readiness, Steam release, itch.io release.

Sources: `ACCEPTANCE_MAP.md:32-103`, `docs/acceptance/FULL_GAME_ACCEPTANCE.md:18-109`

---

## 15. Current Known Blockers

- LICENSE pending/unlicensed; distribution rights unresolved.
- No baseline commit, remote, or issue tracker.
- Blender fails startup (MaterialX ABI error) — blocks DCC/source-asset pipeline.
- No gltf-validator, gltfpack, toktx, ktx, basisu, sox.
- SDL2/GLFW pkg-config unavailable.
- Vulkan pkg-config unavailable (runtime present, no renderer implemented).
- No clean VM/container evidence.
- Production renderer not implemented (Bevy/wgpu selected as V1 spike direction only).
- Production assets below target fidelity (low-poly text-generated, not sculpt/retopo/PBR/skinned).
- Owner visual/audio/input acceptance not performed.
- Legal/trademark/store readiness external-blocked.

Sources: `docs/acceptance/FULL_GAME_ACCEPTANCE.md:9-16`, `ACCEPTANCE_MAP.md:105-121`, `docs/decisions/0009-production-renderer-selection.md:43`

---

## 16. Key Verification Commands

```
./tools/build.sh
./tools/test.sh
cargo build --locked
cargo test --locked
./tools/verify.sh
./tools/run_duel.sh examples/duels/basic_oathyard.duel --out artifacts/latest
./tools/replay_verify.sh artifacts/latest/replay.json
./tools/audit_truth.sh
./tools/audit_secrets.sh . artifacts/secrets/source
./tools/audit_readiness.sh . artifacts/readiness/source
./tools/build_assets.sh
./tools/validate_assets.sh
./tools/renderer_target_audit.sh artifacts/renderer_target/verify
./tools/input_target_audit.sh artifacts/input_target/verify
./tools/audio_target_audit.sh artifacts/audio_target/verify
./tools/package.sh
./tools/smoke_package.sh artifacts/package/oathyard-linux-x86_64.tar
```

Source: `AGENTS.md:16-68`

---

## 17. Source Map

All docs are under the OATHYARD repo root. Paths are relative to repo root.

### Design (canon tier)
- `docs/design/GAME_CANON.md` — full-game canon, identity, truth, body graph, cost model, contact path, weapons, armor, replay, readiness
- `docs/design/DEMO_SCOPE.md` — current verified scope, full-game target, out-of-scope, presentation boundary
- `docs/design/ART_DIRECTION_BRIEF.md` — art target, style anchors, fighter traditions, model revision targets
- `docs/design/PHYSICAL_FIDELITY_ARCHITECTURE.md` — reduced biomechanics/tissue/material truth model specification
- `docs/design/VERDICT_RING_TRAINING_YARD_IDENTITY_AUDIT.md` — arena identity audit

### Acceptance
- `ACCEPTANCE_MAP.md` — acceptance bridge, active goal, non-negotiable invariants, gate tables, blockers, status language
- `docs/acceptance/FULL_GAME_ACCEPTANCE.md` — current status, required gates, completion criteria, external gates
- `docs/acceptance/VISUAL_FIDELITY_ACCEPTANCE_CRITERIA.md` — testable visual fidelity gates, reference set, pass/fail checklists

### Decisions (ADRs)
- `docs/decisions/0001-stack-and-determinism.md` — Rust/Cargo, no external deps, determinism risks
- `docs/decisions/0002-high-fidelity-production-target.md` — high-fidelity target
- `docs/decisions/0002-native-presentation-target.md` — native presentation boundary
- `docs/decisions/0003-truth-vs-presentation-layering.md` — layer contract, PresentationBricks
- `docs/decisions/0003-native-input-model.md` — production input command boundary
- `docs/decisions/0003-rodin-to-production-asset-pipeline.md` — Rodin asset pipeline
- `docs/decisions/0004-renderer-and-asset-pipeline.md` — renderer/asset pipeline boundary
- `docs/decisions/0004-renderer-or-engine-selection.md` — renderer/engine evaluation
- `docs/decisions/0004-audio-runtime-target.md` — audio runtime boundary
- `docs/decisions/0005-cross-platform-verification.md` — cross-platform verification
- `docs/decisions/0007-high-fidelity-production-target.md` — production visual target, baseline evidence, acceptance gates
- `docs/decisions/0008-hifi-wo-01-renderer-backend-adr.md` — superseded renderer backend spike
- `docs/decisions/0009-production-renderer-selection.md` — Bevy/wgpu V1 selection

### Roadmap
- `docs/roadmap/FULL_GAME_ROADMAP.md` — fail-closed roadmap, M0-M6 completed, M7-M17 targets
- `docs/roadmap/PUBLISHABLE_KANBAN.md` — working board, columns, WIP limits, done definitions
- `docs/roadmap/M0_M21_CANON_ACCEPTANCE_DECOMPOSITION.md` — milestone gap/dependency table, pull list
- `docs/roadmap/HIGH_FIDELITY_PRODUCTION_WORK_ORDERS.md` — work-order/spec package
- `docs/roadmap/HIFI_REMEDIATION_PHASE_PLAN.md` — remediation phases
- `docs/roadmap/HIFI_WO_01_RENDERER_BACKEND_IMPACT_AUDIT.md` — renderer backend impact audit

### Research
- `docs/research/FRONTIER_TECH_LEVERAGE.md` — frontier tech register (MotionBricks, Warp, Isaac Lab, Newton, etc.)

### Asset Pipeline
- `docs/asset_pipeline/ASSET_PIPELINE.md` — policy, directories, build/validate commands
- `docs/asset_pipeline/FRONTIER_AUTONOMOUS_ASSET_PRODUCTION_PLAN.md` — autonomous asset plan
- `docs/asset_pipeline/WEAPON_DIVERSITY_CONCEPT_EXTRACTION.md` — weapon diversity concepts
- `docs/asset_pipeline/ASSET_AUTOMATION_RESEARCH_20260630.md` — asset automation research

### Art
- `docs/art/VISUAL_TARGET_HIGH_FIDELITY.md` — visual target spec
- `docs/art/PRODUCTION_ASSET_ACCEPTANCE.md` — production asset acceptance
- `docs/art/RODIN_ASSET_AUDIT.md` — Rodin asset audit
- `docs/art/ASSET_PROVENANCE.md` — asset provenance policy

### Operations
- `docs/operations/ARTIFACT_AND_KANBAN_HYGIENE.md` — artifact/kanban hygiene

### Agent Rules
- `AGENTS.md` — canon precedence, verification commands, determinism rules, forbidden shortcuts, final report requirements

---

## 18. Glossary

- Truth: the authoritative deterministic simulation state at 120 Hz. Integer/fixed-point only.
- Presentation: everything that consumes truth after hashing — renderer, UI, audio, VFX, camera, fight-film.
- PresentationBricks: OATHYARD's internal MotionBricks-inspired presentation-motion layer. Presentation-only.
- Replay: authoritative evidence storing initial state, committed sequences, content/table versions, hashes. Fails loud on mismatch.
- Fight-film: trace-derived camera/cinematic output. Presentation-only.
- Contact packet: deterministic physics/contact data produced by the sequence resolver.
- Capability delta: change to a fighter's legal action set or frame costs resulting from injury/contact.
- Capability-stop: end condition caused by capability loss (collapse, stagger, grip loss), not HP depletion.
- Local publishable package: source tree can build/smoke a native package locally. Lower gate.
- Public/store publishable release: local package plus owner/legal/trademark/store gates. Higher gate.
- Readiness flags: machine-readable booleans that must stay false until their gate is evidenced: production_renderer_complete, owner_visual_acceptance, public_demo_ready, release_candidate_ready, legal_clearance, trademark_clearance, store_readiness.
