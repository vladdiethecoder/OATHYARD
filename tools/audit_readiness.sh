#!/usr/bin/env bash
set -euo pipefail

root="${1:-.}"
out="${2:-artifacts/readiness/verify}"

python3 - "$root" "$out" <<'PY'
import json
import re
import sys
from pathlib import Path

root = Path(sys.argv[1])
out = Path(sys.argv[2])
out.mkdir(parents=True, exist_ok=True)

if not root.exists():
    print(f"readiness audit root not found: {root}", file=sys.stderr)
    sys.exit(2)

skip_dirs = {".git", "target", "artifacts", "assets", "__pycache__"}
if root.name == "oathyard-linux-x86_64":
    skip_dirs = set()

text_suffixes = {
    ".md",
    ".txt",
    ".toml",
    ".rs",
    ".sh",
    ".py",
    ".json",
    ".desktop",
    ".manifest",
    ".oysrc",
}


def read_rel(rel):
    path = root / rel
    if not path.is_file():
        return None
    return path.read_text(encoding="utf-8", errors="replace")


def iter_text_files():
    files = []
    for path in root.rglob("*"):
        if not path.is_file():
            continue
        rel_parts = path.relative_to(root).parts
        if any(part in skip_dirs for part in rel_parts):
            continue
        if path.suffix in text_suffixes or path.name in {"LICENSE", "package_manifest.txt", "Cargo.lock"}:
            files.append(path)
    return sorted(files, key=lambda p: p.as_posix())


checks = []
failures = []


def add_check(check_id, passed, detail):
    checks.append({"id": check_id, "passed": bool(passed), "detail": detail})
    if not passed:
        failures.append(f"{check_id}: {detail}")


mode = "package" if (root / "package_manifest.txt").is_file() else "source"

required = [
    "README.md",
    "AGENTS.md",
    "ACCEPTANCE_MAP.md",
    "LICENSE",
    "docs/design/GAME_CANON.md",
    "docs/design/DEMO_SCOPE.md",
    "docs/roadmap/FULL_GAME_ROADMAP.md",
    "docs/roadmap/PUBLISHABLE_KANBAN.md",
    "docs/acceptance/FULL_GAME_ACCEPTANCE.md",
]
for rel in required:
    add_check(f"required_file:{rel}", (root / rel).is_file(), rel)

license_text = read_rel("LICENSE") or ""
license_pending = "PENDING / UNLICENSED" in license_text
add_check("license_pending_unlicensed", license_pending, "LICENSE must remain pending/unlicensed until owner decision")

readme = read_rel("README.md") or ""
agents = read_rel("AGENTS.md") or ""
acceptance = read_rel("ACCEPTANCE_MAP.md") or ""
full_acceptance = read_rel("docs/acceptance/FULL_GAME_ACCEPTANCE.md") or ""
kanban = read_rel("docs/roadmap/PUBLISHABLE_KANBAN.md") or ""

add_check(
    "readme_mentions_license_pending",
    "license-pending/unlicensed" in readme or "PENDING / UNLICENSED" in readme,
    "README must state license-pending/unlicensed",
)
add_check(
    "agents_forbid_readiness_claims",
    "Do not claim native public demo readiness" in agents
    and "legal clearance" in agents
    and "trademark clearance" in agents,
    "AGENTS.md must keep external readiness claims forbidden",
)
add_check(
    "acceptance_separates_local_and_public_gates",
    "Local publishable package" in acceptance and "Public/store publishable release" in acceptance,
    "ACCEPTANCE_MAP.md must separate local package from public/store gates",
)

expected_false_rows = [
    "Public demo readiness",
    "Release-candidate readiness",
    "Owner-final acceptance",
    "Legal clearance",
    "Trademark clearance",
    "Store readiness",
]
for label in expected_false_rows:
    pattern = re.compile(rf"\|\s*{re.escape(label)}\s*\|\s*`false`\s*\|")
    add_check(
        f"acceptance_gate_false:{label}",
        bool(pattern.search(acceptance)),
        f"{label} row must remain false",
    )

add_check(
    "full_acceptance_status_partial",
    "Partial. The deterministic duel foundation is verified. Full-game completion is not yet claimed." in full_acceptance,
    "Full-game acceptance must not claim completion",
)
add_check(
    "kanban_external_gates_locked",
    "Do not claim public demo readiness" in kanban
    and "BLOCKED / EXTERNAL" in kanban
    and "License is pending/unlicensed" in kanban,
    "Roadmap must keep external gate blockers explicit",
)

source = read_rel("src/lib.rs")
if source is not None:
    add_check(
        "source_public_demo_ready_false",
        re.search(r"PUBLIC_DEMO_READY:\s*bool\s*=\s*false", source) is not None,
        "PUBLIC_DEMO_READY const must be false",
    )
    add_check(
        "source_release_candidate_ready_false",
        re.search(r"RELEASE_CANDIDATE_READY:\s*bool\s*=\s*false", source) is not None,
        "RELEASE_CANDIDATE_READY const must be false",
    )
