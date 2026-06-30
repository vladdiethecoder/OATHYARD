# 0003: Native Input Model

## Decision

OATHYARD uses a deterministic presentation-command input boundary for native UI and replay-facing workflows.

Keyboard, mouse zones, and gamepad-ready commands map to presentation commands such as screen navigation, loadout selection, planning timeline focus, commit/reveal, consequence readout, settings, and quit. These commands may generate or select committed duel inputs through documented UI logic, but they must not write authoritative gameplay truth directly.

No raw device event, analog axis, pointer coordinate, haptic response, audio cue, or UI state is allowed to decide contacts, costs, injuries, capability deltas, end conditions, or replay hashes. Truth remains owned by committed timeline inputs and deterministic simulation.

## Current Evidence

The current local input path includes:

- `input_map.json` with keyboard, mouse, gamepad-ready binding labels, glyphs, and remapping metadata;
- `input_profile.json` with a default XInput-style controller schema and screen reachability flags;
- local Steam Deck schema checklist;
- Linux `/dev/input/js*` joystick-interface smoke where available;
- runtime settings persistence for `hold_to_commit`, `toggle_guard`, and input profile id.

This is local schema, command-flow, and interface evidence. It is not physical controller ergonomics proof, Steam Deck hardware proof, platform compliance, or owner input acceptance.

## Command Boundary

Input commands are presentation commands until they create explicit, replayable game inputs:

- screen navigation commands switch UI state only;
- loadout and mode commands select deterministic content ids;
- planning commands author compact action labels and directional influence;
- commit/reveal submits planned timeline entries;
- replay commands select and verify saved replay artifacts;
- settings commands alter presentation/input/audio preferences only;
- no input command mutates gameplay truth after the replay hash boundary.

All command artifacts must keep `presentation_only: true`, `truth_mutation: false`, and `replay_hash_affects: false` where settings are persisted.

## Required Runtime Input Coverage

The production input target requires:

- keyboard and mouse access to every ship screen;
- gamepad access to every ship screen without terminal interaction;
- remapping for primary action, cancel, navigation, commit/reveal, guard mode, replay selection, and settings;
- hold/toggle alternatives for commit and guard behavior;
- glyphs driven by the active input profile;
- captions or visual equivalents for input-critical audio feedback;
- Steam Deck target resolutions and text scaling checked with the native UI;
- physical controller hardware smoke and Steam Deck hardware evidence before those claims become true;
- owner input acceptance recorded separately.

## Current Local Acceptance

The current local gate may pass when:

- gamepad-ready schema exists and all current screens are marked reachable;
- Linux joystick-interface smoke passes if a joystick-class device is present;
- physical controller hardware, Steam Deck hardware, and owner input acceptance remain false;
- runtime settings persistence roundtrips input preferences without changing replay hashes.

## Rejected Shortcuts

- Treating browser UI, screenshots, or generated text as native input proof.
- Claiming physical gamepad or Steam Deck compliance from schema artifacts alone.
- Letting analog input values, device polling order, haptics, or UI focus state enter authoritative truth.
- Using difficulty, input device, or accessibility settings to modify body stats, action costs, contacts, or injury outcomes.
- Committing store/platform controller claims without hardware or owner evidence.

## Verification

Primary commands:

```sh
./tools/input_map.sh artifacts/input/verify
./tools/gamepad_smoke.sh artifacts/gamepad/verify
./tools/runtime_settings.sh artifacts/settings/verify
./tools/input_target_audit.sh artifacts/input_target/verify
./tools/verify.sh
```
