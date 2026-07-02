# 0001: Stack And Determinism

## Decision

Use Rust with Cargo and no external runtime dependencies for the first OATHYARD slice.

The existing repository already contained `Cargo.toml` with package name `oathyard`, executable name `oathyard`, and `license-file = "LICENSE"`. The license file states `PENDING / UNLICENSED`, so no license is chosen here.

## Inspection Evidence

The repository started near-empty:

- `.gitignore`
- `Cargo.toml`
- `LICENSE`

It was not a Git repository at inspection time.

Available stack:

- `rustc 1.96.0`
- `cargo 1.96.0`
- `/bin/bash`
- `gcc`, `clang`, `cmake`, `make`, `ninja`, `pkg-config`
- `python3`, `node`, `npm`, `zig`

Native graphics dependency check:

- Missing pkg-config metadata: `sdl2`, `glfw3`, `vulkan`
- Present pkg-config metadata: `x11`, `wayland-client`, `egl`, `gl`
- Present Vulkan runtime/tooling command: `vulkaninfo`

Baseline build could not be verified because `Cargo.lock` and the referenced source files did not exist.

## Rationale

Rust/Cargo is the smallest available native source stack already selected by the repo metadata. A no-dependency first slice keeps deterministic build and audit scope small and avoids license review for third-party crates.

Historical note: the first slice used lightweight deterministic presentation artifacts beside trace/report output from the native executable. Superseding policy: standalone non-3D visual outputs are no longer accepted by audits or visual verification. Visual readiness now requires native 3D renderer/engine captures with renderer/asset/camera/replay metadata and `truth_mutation=false`; otherwise the visual gate is blocked. Vulkan runtime/tooling command `vulkaninfo` exists locally, but Vulkan pkg-config metadata is unavailable and no Vulkan renderer is implemented or claimed in this decision.

## Determinism Risks

- Integer overflow: use bounded integer fields, wide integer intermediates for fixed-point/permille edge cases, and checked/saturating operations where capability deltas are applied.
- Serialization drift: replay-relevant outputs use explicit stable field ordering.
- Hidden nondeterminism: truth code avoids RNG, wall-clock time, gameplay floats, and unordered collections.
- Presentation contamination: visual artifacts are generated after truth hashes are computed and consume trace data only.

## Rejected Alternatives

- Unity, Unreal, Godot: too large for this verified source foundation and outside the requested constraints.
- Browser-first app: not acceptable as product presentation for the native target.
- Large Rust dependency stack: unnecessary for the first slice and increases license/determinism audit surface.
- Native Vulkan renderer now: Vulkan runtime/tooling command `vulkaninfo` is present, but Vulkan pkg-config metadata is unavailable; adopting a renderer backend still needs a separate dependency/platform/rendering ADR and measured proof. The current lower-level X11 artifact path keeps the deterministic truth foundation and package smoke small.
