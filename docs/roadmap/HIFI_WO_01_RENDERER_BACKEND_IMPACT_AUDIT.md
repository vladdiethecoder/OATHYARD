# HIFI-WO-01 Renderer Backend Impact Audit

Status: audit artifact only; not implementation evidence.
Date: 2026-06-29
Kanban: t_1941fcf2 (parent epic t_05a6f650)
Work order: HIFI-WO-01 from `docs/roadmap/HIGH_FIDELITY_PRODUCTION_WORK_ORDERS.md`

## Purpose

Measure license, dependency, build, package, capture, input, audio, and truth-boundary impact for every candidate renderer backend path before ADR adoption. This audit informs `docs/decisions/0008-hifi-wo-01-renderer-backend-adr.md`.

## Candidate targets

| ID | Target | pkg-config | License surface | Status |
| --- | --- | --- | --- | --- |
| A | Blocked native-renderer status / non-native status-manifest (current) | x11=1.8.13 | libX11-devel MIT AND X11 | Active local verification only; not product renderer |
| B | Blocked native-renderer status / GLX / OpenGL | x11=1.8.13, gl=1.2 | libglvnd-devel MIT-feh AND MIT-Modern-Variant AND BSD AND GPL-3.0-or-later WITH Autoconf-exception | Safe for disposable spike; recommended immediate ADR path |
| C | Blocked native-renderer status / EGL / OpenGL | x11=1.8.13, egl=1.5, gl=1.2 | libglvnd-devel (same as B) plus libEGL | Measured fallback spike candidate; not primary ADR target |
| D | Raw Wayland / EGL / OpenGL | wayland-client=1.24.0, egl=1.5 | wayland-devel MIT, libglvnd-devel | Deferred: protocol/header/build/capture/package impact unmeasured |
| E | Vulkan direct-loader | vulkan=missing | Vulkan runtime visible (1.4.341) but no pkg-config | Deferred: header/build/capture impact unmeasured |

## Excluded targets

Unity, Unreal, Godot, browser-first frameworks, SDL2 (missing), GLFW (missing), Rust graphics wrappers (winit/wgpu/bevy/miniquad/ash/vulkano/glium/glutin), proprietary renderer SDKs, DCC runtime rendering, vendored blobs, network services, telemetry, and installers are excluded for this slice per `AGENTS.md` forbidden shortcuts and `docs/decisions/0008-hifi-wo-01-renderer-backend-adr.md`.

## Impact dimensions

### License impact

- No third-party source code copied into the project.
- No engine, framework, or proprietary SDK adoption.
- System ABI libraries (libX11, libGL, libEGL) are provided by the host OS package manager under MIT/X11/BSD licenses.
- No new license obligations enter the OATHYARD source tree or package.

### Dependency impact

- Cargo dependency delta: zero. `Cargo.toml` has zero runtime dependencies.
- No vendored dependencies added.
- Spike link surface: host system `x11` and `gl` via `pkg-config` for the generated C spike binary only; this does not enter the Rust project or package.
- `cargo metadata --locked --format-version=1 --no-deps` confirms `dependencies=0`.

### Build impact

- Rust build/test/cargo gates passed: `./tools/build.sh`, `./tools/test.sh`, `cargo build --locked`, `cargo test --locked` all rc 0.
- C spike compile passed: `cc -std=c11 -O2 -Wall -Wextra -Werror -fanalyzer` rc 0 after decoder hardening.
- No build system changes to the Rust project.

### Package impact

- Package tar before spike: ~13363200 bytes.
- Package tar after ADR adoption: ~13434880 bytes (delta from ADR doc inclusion only).
- Spike source, binary, and renderer-spike artifacts are NOT packaged.
- Package manifest retains `public_demo_ready=false`, `release_candidate_ready=false`.

### Capture impact

- Current-run 1920x1080 capture produced from the continuous loop after replay verification.
- Replay hash identical before and after capture: `f17c8f76b9dfae86`.
- Captures are presentation-only evidence; not high-fidelity product visual acceptance.

### Input impact

- No input dependency adopted.
- No physical controller, Steam Deck hardware, or owner input acceptance claimed.
- Existing input command boundary remains governed by native-game-flow/input-target artifacts.

### Audio impact

- No audio backend adopted.
- No mixer/device/loudness/platform claim changed.
- Future audio/VFX must consume hashed truth events after replay verification.

### Truth-boundary impact

- Replay verification before capture: passed.
- Replay verification after capture: passed.
- Final replay hash both times: `f17c8f76b9dfae86`.
- Renderer manifest: `truth_mutation=false`, `presentation_only=true`.
- Renderer reads replay/content hash and runtime glTF geometry only after authoritative hashes.

## Measured tool versions

```text
rustc 1.96.0 (ac68faa20 2026-05-25)
cargo 1.96.0 (30a34c682 2026-05-25)
cc (GCC) 16.1.1 20260515 (Red Hat 16.1.1-2)
python3 Python 3.11.14
pkg-config x11: 1.8.13
pkg-config gl: 1.2
pkg-config egl: 1.5
pkg-config vulkan: missing
pkg-config sdl2: missing
pkg-config glfw3: missing
pkg-config wayland-client: 1.24.0
pkg-config xcb: 1.17.0
pkg-config xi: 1.8.3
pkg-config xcursor: 1.2.3
pkg-config xkbcommon: 1.13.1
glxinfo -B: direct rendering yes, OpenGL 4.6 Mesa 26.1.3
vulkaninfo --summary: Vulkan 1.4.341, AMD RADV + NVIDIA RTX 5090 + llvmpipe visible
```

## ADR-safe shortlist

Only target B (blocked native-renderer status/GLX/OpenGL) is safe to recommend now, and only as a disposable spike. Target C (X11/EGL/OpenGL) is a measured fallback. Targets D and E are deferred until their protocol/header/build/capture/package impact is proven.

## Evidence logs

- `artifacts/t_1941fcf2_renderer_probe.log`
- `artifacts/t_1941fcf2_renderer_link_probe.log`
- `artifacts/t_1941fcf2_oathyard_ldd.log`
- `artifacts/renderer_target/t_1941fcf2/native_presentation_target_report.md`

## Readiness flags preserved

```text
production_renderer_complete: false
owner_visual_acceptance: false
owner_final_acceptance: false
public_demo_ready: false
release_candidate_ready: false
legal_clearance: false
trademark_clearance: false
store_readiness: false
```
