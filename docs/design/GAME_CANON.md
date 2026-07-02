# OATHYARD Full-Game Canon

This document preserves the deterministic bootstrap canon while adding the full-game high-fidelity native-PC 3D production target. Older first-slice, bootstrap, headless, or non-native visual-output wording is historical baseline only and cannot satisfy the full-game target by itself.

## Identity

OATHYARD is a deterministic native-PC 3D planned-time physical melee duel game.

## Production Visual Target

The product target is a premium current-generation native-PC 3D game with high-fidelity dark-fantasy judicial-duel art direction and melee readability ambition benchmarked against Elden Ring and For Honor. Those games are quality/readability references only; OATHYARD must remain original and must not copy names, assets, silhouettes, factions, UI, animations, lore, characters, textures, music, or proprietary mechanics.

Current nonvisual manifests/reports and low-poly glTF checks are local verification evidence only. They prove deterministic data flow and nonzero-Z source-backed geometry; they do not prove high-fidelity product presentation, production renderer completion, owner visual acceptance, public-demo readiness, or release-candidate readiness.

Production visual completion requires source-backed high-detail fighters, armor, weapons, arenas, material maps, lighting, atmosphere, animation presentation, combat feedback, UI, replay/fight-film cameras, deterministic captures, and owner visual acceptance as defined by `docs/decisions/0007-high-fidelity-production-target.md`.

## Core Loop

The core loop is:

`OBSERVE -> PLAN -> COMMIT-REVEAL -> RESOLVE -> CONSEQUENCE -> RE-PLAN`

Players author compact physical action labels plus directional influence, not raw joint puppeteering.

Current canonical action labels include:

- `step`
- `pivot`
- `guard`
- `parry`
- `cut`
- `thrust`
- `brace`
- `bash`
- `hook_bind`
- `grab`
- `shove`
- `kick`
- `recover`

## Truth

- Simulation truth runs at fixed 120 Hz.
- Truth must be deterministic.
- No hidden RNG.
- No wall-clock time in gameplay truth.
- No floating-point gameplay truth.
- No unordered iteration affecting state.
- Render, UI, and audio must not write into gameplay truth.
- Use fixed-point or integer math for truth.

## Frontier Research Leverage and Authoritative Combat Truth

OATHYARD should actively use frontier AI, robotics, simulation, motion, graphics, audio, and asset research to generate, author, train, validate, and present the game. That leverage is subordinate to deterministic combat truth. Nondeterministic neural, GPU, remote-service, or external-solver systems must never become authoritative combat truth unless they are frozen, deterministic, hashed, replayable, and cross-platform verified under this canon.

**Authoritative combat truth** means any source data, runtime state, data path, or decision point that can change live or replayed combat legality, causality, timing, physical outcome, hash, or winner. If changing a value can change replay JSON, trace JSON, contact packets, action validity, frame costs, material/anatomy results, injuries, capability deltas, end condition, final hash, or the content/table hash consumed by replay, that value is combat truth.

Authoritative combat truth includes:

- The fixed 120 Hz simulation tick, phase/turn progression, and `OBSERVE -> PLAN -> COMMIT-REVEAL -> RESOLVE -> CONSEQUENCE -> RE-PLAN` state machine.
- Initial combat state, scenario/loadout manifests, content/table versions, content hashes, and any table used by truth.
- Committed player or AI action labels, directional influence, sequence timing, and validated committed inputs after they cross the commit boundary.
- Action legality, recovery windows, frame-cost calculation, momentum/injury/equipment/capability cost deltas, and any decision that changes the next legal action set.
- Body graph, grip frames, weapon reach/mass/inertia/edge/blunt/pierce/hook/contact profiles, armor coverage/gaps/deformation/deflection/blunt-transfer/binding/detachment, anatomy/material/capability tables, and any production asset metadata used by those calculations.
- Contact ordering, physics/contact packets, armor/material solve, anatomy solve, injury events, capability deltas, grip loss, stagger/collapse/capability-stop outcomes, end state, winner, replay hash, and loud replay mismatch behavior.
- Any AI-derived or externally-derived artifact whose output is read by one of the above paths.

Authoritative combat truth excludes, unless separately promoted by the gates below:

- Renderer, UI, audio, VFX, camera, fight-film, accessibility, captions, settings, device input surfaces before committed action serialization, and all other presentation systems.
- PresentationBricks, MotionBricks-style motion, neural animation, retargeting, interpolation, cloth/secondary motion, facial animation, material wear masks, cinematic cameras, and renderer/engine physics used only after truth hashes exist.
- Offline research solvers, differentiable/GPU simulations, robotics/VLA models, generative 3D/image/video systems, neural rendering, world models, and AI planners while they are used as references, authoring accelerators, training sources, or candidate generators.
- Runtime render meshes, textures, skeleton cosmetics, blendshapes, sounds, particles, and visual effects that do not feed collision/contact/material/anatomy/capability truth.

Every frontier system must be classified before adoption as exactly one of:

1. `offline_research_authoring`: may generate candidate motions, policies, assets, reference traces, solver comparisons, material parameters, benchmark captures, or falsification fixtures. It may not decide live contact, injury, action cost, capability, end state, or hashes.
2. `runtime_presentation`: may consume hashed truth/replay state to produce animation, retargeted render poses, cloth/armor secondary motion, facial animation, audio/VFX, camera, interpolation, UI, captions, accessibility presentation, or fight-film output. It must carry `truth_mutation:false` or equivalent evidence.
3. `runtime_authoritative_truth`: forbidden by default. It may be used only after a separate ADR and current evidence prove every promotion gate below.

