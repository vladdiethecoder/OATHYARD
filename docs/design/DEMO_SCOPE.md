# OATHYARD Scope Boundaries

## Current Verified Scope

The existing verified slice proves a source-built deterministic local duel foundation:

- Build a native Rust executable from source.
- Run one scripted two-fighter local duel.
- Emit deterministic frame-cost, contact, injury, capability, replay, report, and fight-film artifacts.
- Replay the duel bit-exactly and fail loudly on mismatch.
- Verify loadout variation and injury/capability effects with automated tests.
- Verify fixed-point/permille edge behavior and replay schema loud failures with generated audit artifacts.
- Produce a deterministic inspectable visual artifact.

## Full-Game Target

The full-game target adds native menus, local match flow, fighter/loadout selection, deterministic AI/scripted seats, production asset manifests, package smoke tests, fight-film cameras, asset previews, audio/VFX manifests, and automated quality gates.

The production target is now high-fidelity native 3D as recorded in `docs/decisions/0007-high-fidelity-production-target.md`. Current first-slice/headless/SVG/PPM/raw-X11/low-poly evidence remains useful verification evidence, but it does not satisfy the full-game visual target.

The project must not claim full-game completion while any required production system is missing, blocked, or only represented by documentation.

## Out Of Scope Without Explicit Owner Gate

- Online multiplayer readiness.
- Store publishing, deployment, credentials, paid services, telemetry, self-updater, or installer publishing.
- Public-demo readiness, release-candidate readiness, owner-final acceptance, legal clearance, or trademark clearance.

## Presentation

The executable is native Rust. The current renderer path writes deterministic headless artifacts, raw X11/XWayland captures, software-rasterized PPM frames, and SVG/PPM-style visual evidence. These are local verification backends only. A continuous player-facing high-fidelity native 3D renderer or legally available engine integration must be implemented and verified before any product-presentation or high-fidelity claim. Browser/HTML output is QA-only if introduced.

## Acceptance

`./tools/verify.sh` is the local gate. For the full-game target it must build, test, build and validate assets, run deterministic duels and match sweeps, replay-verify, audit truth, capture screenshots/renders, package locally, smoke-test the package, and validate final artifacts.
