# OATHYARD Frontier Tech Leverage Register

Status date: 2026-06-30
Scope: research and integration boundary, not implementation evidence.

This register converts current frontier graphics, motion, simulation, asset, and facial-animation sources into OATHYARD-specific work constraints. It does not promote any external solver, neural model, renderer, engine, or generated asset into authoritative gameplay truth.

## Non-negotiable OATHYARD boundary

- Authoritative truth remains fixed-step 120 Hz, deterministic, replayable, hash-covered, and integer/fixed-point only.
- Renderer, UI, animation, VFX, audio, camera, asset tools, and external simulation tools may consume truth only after authoritative hashes/replay artifacts exist.
- Frontier systems are assigned to exactly one OATHYARD layer in this register:
  1. `offline_research_authoring`
  2. `runtime_presentation`
  3. `runtime_authoritative_truth`
- `runtime_authoritative_truth` is forbidden by default. A frontier system can enter that layer only after a separate ADR proves deterministic fixed-version replay, no hidden RNG, no wall-clock truth, no gameplay floats, hash coverage, cross-platform verification, and unchanged replay semantics.
- Named integration claims are forbidden unless the exact code/model/SDK, license, build, runtime invocation, hardware requirements, and verification evidence are present in the repo.
- Current raw X11, SVG, PPM, low-poly glTF, debug labels, wireframes, and software-raster captures are debug-local evidence only.

## Source register

### NVIDIA MotionBricks / MotionBricks-style smart primitives

- Source link: https://arxiv.org/abs/2604.24833 ; https://nvlabs.github.io/motionbricks/ ; https://github.com/NVlabs/GR00T-WholeBodyControl/blob/main/motionbricks/README.md
- Date/version: arXiv submitted 2026-04-27; project page/preview release reports 2026-04-27 public preview; paper states ACM TOG / SIGGRAPH 2026.
- License/availability: paper is CC BY 4.0 on arXiv; project page says preview code is in `GR00T-WholeBodyControl/motionbricks`; OATHYARD has not verified or adopted the repository license/checkpoints locally.
- Hardware/toolchain requirements: Python 3.10+, CUDA-capable GPU, Git LFS, conda/pip install path, optional pretrained checkpoints around 2.2 GB per README.
- Intended OATHYARD layer: `runtime_presentation` for an internal `PresentationBricks` layer; `offline_research_authoring` for motion-candidate study.
- Determinism risk: neural sampling, GPU kernels, checkpoint drift, framework version drift, and style/primitive authoring can introduce nondeterministic presentation unless frozen and recorded; must never alter truth hashes.
- IP/provenance risk: mocap/checkpoint license and generated-motion provenance must be recorded before any production use; reference demos cannot be copied as OATHYARD animation identity.
- Integration plan: build `PresentationBricks` as MotionBricks-inspired internal API first: truth poses/events/replay traces in, presentation motion/retargeted render skeleton poses out. Do not claim NVIDIA MotionBricks integration without local access/build/license/runtime proof.
- Fallback plan: authored deterministic pose library, procedural interpolation, or offline-retargeted clips generated from repo-owned sources, all presentation-only.
- Acceptance checks: toggling PresentationBricks leaves replay JSON, trace JSON, final hash, contact packets, cost breakdowns, capability deltas, and end condition byte-identical; capture report states `truth_mutation:false`.

### NVIDIA Warp

- Source link: https://github.com/NVIDIA/warp ; https://nvidia.github.io/warp/stable/
- Date/version: stable docs read as Warp 1.14.0 on 2026-06-29.
- License/availability: GitHub/extracted docs state Apache License 2.0 for Warp; source builds may involve additional third-party licenses such as NVIDIA libmathdx.
- Hardware/toolchain requirements: Python 3.10+, CUDA-capable NVIDIA GPU for GPU path, CPU fallback for some workflows; optional examples/USD dependencies.
- Intended OATHYARD layer: `offline_research_authoring`.
- Determinism risk: GPU floating-point ordering, differentiable kernels, random-number APIs, compiler/cache/version drift, and hardware-specific results are unsuitable for live truth without separate proof.
- IP/provenance risk: low; source license is permissive, but generated data and third-party example assets must carry provenance.
- Integration plan: use Warp only to generate solver-reference datasets for contact/friction/cloth/MPM/FEM/SPH/DEM experiments and to test hypotheses against OATHYARD integer truth abstractions.
- Fallback plan: Python/Rust deterministic reference cases, analytic small-case fixtures, or Project Chrono/MuJoCo/PhysX comparisons.
- Acceptance checks: reference outputs stored as non-authoritative fixtures with version, command, seed, hardware, and hash; OATHYARD truth never imports Warp state directly.