AI and planner systems may train on frontier data or use frontier models offline, but runtime AI seats are not outcome authorities. They may emit only legal planned action labels and directional influence for the normal commit path; truth still decides legality, contact, material/anatomy results, injuries, capabilities, costs, hashes, end state, and winner.

An AI-derived asset or logic artifact may be promoted to authoritative combat truth only when all five gates pass:

1. **Frozen** - The exact weights, checkpoints, source data, prompts/control inputs, seeds, parameters, tool versions, generated assets, derived tables, code, and build/export commands are snapshot-committed or otherwise content-addressed in the repository. No mutable remote service, floating model alias, hidden cache, unrecorded seed, or unversioned GPU/driver-dependent output may be part of truth.
2. **Deterministic** - The same initial state, committed inputs, content versions, platform target, and command produce bit-identical truth outputs. Truth promotion requires fixed-point/integer semantics, ordered iteration, no hidden RNG, no wall-clock input, no nondeterministic threading/GPU reductions, and no presentation writeback. If a neural/GPU system cannot prove bit-identical inference, only its frozen exported output may be considered for truth.
3. **Hashed** - Every promoted source, runtime artifact, generated table, model output, and relevant tool/config manifest is content-addressable. Hashes are published in manifests and replay records, verified at load/replay time, and mismatches fail loudly before use.
4. **Replayable** - The full result is reproducible from recorded initial state, committed inputs, content/table versions, hashes, commands, and platform target with no hidden network, wall-clock, cache, random, or process state. Replay must re-run deterministic truth and fail loudly on any byte mismatch.
5. **Cross-platform verified** - The artifact passes the same replay/hash tests on every supported OATHYARD target platform and required build configuration. Authoritative combat outputs require zero tolerance: replay-relevant bytes and hashes must match exactly. If a platform needs tolerances, the artifact is not combat truth until its outputs are quantized or otherwise reduced to bit-identical canonical truth.

Reviewer acceptance criteria for any proposed AI-derived combat asset or logic:

- It names its layer: `offline_research_authoring`, `runtime_presentation`, or `runtime_authoritative_truth`.
- It identifies every data path it touches and states whether any output can affect action legality, contact, cost, material/anatomy solve, injury, capability, end state, replay data, or hashes.
- If it is not truth, a presentation/isolation check proves replay JSON, trace JSON, contact packets, costs, capability deltas, end condition, and final hash stay byte-identical with the system enabled/disabled.
- If it is truth, the review includes a freeze manifest, deterministic reproduction command, hash manifest, replay artifact, cross-platform matrix, mismatch/fail-loud evidence, and ADR approving truth promotion.
- Missing evidence means the artifact remains non-authoritative, regardless of visual quality, model quality, local convenience, or benchmark performance.
- No generated, neural, or external asset may enter production combat truth through renderer metadata, animation events, collision side channels, hidden AI state, or presentation feedback.
- Passing this truth gate does not imply high-fidelity visual completion, owner acceptance, public-demo readiness, release-candidate readiness, legal clearance, or store readiness.

## Canonical Body Graph

Truth joints:

| Id | Joint |
| --- | --- |
| 0 | root |
| 1 | spine_lower |
| 2 | spine_upper |
| 3 | neck_head |
| 4 | shoulder_r |
| 5 | elbow_r |
| 6 | wrist_r |
| 7 | shoulder_l |
| 8 | elbow_l |
| 9 | wrist_l |
| 10 | hip_r |
| 11 | knee_r |
| 12 | ankle_r |
| 13 | hip_l |
| 14 | knee_l |
| 15 | ankle_l |

Also include `grip_r` and `grip_l` frames.

## Cost Model

Dynamic frame cost:

`Current Frame Cost = Base x Body x Equipment x State x Momentum x Injury`

UI and logs must show base cost, current cost, and the physical reason for each delta.

## Contact Truth Path

`Timeline input -> sequence resolver -> physics/contact packet -> armor/material solve -> anatomy solve -> injury events -> capability deltas -> updated action validity/frame costs -> replay hash`

There is no HP. The body/material/capability state is the health model.

Forbidden arbitrary stats include:

- damage bonuses
- speed bonuses
- armor points
- DPS
- crit chance
- super meter

## Weapons

Weapons are physical. They are defined by length, mass distribution, moment of inertia, edge/blunt/pierce/hook profile, grip points, reach, alignment, and follow-through.

## Armor

Armor is physical. It is defined by pieces, mass, inertia, coverage, gaps, deformation, deflection, blunt transfer, binding, damage, and detachment.

## Replay

Replay is authoritative evidence. It stores initial state, committed sequences, content/table versions, and hashes. Replay re-runs deterministic truth and fails loud on mismatch.

## Fight Film

Fight-film output is trace-derived only. The first slice may output a deterministic shot-manifest JSON instead of encoded video.

## Product Readiness

Native product target remains the goal. Browser/HTML may only be a QA harness and must not be claimed as product presentation.

Public-demo-ready and release-candidate-ready remain false unless explicit owner/human gates are actually satisfied.
