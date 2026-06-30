use std::fs;
use std::path::Path;
use std::process::Command;

use oathyard::{
    run_scenario_text, verify_replay_export_bundle, verify_replay_text,
    write_accessibility_artifacts, write_ai_duel_artifacts, write_ai_sweep_artifacts,
    write_animation_state_machine_artifacts, write_audio_mixer_artifacts,
    write_audio_vfx_artifacts, write_contact_matrix_artifacts, write_gamepad_smoke_artifacts,
    write_input_artifacts, write_match_artifacts, write_negative_input_audit_artifacts,
    write_pbr_material_artifacts, write_presentation_bricks_artifacts, write_replay_export_bundle,
    write_runtime_settings_artifacts, write_truth_edge_audit_artifacts,
    write_truth_stress_artifacts, ActionLabel, ARENAS, ARMORS, FIGHTER_TRADITIONS, WEAPONS,
};

const BASIC: &str = include_str!("../examples/duels/basic_oathyard.duel");

struct AssetGenerationTestLock {
    path: std::path::PathBuf,
}

impl Drop for AssetGenerationTestLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn acquire_asset_generation_test_lock() -> AssetGenerationTestLock {
    let path = Path::new("target/tmp/oathyard_asset_generation_test.lock").to_path_buf();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create asset-generation test lock parent");
    }
    for _ in 0..2400 {
        match fs::create_dir(&path) {
            Ok(()) => return AssetGenerationTestLock { path },
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                std::thread::sleep(std::time::Duration::from_millis(250));
            }
            Err(error) => panic!("create asset-generation test lock: {error}"),
        }
    }
    panic!("timed out waiting 600s for asset-generation test lock");
}

#[test]
fn repeated_run_is_byte_stable_for_replay_outputs() {
    let first = run_scenario_text(BASIC).expect("first run");
    let second = run_scenario_text(BASIC).expect("second run");

    assert_eq!(first.final_state_hash, second.final_state_hash);
    assert_eq!(first.turn_hashes, second.turn_hashes);
    assert_eq!(first.trace_json, second.trace_json);
    assert_eq!(first.replay_json, second.replay_json);
    assert_eq!(
        first.fight_film_manifest_json,
        second.fight_film_manifest_json
    );
}

#[test]
fn replay_verifier_reproduces_hashes() {
    let result = run_scenario_text(BASIC).expect("run");
    let replayed = verify_replay_text(&result.replay_json).expect("replay verifies");

    assert_eq!(result.initial_state_hash, replayed.initial_state_hash);
    assert_eq!(result.turn_hashes, replayed.turn_hashes);
    assert_eq!(result.final_state_hash, replayed.final_state_hash);
    assert_eq!(result.end_condition, replayed.end_condition);
    assert!(result.replay_json.contains("\"end_condition_status\""));
    assert!(result.replay_json.contains("\"end_condition_winner\""));
}

#[test]
fn replay_verifier_requires_end_condition_metadata() {
    let result = run_scenario_text(BASIC).expect("run");
    let status_field = format!(
        "  \"end_condition_status\": \"{}\",\n",
        result.end_condition.status
    );
    let missing_status = result.replay_json.replacen(&status_field, "", 1);

    assert_ne!(
        result.replay_json, missing_status,
        "generated replay fixture must include end_condition_status metadata"
    );
    let Err(error) = verify_replay_text(&missing_status) else {
        panic!("replay missing end_condition_status was accepted");
    };
    assert!(
        error
            .to_string()
            .contains("replay missing end_condition_status"),
        "expected missing end_condition_status error, got {error}"
    );

    let winner = result
        .end_condition
        .winner
        .map(|seat| format!("seat_{seat}"))
        .unwrap_or_else(|| "none".to_string());
    let winner_field = format!("  \"end_condition_winner\": \"{}\",\n", winner);
    let missing_winner = result.replay_json.replacen(&winner_field, "", 1);

    assert_ne!(
        result.replay_json, missing_winner,
        "generated replay fixture must include end_condition_winner metadata"
    );
    let Err(error) = verify_replay_text(&missing_winner) else {
        panic!("replay missing end_condition_winner was accepted");
    };
    assert!(
        error
            .to_string()
            .contains("replay missing end_condition_winner"),
        "expected missing end_condition_winner error, got {error}"
    );
}

#[test]
fn replay_verifier_rejects_truth_hz_mismatch() {
    let result = run_scenario_text(BASIC).expect("run");
    let tampered = result
        .replay_json
        .replacen("\"truth_hz\": 120", "\"truth_hz\": 60", 1);

    assert_ne!(
        result.replay_json, tampered,
        "generated replay fixture must include truth_hz metadata"
    );
    let Err(error) = verify_replay_text(&tampered) else {
        panic!("tampered replay truth_hz was accepted");
    };
    assert!(
        error.to_string().contains("truth_hz mismatch"),
        "expected truth_hz mismatch error, got {error}"
    );
}

#[test]
fn replay_verifier_rejects_malformed_truth_hz_scalar() {
    let result = run_scenario_text(BASIC).expect("run");
    let tampered = result
        .replay_json
        .replacen("\"truth_hz\": 120", "\"truth_hz\": 120oops", 1);

    assert_ne!(
        result.replay_json, tampered,
        "generated replay fixture must include truth_hz metadata"
    );
    let Err(error) = verify_replay_text(&tampered) else {
        panic!("malformed replay truth_hz scalar was accepted");
    };
    assert!(
        error.to_string().contains("truth_hz"),
        "expected truth_hz scalar validation error, got {error}"
    );
}

#[test]
fn replay_export_bundle_contains_verified_artifacts_and_hash_manifest() {
    let root = std::path::Path::new("target/tmp/oathyard_export_bundle_test");
    fs::create_dir_all(root).expect("export bundle temp dir");
    let replay_path = root.join("source_replay.json");
    let out_dir = root.join("bundle");
    let source = run_scenario_text(BASIC).expect("run");
    fs::write(&replay_path, &source.replay_json).expect("write replay");

    let exported = write_replay_export_bundle(&replay_path, &out_dir).expect("export bundle");
    let verified = verify_replay_export_bundle(&out_dir).expect("verify bundle");

    assert_eq!(source.final_state_hash, exported.final_state_hash);
    assert_eq!(source.final_state_hash, verified.final_state_hash);
    let manifest =
        fs::read_to_string(out_dir.join("export_bundle_manifest.json")).expect("manifest");
    let report = fs::read_to_string(out_dir.join("export_bundle_report.md")).expect("report");
    let hashes = fs::read_to_string(out_dir.join("bundle_hashes.txt")).expect("hashes");
    assert!(manifest.contains("\"schema\": \"oathyard.replay_export_bundle.v1\""));
    assert!(manifest.contains("\"source\": \"verified-replay-export\""));
    assert!(manifest.contains("\"replay_verified\": true"));
    assert!(manifest.contains("\"presentation_only\": true"));
    assert!(manifest.contains("\"truth_mutation\": false"));
    assert!(manifest.contains("\"path\": \"trace.json\""));
    assert!(report.contains("Status: PASSED"));
    assert!(report.contains("Replay verified: `true`"));
    assert!(report.contains("Truth mutation: `none`"));
    assert!(hashes.contains("trace.json"));
    assert!(hashes.contains("export_bundle_manifest.json"));
    fs::write(out_dir.join("trace.json"), "{}\n").expect("tamper trace");
    let error = verify_replay_export_bundle(&out_dir).expect_err("tamper should fail");
    assert!(error.to_string().contains("export bundle hash mismatch"));
}

#[test]
fn loadout_variation_changes_costs_and_outcome_deterministically() {
    let mail = run_scenario_text(BASIC).expect("mail scenario");
    let gambeson_text = BASIC.replace(
        "fighter 1 vale longsword mail_hauberk",
        "fighter 1 vale longsword gambeson",
    );
    let gambeson = run_scenario_text(&gambeson_text).expect("gambeson scenario");

    assert_ne!(mail.final_state_hash, gambeson.final_state_hash);
    assert_ne!(
        mail.turns[0].costs[1].current_frames,
        gambeson.turns[0].costs[1].current_frames
    );
    assert_ne!(
        mail.turns[0].contacts[0].material_result,
        gambeson.turns[0].contacts[0].material_result
    );
}

#[test]
fn injury_changes_future_action_cost_and_capability() {
    let result = run_scenario_text(BASIC).expect("basic scenario");
    let first_contact = &result.turns[0].contacts[0];

    assert_eq!(first_contact.capability_delta.recovery_slowdown_add, 12);
    assert!(first_contact.capability_delta.torso_rotation_delta < 0);
    assert!(first_contact
        .cause_chain
        .contains("next thrust recovery +12 frames"));

    let pre_injury_guard_cost = result.turns[0].costs[1].current_frames;
    let post_injury_thrust_cost = result.turns[1].costs[1].current_frames;
    assert!(post_injury_thrust_cost > pre_injury_guard_cost);
}

