# 3D-Only Visual Evidence Policy

OATHYARD visual verification is 3D-only.

## Allowed visual evidence

A capture may count as visual evidence only when all of these are true:

1. It is produced by a native 3D renderer or engine client.
2. It is captured from a camera/render path inside that native 3D client.
3. Its manifest records renderer/backend identity, command, asset manifest, camera metadata, replay or trace hash, content hash, capture hash, and resolution.
4. Its manifest records `truth_mutation=false`.
5. The capture is current-run evidence generated after replay/truth verification.

File extension alone never proves visual validity. PNG/JPEG/EXR-like captures are allowed only when the manifest proves they came from the native 3D path above.

## Allowed nonvisual evidence

These remain valid and must be preserved:

- JSON traces and manifests;
- replay files;
- final-state hashes;
- Markdown reports;
- logs, schemas, hash manifests, command output;
- camera/shot manifests and fight-film metadata that do not pretend to be visual proof.

## Forbidden visual substitutes

Normal audits, bundles, and visual gates must not generate or require standalone two-dimensional diagrams, frame dumps, proof packets, debug panels, browser canvas output, or fallback captures as visual evidence.

UI overlays are allowed only when captured inside the native 3D client. Standalone UI mockups or diagrammatic substitutes do not count.

## Blocked status

When no native 3D renderer capture exists, tools must write blocked JSON/Markdown status and keep nonvisual verification passing where possible. They must not fabricate visual readiness from JSON/Markdown alone.

## Gate

`./tools/audit_visual_artifacts.sh` scans tracked files and normal generated verification artifacts. `./tools/verify.sh` calls it after generating the normal replay/bundle/native-status artifacts.
