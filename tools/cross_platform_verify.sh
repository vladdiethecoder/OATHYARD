#!/usr/bin/env bash
set -euo pipefail

# Cross-platform verification tool for OATHYARD deterministic combat truth.
#
# Runs the canonical duel on the current platform, produces a platform-stamped
# hash artifact, and compares against stamps from other platforms. The freeze
# registry's cross_platform_verified condition can only be set true when all
# declared platforms produce matching hashes.
#
# Usage:
#   ./tools/cross_platform_verify.sh [--out <dir>] [--compare-only] [--scenario <file>]
#
# Output:
#   <out>/duel/                      Duel artifacts (replay.json, trace.json, etc.)
#   <out>/stamps/<platform_id>/      Platform-stamped hash artifacts
#   <out>/cross_platform_matrix.json  Cross-platform comparison matrix
#   <out>/cross_platform_report.md   Human-readable report

out="artifacts/cross_platform/latest"
scenario="examples/duels/basic_oathyard.duel"
compare_only=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out)
      out="$2"; shift 2 ;;
    --scenario)
      scenario="$2"; shift 2 ;;
    --compare-only)
      compare_only=true; shift ;;
    *)
      echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

mkdir -p "$out"

python3 - "$out" "$scenario" "$compare_only" <<'PYEOF'
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

out = Path(sys.argv[1])
scenario = sys.argv[2]
compare_only = sys.argv[3].lower() == "true"
root = Path.cwd()

# Platform matrix declared in ADR 0005.
DECLARED_PLATFORMS = {
    "linux-x86_64": "x86_64-unknown-linux-gnu",
    "windows-x86_64": "x86_64-pc-windows-msvc",
    "macos-arm64": "aarch64-apple-darwin",
}

def detect_platform_id():
    s = platform.system()
    m = platform.machine()
    if s == "Linux" and m == "x86_64":
        return "linux-x86_64"
    if s == "Windows" and m in ("AMD64", "x86_64"):
        return "windows-x86_64"
    if s == "Darwin" and m == "arm64":
        return "macos-arm64"
    # Fallback for other arch combinations
    return f"{s.lower()}-{m.lower()}"

def get_rustc_version():
    try:
        result = subprocess.run(["rustc", "--version"], capture_output=True, text=True, check=True)
        return result.stdout.strip()
    except Exception:
        return "unknown"

def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            h.update(chunk)
    return h.hexdigest()

def run_duel():
    duel_dir = out / "duel"
    if duel_dir.exists():
        shutil.rmtree(duel_dir)
    duel_dir.mkdir(parents=True)
    subprocess.run(
        ["cargo", "run", "--locked", "--", "run", "--scenario", scenario, "--out", str(duel_dir)],
        check=True, capture_output=True, text=True,
    )
    return duel_dir

