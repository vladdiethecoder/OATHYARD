# 0005: Cross-Platform Verification Matrix

## Decision

OATHYARD's deterministic combat truth must produce byte-for-byte identical
hash artifacts on every supported target platform. This ADR declares the
platform matrix and the verification methodology that gates the
`cross_platform_verified` freeze condition.

### Target platform matrix

| Platform ID | OS | Architecture | Rust target triple |
| --- | --- | --- | --- |
| `linux-x86_64` | Linux (Fedora) | x86_64 | `x86_64-unknown-linux-gnu` |
| `windows-x86_64` | Windows 10/11 | x86_64 | `x86_64-pc-windows-msvc` |
| `macos-arm64` | macOS (Apple Silicon) | aarch64 | `aarch64-apple-darwin` |

All three are first-class release targets. The game is native-PC only; no
mobile or console targets are declared at this milestone.

### Determinism basis

The combat simulation uses:
- Pure integer/fixed-point arithmetic (`Fixed` with `i64` milli scale)
- No floating-point in truth computation
- No platform-specific randomness
- Deterministic contact ordering (`frame_then_attacker_then_defender_then_action_then_target_then_direction`)

Therefore `final_state_hash`, `content_hash`, `initial_state_hash`, `replay.json`,
and `trace.json` must be byte-for-byte identical across all three platforms when
built from the same `Cargo.lock`.

### Verification methodology

1. Each platform runs `tools/cross_platform_verify.sh`, which:
   - Builds with `cargo build --locked`
   - Runs the canonical duel (`examples/duels/basic_oathyard.duel`)
   - Produces a `platform_hash_stamp.json` containing platform metadata
     (OS, arch, rustc version, kernel) and sha256 digests of every hash artifact
   - Writes the stamp to `artifacts/cross_platform/stamps/<platform_id>/`

2. Stamps are exchanged via `tools/cross_platform_hash_exchange.sh`:
   - `--export` bundles the current platform's stamp into a portable tar.gz
   - `--import <bundle>` ingests a stamp from another platform
   - Imported stamps land in `artifacts/cross_platform/stamps/<platform_id>/`

3. The matrix artifact (`cross_platform_matrix.json`) tracks which platforms
   have produced stamps and whether their hashes match.

4. `cross_platform_verified: true` may only be set in a freeze registry entry
   when the matrix shows all declared platforms have produced matching stamps.
   A single-platform run cannot self-attest cross-platform verification.

### Enforcement point

The freeze registry (`artifacts/freeze/v1/index/by_scope/<scope>/<asset_id>.json`)
is the authoritative source for freeze conditions. `freeze_status.rs` reads the
`cross_platform_verified` boolean from the registry. The verification script
writes a registry entry only when automated comparison of stamps from all
declared platforms passes.

## Rationale

The freeze-boundary audit (task t_e1bd2139) identified that
`cross_platform_verified` was a self-attested boolean with no enforcement
infrastructure. `tools/verify.sh` only compared two runs on the same machine,
not across platforms. This closes that gap by:

- Declaring exactly which platforms must agree
- Producing machine-checkable hash stamps with full platform provenance
- Providing an exchange mechanism for stamps between machines
- Enforcing that the registry boolean can only be true when all platforms match

## Consequences

- Cross-platform releases require running the verification script on all three
  target platforms and exchanging stamps before declaring any freeze.
- CI (`.github/workflows/cross-platform.yml`) runs `cargo test --locked` on all
  three platforms as a first-line guard.
- Any platform-specific divergence in hash output is a release-blocking bug
  that must be traced to its root cause in the simulation or serialization code.