#[test]
fn simultaneous_contact_packets_are_ordered_by_truth_frame() {
    let scenario = "\
scenario contact_order_stress
fighter 0 reed ash_spear lamellar
fighter 1 bruiser iron_maul bruiser_padded_plate
turn 0 0 thrust forward weapon_arm
turn 0 1 shove forward torso
";
    let result = run_scenario_text(scenario).expect("contact ordering run");
    let replayed = verify_replay_text(&result.replay_json).expect("contact ordering replay");
    let turn = &result.turns[0];

    assert_eq!(result.final_state_hash, replayed.final_state_hash);
    assert_eq!(turn.contacts.len(), 2);
    assert_eq!(turn.contacts[0].frame, 20);
    assert_eq!(turn.contacts[0].attacker, 1);
    assert_eq!(turn.contacts[1].frame, 28);
    assert_eq!(turn.contacts[1].attacker, 0);
    assert!(turn
        .contacts
        .windows(2)
        .all(|pair| pair[0].frame <= pair[1].frame));
    assert!(result.trace_json.contains(
        "\"contact_order_rule\": \"frame_then_attacker_then_defender_then_action_then_target_then_direction\""
    ));
    let first_report_contact = result
        .report_md
        .find("Frame 20 fighter 1")
        .expect("earlier contact in report");
    let second_report_contact = result
        .report_md
        .find("Frame 28 fighter 0")
        .expect("later contact in report");
    assert!(first_report_contact < second_report_contact);
}

#[test]
fn deterministic_end_condition_records_physical_capability_stop() {
    let stress = "\
scenario capability_stop_stress
fighter 0 reed ash_spear lamellar
fighter 1 bruiser iron_maul bruiser_padded_plate
turn 0 0 thrust forward weapon_arm
turn 0 1 brace center torso
turn 1 0 guard center torso
turn 1 1 bash forward torso
turn 2 0 thrust forward weapon_arm
turn 2 1 shove forward torso
turn 3 0 recover center torso
turn 3 1 bash forward torso
turn 4 0 recover center torso
turn 4 1 guard center torso
turn 5 0 recover center torso
turn 5 1 bash forward torso
turn 6 0 recover center torso
turn 6 1 shove forward torso
turn 7 0 recover center torso
turn 7 1 bash forward torso
";
    let first = run_scenario_text(stress).expect("capability stop stress run");
    let second = run_scenario_text(stress).expect("capability stop repeat run");
    let replayed = verify_replay_text(&first.replay_json).expect("capability stop replay");

    assert_eq!(first.final_state_hash, second.final_state_hash);
    assert_eq!(first.end_condition, second.end_condition);
    assert_eq!(first.end_condition, replayed.end_condition);
    assert_eq!(first.end_condition.status, "seat_1_victory_capability_stop");
    assert_eq!(first.end_condition.winner, Some(1));
    assert!(first.end_condition.fighters[0].incapacitated);
    assert_eq!(
        first.end_condition.fighters[0].stop_kind,
        "torso_rotation_locked"
    );
    assert!(first.end_condition.fighters[0].torso_rotation_permille <= 360);
    assert!(first.end_condition.fighters[0].recovery_slowdown_frames >= 20);
    assert!(first.trace_json.contains("\"end_condition\""));
    assert!(first.trace_json.contains("\"torso_rotation_locked\""));
    assert!(first.report_md.contains("## End Condition"));
    assert!(first.report_md.contains("seat_1_victory_capability_stop"));
}

#[test]
fn truth_source_avoids_forbidden_shortcut_tokens() {
    let mut files = Vec::new();
    collect_rust_source_files(Path::new("src"), &mut files);
    files.sort();
    let mut source = String::new();
    for file in files {
        source.push_str(&fs::read_to_string(file).expect("source"));
        source.push('\n');
    }
    let forbidden = [
        "hit_points",
        "health_points",
        "armor_points",
        "dps",
        "crit_chance",
        "super_meter",
        "bonus_damage",
        "damage_bonus",
        "speed_bonus",
        "+damage",
        "+speed",
    ];
    for token in forbidden {
        assert!(
            !source.contains(token),
            "truth source contains forbidden token {token}"
        );
    }
}

