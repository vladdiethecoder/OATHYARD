# OATHYARD spikes

This directory is an archive for disposable toolchain experiments and measured spike results. It is retained for design provenance only.

Rules:

- Spikes are not production substrate, package entrypoints, or release evidence.
- Spike outputs must go under ignored `artifacts/` paths and remain regenerable or disposable.
- A spike can influence production only through an accepted ADR that records dependency footprint, license surface, package impact, deterministic truth boundary, measured results, and explicit non-claims.
- Historical alternate renderer spike implementations are intentionally absent from this tree; the only retained renderer implementation is the native software 3D path in the Rust source.