### NVIDIA Isaac Lab

- Source link: https://isaac-sim.github.io/IsaacLab/main/index.html ; https://developer.nvidia.com/isaac/lab ; https://github.com/isaac-sim/IsaacLab
- Date/version: NVIDIA developer page describes Isaac Lab 2.3 early developer preview; GitHub README maps v2.3.x to Isaac Sim 4.5/5.0/5.1.
- License/availability: Isaac Lab framework is BSD-3-Clause; `isaaclab_mimic` is Apache-2.0; Isaac Sim and some dependencies have proprietary terms.
- Hardware/toolchain requirements: NVIDIA GPU/Isaac Sim stack for the intended accelerated workflow; Python environment; Omniverse/Isaac Sim dependency footprint.
- Intended OATHYARD layer: `offline_research_authoring`.
- Determinism risk: large simulator/runtime stack, GPU physics, domain randomization, RL/imitation-learning loops, and Isaac Sim dependency versions cannot be live truth without separate deterministic replay proof.
- IP/provenance risk: datasets, robot assets, simulator assets, and policies must be licensed and recorded; robotics examples cannot become OATHYARD production art or gameplay identity.
- Integration plan: use as optional lab environment for policy/solver/reference comparisons and synthetic tests; export only reduced observations/fixtures into OATHYARD review datasets.
- Fallback plan: internal deterministic fixtures, Newton standalone, MJWarp, Warp examples, or no external solver.
- Acceptance checks: all outputs classified as reference data; AI/planner outputs remain legal action proposals only; truth decides contacts/injuries/capabilities.

### Newton Physics

- Source link: https://github.com/newton-physics/newton ; https://newton-physics.github.io/newton/latest/faq.html
- Date/version: GitHub extraction reports latest release v1.3.0 on 2026-06-11; FAQ read on 2026-06-29.
- License/availability: Apache-2.0 code, CC-BY-4.0 documentation per GitHub extraction; Linux Foundation project initiated by Disney Research, Google DeepMind, and NVIDIA.
- Hardware/toolchain requirements: Python 3.10+; Linux/Windows/macOS CPU; NVIDIA GPU Maxwell+ with driver 545+ for CUDA 12 GPU path.
- Intended OATHYARD layer: `offline_research_authoring`.
- Determinism risk: Warp/GPU kernels, multiple solver backends, MPM/cloth/soft-body contact, and evolving releases can drift; unsuitable as hidden live truth.
- IP/provenance risk: example assets and USD scenes require license tracking; generated simulation data must be stored as reference, not source art.
- Integration plan: optional reference engine for cloth, cable, granular, softbody, robot/contact, and custom-solver experiments; compare against OATHYARD fixed-point cases.
- Fallback plan: Warp-only, Project Chrono, PhysX, MJWarp, or hand-authored deterministic reduced cases.
- Acceptance checks: `tools/sim_reference_compare.sh` records version/import availability and never writes external states into replay truth.

### MuJoCo / MuJoCo Warp (MJWarp)

- Source link: https://mujoco.readthedocs.io/en/latest/mjwarp/ ; https://github.com/google-deepmind/mujoco_warp
- Date/version: GitHub extraction reports MJWarp latest release `v3.10.0.1` on 2026-06-26.
- License/availability: Apache-2.0 per GitHub extraction.
- Hardware/toolchain requirements: Python package `mujoco-warp`, NVIDIA GPU for intended throughput, `uv` source workflow for development.
- Intended OATHYARD layer: `offline_research_authoring`.
- Determinism risk: docs state MJWarp is optimized for throughput rather than low-latency real-time; not currently differentiable via Warp AD; CUDA/GPU/batched-contact behavior can drift by version/hardware.
- IP/provenance risk: MJCF assets and benchmark scenes require license/provenance; robotics scenes are references only.
- Integration plan: use to batch-test contact-rich reduced scenes and compare qualitative/aggregate outcomes against OATHYARD contact/friction abstractions.
- Fallback plan: standard MuJoCo CPU, Newton, Warp, PhysX, Chrono, or internal fixtures.
- Acceptance checks: reference artifacts include MJWarp version, XML/MJCF hash, batch sizes, solver options, hardware, and comparison verdict; OATHYARD truth artifacts remain unchanged.

