#!/usr/bin/env python3
"""
Run Khronos glTF Validator over OATHYARD model-candidate source glTFs.

This creates external glTF-format evidence only. It does not claim Blender/DCC
round-trip validation, topology/manifold acceptance, native renderer capture,
ownership/legal clearance, owner acceptance, or production readiness.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_RUN_ID = "t_73291be5"
VALIDATOR_PACKAGE = "gltf-validator"
NODE_VALIDATOR = r"""
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const validator = require('gltf-validator');

const repoRoot = process.argv[2];
const manifestPath = process.argv[3];
const outPath = process.argv[4];
const toolPath = process.argv[5];
const runId = process.argv[6];
const validatorPackageJson = require('gltf-validator/package.json');

function sha256(file) {
  return crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
}

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

async function validateGltf(absPath) {
  const base = path.dirname(absPath);
  const bytes = new Uint8Array(fs.readFileSync(absPath));
  return await validator.validateBytes(bytes, {
    uri: absPath,
    maxIssues: 1000,
    externalResourceFunction: (uri) => new Promise((resolve, reject) => {
      if (uri.startsWith('data:')) {
        reject(new Error('unexpected data URI in candidate validation input'));
        return;
      }
      const resolved = path.resolve(base, uri);
      fs.readFile(resolved, (err, data) => err ? reject(err) : resolve(new Uint8Array(data)));
    }),
  });
}

