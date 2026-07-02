#!/usr/bin/env bash
set -euo pipefail

scenario="${1:-examples/duels/basic_oathyard.duel}"
out="${2:-artifacts/animation_state_machine/latest}"

mkdir -p "$out"

pre="$out/replay_before_animation"
post="$out/replay_after_animation"

./tools/run_duel.sh "$scenario" --out "$pre"
./tools/replay_verify.sh "$pre/replay.json"

cargo run --locked -- animation-state-machine --scenario "$scenario" --out "$out/evidence"

./tools/run_duel.sh "$scenario" --out "$post"
./tools/replay_verify.sh "$post/replay.json"

python3 - "$out" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
pre = root / "replay_before_animation"
post = root / "replay_after_animation"
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

manifest = evidence / "animation_state_machine_manifest.json"
sequence = evidence / "animation_state_sequence.json"
retargeting = evidence / "animation_retargeting_bridge.json"
report = evidence / "animation_state_machine_report.md"

if pre_hash != post_hash:
    raise SystemExit(
        f"final hash mismatch after animation state machine: {pre_hash} != {post_hash}"
    )
if pre_sha != post_sha:
    raise SystemExit(
        f"replay JSON sha256 mismatch after animation state machine: {pre_sha} != {post_sha}"
    )

manifest_payload = json.loads(manifest.read_text(encoding="utf-8"))
if manifest_payload.get("truth_mutation") is not False:
    raise SystemExit("manifest truth_mutation is not false")
if manifest_payload.get("presentation_only") is not True:
    raise SystemExit("manifest presentation_only is not true")
if manifest_payload.get("layer") != "runtime_presentation":
    raise SystemExit("manifest layer is not runtime_presentation")

state_labels = manifest_payload.get("state_labels", [])
reaction_labels = manifest_payload.get("reaction_labels", [])
expected_states = [
    "observe", "plan", "step", "pivot", "guard", "parry",
    "cut", "thrust", "brace", "bash", "hook_bind", "grab",
    "shove", "kick", "recover",
]
expected_reactions = ["bind", "stagger", "collapse", "injury", "recovery"]
if state_labels != expected_states:
    raise SystemExit(f"state_labels mismatch: {state_labels}")
if reaction_labels != expected_reactions:
    raise SystemExit(f"reaction_labels mismatch: {reaction_labels}")

for artifact in [manifest, sequence, retargeting, report]:
    if not artifact.is_file() or artifact.stat().st_size == 0:
        raise SystemExit(f"missing or empty artifact: {artifact}")

summary = {
    "schema": "oathyard.animation_state_machine_wrapper.v1",
    "passed": True,
    "pre_replay": pre_replay.as_posix(),
    "post_replay": post_replay.as_posix(),
    "pre_replay_sha256": pre_sha,
    "post_replay_sha256": post_sha,
    "final_state_hash": pre_hash,
    "replay_sha256_pre_post_identical": pre_sha == post_sha,
    "truth_mutation": False,
    "presentation_only": True,
    "layer": "runtime_presentation",
    "state_count": len(state_labels),
    "reaction_count": len(reaction_labels),
    "transition_count": len(manifest_payload.get("transitions", [])),
    "animation_frame_count": manifest_payload.get("animation_frame_count"),
    "reaction_log_count": manifest_payload.get("reaction_log_count"),
    "evidence_dir": evidence.as_posix(),
}
(root / "animation_state_machine_wrapper.json").write_text(
    json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8"
)

wrapper_report = [
    "# OATHYARD Animation State Machine Wrapper Report",
    "",
    "Status: PASSED",
    f"- Scenario: `{summary['pre_replay'].split('/')[-1]}`",
    f"- Final state hash: `{pre_hash}`",
    f"- Replay SHA256 pre/post identical: `{pre_sha == post_sha}`",
    f"- Truth mutation: `false`",
    f"- Layer: `runtime_presentation`",
    f"- States: `{len(state_labels)}`",
    f"- Reactions: `{len(reaction_labels)}`",
    f"- Transitions: `{summary['transition_count']}`",
    f"- Animation frames: `{summary['animation_frame_count']}`",
    f"- Reaction log entries: `{summary['reaction_log_count']}`",
    "",
    "Artifacts:",
    f"- Manifest: `{manifest.as_posix()}`",
    f"- Sequence: `{sequence.as_posix()}`",
    f"- Retargeting: `{retargeting.as_posix()}`",
    f"- Report: `{report.as_posix()}`",
]
(root / "animation_state_machine_wrapper_report.md").write_text(
    "\n".join(wrapper_report) + "\n", encoding="utf-8"
)
PY

echo "animation state machine: $out"