### NVIDIA PhysX

- Source link: https://nvidia-omniverse.github.io/PhysX/physx/latest/index.html ; https://github.com/NVIDIA-Omniverse/PhysX
- Date/version: PhysX docs extraction reports version 5.8.0 built 2026-05-18; GitHub extraction reports ovphysx latest 0.4.13 on 2026-06-02 using PhysX SDK 5.9.0.
- License/availability: CPU source under BSD-3-Clause; GPU binaries included at no cost per docs; repository license BSD-3-Clause.
- Hardware/toolchain requirements: C++/CUDA optional GPU path; Omniverse/ovphysx optional Python/USD integration.
- Intended OATHYARD layer: `offline_research_authoring`.
- Determinism risk: GPU binaries, soft bodies/liquids/cloth/inflatables, SDF collision, and real-time solver heuristics cannot be authoritative truth without separate deterministic replay proof.
- IP/provenance risk: example scenes, Omniverse assets, and plugin data require tracking; no proprietary sample content may enter production assets.
- Integration plan: use as reference for rigid/soft/contact/fracture/fluid concepts and USD physics interoperability; do not silently replace OATHYARD solver.
- Fallback plan: Newton/Warp/Chrono/MJWarp or reduced deterministic fixtures.
- Acceptance checks: reference comparison report records PhysX version, backend CPU/GPU, scene hash, and differences; truth hashes unchanged.

### Project Chrono

- Source link: https://projectchrono.org/ ; https://api.projectchrono.org/
- Date/version: project site/API inspected 2026-06-29; specific local Chrono version not installed or adopted.
- License/availability: Project Chrono site states BSD-3 license.
- Hardware/toolchain requirements: C++ library; PyChrono via Anaconda for Python; optional modules for vehicles, FEA, granular flows, fluid-solid interaction, visualization.
- Intended OATHYARD layer: `offline_research_authoring`.
- Determinism risk: numerical solvers, timesteppers, collision/friction choices, threading, and floating-point results can drift; not live truth by default.
- IP/provenance risk: sample scenes/assets and co-sim data require provenance; no external art import without license.
- Integration plan: use for rigid/constraint/friction/vehicle-terrain/granular and FEA reference cases where it kills or supports internal hypotheses.
- Fallback plan: Newton, PhysX, Warp, MJWarp, or deterministic reduced internal solvers.
- Acceptance checks: all comparisons are one-way reference reports; no Chrono state enters replay truth.

### Dense contact, FEM, SPH, DEM, MPM, cloth, deformable, and granular solver families

- Source link: Warp example docs, Newton examples, PhysX 5 docs, and Project Chrono feature documentation listed above.
- Date/version: source family inspected 2026-06-29 through the concrete tools above.
- License/availability: inherits the selected tool license; no generic solver-family license exists.
- Hardware/toolchain requirements: selected solver stack, typically CUDA GPU or C++/Python numerical environment.
- Intended OATHYARD layer: `offline_research_authoring`.
- Determinism risk: numerical stiffness, contact island ordering, GPU reductions, solver iteration counts, and float precision make these reference systems only.
- IP/provenance risk: generated benchmark scenes and material tables become project data and must be source/provenance tracked.
- Integration plan: generate reference curves/failure cases for feature-local contact, friction, bind/hook constraints, armor layers, straps/buckles/laces, cloth/leather/mail/plate deformation, flesh/bone/tendon abstractions, footing terrain, debris, and delayed threats.
- Fallback plan: small analytic cases and OATHYARD integer solver tests.
- Acceptance checks: each imported datum names source solver, version, command, seed, units, tolerances, and whether it falsifies an OATHYARD hypothesis.

### Nanite/Lumen-class renderer target, RTX/path tracing, and upscaling references

