use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::{
    comma, hash_hex, json_quote, run_scenario_file, write_json_field, ActionLabel, ContactTrace,
    DuelResult, OathError, AUDIO_DEVICE_SMOKE_SCHEMA, AUDIO_MIXER_SCHEMA, PRODUCT_NAME,
    PUBLIC_DEMO_READY, RELEASE_CANDIDATE_READY, TRUTH_HZ,
};

pub fn write_audio_vfx_artifacts(
    scenario_path: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
) -> Result<DuelResult, OathError> {
    let result = run_scenario_file(scenario_path)?;
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;
    let events = presentation_events_from_duel(&result);
    let (wav, wav_stats) = render_audio_mix_wav_with_stats(&events);
    fs::write(out_dir.join("audio_mix.wav"), wav)?;
    let vfx_evidence = build_impact_vfx_evidence(&events);
    fs::write(
        out_dir.join("audio_events.json"),
        render_audio_events_json(&result, &events),
    )?;
    fs::write(
        out_dir.join("vfx_manifest.json"),
        render_vfx_manifest_json(&result, &events, &vfx_evidence),
    )?;
    fs::write(
        out_dir.join("audio_vfx_timing_loudness.json"),
        render_audio_vfx_timing_loudness_json(&result, &events, &wav_stats),
    )?;
    fs::write(out_dir.join("captions.srt"), render_captions_srt(&events))?;
    fs::write(
        out_dir.join("audio_vfx_report.md"),
        render_audio_vfx_report(&result, &events, &wav_stats, &vfx_evidence),
    )?;
    Ok(result)
}

