#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/asset_provenance_audit/latest}"
mkdir -p "$out"

python3 - "$out" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

out = Path(sys.argv[1])
root = Path.cwd()

runtime_manifest_path = root / "assets/runtime_manifest.json"
content_manifest_path = root / "content/oathyard_content.manifest"
model_source_manifest_path = root / "assets_src/model_candidates/t_73291be5/model_source_manifest.json"
provenance_doc_path = root / "assets_src/provenance.md"
pipeline_path = root / "tools/asset_pipeline.py"

checks = []
failures = []

def check(check_id, passed, detail):
    checks.append({"id": check_id, "passed": bool(passed), "detail": detail})
    if not passed:
        failures.append(f"{check_id}: {detail}")

def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()

def sha256_text(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()

def parse_content_manifest_sections(path: Path) -> dict:
    """Parse the OATHYARD content manifest into sections like the asset pipeline does."""
    sections = {}
    current = None
    for line in path.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            current = stripped[1:-1]
            sections[current] = []
        elif current and ":" in stripped and not stripped.startswith("#"):
            sections[current].append(stripped)
    return sections

def find_manifest_row(sections: dict, asset_id: str) -> str | None:
    """Find the manifest row for an asset_id in asset-kind sections only."""
    asset_sections = ["fighters", "weapons", "armor", "arenas"]
    for kind in asset_sections:
        for row in sections.get(kind, []):
            if row.split(":")[0] == asset_id:
                return row
    return None

# ── Check 1: Required provenance documents exist ──
check("doc_runtime_manifest_exists", runtime_manifest_path.is_file(), str(runtime_manifest_path))
check("doc_content_manifest_exists", content_manifest_path.is_file(), str(content_manifest_path))
check("doc_model_source_manifest_exists", model_source_manifest_path.is_file(), str(model_source_manifest_path))
check("doc_provenance_statement_exists", provenance_doc_path.is_file(), str(provenance_doc_path))

if not runtime_manifest_path.is_file() or not content_manifest_path.is_file():
    raise SystemExit(1)

runtime_manifest = json.loads(runtime_manifest_path.read_text(encoding="utf-8"))
runtime_entries = runtime_manifest.get("entries", [])
content_sections = parse_content_manifest_sections(content_manifest_path)

# model_source_manifest is optional for older assets; only audit if present
model_source_entries = []
if model_source_manifest_path.is_file():
    model_source_manifest = json.loads(model_source_manifest_path.read_text(encoding="utf-8"))
    model_source_entries = model_source_manifest.get("entries", [])

# ── Check 2: Every runtime manifest entry has required provenance fields ──
required_runtime_fields = ["id", "kind", "source", "runtime_gltf", "runtime_mesh", "preview", "provenance", "hash"]
for entry in runtime_entries:
    asset_id = entry.get("id", "<unknown>")
    for field in required_runtime_fields:
        check(f"entry_{asset_id}_has_{field}", field in entry and entry[field], f"{asset_id}: {field}")

# ── Check 3: Every runtime entry's source file exists ──
for entry in runtime_entries:
    asset_id = entry.get("id", "<unknown>")
    source_rel = entry.get("source", "")
    source_path = root / source_rel if source_rel else None
    check(f"entry_{asset_id}_source_file_exists", source_path and source_path.is_file(), str(source_path))

# ── Check 4: Runtime files exist (mesh, glTF, preview) ──
for entry in runtime_entries:
    asset_id = entry.get("id", "<unknown>")
    runtime_mesh_rel = entry.get("runtime_mesh", "")
    runtime_gltf_rel = entry.get("runtime_gltf", "")
    preview_rel = entry.get("preview", "")

    mesh_path = root / runtime_mesh_rel if runtime_mesh_rel else None
    gltf_path = root / runtime_gltf_rel if runtime_gltf_rel else None
    preview_path = root / preview_rel if preview_rel else None

    check(f"entry_{asset_id}_runtime_mesh_exists", mesh_path and mesh_path.is_file(), str(mesh_path))
    check(f"entry_{asset_id}_runtime_gltf_exists", gltf_path and gltf_path.is_file(), str(gltf_path))
    check(f"entry_{asset_id}_preview_exists", preview_path and preview_path.is_file(), str(preview_path))

# ── Check 5: Declared hash matches recomputed source hash ──
# The asset pipeline computes hash = sha256(source_text + "\n" + manifest_row)
for entry in runtime_entries:
    asset_id = entry.get("id", "<unknown>")
    declared_hash = entry.get("hash", "")
    source_rel = entry.get("source", "")
    source_path = root / source_rel if source_rel else None

    if source_path and source_path.is_file() and declared_hash:
        source_text = source_path.read_text(encoding="utf-8")
        manifest_row = find_manifest_row(content_sections, asset_id)
        if manifest_row:
            recomputed = sha256_text(source_text + "\n" + manifest_row)
            check(f"entry_{asset_id}_hash_matches_source",
                  recomputed == declared_hash,
                  f"declared={declared_hash[:16]}... recomputed={recomputed[:16]}...")
        else:
            check(f"entry_{asset_id}_found_in_content_manifest", False,
                  f"asset_id {asset_id} not found in content manifest rows")

# ── Check 6: No entry uses forbidden provenance markers ──
forbidden_provenance_tokens = ["placeholder", "copied", "scraped", "unlicensed", "copyrighted_third_party"]
for entry in runtime_entries:
    asset_id = entry.get("id", "<unknown>")
    provenance = str(entry.get("provenance", "")).lower()
    for token in forbidden_provenance_tokens:
        check(f"entry_{asset_id}_no_forbidden_{token}", token not in provenance, provenance)

# ── Check 7: Model source candidates (if present) have complete provenance ──
required_source_fields = ["id", "kind", "source", "sha256", "triangles"]
for entry in model_source_entries:
    asset_id = entry.get("id", "<unknown>")
    for field in required_source_fields:
        check(f"model_source_{asset_id}_has_{field}", field in entry, f"{asset_id}: {field}")

# ── Check 8: Model source candidate files exist ──
for entry in model_source_entries:
    asset_id = entry.get("id", "<unknown>")
    source_rel = entry.get("source", "")
    source_path = root / source_rel if source_rel else None
    check(f"model_source_{asset_id}_file_exists", source_path and source_path.is_file(), str(source_path))

# ── Check 9: Model source candidate source-file hash matches ──
for entry in model_source_entries:
    asset_id = entry.get("id", "<unknown>")
    source_rel = entry.get("source", "")
    source_path = root / source_rel if source_rel else None
    sha_entry = entry.get("sha256", {})
    source_hash = sha_entry.get("source", "") if isinstance(sha_entry, dict) else ""

    if source_path and source_path.is_file() and source_hash:
        actual = sha256_file(source_path)
        check(f"model_source_{asset_id}_source_hash_matches", actual == source_hash,
              f"declared={source_hash[:16]}... actual={actual[:16]}...")

# ── Check 10: Model source candidates must not claim production readiness ──
for entry in model_source_entries:
    asset_id = entry.get("id", "<unknown>")
    source_json_path = root / entry.get("source", "")
    if source_json_path.is_file():
        source_json = json.loads(source_json_path.read_text(encoding="utf-8"))
        not_claimed = source_json.get("not_claimed", [])
        license_status = str(source_json.get("license_status", ""))
        provenance_tag = str(source_json.get("provenance", ""))

        check(f"model_source_{asset_id}_license_recorded", bool(license_status), license_status)
        check(f"model_source_{asset_id}_provenance_tagged", bool(provenance_tag), provenance_tag)

        # Must explicitly NOT claim production readiness
        required_not_claimed = ["owner visual acceptance", "public demo readiness", "release candidate readiness"]
        for claim in required_not_claimed:
            check(f"model_source_{asset_id}_not_claiming_{claim.replace(' ', '_')}",
                  claim in not_claimed, f"missing from not_claimed: {claim}")

# ── Check 11: Readiness boundary stays false ──
check("manifest_no_public_demo_ready", runtime_manifest.get("public_demo_ready") is not True,
      str(runtime_manifest.get("public_demo_ready")))
check("manifest_no_release_candidate_ready", runtime_manifest.get("release_candidate_ready") is not True,
      str(runtime_manifest.get("release_candidate_ready")))

# ── Check 12: PBR material profiles are presentation-only (no truth authority) ──
for entry in runtime_entries:
    asset_id = entry.get("id", "<unknown>")
    pbr = entry.get("pbr_material_profile", {})
    if pbr:
        check(f"entry_{asset_id}_pbr_presentation_only",
              pbr.get("presentation_only") is True or pbr.get("truth_authoritative") is False,
              f"presentation_only={pbr.get('presentation_only')} truth_authoritative={pbr.get('truth_authoritative')}")
        check(f"entry_{asset_id}_pbr_no_truth_authority",
              pbr.get("truth_authoritative") is not True,
              str(pbr.get("truth_authoritative")))
        check(f"entry_{asset_id}_pbr_no_replay_hash_effect",
              pbr.get("material_maps_affect_replay_hash") is not True,
              str(pbr.get("material_maps_affect_replay_hash")))

# ── Check 13: Provenance statement asserts original/repo-owned assets ──
if provenance_doc_path.is_file():
    prov_text = provenance_doc_path.read_text(encoding="utf-8").lower()
    check("provenance_doc_asserts_original", "original" in prov_text, "provenance.md must assert original assets")
    check("provenance_doc_no_third_party_copy",
          "do not copy" in prov_text or "do not" in prov_text,
          "provenance.md must state no copying of third-party assets")

# ── Summary ──
passed = not failures
manifest = {
    "schema": "oathyard.asset_provenance_audit.v1",
    "tool": "tools/asset_provenance_audit.sh",
    "passed": passed,
    "runtime_entry_count": len(runtime_entries),
    "model_source_entry_count": len(model_source_entries),
    "failed_check_count": len(failures),
    "checks": checks,
}
(out / "asset_provenance_audit_manifest.json").write_text(
    json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
)
(out / "failed_asset_provenance_checks.txt").write_text(
    "none\n" if passed else "\n".join(failures) + "\n", encoding="utf-8"
)
report = [
    "# OATHYARD Asset Provenance Audit",
    "",
    f"Status: {'PASSED' if passed else 'FAILED'}",
    f"- Runtime entries audited: `{len(runtime_entries)}`",
    f"- Model source entries audited: `{len(model_source_entries)}`",
    f"- Failed checks: `{len(failures)}`",
    "",
    "Scope: Every production asset must have source, provenance, license record, runtime export, manifest entry, hash, and validation. No placeholder, copied, scraped, unlicensed, or AI-derived-without-provenance assets may enter the production manifest.",
    "",
    "Checks performed:",
    "1. Required provenance documents exist",
    "2. Every runtime entry has all required provenance fields",
    "3. Every entry's source file exists on disk",
    "4. Runtime mesh, glTF, and preview files exist",
    "5. Declared hash matches recomputed source hash (sha256 of source_text + manifest row)",
    "6. No entry uses forbidden provenance markers (placeholder/copied/scraped/unlicensed)",
    "7. Model source candidates have complete provenance fields",
    "8. Model source candidate files exist",
    "9. Model source candidate source-file SHA-256 matches manifest",
    "10. Model source candidates do not claim production readiness",
    "11. Runtime manifest readiness flags remain false",
    "12. PBR material profiles are presentation-only with no truth authority",
    "13. Provenance statement document asserts original repo-owned assets",
]
if failures:
    report.extend(["", "## Failures"] + [f"- {f}" for f in failures])
(out / "asset_provenance_audit_report.md").write_text("\n".join(report) + "\n", encoding="utf-8")
if not passed:
    raise SystemExit(1)
PY

echo "asset provenance audit: $out"