- Source link: Epic Nanite/Lumen docs: https://dev.epicgames.com/documentation/en-us/unreal-engine/nanite-virtualized-geometry-in-unreal-engine ; https://dev.epicgames.com/documentation/en-us/unreal-engine/lumen-global-illumination-and-reflections-in-unreal-engine ; Unreal Engine EULA: https://www.unrealengine.com/en-US/eula/unreal ; OATHYARD custom-native ADR/audit: `docs/decisions/0008-hifi-wo-01-renderer-backend-adr.md`, `docs/roadmap/HIFI_WO_01_RENDERER_BACKEND_IMPACT_AUDIT.md` ; NVIDIA RTX/DLSS: https://developer.nvidia.com/rtx/ray-tracing ; https://developer.nvidia.com/rtx/dlss ; https://github.com/NVIDIA/DLSS/blob/main/LICENSE.txt ; AMD FSR/FidelityFX: https://gpuopen.com/fsr/ ; https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK ; https://github.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/releases/tag/v2.3.0 ; https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/main/docs/license.md ; Intel XeSS: https://github.com/intel/xess ; https://www.intel.com/content/www/us/en/developer/articles/technical/xess-sr-developer-guide.html ; https://github.com/intel/xess/blob/main/LICENSE.txt
- Date/version: source-checked 2026-06-30. Unreal Nanite/Lumen docs are Unreal Engine 5.8; Nanite is UE virtualized geometry for pixel-scale detail/high object counts and Lumen is UE's dynamic GI/reflections system. OATHYARD custom native path is ADR 0008 dated 2026-06-29T20:34:41Z and currently only a raw X11/GLX/OpenGL spike boundary. NVIDIA DLSS page lists DLSS 4.5 and UE plugin package last updated May 2026 with Streamline 2.11.1 / NGX 310.6.0; the DLSS SDK license file is v. March 14, 2024. AMD sources disagree at the product/package boundary: GPUOpen FSR page lists FSR 4.0.2, while FidelityFX SDK release v2.3.0 dated 2026-06-24 includes FSR Upscaling 4.1.1, FSR Frame Generation 4.0.1, Ray Regeneration 1.2.0, and related 3.x components. Intel XeSS GitHub lists SDK 3.0.1 on 2026-04-16; Intel XeSS-SR Developer Guide 2.0 is dated 2025-03-17.
- License/availability: no renderer/upscaler/engine dependency is adopted. Custom native path currently copies no third-party source and links only host system X11/GL libraries in disposable spike artifacts; no package/runtime dependency adoption. Unreal Engine is available only under Epic's proprietary Unreal Engine EULA; the EULA grants a non-exclusive, non-transferable, non-sublicensable license, imposes seat/royalty terms depending on use, and states the Royalty Rate is 5% of Royalty Revenue unless a reduced rate applies. NVIDIA DLSS/RTX SDKs are under the NVIDIA RTX SDKs License: non-exclusive/non-transferable SDK license, object-code application distribution conditions, no standalone SDK distribution, no reverse engineering, and no endorsement/trademark use without NVIDIA approval. AMD FidelityFX/FSR default license permits binary-form redistribution only with no reverse engineering/decompilation/disassembly; listed source/sample files are MIT; third-party notices must be preserved. Intel XeSS uses Intel Simplified Software License (October 2022): binary-form software may be redistributed without modification, with copyright/terms reproduced, no reverse engineering/decompilation/disassembly, and no Intel endorsement without written permission. All SDK/package terms must be re-read from the exact downloaded package before any adoption ADR.
- Hardware/toolchain requirements: custom native spike currently requires Linux X11/GLX/OpenGL host libraries (`x11=1.8.13`, `gl=1.2`, `egl=1.5` measured; Vulkan/SDL2/GLFW pkg-config missing) and proves only local capture feasibility. Nanite supports current console/desktop platforms using latest drivers with DirectX 12 and Shader Model 6; Nanite streaming expects SSD-class storage and has material/deformation support limits. Lumen uses software or hardware ray tracing paths, mesh distance fields for software tracing, and hardware ray tracing only when supported by video card/RHI/OS. NVIDIA RTX/DLSS requires supported RTX GPU/driver/SDK integration; DLSS Multi Frame Generation/Dynamic MFG is RTX 50/Blackwell-class in the current page. AMD FSR/FidelityFX SDK v2.3.0 sample ecosystem is DirectX 12/HLSL/Windows-oriented with Visual Studio 2022 and Windows 10 SDK 10.0.18362.0 minimum per release notes; GPUOpen FSR 4 page lists DirectX 12 and UE 5.1-5.6 plugin support. Intel XeSS-SR requires Windows 10/11 x64 for its guide path, DirectX 12 with Intel Iris Xe or later or other GPU with SM 6.4/DP4a, DirectX 11 with Intel Arc or later, or Vulkan 1.1 with required extensions/features.
- Intended OATHYARD layer: `runtime_presentation`.
- Determinism risk: renderer frame timing, streaming, culling, temporal effects, upscaling, ray tracing, and GPU work must never feed truth.
- IP/provenance risk: reference bar only; do not copy assets, UI, animations, lore, silhouettes, or proprietary mechanics from Elden Ring, For Honor, Unreal demos, or vendor samples.
- Integration plan: keep this as a decision record, not implementation adoption. Near-term custom native work may continue only as dependency-zero spike evidence under ADR 0008. Unreal/Nanite/Lumen, NVIDIA RTX/DLSS, AMD FSR/FidelityFX, and Intel XeSS each require a separate owner-approved backend/license/dependency ADR with exact package download, license readback, build/link/package delta, runtime invocation, hardware matrix, truth-isolation proof, visual benchmark delta, and rollback plan before entering Cargo/package/runtime paths.
- Fallback plan: current raw X11/PPM/software-raster evidence remains Tier 0 debug-local verification; custom OpenGL/Vulkan/direct native renderer with documented approximations remains the lowest-dependency production path candidate; if proprietary SDK terms, hardware limits, package impact, or visual artifacts fail, disable the candidate and keep `production_renderer_complete:false`.
- Acceptance checks: no candidate may set `production_renderer_complete`, `owner_visual_acceptance`, `public_demo_ready`, or `release_candidate_ready` true from metadata alone. Required checks are current-run build/package/runtime impact logs, presentation truth isolation (`truth_mutation:false` and byte-identical replay JSON/trace/contact/cost/capability/end/hash with candidate on/off), 1920x1080+ native captures, `tools/capture_high_fidelity_screens.sh`, `tools/visual_benchmark.sh`, `tools/research_audit.sh`, and a follow-up implementation ADR before dependency adoption.