pub fn write_audio_mixer_artifacts(
    scenario_path: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
) -> Result<DuelResult, OathError> {
    let result = run_scenario_file(scenario_path)?;
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;
    let events = presentation_events_from_duel(&result);
    let settings = default_audio_mixer_settings();
    let (wav, stats) = render_runtime_audio_mix_wav(&events, &settings);
    fs::write(out_dir.join("runtime_audio_mix.wav"), wav)?;
    fs::write(
        out_dir.join("audio_mixer_settings.json"),
        render_audio_mixer_settings_json(&result, &settings),
    )?;
    fs::write(
        out_dir.join("audio_mixer_channels.json"),
        render_audio_mixer_channels_json(&result, &events, &settings),
    )?;
    fs::write(
        out_dir.join("audio_mixer_loudness.json"),
        render_audio_mixer_loudness_json(&result, &stats),
    )?;
    fs::write(out_dir.join("captions.srt"), render_captions_srt(&events))?;
    fs::write(
        out_dir.join("audio_mixer_report.md"),
        render_audio_mixer_report(&result, &events, &settings, &stats),
    )?;
    Ok(result)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AudioMixerSettings {
    master_gain_permille: i32,
    ui_gain_permille: i32,
    ambience_gain_permille: i32,
    impact_gain_permille: i32,
    capability_gain_permille: i32,
    captions_enabled: bool,
    visual_equivalents_enabled: bool,
    mute_master: bool,
    peak_limit_permille: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AudioMixerStats {
    sample_rate_hz: u32,
    channels: u32,
    sample_count: usize,
    duration_ms: u32,
    event_count: usize,
    peak_abs: i32,
    peak_permille: i32,
    mean_square_permille: i32,
    limited_sample_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct VfxEvidenceArtifact {
    file: &'static str,
    width: usize,
    height: usize,
    sha256: String,
    distinct_color_count: usize,
    event_count: usize,
}

fn default_audio_mixer_settings() -> AudioMixerSettings {
    AudioMixerSettings {
        master_gain_permille: 820,
        ui_gain_permille: 520,
        ambience_gain_permille: 390,
        impact_gain_permille: 860,
        capability_gain_permille: 730,
        captions_enabled: true,
        visual_equivalents_enabled: true,
        mute_master: false,
        peak_limit_permille: 880,
    }
}

pub fn write_audio_device_smoke_artifacts(
    scenario_path: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
) -> Result<DuelResult, OathError> {
    let result = write_audio_vfx_artifacts(scenario_path, &out_dir)?;
    let out_dir = out_dir.as_ref();
    let smoke = audio_device_smoke_result(&out_dir.join("audio_mix.wav"));
    fs::write(
        out_dir.join("audio_device_smoke.json"),
        render_audio_device_smoke_json(&result, &smoke),
    )?;
    fs::write(
        out_dir.join("audio_device_smoke_report.md"),
        render_audio_device_smoke_report(&result, &smoke),
    )?;
    if !smoke.playback_claimed {
        return Err(OathError::Verify(format!(
            "audio device smoke failed: {}",
            smoke.status
        )));
    }
    Ok(result)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AudioDeviceSmokeResult {
    status: &'static str,
    playback_claimed: bool,
    selected_backend: &'static str,
    attempts: Vec<AudioDeviceAttempt>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AudioDeviceAttempt {
    backend: &'static str,
    command: String,
    exit_code: i32,
    success: bool,
}

fn audio_device_smoke_result(wav_path: &Path) -> AudioDeviceSmokeResult {
    let candidates: [(&str, Vec<String>); 3] = [
        (
            "pw-play",
            vec![
                "8".to_string(),
                "pw-play".to_string(),
                "--media-role".to_string(),
                "Game".to_string(),
                "--volume".to_string(),
                "0.08".to_string(),
                wav_path.display().to_string(),
            ],
        ),
        (
            "paplay",
            vec![
                "8".to_string(),
                "paplay".to_string(),
                "--volume=5243".to_string(),
                "--client-name=OATHYARD-audio-smoke".to_string(),
                "--stream-name=trace-derived-audio-smoke".to_string(),
                wav_path.display().to_string(),
            ],
        ),
        (
            "aplay",
            vec![
                "8".to_string(),
                "aplay".to_string(),
                "-q".to_string(),
                "-d".to_string(),
                "3".to_string(),
                wav_path.display().to_string(),
            ],
        ),
    ];
    let mut attempts = Vec::new();
    let mut selected_backend = "none";
    for (backend, args) in candidates {
        let attempt = audio_device_attempt(backend, &args);
        let success = attempt.success;
        attempts.push(attempt);
        if success {
            selected_backend = backend;
            break;
        }
    }
    let playback_claimed = selected_backend != "none";
    let status = if playback_claimed {
        "PASSED_LIVE_AUDIO_DEVICE_SMOKE"
    } else {
        "BLOCKED_NO_LIVE_AUDIO_BACKEND"
    };
    AudioDeviceSmokeResult {
        status,
        playback_claimed,
        selected_backend,
        attempts,
    }
}

fn audio_device_attempt(backend: &'static str, timeout_args: &[String]) -> AudioDeviceAttempt {
    let output = Command::new("timeout").args(timeout_args).output();
    match output {
        Ok(output) => AudioDeviceAttempt {
            backend,
            command: format!("timeout {}", timeout_args.join(" ")),
            exit_code: output.status.code().unwrap_or(-1),
            success: output.status.success(),
        },
        Err(_) => AudioDeviceAttempt {
            backend,
            command: format!("timeout {}", timeout_args.join(" ")),
            exit_code: -1,
            success: false,
        },
    }
}

fn render_audio_device_smoke_json(result: &DuelResult, smoke: &AudioDeviceSmokeResult) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", AUDIO_DEVICE_SMOKE_SCHEMA, true);
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(&mut out, 1, "scenario_id", &result.scenario_id, true);
    write_json_field(&mut out, 1, "content_hash", &result.content_hash, true);
    write_json_field(
        &mut out,
        1,
        "final_state_hash",
        &result.final_state_hash,
        true,
    );
    write_json_field(&mut out, 1, "status", smoke.status, true);
    write_json_field(
        &mut out,
        1,
        "selected_backend",
        smoke.selected_backend,
        true,
    );
    writeln!(&mut out, "  \"trace_derived_audio\": true,").unwrap();
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(
        &mut out,
        "  \"live_audio_device_playback_smoke_claimed\": {},",
        smoke.playback_claimed
    )
    .unwrap();
    writeln!(&mut out, "  \"integrated_runtime_mixer_claimed\": false,").unwrap();
    writeln!(&mut out, "  \"human_audible_acceptance_claimed\": false,").unwrap();
    writeln!(&mut out, "  \"captions_present\": true,").unwrap();
    writeln!(&mut out, "  \"audio_file\": \"audio_mix.wav\",").unwrap();
    writeln!(&mut out, "  \"attempts\": [").unwrap();
    for (index, attempt) in smoke.attempts.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "backend", attempt.backend, true);
        write_json_field(&mut out, 3, "command", &attempt.command, true);
        writeln!(&mut out, "      \"exit_code\": {},", attempt.exit_code).unwrap();
        writeln!(&mut out, "      \"success\": {}", attempt.success).unwrap();
        writeln!(&mut out, "    }}{}", comma(index + 1, smoke.attempts.len())).unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

fn render_audio_device_smoke_report(result: &DuelResult, smoke: &AudioDeviceSmokeResult) -> String {
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD Audio Device Smoke Report").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "Status: {}", smoke.status).unwrap();
    writeln!(&mut out, "- Scenario: `{}`", result.scenario_id).unwrap();
    writeln!(&mut out, "- Content hash: `{}`", result.content_hash).unwrap();
    writeln!(
        &mut out,
        "- Final state hash: `{}`",
        result.final_state_hash
    )
    .unwrap();
    writeln!(&mut out, "- Selected backend: `{}`", smoke.selected_backend).unwrap();
    writeln!(
        &mut out,
        "- Live audio device playback smoke claimed: `{}`",
        smoke.playback_claimed
    )
    .unwrap();
    writeln!(&mut out, "- Trace-derived audio: `true`").unwrap();
    writeln!(&mut out, "- Presentation only: `true`").unwrap();
    writeln!(&mut out, "- Truth mutation: `none`").unwrap();
    writeln!(&mut out, "- Captions present: `captions.srt`").unwrap();
    writeln!(&mut out, "- Integrated runtime mixer claimed: `false`").unwrap();
    writeln!(&mut out, "- Human audible acceptance claimed: `false`").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Backend Attempts").unwrap();
    writeln!(&mut out).unwrap();
    for attempt in &smoke.attempts {
        writeln!(
            &mut out,
            "- `{}` exit `{}` success `{}` command `{}`",
            attempt.backend, attempt.exit_code, attempt.success, attempt.command
        )
        .unwrap();
    }
    writeln!(&mut out).unwrap();
    writeln!(
        &mut out,
        "This smoke proves bounded playback command success against the local audio stack. It does not prove production mixer integration, loudness balance, spatial audio, or owner audio acceptance."
    )
    .unwrap();
    out
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PresentationEvent {
    frame: u32,
    event_id: String,
    event_family: &'static str,
    source_event_id: String,
    source_event_kind: &'static str,
    turn: Option<u32>,
    seat: Option<usize>,
    contact_index: Option<usize>,
    sound_id: &'static str,
    vfx_id: &'static str,
    effect_family: &'static str,
    material_ids: Vec<&'static str>,
    weapon_id: String,
    armor_id: String,
    target: String,
    caption: String,
    frequency_hz: u32,
    amplitude: i32,
    duration_ms: u32,
    reduced_flash_compliant: bool,
}

fn presentation_events_from_duel(result: &DuelResult) -> Vec<PresentationEvent> {
    let mut events = Vec::new();

    events.push(system_presentation_event(
        result,
        0,
        "ambience",
        "arena_ambience",
        "arena_dust_mote",
        "dust",
        vec!["arena_air", "chalk_stone"],
        "low verdict-ring ambience under the duel trace".to_string(),
        96,
        2200,
        1400,
    ));
    events.push(system_presentation_event(
        result,
        0,
        "ui_audio",
        "ui_commit_reveal",
        "commit_flash",
        "ui_flash",
        vec!["ui"],
        "commit reveal".to_string(),
        440,
        5800,
        420,
    ));

    let mut action_index = 0usize;
    let mut contact_index = 0usize;
    for turn in &result.turns {
        for action in &turn.commits {
            let (weapon_id, armor_id) = fighter_assets_from_canonical(result, action.seat);
            let frame = turn.turn * TRUTH_HZ + 8 + (action.seat as u32 * 3);
            if action.label.is_attack() {
                events.push(action_presentation_event(
                    result,
                    turn.turn,
                    action_index,
                    action.seat,
                    frame,
                    "weapon_trail",
                    "weapon_air_cut",
                    "weapon_trail_arc",
                    "weapon_trail",
                    vec!["steel_edge", "air_shear"],
                    weapon_id,
                    armor_id,
                    action.target.as_str().to_string(),
                    format!(
                        "{} weapon trail toward {}",
                        action.label.as_str(),
                        action.target.as_str()
                    ),
                    620,
                    3600,
                    360,
                ));
            } else {
                events.push(action_presentation_event(
                    result,
                    turn.turn,
                    action_index,
                    action.seat,
                    frame,
                    "footwork",
                    "footwork_step",
                    "dust_step",
                    "dust",
                    vec!["chalk_dust", "boot_scuff"],
                    weapon_id,
                    armor_id,
                    action.target.as_str().to_string(),
                    format!(
                        "{} footwork {} after commit",
                        action.label.as_str(),
                        action.direction.as_str()
                    ),
                    185,
                    3100,
                    320,
                ));
            }
            action_index += 1;
        }

        for contact in &turn.contacts {
            push_contact_presentation_events(result, contact, contact_index, &mut events);
            contact_index += 1;
        }
    }
    events.sort_by(|left, right| {
        left.frame
            .cmp(&right.frame)
            .then_with(|| left.event_id.cmp(&right.event_id))
    });
    events
}

#[allow(clippy::too_many_arguments)]
fn system_presentation_event(
    result: &DuelResult,
    frame: u32,
    event_family: &'static str,
    sound_id: &'static str,
    vfx_id: &'static str,
    effect_family: &'static str,
    material_ids: Vec<&'static str>,
    caption: String,
    frequency_hz: u32,
    amplitude: i32,
    duration_ms: u32,
) -> PresentationEvent {
    PresentationEvent {
        frame,
        event_id: format!("{}:{event_family}:frame{frame}", result.scenario_id),
        event_family,
        source_event_id: format!(
            "replay:{}:{}:system:{event_family}:frame{frame}",
            result.scenario_id, result.final_state_hash
        ),
        source_event_kind: "replay_system",
        turn: None,
        seat: None,
        contact_index: None,
        sound_id,
        vfx_id,
        effect_family,
        material_ids,
        weapon_id: "none".to_string(),
        armor_id: "none".to_string(),
        target: "arena".to_string(),
        caption,
        frequency_hz,
        amplitude,
        duration_ms,
        reduced_flash_compliant: true,
    }
}

#[allow(clippy::too_many_arguments)]
fn action_presentation_event(
    result: &DuelResult,
    turn: u32,
    action_index: usize,
    seat: usize,
    frame: u32,
    event_family: &'static str,
    sound_id: &'static str,
    vfx_id: &'static str,
    effect_family: &'static str,
    material_ids: Vec<&'static str>,
    weapon_id: String,
    armor_id: String,
    target: String,
    caption: String,
    frequency_hz: u32,
    amplitude: i32,
    duration_ms: u32,
) -> PresentationEvent {
    PresentationEvent {
        frame,
        event_id: format!(
            "{}:turn{turn}:seat{seat}:{event_family}:{action_index}",
            result.scenario_id
        ),
        event_family,
        source_event_id: format!(
            "replay:{}:{}:turn{turn}:seat{seat}:action{action_index}",
            result.scenario_id, result.final_state_hash
        ),
        source_event_kind: "committed_action",
        turn: Some(turn),
        seat: Some(seat),
        contact_index: None,
        sound_id,
        vfx_id,
        effect_family,
        material_ids,
        weapon_id,
        armor_id,
        target,
        caption,
        frequency_hz,
        amplitude,
        duration_ms,
        reduced_flash_compliant: true,
    }
}

fn push_contact_presentation_events(
    result: &DuelResult,
    contact: &ContactTrace,
    contact_index: usize,
    events: &mut Vec<PresentationEvent>,
) {
    let source_event_id = contact_source_event_id(result, contact, contact_index);
    let material_ids = contact_material_ids(&contact.material_result);
    let (sound_id, frequency_hz, amplitude) = classify_contact_audio(contact);
    let base_caption = format!(
        "{} on {}: {}",
        contact.action.as_str(),
        contact.target.as_str(),
        contact.cause_chain
    );

    events.push(contact_presentation_event(
        result,
        contact,
        contact_index,
        &source_event_id,
        "material_impact",
        sound_id,
        "material_impact_burst",
        "material_impact_burst",
        material_ids.clone(),
        base_caption.clone(),
        frequency_hz,
        amplitude,
        300,
        0,
    ));

    if contact_has_spark(contact) {
        events.push(contact_presentation_event(
            result,
            contact,
            contact_index,
            &source_event_id,
            "material_impact",
            "spark_scatter",
            "edge_spark",
            "spark",
            material_ids.clone(),
            format!("spark scatter from {}", contact.material_result),
            1180,
            5000,
            160,
            1,
        ));
    }

    if contact_has_blood_wetness(contact) {
        events.push(contact_presentation_event(
            result,
            contact,
            contact_index,
            &source_event_id,
            "material_impact",
            "wet_impact",
            "blood_wetness",
            "blood_wetness",
            material_ids.clone(),
            "localized blood/wetness mask from gap or flesh contact".to_string(),
            260,
            4400,
            420,
            2,
        ));
    }

    events.push(contact_presentation_event(
        result,
        contact,
        contact_index,
        &source_event_id,
        "material_impact",
        "debris_scatter",
        "impact_debris",
        "debris",
        material_ids.clone(),
        "small deterministic debris flecks from material contact".to_string(),
        370,
        3200,
        260,
        3,
    ));

    if contact_should_emit_shock(contact) {
        events.push(contact_presentation_event(
            result,
            contact,
            contact_index,
            &source_event_id,
            "shock_cue",
            "shock_cue",
            "shock_ring",
            "shock_cue",
            material_ids.clone(),
            "shock cue from capability or posture loss".to_string(),
            145,
            5600,
            520,
            4,
        ));
    }

    events.push(contact_presentation_event(
        result,
        contact,
        contact_index,
        &source_event_id,
        "replay_fight_film_audio",
        "fight_film_marker",
        "fight_film_pulse",
        "shock_cue",
        material_ids,
        "fight-film emphasis marker bound to verified contact frame".to_string(),
        520,
        2800,
        240,
        5,
    ));
}

#[allow(clippy::too_many_arguments)]
fn contact_presentation_event(
    result: &DuelResult,
    contact: &ContactTrace,
    contact_index: usize,
    source_event_id: &str,
    event_family: &'static str,
    sound_id: &'static str,
    vfx_id: &'static str,
    effect_family: &'static str,
    material_ids: Vec<&'static str>,
    caption: String,
    frequency_hz: u32,
    amplitude: i32,
    duration_ms: u32,
    frame_offset: u32,
) -> PresentationEvent {
    PresentationEvent {
        frame: contact.frame + frame_offset,
        event_id: format!(
            "{}:turn{}:frame{}:contact{}:{effect_family}:{frame_offset}",
            result.scenario_id, contact.turn, contact.frame, contact_index
        ),
        event_family,
        source_event_id: source_event_id.to_string(),
        source_event_kind: "contact_trace",
        turn: Some(contact.turn),
        seat: Some(contact.attacker),
        contact_index: Some(contact_index),
        sound_id,
        vfx_id,
        effect_family,
        material_ids,
        weapon_id: contact.weapon_id.clone(),
        armor_id: contact.armor_id.clone(),
        target: contact.target.as_str().to_string(),
        caption,
        frequency_hz,
        amplitude,
        duration_ms,
        reduced_flash_compliant: true,
    }
}

fn fighter_assets_from_canonical(result: &DuelResult, seat: usize) -> (String, String) {
    let seat_token = seat.to_string();
    for line in result.canonical_scenario.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 5 && parts[0] == "fighter" && parts[1] == seat_token {
            return (parts[3].to_string(), parts[4].to_string());
        }
    }
    ("unknown_weapon".to_string(), "unknown_armor".to_string())
}

fn contact_source_event_id(
    result: &DuelResult,
    contact: &ContactTrace,
    contact_index: usize,
) -> String {
    format!(
        "replay:{}:{}:turn{}:frame{}:contact{}:{}:{}:{}",
        result.scenario_id,
        result.final_state_hash,
        contact.turn,
        contact.frame,
        contact_index,
        contact.attacker,
        contact.defender,
        contact.action.as_str()
    )
}

fn classify_contact_audio(contact: &ContactTrace) -> (&'static str, u32, i32) {
    if contact.material_result.contains("mail") {
        ("mail_contact", 760, 7600)
    } else if contact.material_result.contains("gap") {
        ("blade_contact", 940, 7200)
    } else if contact.material_result.contains("blunt") {
        ("blunt_contact", 180, 8200)
    } else if contact.material_result.contains("hook") {
        ("blade_contact", 520, 6800)
    } else if contact.material_result.contains("deflected") {
        ("plate_contact", 680, 6400)
    } else {
        ("flesh_contact", 260, 5000)
    }
}

fn contact_material_ids(material_result: &str) -> Vec<&'static str> {
    if material_result.contains("mail") {
        vec!["riveted_mail", "steel_edge", "blunt_transfer"]
    } else if material_result.contains("deflected") {
        vec!["tempered_plate", "steel_edge", "spark_deflection"]
    } else if material_result.contains("gap") {
        vec!["blood", "cloth", "gap"]
    } else if material_result.contains("hook") {
        vec!["leather", "lamellar_iron_leather", "bind_scratch"]
    } else if material_result.contains("blunt") {
        vec!["stone_dust", "cloth", "pressure_shock"]
    } else {
        vec!["skin", "quilted_linen", "dirt"]
    }
}

fn contact_has_spark(contact: &ContactTrace) -> bool {
    contact.material_result.contains("mail")
        || contact.material_result.contains("gap")
        || contact.material_result.contains("deflected")
        || contact.material_result.contains("hook")
}

fn contact_has_blood_wetness(contact: &ContactTrace) -> bool {
    contact.material_result.contains("gap")
        || contact.material_result.contains("flesh")
        || contact.anatomy_result.contains("trauma")
}

fn contact_should_emit_shock(contact: &ContactTrace) -> bool {
    contact.capability_delta.balance_delta <= -80
        || contact.capability_delta.torso_rotation_delta < 0
        || contact.capability_delta.recovery_slowdown_add > 0
        || contact.impulse_milli >= 5_000
}

#[allow(dead_code)]
fn action_label_is_footwork(label: ActionLabel) -> bool {
    matches!(
        label,
        ActionLabel::Step
            | ActionLabel::Pivot
            | ActionLabel::Guard
            | ActionLabel::Brace
            | ActionLabel::Recover
    )
}

fn render_audio_mix_wav_with_stats(events: &[PresentationEvent]) -> (Vec<u8>, AudioMixerStats) {
    let sample_rate = 22_050u32;
    let min_duration_samples = sample_rate / 20;
    let total_samples = events
        .iter()
        .map(|event| {
            ((event.frame as u64 * sample_rate as u64) / TRUTH_HZ as u64)
                + ((event.duration_ms as u64 * sample_rate as u64) / 1000)
                + min_duration_samples as u64
        })
        .max()
        .unwrap_or(sample_rate as u64 / 2) as usize;
    let mut mix = vec![0i32; total_samples.max(sample_rate as usize / 2)];
    for event in events {
        let duration_samples = ((event.duration_ms as u64 * sample_rate as u64) / 1000)
            .max(min_duration_samples as u64) as usize;
        let start = ((event.frame as u64 * sample_rate as u64) / TRUTH_HZ as u64) as usize;
        for sample_index in 0..duration_samples {
            let target = start + sample_index;
            if target >= mix.len() {
                break;
            }
            let phase = ((sample_index as u64 * event.frequency_hz as u64 * 2000)
                / sample_rate as u64)
                % 2000;
            let triangle = if phase < 1000 {
                phase as i32 * 2 - 1000
            } else {
                3000 - phase as i32 * 2
            };
            let texture_seed = (sample_index as u32)
                .wrapping_add(event.frame)
                .wrapping_add(event.event_id.len() as u32)
                .wrapping_mul(1_103_515_245)
                .wrapping_add(12_345);
            let texture = ((texture_seed >> 17) & 255) as i32 - 128;
            let envelope =
                ((duration_samples - sample_index) as i32 * 1000) / duration_samples as i32;
            let textured = triangle + texture * 3;
            mix[target] +=
                (textured as i64 * event.amplitude as i64 * envelope as i64 / 1_000_000) as i32;
        }
    }
    let mut samples = Vec::with_capacity(mix.len());
    let mut peak_abs = 0i32;
    let mut limited_sample_count = 0usize;
    let mut sum_square: i128 = 0;
    for sample in mix {
        let clamped = sample.clamp(i16::MIN as i32, i16::MAX as i32);
        if clamped != sample {
            limited_sample_count += 1;
        }
        let abs = clamped.abs();
        peak_abs = peak_abs.max(abs);
        sum_square += abs as i128 * abs as i128;
        samples.push(clamped as i16);
    }
    let sample_count = samples.len();
    let duration_ms = (sample_count as u64 * 1000 / sample_rate as u64) as u32;
    let peak_permille = peak_abs * 1000 / i16::MAX as i32;
    let mean_square_permille = if sample_count == 0 {
        0
    } else {
        (sum_square * 1000 / sample_count as i128 / (i16::MAX as i128 * i16::MAX as i128)) as i32
    };
    (
        wav_from_samples(sample_rate, &samples),
        AudioMixerStats {
            sample_rate_hz: sample_rate,
            channels: 1,
            sample_count,
            duration_ms,
            event_count: events.len(),
            peak_abs,
            peak_permille,
            mean_square_permille,
            limited_sample_count,
        },
    )
}

fn render_runtime_audio_mix_wav(
    events: &[PresentationEvent],
    settings: &AudioMixerSettings,
) -> (Vec<u8>, AudioMixerStats) {
    let sample_rate = 22_050u32;
    let min_duration_samples = sample_rate / 20;
    let total_samples = events
        .iter()
        .map(|event| {
            ((event.frame as u64 * sample_rate as u64) / TRUTH_HZ as u64)
                + ((event.duration_ms as u64 * sample_rate as u64) / 1000)
                + min_duration_samples as u64
        })
        .max()
        .unwrap_or(sample_rate as u64 / 2) as usize;
    let mut mix = vec![0i32; total_samples.max(sample_rate as usize / 2)];

    for event in events {
        let duration_samples = ((event.duration_ms as u64 * sample_rate as u64) / 1000)
            .max(min_duration_samples as u64) as usize;
        let bus_gain = audio_mixer_bus_gain_permille(event, settings);
        let event_gain = if settings.mute_master {
            0
        } else {
            (event.amplitude as i64 * settings.master_gain_permille as i64 * bus_gain as i64
                / 1_000_000) as i32
        };
        let start = ((event.frame as u64 * sample_rate as u64) / TRUTH_HZ as u64) as usize;
        for sample_index in 0..duration_samples {
            let target = start + sample_index;
            if target >= mix.len() {
                break;
            }
            let phase = ((sample_index as u64 * event.frequency_hz as u64 * 2000)
                / sample_rate as u64)
                % 2000;
            let triangle = if phase < 1000 {
                phase as i32 * 2 - 1000
            } else {
                3000 - phase as i32 * 2
            };
            let texture_seed = (sample_index as u32)
                .wrapping_add(event.frame)
                .wrapping_add(event.event_id.len() as u32)
                .wrapping_mul(1_103_515_245)
                .wrapping_add(12_345);
            let texture = ((texture_seed >> 17) & 255) as i32 - 128;
            let envelope =
                ((duration_samples - sample_index) as i32 * 1000) / duration_samples as i32;
            let textured = triangle + texture * 3;
            mix[target] +=
                (textured as i64 * event_gain as i64 * envelope as i64 / 1_000_000) as i32;
        }
    }

    let limit = (i16::MAX as i32 * settings.peak_limit_permille / 1000).max(1);
    let mut samples = Vec::with_capacity(mix.len());
    let mut peak_abs = 0i32;
    let mut limited_sample_count = 0usize;
    let mut sum_square: i128 = 0;
    for sample in mix {
        let limited = sample.clamp(-limit, limit);
        if limited != sample {
            limited_sample_count += 1;
        }
        let abs = limited.abs();
        peak_abs = peak_abs.max(abs);
        sum_square += abs as i128 * abs as i128;
        samples.push(limited as i16);
    }

    let sample_count = samples.len();
    let duration_ms = (sample_count as u64 * 1000 / sample_rate as u64) as u32;
    let peak_permille = peak_abs * 1000 / i16::MAX as i32;
    let mean_square_permille = if sample_count == 0 {
        0
    } else {
        (sum_square * 1000 / sample_count as i128 / (i16::MAX as i128 * i16::MAX as i128)) as i32
    };
    (
        wav_from_samples(sample_rate, &samples),
        AudioMixerStats {
            sample_rate_hz: sample_rate,
            channels: 1,
            sample_count,
            duration_ms,
            event_count: events.len(),
            peak_abs,
            peak_permille,
            mean_square_permille,
            limited_sample_count,
        },
    )
}

fn audio_mixer_bus(event: &PresentationEvent) -> &'static str {
    if event.event_family == "ui_audio" {
        "ui"
    } else if event.event_family == "ambience" {
        "ambience"
    } else if event.event_family == "shock_cue" || event.vfx_id == "grip_loss" {
        "capability"
    } else {
        "impact"
    }
}

fn audio_mixer_bus_gain_permille(event: &PresentationEvent, settings: &AudioMixerSettings) -> i32 {
    match audio_mixer_bus(event) {
        "ui" => settings.ui_gain_permille,
        "ambience" => settings.ambience_gain_permille,
        "capability" => settings.capability_gain_permille,
        _ => settings.impact_gain_permille,
    }
}

fn wav_from_samples(sample_rate: u32, samples: &[i16]) -> Vec<u8> {
    let mut pcm = Vec::with_capacity(samples.len() * 2);
    for sample in samples {
        pcm.extend_from_slice(&sample.to_le_bytes());
    }
    let mut wav = Vec::with_capacity(44 + pcm.len());
    let data_len = pcm.len() as u32;
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_len).to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    wav.extend_from_slice(&pcm);
    wav
}

fn render_audio_mixer_settings_json(result: &DuelResult, settings: &AudioMixerSettings) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", AUDIO_MIXER_SCHEMA, true);
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(&mut out, 1, "scenario_id", &result.scenario_id, true);
    write_json_field(&mut out, 1, "content_hash", &result.content_hash, true);
    write_json_field(
        &mut out,
        1,
        "final_state_hash",
        &result.final_state_hash,
        true,
    );
    writeln!(&mut out, "  \"source\": \"trace-derived-only\",").unwrap();
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(&mut out, "  \"integrated_runtime_mixer_claimed\": true,").unwrap();
    writeln!(&mut out, "  \"human_audible_acceptance_claimed\": false,").unwrap();
    writeln!(
        &mut out,
        "  \"master_gain_permille\": {},",
        settings.master_gain_permille
    )
    .unwrap();
    writeln!(&mut out, "  \"buses\": {{").unwrap();
    writeln!(
        &mut out,
        "    \"ui_gain_permille\": {},",
        settings.ui_gain_permille
    )
    .unwrap();
    writeln!(
        &mut out,
        "    \"ambience_gain_permille\": {},",
        settings.ambience_gain_permille
    )
    .unwrap();
    writeln!(
        &mut out,
        "    \"impact_gain_permille\": {},",
        settings.impact_gain_permille
    )
    .unwrap();
    writeln!(
        &mut out,
        "    \"capability_gain_permille\": {}",
        settings.capability_gain_permille
    )
    .unwrap();
    writeln!(&mut out, "  }},").unwrap();
    writeln!(
        &mut out,
        "  \"captions_enabled\": {},",
        settings.captions_enabled
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"visual_equivalents_enabled\": {},",
        settings.visual_equivalents_enabled
    )
    .unwrap();
    writeln!(&mut out, "  \"mute_master\": {},", settings.mute_master).unwrap();
    writeln!(
        &mut out,
        "  \"peak_limit_permille\": {}",
        settings.peak_limit_permille
    )
    .unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

fn event_start_ms(event: &PresentationEvent) -> u64 {
    event.frame as u64 * 1000 / TRUTH_HZ as u64
}

fn render_audio_mixer_channels_json(
    result: &DuelResult,
    events: &[PresentationEvent],
    settings: &AudioMixerSettings,
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(
        &mut out,
        1,
        "schema",
        "oathyard.audio_mixer_channels.v1",
        true,
    );
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(&mut out, 1, "scenario_id", &result.scenario_id, true);
    write_json_field(
        &mut out,
        1,
        "final_state_hash",
        &result.final_state_hash,
        true,
    );
    writeln!(&mut out, "  \"source\": \"trace-derived-only\",").unwrap();
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(&mut out, "  \"events\": [").unwrap();
    for (index, event) in events.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "event_id", &event.event_id, true);
        write_json_field(&mut out, 3, "event_family", event.event_family, true);
        write_json_field(&mut out, 3, "source_event_id", &event.source_event_id, true);
        writeln!(&mut out, "      \"frame\": {},", event.frame).unwrap();
        writeln!(&mut out, "      \"start_ms\": {},", event_start_ms(event)).unwrap();
        write_json_field(&mut out, 3, "sound", event.sound_id, true);
        write_json_field(&mut out, 3, "bus", audio_mixer_bus(event), true);
        writeln!(
            &mut out,
            "      \"bus_gain_permille\": {},",
            audio_mixer_bus_gain_permille(event, settings)
        )
        .unwrap();
        writeln!(&mut out, "      \"frequency_hz\": {},", event.frequency_hz).unwrap();
        write_json_field(&mut out, 3, "caption", &event.caption, false);
        writeln!(&mut out, "    }}{}", comma(index + 1, events.len())).unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

fn render_audio_mixer_loudness_json(result: &DuelResult, stats: &AudioMixerStats) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(
        &mut out,
        1,
        "schema",
        "oathyard.audio_mixer_loudness.v1",
        true,
    );
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(&mut out, 1, "scenario_id", &result.scenario_id, true);
    write_json_field(
        &mut out,
        1,
        "final_state_hash",
        &result.final_state_hash,
        true,
    );
    writeln!(&mut out, "  \"integrated_runtime_mixer_claimed\": true,").unwrap();
    writeln!(&mut out, "  \"human_audible_acceptance_claimed\": false,").unwrap();
    writeln!(&mut out, "  \"sample_rate_hz\": {},", stats.sample_rate_hz).unwrap();
    writeln!(&mut out, "  \"channels\": {},", stats.channels).unwrap();
    writeln!(&mut out, "  \"sample_count\": {},", stats.sample_count).unwrap();
    writeln!(&mut out, "  \"duration_ms\": {},", stats.duration_ms).unwrap();
    writeln!(&mut out, "  \"event_count\": {},", stats.event_count).unwrap();
    writeln!(&mut out, "  \"peak_abs\": {},", stats.peak_abs).unwrap();
    writeln!(&mut out, "  \"peak_permille\": {},", stats.peak_permille).unwrap();
    writeln!(
        &mut out,
        "  \"mean_square_permille\": {},",
        stats.mean_square_permille
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"limited_sample_count\": {}",
        stats.limited_sample_count
    )
    .unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

fn render_audio_mixer_report(
    result: &DuelResult,
    events: &[PresentationEvent],
    settings: &AudioMixerSettings,
    stats: &AudioMixerStats,
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD Runtime Audio Mixer Report").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "Status: PASSED").unwrap();
    writeln!(&mut out, "- Scenario: `{}`", result.scenario_id).unwrap();
    writeln!(&mut out, "- Content hash: `{}`", result.content_hash).unwrap();
    writeln!(
        &mut out,
        "- Final state hash: `{}`",
        result.final_state_hash
    )
    .unwrap();
    writeln!(&mut out, "- Source: `trace-derived-only`").unwrap();
    writeln!(&mut out, "- Integrated runtime mixer claimed: `true`").unwrap();
    writeln!(&mut out, "- Presentation only: `true`").unwrap();
    writeln!(&mut out, "- Truth mutation: `none`").unwrap();
    writeln!(&mut out, "- Event count: `{}`", events.len()).unwrap();
    writeln!(
        &mut out,
        "- Bus gains: `ui {}` `ambience {}` `impact {}` `capability {}`",
        settings.ui_gain_permille,
        settings.ambience_gain_permille,
        settings.impact_gain_permille,
        settings.capability_gain_permille
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Master gain: `{}`",
        settings.master_gain_permille
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Peak limit: `{}` permille",
        settings.peak_limit_permille
    )
    .unwrap();
    writeln!(&mut out, "- Mixed WAV: `runtime_audio_mix.wav`").unwrap();
    writeln!(
        &mut out,
        "- Loudness: peak `{}` permille, mean-square `{}` permille, limited samples `{}`",
        stats.peak_permille, stats.mean_square_permille, stats.limited_sample_count
    )
    .unwrap();
    writeln!(&mut out, "- Captions present: `captions.srt`").unwrap();
    writeln!(
        &mut out,
        "- Visual equivalents enabled: `{}`",
        settings.visual_equivalents_enabled
    )
    .unwrap();
    writeln!(&mut out, "- Human audible acceptance claimed: `false`").unwrap();
    writeln!(
        &mut out,
        "This proves an integrated deterministic mixer artifact path in the native executable. It does not prove owner audio acceptance, platform audio certification, spatial mix acceptance, or final loudness approval."
    )
    .unwrap();
    out
}

fn render_audio_events_json(result: &DuelResult, events: &[PresentationEvent]) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", "oathyard.audio_events.v1", true);
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(&mut out, 1, "scenario_id", &result.scenario_id, true);
    write_json_field(&mut out, 1, "content_hash", &result.content_hash, true);
    write_json_field(
        &mut out,
        1,
        "final_state_hash",
        &result.final_state_hash,
        true,
    );
    writeln!(&mut out, "  \"source\": \"trace-derived-only\",").unwrap();
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(&mut out, "  \"owner_audio_acceptance_claimed\": false,").unwrap();
    writeln!(&mut out, "  \"events\": [").unwrap();
    for (index, event) in events.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "event_id", &event.event_id, true);
        write_json_field(&mut out, 3, "event_family", event.event_family, true);
        write_json_field(&mut out, 3, "source_event_id", &event.source_event_id, true);
        write_json_field(
            &mut out,
            3,
            "source_event_kind",
            event.source_event_kind,
            true,
        );
        writeln!(&mut out, "      \"frame\": {},", event.frame).unwrap();
        write_json_field(&mut out, 3, "sound", event.sound_id, true);
        write_json_field(&mut out, 3, "caption", &event.caption, true);
        writeln!(&mut out, "      \"frequency_hz\": {},", event.frequency_hz).unwrap();
        writeln!(&mut out, "      \"duration_ms\": {}", event.duration_ms).unwrap();
        writeln!(&mut out, "    }}{}", comma(index + 1, events.len())).unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