(async () => {
  const manifest = readJson(manifestPath);
  const entries = manifest.entries || [];
  const results = [];
  let totalErrors = 0;
  let totalWarnings = 0;
  let totalInfos = 0;
  for (const entry of entries) {
    const gltfRel = entry.runtime_gltf;
    const binRel = entry.runtime_bin;
    const abs = path.resolve(repoRoot, gltfRel);
    const report = await validateGltf(abs);
    const issues = report.issues || {};
    const numErrors = Number(issues.numErrors || 0);
    const numWarnings = Number(issues.numWarnings || 0);
    const numInfos = Number(issues.numInfos || 0);
    totalErrors += numErrors;
    totalWarnings += numWarnings;
    totalInfos += numInfos;
    results.push({
      asset_id: entry.id,
      kind: entry.kind,
      runtime_gltf: gltfRel,
      runtime_gltf_sha256: sha256(abs),
      runtime_bin: binRel,
      runtime_bin_sha256: binRel ? sha256(path.resolve(repoRoot, binRel)) : '',
      passed: numErrors === 0,
      numErrors,
      numWarnings,
      numInfos,
      messages: (issues.messages || []).map((message) => ({
        severity: message.severity,
        code: message.code,
        message: message.message,
        pointer: message.pointer || '',
      })),
      asset: report.asset || {},
      validation_target: 'source_candidate_gltf',
      truth_mutation: false,
      production_ready_after_this_evidence: false,
    });
  }
  const passed = results.length > 0 && results.every((item) => item.passed);
  const evidence = {
    schema: 'oathyard.khronos_gltf_validation.v1',
    tool: toolPath,
    run_id: runId,
    validator_package: 'gltf-validator',
    validator_version: validatorPackageJson.version || '',
    node_version: process.version,
    source_manifest: path.relative(repoRoot, manifestPath),
    source_manifest_sha256: sha256(manifestPath),
    asset_count: results.length,
    passed,
    total_errors: totalErrors,
    total_warnings: totalWarnings,
    total_infos: totalInfos,
    external_khronos_validation_claimed: passed,
    external_dcc_validation_claimed: false,
    production_ready_after_this_evidence: false,
    truth_mutation: false,
    readiness_flags: {
      production_asset_ready: false,
      owner_visual_accepted: false,
      public_demo_visual_ready: false,
      release_candidate_ready: false,
    },
    results,
  };
  fs.writeFileSync(outPath, JSON.stringify(evidence, null, 2) + '\n');
})().catch((err) => {
  console.error(err && err.stack || String(err));
  process.exit(2);
});
"""


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def run(command: list[str], cwd: Path) -> None:
    subprocess.run(command, cwd=cwd, check=True)


def ensure_validator_install(cache_dir: Path) -> Path:
    cache_dir.mkdir(parents=True, exist_ok=True)
    package_json = cache_dir / "node_modules" / VALIDATOR_PACKAGE / "package.json"
    if not package_json.is_file():
        if not (cache_dir / "package.json").is_file():
            run(["npm", "init", "-y"], cache_dir)
        run(["npm", "install", VALIDATOR_PACKAGE, "--no-audit", "--no-fund"], cache_dir)
    if not package_json.is_file():
        raise SystemExit(f"{VALIDATOR_PACKAGE} did not install under {cache_dir}")
    return package_json


def write_markdown(json_path: Path, md_path: Path) -> None:
    evidence = json.loads(json_path.read_text(encoding="utf-8"))
    lines = [
        "# OATHYARD Khronos glTF Validator Evidence",
        "",
        f"Status: {'PASSED' if evidence['passed'] else 'FAILED'}",
        "Evidence class: external Khronos glTF validation for source-candidate glTF files only.",
        "",
        f"- Validator package: `{evidence['validator_package']}` `{evidence['validator_version']}`",
        f"- Asset count: `{evidence['asset_count']}`",
        f"- Total errors: `{evidence['total_errors']}`",
        f"- Total warnings: `{evidence['total_warnings']}`",
        f"- Total infos: `{evidence['total_infos']}`",
        "- External DCC/Blender validation claimed: `false`",
        "- Production asset ready: `false`",
        "- Owner visual accepted: `false`",
        "- Public demo ready: `false`",
        "- Release candidate ready: `false`",
        "- Truth mutation: `false`",
        "",
        "## Per-asset results",
        "",
        "| Asset | Kind | Errors | Warnings | Infos | glTF |",
        "| --- | --- | ---: | ---: | ---: | --- |",
    ]
    for item in evidence["results"]:
        lines.append(
            f"| `{item['asset_id']}` | `{item['kind']}` | {item['numErrors']} | {item['numWarnings']} | {item['numInfos']} | `{item['runtime_gltf']}` |"
        )
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-id", default=DEFAULT_RUN_ID)
    parser.add_argument(
        "--out",
        default="",
        help="Evidence JSON path. Default: assets/model_candidates/<run-id>/validation/khronos_gltf_validation.json",
    )
    parser.add_argument(
        "--cache-dir",
        default="",
        help="Optional npm cache/install directory. Default: temporary directory under /tmp.",
    )
    args = parser.parse_args()

    manifest = ROOT / "assets" / "model_candidates" / args.run_id / "model_candidate_manifest.json"
    if not manifest.is_file():
        raise SystemExit(f"missing model-candidate manifest: {manifest.relative_to(ROOT)}")
    out_json = ROOT / args.out if args.out else ROOT / "assets" / "model_candidates" / args.run_id / "validation" / "khronos_gltf_validation.json"
    out_md = out_json.with_suffix(".md")
    out_json.parent.mkdir(parents=True, exist_ok=True)

    temp_context = None
    if args.cache_dir:
        cache_dir = ROOT / args.cache_dir if not Path(args.cache_dir).is_absolute() else Path(args.cache_dir)
    else:
        temp_context = tempfile.TemporaryDirectory(prefix="oathyard-gltf-validator-")
        cache_dir = Path(temp_context.name)

    try:
        ensure_validator_install(cache_dir)
        node_script = cache_dir / "validate_candidates.cjs"
        node_script.write_text(NODE_VALIDATOR, encoding="utf-8")
        run(
            [
                "node",
                str(node_script),
                str(ROOT),
                str(manifest),
                str(out_json),
                "tools/model_candidates/khronos_validate_candidates.py",
                args.run_id,
            ],
            cache_dir,
        )
        write_markdown(out_json, out_md)
    finally:
        if temp_context is not None:
            temp_context.cleanup()

    evidence = json.loads(out_json.read_text(encoding="utf-8"))
    print(json.dumps({
        "evidence": out_json.relative_to(ROOT).as_posix(),
        "evidence_sha256": sha256_file(out_json),
        "asset_count": evidence["asset_count"],
        "passed": evidence["passed"],
        "total_errors": evidence["total_errors"],
        "total_warnings": evidence["total_warnings"],
        "total_infos": evidence["total_infos"],
    }, indent=2, sort_keys=True))
    return 0 if evidence["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
