#!/usr/bin/env bash
set -euo pipefail

out="${1:-artifacts/audio_target/verify}"
mixer_settings_json="${2:-artifacts/audio_mixer/verify/audio_mixer_settings.json}"
mixer_channels_json="${3:-artifacts/audio_mixer/verify/audio_mixer_channels.json}"
mixer_loudness_json="${4:-artifacts/audio_mixer/verify/audio_mixer_loudness.json}"
audio_device_json="${5:-artifacts/audio_device/verify/audio_device_smoke.json}"
audio_events_json="${6:-artifacts/audio_vfx/verify/audio_events.json}"
settings_json="${7:-artifacts/settings/verify/runtime_settings.saved.json}"
captions_srt="${8:-artifacts/audio_vfx/verify/captions.srt}"
adr="docs/decisions/0004-audio-runtime-target.md"

python3 - "$out" "$mixer_settings_json" "$mixer_channels_json" "$mixer_loudness_json" "$audio_device_json" "$audio_events_json" "$settings_json" "$captions_srt" "$adr" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

out = Path(sys.argv[1])
mixer_settings_path = Path(sys.argv[2])
mixer_channels_path = Path(sys.argv[3])
mixer_loudness_path = Path(sys.argv[4])
audio_device_path = Path(sys.argv[5])
audio_events_path = Path(sys.argv[6])
settings_path = Path(sys.argv[7])
captions_path = Path(sys.argv[8])
adr_path = Path(sys.argv[9])
out.mkdir(parents=True, exist_ok=True)


def read_json(path: Path) -> dict:
    if not path.is_file():
        raise FileNotFoundError(path)
    return json.loads(path.read_text(encoding="utf-8"))


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


checks = []


def check(check_id: str, passed: bool, detail: str) -> None:
    checks.append({"id": check_id, "passed": bool(passed), "detail": detail})


try:
    mixer_settings = read_json(mixer_settings_path)
    mixer_channels = read_json(mixer_channels_path)
    mixer_loudness = read_json(mixer_loudness_path)
    audio_device = read_json(audio_device_path)
    audio_events = read_json(audio_events_path)
    runtime_settings = read_json(settings_path)
    captions_text = captions_path.read_text(encoding="utf-8")
    adr_text = adr_path.read_text(encoding="utf-8")
except Exception as error:  # noqa: BLE001 - audit artifact records exact failure.
    mixer_settings = {}
    mixer_channels = {}
    mixer_loudness = {}
    audio_device = {}
    audio_events = {}
    runtime_settings = {}
    captions_text = ""
    adr_text = ""
    check("inputs_readable", False, str(error))
else:
    check("inputs_readable", True, "ADR and audio evidence artifacts loaded")

check(
    "adr_present",
    adr_path.is_file() and "# 0004: Audio Runtime Target" in adr_text,
    str(adr_path),
)
for heading in [
    "## Decision",
    "## Production Acceptance Target",
    "## Current Local Acceptance",
    "## Truth Boundary",
    "## Rejected Shortcuts",
]:
    check(f"adr_heading_{heading[3:].lower().replace(' ', '_')}", heading in adr_text, heading)

check("mixer_schema", mixer_settings.get("schema") == "oathyard.audio_mixer.v1", mixer_settings_path.as_posix())
check("mixer_source_trace", mixer_settings.get("source") == "trace-derived-only", mixer_settings.get("source", ""))
check("mixer_presentation_only", mixer_settings.get("presentation_only") is True, "mixer")
check("mixer_truth_mutation_false", mixer_settings.get("truth_mutation") is False, "mixer")
check("mixer_integrated_claimed", mixer_settings.get("integrated_runtime_mixer_claimed") is True, "mixer")
check("mixer_owner_not_claimed", mixer_settings.get("human_audible_acceptance_claimed") is False, "mixer")
check("mixer_captions_enabled", mixer_settings.get("captions_enabled") is True, "mixer")
check("mixer_visual_equivalents", mixer_settings.get("visual_equivalents_enabled") is True, "mixer")
check("mixer_gains_integer", all(isinstance(value, int) for value in [
    mixer_settings.get("master_gain_permille"),
    mixer_settings.get("buses", {}).get("ui_gain_permille"),
    mixer_settings.get("buses", {}).get("impact_gain_permille"),
    mixer_settings.get("buses", {}).get("capability_gain_permille"),
    mixer_settings.get("peak_limit_permille"),
]), "permille gains")

