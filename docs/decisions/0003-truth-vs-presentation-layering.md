# 0003: Truth vs Presentation Layering

Status: Accepted boundary contract; not implementation evidence.
Date: 2026-06-29
Related: `docs/research/FRONTIER_TECH_LEVERAGE.md`, `docs/decisions/0002-high-fidelity-production-target.md`, `docs/decisions/0007-high-fidelity-production-target.md`

## Context

OATHYARD's main technical value is replayable, physically causal planned-time melee truth. Frontier motion, graphics, simulation, audio, facial, and asset systems can improve presentation and research velocity, but they must not erase the deterministic core by becoming unexplained truth oracles.

## Layer contract

Every frontier or production-facing system must be assigned to exactly one layer before adoption.

### 1. `offline_research_authoring`

May generate:

- candidate motions;
- reference physics traces;
- solver comparison data;
- AI/planner training data;
- material parameters;
- concept art or blockouts;
- source asset drafts;
- benchmark captures;
- falsification fixtures.

May not directly decide live-game contacts, injuries, action costs, capability deltas, end states, or replay hashes.

Examples: Warp, Isaac Lab, Newton, MJWarp, PhysX, Chrono, OpenUSD source pipeline, generative 3D proposals.

### 2. `runtime_presentation`

May consume hashed truth/replay state to generate:

- renderer poses and interpolation;
- `PresentationBricks` motion;
- retargeted render skeletons;
- cloth/armor secondary motion;
- facial animation;
- audio/VFX;
- camera/fight-film output;
- UI/captions/accessibility presentation;
- material damage/wear masks;
- runtime glTF/GLB asset display.

May not mutate action validity, contact, damage, injury, recovery cost, capability, end-state, content hash, replay hash, or authoritative trace fields.

Examples: MotionBricks-inspired `PresentationBricks`, Nanite/Lumen-class renderer path, glTF runtime assets, Audio2Face-3D facial presentation.

### 3. `runtime_authoritative_truth`

Forbidden by default.

A system can enter this layer only if a separate ADR proves:

- deterministic fixed-version implementation;
- no hidden RNG;
- no wall-clock truth input;
- no gameplay floats unless replaced by deterministic fixed-point semantics;
- ordered truth iteration;
- replay serialization and hash coverage;
- byte-identical or explicitly tolerance-bounded cross-platform verification;
- loud failure on mismatch;
- complete negative tests for corrupted inputs and schema drift;
- no presentation-side writes back into truth.

No current frontier system in `docs/research/FRONTIER_TECH_LEVERAGE.md` is accepted into this layer.

## PresentationBricks contract

`PresentationBricks` is OATHYARD's internal MotionBricks-inspired presentation-motion layer. It may be implemented with authored clips, procedural logic, neural models, offline generated clips, or a future verified MotionBricks integration, but its authority is the same: presentation only.

It must consume:

- truth poses;
- action labels;
- contact events;
- material/anatomy solve events;
- capability changes;
- replay traces;
- camera/capture state;
- content and asset hashes.

It may output:

- locomotion presentation;
- guard/weapon handling;
- bind/hook reactions;
- stumbles, falls, collapse, recovery;
- object interaction;
- retargeted render-skeleton poses;
- fight-film camera timing hints.

It must never output authoritative:

- hit/contact decisions;
- damage/injury/capability results;
- action legality;
- action/cost/frame timing;
- end state/winner;
- replay hash/content hash;
- hidden stat boosts.

## Required invariants

- Truth is computed first; presentation consumes after hash.
- Every presentation artifact carries `truth_mutation:false` or equivalent.
- Every generated/AI/neural output has provenance and layer classification.
- Toggling presentation systems cannot change truth output.
- If a presentation system needs a value that truth did not expose, add a truth-read-only export schema; do not let presentation infer and feed state back.
- If a reference solver disagrees with OATHYARD truth, record the disagreement and design a falsifying deterministic test; do not silently copy the reference result.

## Verification

The boundary is executable through:

```sh
./tools/presentation_truth_isolation.sh
./tools/sim_reference_compare.sh
./tools/ai_planner_audit.sh
./tools/audit_truth.sh
./tools/audit_readiness.sh . artifacts/readiness/source
```

Passing these commands proves only the checked boundary properties. It does not prove high-fidelity renderer completion, production asset quality, public-demo readiness, release-candidate readiness, or owner visual acceptance.
