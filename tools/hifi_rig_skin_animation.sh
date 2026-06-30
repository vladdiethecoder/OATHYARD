#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/hifi_rig_skin_animation/t_54eabe83}"
scenario="${2:-examples/duels/basic_oathyard.duel}"

mkdir -p "$out"

pre="$out/replay_before_animation_capture"
post="$out/replay_after_animation_capture"

./tools/run_duel.sh "$scenario" --out "$pre"
./tools/replay_verify.sh "$pre/replay.json"

python3 tools/hifi_rig_skin_animation.py \
  --out "$out/evidence" \
  --candidate-manifest assets/model_candidates/t_73291be5/model_candidate_manifest.json \
  --replay-artifacts "$pre"

./tools/run_duel.sh "$scenario" --out "$post"
./tools/replay_verify.sh "$post/replay.json"

python3 - "$out" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
pre = root / "replay_before_animation_capture"
post = root / "replay_after_animation_capture"
evidence = root / "evidence"

def sha(path):
    h = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()

def final_hash(path):
    text = path.read_text(encoding="utf-8")
    data = json.loads(text)
    stack = [data]
    while stack:
        value = stack.pop()
        if isinstance(value, dict):
            for key in ("final_state_hash", "final_hash", "state_hash"):
                candidate = value.get(key)
                if isinstance(candidate, str) and candidate:
                    return candidate
            stack.extend(value.values())
        elif isinstance(value, list):
            stack.extend(value)
    raise SystemExit(f"missing final hash in {path}")

pre_replay = pre / "replay.json"
post_replay = post / "replay.json"
pre_hash = final_hash(pre_replay)
post_hash = final_hash(post_replay)
pre_sha = sha(pre_replay)
post_sha = sha(post_replay)
manifest = evidence / "rig_skin_animation_manifest.json"
validation = evidence / "rig_skin_animation_validation.json"
report = evidence / "rig_skin_animation_report.md"
if pre_hash != post_hash:
    raise SystemExit(f"final hash mismatch after animation capture: {pre_hash} != {post_hash}")
if pre_sha != post_sha:
    raise SystemExit(f"replay JSON sha256 mismatch after animation capture: {pre_sha} != {post_sha}")
validation_payload = json.loads(validation.read_text(encoding="utf-8"))
if validation_payload.get("passed") is not True:
    raise SystemExit(f"rig/skin animation validation failed: {validation_payload.get('failures')}")
summary = {
    "schema": "oathyard.hifi_rig_skin_animation_wrapper.v1",
    "passed": True,
    "pre_replay": pre_replay.as_posix(),
    "post_replay": post_replay.as_posix(),
    "pre_replay_sha256": pre_sha,
    "post_replay_sha256": post_sha,
    "final_state_hash": pre_hash,
    "manifest": manifest.as_posix(),
    "validation": validation.as_posix(),
    "report": report.as_posix(),
    "truth_mutation": False,
    "public_demo_ready": False,
    "release_candidate_ready": False,
    "owner_visual_acceptance": False,
}
(root / "hifi_rig_skin_animation_wrapper.json").write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
(root / "hifi_rig_skin_animation_wrapper_report.md").write_text("\n".join([
    "# OATHYARD HIFI-WO-05 wrapper verification",
    "",
    "Status: PASSED",
    f"- Pre replay: `{pre_replay.as_posix()}` sha256 `{pre_sha}`",
    f"- Post replay: `{post_replay.as_posix()}` sha256 `{post_sha}`",
    f"- Final state hash: `{pre_hash}`",
    f"- Evidence manifest: `{manifest.as_posix()}`",
    f"- Evidence validation: `{validation.as_posix()}`",
    f"- Evidence report: `{report.as_posix()}`",
    "- Truth mutation: `false`",
    "- Public demo ready: `false`",
    "- Release candidate ready: `false`",
    "- Owner visual acceptance: `false`",
    "",
]) + "\n", encoding="utf-8")
print(json.dumps(summary, indent=2, sort_keys=True))
PY

python3 -m json.tool "$out/evidence/rig_skin_animation_manifest.json" >/dev/null
python3 -m json.tool "$out/evidence/rig_skin_animation_validation.json" >/dev/null
python3 -m json.tool "$out/evidence/runtime_animation_state_machine_handoff.json" >/dev/null
python3 -m json.tool "$out/hifi_rig_skin_animation_wrapper.json" >/dev/null

test -s "$out/evidence/pose_sheets/idle_walk_guard_action_pose_sheet.png"
test -s "$out/evidence/pose_sheets/armor_no_clipping_sheet.png"
test -s "$out/evidence/pose_sheets/weapon_grip_alignment_sheet.png"
test -s "$out/evidence/pose_sheets/injury_capability_pose_consequence_sheet.png"
test -s "$out/evidence/pose_sheets/runtime_animation_state_matrix_pose_sheet.png"
test -s "$out/evidence/pose_sheets/rig_separation_anchor_schema_sheet.png"