Backend/upscaler/interchange decision state on 2026-06-30:

| Candidate | Current decision | Adoption blocker |
| --- | --- | --- |
| Custom native renderer path | Allowed only as dependency-zero spike evidence; no production adoption. | Needs production-renderer ADR with measured visual quality, package delta, input/audio impact, and truth-isolation proof. |
| Unreal Engine Nanite/Lumen-class path | Quality/reference bar only; not adopted. | Epic EULA/royalty/seat review, engine dependency footprint, package/build/capture/input/audio impact, and owner-approved ADR. |
| RTX/path tracing | Reference/runtime-presentation candidate only. | Exact SDK choice/license/package, RTX hardware path, denoiser/upscaler interaction, and package/runtime proof missing. |
| NVIDIA DLSS | Runtime-presentation candidate only. | NVIDIA RTX SDK license/package review, supported GPU/driver matrix, and no-truth-mutation capture proof missing. |
| AMD FSR/FidelityFX | Runtime-presentation candidate only. | Source/package version conflict must be resolved, binary/MIT split and notices read back from exact package, and DirectX/UE/support path measured. |
| Intel XeSS | Runtime-presentation candidate only. | Binary-only Intel license, D3D/Vulkan feature requirements, driver/runtime selection, and package impact not measured. |

### glTF/GLB runtime asset delivery

- Source link: https://www.khronos.org/gltf/ ; https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html
- Date/version: glTF 2.0 is ISO/IEC 12113:2022 per Khronos; spec/resources inspected 2026-06-30.
- License/availability: Khronos describes glTF as a royalty-free asset-delivery specification; the spec text grants conditional copyright permission to use/reproduce the unmodified specification without fee or royalty but notes patent/trademark/adopter-process limits. OATHYARD uses local structural glTF today, not external Khronos validator proof; Khronos glTF-Validator is Apache-2.0 if adopted as tooling.
- Hardware/toolchain requirements: asset exporters/validators; optional Khronos validator, glTF Asset Auditor, texture compression/KTX/BasisU tools, and runtime loader capable of deterministic manifest-hash verification.
- Intended OATHYARD layer: `runtime_presentation`.
- Determinism risk: asset loading must be deterministic and manifest-hash checked; animations/skins are presentation-only unless separately promoted.
- IP/provenance risk: runtime format does not solve ownership; every asset still needs source, provenance/license, manifest, and hashes.
- Integration plan: keep glTF/GLB as runtime skinned mesh/material/animation delivery where feasible; add validator and asset-auditor checks when tools are locally available; do not treat format validity as production art acceptance.
- Fallback plan: current deterministic text mesh/source formats plus generated glTF; later GLB packaging after tool availability and package-size measurement.
- Acceptance checks: every production asset has source, provenance, runtime export, hashes, material/rig/contact metadata, preview, in-engine screenshot, validation result, and false readiness flags until human/owner gates pass.

