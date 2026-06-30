#!/usr/bin/env bash
set -euo pipefail

root="${1:-.}"
out="${2:-artifacts/secrets/source}"

python3 - "$root" "$out" <<'PY'
import json
import re
import sys
from pathlib import Path

root = Path(sys.argv[1])
out = Path(sys.argv[2])
out.mkdir(parents=True, exist_ok=True)

if not root.exists():
    print(f"secrets audit root not found: {root}", file=sys.stderr)
    sys.exit(2)

skip_dirs = {".git", ".hermes", ".venv", "target", "__pycache__", "node_modules", "venv"}
skip_suffixes = {
    ".a",
    ".bmp",
    ".gif",
    ".gz",
    ".jpg",
    ".jpeg",
    ".o",
    ".png",
    ".ppm",
    ".rlib",
    ".so",
    ".tar",
    ".wav",
    ".zip",
}
text_suffixes = {
    ".desktop",
    ".duel",
    ".env",
    ".gltf",
    ".json",
    ".lock",
    ".manifest",
    ".md",
    ".oysrc",
    ".py",
    ".rs",
    ".sha256",
    ".sh",
    ".svg",
    ".toml",
    ".txt",
}
text_names = {
    ".gitignore",
    "AGENTS.md",
    "Cargo.lock",
    "LICENSE",
    "README.md",
    "package_manifest.txt",
}

secret_patterns = [
    (
        "private_key_block",
        re.compile(r"(?m)^-----BEGIN [A-Z0-9 ]*PRIVATE KEY-----$"),
    ),
    (
        "aws_access_key",
        re.compile(r"\b(?:AKIA|ASIA)[0-9A-Z]{16}\b"),
    ),
    (
        "github_token",
        re.compile(r"\bgh[pousr]_[A-Za-z0-9_]{36,255}\b"),
    ),
    (
        "openai_api_key",
        re.compile(r"\bsk-[A-Za-z0-9]{20,}\b"),
    ),
    (
        "stripe_secret_key",
        re.compile(r"\b(?:sk|rk)_live_[0-9A-Za-z]{16,}\b"),
    ),
    (
        "stripe_webhook_secret",
        re.compile(r"\bwhsec_[0-9A-Za-z]{16,}\b"),
    ),
    (
        "slack_token",
        re.compile(r"\bxox[baprs]-[0-9A-Za-z-]{20,}\b"),
    ),
    (
        "discord_webhook",
        re.compile(r"https://(?:canary\.)?discord(?:app)?\.com/api/webhooks/[0-9]+/[A-Za-z0-9_-]+"),
    ),
]

assignment_pattern = re.compile(
    r"""(?ix)
    \b(
        api[_-]?key|secret|token|password|passwd|private[_-]?key|
        client[_-]?secret|steam[_-]?web[_-]?api[_-]?key
    )\b
    \s*[:=]\s*
    ["']?([^"'\s,;#}{\]]{8,})["']?
    """
)

safe_assignment_values = {
    "blocked",
    "changeme",
    "example",
    "false",
    "local",
    "none",
    "null",
    "pending",
    "placeholder",
    "redacted",
    "test",
    "true",
    "unlicensed",
    "vibecoding",
    "your_api_key",
}


def should_scan(path: Path) -> bool:
    rel_parts = path.relative_to(root).parts
    if any(part in skip_dirs for part in rel_parts):
        return False
    if path.suffix.lower() in skip_suffixes:
        return False
    if path.suffix.lower() in text_suffixes:
        return True
    return path.name in text_names


def redacted(text: str) -> str:
    text = text.replace("\n", "\\n")
    if len(text) <= 12:
        return "<redacted>"
    return f"{text[:4]}...{text[-4:]}"


files = []
for path in root.rglob("*"):
    if path.is_file() and should_scan(path):
        files.append(path)
files.sort(key=lambda p: p.relative_to(root).as_posix())

findings = []
for path in files:
    rel = path.relative_to(root).as_posix()
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError as error:
        findings.append(
            {
                "file": rel,
                "line": 0,
                "kind": "read_error",
                "evidence": str(error),
            }
        )
        continue

    for kind, pattern in secret_patterns:
        for match in pattern.finditer(text):
            line = text.count("\n", 0, match.start()) + 1
            findings.append(
                {
                    "file": rel,
                    "line": line,
                    "kind": kind,
                    "evidence": redacted(match.group(0)),
                }
            )

    for match in assignment_pattern.finditer(text):
        key = match.group(1).lower()
        value = match.group(2)
        normalized = value.strip("\"'").lower()
        line_start = text.rfind("\n", 0, match.start()) + 1
        line_prefix = text[line_start:match.start()].strip().lower()
        expression_prefixes = (
            "args.",
            "bpy.",
            "cfg.",
            "client.",
            "config.",
            "credentials.",
            "e.",
            "env.",
            "finding[",
            "get_",
            "headers[",
            "hide_",
            "match.",
            "os.",
            "prompt",
            "request.",
            "response.",
            "self.",
            "str(",
            "tokenize.",
            "url_",
        )
        if (
            normalized in safe_assignment_values
            or normalized.startswith("<")
            or normalized.startswith("${")
            or normalized.startswith("$")
            or normalized.startswith(expression_prefixes)
            or normalized.startswith("(")
            or normalized.endswith(")")
            or "example" in normalized
            or "placeholder" in normalized
            or "redacted" in normalized
            or line_prefix.startswith(("if ", "elif ", "while ", "assert "))
        ):
            continue
        line = text.count("\n", 0, match.start()) + 1
        findings.append(
            {
                "file": rel,
                "line": line,
                "kind": f"credential_assignment:{key}",
                "evidence": redacted(value),
            }
        )

passed = not findings
report = {
    "schema": "oathyard.secrets_audit.v1",
    "product": "OATHYARD",
    "root_kind": "package" if (root / "package_manifest.txt").is_file() else "source",
    "files_scanned": len(files),
    "findings_count": len(findings),
    "findings": findings,
    "public_demo_ready": False,
    "release_candidate_ready": False,
    "passed": passed,
}

(out / "secrets_audit.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

lines = [
    "# OATHYARD Secrets Audit",
    "",
    f"Status: {'PASSED' if passed else 'FAILED'}",
    f"- Root kind: `{report['root_kind']}`",
    f"- Files scanned: `{len(files)}`",
    f"- Findings: `{len(findings)}`",
    "- Public demo ready: `false`",
    "- Release candidate ready: `false`",
    "",
    "## Findings",
]
if findings:
    for finding in findings:
        lines.append(
            f"- `{finding['kind']}` `{finding['file']}:{finding['line']}` evidence `{finding['evidence']}`"
        )
else:
    lines.append("- none")

(out / "secrets_audit_report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")

if not passed:
    for finding in findings:
        print(
            f"{finding['kind']}: {finding['file']}:{finding['line']} {finding['evidence']}",
            file=sys.stderr,
        )
    sys.exit(1)
PY

python3 -m json.tool "$out/secrets_audit.json" >/dev/null
grep -q '"schema": "oathyard.secrets_audit.v1"' "$out/secrets_audit.json"
grep -q '"findings_count": 0' "$out/secrets_audit.json"
grep -q '"passed": true' "$out/secrets_audit.json"
grep -q 'Status: PASSED' "$out/secrets_audit_report.md"
grep -q 'Findings: `0`' "$out/secrets_audit_report.md"

echo "secrets audit passed"
