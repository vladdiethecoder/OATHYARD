#!/usr/bin/env python3
"""Safe Rodin API requester for OATHYARD high-fidelity asset candidates.

This submits concept-controlled requests to Hyper3D/Rodin without printing API
keys, downloads returned files, and writes a fail-closed provenance manifest.
Generation output is still only a proposal until Blender import, visual audit,
native-renderer evidence, and owner acceptance pass.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import time
from pathlib import Path
from typing import Any
from urllib.parse import urlparse

import requests

ROOT = Path(__file__).resolve().parents[1]
API_URL = "https://api.hyper3d.com/api/v2/rodin"
STATUS_URL = "https://api.hyper3d.com/api/v2/status"
DOWNLOAD_URL = "https://api.hyper3d.com/api/v2/download"
DEFAULT_SPEC = ROOT / "assets_src/reference/concepts/weapon_diversity_concept_spec.json"
DEFAULT_CROPS = ROOT / "assets_src/reference/concepts/weapon_panel_crops"
PUBLIC_KEY_SOURCE = ROOT / "external/rodin/blender-mcp-rodin-integration/addon.py"

PROMPT_OVERRIDES = {
    "dual_daggers": "Two separate high-fidelity fantasy dual daggers for a dark medieval judicial duel game. Two independent short blades, no connecting bar, each with steel leaf blade, sharpened bevels, separate leather grip, guard, pommel, subtle engraved brass fittings, worn PBR metal, game asset on neutral background.",
    "shield_one_handed": "High-fidelity round shield and one-handed arming sword set for a dark medieval judicial duel game. Round steel-and-leather shield with raised boss, rim rivets, straps, plus a complete separate one-handed sword with blade, crossguard, leather grip and pommel. Worn PBR materials, strong readable silhouettes.",
    "rotary_revolver": "High-fidelity fantasy rotary revolver hand cannon for a dark medieval judicial duel game. Clearly visible revolving cylinder with chamber holes, thick barrel, hammer, trigger guard, leather grip, brass and dark gunmetal, PBR worn metal, not a generic pistol, game asset on neutral background.",
    "hooked_chain": "High-fidelity hooked chain weapon for a dark medieval judicial duel game. Long articulated iron chain with visible links, curved grappling hook on one end, dense counterweight on the other, worn dark steel, readable hook silhouette, PBR materials, game asset on neutral background.",
}


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def redact(text: str) -> str:
    text = re.sub(r"Bearer\s+[A-Za-z0-9._\-]+", "Bearer [REDACTED]", text)
    text = re.sub(r"([A-Za-z0-9_\-]{24,})", "[REDACTED]", text)
    return text


def get_api_key() -> tuple[str | None, str]:
    env_key = os.environ.get("HYPER3D_API_KEY")
    if env_key:
        return env_key, "HYPER3D_API_KEY"
    if PUBLIC_KEY_SOURCE.is_file():
        s = PUBLIC_KEY_SOURCE.read_text(encoding="utf-8", errors="ignore")
        m = re.search(r"RODIN_FREE_TRIAL_KEY\s*=\s*\"([^\"]+)\"", s)
        if m and m.group(1) and m.group(1) != "***":
            return m.group(1), "rodin_public_free_trial_key_from_local_addon"
    return None, "missing"


def find_urls(obj: Any) -> list[str]:
    urls: list[str] = []
    if isinstance(obj, dict):
        for v in obj.values():
            urls.extend(find_urls(v))
    elif isinstance(obj, list):
        for v in obj:
            urls.extend(find_urls(v))
    elif isinstance(obj, str) and obj.startswith(("http://", "https://")):
        urls.append(obj)
    return urls


def request_json(method: str, url: str, headers: dict[str, str], **kwargs) -> dict[str, Any]:
    r = requests.request(method, url, headers=headers, timeout=300, **kwargs)
    try:
        payload = r.json()
    except Exception:
        payload = {"raw_text": redact(r.text[:2000])}
    if not r.ok:
        raise RuntimeError(f"HTTP {r.status_code} {redact(json.dumps(payload)[:2000])}")
    return payload


def submit_one(weapon: dict[str, Any], crop: Path | None, out_dir: Path, args: argparse.Namespace, api_key: str, key_source: str) -> dict[str, Any]:
    prompt = PROMPT_OVERRIDES.get(weapon["id"], weapon.get("generation_notes", weapon["name"]))
    headers = {"Authorization": f"Bearer {api_key}"}
    data = {
        "tier": args.tier,
        "geometry_file_format": "glb",
        "material": "PBR",
        "quality": args.quality,
        "mesh_mode": args.mesh_mode,
        "use_original_alpha": "false",
        "TAPose": "false",
        "preview_render": "true",
        "hd_texture": "true" if args.hd_texture else "false",
        "texture_delight": "true" if args.texture_delight else "false",
        "is_micro": "false",
        "geometry_instruct_mode": args.geometry_instruct_mode,
        "prompt": prompt,
        "seed": str(args.seed),
    }
    files = []
    opened = []
    try:
        if crop and crop.is_file() and not args.prompt_only:
            f = crop.open("rb")
            opened.append(f)
            files.append(("images", (crop.name, f, "image/png")))
        submit = request_json("POST", API_URL, headers, data=data, files=files)
    finally:
        for f in opened:
            f.close()
    subscription_key = submit.get("jobs", {}).get("subscription_key")
    task_uuid = submit.get("uuid")
    if not subscription_key or not task_uuid:
        raise RuntimeError(f"Rodin submit response missing uuid/subscription_key: {redact(json.dumps(submit)[:2000])}")

    status_log = []
    final_status = None
    for attempt in range(args.max_retries):
        status = request_json("POST", STATUS_URL, headers, json={"subscription_key": subscription_key})
        jobs = status.get("jobs", [])
        states = [j.get("status") for j in jobs]
        status_log.append({"attempt": attempt + 1, "states": states})
        if states and all(s == "Done" for s in states):
            final_status = "Done"
            break
        if any(s == "Failed" for s in states):
            raise RuntimeError(f"Rodin task failed: {redact(json.dumps(status)[:2000])}")
        time.sleep(args.poll_interval)
    if final_status != "Done":
        raise TimeoutError(f"Rodin task timed out after {args.max_retries} polls; latest={status_log[-1] if status_log else None}")

    download = request_json("POST", DOWNLOAD_URL, headers, json={"task_uuid": task_uuid})
    urls = find_urls(download)
    raw_dir = out_dir / "rodin_raw"
    raw_dir.mkdir(parents=True, exist_ok=True)
    downloaded = []
    for idx, url in enumerate(urls):
        parsed = urlparse(url)
        name = Path(parsed.path).name or f"download_{idx}"
        if "." not in name:
            name = f"download_{idx}.bin"
        dest = raw_dir / name
        rr = requests.get(url, stream=True, timeout=300)
        if rr.ok:
            with dest.open("wb") as f:
                for chunk in rr.iter_content(1024 * 1024):
                    if chunk:
                        f.write(chunk)
            downloaded.append({"file": str(dest.relative_to(ROOT)), "sha256": sha256_file(dest), "bytes": dest.stat().st_size})
    manifest = {
        "schema": "oathyard.rodin_generation_candidate.v1",
        "asset_id": weapon["id"],
        "name": weapon["name"],
        "source_concept_crop": str(crop.relative_to(ROOT)) if crop and crop.is_file() else None,
        "prompt": prompt,
        "tier": args.tier,
        "quality": args.quality,
        "mesh_mode": args.mesh_mode,
        "material": "PBR",
        "hd_texture": args.hd_texture,
        "texture_delight": args.texture_delight,
        "geometry_instruct_mode": args.geometry_instruct_mode,
        "seed": args.seed,
        "api_key_source": key_source,
        "task_uuid": task_uuid,
        "status_log": status_log,
        "downloaded_files": downloaded,
        "download_response_redacted": json.loads(redact(json.dumps(download))) if isinstance(download, dict) else {},
        "truth_boundary": {"presentation_only": True, "truth_authoritative": False, "does_not_mutate_gameplay_truth": True},
        "not_claimed": ["production asset completion", "owner visual acceptance", "native in-engine runtime capture", "public demo readiness", "release candidate readiness"],
    }
    (out_dir / "rodin_generation_manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return manifest


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--run-id", required=True)
    p.add_argument("--spec", default=str(DEFAULT_SPEC))
    p.add_argument("--only", required=True, help="comma separated weapon ids")
    p.add_argument("--tier", default="Gen-2.5-High")
    p.add_argument("--quality", default="high")
    p.add_argument("--mesh-mode", default="Quad")
    p.add_argument("--geometry-instruct-mode", default="faithful")
    p.add_argument("--seed", type=int, default=17323)
    p.add_argument("--hd-texture", action="store_true")
    p.add_argument("--texture-delight", action="store_true")
    p.add_argument("--prompt-only", action="store_true")
    p.add_argument("--poll-interval", type=int, default=10)
    p.add_argument("--max-retries", type=int, default=60)
    args = p.parse_args()

    api_key, key_source = get_api_key()
    out_root = ROOT / "artifacts/production_candidates" / args.run_id / "rodin"
    out_root.mkdir(parents=True, exist_ok=True)
    if not api_key:
        (out_root / "rodin_blocker.json").write_text(json.dumps({"blocker": "missing Hyper3D/Rodin API key", "checked": ["HYPER3D_API_KEY", str(PUBLIC_KEY_SOURCE.relative_to(ROOT))]}, indent=2) + "\n", encoding="utf-8")
        print(json.dumps({"ok": False, "blocker": "missing Hyper3D/Rodin API key", "out": str(out_root)}, indent=2))
        return 2

    spec_path = Path(args.spec)
    if not spec_path.is_absolute():
        spec_path = ROOT / spec_path
    data = json.loads(spec_path.read_text(encoding="utf-8"))
    wanted = {x.strip() for x in args.only.split(",") if x.strip()}
    weapons = [w for w in data["weapons"] if w["id"] in wanted]
    if not weapons:
        raise SystemExit("no selected weapons found")
    results = []
    failures = []
    for weapon in weapons:
        crop = DEFAULT_CROPS / f"{weapon['id']}.png"
        asset_out = out_root / weapon["id"]
        asset_out.mkdir(parents=True, exist_ok=True)
        try:
            results.append(submit_one(weapon, crop, asset_out, args, api_key, key_source))
            print(json.dumps({"asset_id": weapon["id"], "status": "rodin_done", "download_count": len(results[-1].get("downloaded_files", []))}, indent=2))
        except Exception as e:
            failure = {"asset_id": weapon["id"], "error": redact(str(e))}
            failures.append(failure)
            (asset_out / "rodin_generation_error.json").write_text(json.dumps(failure, indent=2) + "\n", encoding="utf-8")
            print(json.dumps({"asset_id": weapon["id"], "status": "rodin_failed", "error": failure["error"]}, indent=2))
    summary = {
        "schema": "oathyard.rodin_generation_run.v1",
        "run_id": args.run_id,
        "key_source": key_source,
        "selected": sorted(wanted),
        "successes": [m["asset_id"] for m in results],
        "failures": failures,
        "truth_boundary": {"presentation_only": True, "truth_authoritative": False, "does_not_mutate_gameplay_truth": True},
    }
    (out_root / "rodin_generation_run_manifest.json").write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(json.dumps(summary, indent=2))
    return 0 if results else 1


if __name__ == "__main__":
    raise SystemExit(main())