### OpenUSD / AOUSD source interchange

- Source link: https://aousd.org/ ; https://aousd.org/news/core-spec-announcement/ ; https://aousd.org/news/alliance-for-openusd-announces-new-member-milestone-industrial-momentum-and-core-specification-progress/ ; https://github.com/PixarAnimationStudios/OpenUSD ; https://github.com/PixarAnimationStudios/OpenUSD/blob/release/LICENSE.txt
- Date/version: AOUSD Core Specification 1.0 announced 2025-12-17; AOUSD 2026-03-25 update describes Core Spec 1.1 roadmap and Characters/Motion/Interactivity interest group; Pixar OpenUSD GitHub lists Version 26.05 as latest release on 2026-04-24 and active `dev` commits inspected 2026-06-30.
- License/availability: AOUSD is a standardization body, not a runtime dependency. Pixar OpenUSD source is under the Tomorrow Open Source Technology License 1.0, described in the license file as differing from Apache License 2.0 in Section 6 Trademarks, with bundled third-party licenses; exact binary/source distribution obligations must be re-read from the selected OpenUSD release before tooling adoption.
- Hardware/toolchain requirements: USD C++/Python toolchain/SDK, CMake/compiler/Python dependencies, DCC support, conversion/flattening/validation tools, and package policy for any generated source/interchange files; not currently adopted.
- Intended OATHYARD layer: `offline_research_authoring`.
- Determinism risk: USD composition/layering/variants are source-pipeline concerns; runtime conversion must be hash-stable before use.
- IP/provenance risk: USD can combine many sources; every layer, variant, reference, texture, rig, and external payload needs provenance and license records.
- Integration plan: use OpenUSD or equivalent as source/interchange for high-detail asset production when toolchain is available, then export deterministic runtime glTF/GLB or successor manifests; no USD runtime composition path may affect combat truth.
- Fallback plan: source text specs, Blender/GLB when Blender works, or direct repo-authored glTF sources.
- Acceptance checks: source-to-runtime build records USD/layer hashes, flatten/export command, runtime hash, license/provenance, validation status, and proof that generated runtime assets remain presentation-only unless a separate truth-promotion ADR passes.

### NVIDIA Audio2Face-3D / facial animation

- Source link: https://arxiv.org/abs/2508.16401 ; https://github.com/NVIDIA/Audio2Face-3D ; https://huggingface.co/nvidia/Audio2Face-3D-v3.0
- Date/version: arXiv submitted 2025-08-22; Hugging Face model `Audio2Face-3D-v3.0` released 2025-09-24.
- License/availability: paper states networks/SDK/training framework/example dataset open-sourced; GitHub extraction lists SDK/Maya/UE plugins under MIT, training framework under Apache, NIM under NVIDIA software/product terms, and HF model under NVIDIA Open Model License.
- Hardware/toolchain requirements: Linux/Windows; supported NVIDIA GPU families listed on HF include Pascal through Blackwell; TensorRT inference engine for model card; SDK/plugin toolchain if adopted.
- Intended OATHYARD layer: `runtime_presentation`.
- Determinism risk: audio-driven neural output, emotion conditioning, TensorRT/runtime versions, and model updates are nondeterministic presentation risks; cannot alter combat truth.
- IP/provenance risk: voice/audio data, facial capture data, model licenses, and generated animation provenance must be tracked; no likeness misuse.
- Integration plan: use only for witness/verdict/cinematic/fight-film facial closeups after truth events, not combat contact/injury/cost decisions.
- Fallback plan: authored facial blendshapes, static witness shots, or no facial animation until source/provenance is resolved.
- Acceptance checks: face animation toggles leave truth/replay hashes unchanged; manifest records model/version/license/audio source/provenance and `truth_mutation:false`.

### Generative 3D tools and neural asset/model generators

