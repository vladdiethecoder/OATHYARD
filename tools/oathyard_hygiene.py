#!/usr/bin/env python3
"""OATHYARD artifact + Kanban hygiene.

Creates a single current-state entrypoint under artifacts/current/ and, when
explicitly requested, non-destructively archives stale generated artifact dirs
under artifacts/_stale/<stamp>/ with a manifest.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
ARTIFACTS = ROOT / "artifacts"
TASK_RE = re.compile(r"t_[0-9a-f]{8}")
FOCUS_RE = re.compile(
    r"PHYS|BIOMECH|TISSUE|FLESH|MATERIAL|DEFORM|RENDER|RENDERER|BEVY|WGPU|UNREAL|NATIVE|CAPTURE|HIFI|VISUAL|ASSET|OWNER",
    re.IGNORECASE,
)
PROTECTED_TOP = {
    "current",
    "_stale",
    "kanban",
    "verify_a",
    "verify_b",
    "final",
    "final_verify",
    "final_gates",
    "gates",
    "readiness",
    "secrets",
    "truth_stress",
    "truth_edge",
    "negative_audit",
    "asset_budget",
    "runtime_3d",
    "environment",
    "package",
    "package_smoke",
    "publishable",
}
STALE_NOISE_RE = re.compile(
    r"labeled|overlay|fresh|probe|isolated|ad_hoc|lighting_post|repair_|frontier_upgrade|hifi_|hud_menu|visual_review",
    re.IGNORECASE,
)


@dataclass
class ArtifactEntry:
    path: str
    name: str
    bytes: int
    age_hours: float
    mtime_utc: str
    protected: bool
    active_reference: bool
    stale_candidate: bool
    reason: str


def run(cmd: list[str], *, check: bool = False) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=ROOT, text=True, capture_output=True, check=check)


def utc(ts: float | None = None) -> str:
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(time.time() if ts is None else ts))


def stamp() -> str:
    return time.strftime("%Y%m%dT%H%M%SZ", time.gmtime())


def human(n: int) -> str:
    units = ["B", "KiB", "MiB", "GiB", "TiB"]
    x = float(n)
    for unit in units:
        if x < 1024 or unit == units[-1]:
            return f"{x:.1f} {unit}"
        x /= 1024
    return f"{x:.1f} TiB"


def du_bytes(path: Path) -> int:
    if not path.exists():
        return 0
    cp = subprocess.run(["du", "-sb", "--", str(path)], text=True, capture_output=True)
    if cp.returncode != 0 or not cp.stdout.strip():
        return 0
    return int(cp.stdout.split()[0])


def newest_mtime(path: Path) -> float:
    if not path.exists():
        return 0.0
    if path.is_file() or path.is_symlink():
        return path.stat().st_mtime
    newest = path.stat().st_mtime
    for dirpath, dirnames, filenames in os.walk(path):
        for name in dirnames + filenames:
            try:
                mt = (Path(dirpath) / name).stat().st_mtime
                newest = max(newest, mt)
            except OSError:
                pass
    return newest


def kanban_list(status: str) -> list[dict[str, Any]]:
    cp = run(["hermes", "kanban", "list", "--status", status, "--json"])
    if cp.returncode != 0:
        return []
    try:
        data = json.loads(cp.stdout)
        return data if isinstance(data, list) else []
    except json.JSONDecodeError:
        return []


def kanban_diagnostics() -> dict[str, Any]:
    cp = run(["hermes", "kanban", "diagnostics", "--json"])
    payload: Any = None
    if cp.stdout.strip():
        try:
            payload = json.loads(cp.stdout)
        except json.JSONDecodeError:
            payload = cp.stdout.strip()
    return {"exit_code": cp.returncode, "stdout": payload, "stderr": cp.stderr.strip()}


def collect_kanban() -> dict[str, Any]:
    statuses = {s: kanban_list(s) for s in ["running", "ready", "todo", "blocked", "done"]}
    active = statuses["running"] + statuses["ready"] + statuses["todo"] + statuses["blocked"]
    active_ids = {str(t["id"]) for t in active if isinstance(t.get("id"), str)}
    done_ids = {str(t["id"]) for t in statuses["done"] if isinstance(t.get("id"), str)}
    focus = [
        t
        for t in active
        if FOCUS_RE.search((t.get("title") or "") + "\n" + (t.get("body") or ""))
    ]
    focus.sort(key=lambda t: (-(t.get("priority") or 0), t.get("status") or "", t.get("id") or ""))
    return {
        "statuses": statuses,
        "active_ids": sorted(active_ids),
        "done_ids": sorted(done_ids),
        "focus": focus,
        "diagnostics": kanban_diagnostics(),
    }


def classify_artifact(path: Path, active_ids: set[str], done_ids: set[str], min_age_hours: float) -> ArtifactEntry:
    size = du_bytes(path)
    mt = newest_mtime(path)
    age_h = max(0.0, (time.time() - mt) / 3600.0) if mt else 0.0
    name = path.name
    found_ids = set(TASK_RE.findall(str(path)))
    active_ref = bool(found_ids & active_ids)
    done_ref = bool(found_ids & done_ids)
    protected = name in PROTECTED_TOP or active_ref
    reason = "protected"
    stale = False
    if active_ref:
        reason = "active_task_reference"
    elif name in PROTECTED_TOP:
        reason = "protected_top_level"
    elif age_h < min_age_hours:
        reason = f"younger_than_{min_age_hours:g}h"
    elif done_ref:
        stale = True
        reason = "done_task_artifact"
    elif name.startswith("t_"):
        stale = True
        reason = "inactive_task_artifact"
    elif STALE_NOISE_RE.search(name):
        stale = True
        reason = "stale_probe_or_visual_iteration_artifact"
    elif FOCUS_RE.search(name):
        stale = True
        reason = "old_focus_artifact_not_referenced_by_active_task"
    else:
        reason = "old_nonfocus_artifact_review_required"
    return ArtifactEntry(
        path=str(path.relative_to(ROOT)),
        name=name,
        bytes=size,
        age_hours=round(age_h, 2),
        mtime_utc=utc(mt) if mt else "unknown",
        protected=protected,
        active_reference=active_ref,
        stale_candidate=stale and not protected,
        reason=reason,
    )


def collect_artifacts(active_ids: set[str], done_ids: set[str], min_age_hours: float) -> list[ArtifactEntry]:
    if not ARTIFACTS.exists():
        return []
    entries = []
    for path in ARTIFACTS.iterdir():
        if path.name in {".", ".."}:
            continue
        try:
            entries.append(classify_artifact(path, active_ids, done_ids, min_age_hours))
        except OSError:
            continue
    entries.sort(key=lambda e: e.bytes, reverse=True)
    return entries


def safe_archive(entries: list[ArtifactEntry], max_move: int) -> list[dict[str, Any]]:
    if max_move <= 0:
        return []
    now = stamp()
    dest_root = ARTIFACTS / "_stale" / now
    moves: list[dict[str, Any]] = []
    for entry in [e for e in entries if e.stale_candidate][:max_move]:
        src = (ROOT / entry.path).resolve(strict=False)
        try:
            src.relative_to(ARTIFACTS.resolve(strict=False))
        except ValueError:
            moves.append({"source": entry.path, "error": "outside_artifacts_guard"})
            continue
        if not src.exists():
            moves.append({"source": entry.path, "error": "missing"})
            continue
        if src.name in PROTECTED_TOP:
            moves.append({"source": entry.path, "error": "protected_top_guard"})
            continue
        dest_root.mkdir(parents=True, exist_ok=True)
        dest = dest_root / src.name
        suffix = 1
        while dest.exists():
            suffix += 1
            dest = dest_root / f"{src.name}.{suffix}"
        shutil.move(str(src), str(dest))
        moves.append(
            {
                "source": entry.path,
                "archive": str(dest.relative_to(ROOT)),
                "bytes": entry.bytes,
                "reason": entry.reason,
            }
        )
    if moves:
        manifest = dest_root / "archive_manifest.json"
        manifest.write_text(json.dumps({"created_utc": utc(), "moves": moves}, indent=2) + "\n")
    return moves


def task_line(t: dict[str, Any]) -> str:
    title = (t.get("title") or "").split("\n", 1)[0]
    return f"- `{t.get('id')}` `{t.get('status')}` `{t.get('assignee')}` p{t.get('priority')}: {title}"


def write_outputs(out_dir: Path, kanban: dict[str, Any], artifacts: list[ArtifactEntry], moves: list[dict[str, Any]], args: argparse.Namespace) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    status_counts = {k: len(v) for k, v in kanban["statuses"].items()}
    top = artifacts[:40]
    stale = [e for e in artifacts if e.stale_candidate]
    payload = {
        "generated_utc": utc(),
        "root": str(ROOT),
        "min_age_hours": args.min_age_hours,
        "archive_requested": args.archive,
        "max_move": args.max_move,
        "status_counts": status_counts,
        "diagnostics": kanban["diagnostics"],
        "focus_tasks": kanban["focus"],
        "artifact_inventory": [asdict(e) for e in artifacts],
        "stale_candidates": [asdict(e) for e in stale],
        "archive_moves": moves,
    }
    (out_dir / "kanban_focus.json").write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
    (out_dir / "artifact_inventory.json").write_text(json.dumps([asdict(e) for e in artifacts], indent=2, sort_keys=True) + "\n")

    lines = [
        "# OATHYARD Current Renderer / Physical-Fidelity Focus",
        "",
        f"Generated UTC: `{payload['generated_utc']}`",
        f"Repository: `{ROOT}`",
        "",
        "## Board health",
        "",
        f"- Diagnostics exit code: `{kanban['diagnostics']['exit_code']}`",
        f"- Diagnostics payload: `{json.dumps(kanban['diagnostics']['stdout'], ensure_ascii=False)}`",
        "- Status counts: " + ", ".join(f"`{k}={v}`" for k, v in status_counts.items()),
        "",
        "## Active focus tasks",
        "",
    ]
    if kanban["focus"]:
        lines.extend(task_line(t) for t in kanban["focus"][:30])
    else:
        lines.append("- none")
    lines += [
        "",
        "## Required ordering guard",
        "",
        "Physical-fidelity work gates renderer/capture/import work. Renderer-only evidence does not close For Honor / Elden Ring-class target gaps.",
        "",
        "## Top artifact directories/files",
        "",
    ]
    for e in top:
        flag = "STALE-CANDIDATE" if e.stale_candidate else ("ACTIVE" if e.active_reference else "keep/review")
        lines.append(f"- `{human(e.bytes)}` age `{e.age_hours:.1f}h` `{e.path}` — {flag}; {e.reason}")
    lines += ["", "## Stale candidates", ""]
    if stale:
        for e in stale[:50]:
            lines.append(f"- `{human(e.bytes)}` age `{e.age_hours:.1f}h` `{e.path}` — {e.reason}")
    else:
        lines.append("- none under current retention/classification")
    lines += ["", "## Archive moves this run", ""]
    if moves:
        for m in moves:
            if "archive" in m:
                lines.append(f"- `{human(int(m.get('bytes', 0)))}` `{m['source']}` -> `{m['archive']}` ({m.get('reason')})")
            else:
                lines.append(f"- `{m.get('source')}` ERROR {m.get('error')}")
    else:
        lines.append("- none")
    lines += [
        "",
        "## Commands",
        "",
        "```sh",
        "python3 tools/oathyard_hygiene.py --out artifacts/current --min-age-hours 24",
        "python3 tools/oathyard_hygiene.py --out artifacts/current --min-age-hours 72 --archive --max-move 20",
        "hermes kanban diagnostics --json",
        "```",
        "",
    ]
    text = "\n".join(lines)
    (out_dir / "LATEST.md").write_text(text, encoding="utf-8")
    (out_dir / "kanban_focus.md").write_text(text, encoding="utf-8")
    stale_lines = ["# OATHYARD Stale Artifact Candidates", "", f"Generated UTC: `{utc()}`", ""]
    stale_lines.extend(lines[lines.index("## Stale candidates") :])
    (out_dir / "stale_artifacts.md").write_text("\n".join(stale_lines), encoding="utf-8")


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Maintain OATHYARD current artifact/Kanban focus reports and optionally archive stale generated artifacts.")
    parser.add_argument("--out", default="artifacts/current", help="Output directory for latest focus reports")
    parser.add_argument("--min-age-hours", type=float, default=24.0, help="Only consider stale candidates older than this")
    parser.add_argument("--archive", action="store_true", help="Move stale candidates to artifacts/_stale/<stamp>/")
    parser.add_argument("--max-move", type=int, default=0, help="Maximum stale candidates to move when --archive is set")
    parser.add_argument("--quiet-ok", action="store_true", help="Print only on diagnostics failure or archive moves")
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    out_dir = (ROOT / args.out).resolve(strict=False)
    kanban = collect_kanban()
    active_ids = set(kanban["active_ids"])
    done_ids = set(kanban["done_ids"])
    artifacts = collect_artifacts(active_ids, done_ids, args.min_age_hours)
    moves = safe_archive(artifacts, args.max_move if args.archive else 0)
    if moves:
        # Re-scan after moving so the current report reflects the current tree.
        artifacts = collect_artifacts(active_ids, done_ids, args.min_age_hours)
    write_outputs(out_dir, kanban, artifacts, moves, args)

    diagnostics_ok = kanban["diagnostics"].get("exit_code") == 0 and kanban["diagnostics"].get("stdout") in ([], None)
    stale_count = sum(1 for e in artifacts if e.stale_candidate)
    moved_bytes = sum(int(m.get("bytes", 0)) for m in moves if "bytes" in m)
    summary = f"OATHYARD hygiene: diagnostics_ok={diagnostics_ok} focus={len(kanban['focus'])} stale_candidates={stale_count} moved={len(moves)} moved_bytes={human(moved_bytes)} report={out_dir / 'LATEST.md'}"
    if args.quiet_ok and diagnostics_ok and not moves:
        return 0
    print(summary)
    return 0 if diagnostics_ok else 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
