# Asset Provenance and License Register

Status: Active fail-closed register.
Date: 2026-07-02

## Current source classes

| Source class | Current local evidence | License/export status | Production eligibility |
| --- | --- | --- | --- |
| Repo-owned `.oysrc` low-poly runtime assets | `assets_src/*/*.oysrc`, `assets/runtime_manifest.json` | Repo-owned source text; project `LICENSE` still pending/unlicensed | Debug/local regression only, not high-fidelity production art. |
| Repo-owned model candidates | `assets_src/model_candidates/t_73291be5`, `assets/model_candidates/t_73291be5`, `assets/presentation_manifest.json` | `owner_approved_internal_project_use`; Unit-082 owner context approves Rodin/Meshy/model-generated asset use for internal/project use | Candidate/current-run evidence only until visual QA/benchmark/renderer target gates and owner visual review pass. |
| Rodin/Hyper3D API outputs | Local t_73291be5 candidate metadata and runtime exports | Owner-approved internal/project use in Unit-082 context; no public/store/legal/trademark clearance claimed | Candidate/current-run evidence only; not public-demo, release, store, legal, trademark, or owner-visual accepted. |
| Third-party benchmark games | Elden Ring / For Honor references in docs only | No assets may be copied | Quality reference only; not an asset source. |

## Current generated/model-candidate audit state

Unit-082 reconciles the stale `pending_project_license_review` wording for local model candidates to owner-approved internal/project use. This resolves repository metadata drift for project use only. It does not grant production-ready visuals, shipping, public-demo, owner-accepted, legal, trademark, commercial-store, or store readiness.

Each audited model-candidate record must expose source/runtime/texture hashes, source prompt/image or explicit missing status, Rodin task/download/export IDs or explicit missing status, license/commercial status, protected-IP risk status, geometry counts and bounds, material/texture/UV/normal/tangent status, contact/rig/truth-boundary status, capture/package status, `candidate_only`, `production_ready`, blockers, and the next required unblock action.

## Hyper3D/Rodin source research snapshot

Current external-source checks on 2026-06-30:

- `https://developer.hyper3d.ai/get-started/readme-1`: Rodin API requires Business subscription and bearer API key; generated assets are downloaded through async task/status/download flow.
- `https://developer.hyper3d.ai/api-specification/rodin-gen2.5`: Gen-2.5 can generate mesh and textures; output formats include GLB, USDZ, FBX, OBJ, STL; material can be PBR; PBR includes base color, metallicness, normal, and roughness texture; quality/face-count and TApose parameters exist.
- `https://hyper3d.ai/pricing`: Creator plan lists unlimited export and any use; Business plan lists API access and commercial license benefits for ChatAvatar; FAQ says paid plans include broader export and usage rights and current terms should be reviewed.
- `https://hyper3d.ai/Terms`: current extraction returned 404, so it is not a usable terms snapshot.

Conclusion: current project context approves internal/project use for existing Rodin/Meshy/model-generated assets. Do not infer shipping, public-demo, legal, trademark, commercial-store, or store readiness from that approval; require explicit repo evidence and owner/legal acceptance before those claims.

## Required metadata per asset

- source prompt/source image path and SHA-256;
- generation/import tool, model/tier, version, date, endpoint, and parameters;
- account/subscription plan class without secrets;
- task IDs and download receipt IDs without secrets;
- terms URL, retrieval timestamp, and terms snapshot hash;
- export file paths and SHA-256;
- material/texture file paths and SHA-256;
- IP/originality risk statement;
- commercial use allowed: `true`, `false`, or `unverified`;
- project license status;
- acceptance status.