- Source link: project-specific source must be recorded per tool; no generic source is adopted by this register.
- Date/version: tool-specific; record exact model/version/checkpoint/date before use.
- License/availability: tool-specific; generated output is not production asset acceptance.
- Hardware/toolchain requirements: tool-specific GPU/CPU/DCC/export stack.
- Intended OATHYARD layer: `offline_research_authoring`.
- Determinism risk: prompt/model/checkpoint/sampling seed drift and hidden training-data provenance; outputs are concept/blockout candidates only.
- IP/provenance risk: highest asset-lane risk. Generated assets require source prompt/control image/provenance/license status, human art pass, validation, and in-engine evidence before production consideration.
- Integration plan: use for concept, multiview/blockout, topology ideas, and style exploration only; production assets must pass source-backed pipeline gates.
- Fallback plan: authored repo-owned assets and deterministic text mesh/source specs.
- Acceptance checks: generated/external asset cannot enter production manifest without source file, provenance/license record, runtime export, manifest entry, preview, in-engine screenshot, collision/contact/material metadata, and visual review.

### ComFree-Sim / GPU-parallel analytical contact

- Source link: https://arxiv.org/abs/2603.12185
- Date/version: arXiv 2603.12185 submitted 2026; inspected 2026-06-29.
- License/availability: paper describes open contact engine; verify code/model availability and license before adoption.
- Hardware/toolchain requirements: CUDA GPU; Python/C++ numerical stack.
- Intended OATHYARD layer: `offline_research_authoring`.
- Determinism risk: GPU-parallel analytical contact, 6D contact with tangential/torsional/rolling friction; throughput-oriented, version/hardware drift risk; reference only.
- IP/provenance risk: robotics benchmark scenes/assets require provenance; no external art.
- Integration plan: use as offline reference solver for dense contact scenes (swords, armor, shields, binds, hooks, grabs, footing); compare against OATHYARD contact/friction abstractions.
- Fallback plan: MuJoCo/MJWarp, Newton, PhysX, Chrono, or deterministic reduced fixtures.
- Acceptance checks: reference comparison report records solver version, scene hash, and differences; truth hashes unchanged.

### Kamino / massively parallel multi-body simulation

- Source link: https://arxiv.org/abs/2603.16536
- Date/version: arXiv 2603.16536 submitted 2026; inspected 2026-06-29.
- License/availability: paper describes GPU-based simulator; verify code availability and license before adoption.
- Hardware/toolchain requirements: CUDA GPU; C++/Python.
- Intended OATHYARD layer: `offline_research_authoring`.
- Determinism risk: massively parallel heterogeneous coupled systems with loop topologies; GPU reductions and solver iteration counts can drift; reference only.
- IP/provenance risk: benchmark scenes/assets require provenance tracking.
- Integration plan: study for straps, articulated armor, chain weapons, hooked constraints, bound weapons, and cyborg/apparatus fighters with kinematic loops.
- Fallback plan: PhysX articulations, Newton, MJWarp, or internal constraint solvers.
- Acceptance checks: reference comparison report records solver version, topology, and findings; truth hashes unchanged.

### NVIDIA Cosmos / world foundation models

- Source link: https://arxiv.org/abs/2503.14492 ; https://www.nvidia.com/en-us/ai/cosmos/
- Date/version: Cosmos-Transfer1 paper submitted 2025-03; NVIDIA Cosmos page inspected 2026-06-29.
- License/availability: NVIDIA Cosmos terms; verify license before adoption.
- Hardware/toolchain requirements: NVIDIA GPU; Cosmos model weights/API.
- Intended OATHYARD layer: `offline_research_authoring`.
- Determinism risk: generative world model with spatial controls (segmentation, depth, edges); prompt/model/checkpoint/sampling drift; synthetic data only.
- IP/provenance risk: generated content is synthetic reference; cannot become canonical lore/art/final assets without provenance, license, and art review.
- Integration plan: use for environment reference, synthetic training footage, visual scenario exploration, and test-case generation.
- Fallback plan: authored reference boards, procedural environment generation, or no synthetic reference.
- Acceptance checks: synthetic outputs are labeled as non-canonical reference; no Cosmos-generated content enters production manifest without full asset gates.

### Generative 3D: Hunyuan3D, TRELLIS, LATTICE, Points-to-3D, SymTRELLIS