fn collect_rust_source_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).expect("source directory") {
        let path = entry.expect("source directory entry").path();
        if path.is_dir() {
            collect_rust_source_files(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

#[test]
fn expanded_content_meets_full_game_family_counts() {
    assert!(WEAPONS.len() >= 8);
    assert!(WEAPONS.iter().any(|weapon| weapon.id == "billhook"
        && weapon.hook_permille >= 800
        && weapon.grip_points == 2));
    assert!(ARMORS.len() >= 6);
    assert!(FIGHTER_TRADITIONS.len() >= 6);
    assert!(ARENAS.len() >= 2);
}

#[test]
fn expanded_action_labels_parse() {
    for label in [
        "step",
        "pivot",
        "guard",
        "parry",
        "cut",
        "thrust",
        "brace",
        "bash",
        "hook_bind",
        "grab",
        "shove",
        "kick",
        "recover",
    ] {
        assert_eq!(ActionLabel::parse(label).expect(label).as_str(), label);
    }
}

#[test]
fn match_artifacts_are_generated() {
    let root = std::path::Path::new("target/tmp/oathyard_test_artifacts");
    let match_dir = root.join("match");
    write_match_artifacts("examples/duels/axe_vs_spear.duel", &match_dir, 3)
        .expect("match artifacts");

    assert!(match_dir.join("match_summary.json").is_file());
    assert!(match_dir.join("round_1/replay.json").is_file());
}

#[test]
fn audio_vfx_artifacts_are_trace_derived() {
    let root = std::path::Path::new("target/tmp/oathyard_audio_vfx_test");
    let baseline = run_scenario_text(BASIC).expect("baseline truth run");
    let generated = write_audio_vfx_artifacts("examples/duels/basic_oathyard.duel", root)
        .expect("audio vfx artifacts");

    let wav = fs::read(root.join("audio_mix.wav")).expect("wav");
    let events = fs::read_to_string(root.join("audio_events.json")).expect("events");
    let vfx = fs::read_to_string(root.join("vfx_manifest.json")).expect("vfx");
    let captions = fs::read_to_string(root.join("captions.srt")).expect("captions");
    let timing =
        fs::read_to_string(root.join("audio_vfx_timing_loudness.json")).expect("timing loudness");
    let contact_sheet = fs::read(root.join("impact_vfx_contact_sheet.ppm")).expect("contact sheet");

    assert_eq!(baseline.trace_json, generated.trace_json);
    assert_eq!(baseline.replay_json, generated.replay_json);
    assert_eq!(baseline.final_state_hash, generated.final_state_hash);

    assert!(wav.starts_with(b"RIFF"));
    assert!(contact_sheet.starts_with(b"P6"));
    assert!(events.contains("\"schema\": \"oathyard.audio_events.v1\""));
    assert!(events.contains("trace-derived-only"));
    assert!(events.contains("\"truth_mutation\": false"));
    assert!(events.contains("\"owner_audio_acceptance_claimed\": false"));
    for family in [
        "ui_audio",
        "ambience",
        "footwork",
        "weapon_trail",
        "material_impact",
        "shock_cue",
        "replay_fight_film_audio",
    ] {
        assert!(events.contains(&format!("\"event_family\": \"{family}\"")));
    }
    assert!(vfx.contains("\"schema\": \"oathyard.vfx_manifest.v1\""));
    assert!(vfx.contains("\"presentation_only\": true"));
    assert!(vfx.contains("\"reduced_flash_compliant\": true"));
    assert!(vfx.contains("\"owner_visual_acceptance\": false"));
    assert!(vfx.contains("\"source_event_id\":"));
    assert!(vfx.contains("\"material_ids\":"));
    for family in [
        "spark",
        "dust",
        "blood_wetness",
        "debris",
        "material_impact_burst",
        "weapon_trail",
        "shock_cue",
    ] {
        assert!(vfx.contains(&format!("\"effect_family\": \"{family}\"")));
    }
    assert!(timing.contains("\"timing_source\": \"truth_frame_120hz_after_hash\""));
    assert!(timing.contains("\"peak_permille\":"));
    assert!(timing.contains("\"device_playback_scope\": \"not_claimed_here\""));
    assert!(captions.contains("-->"));
}

#[test]
fn runtime_audio_mixer_artifacts_are_trace_derived_and_truth_read_only() {
    let root = std::path::Path::new("target/tmp/oathyard_audio_mixer_test");
    write_audio_mixer_artifacts("examples/duels/basic_oathyard.duel", root)
        .expect("audio mixer artifacts");

    let wav = fs::read(root.join("runtime_audio_mix.wav")).expect("runtime mixer wav");
    let settings =
        fs::read_to_string(root.join("audio_mixer_settings.json")).expect("mixer settings");
    let channels =
        fs::read_to_string(root.join("audio_mixer_channels.json")).expect("mixer channels");
    let loudness =
        fs::read_to_string(root.join("audio_mixer_loudness.json")).expect("mixer loudness");
    let report = fs::read_to_string(root.join("audio_mixer_report.md")).expect("mixer report");

    assert!(wav.starts_with(b"RIFF"));
    assert!(settings.contains("\"schema\": \"oathyard.audio_mixer.v1\""));
    assert!(settings.contains("\"integrated_runtime_mixer_claimed\": true"));
    assert!(settings.contains("\"human_audible_acceptance_claimed\": false"));
    assert!(settings.contains("\"truth_mutation\": false"));
    assert!(channels.contains("\"schema\": \"oathyard.audio_mixer_channels.v1\""));
    assert!(channels.contains("\"bus\": \"impact\""));
    assert!(loudness.contains("\"schema\": \"oathyard.audio_mixer_loudness.v1\""));
    assert!(loudness.contains("\"peak_permille\":"));
    assert!(report.contains("Status: PASSED"));
    assert!(report.contains("Truth mutation: `none`"));
}

#[test]
fn input_map_artifacts_are_remap_ready_and_presentation_only() {
    let root = std::path::Path::new("target/tmp/oathyard_input_map_test");
    write_input_artifacts(root).expect("input artifacts");

    let map = fs::read_to_string(root.join("input_map.json")).expect("input map");
    let profile = fs::read_to_string(root.join("input_profile.json")).expect("input profile");
    let deck =
        fs::read_to_string(root.join("steam_deck_checklist.md")).expect("deck input checklist");
    let report =
        fs::read_to_string(root.join("input_remap_report.md")).expect("input remap report");

    assert!(map.contains("\"schema\": \"oathyard.input_map.v1\""));
    assert!(map.contains("\"controller_profile\": \"input_profile.json\""));
    assert!(map.contains("\"remappable\": true"));
    assert!(map.contains("\"truth_mutation\": false"));
    assert!(map.contains("\"mouse\": \"bottom_right_click\""));
    assert!(map.contains("\"gamepad_ready\": \"gamepad_south\""));
    assert!(map.contains("\"glyph\": \"A\""));
    for action in [
        "main_menu_start",
        "settings_accessibility",
        "fighter_select",
        "observe",
        "plan",
        "resolve",
        "replay_browser",
        "fight_film",
        "performance_debug_overlay",
    ] {
        assert!(map.contains(&format!("\"action\": \"{action}\"")));
    }
    assert!(profile.contains("\"schema\": \"oathyard.input_profile.v1\""));
    assert!(profile.contains("\"all_current_screens_reachable_with_default_controller\": true"));
    assert!(profile.contains("\"steam_deck_local_schema_check_passed\": true"));
    assert!(profile.contains("\"physical_gamepad_hardware_claimed\": false"));
    assert!(profile.contains("\"steam_deck_hardware_claimed\": false"));
    assert!(profile.contains("\"owner_input_acceptance_claimed\": false"));
    assert!(profile.contains("\"boundary\": \"presentation_command_only\""));
    for screen in [
        "main_menu",
        "mode_select",
        "settings_accessibility",
        "fighter_select",
        "loadout_select",
        "observe",
        "plan",
        "commit_reveal",
        "resolve",
        "consequence",
        "replay_browser",
        "fight_film",
        "performance_debug_overlay",
    ] {
        assert!(profile.contains(&format!("\"screen\": \"{screen}\"")));
        assert!(deck.contains(&format!("`{screen}`")));
    }
    assert!(deck.contains("PASSED_LOCAL_INPUT_SCHEMA"));
    assert!(deck.contains("Steam Deck hardware claimed: `false`"));
    assert!(deck.contains("Owner input acceptance claimed: `false`"));
    assert!(report.contains("Status: PASSED"));
    assert!(report.contains("native X11 ButtonPress zones"));
    assert!(report.contains("hardware smoke not claimed"));
}

#[test]
fn gamepad_smoke_artifacts_are_presentation_only_and_honest_about_hardware() {
    let root = std::path::Path::new("target/tmp/oathyard_gamepad_smoke_test");
    write_gamepad_smoke_artifacts(root).expect("gamepad smoke artifacts");

    let smoke = fs::read_to_string(root.join("gamepad_smoke.json")).expect("gamepad smoke json");
    let report =
        fs::read_to_string(root.join("gamepad_smoke_report.md")).expect("gamepad smoke report");

    assert!(smoke.contains("\"schema\": \"oathyard.gamepad_smoke.v1\""));
    assert!(smoke.contains("\"presentation_only\": true"));
    assert!(smoke.contains("\"truth_mutation\": false"));
    assert!(smoke.contains("\"physical_gamepad_hardware_claimed\": false"));
    assert!(smoke.contains("\"steam_deck_hardware_claimed\": false"));
    assert!(smoke.contains("\"device_count\":"));
    assert!(report.contains("OATHYARD Gamepad Smoke Report"));
    assert!(report.contains("Truth mutation: `none`"));
    assert!(report.contains("Physical gamepad hardware claimed: `false`"));
    assert!(report.contains("Steam Deck hardware claimed: `false`"));
}

#[test]
fn accessibility_settings_are_presentation_only_and_caption_ready() {
    let root = std::path::Path::new("target/tmp/oathyard_accessibility_test");
    write_accessibility_artifacts(root).expect("accessibility artifacts");

    let settings =
        fs::read_to_string(root.join("accessibility_settings.json")).expect("settings json");
    let report = fs::read_to_string(root.join("accessibility_report.md")).expect("report");

    assert!(settings.contains("\"schema\": \"oathyard.accessibility_settings.v1\""));
    assert!(settings.contains("\"presentation_only\": true"));
    assert!(settings.contains("\"truth_mutation\": false"));
    assert!(settings.contains("\"captions_default\": true"));
    assert!(settings.contains("\"critical_audio_visual_equivalent\": true"));
    assert!(settings.contains("\"remapping_supported\": true"));
    assert!(settings.contains("\"text_scale_min_permille\": 1000"));
    assert!(settings.contains("\"text_scale_default_permille\": 1150"));
    assert!(settings.contains("\"text_scale_max_permille\": 1600"));
    assert!(settings.contains("\"high_contrast_mode\": true"));
    assert!(settings.contains("\"reduced_motion_mode\": true"));
    assert!(settings.contains("\"flash_events_per_second_max\": 0"));
    assert!(settings.contains("\"camera_shake_permille\": 0"));
    assert!(settings.contains("\"color_only_information\": false"));
    assert!(settings.contains("\"hardware_gamepad_smoke_claimed\": false"));
    assert!(settings.contains("\"owner_visual_accepted\": false"));
    assert!(report.contains("Status: PASSED"));
    assert!(report.contains("Truth mutation: `none`"));
    assert!(report.contains("Critical audio visual equivalent: `true`"));
    assert!(report.contains("Gamepad hardware smoke claimed: `false`"));
}

#[test]
fn runtime_settings_roundtrip_persists_without_truth_mutation() {
    let root = std::path::Path::new("target/tmp/oathyard_runtime_settings_test");
    write_runtime_settings_artifacts(root).expect("runtime settings artifacts");

    let saved = fs::read_to_string(root.join("runtime_settings.saved.json")).expect("saved json");
    let loaded =
        fs::read_to_string(root.join("runtime_settings.loaded.json")).expect("loaded json");
    let report =
        fs::read_to_string(root.join("runtime_settings_report.md")).expect("settings report");
    assert_eq!(saved, loaded);
    assert!(saved.contains("\"schema\": \"oathyard.runtime_settings.v1\""));
    assert!(saved.contains("\"presentation_only\": true"));
    assert!(saved.contains("\"truth_mutation\": false"));
    assert!(saved.contains("\"replay_hash_affects\": false"));
    assert!(saved.contains("\"uses_wall_clock\": false"));
    assert!(saved.contains("\"hidden_rng\": false"));
    assert!(saved.contains("\"text_scale_permille\": 1400"));
    assert!(saved.contains("\"master_gain_permille\": 720"));
    assert!(saved.contains("\"hold_to_commit\": true"));
    assert!(saved.contains("\"toggle_guard\": true"));
    assert!(report.contains("Status: PASSED"));
    assert!(report.contains("Roundtrip byte exact: `true`"));
    assert!(report.contains("Truth mutation: `none`"));
    assert!(report.contains("Replay hash affects: `false`"));
}

#[test]
fn runtime_asset_manifest_references_valid_local_gltf_outputs() {
    let _asset_lock = acquire_asset_generation_test_lock();
    let asset_pipeline = Command::new("python3")
        .args(["tools/asset_pipeline.py", "build"])
        .output()
        .expect("run asset pipeline for cargo-test clean checkout asset outputs");
    assert!(
        asset_pipeline.status.success(),
        "asset pipeline failed for cargo-test clean checkout asset outputs\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&asset_pipeline.stdout),
        String::from_utf8_lossy(&asset_pipeline.stderr)
    );

    let manifest = fs::read_to_string("assets/runtime_manifest.json").expect("runtime manifest");
    let fighter_gltf =
        fs::read_to_string("assets/gltf/saltreach_duelist.gltf").expect("fighter gltf");
    let weapon_gltf = fs::read_to_string("assets/gltf/longsword.gltf").expect("weapon gltf");
    let material_manifest = fs::read_to_string("assets/materials/pbr_surface_manifest.json")
        .expect("pbr material manifest");
    let gltf_report =
        fs::read_to_string("assets/gltf_validation_report.md").expect("gltf validation report");

    assert!(manifest.contains("\"runtime_gltf\": \"assets/gltf/longsword.gltf\""));
    assert!(manifest.contains("\"runtime_gltf\": \"assets/gltf/oathyard_verdict_ring.gltf\""));
    assert!(fighter_gltf.contains("\"version\": \"2.0\""));
    assert!(fighter_gltf.contains("\"truth_authoritative\": false"));
    assert!(fighter_gltf.contains("\"canonical_truth_joints\""));
    assert!(fighter_gltf.contains("\"grip_r\""));
    assert!(fighter_gltf.contains("\"grip_l\""));
    assert!(fighter_gltf.contains("\"pbr_material_schema\": \"oathyard.pbr_surface.v1\""));
    assert!(weapon_gltf.contains("tempered_steel_edge_worn"));
    assert!(weapon_gltf.contains("\"uri\": \"data:application/octet-stream;base64,"));
    assert!(manifest
        .contains("\"pbr_material_manifest\": \"assets/materials/pbr_surface_manifest.json\""));
    assert!(manifest.contains("\"pbr_all_required_channels_covered\": true"));
    assert!(material_manifest.contains("\"schema\": \"oathyard.pbr_surface_manifest.v1\""));
    assert!(material_manifest.contains("\"all_required_channels_covered\": true"));
    assert!(material_manifest.contains("steel_scratches"));
    assert!(material_manifest.contains("hair_skin_variation"));
    assert!(gltf_report.contains("Status: PASSED"));
    assert!(
        gltf_report.contains("glTF material extras reference source-backed PBR material profiles")
    );
    assert!(gltf_report.contains("External Khronos validator: unavailable"));
}

#[test]
fn high_detail_presentation_manifest_is_validated_and_loud_fail() {
    let _asset_lock = acquire_asset_generation_test_lock();
    let integration = Command::new("python3")
        .args([
            "tools/model_candidates/integrate_t73291be5_presentation.py",
            "build",
        ])
        .output()
        .expect("run presentation integration for cargo-test clean checkout");
    assert!(
        integration.status.success(),
        "presentation integration failed for cargo-test clean checkout\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&integration.stdout),
        String::from_utf8_lossy(&integration.stderr)
    );
    let production_visual = Command::new("python3")
        .args(["tools/production_visual_manifest.py"])
        .output()
        .expect("write production visual manifest for cargo-test clean checkout");
    assert!(
        production_visual.status.success(),
        "production visual manifest failed for cargo-test clean checkout\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&production_visual.stdout),
        String::from_utf8_lossy(&production_visual.stderr)
    );

    let manifest_path = Path::new("assets/presentation_manifest.json");
    let manifest = fs::read_to_string(manifest_path).expect("presentation manifest");
    let production_manifest = fs::read_to_string("assets/production_visual_manifest.json")
        .expect("production visual manifest");
    let production_candidate_manifest =
        fs::read_to_string("assets/production_candidate_visual_manifest.json")
            .expect("production candidate visual manifest");

    assert!(manifest.contains("\"schema\": \"oathyard.presentation_assets.v1\""));
    assert!(manifest.contains("\"candidate_run_id\": \"t_73291be5\""));
    assert!(
        manifest.contains("\"runtime_gltf\": \"assets/presentation_gltf/saltreach_duelist.gltf\"")
    );
    assert!(manifest.contains("\"runtime_gltf\": \"assets/presentation_gltf/longsword.gltf\""));
    assert!(manifest.contains("\"presentation_only\": true"));
    assert!(manifest.contains("\"truth_authoritative\": false"));
    assert!(manifest.contains("\"truth_mutation\": false"));
    assert!(manifest.contains("\"public_demo_ready\": false"));
    assert!(manifest.contains("\"owner_visual_acceptance\": false"));
    assert!(manifest.contains("\"external_khronos_validation_claimed\": false"));
    assert!(manifest.contains("\"total_triangles\": 161172"));
    assert!(manifest.contains("\"schema\": \"oathyard.asset_toolchain.v1\""));
    assert!(manifest.contains("\"schema\": \"oathyard.production_asset_pipeline_validation.v1\""));
    assert!(manifest.contains("\"production_validation_passed\": true"));
    assert!(manifest.contains("\"schema\": \"oathyard.production_contact_profile.v1\""));
    assert!(manifest.contains("\"schema\": \"oathyard.production_material_validation.v1\""));
    assert!(manifest.contains("\"schema\": \"oathyard.production_rig_validation.v1\""));
    assert!(manifest.contains("\"schema\": \"oathyard.production_capture_validation.v1\""));
    assert!(manifest.contains("\"source_hash\":"));
    assert!(manifest.contains("\"license_status\":"));
    assert!(manifest.contains("\"tool_hashes\":"));
    assert!(manifest.contains("\"runtime_mesh_hash\":"));
    assert!(manifest.contains("\"preview_hash\":"));
    assert!(manifest.contains("\"capture_hashes\":"));
    assert!(manifest.contains("\"contact_geometry\": \"edge_pierce_cross\""));
    assert!(manifest.contains("\"canonical_truth_joint_count\": 16"));
    assert!(manifest.contains("\"backend\": \"deterministic_software_product_capture\""));
    assert!(production_manifest.contains("\"schema\": \"oathyard.production_visual_assets.v1\""));
    assert!(production_manifest.contains("\"production_assets_complete\": false"));
    assert!(production_manifest.contains("\"production_renderer_complete\": false"));
    assert!(production_manifest.contains("\"owner_visual_acceptance\": false"));
    assert!(production_manifest.contains(
        "\"production_candidate_manifest\": \"assets/production_candidate_visual_manifest.json\""
    ));
    assert!(production_manifest.contains("\"entry_count\": 0"));
    assert!(production_manifest.contains("\"entries\": []"));
    assert!(production_candidate_manifest
        .contains("\"schema\": \"oathyard.production_candidate_visual_assets.v1\""));
    assert!(
        production_candidate_manifest.contains("\"production_candidate_assets_complete\": true")
    );
    assert!(production_candidate_manifest.contains("\"source_file\":"));
    assert!(production_candidate_manifest.contains("\"provenance_license\":"));
    assert!(production_candidate_manifest.contains("\"authoring_process\":"));
    assert!(production_candidate_manifest.contains("\"runtime_export\":"));
    assert!(production_candidate_manifest.contains("\"in_engine_screenshot\":"));
    assert!(production_candidate_manifest.contains("\"validation_result\":"));
    assert!(production_candidate_manifest.contains("\"armor_sockets\":"));
    assert!(production_candidate_manifest.contains("\"contact_geometry\":"));
    assert!(production_candidate_manifest.contains("\"collision_footing_metadata\":"));

    let structural_assets = Command::new("./tools/build_assets.sh")
        .output()
        .expect("build structural assets before local asset validation");
    assert!(
        structural_assets.status.success(),
        "structural asset build failed before local validation\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&structural_assets.stdout),
        String::from_utf8_lossy(&structural_assets.stderr)
    );

    let validate_out = Path::new("target/tmp/oathyard_asset_validation_local_gate_test");
    if validate_out.exists() {
        fs::remove_dir_all(validate_out).expect("clear old validate-assets output");
    }
    let validation = Command::new("./tools/validate_assets.sh")
        .arg(validate_out)
        .output()
        .expect("run local asset validation gate");
    assert!(
        validation.status.success(),
        "local asset validation must pass while final high-fidelity production assets remain blocked\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&validation.stdout),
        String::from_utf8_lossy(&validation.stderr)
    );
    let validation_manifest =
        fs::read_to_string(validate_out.join("production_asset_validation_manifest.json"))
            .expect("production asset validation manifest");
    assert!(validation_manifest.contains("\"local_asset_gate_passed\": true"));
    assert!(validation_manifest.contains("\"production_assets_complete\": false"));
    assert!(
        validation_manifest.contains("\"production_candidate_manifest_claimed_complete\": true")
    );
    assert!(validation_manifest.contains("\"high_fidelity_production_gate_passed\": false"));
    assert!(validation_manifest.contains("\"owner_visual_acceptance\": false"));
    assert!(validation_manifest.contains("\"public_demo_ready\": false"));
    assert!(validation_manifest.contains("\"release_candidate_ready\": false"));
    assert!(validation_manifest
        .contains("t_73291be5 model candidates are production-candidate evidence"));

    let previews_out = Path::new("target/tmp/oathyard_asset_previews_local_gate_test");
    if previews_out.exists() {
        fs::remove_dir_all(previews_out).expect("clear old render-asset-previews output");
    }
    let previews = Command::new("./tools/render_asset_previews.sh")
        .arg(previews_out)
        .output()
        .expect("run local asset preview gate");
    assert!(
        previews.status.success(),
        "local asset preview gate must pass while final high-fidelity preview assets remain blocked\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&previews.stdout),
        String::from_utf8_lossy(&previews.stderr)
    );
    let preview_manifest = fs::read_to_string(previews_out.join("asset_preview_manifest.json"))
        .expect("asset preview manifest");
    assert!(preview_manifest.contains("\"passed\": true"));
    assert!(preview_manifest.contains("\"local_preview_gate_passed\": true"));
    assert!(preview_manifest.contains("\"failed_check_count\": 0"));
    assert!(preview_manifest.contains("\"production_candidate_previews_present\": true"));
    assert!(preview_manifest.contains("\"production_asset_previews_complete\": false"));
    assert!(preview_manifest.contains("\"high_fidelity_production_preview_gate_passed\": false"));
    assert!(preview_manifest.contains("\"production_preview_blockers\":"));
    assert!(
        preview_manifest.contains("t_73291be5 previews/captures are production-candidate evidence")
    );

    let runtime_mesh = fs::read_to_string("assets/presentation_runtime/longsword.mesh.json")
        .expect("presentation runtime mesh");
    assert!(runtime_mesh.contains("\"schema\": \"oathyard.presentation_runtime_asset.v1\""));
    assert!(runtime_mesh.contains("\"schema\": \"oathyard.production_contact_profile.v1\""));
    assert!(runtime_mesh.contains("\"production_validation_passed\": true"));

    let report = fs::read_to_string(
        "artifacts/model_candidates/t_73291be5/presentation_integration/presentation_asset_integration_report.md",
    )
    .expect("presentation integration report");
    assert!(report.contains("Status: PASSED"));
    assert!(report.contains("Production validation: `true`"));
    assert!(report.contains("Source/provenance/license/toolchain/runtime-hash/preview/in-engine/contact/material/rig validation is fail-closed"));
}

#[test]
fn pbr_material_artifacts_cover_surfaces_events_and_keep_truth_hashes_stable() {
    let root = std::path::Path::new("target/tmp/oathyard_pbr_materials_test");
    let result = write_pbr_material_artifacts("examples/duels/basic_oathyard.duel", root)
        .expect("pbr material artifacts");

    let manifest =
        fs::read_to_string(root.join("pbr_material_manifest.json")).expect("pbr material manifest");
    let report =
        fs::read_to_string(root.join("pbr_material_report.md")).expect("pbr material report");

    assert!(manifest.contains("\"schema\": \"oathyard.pbr_material_artifacts.v1\""));
    assert!(manifest.contains("\"surface_schema\": \"oathyard.pbr_surface.v1\""));
    assert!(manifest.contains("\"event_schema\": \"oathyard.pbr_material_event.v1\""));
    assert!(manifest.contains("\"source\": \"verified-replay-after-truth-hash\""));
    assert!(manifest.contains("\"replay_verified\": true"));
    assert!(manifest.contains("\"truth_mutation\": false"));
    assert!(manifest.contains("\"material_maps_affect_replay_hash\": false"));
    assert!(manifest.contains("\"all_required_channels_covered\": true"));
    assert!(manifest.contains("\"flat_recolor_rejected\": true"));
    assert!(manifest.contains("\"surface_count\": 8"));
    assert!(manifest.contains("\"material_result_count\": 6"));
    for channel in [
        "albedo",
        "roughness_metallic",
        "normal_height",
        "edge_wear",
        "blood_wetness",
        "cloth_grain",
        "steel_scratches",
        "leather_strain",
        "stone_dust",
        "stitching",
        "hair_skin_variation",
    ] {
        assert!(
            manifest.contains(&format!("\"channel\": \"{channel}\", \"covered\": true")),
            "missing channel {channel}"
        );
    }
    for expected in [
        "tempered_steel_edge_worn",
        "quilted_linen_stitched",
        "wet_blood_trace_overlay",
        "mail_absorbed_edge_with_blunt_transfer",
        "gap_penetration_with_binding",
        "hook_bind_torque_loss",
        "deflected_with_posture_shock",
        "blunt_transfer_stance_break",
        "low_coverage_blunt_transfer",
    ] {
        assert!(
            manifest.contains(expected),
            "missing material evidence {expected}"
        );
    }
    assert!(manifest.contains(&format!(
        "\"disabled_final_state_hash\": \"{}\"",
        result.final_state_hash
    )));
    assert!(manifest.contains(&format!(
        "\"enabled_final_state_hash\": \"{}\"",
        result.final_state_hash
    )));
    assert!(root.join("pbr_material_surface_atlas.ppm").is_file());
    assert!(root.join("pbr_material_response_sheet.ppm").is_file());
    assert!(report.contains("Status: PASSED"));
    assert!(report.contains("Truth mutation: `none`"));
    assert!(report.contains("Flat recolor rejected: `true`"));
}

#[test]
fn contact_matrix_covers_shipped_weapons_armor_actions_and_targets() {
    let root = std::path::Path::new("target/tmp/oathyard_contact_matrix_test");
    write_contact_matrix_artifacts(root).expect("contact matrix artifacts");

    let matrix = fs::read_to_string(root.join("contact_matrix.json")).expect("contact matrix json");
    let report =
        fs::read_to_string(root.join("contact_matrix_report.md")).expect("contact matrix report");

    assert!(matrix.contains("\"schema\": \"oathyard.contact_matrix.v1\""));
    assert!(matrix.contains("\"weapons\": 8"));
    assert!(matrix.contains("\"armors\": 6"));
    assert!(matrix.contains("\"attack_labels\": 7"));
    assert!(matrix.contains("\"targets\": 4"));
    assert!(matrix.contains("\"combinations\": 1344"));
    assert!(matrix.contains("\"contacts\": 1344"));
    assert!(matrix.contains("\"invalid_actions\": 0"));
    assert!(matrix.contains("\"invariants_passed\": true"));
    assert!(matrix.contains("mail_absorbed_edge_with_blunt_transfer"));
    assert!(matrix.contains("gap_penetration_with_binding"));
    assert!(matrix.contains("hook_bind_torque_loss"));
    assert!(matrix.contains("blunt_transfer_stance_break"));
    assert!(matrix.contains("\"torque_delta\":"));
    assert!(matrix.contains("\"invalidates_thrust\": true"));
    assert!(matrix.contains("mail_cut_blunt_transfer_slows_recovery"));
    assert!(matrix.contains("weapon_arm_gap_penetration_compromises_grip"));
    assert!(matrix.contains("hook_bind_reduces_torque_and_grip"));
    assert!(matrix.contains("blunt_transfer_breaks_stance"));
    assert!(matrix.contains("deflection_still_applies_posture_shock"));
    assert!(matrix.contains("low_coverage_transfers_capability_loss"));
    assert!(matrix.contains("physical_costs_vary_from_base"));
    assert!(matrix.contains("cause_chain"));
    assert!(report.contains("Status: PASSED"));
    assert!(report.contains("Weapons covered: `8`"));
    assert!(report.contains("Contacts generated: `1344`"));
    assert!(report.contains("## Invariants"));
    assert!(report.contains("weapon-arm gap penetration applies severe right-grip loss"));
    assert!(report.contains("hook/bind results reduce weapon-side torque"));
}

#[test]
fn deterministic_ai_duel_emits_legal_seedless_plan_and_replay() {
    let first_root = std::path::Path::new("target/tmp/oathyard_ai_duel_test_a");
    let second_root = std::path::Path::new("target/tmp/oathyard_ai_duel_test_b");
    let first = write_ai_duel_artifacts(first_root, 6).expect("first ai duel");
    let second = write_ai_duel_artifacts(second_root, 6).expect("second ai duel");

    assert_eq!(first.final_state_hash, second.final_state_hash);
    assert_eq!(first.replay_json, second.replay_json);
    assert_eq!(first.trace_json, second.trace_json);
    assert!(first.turns.iter().any(|turn| !turn.contacts.is_empty()));
    assert!(first
        .turns
        .iter()
        .flat_map(|turn| turn.costs.iter())
        .all(|cost| cost.action_valid));

    let plan = fs::read_to_string(first_root.join("ai_plan.json")).expect("ai plan json");
    let report = fs::read_to_string(first_root.join("ai_plan_report.md")).expect("ai plan report");

    assert!(plan.contains("\"schema\": \"oathyard.ai_plan.v1\""));
    assert!(plan.contains("\"hidden_rng\": false"));
    assert!(plan.contains("\"wall_clock\": false"));
    assert!(plan.contains("\"difficulty_changes_body_stats\": false"));
    assert!(plan.contains("\"legal_actions\": true"));
    assert!(plan.contains("\"all_truth_actions_valid\": true"));
    assert!(plan.contains("\"planner_reason\""));
    assert!(plan.contains("\"outcome_authority\": \"truth_replay_only\""));
    assert!(first_root.join("ai_scenario.duel").is_file());
    assert!(first_root.join("replay.json").is_file());
    assert!(report.contains("Status: PASSED"));
    assert!(report.contains("Truth Cause Chains"));
}

#[test]
fn deterministic_ai_sweep_covers_multiple_pairings_and_repeats_stably() {
    let root = std::path::Path::new("target/tmp/oathyard_ai_sweep_test");
    write_ai_sweep_artifacts(root).expect("ai sweep");

    let sweep = fs::read_to_string(root.join("ai_sweep.json")).expect("ai sweep json");
    let report = fs::read_to_string(root.join("ai_sweep_report.md")).expect("ai sweep report");

    assert!(sweep.contains("\"schema\": \"oathyard.ai_sweep.v1\""));
    assert!(sweep.contains("\"runs_per_pairing\": 2"));
    assert!(sweep.contains("\"pairing_count\": 6"));
    assert!(sweep.contains("\"total_contacts\": 37"));
    assert!(sweep.contains("\"capability_reaction_count\": 58"));
    assert!(sweep.contains("\"distinct_action_labels\": 11"));
    assert!(sweep.contains("\"policy_style_count\": 6"));
    assert!(sweep.contains("\"unique_final_hashes\": 6"));
    assert!(sweep.contains("\"all_pairings_stable\": true"));
    assert!(sweep.contains("\"all_replays_verified\": true"));
    assert!(sweep.contains("\"all_actions_legal\": true"));
    assert!(sweep.contains("\"all_truth_actions_valid\": true"));
    assert!(sweep.contains("\"difficulty_changes_body_stats\": false"));
    assert!(sweep.contains("\"body_stat_mutation_by_ai\": false"));
    assert!(sweep.contains("\"outcome_authority\": \"truth_replay_only\""));
    assert!(sweep.contains("\"id\": \"reach_vs_mail\""));
    assert!(sweep.contains("\"id\": \"hook_vs_plate\""));
    assert!(sweep.contains("\"id\": \"maul_vs_fencer\""));
    assert!(sweep.contains("\"id\": \"curve_vs_spear_lamellar\""));
    assert!(sweep.contains("\"id\": \"shield_counter_vs_curve\""));
    assert!(sweep.contains("\"id\": \"spear_vs_maul_pressure\""));
    assert!(sweep.contains("\"policy_0\": \"guard_counter\""));
    assert!(sweep.contains("\"policy_1\": \"reach_pressure\""));
    assert!(sweep.contains("\"policy_0\": \"bind_control\""));
    assert!(sweep.contains("\"policy_0\": \"heavy_pressure\""));
    assert!(sweep.contains("\"policy_1\": \"evasive_counter\""));
    assert!(sweep.contains("\"policy_0\": \"low_line_disruptor\""));
    assert!(sweep.contains("\"stable_committed_sequences\": true"));
    assert!(sweep.contains("\"stable_replay\": true"));
    assert!(sweep.contains("\"stable_trace\": true"));
    assert!(sweep.contains("\"end_condition_status\""));
    assert!(sweep.contains("\"repeat_end_condition_status\""));
    assert!(sweep.contains("\"end_condition_winner\""));
    assert!(sweep.contains("\"repeat_end_condition_winner\""));
    assert!(sweep.contains("\"seat_1_victory_capability_stop\""));
    assert!(sweep.contains("\"action\": \"hook_bind\""));
    assert!(sweep.contains("\"action\": \"parry\""));
    assert!(sweep.contains("\"action\": \"shove\""));
    assert!(sweep.contains("\"action\": \"kick\""));
    assert!(root.join("hook_vs_plate/run_a/replay.json").is_file());
    assert!(root.join("hook_vs_plate/run_b/replay.json").is_file());
    assert!(root
        .join("shield_counter_vs_curve/run_a/replay.json")
        .is_file());
    assert!(root
        .join("spear_vs_maul_pressure/run_b/replay.json")
        .is_file());
    assert!(report.contains("Status: PASSED"));
    assert!(report.contains("## Action Coverage"));
    assert!(report.contains("## Policy Styles"));
    assert!(report.contains("end condition `seat_1_victory_capability_stop`"));
}

#[test]
fn truth_stress_runs_long_replay_traces_stably() {
    let root = std::path::Path::new("target/tmp/oathyard_truth_stress_test");
    write_truth_stress_artifacts(root).expect("truth stress artifacts");

    let stress = fs::read_to_string(root.join("truth_stress.json")).expect("truth stress json");
    let report = fs::read_to_string(root.join("truth_stress_report.md")).expect("truth report");

    assert!(stress.contains("\"schema\": \"oathyard.truth_stress.v1\""));
    assert!(stress.contains("\"truth_hz\": 120"));
    assert!(stress.contains("\"hidden_rng\": false"));
    assert!(stress.contains("\"wall_clock\": false"));
    assert!(stress.contains("\"gameplay_floats\": false"));
    assert!(stress.contains("\"runs_per_pairing\": 2"));
    assert!(stress.contains("\"pairing_count\": 6"));
    assert!(stress.contains("\"stress_turn_count\": 24"));
    assert!(stress.contains("\"minimum_total_contacts_required\": 72"));
    assert!(stress.contains("\"minimum_capability_reactions_required\": 150"));
    assert!(stress.contains("\"minimum_capability_stops_required\": 4"));
    assert!(stress.contains("\"minimum_distinct_final_hashes_required\": 5"));
    assert!(stress.contains("\"minimum_recovery_slowdown_required\": 32"));
    assert!(stress.contains("\"maximum_min_balance_required\": 100"));
    assert!(stress.contains("\"maximum_min_grip_r_required\": 100"));
    assert!(stress.contains("\"maximum_min_torque_required\": 100"));
    assert!(stress.contains("\"distinct_final_hash_count\":"));
    assert!(stress.contains("\"max_recovery_slowdown_frames\":"));
    assert!(stress.contains("\"min_balance_permille\":"));
    assert!(stress.contains("\"min_grip_r_permille\":"));
    assert!(stress.contains("\"min_torque_permille\":"));
    assert!(stress.contains("\"stress_thresholds_passed\": true"));
    assert!(stress.contains("\"all_stress_cases_stable\": true"));
    assert!(stress.contains("\"all_contact_packets_ordered\": true"));
    assert!(stress.contains("\"all_turn_hash_chains_stable\": true"));
    assert!(stress.contains("\"capability_stop_count\":"));
    assert!(stress.contains("\"contact_order_rule\": \"frame_then_attacker_then_defender_then_action_then_target_then_direction\""));
    assert!(stress.contains("\"outcome_authority\": \"truth_replay_only\""));
    assert!(stress.contains("\"stable_turn_hash_chain\": true"));
    assert!(stress.contains("\"capability_stop_end_condition\": true"));
    assert!(root
        .join("spear_vs_maul_pressure/run_a/replay.json")
        .is_file());
    assert!(root
        .join("spear_vs_maul_pressure/run_b/replay.json")
        .is_file());
    assert!(report.contains("Status: PASSED"));
    assert!(report.contains("Stress turn count: `24`"));
    assert!(report.contains("Capability-stop end conditions:"));
    assert!(report.contains("Adversarial thresholds passed: `true`"));
    assert!(report.contains("## Adversarial Thresholds"));
    assert!(report.contains(
        "Contact order rule: `frame_then_attacker_then_defender_then_action_then_target_then_direction`"
    ));
}

#[test]
fn truth_edge_audit_covers_overflow_clamps_and_replay_compatibility() {
    let root = std::path::Path::new("target/tmp/oathyard_truth_edge_audit_test");
    write_truth_edge_audit_artifacts(root).expect("truth edge audit artifacts");

    let audit = fs::read_to_string(root.join("truth_edge_audit.json")).expect("edge audit json");
    let report =
        fs::read_to_string(root.join("truth_edge_audit_report.md")).expect("edge audit report");

    assert!(audit.contains("\"schema\": \"oathyard.truth_edge_audit.v1\""));
    assert!(audit.contains("\"truth_hz\": 120"));
    assert!(audit.contains("\"fixed_point_scale\": 1000"));
    assert!(audit.contains("\"overflow_policy\": \"i128_intermediate_then_saturate_or_clamp\""));
    assert!(audit.contains("\"hidden_rng\": false"));
    assert!(audit.contains("\"wall_clock\": false"));
    assert!(audit.contains("\"gameplay_floats\": false"));
    assert!(audit.contains("\"unordered_truth_iteration\": false"));
    assert!(audit.contains("\"all_edge_cases_passed\": true"));
    assert!(audit.contains("\"id\": \"permille_positive_overflow_saturates\""));
    assert!(audit.contains("\"id\": \"fixed_ratio_zero_denominator_saturates\""));
    assert!(audit.contains("\"id\": \"capability_lower_clamp_and_validity\""));
    assert!(audit.contains("\"id\": \"contact_tie_breaker_signature\""));
    assert!(audit.contains("\"id\": \"unsupported_schema_fails_loud\""));
    assert!(audit.contains("\"id\": \"missing_required_field_fails_loud\""));
    assert!(audit.contains("\"id\": \"mismatched_final_hash_fails_loud\""));
    assert!(audit.contains("\"id\": \"missing_end_condition_status_fails_loud\""));
    assert!(audit.contains("\"id\": \"missing_end_condition_winner_fails_loud\""));
    assert!(audit.contains("\"id\": \"truth_hz_mismatch_fails_loud\""));
    assert!(audit.contains("\"id\": \"truth_hz_malformed_fails_loud\""));
    assert!(audit.contains("verification error: replay schema mismatch"));
    assert!(audit.contains("verification error: replay missing scenario_canonical"));
    assert!(audit.contains("verification error: final state hash mismatch"));
    assert!(audit.contains("verification error: replay missing end_condition_status"));
    assert!(audit.contains("verification error: replay missing end_condition_winner"));
    assert!(audit.contains("verification error: replay truth_hz mismatch"));
    assert!(audit.contains("verification error: replay truth_hz"));
    assert!(report.contains("Status: PASSED"));
    assert!(report.contains("Overflow policy: `i128_intermediate_then_saturate_or_clamp`"));
    assert!(report.contains("replay `unsupported_schema_fails_loud`"));
    assert!(report.contains("replay `missing_end_condition_status_fails_loud`"));
    assert!(report.contains("replay `missing_end_condition_winner_fails_loud`"));
    assert!(report.contains("replay `truth_hz_mismatch_fails_loud`"));
    assert!(report.contains("replay `truth_hz_malformed_fails_loud`"));
}

#[test]
fn negative_input_audit_rejects_bad_scenarios_replays_content_and_bundles() {
    let root = std::path::Path::new("target/tmp/oathyard_negative_input_audit_test");
    write_negative_input_audit_artifacts(root).expect("negative input audit artifacts");

    let audit = fs::read_to_string(root.join("negative_input_audit.json"))
        .expect("negative input audit json");
    let report = fs::read_to_string(root.join("negative_input_audit_report.md"))
        .expect("negative input audit report");

    assert!(audit.contains("\"schema\": \"oathyard.negative_input_audit.v1\""));
    assert!(audit.contains("\"truth_hz\": 120"));
    assert!(audit.contains("\"case_count\": 13"));
    assert!(audit.contains("\"all_failed_loudly\": true"));
    assert!(audit.contains("\"all_cases_passed\": true"));
    assert!(audit.contains("\"public_demo_ready\": false"));
    assert!(audit.contains("\"release_candidate_ready\": false"));
    for case_id in [
        "scenario_missing_id_fails_loud",
        "scenario_missing_fighter_fails_loud",
        "scenario_unknown_action_fails_loud",
        "scenario_duplicate_turn_seat_fails_loud",
        "scenario_unknown_target_fails_loud",
        "scenario_unknown_weapon_fails_loud",
        "content_manifest_schema_fails_loud",
        "content_manifest_readiness_true_fails_loud",
        "content_manifest_missing_rows_fails_loud",
        "replay_unsupported_schema_fails_loud",
        "replay_missing_scenario_fails_loud",
        "replay_mismatched_final_hash_fails_loud",
        "export_bundle_tamper_fails_loud",
    ] {
        assert!(audit.contains(case_id), "missing case id {case_id}");
    }
    for expected in [
        "unknown action label",
        "scenario missing fighter seat 1",
        "content manifest schema mismatch",
        "public_demo_ready must remain false",
        "fighters count 0 below required 6",
        "replay schema mismatch",
        "replay missing scenario_canonical",
        "final state hash mismatch",
        "export bundle hash mismatch",
    ] {
        assert!(
            audit.contains(expected),
            "missing expected failure {expected}"
        );
    }
    assert!(report.contains("Status: PASSED"));
    assert!(report.contains("All failed loudly: `true`"));
    assert!(report.contains("export_bundle_tamper_fails_loud"));
}

#[test]
fn animation_state_machine_is_presentation_only_and_truth_stable() {
    let root = std::path::Path::new("target/tmp/oathyard_animation_state_machine_test");
    let result =
        write_animation_state_machine_artifacts("examples/duels/basic_oathyard.duel", root)
            .expect("animation state machine artifacts");

    let baseline = run_scenario_text(BASIC).expect("baseline");
    assert_eq!(
        result.final_state_hash, baseline.final_state_hash,
        "animation state machine must not mutate truth final state hash"
    );
    assert_eq!(
        result.replay_json, baseline.replay_json,
        "animation state machine must not mutate replay JSON"
    );
    assert_eq!(
        result.trace_json, baseline.trace_json,
        "animation state machine must not mutate trace JSON"
    );

    let manifest = fs::read_to_string(root.join("animation_state_machine_manifest.json"))
        .expect("manifest json");
    let sequence =
        fs::read_to_string(root.join("animation_state_sequence.json")).expect("sequence json");
    let retargeting = fs::read_to_string(root.join("animation_retargeting_bridge.json"))
        .expect("retargeting json");
    let report =
        fs::read_to_string(root.join("animation_state_machine_report.md")).expect("report md");

    assert!(manifest.contains("\"schema\": \"oathyard.animation_state_machine.v1\""));
    assert!(manifest.contains("\"layer\": \"runtime_presentation\""));
    assert!(manifest.contains("\"presentation_only\": true"));
    assert!(manifest.contains("\"truth_mutation\": false"));
    assert!(manifest.contains("\"owner_visual_acceptance\": false"));

    for state in [
        "observe",
        "plan",
        "step",
        "pivot",
        "guard",
        "parry",
        "cut",
        "thrust",
        "brace",
        "bash",
        "hook_bind",
        "grab",
        "shove",
        "kick",
        "recover",
    ] {
        assert!(
            manifest.contains(&format!("\"state\": \"{state}\"")),
            "missing state {state}"
        );
    }
    for reaction in ["bind", "stagger", "collapse", "injury", "recovery"] {
        assert!(
            manifest.contains(&format!("\"reaction\": \"{reaction}\"")),
            "missing reaction {reaction}"
        );
    }

    assert!(manifest.contains("\"may_predecide_contact\": false"));
    assert!(manifest.contains("\"may_predecide_injury\": false"));
    assert!(manifest.contains("\"may_modify_action_cost\": false"));

    assert!(manifest.contains("\"truth_joint_count\": 16"));
    assert!(manifest.contains("\"consumes_truth_joints_after_hash\": true"));

    assert!(sequence.contains("\"presentation_only\": true"));
    assert!(sequence.contains("\"truth_mutation\": false"));
    assert!(sequence.contains("\"source\": \"truth-after-hash-committed-actions-and-contacts\""));

    assert!(retargeting.contains("\"schema\": \"oathyard.animation_retargeting_bridge.v1\""));
    assert!(retargeting.contains("\"input_boundary\": \"truth_joints_after_hash\""));
    assert!(retargeting.contains("\"grip_r\""));
    assert!(retargeting.contains("\"grip_l\""));

    assert!(root.join("animation_contact_sheet.ppm").is_file());
    assert!(report.contains("Status: PASSED"));
    assert!(report.contains("Truth mutation: `none`"));
    assert!(report.contains("Layer: `runtime_presentation`"));
}

#[test]
fn presentation_bricks_layer_is_motionbricks_inspired_and_truth_stable() {
    let root = std::path::Path::new("target/tmp/oathyard_presentation_bricks_test");
    let result = write_presentation_bricks_artifacts("examples/duels/basic_oathyard.duel", root)
        .expect("presentation bricks artifacts");

    let baseline = run_scenario_text(BASIC).expect("baseline");
    assert_eq!(
        result.final_state_hash, baseline.final_state_hash,
        "PresentationBricks must not mutate truth final state hash"
    );
    assert_eq!(
        result.replay_json, baseline.replay_json,
        "PresentationBricks must not mutate replay JSON"
    );
    assert_eq!(
        result.trace_json, baseline.trace_json,
        "PresentationBricks must not mutate trace JSON"
    );

    let manifest = fs::read_to_string(root.join("presentation_bricks_manifest.json"))
        .expect("presentation bricks manifest");
    let sequence = fs::read_to_string(root.join("presentation_bricks_sequence.json"))
        .expect("presentation bricks sequence");
    let retargeting = fs::read_to_string(root.join("presentation_bricks_retargeting_bridge.json"))
        .expect("presentation bricks retargeting");
    let report = fs::read_to_string(root.join("presentation_bricks_report.md"))
        .expect("presentation bricks report");

    assert!(manifest.contains("\"schema\": \"oathyard.presentation_bricks.v1\""));
    assert!(manifest.contains("\"layer\": \"runtime_presentation\""));
    assert!(manifest.contains("\"motion_system\": \"MotionBricks-inspired PresentationBricks\""));
    assert!(manifest.contains("\"named_vendor_integration_claimed\": false"));
    assert!(manifest.contains("\"actual_motionbricks_sdk_access_verified\": false"));
    assert!(manifest.contains("\"consumes_truth_poses_after_hash\": true"));
    assert!(manifest.contains("\"consumes_action_labels\": true"));
    assert!(manifest.contains("\"consumes_contact_events\": true"));
    assert!(manifest.contains("\"consumes_capability_changes\": true"));
    assert!(manifest.contains("\"may_decide_hits\": false"));
    assert!(manifest.contains("\"may_decide_contacts\": false"));
    assert!(manifest.contains("\"may_write_action_costs\": false"));
    assert!(manifest.contains("\"may_write_capability_deltas\": false"));
    assert!(manifest.contains("\"writes_replay_hashes\": false"));

    for primitive in [
        "locomotion",
        "guard_transition",
        "weapon_handling",
        "bind_hook",
        "stumble",
        "fall",
        "collapse",
        "recovery",
        "object_interaction",
        "fight_film_moment",
    ] {
        assert!(
            manifest.contains(&format!("\"primitive\": \"{primitive}\"")),
            "missing PresentationBricks smart primitive {primitive}"
        );
    }

    assert!(sequence.contains("\"source\": \"truth-after-hash-committed-actions-and-contacts\""));
    assert!(sequence.contains("\"presentation_only\": true"));
    assert!(sequence.contains("\"truth_mutation\": false"));
    assert!(retargeting.contains("\"schema\": \"oathyard.presentation_bricks_retargeting.v1\""));
    assert!(retargeting.contains("\"canonical_truth_joint_mapping\": true"));
    assert!(retargeting.contains("\"cosmetic_only_bones_separated_from_truth\": true"));
    assert!(root.join("presentation_bricks_contact_sheet.ppm").is_file());
    assert!(report.contains("Status: PASSED"));
    assert!(report.contains("Motion system: `MotionBricks-inspired PresentationBricks`"));
    assert!(report.contains("Named vendor integration claimed: `false`"));
    assert!(report.contains("Truth mutation: `none`"));
}

#[test]
fn native_render_product_capture_lane_is_clean_and_manifested() {
    let source = fs::read_to_string("src/lib.rs").expect("source");
    let native_tool = fs::read_to_string("tools/native_combat_render.sh").expect("native tool");
    let hifi_tool =
        fs::read_to_string("tools/capture_high_fidelity_screens.sh").expect("hifi tool");

    assert!(
        source.contains("fn draw_native_software_camera_readability_overlay("),
        "missing helper must not leave native software renderer uncompilable"
    );
    assert!(
        source.contains("fn render_native_product_resolution_capture("),
        "native high-resolution captures must use a clean product-mode software framebuffer"
    );
    assert!(source.contains("product_mode_clean_captures"));
    assert!(source.contains("debug_overlay"));
    assert!(source.contains("software-product-renderer-clean-frame-after-truth-hash"));
    assert!(source.contains("verdict_ring_establishing"));
    assert!(source.contains("combat_contact_readability"));
    assert!(source.contains("material_impact_closeup"));
    for camera in [
        "first_person_guard_line",
        "third_person_verdict_ring",
        "planning_tactical_reach",
        "consequence_aftermath_dwell",
        "fight_film_orbit",
        "asset_closeup_weapon_armor",
    ] {
        assert!(
            source.contains(camera),
            "native renderer source must explicitly define six-camera readability mode {camera}"
        );
        assert!(
            native_tool.contains(camera)
                || native_tool.contains(&camera.replace("_tactical_reach", "")),
            "native render tool must assert six-camera readability evidence for {camera}"
        );
    }
    for file in [
        "native_combat_3d_first_person.ppm",
        "native_combat_3d_third_person.ppm",
        "native_combat_3d_planning.ppm",
        "native_combat_3d_consequence.ppm",
        "native_combat_3d_fight_film.ppm",
        "native_combat_3d_asset_closeup.ppm",
        "native_product_fight_film_1920x1080.ppm",
    ] {
        assert!(
            native_tool.contains(file),
            "native render tool must assert six-camera capture artifact {file} exists"
        );
    }
    assert!(source.contains("product_mode_clean_capture_count"));
    assert!(source.contains("native_renderer_assert_truth_unchanged"));
    assert!(source.contains("native_renderer_truth_writeback_audit.md"));
    assert!(source.contains("render_native_renderer_truth_writeback_audit_report"));
    assert!(source.contains("capture_after_truth_hash"));
    assert!(source.contains("verified_replay_trace_input"));
    assert!(source.contains("replay_verified"));
    assert!(source.contains("native_capture_timestamp_ms"));
    assert!(source.contains("write_native_capture_camera_metadata_json"));
    assert!(source.contains("replay_source_hash"));
    assert!(source.contains("timestamp_ms"));
    assert!(source.contains("position_milli"));
    assert!(source.contains("Pre/post truth hashes equal"));
    assert!(source.contains("PRODUCTION_RENDERER_MANIFEST_SCHEMA"));
    assert!(source.contains("fn write_native_production_renderer_bundle("));
    assert!(source.contains("fn render_native_production_renderer_capture("));
    assert!(source.contains("production-renderer-state-frame-from-replay-trace-after-truth-hash"));

    for file in [
        "native_product_verdict_ring_1920x1080.ppm",
        "native_product_contact_1920x1080.ppm",
        "native_product_material_closeup_1920x1080.ppm",
    ] {
        assert!(
            native_tool.contains(file),
            "native render tool must assert product capture {file} exists"
        );
        assert!(
            hifi_tool.contains(file) || hifi_tool.contains("native_product_"),
            "high-fidelity screen reducer must classify product capture {file}"
        );
    }
    assert!(native_tool.contains("native_production_renderer_manifest.json"));
    assert!(native_tool.contains("native_renderer_truth_writeback_audit.md"));
    assert!(native_tool.contains("OATHYARD_NATIVE_COMBAT_OUT"));
    assert!(native_tool.contains("out = Path(os.environ[\"OATHYARD_NATIVE_COMBAT_OUT\"])"));
    assert!(!native_tool.contains("Path(\"$out\""));
    assert!(native_tool.contains("product_mode_clean_captures"));
    assert!(native_tool.contains("required_modes.issubset"));
    assert!(native_tool.contains("cap[\"capture_after_truth_hash\"] is True"));
    assert!(native_tool.contains("cap[\"truth_mutation\"] is False"));
    assert!(native_tool.contains("capture[\"capture_after_truth_hash\"] is True"));
    assert!(native_tool.contains("mutation_proof[\"changed_fields\"] == []"));
    assert!(native_tool.contains("production_renderer_state_capture"));
    assert!(hifi_tool.contains("artifacts/production_renderer/latest"));
    assert!(hifi_tool.contains("production_renderer_manifest_backed"));
    assert!(hifi_tool.contains("production_renderer_"));
    assert!(hifi_tool.contains("debug_local_evidence = False"));
    assert!(hifi_tool.contains("production_asset_evidence = True"));
}

#[test]
fn wgpu_renderer_spike_lane_is_source_buildable_and_truth_isolated() {
    let manifest = std::fs::read_to_string("spikes/wgpu_renderer/Cargo.toml")
        .expect("missing source-buildable wgpu renderer spike manifest");
    let source = std::fs::read_to_string("spikes/wgpu_renderer/src/main.rs")
        .expect("missing wgpu renderer spike source");
    let shader = std::fs::read_to_string("spikes/wgpu_renderer/src/verdict_ring.wgsl")
        .expect("missing wgpu verdict-ring shader");
    let tool = std::fs::read_to_string("tools/wgpu_renderer_spike.sh")
        .expect("missing wgpu renderer spike tool");
    let truth_tool = std::fs::read_to_string("tools/presentation_truth_isolation.sh")
        .expect("missing presentation truth isolation tool");
    let hifi_tool = std::fs::read_to_string("tools/capture_high_fidelity_screens.sh")
        .expect("missing high fidelity capture tool");

    assert!(manifest.contains("wgpu = \"29.0.3\""));
    assert!(manifest.contains("bevy_ecs = \"0.19"));
    assert!(manifest.contains("png = \"0.18.1\""));
    assert!(source.contains("bevy_ecs::world::World"));
    assert!(source.contains("wgpu::Instance::new"));
    assert!(source.contains("request_adapter"));
    assert!(source.contains("PowerPreference::HighPerformance"));
    assert!(source.contains("TextureFormat::Rgba8UnormSrgb"));
    assert!(source.contains("TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC"));
    assert!(source.contains("create_render_pipeline"));
    assert!(source.contains("copy_texture_to_buffer"));
    assert!(source.contains("production_renderer_wgpu_spike_1920x1080.png"));
    assert!(source.contains("post_hash_presentation_packet.json"));
    assert!(source.contains("oathyard.production_renderer_manifest.v1"));
    assert!(source.contains("production_renderer_complete: false"));
    assert!(source.contains("owner_visual_acceptance: false"));
    assert!(source.contains("presentation_truth_isolation_passed: true"));
    assert!(source.contains("truth_mutation: false"));
    assert!(shader.contains("fn raymarch_scene"));
    assert!(shader.contains("fog"));
    assert!(shader.contains("contact_bloom"));
    assert!(tool.contains("./tools/run_duel.sh"));
    assert!(tool.contains("./tools/replay_verify.sh"));
    assert!(tool.contains("cargo run --locked --manifest-path spikes/wgpu_renderer/Cargo.toml"));
    assert!(tool.contains("post_hash_presentation_packet.json"));
    assert!(tool.contains("production_renderer_manifest.json"));
    assert!(tool.contains("presentation_truth_isolation_passed"));
    assert!(tool.contains("\"truth_mutation\": false"));
    assert!(truth_tool.contains("tools/wgpu_renderer_spike.sh"));
    assert!(hifi_tool.contains("path.suffix.lower() == '.png'"));
    assert!(hifi_tool
        .contains("native_3d_production_renderer_evidence = is_non_debug_production_renderer_png"));
}
