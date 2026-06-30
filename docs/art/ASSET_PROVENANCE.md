# Asset Provenance and License Register

Status: Active fail-closed register.
Date: 2026-06-30

## Current source classes

| Source class | Current local evidence | License/export status | Production eligibility |
| --- | --- | --- | --- |
| Repo-owned `.oysrc` low-poly runtime assets | `assets_src/*/*.oysrc`, `assets/runtime_manifest.json` | Repo-owned source text; project `LICENSE` still pending/unlicensed | Debug/local regression only, not high-fidelity production art. |
| Repo-owned model candidates | `assets_src/model_candidates/t_73291be5`, `assets/model_candidates/t_73291be5`, `assets/presentation_manifest.json` | `repo_owned_original_internal_candidate_pending_project_license_review` | Candidate-only until DCC/source approval, external validation, renderer proof, and owner review. |
| Rodin/Hyper3D API outputs | No completed local Rodin export packet found by current inspection | Hyper3D docs/pricing suggest plan-dependent export/use rights; no generation-time terms/account/receipt locally recorded | Candidate-only / license-pending until terms and receipts are captured. |
| Third-party benchmark games | Elden Ring / For Honor references in docs only | No assets may be copied | Quality reference only; not an asset source. |

## Hyper3D/Rodin source research snapshot

Current external-source checks on 2026-06-30:

- `https://developer.hyper3d.ai/get-started/readme-1`: Rodin API requires Business subscription and bearer API key; generated assets are downloaded through async task/status/download flow.
- `https://developer.hyper3d.ai/api-specification/rodin-gen2.5`: Gen-2.5 can generate mesh and textures; output formats include GLB, USDZ, FBX, OBJ, STL; material can be PBR; PBR includes base color, metallicness, normal, and roughness texture; quality/face-count and TApose parameters exist.
- `https://hyper3d.ai/pricing`: Creator plan lists unlimited export and any use; Business plan lists API access and commercial license benefits for ChatAvatar; FAQ says paid plans include broader export and usage rights and current terms should be reviewed.
- `https://hyper3d.ai/Terms`: current extraction returned 404, so it is not a usable terms snapshot.

Conclusion: do not assume Rodin exports are licensed for shipping. Require saved terms, account/plan, generation receipt, download receipt, and owner/legal acceptance before production use.

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