check("channels_schema", mixer_channels.get("schema") == "oathyard.audio_mixer_channels.v1", mixer_channels_path.as_posix())
channel_events = mixer_channels.get("events", [])
channel_buses = {entry.get("bus") for entry in channel_events}
check("channels_have_events", len(channel_events) >= 1, str(len(channel_events)))
check("channels_have_ui_bus", "ui" in channel_buses, ",".join(sorted(str(item) for item in channel_buses)))
check("channels_have_impact_or_capability_bus", bool({"impact", "capability"} & channel_buses), ",".join(sorted(str(item) for item in channel_buses)))
check("channels_truth_mutation_false", mixer_channels.get("truth_mutation") is False, "channels")

check("loudness_schema", mixer_loudness.get("schema") == "oathyard.audio_mixer_loudness.v1", mixer_loudness_path.as_posix())
check("loudness_sample_rate", mixer_loudness.get("sample_rate_hz") == 22050, str(mixer_loudness.get("sample_rate_hz")))
check("loudness_channels", mixer_loudness.get("channels") == 1, str(mixer_loudness.get("channels")))
check("loudness_event_count", int(mixer_loudness.get("event_count", 0)) >= 1, str(mixer_loudness.get("event_count")))
check("loudness_peak_below_limit", int(mixer_loudness.get("peak_permille", 1001)) <= int(mixer_settings.get("peak_limit_permille", 0)), str(mixer_loudness.get("peak_permille")))
check("loudness_owner_not_claimed", mixer_loudness.get("human_audible_acceptance_claimed") is False, "loudness")

check("device_schema", audio_device.get("schema") == "oathyard.audio_device_smoke.v1", audio_device_path.as_posix())
check("device_status_passed", audio_device.get("status") == "PASSED_LIVE_AUDIO_DEVICE_SMOKE", audio_device.get("status", ""))
check("device_backend_allowed", audio_device.get("selected_backend") in {"pw-play", "paplay", "aplay"}, audio_device.get("selected_backend", ""))
check("device_trace_derived", audio_device.get("trace_derived_audio") is True, "device")
check("device_presentation_only", audio_device.get("presentation_only") is True, "device")
check("device_truth_mutation_false", audio_device.get("truth_mutation") is False, "device")
check("device_playback_claimed", audio_device.get("live_audio_device_playback_smoke_claimed") is True, "device")
check("device_owner_not_claimed", audio_device.get("human_audible_acceptance_claimed") is False, "device")
check("device_attempt_success", any(entry.get("success") is True for entry in audio_device.get("attempts", [])), "attempts")

check("events_schema", audio_events.get("schema") == "oathyard.audio_events.v1", audio_events_path.as_posix())
events = audio_events.get("events", [])
check("events_trace_derived", audio_events.get("source") == "trace-derived-only", audio_events.get("source", ""))
check("events_presentation_only", audio_events.get("presentation_only") is True, "events")
check("events_have_captions", all(entry.get("caption") for entry in events), str(len(events)))
check("events_have_contact_audio", any(entry.get("sound") != "ui_commit_reveal" for entry in events), "contact audio")

check("captions_file_present", captions_path.is_file() and captions_path.stat().st_size > 0, captions_path.as_posix())
check("captions_srt_timing", "-->" in captions_text, "srt timing")
check("captions_include_commit", "commit reveal" in captions_text, "commit reveal caption")

check("settings_schema", runtime_settings.get("schema") == "oathyard.runtime_settings.v1", settings_path.as_posix())
check("settings_presentation_only", runtime_settings.get("presentation_only") is True, "settings")
check("settings_truth_mutation_false", runtime_settings.get("truth_mutation") is False, "settings")
check("settings_replay_hash_unaffected", runtime_settings.get("replay_hash_affects") is False, "settings")
check("settings_audio_gains_persisted", all(isinstance(runtime_settings.get(key), int) for key in [
    "master_gain_permille",
    "ui_gain_permille",
    "impact_gain_permille",
    "capability_gain_permille",
]), "settings gains")
check("settings_captions_enabled", runtime_settings.get("captions_enabled") is True, "settings")

