# 0002: Remove Standalone Non-3D Visual Artifacts

Status: Accepted
Date: 2026-07-01

## Decision

OATHYARD no longer accepts standalone non-3D visual artifacts as visual evidence in source, normal generated outputs, replay/export bundles, audits, docs, or visual-verification gates.

Accepted visual evidence must come from a native 3D renderer or engine client and must be backed by a manifest proving:

- native renderer/backend identity;
- current-run capture command;
- renderer, asset, camera, replay, content-hash, and frame/hash metadata;
- `truth_mutation=false`;
- capture came from the native 3D render/camera path, not a standalone diagram or proof packet.

If that native 3D path is absent, visual readiness is blocked. JSON, replay, Markdown, logs, manifests, schemas, and deterministic hashes remain allowed nonvisual evidence.

## Consequences

- Duel runs emit trace/replay/report/hash/fight-film metadata only.
- Replay export bundles contain truth/replay/report/manifest/hash evidence only.
- Bundle verification fails on forbidden visual files or non-3D visual-proof roles.
- Visual gap/benchmark/screen tools must report blocked status rather than generate fallback visual proof.
- `tools/audit_visual_artifacts.sh` is the enforced gate for tracked files and normal verification artifacts.
- Public-demo-ready, release-candidate-ready, owner visual acceptance, and production-renderer-complete remain false unless their separate gates are actually performed.

## Canon preservation

This does not change the deterministic Rust truth core. Truth remains fixed 120 Hz, integer/fixed-point, replay-authoritative, no hidden RNG, no HP/stat shortcut model, and no presentation writes into gameplay truth.