else:
    add_check("source_constants_not_packaged", mode == "package", "source constants absent only in package audit mode")

package_manifest = read_rel("package_manifest.txt")
if package_manifest is not None:
    add_check(
        "package_manifest_public_demo_ready_false",
        "public_demo_ready=false" in package_manifest,
        "package manifest must keep public_demo_ready=false",
    )
    add_check(
        "package_manifest_release_candidate_ready_false",
        "release_candidate_ready=false" in package_manifest,
        "package manifest must keep release_candidate_ready=false",
    )

appstream_blocker = read_rel("packaging/linux/APPSTREAM_BLOCKED.md") or read_rel(
    "docs/packaging/linux-appstream-blocked.md"
)
add_check(
    "appstream_blocked_when_unlicensed",
    appstream_blocker is not None and "Status: BLOCKED_LICENSE_PENDING" in appstream_blocker,
    "AppStream/metainfo must stay blocked while license is pending",
)

true_patterns = [
    ("public_demo_ready", re.compile(r"public_demo_ready\s*[:=]\s*true", re.IGNORECASE)),
    ("release_candidate_ready", re.compile(r"release_candidate_ready\s*[:=]\s*true", re.IGNORECASE)),
    ("owner_final_acceptance", re.compile(r"owner_final_acceptance\s*[:=]\s*true", re.IGNORECASE)),
    ("legal_clearance", re.compile(r"legal_clearance\s*[:=]\s*true", re.IGNORECASE)),
    ("trademark_clearance", re.compile(r"trademark_clearance\s*[:=]\s*true", re.IGNORECASE)),
    ("store_readiness", re.compile(r"store_readiness\s*[:=]\s*true", re.IGNORECASE)),
    ("public_demo_const", re.compile(r"PUBLIC_DEMO_READY:\s*bool\s*=\s*true")),
    ("release_candidate_const", re.compile(r"RELEASE_CANDIDATE_READY:\s*bool\s*=\s*true")),
]

matches = []
for path in iter_text_files():
    text = path.read_text(encoding="utf-8", errors="replace")
    rel = path.relative_to(root).as_posix()
    for name, pattern in true_patterns:
        for match in pattern.finditer(text):
            line = text.count("\n", 0, match.start()) + 1
            matches.append({"file": rel, "line": line, "pattern": name})

add_check(
    "no_machine_readiness_true_flags",
    not matches,
    "no machine-readable readiness flag may be true",
)

package_docs_agree = True
if mode == "package":
    package_docs_agree = (
        "license-pending/unlicensed" in readme
        and "Local publishable package" in acceptance
        and "public_demo_ready=false" in (package_manifest or "")
        and "release_candidate_ready=false" in (package_manifest or "")
    )
add_check(
    "package_docs_agree" if mode == "package" else "source_docs_agree",
    package_docs_agree,
    "package/source docs and manifests must agree on readiness boundaries",
)

passed = not failures
report = {
    "schema": "oathyard.readiness_audit.v1",
    "product": "OATHYARD",
    "mode": mode,
    "license_pending_unlicensed": license_pending,
    "public_demo_ready": False,
    "release_candidate_ready": False,
    "owner_final_acceptance": False,
    "legal_clearance": False,
    "trademark_clearance": False,
    "store_readiness": False,
    "appstream_metadata_generated": False,
    "checks": checks,
    "true_flag_matches": matches,
    "passed": passed,
}

(out / "readiness_audit.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

lines = [
    "# OATHYARD Readiness Drift Audit",
    "",
    f"Status: {'PASSED' if passed else 'FAILED'}",
    f"- Mode: `{mode}`",
    f"- License pending/unlicensed: `{str(license_pending).lower()}`",
    "- Public demo ready: `false`",
    "- Release candidate ready: `false`",
    "- Owner final acceptance: `false`",
    "- Legal clearance: `false`",
    "- Trademark clearance: `false`",
    "- Store readiness: `false`",
    "- AppStream metadata generated: `false`",
    "",
    "## Checks",
]
for check in checks:
    lines.append(
        f"- `{check['id']}`: `{'passed' if check['passed'] else 'failed'}` - {check['detail']}"
    )
if matches:
    lines.extend(["", "## True Flag Matches"])
    for match in matches:
        lines.append(f"- `{match['file']}:{match['line']}` `{match['pattern']}`")
else:
    lines.extend(["", "## True Flag Matches", "- none"])

(out / "readiness_audit_report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")

if not passed:
    for failure in failures:
        print(failure, file=sys.stderr)
    sys.exit(1)

print("readiness audit passed")
PY