- Source link: Hunyuan3D: https://github.com/Tencent/Hunyuan3D-2 ; TRELLIS: https://github.com/microsoft/TRELLIS ; LATTICE: https://arxiv.org/abs/2512.03052 ; Points-to-3D: https://arxiv.org/abs/2603.18782
- Date/version: Hunyuan3D-2.0 open-sourced 2025-03; TRELLIS/LATTICE/Points-to-3D papers inspected 2026-06-29.
- License/availability: Hunyuan3D-2 MIT per GitHub; TRELLIS MIT per GitHub; LATTICE/Points-to-3D verify per source; all outputs are draft candidates.
- Hardware/toolchain requirements: GPU inference; Python/CUDA stack.
- Intended OATHYARD layer: `offline_research_authoring`.
- Determinism risk: generative model output is concept/blockout only; prompt/model/checkpoint/sampling drift.
- IP/provenance risk: highest asset-lane risk. Generated meshes require source prompt/control image/provenance/license, human art pass, retopology, UVs, rigging, material work, and in-engine validation.
- Integration plan: use for concept/blockout meshes, rapid ideation, symmetry checks (SymTRELLIS for weapons/armor/shields), and non-final asset drafts.
- Fallback plan: repo-owned authored assets and deterministic text-spec pipeline.
- Acceptance checks: no generated mesh enters production manifest without source file, provenance, license clearance, human art pass, runtime export, validation, and `tools/asset_provenance_audit.sh` pass.

### Nerfstudio / 4D Gaussian Splatting / NeRF reference capture

- Source link: Nerfstudio: https://arxiv.org/abs/2302.04264 ; https://github.com/nerfstudio-project/nerfstudio
- Date/version: Nerfstudio arXiv 2302.04264; GitHub inspected 2026-06-29.
- License/availability: Nerfstudio Apache-2.0 per GitHub; verify 4DGS source/license separately.
- Hardware/toolchain requirements: CUDA GPU; Python; camera capture rig for source footage.
- Intended OATHYARD layer: `offline_research_authoring`.
- Determinism risk: radiance field reconstruction is approximate, viewpoint-dependent, and not authoritative geometry; reference only.
- IP/provenance risk: captured footage/locations require permission and provenance; NeRF output is reference, not production collision truth.
- Integration plan: use for reference capture, photogrammetry, environment study, or cinematic reference of real-world textures/materials/geometry.
- Fallback plan: authored reference boards, procedural materials, or manual art reference.
- Acceptance checks: NeRF/4DGS outputs are labeled non-authoritative reference; no radiance-field geometry enters combat collision without separate deterministic validation.

## OATHYARD-specific implementation requirements

### PresentationBricks internal layer

`PresentationBricks` is the internal OATHYARD name for MotionBricks-inspired presentation motion. It must:

- consume truth poses, action labels, contact events, capability changes, and replay traces after hashes;
- generate locomotion, guard transitions, weapon handling, bind/hook reactions, stumbles, falls, collapse, recovery, object interaction, and fight-film camera motion;
- retarget canonical truth joints to high-fidelity render skeletons;
- never decide hits, contact, damage, action cost, injuries, capability deltas, end states, or replay hashes;
- be covered by `tools/presentation_truth_isolation.sh`.

### Broad-core simulation research

External solvers may be used only as:

- offline reference;
- regression dataset producer;
- AI/planner training source;
- solver-comparison oracle;
- authoring tool.

They may not silently replace OATHYARD truth. Internal truth remains deterministic, fixed-step, fixed-point/integer, replayable, and hash-audited.

### High-fidelity renderer and asset bar

The production renderer must replace or supersede raw-X11/PPM/debug rendering with native high-fidelity 3D evidence at 1920x1080+ minimum. It must support skinned fighters, layered armor/clothing, high-quality weapons, OATHYARD arena environments, PBR/equivalent materials, dynamic lights, shadows, GI/reflection solution or documented approximation, atmosphere, cinematic replay cameras, performance instrumentation, and strict truth isolation.

Do not call any of these high fidelity: PPM line art, SVG timelines, debug overlays, wireframes, cubes/capsules/primitives, untextured meshes, software silhouettes, or screenshots without loaded production assets.

## Required verification commands

- `./tools/research_audit.sh`
- `./tools/presentation_truth_isolation.sh`
- `./tools/build_assets.sh`
- `./tools/validate_assets.sh`
- `./tools/render_asset_previews.sh`
- `./tools/capture_high_fidelity_screens.sh`
- `./tools/visual_benchmark.sh`
- `./tools/sim_reference_compare.sh`
- `./tools/ai_planner_audit.sh`
- `./tools/asset_provenance_audit.sh`
- `./tools/final_acceptance.sh`

Some commands are intentionally fail-closed until a production renderer and production assets exist. A failing high-fidelity benchmark is valid current evidence; it must not be rewritten into readiness language.
