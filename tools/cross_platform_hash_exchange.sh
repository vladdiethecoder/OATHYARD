#!/usr/bin/env bash
set -euo pipefail

# Cross-platform hash exchange tool for OATHYARD.
#
# Exports platform-specific hash stamps to a portable tar.gz for transfer
# to another platform, imports stamps from other platforms, and compares
# all collected stamps.
#
# Usage:
#   ./tools/cross_platform_hash_exchange.sh --export [--out <bundle.tar.gz>]
#   ./tools/cross_platform_hash_exchange.sh --import <bundle.tar.gz>
#   ./tools/cross_platform_hash_exchange.sh --compare
#
# The export bundles the current platform's stamp from
# artifacts/cross_platform/latest/stamps/<platform_id>/
# The import places stamps into the same directory structure.
# Compare runs the comparison logic from cross_platform_verify.sh.

base_dir="artifacts/cross_platform/latest"
action=""
import_bundle=""
export_path=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --export)
      action="export"; shift ;;
    --import)
      action="import"
      if [[ $# -lt 2 ]]; then echo "--import requires a bundle path" >&2; exit 2; fi
      import_bundle="$2"; shift 2 ;;
    --compare)
      action="compare"; shift ;;
    --out)
      export_path="$2"; shift 2 ;;
    *)
      echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

if [[ -z "$action" ]]; then
  echo "usage: $0 --export | --import <bundle.tar.gz> | --compare" >&2
  exit 2
fi

python3 - "$action" "$import_bundle" "$export_path" <<'PYEOF'
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
import tarfile
import tempfile
from datetime import datetime, timezone
from pathlib import Path

action = sys.argv[1]
import_bundle = sys.argv[2] if len(sys.argv) > 2 else ""
export_path = sys.argv[3] if len(sys.argv) > 3 else ""
root = Path.cwd()
base_dir = root / "artifacts/cross_platform/latest"
stamp_root = base_dir / "stamps"

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
    return f"{s.lower()}-{m.lower()}"

def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            h.update(chunk)
    return h.hexdigest()

# --- Export ---
if action == "export":
    platform_id = detect_platform_id()
    stamp_dir = stamp_root / platform_id
    stamp_file = stamp_dir / "platform_hash_stamp.json"

    if not stamp_file.exists():
        print(f"[hash_exchange] no stamp found for {platform_id}. Run cross_platform_verify.sh first.", file=sys.stderr)
        sys.exit(1)

    if not export_path:
        export_path = str(base_dir / f"oathyard_stamp_{platform_id}.tar.gz")

    export_path = Path(export_path)
    export_path.parent.mkdir(parents=True, exist_ok=True)

    # Create the bundle: tar.gz containing the stamp dir
    with tarfile.open(export_path, "w:gz") as tar:
        tar.add(stamp_file, arcname=f"stamps/{platform_id}/platform_hash_stamp.json")

    # Compute sha256 of the bundle
    bundle_sha = sha256_file(export_path)
    print(f"[hash_exchange] exported {platform_id} stamp to {export_path}")
    print(f"[hash_exchange] bundle sha256: {bundle_sha}")
    print(f"[hash_exchange] transfer this file to the other platform and run:")
    print(f"  ./tools/cross_platform_hash_exchange.sh --import {export_path.name}")
    sys.exit(0)