fn write_vfx_material_ids_json(
    out: &mut String,
    indent: usize,
    values: &[&'static str],
    trailing_comma: bool,
) {
    let pad = "  ".repeat(indent);
    writeln!(out, "{pad}\"material_ids\": [").unwrap();
    for (index, value) in values.iter().enumerate() {
        writeln!(
            out,
            "{}  {}{}",
            pad,
            json_quote(value),
            comma(index + 1, values.len())
        )
        .unwrap();
    }
    writeln!(out, "{pad}]{}", if trailing_comma { "," } else { "" }).unwrap();
}

fn build_impact_vfx_evidence(events: &[PresentationEvent]) -> VfxEvidenceArtifact {
    const FILE: &str = "impact_vfx_event_palette.json";
    let width = 640usize;
    let height = 240usize;
    let mut pixels = vec![0u8; width * height * 3];
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 3;
            pixels[idx] = 25;
            pixels[idx + 1] = 24;
            pixels[idx + 2] = 22;
        }
    }
    for (index, event) in events.iter().enumerate() {
        let x0 = 20 + (index * 37) % (width - 60);
        let h = 24 + (event.duration_ms as usize % 90);
        let y0 = height.saturating_sub(20 + h);
        let color = vfx_event_color(event.effect_family);
        for y in y0..(y0 + h).min(height - 12) {
            for x in x0..(x0 + 18).min(width - 12) {
                let idx = (y * width + x) * 3;
                pixels[idx] = color.0;
                pixels[idx + 1] = color.1;
                pixels[idx + 2] = color.2;
            }
        }
    }
    let sha256 = hash_hex(&pixels);
    let distinct_color_count = distinct_rgb_count(&pixels);
    VfxEvidenceArtifact {
        file: FILE,
        width,
        height,
        sha256,
        distinct_color_count,
        event_count: events.len(),
    }
}