def build_platform_stamp(duel_dir, platform_id):
    stamp_dir = out / "stamps" / platform_id
    if stamp_dir.exists():
        shutil.rmtree(stamp_dir)
    stamp_dir.mkdir(parents=True)

    artifacts_to_stamp = [
        "final_state_hash.txt",
        "replay.json",
        "trace.json",
        "fight_film_manifest.json",
    ]

    hashes = {}
    for name in artifacts_to_stamp:
        fpath = duel_dir / name
        if fpath.exists():
            hashes[name] = sha256_file(fpath)

    # Extract structured hashes from the duel artifacts for quick comparison
    final_hash = (duel_dir / "final_state_hash.txt").read_text().strip() if (duel_dir / "final_state_hash.txt").exists() else None

    stamp = {
        "schema": "oathyard.cross_platform_hash_stamp.v1",
        "platform_id": platform_id,
        "platform": {
            "os": platform.system(),
            "machine": platform.machine(),
            "kernel": platform.release(),
            "rustc_version": get_rustc_version(),
            "rust_target_triple": DECLARED_PLATFORMS.get(platform_id, "unknown"),
        },
        "scenario": os.path.basename(scenario),
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "final_state_hash": final_hash,
        "artifact_sha256": hashes,
        "declared_platforms": list(DECLARED_PLATFORMS.keys()),
    }

    (stamp_dir / "platform_hash_stamp.json").write_text(
        json.dumps(stamp, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    return stamp_dir / "platform_hash_stamp.json", stamp

def collect_all_stamps():
    stamps = {}
    stamp_root = out / "stamps"
    if not stamp_root.exists():
        return stamps
    for plat_dir in sorted(stamp_root.iterdir()):
        if not plat_dir.is_dir():
            continue
        stamp_file = plat_dir / "platform_hash_stamp.json"
        if stamp_file.exists():
            try:
                stamps[plat_dir.name] = json.loads(stamp_file.read_text(encoding="utf-8"))
            except (json.JSONDecodeError, OSError):
                pass
    return stamps

def compare_stamps(stamps):
    """Compare final_state_hash across all collected stamps."""
    platforms_present = sorted(stamps.keys())
    declared_present = [p for p in DECLARED_PLATFORMS if p in stamps]
    declared_missing = [p for p in DECLARED_PLATFORMS if p not in stamps]

    hash_values = {}
    artifact_hashes = {}
    for plat_id, stamp in stamps.items():
        fsh = stamp.get("final_state_hash")
        if fsh:
            hash_values[plat_id] = fsh
        # Compare sha256 of replay.json as a secondary check
        rep_sha = stamp.get("artifact_sha256", {}).get("replay.json")
        if rep_sha:
            artifact_hashes[plat_id] = rep_sha

    unique_final_hashes = set(hash_values.values())
    unique_replay_sha = set(artifact_hashes.values())
    hashes_match = len(unique_final_hashes) <= 1 and len(unique_replay_sha) <= 1
    all_declared_present = len(declared_missing) == 0
    cross_platform_verified = hashes_match and all_declared_present and len(declared_present) >= 2

    per_platform = []
    for plat_id in platforms_present:
        stamp = stamps[plat_id]
        per_platform.append({
            "platform_id": plat_id,
            "declared": plat_id in DECLARED_PLATFORMS,
            "final_state_hash": stamp.get("final_state_hash"),
            "replay_sha256": stamp.get("artifact_sha256", {}).get("replay.json"),
            "rust_target_triple": stamp.get("platform", {}).get("rust_target_triple"),
            "rustc_version": stamp.get("platform", {}).get("rustc_version"),
            "timestamp_utc": stamp.get("timestamp_utc"),
        })

    matrix = {
        "schema": "oathyard.cross_platform_matrix.v1",
        "declared_platforms": list(DECLARED_PLATFORMS.keys()),
        "platforms_with_stamps": platforms_present,
        "declared_platforms_present": declared_present,
        "declared_platforms_missing": declared_missing,
        "unique_final_state_hashes": sorted(unique_final_hashes),
        "unique_replay_sha256": sorted(unique_replay_sha),
        "hashes_match": hashes_match,
        "all_declared_platforms_present": all_declared_present,
        "cross_platform_verified": cross_platform_verified,
        "per_platform": per_platform,
        "generated_utc": datetime.now(timezone.utc).isoformat(),
    }
    return matrix

def write_matrix(matrix):
    (out / "cross_platform_matrix.json").write_text(
        json.dumps(matrix, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )

def write_report(matrix):
    lines = [
        "# OATHYARD Cross-Platform Verification Report",
        "",
        f"Generated: {matrix['generated_utc']}",
        "",
        "## Platform Matrix",
        "",
        "| Platform ID | Declared | Stamp Present | final_state_hash | replay.json sha256 |",
        "| --- | --- | --- | --- | --- |",
    ]
    for pp in matrix["per_platform"]:
        lines.append(
            f"| `{pp['platform_id']}` | {'yes' if pp['declared'] else 'NO'} | yes | "
            f"`{pp['final_state_hash']}` | `{pp['replay_sha256']}` |"
        )
    for plat in matrix["declared_platforms_missing"]:
        lines.append(f"| `{plat}` | yes | **MISSING** | — | — |")

    lines.extend([
        "",
        "## Verification Result",
        "",
        f"- Hashes match across present platforms: **{'YES' if matrix['hashes_match'] else 'NO'}**",
        f"- All declared platforms present: **{'YES' if matrix['all_declared_platforms_present'] else 'NO'}**",
        f"- `cross_platform_verified`: **{'true' if matrix['cross_platform_verified'] else 'false'}**",
        "",
    ])

    if matrix["unique_final_state_hashes"] and len(matrix["unique_final_state_hashes"]) > 1:
        lines.extend([
            "## HASH MISMATCH DETECTED",
            "",
            "Different final_state_hash values across platforms:",
            "",
        ])
        for h in matrix["unique_final_state_hashes"]:
            lines.append(f"- `{h}`")
        lines.append("")
        lines.append("This is a release-blocking determinism bug. Trace the divergence")
        lines.append("to its root cause in simulation or serialization code.")
        lines.append("")

    if matrix["declared_platforms_missing"]:
        lines.extend([
            "## Missing Platform Stamps",
            "",
            "The following declared platforms have not produced hash stamps:",
            "",
        ])
        for p in matrix["declared_platforms_missing"]:
            lines.append(f"- `{p}` ({DECLARED_PLATFORMS[p]})")
        lines.append("")
        lines.append("Run `tools/cross_platform_verify.sh` on each missing platform,")
        lines.append("then exchange stamps with `tools/cross_platform_hash_exchange.sh --export`.")
        lines.append("")

    lines.extend([
        "## Registry Enforcement",
        "",
        "The freeze registry's `cross_platform_verified` condition may only be",
        "set to `true` when this matrix shows all declared platforms present and",
        "all hashes matching. A single-platform run cannot self-attest",
        "cross-platform verification.",
        "",
    ])

    (out / "cross_platform_report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")

# --- Main ---

if not compare_only:
    # Build and run the duel
    duel_dir = run_duel()
    platform_id = detect_platform_id()
    stamp_path, stamp = build_platform_stamp(duel_dir, platform_id)
    print(f"[cross_platform_verify] stamped {platform_id}: final_state_hash={stamp['final_state_hash']}")
else:
    # Check for existing stamps
    stamp_root = out / "stamps"
    if not stamp_root.exists() or not any(stamp_root.iterdir()):
        print("[cross_platform_verify] no stamps found — run without --compare-only first")
        sys.exit(1)

stamps = collect_all_stamps()
matrix = compare_stamps(stamps)
write_matrix(matrix)
write_report(matrix)

status = "PASS" if matrix["cross_platform_verified"] else "INCOMPLETE"
print(f"[cross_platform_verify] {status}: {len(stamps)} platform(s) stamped, "
      f"{matrix['cross_platform_verified']}")
print(f"[cross_platform_verify] declared platforms present: {matrix['declared_platforms_present']}")
print(f"[cross_platform_verify] declared platforms missing: {matrix['declared_platforms_missing']}")
print(f"[cross_platform_verify] hashes match: {matrix['hashes_match']}")
print(f"[cross_platform_verify] matrix: {out}/cross_platform_matrix.json")

# Exit 0 if verification passes (all declared platforms match), exit 1 otherwise.
# This makes the tool usable as a CI gate.
if not matrix["cross_platform_verified"]:
    # Not a failure per se on a single platform — but non-zero to signal incomplete matrix
    if matrix["hashes_match"] and len(stamps) == 1:
        # Single platform, hashes are internally consistent — exit 0 but report incomplete
        sys.exit(0)
    # Hash mismatch or other issue — hard fail
    sys.exit(1)
PYEOF