public_false = all(value is False for value in [
    runtime_settings.get("public_demo_ready"),
    mixer_settings.get("public_demo_ready", False),
    audio_device.get("public_demo_ready", False),
])
release_false = all(value is False for value in [
    runtime_settings.get("release_candidate_ready"),
    mixer_settings.get("release_candidate_ready", False),
    audio_device.get("release_candidate_ready", False),
])
check("public_demo_ready_false", public_false, "false")
check("release_candidate_ready_false", release_false, "false")

passed = all(item["passed"] for item in checks)
artifact_hashes = {
    "adr": sha256(adr_path) if adr_path.is_file() else "",
    "audio_mixer_settings": sha256(mixer_settings_path) if mixer_settings_path.is_file() else "",
    "audio_mixer_channels": sha256(mixer_channels_path) if mixer_channels_path.is_file() else "",
    "audio_mixer_loudness": sha256(mixer_loudness_path) if mixer_loudness_path.is_file() else "",
    "audio_device_smoke": sha256(audio_device_path) if audio_device_path.is_file() else "",
    "audio_events": sha256(audio_events_path) if audio_events_path.is_file() else "",
    "runtime_settings": sha256(settings_path) if settings_path.is_file() else "",
    "captions": sha256(captions_path) if captions_path.is_file() else "",
}

report = {
    "schema": "oathyard.audio_runtime_target_audit.v1",
    "product": "OATHYARD",
    "adr": adr_path.as_posix(),
    "event_source": "verified_trace_replay_after_hash",
    "current_local_backends": ["pw-play", "paplay", "aplay"],
    "selected_local_backend": audio_device.get("selected_backend", "none"),
    "runtime_mixer_artifact_claimed": mixer_settings.get("integrated_runtime_mixer_claimed") is True,
    "live_audio_device_smoke_claimed": audio_device.get("live_audio_device_playback_smoke_claimed") is True,
    "captions_present": captions_path.is_file() and captions_path.stat().st_size > 0,
    "presentation_only": True,
    "truth_mutation": False,
    "replay_hash_affects": False,
    "shipping_backend_finalized": False,
    "platform_loudness_acceptance_claimed": False,
    "owner_audio_acceptance_claimed": False,
    "public_demo_ready": False,
    "release_candidate_ready": False,
    "checks": checks,
    "artifact_hashes": artifact_hashes,
    "next_required_artifacts": [
        "shipping backend ADR and package-stable runtime backend",
        "loopback or equivalent clean-target audio capture",
        "expanded original source-backed audio asset set",
        "platform loudness/accessibility review",
        "owner audio acceptance pack",
    ],
    "passed": passed,
}

(out / "audio_runtime_target.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

lines = [
    "# OATHYARD Audio Runtime Target Audit",
    "",
    f"Status: {'PASSED' if passed else 'FAILED'}",
    "",
    f"- ADR: `{adr_path.as_posix()}`",
    "- Event source: `verified_trace_replay_after_hash`",
    f"- Selected local backend: `{report['selected_local_backend']}`",
    f"- Runtime mixer artifact claimed: `{str(report['runtime_mixer_artifact_claimed']).lower()}`",
    f"- Live audio-device smoke claimed: `{str(report['live_audio_device_smoke_claimed']).lower()}`",
    f"- Captions present: `{str(report['captions_present']).lower()}`",
    "- Presentation only: `true`",
    "- Truth mutation: `none`",
    "- Replay hash affects: `false`",
    "- Shipping backend finalized: `false`",
    "- Platform loudness acceptance claimed: `false`",
    "- Owner audio acceptance claimed: `false`",
    "- Public demo ready: `false`",
    "- Release candidate ready: `false`",
    "",
    "## Evidence Hashes",
    "",
]
for name, value in artifact_hashes.items():
    lines.append(f"- `{name}`: `{value}`")

lines.extend(["", "## Checks", ""])
for item in checks:
    state = "pass" if item["passed"] else "fail"
    lines.append(f"- `{item['id']}`: `{state}` - {item['detail']}")

lines.extend(["", "## Remaining Audio Work", ""])
for item in report["next_required_artifacts"]:
    lines.append(f"- {item}")

(out / "audio_runtime_target_report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")

if not passed:
    print("audio runtime target audit failed", file=sys.stderr)
    sys.exit(1)
PY