# --- Import ---
elif action == "import":
    if not import_bundle:
        print("[hash_exchange] --import requires a bundle path", file=sys.stderr)
        sys.exit(2)

    bundle = Path(import_bundle)
    if not bundle.exists():
        print(f"[hash_exchange] bundle not found: {bundle}", file=sys.stderr)
        sys.exit(1)

    stamp_root.mkdir(parents=True, exist_ok=True)

    with tempfile.TemporaryDirectory() as tmp:
        with tarfile.open(bundle, "r:gz") as tar:
            # Safe extraction: only allow stamps/ prefix
            for member in tar.getmembers():
                if not member.name.startswith("stamps/"):
                    print(f"[hash_exchange] refusing to extract non-stamp path: {member.name}", file=sys.stderr)
                    sys.exit(1)
            tar.extractall(tmp)

        imported_root = Path(tmp) / "stamps"
        imported_count = 0
        for plat_dir in imported_root.iterdir():
            if not plat_dir.is_dir():
                continue
            stamp_file = plat_dir / "platform_hash_stamp.json"
            if not stamp_file.exists():
                continue
            target = stamp_root / plat_dir.name
            if target.exists():
                shutil.rmtree(target)
            target.mkdir(parents=True, exist_ok=True)
            shutil.copy2(stamp_file, target / "platform_hash_stamp.json")
            imported_count += 1
            print(f"[hash_exchange] imported stamp for {plat_dir.name}")

    if imported_count == 0:
        print("[hash_exchange] no valid stamps found in bundle", file=sys.stderr)
        sys.exit(1)

    print(f"[hash_exchange] imported {imported_count} stamp(s) into {stamp_root}")
    print(f"[hash_exchange] run --compare to check the cross-platform matrix")
    sys.exit(0)

# --- Compare ---
elif action == "compare":
    stamps = {}
    if stamp_root.exists():
        for plat_dir in sorted(stamp_root.iterdir()):
            if not plat_dir.is_dir():
                continue
            stamp_file = plat_dir / "platform_hash_stamp.json"
            if stamp_file.exists():
                try:
                    stamps[plat_dir.name] = json.loads(stamp_file.read_text(encoding="utf-8"))
                except (json.JSONDecodeError, OSError):
                    pass

    if not stamps:
        print("[hash_exchange] no stamps found. Run cross_platform_verify.sh on each platform.", file=sys.stderr)
        sys.exit(1)

    platforms_present = sorted(stamps.keys())
    declared_present = [p for p in DECLARED_PLATFORMS if p in stamps]
    declared_missing = [p for p in DECLARED_PLATFORMS if p not in stamps]

    hash_values = {}
    replay_hashes = {}
    for plat_id, stamp in stamps.items():
        fsh = stamp.get("final_state_hash")
        if fsh:
            hash_values[plat_id] = fsh
        rep_sha = stamp.get("artifact_sha256", {}).get("replay.json")
        if rep_sha:
            replay_hashes[plat_id] = rep_sha

    unique_final = set(hash_values.values())
    unique_replay = set(replay_hashes.values())
    hashes_match = len(unique_final) <= 1 and len(unique_replay) <= 1
    all_declared_present = len(declared_missing) == 0
    cross_platform_verified = hashes_match and all_declared_present and len(declared_present) >= 2

    print(f"[hash_exchange] platforms with stamps: {platforms_present}")
    print(f"[hash_exchange] declared platforms present: {declared_present}")
    print(f"[hash_exchange] declared platforms missing: {declared_missing}")

    print()
    print(f"{'Platform':<20} {'final_state_hash':<18} {'replay.json sha256 (first 16)'}")
    print("-" * 70)
    for plat_id in platforms_present:
        fsh = hash_values.get(plat_id, "—")
        rep = replay_hashes.get(plat_id, "—")[:16]
        print(f"{plat_id:<20} {fsh:<18} {rep}")

    print()
    if not hashes_match:
        print(f"[hash_exchange] HASH MISMATCH: unique final_state_hashes = {sorted(unique_final)}")
        print(f"[hash_exchange] unique replay.json sha256 = {sorted(unique_replay)}")
        print("[hash_exchange] This is a release-blocking determinism bug.")
        sys.exit(1)

    if cross_platform_verified:
        print("[hash_exchange] CROSS-PLATFORM VERIFIED: all declared platforms match.")
        print("[hash_exchange] The freeze registry's cross_platform_verified may be set true.")
        sys.exit(0)
    else:
        print(f"[hash_exchange] Hashes match but matrix incomplete: {len(declared_missing)} platform(s) missing.")
        print("[hash_exchange] Collect stamps from all declared platforms before setting cross_platform_verified=true.")
        sys.exit(0)

else:
    print(f"[hash_exchange] unknown action: {action}", file=sys.stderr)
    sys.exit(2)
PYEOF
