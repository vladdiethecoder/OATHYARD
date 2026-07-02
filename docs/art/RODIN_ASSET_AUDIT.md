# Rodin / Generated Asset Audit Policy

Status: Active fail-closed policy.
Date: 2026-07-02

## Baseline

The uploaded/replacement-batch turntable/image-rollup evidence is V0.5 `candidate_asset_preview` evidence. It is useful for geometry availability, multi-axis orientation checks, and first silhouette review. It is not final visual fidelity, not public-demo readiness, not owner acceptance, and not Elden Ring / For Honor-class presentation.

Current local candidate evidence is primarily under `assets_src/model_candidates/t_73291be5`, `assets/model_candidates/t_73291be5`, `assets/presentation_manifest.json`, and `artifacts/model_candidates/t_73291be5`. Local inspection did not find a completed raw Rodin export packet with generation receipts and terms snapshot.

## Required audit command

```sh
./tools/audit_generated_assets.sh artifacts/asset_audit/latest
```

Required output:

- `artifacts/asset_audit/latest/generated_asset_audit.json`
- `artifacts/asset_audit/latest/generated_asset_audit.md`
- `artifacts/asset_audit/latest/generated_asset_audit.csv`
- `artifacts/asset_audit/latest/blocked_asset_evidence.md`
- `artifacts/asset_audit/latest/asset_state_summary.md`
- `artifacts/asset_audit/latest/generated_asset_quarantine_manifest.json`
- `artifacts/asset_audit/latest/generated_asset_quarantine_report.md`
- `artifacts/asset_audit/latest/generated_asset_production_unblock_matrix.json`
- `artifacts/asset_audit/latest/generated_asset_production_unblock_matrix.md`

Legacy Rodin-named mirror outputs are also kept for compatibility:

- `artifacts/asset_audit/latest/rodin_asset_audit.json`
- `artifacts/asset_audit/latest/rodin_asset_audit.md`
- `artifacts/asset_audit/latest/rodin_asset_audit.csv`

The command must record every generated/imported/model-candidate asset with provenance, license, mesh, material, UV, rig, skin, scale/orientation, truth-joint mapping, contact/physics profile, runtime suitability, and acceptance status.

Unit-046 hardened field requirements per asset include `asset_id`, `asset_class`, `source_path`, `runtime_path`, `generation_import_tool`, `tool_version`, `generation_date`, source prompt/image path-or-hash fields, Rodin task/download/export IDs or explicit missing status, source/runtime/texture hashes, license/commercial/IP-risk status, vertices/indices/triangles/bounds, UV/normal/tangent/material/texture status, rig/skin/truth-joint/contact/mass/collision status, capture/package/truth-isolation status, `acceptance_state`, `production_ready`, `candidate_only`, `blockers`, and `next_action`.

## Fail-closed rules

- Missing source prompt/image or source manifest => `candidate-only`.
- Missing Rodin task/download/terms/subscription evidence => `candidate-only / license-pending`.
- Paid-plan/commercial-use terms not captured at generation date => `license-pending`.
- Any third-party IP similarity risk unresolved => `candidate-only / IP-risk-pending`.
- Missing DCC/interchange source, UV/PBR/material maps, rig/skin where required, contact/physics profile, native production-renderer screenshot, or truth-isolation proof => not production-ready.
- Candidate texture-channel presence is not production material quality. Low-resolution candidate textures may clear channel-presence checks while remaining blocked from production-material acceptance.
- Candidate-only assets must not be placed in the production asset manifest; they belong in a candidate manifest or rejected quarantine.

## Acceptance status meanings

- `candidate`: available for review only.
- `license-pending`: candidate/source evidence exists but license/commercial-use evidence is unresolved.
- `source-approved`: source/provenance/license/IP risk approved, but technical art gates may remain.
- `technical-clean`: machine/DCC/geometry/material-channel checks pass, but production capture, package, owner, or legal gates may remain.
- `gameplay-profiled`: contact/rig/truth-boundary profiles pass where applicable, but production capture, package, owner, or legal gates may remain.
- `in-engine-candidate`: native/candidate renderer evidence exists, but it is not production-renderer acceptance.
- `production-ready`: source-approved plus technical, material, rig, runtime, capture, package, and truth-isolation gates pass.
- `rejected`: not allowed forward without regeneration/rework.
- `debug-only`: local verification artifact only.
