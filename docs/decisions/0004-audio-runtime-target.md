# 0004: Audio Runtime Target

Status: Accepted for local verification target; production audio acceptance incomplete.

Date: 2026-06-29

## Context

OATHYARD now produces deterministic trace-derived audio events, procedural WAV output, captions, VFX event manifests, runtime mixer settings, loudness metrics, and a bounded local audio-device smoke through the host backend. The project still must not claim shipping audio completion, final loudness/platform acceptance, or owner audio acceptance from these local artifacts alone.

The audio path is presentation-only. It consumes committed truth/replay events after truth hashing and must never affect gameplay state, replay hashes, AI decisions, contact outcomes, action costs, or capability state.

## Decision

Keep the current production candidate audio boundary as:

- source of events: verified truth trace/replay output only;
- generated assets: repo-owned procedural integer WAV, captions, and VFX manifests;
- runtime mixer: deterministic integer gain, bus, and peak-limit artifact generation in the native executable;
- live-device smoke: bounded local playback command against available Linux audio backends;
- accepted local backend evidence: `pw-play`, `paplay`, or `aplay` command success only;
- shipping backend: not finalized;
- owner audio acceptance: not claimed;
- platform loudness/compliance acceptance: not claimed.

Do not add SDL audio, OpenAL Soft, FMOD, Wwise, miniaudio, PipeWire bindings, PulseAudio bindings, ALSA bindings, or any other dependency without a follow-up ADR that records source, license, package impact, fallback, deterministic boundary, and package smoke evidence.

## Production Acceptance Target

A future production audio target must prove all of the following before audio can be called complete:

- package-stable audio backend selected by ADR;
- native runtime can play UI, impact, armor/material, capability, ambience, and settings-preview audio from packaged assets;
- captions or visual equivalents exist for combat-critical audio;
- master/UI/impact/capability gains persist through settings and do not change replay hashes;
- loopback or equivalent capture verifies output path on a clean target;
- peak and loudness budgets are measured on representative content;
- owner audio acceptance is explicitly recorded;
- public-demo-ready and release-candidate-ready remain false until legal/store/owner gates also pass.

## Current Local Acceptance

The current local evidence is intentionally narrower:

- `./tools/audio_vfx_render.sh examples/duels/basic_oathyard.duel artifacts/audio_vfx/verify`
- `./tools/audio_mixer.sh examples/duels/basic_oathyard.duel artifacts/audio_mixer/verify`
- `./tools/audio_device_smoke.sh examples/duels/basic_oathyard.duel artifacts/audio_device/verify`
- `./tools/runtime_settings.sh artifacts/settings/verify`
- `./tools/audio_target_audit.sh artifacts/audio_target/verify`

The audit requires trace-derived audio, captions, deterministic mixer artifacts, local audio-device smoke, persisted audio gain settings, false truth mutation, false replay-hash effects, and false owner/platform acceptance claims.

## Truth Boundary

Audio code may read:

- `DuelResult`, trace contacts, replay hashes, and final state hashes after truth resolution;
- runtime presentation settings marked `presentation_only`;
- local host audio command status for smoke evidence.

Audio code must not write:

- fighter state;
- body/material/capability state;
- contact packets;
- action validity or frame costs;
- replay input/hash data;
- AI planning choices.

## Rejected Shortcuts

- No audio timing, random seed, playback result, latency measurement, or device status may affect truth.
- No copyrighted or scraped audio may enter production assets.
- No "works on my speaker" claim can substitute for owner acceptance or loopback/platform evidence.
- No web page, browser audio, or media preview can satisfy native product audio.
- No final loudness/platform certification is claimed from generated integer WAV metrics alone.

## Remaining Work

- Shipping backend ADR and package-stable runtime path.
- Loopback or equivalent capture on a clean target.
- Platform loudness and accessibility acceptance.
- Owner audio acceptance pack.
- Audio asset expansion beyond the current procedural trace-derived seed.
