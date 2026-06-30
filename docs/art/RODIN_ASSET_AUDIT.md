# Rodin / Generated Asset Audit Policy

Status: Active fail-closed policy.
Date: 2026-06-30

## Baseline

The uploaded/replacement-batch turntable/contact-sheet evidence is V0.5 `candidate_asset_preview` evidence. It is useful for geometry availability, multi-axis orientation checks, and first silhouette review. It is not final visual fidelity, not public-demo readiness, not owner acceptance, and not Elden Ring / For Honor-class presentation.

Current local candidate evidence is primarily under `assets_src/model_candidates/t_73291be5`, `assets/model_candidates/t_73291be5`, `assets/presentation_manifest.json`, and `artifacts/model_candidates/t_73291be5`. Local inspection did not find a completed raw Rodin export packet with generation receipts and terms snapshot.

## Required audit command

```sh
./tools/audit_rodin_assets.sh artifacts/asset_audit/latest
```

Required output:

- `artifacts/asset_audit/latest/rodin_asset_audit.json`
- `artifacts/asset_audit/latest/rodin_asset_audit.md`
- `artifacts/asset_audit/latest/rodin_asset_audit.csv`

The command must record every generated/imported/model-candidate asset with provenance, license, mesh, material, UV, rig, skin, scale/orientation, truth-joint mapping, contact/physics profile, runtime suitability, and acceptance status.

## Fail-closed rules

- Missing source prompt/image or source manifest => `candidate-only`.
- Missing Rodin task/download/terms/subscription evidence => `candidate-only / license-pending`.
- Paid-plan/commercial-use terms not captured at generation date => `license-pending`.
- Any third-party IP similarity risk unresolved => `candidate-only / IP-risk-pending`.
- Missing DCC/interchange source, UV/PBR/material maps, rig/skin where required, contact/physics profile, native in-engine screenshot, or truth-isolation proof => not production-ready.
- Candidate-only assets must not be placed in the production asset manifest; they belong in a candidate manifest or rejected quarantine.

## Acceptance status meanings

- `candidate`: available for review only.
- `source-approved`: source/provenance/license/IP risk approved, but technical art gates may remain.
- `production-ready`: source-approved plus technical, material, rig, runtime, capture, package, and truth-isolation gates pass.
- `rejected`: not allowed forward without regeneration/rework.
- `debug-only`: local verification artifact only.
