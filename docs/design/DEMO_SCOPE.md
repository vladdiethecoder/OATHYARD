# OATHYARD Scope Boundaries

## Current Verified Scope

The existing verified slice proves a source-built deterministic local duel foundation:

- Build a native Rust executable from source.
- Run one scripted two-fighter local duel.
- Emit deterministic frame-cost, contact, injury, capability, replay, report, and fight-film metadata artifacts.
- Replay the duel bit-exactly and fail loudly on mismatch.
- Verify loadout variation and injury/capability effects with automated tests.
- Verify fixed-point/permille edge behavior and replay schema loud failures with generated audit artifacts.
- Keep visual readiness blocked unless a native 3D renderer/engine capture path produces manifest-backed evidence.

## Full-Game Target

The full-game target adds native menus, local match flow, fighter/loadout selection, deterministic AI/scripted seats, production asset manifests, package smoke tests, fight-film cameras, asset previews, audio/VFX manifests, and automated quality gates.

The production target is now high-fidelity native 3D as recorded in `docs/decisions/0007-high-fidelity-production-target.md`. Earlier standalone fallback visual artifacts are no longer accepted by normal audits or visual verification; if native 3D capture is absent, the correct status is blocked.

The project must not claim full-game completion while any required production system is missing, blocked, or only represented by documentation.

## Out Of Scope Without Explicit Owner Gate

- Online multiplayer readiness.
- Store publishing, deployment, credentials, paid services, telemetry, self-updater, or installer publishing.
- Public-demo readiness, release-candidate readiness, owner-final acceptance, legal clearance, or trademark clearance.

## Presentation

The executable is native Rust. Accepted visual evidence must come from a continuous player-facing high-fidelity native 3D renderer or legally available engine integration, captured with renderer/asset/camera/replay metadata and no truth mutation. Standalone diagrams, proof packets, fallback captures, and browser output cannot satisfy product-presentation or high-fidelity claims.

## Acceptance

`./tools/verify.sh` is the local gate. For the full-game target it must build, test, build and validate assets, run deterministic duels and match sweeps, replay-verify, audit truth, capture screenshots/renders, package locally, smoke-test the package, and validate final artifacts.