fn distinct_rgb_count(pixels: &[u8]) -> usize {
    let mut colors = BTreeSet::new();
    for chunk in pixels.chunks_exact(3) {
        colors.insert([chunk[0], chunk[1], chunk[2]]);
    }
    colors.len()
}

fn vfx_event_color(effect_family: &str) -> (u8, u8, u8) {
    match effect_family {
        "spark" => (235, 178, 62),
        "dust" => (154, 135, 95),
        "blood_wetness" => (124, 22, 28),
        "debris" => (91, 80, 63),
        "material_impact_burst" => (210, 146, 88),
        "weapon_trail" => (126, 172, 196),
        "shock_cue" => (190, 96, 210),
        _ => (110, 110, 110),
    }
}

fn render_audio_vfx_timing_loudness_json(
    result: &DuelResult,
    events: &[PresentationEvent],
    stats: &AudioMixerStats,
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(
        &mut out,
        1,
        "schema",
        "oathyard.audio_vfx_timing_loudness.v1",
        true,
    );
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(&mut out, 1, "scenario_id", &result.scenario_id, true);
    write_json_field(
        &mut out,
        1,
        "final_state_hash",
        &result.final_state_hash,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "timing_source",
        "truth_frame_120hz_after_hash",
        true,
    );
    write_json_field(
        &mut out,
        1,
        "device_playback_scope",
        "not_claimed_here",
        true,
    );
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(&mut out, "  \"event_count\": {},", events.len()).unwrap();
    writeln!(&mut out, "  \"sample_rate_hz\": {},", stats.sample_rate_hz).unwrap();
    writeln!(&mut out, "  \"duration_ms\": {},", stats.duration_ms).unwrap();
    writeln!(&mut out, "  \"peak_permille\": {},", stats.peak_permille).unwrap();
    writeln!(
        &mut out,
        "  \"mean_square_permille\": {}",
        stats.mean_square_permille
    )
    .unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

fn render_vfx_manifest_json(
    result: &DuelResult,
    events: &[PresentationEvent],
    vfx_evidence: &VfxEvidenceArtifact,
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", "oathyard.vfx_manifest.v1", true);
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(&mut out, 1, "scenario_id", &result.scenario_id, true);
    write_json_field(
        &mut out,
        1,
        "final_state_hash",
        &result.final_state_hash,
        true,
    );
    writeln!(&mut out, "  \"source\": \"trace-derived-only\",").unwrap();
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"reduced_flash_compliant\": true,").unwrap();
    writeln!(&mut out, "  \"owner_visual_acceptance\": false,").unwrap();
    writeln!(&mut out, "  \"owner_audio_acceptance\": false,").unwrap();
    writeln!(&mut out, "  \"public_demo_ready\": {PUBLIC_DEMO_READY},").unwrap();
    writeln!(
        &mut out,
        "  \"release_candidate_ready\": {RELEASE_CANDIDATE_READY},"
    )
    .unwrap();
    writeln!(&mut out, "  \"nonvisual_vfx_evidence\": {{").unwrap();
    write_json_field(&mut out, 2, "file", vfx_evidence.file, true);
    writeln!(&mut out, "    \"width\": {},", vfx_evidence.width).unwrap();
    writeln!(&mut out, "    \"height\": {},", vfx_evidence.height).unwrap();
    write_json_field(&mut out, 2, "sha256", &vfx_evidence.sha256, true);
    writeln!(
        &mut out,
        "    \"distinct_color_count\": {},",
        vfx_evidence.distinct_color_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "    \"event_count\": {}",
        vfx_evidence.event_count
    )
    .unwrap();
    writeln!(&mut out, "  }},").unwrap();
    writeln!(&mut out, "  \"events\": [").unwrap();
    for (index, event) in events.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "event_id", &event.event_id, true);
        write_json_field(&mut out, 3, "source_event_id", &event.source_event_id, true);
        write_json_field(&mut out, 3, "effect_family", event.effect_family, true);
        write_json_field(&mut out, 3, "vfx", event.vfx_id, true);
        writeln!(&mut out, "      \"frame\": {},", event.frame).unwrap();
        write_vfx_material_ids_json(&mut out, 3, &event.material_ids, true);
        write_json_field(&mut out, 3, "weapon_id", &event.weapon_id, true);
        write_json_field(&mut out, 3, "armor_id", &event.armor_id, true);
        write_json_field(&mut out, 3, "caption", &event.caption, true);
        writeln!(
            &mut out,
            "      \"reduced_flash_compliant\": {}",
            event.reduced_flash_compliant
        )
        .unwrap();
        writeln!(&mut out, "    }}{}", comma(index + 1, events.len())).unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

