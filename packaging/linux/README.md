# OATHYARD Linux Desktop Metadata

This directory contains local Linux desktop integration metadata for the packaged native build.

- `io.oathyard.OATHYARD.desktop` launches `oathyard` with no arguments. Package smoke verifies this through `PATH=<package>/bin`.
- The package currently omits standalone icon artwork under the 3D-only visual evidence policy; desktop metadata remains local/package-only and does not count as visual evidence.
- AppStream/metainfo XML is intentionally not generated yet because the project is `PENDING / UNLICENSED`; AppStream requires license metadata that must not be invented by automation.

This metadata does not claim public demo readiness, store readiness, legal clearance, trademark clearance, release-candidate readiness, or owner-final acceptance.