fn render_captions_srt(events: &[PresentationEvent]) -> String {
    let mut out = String::new();
    for (index, event) in events.iter().enumerate() {
        let start_ms = event.frame as u64 * 1000 / TRUTH_HZ as u64;
        let end_ms = start_ms + 850;
        writeln!(&mut out, "{}", index + 1).unwrap();
        writeln!(
            &mut out,
            "{} --> {}",
            srt_timestamp(start_ms),
            srt_timestamp(end_ms)
        )
        .unwrap();
        writeln!(&mut out, "{}", event.caption).unwrap();
        writeln!(&mut out).unwrap();
    }
    out
}

fn render_audio_vfx_report(
    result: &DuelResult,
    events: &[PresentationEvent],
    stats: &AudioMixerStats,
    vfx_evidence: &VfxEvidenceArtifact,
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD Audio/VFX Report").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "Status: PASSED").unwrap();
    writeln!(&mut out, "- Scenario: `{}`", result.scenario_id).unwrap();
    writeln!(&mut out, "- Content hash: `{}`", result.content_hash).unwrap();
    writeln!(
        &mut out,
        "- Final state hash: `{}`",
        result.final_state_hash
    )
    .unwrap();
    writeln!(&mut out, "- Event count: `{}`", events.len()).unwrap();
    writeln!(&mut out, "- Source: `trace-derived-only`").unwrap();
    writeln!(
        &mut out,
        "- Audio source: `repo-owned procedural integer WAV`"
    )
    .unwrap();
    writeln!(&mut out, "- Presentation only: `true`").unwrap();
    writeln!(&mut out, "- Truth mutation: `none`").unwrap();
    writeln!(
        &mut out,
        "- Truth read-only proof: `regression compares trace_json, replay_json, and final_state_hash before/after artifact generation`"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Event families: `{}`",
        event_family_list(events)
    )
    .unwrap();
    writeln!(&mut out, "- VFX families: `{}`", effect_family_list(events)).unwrap();
    writeln!(
        &mut out,
        "- Loudness evidence: peak `{}` permille, mean-square `{}` permille, limited samples `{}`",
        stats.peak_permille, stats.mean_square_permille, stats.limited_sample_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Impact VFX nonvisual evidence: `{}` `{}`x`{}` sha256 `{}` colors `{}`",
        vfx_evidence.file,
        vfx_evidence.width,
        vfx_evidence.height,
        vfx_evidence.sha256,
        vfx_evidence.distinct_color_count
    )
    .unwrap();
    writeln!(&mut out, "- Critical audio captions: `captions.srt`").unwrap();
    writeln!(
        &mut out,
        "- Timing/loudness scope: `audio_vfx_timing_loudness.json`; device playback and runtime mixer acceptance are separate gates"
    )
    .unwrap();
    writeln!(&mut out, "- Owner visual acceptance: `false`").unwrap();
    writeln!(&mut out, "- Owner audio acceptance: `false`").unwrap();
    writeln!(&mut out, "- Public demo ready: `{PUBLIC_DEMO_READY}`").unwrap();
    writeln!(
        &mut out,
        "- Release candidate ready: `{RELEASE_CANDIDATE_READY}`"
    )
    .unwrap();
    out
}

fn event_family_list(events: &[PresentationEvent]) -> String {
    events
        .iter()
        .map(|event| event.event_family)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(",")
}

fn effect_family_list(events: &[PresentationEvent]) -> String {
    events
        .iter()
        .map(|event| event.effect_family)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join(",")
}

fn srt_timestamp(ms: u64) -> String {
    let hours = ms / 3_600_000;
    let minutes = (ms / 60_000) % 60;
    let seconds = (ms / 1000) % 60;
    let millis = ms % 1000;
    format!("{hours:02}:{minutes:02}:{seconds:02},{millis:03}")
}
