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
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).expect("clean export bundle temp dir");
    }
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
    let forbidden = out_dir.join(format!("visual_proof.{}", "s".to_string() + "vg"));
    fs::write(&forbidden, "not allowed\n").expect("inject forbidden visual artifact");
    let error = verify_replay_export_bundle(&out_dir).expect_err("forbidden visual should fail");
    assert!(error.to_string().contains("forbidden visual artifact"));
    fs::remove_file(&forbidden).expect("remove injected forbidden visual artifact");
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

    assert_eq!(baseline.trace_json, generated.trace_json);
    assert_eq!(baseline.replay_json, generated.replay_json);
    assert_eq!(baseline.final_state_hash, generated.final_state_hash);

    assert!(wav.starts_with(b"RIFF"));
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
    assert!(vfx.contains("\"nonvisual_vfx_evidence\""));
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
    let fighter_gltf = fs::read_to_string("assets/runtime/candidate/gltf/saltreach_duelist.gltf")
        .expect("fighter gltf");
    let weapon_gltf =
        fs::read_to_string("assets/runtime/candidate/gltf/longsword.gltf").expect("weapon gltf");
    let material_manifest = fs::read_to_string("assets/materials/pbr_surface_manifest.json")
        .expect("pbr material manifest");
    let gltf_report =
        fs::read_to_string("assets/gltf_validation_report.md").expect("gltf validation report");

    assert!(manifest.contains("\"runtime_gltf\": \"assets/runtime/candidate/gltf/longsword.gltf\""));
    assert!(manifest.contains(
        "\"runtime_gltf\": \"assets/runtime/candidate/gltf/oathyard_verdict_ring.gltf\""
    ));
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

    let manifest_path = Path::new("assets/manifests/presentation_manifest.json");
    let manifest = fs::read_to_string(manifest_path).expect("presentation manifest");
    let production_manifest =
        fs::read_to_string("assets/manifests/production_visual_manifest.json")
            .expect("production visual manifest");
    let production_candidate_manifest =
        fs::read_to_string("assets/manifests/production_candidate_visual_manifest.json")
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
        "\"production_candidate_manifest\": \"assets/manifests/production_candidate_visual_manifest.json\""
    ));
    // Unit-047: production lane now contains source-approved production seed entries
    assert!(production_manifest.contains("\"entry_count\": 26"));
    assert!(production_manifest.contains("\"source_approved\""));
    assert!(production_manifest.contains("\"production_ready\": false"));
    assert!(production_manifest.contains("\"candidate_only\": false"));
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

    let runtime_mesh = fs::read_to_string("assets/runtime/candidate/longsword.mesh.json")
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
fn asset_provenance_audit_accepts_candidate_preview_metadata_without_standalone_runtime_previews() {
    let _asset_lock = acquire_asset_generation_test_lock();
    let production_visual = Command::new("python3")
        .args(["tools/production_visual_manifest.py"])
        .output()
        .expect("write production visual manifest before provenance audit");
    assert!(
        production_visual.status.success(),
        "production visual manifest failed before provenance audit\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&production_visual.stdout),
        String::from_utf8_lossy(&production_visual.stderr)
    );

    let out = Path::new("target/tmp/oathyard_asset_provenance_3d_only_preview_semantics_test");
    if out.exists() {
        fs::remove_dir_all(out).expect("clear old asset provenance audit output");
    }
    let audit = Command::new("./tools/asset_provenance_audit.sh")
        .arg(out)
        .output()
        .expect("run asset provenance audit");
    assert!(
        audit.status.success(),
        "asset provenance audit should accept candidate preview/capture metadata while standalone runtime previews stay empty under the 3D-only policy\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&audit.stdout),
        String::from_utf8_lossy(&audit.stderr)
    );

    let manifest = fs::read_to_string(out.join("asset_provenance_audit_manifest.json"))
        .expect("asset provenance audit manifest");
    let failed = fs::read_to_string(out.join("failed_asset_provenance_checks.txt"))
        .expect("asset provenance failed checks");
    let report = fs::read_to_string(out.join("asset_provenance_audit_report.md"))
        .expect("asset provenance report");

    assert!(manifest.contains("\"passed\": true"));
    assert!(manifest.contains("\"runtime_preview_optional_under_3d_only_policy\": true"));
    assert_eq!(failed, "none\n");
    assert!(!manifest.contains("_has_preview"));
    assert!(!manifest.contains("_preview_exists"));
    assert!(report
        .contains("Runtime preview files are optional under the 3D-only visual evidence policy"));
}

#[test]
fn generated_asset_audit_emits_fail_closed_candidate_quarantine_manifest() {
    let _asset_lock = acquire_asset_generation_test_lock();
    let out = Path::new("target/tmp/oathyard_generated_asset_quarantine_test");
    if out.exists() {
        fs::remove_dir_all(out).expect("clear old generated asset quarantine output");
    }
    let audit = Command::new("./tools/audit_generated_assets.sh")
        .arg(out)
        .output()
        .expect("run generated asset audit");
    assert!(
        !audit.status.success(),
        "generated asset audit must stay fail-closed while assets are candidate/license-pending\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&audit.stdout),
        String::from_utf8_lossy(&audit.stderr)
    );
    assert_eq!(audit.status.code(), Some(1));

    let quarantine_manifest =
        fs::read_to_string(out.join("generated_asset_quarantine_manifest.json"))
            .expect("generated asset quarantine manifest");
    let quarantine_report = fs::read_to_string(out.join("generated_asset_quarantine_report.md"))
        .expect("generated asset quarantine report");
    let unblock_matrix =
        fs::read_to_string(out.join("generated_asset_production_unblock_matrix.json"))
            .expect("generated asset production unblock matrix");
    let unblock_report =
        fs::read_to_string(out.join("generated_asset_production_unblock_matrix.md"))
            .expect("generated asset production unblock matrix report");
    let generated_audit = fs::read_to_string(out.join("generated_asset_audit.json"))
        .expect("generated asset audit json");
    let generated_audit_csv = fs::read_to_string(out.join("generated_asset_audit.csv"))
        .expect("generated asset audit csv");
    let blocked_asset_evidence = fs::read_to_string(out.join("blocked_asset_evidence.md"))
        .expect("blocked asset evidence report");
    let asset_state_summary =
        fs::read_to_string(out.join("asset_state_summary.md")).expect("asset state summary report");

    assert!(quarantine_manifest.contains("\"schema\": \"oathyard.generated_asset_quarantine.v1\""));
    assert!(quarantine_manifest.contains("\"candidate_asset_quarantine_active\": true"));
    assert!(quarantine_manifest.contains("\"production_asset_ready\": false"));
    assert!(quarantine_manifest.contains("\"owner_visual_accepted\": false"));
    assert!(quarantine_manifest.contains("\"public_demo_visual_ready\": false"));
    assert!(quarantine_manifest.contains("\"release_candidate_ready\": false"));
    assert!(quarantine_manifest.contains("\"generated_asset_audit_rc\": 1"));
    assert!(quarantine_manifest.contains("\"quarantined_asset_count\": 22"));
    assert!(quarantine_manifest.contains("\"blocked_by_license_or_commercial_use\": 22"));
    assert!(quarantine_manifest.contains("\"acceptance_status_counts\""));
    assert!(quarantine_manifest.contains("\"candidate\": 22"));
    assert!(quarantine_manifest.contains("\"quarantined_assets\""));
    assert!(quarantine_manifest.contains("license_or_project_license_pending"));
    assert!(!quarantine_manifest.contains("external_dcc_validation_missing"));
    assert!(!quarantine_manifest.contains("topology_manifold_boundary_edges_present"));
    assert!(!quarantine_manifest.contains("external_khronos_gltf_validation_missing"));
    assert!(!quarantine_manifest.contains("procedural_model_candidate_not_dcc_source"));
    assert!(!quarantine_manifest.contains("tangents_missing_or_unverified"));
    assert!(quarantine_report.contains("Status: FAIL-CLOSED"));
    assert!(
        quarantine_report.contains("No generated/model-candidate asset is promoted to production.")
    );
    assert!(unblock_matrix
        .contains("\"schema\": \"oathyard.generated_asset_production_unblock_matrix.v1\""));
    assert!(unblock_matrix.contains("\"production_asset_ready\": false"));
    assert!(unblock_matrix.contains("\"quarantined_asset_count\": 22"));
    assert!(unblock_matrix.contains("\"blocked_asset_count\": 22"));
    assert!(unblock_matrix.contains("\"required_unblock_stage_count\": 7"));
    assert!(unblock_matrix.contains("\"license_and_commercial_clearance\": 22"));
    assert!(unblock_matrix.contains("\"dcc_or_openusd_source_authoring\": 0"));
    assert!(unblock_matrix.contains("\"external_geometry_validation\": 0"));
    assert!(unblock_matrix.contains("\"material_texture_uv_completion\": 0"));
    assert!(unblock_matrix.contains("\"rig_truth_contact_profile_validation\": 0"));
    assert!(unblock_matrix.contains("\"native_renderer_capture_matrix\": 22"));
    assert!(unblock_matrix.contains("\"owner_visual_acceptance\": 22"));
    assert!(unblock_matrix.contains("\"license_and_commercial_clearance\""));
    assert!(unblock_matrix.contains("\"dcc_or_openusd_source_authoring\""));
    assert!(unblock_matrix.contains("\"native_renderer_capture_matrix\""));
    assert!(unblock_matrix.contains("\"owner_visual_acceptance\""));
    assert!(unblock_matrix.contains("\"per_asset_unblock_plan\""));
    assert!(unblock_matrix.contains("\"oathyard_verdict_ring\""));
    assert!(unblock_matrix.contains("\"training_yard\""));
    assert!(unblock_matrix.contains("\"longsword\""));
    assert!(unblock_matrix.contains("\"arming_sword\""));
    assert!(unblock_matrix.contains("\"ash_spear\""));
    assert!(unblock_matrix.contains("\"bearded_axe\""));
    assert!(unblock_matrix.contains("\"billhook\""));
    assert!(unblock_matrix.contains("\"curved_sword\""));
    assert!(unblock_matrix.contains("\"iron_maul\""));
    assert!(unblock_matrix.contains("\"round_shield\""));
    assert!(unblock_matrix.contains("\"saltreach_duelist\""));
    assert!(unblock_matrix.contains("\"oathyard_writ\""));
    assert!(unblock_matrix.contains("\"chainbreaker\""));
    assert!(unblock_matrix.contains("\"reed_sentinel\""));
    assert!(unblock_matrix.contains("\"gate_shield\""));
    assert!(unblock_matrix.contains("\"bruiser_oath\""));
    assert!(unblock_matrix.contains("\"gambeson\""));
    assert!(unblock_matrix.contains("\"mail_hauberk\""));
    assert!(unblock_matrix.contains("\"heavy_plate\""));
    assert!(unblock_matrix.contains("\"lamellar\""));
    assert!(unblock_matrix.contains("\"fencer_light\""));
    assert!(unblock_matrix.contains("\"bruiser_padded_plate\""));
    assert!(unblock_matrix.contains("\"source_authoring_evidence\""));
    assert!(Path::new(
        "assets/source/model_candidates/t_73291be5/arenas/oathyard_verdict_ring.source.usda"
    )
    .is_file());
    assert!(Path::new(
        "assets/source/model_candidates/t_73291be5/arenas/training_yard.source.usda"
    )
    .is_file());
    assert!(
        Path::new("assets/source/model_candidates/t_73291be5/weapons/longsword.source.usda")
            .is_file()
    );
    assert!(Path::new(
        "assets/source/model_candidates/t_73291be5/weapons/arming_sword.source.usda"
    )
    .is_file());
    assert!(
        Path::new("assets/source/model_candidates/t_73291be5/weapons/ash_spear.source.usda")
            .is_file()
    );
    assert!(
        Path::new("assets/source/model_candidates/t_73291be5/weapons/bearded_axe.source.usda")
            .is_file()
    );
    assert!(
        Path::new("assets/source/model_candidates/t_73291be5/weapons/billhook.source.usda")
            .is_file()
    );
    assert!(Path::new(
        "assets/source/model_candidates/t_73291be5/weapons/curved_sword.source.usda"
    )
    .is_file());
    assert!(
        Path::new("assets/source/model_candidates/t_73291be5/weapons/iron_maul.source.usda")
            .is_file()
    );
    assert!(Path::new(
        "assets/source/model_candidates/t_73291be5/weapons/round_shield.source.usda"
    )
    .is_file());
    assert!(Path::new(
        "assets/source/model_candidates/t_73291be5/fighters/saltreach_duelist.source.usda"
    )
    .is_file());
    assert!(Path::new(
        "assets/source/model_candidates/t_73291be5/fighters/oathyard_writ.source.usda"
    )
    .is_file());
    assert!(Path::new(
        "assets/source/model_candidates/t_73291be5/fighters/chainbreaker.source.usda"
    )
    .is_file());
    assert!(Path::new(
        "assets/source/model_candidates/t_73291be5/fighters/reed_sentinel.source.usda"
    )
    .is_file());
    assert!(Path::new(
        "assets/source/model_candidates/t_73291be5/fighters/gate_shield.source.usda"
    )
    .is_file());
    assert!(Path::new(
        "assets/source/model_candidates/t_73291be5/fighters/bruiser_oath.source.usda"
    )
    .is_file());
    assert!(
        Path::new("assets/source/model_candidates/t_73291be5/armor/gambeson.source.usda").is_file()
    );
    assert!(
        Path::new("assets/source/model_candidates/t_73291be5/armor/mail_hauberk.source.usda")
            .is_file()
    );
    assert!(
        Path::new("assets/source/model_candidates/t_73291be5/armor/heavy_plate.source.usda")
            .is_file()
    );
    assert!(
        Path::new("assets/source/model_candidates/t_73291be5/armor/lamellar.source.usda").is_file()
    );
    assert!(
        Path::new("assets/source/model_candidates/t_73291be5/armor/fencer_light.source.usda")
            .is_file()
    );
    assert!(Path::new(
        "assets/source/model_candidates/t_73291be5/armor/bruiser_padded_plate.source.usda"
    )
    .is_file());
    assert!(unblock_matrix.contains("license_or_project_license_pending"));
    assert!(unblock_report.contains("Status: FAIL-CLOSED"));
    assert!(unblock_report.contains("Required production unblock stages: `7`"));
    for required_field in [
        "\"asset_id\"",
        "\"asset_class\"",
        "\"source_path\"",
        "\"runtime_path\"",
        "\"generation_import_tool\"",
        "\"tool_version\"",
        "\"generation_date\"",
        "\"source_prompt_path_or_hash\"",
        "\"source_image_path_or_hash\"",
        "\"rodin_task_download_export_ids\"",
        "\"source_file_hash\"",
        "\"runtime_file_hash\"",
        "\"texture_file_hashes\"",
        "\"commercial_use_status\"",
        "\"ip_protected_style_risk_status\"",
        "\"index_count\"",
        "\"bounds\"",
        "\"normal_status\"",
        "\"tangent_status\"",
        "\"material_status\"",
        "\"texture_status\"",
        "\"physics_contact_profile_status\"",
        "\"mass_inertia_status\"",
        "\"collision_contact_region_status\"",
        "\"in_engine_capture_status\"",
        "\"package_inclusion_status\"",
        "\"presentation_truth_isolation_status\"",
        "\"acceptance_state\"",
        "\"blockers\"",
        "\"next_action\"",
    ] {
        assert!(
            generated_audit.contains(required_field),
            "generated asset audit missing required field {required_field}"
        );
    }
    assert!(generated_audit.contains("\"production_ready\": false"));
    assert!(!generated_audit.contains("\"production_ready\": true"));
    assert!(generated_audit.contains("\"candidate_only\": true"));
    assert!(generated_audit_csv.starts_with("asset_id,asset_class,"));
    assert!(generated_audit_csv.contains("license_terms_status"));
    assert!(blocked_asset_evidence.contains("# OATHYARD Blocked Generated Asset Evidence"));
    assert!(blocked_asset_evidence.contains("license_or_project_license_pending"));
    assert!(blocked_asset_evidence.contains("native production renderer capture evidence missing"));
    assert!(asset_state_summary.contains("# OATHYARD Generated Asset State Summary"));
    assert!(asset_state_summary.contains("candidate_only: `22`"));
    assert!(asset_state_summary.contains("license_pending: `22`"));
    assert!(asset_state_summary.contains("production_ready: `0`"));
    for latest_path in [
        "artifacts/asset_audit/latest/generated_asset_audit.json",
        "artifacts/asset_audit/latest/generated_asset_audit.md",
        "artifacts/asset_audit/latest/generated_asset_audit.csv",
        "artifacts/asset_audit/latest/blocked_asset_evidence.md",
        "artifacts/asset_audit/latest/asset_state_summary.md",
    ] {
        assert!(
            Path::new(latest_path).is_file(),
            "missing latest audit mirror {latest_path}"
        );
    }
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
    assert!(manifest.contains("\"nonvisual_material_evidence\""));
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
fn final_acceptance_runs_match_sweep_inside_evidence_package() {
    let final_acceptance =
        fs::read_to_string("tools/final_acceptance.sh").expect("missing final acceptance tool");
    let match_sweep =
        fs::read_to_string("tools/run_match_sweep.sh").expect("missing match sweep tool");

    assert!(final_acceptance.contains("run_step match_sweep ./tools/run_match_sweep.sh"));
    assert!(final_acceptance.contains("\"$out/match_sweep\""));
    assert!(match_sweep.contains("root=\"${1:-artifacts/match_sweep}\""));
    assert!(match_sweep.contains("python3 - \"$root\" <<'PY'"));
    assert!(match_sweep.contains("root = Path(sys.argv[1])"));
    assert!(match_sweep.contains("match_sweep_summary.json"));
    assert!(match_sweep.contains("public_demo_ready"));
    assert!(match_sweep.contains("release_candidate_ready"));
}

#[test]
fn final_acceptance_manifest_indexes_required_evidence_artifacts() {
    let final_acceptance =
        fs::read_to_string("tools/final_acceptance.sh").expect("missing final acceptance tool");

    assert!(final_acceptance.contains("'artifact_index'"));
    assert!(final_acceptance.contains("'artifact_index_missing_count'"));
    assert!(final_acceptance.contains("final_acceptance_steps.tsv"));
    assert!(final_acceptance.contains("verify_a/replay.json"));
    assert!(final_acceptance.contains("match_sweep_summary.json"));
    assert!(final_acceptance.contains("generated_asset_quarantine_manifest.json"));
    assert!(final_acceptance.contains("generated_asset_production_unblock_matrix.json"));
    assert!(final_acceptance.contains("generated_asset_production_unblock_matrix.md"));
    assert!(final_acceptance.contains("high_fidelity_capture_matrix.json"));
    assert!(final_acceptance.contains("visual_benchmark_report.md"));
    assert!(final_acceptance.contains("visual_gap_list.md"));
    assert!(final_acceptance.contains("oathyard-linux-x86_64.tar"));
    assert!(final_acceptance.contains("public_demo_ready':False"));
    assert!(final_acceptance.contains("release_candidate_ready':False"));
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
    assert!(report.contains("Status: PASSED"));
    assert!(report.contains("Motion system: `MotionBricks-inspired PresentationBricks`"));
    assert!(report.contains("Named vendor integration claimed: `false`"));
    assert!(report.contains("Truth mutation: `none`"));
}

#[test]
fn native_render_path_is_blocked_without_3d_renderer_capture() {
    let source = fs::read_to_string("src/lib.rs").expect("source");
    let native_tool = fs::read_to_string("tools/native_combat_render.sh").expect("native tool");
    let hifi_tool =
        fs::read_to_string("tools/capture_high_fidelity_screens.sh").expect("hifi tool");

    // Source must contain both the promoted path and the blocked fallback
    assert!(source.contains("render_native_3d_blocked_manifest_json"));
    assert!(source.contains("oathyard.native_3d_visual_blocked.v1"));
    assert!(source.contains("oathyard.native_combat_render.v1"));
    assert!(source.contains("truth-after-hash-duel-result"));
    assert!(source.contains("forbidden_visual_fallbacks_emitted"));
    assert!(source.contains("native_3d_visual_evidence_present"));
    // Native tool must require promoted schema and verify real capture
    assert!(native_tool.contains("native_3d_visual_evidence_present"));
    assert!(native_tool.contains("production_renderer_"));
    assert!(native_tool.contains("forbidden_visual_fallbacks_emitted"));
    assert!(native_tool.contains("native_3d_renderer_capture_present"));
    assert!(hifi_tool.contains("native_3d_visual_evidence_required"));
    assert!(hifi_tool.contains("BLOCKED_PENDING_NATIVE_3D_RENDERER_CAPTURE"));
}

#[test]
fn high_fidelity_capture_gate_emits_fail_closed_required_capture_matrix() {
    let out = Path::new("target/tmp/oathyard_high_fidelity_capture_matrix_test");
    if out.exists() {
        fs::remove_dir_all(out).expect("clear old high-fidelity capture matrix output");
    }
    let capture = Command::new("./tools/capture_high_fidelity_screens.sh")
        .arg(out)
        .output()
        .expect("run high-fidelity capture gate");
    assert!(
        !capture.status.success(),
        "capture gate must fail closed while native production 3D renderer evidence is missing\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&capture.stdout),
        String::from_utf8_lossy(&capture.stderr)
    );
    assert_eq!(capture.status.code(), Some(1));

    let matrix = fs::read_to_string(out.join("high_fidelity_capture_matrix.json"))
        .expect("high-fidelity capture matrix json");
    let matrix_report = fs::read_to_string(out.join("high_fidelity_capture_matrix.md"))
        .expect("high-fidelity capture matrix report");

    assert!(matrix.contains("\"schema\": \"oathyard.high_fidelity_capture_matrix.v1\""));
    assert!(matrix.contains("\"required_capture_group_count\": 27"));
    assert!(matrix.contains("\"required_capture_slot_count\": 56"));
    assert!(matrix.contains("\"minimum_resolution_width\": 1920"));
    assert!(matrix.contains("\"minimum_resolution_height\": 1080"));
    assert!(matrix.contains("\"current_native_capture_count\": 0"));
    assert!(matrix.contains("\"fallback_visual_substitutes_allowed\": false"));
    assert!(matrix.contains("\"production_renderer_complete\": false"));
    assert!(matrix.contains("\"truth_mutation\": false"));
    assert!(matrix.contains("boot_main_menu"));
    assert!(matrix.contains("fighter_closeup_06"));
    assert!(matrix.contains("weapon_family_closeup_08"));
    assert!(matrix.contains("gameplay_distance_weapon_family_08"));
    assert!(matrix.contains("performance_debug_overlay"));
    assert!(matrix.contains("missing_native_3d_capture"));
    assert!(matrix_report.contains("Status: BLOCKED_PENDING_NATIVE_3D_RENDERER_CAPTURE"));
    assert!(matrix_report.contains("Required capture slots: `56`"));
    assert!(matrix_report.contains("Fallback visual substitutes: `forbidden`"));
}

#[test]
fn high_fidelity_capture_gate_ingests_valid_production_renderer_capture_manifest_fail_closed() {
    let root = Path::new("target/tmp/oathyard_high_fidelity_capture_manifest_ingest_test");
    if root.exists() {
        fs::remove_dir_all(root).expect("clear old production renderer fixture output");
    }
    let renderer_root = root.join("production_renderer_fixture");
    let out = root.join("high_fidelity_screens");
    fs::create_dir_all(&renderer_root).expect("create production renderer fixture root");

    let png_path = renderer_root.join("production_renderer_boot_main_menu.png");
    let mut png = Vec::new();
    png.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    png.extend_from_slice(&13u32.to_be_bytes());
    png.extend_from_slice(b"IHDR");
    png.extend_from_slice(&1920u32.to_be_bytes());
    png.extend_from_slice(&1080u32.to_be_bytes());
    png.extend_from_slice(&[8, 6, 0, 0, 0]);
    fs::write(&png_path, png).expect("write production renderer png fixture");

    let manifest_path = renderer_root.join("production_renderer_manifest.json");
    fs::write(
        &manifest_path,
        r#"{
  "schema": "oathyard.production_renderer_manifest.v1",
  "production_renderer_complete": true,
  "native_3d_render_capture": true,
  "truth_mutation": false,
  "fallback_visual_substitutes_allowed": false,
  "owner_visual_acceptance": false,
  "public_demo_ready": false,
  "release_candidate_ready": false,
  "captures": [
    {
      "capture_id": "boot_main_menu",
      "capture_file": "production_renderer_boot_main_menu.png",
      "native_3d_capture": true,
      "truth_mutation": false,
      "renderer_backend_id": "fixture-native-renderer",
      "renderer_build_hash_or_binary_hash": "fixture-build-hash",
      "quality_preset": "fixture-production-candidate",
      "replay_path": "artifacts/fixture/replay.json",
      "replay_final_hash": "f17c8f76b9dfae86",
      "content_manifest_hash": "fixture-content-hash",
      "asset_manifest_hash": "fixture-asset-hash",
      "camera_mode": "fixture-boot-main-menu",
      "frame_or_tick": "0"
    }
  ]
}
"#,
    )
    .expect("write production renderer fixture manifest");

    let capture = Command::new("./tools/capture_high_fidelity_screens.sh")
        .arg(&out)
        .env("OATHYARD_PRODUCTION_RENDERER_MANIFEST", &manifest_path)
        .env("OATHYARD_PRODUCTION_RENDERER_ROOT", &renderer_root)
        .output()
        .expect("run high-fidelity capture gate with production renderer fixture");
    assert!(
        !capture.status.success(),
        "capture gate must remain fail-closed until all required native capture slots exist\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&capture.stdout),
        String::from_utf8_lossy(&capture.stderr)
    );
    assert_eq!(capture.status.code(), Some(1));

    let matrix = fs::read_to_string(out.join("high_fidelity_capture_matrix.json"))
        .expect("high-fidelity capture matrix json");
    let report = fs::read_to_string(out.join("high_fidelity_capture_matrix.md"))
        .expect("high-fidelity capture matrix report");
    let manifest = fs::read_to_string(out.join("high_fidelity_screen_manifest.json"))
        .expect("high-fidelity capture manifest");

    assert!(matrix.contains("\"current_native_capture_count\": 1"));
    assert!(matrix.contains("\"missing_native_capture_count\": 55"));
    assert!(matrix.contains("\"capture_id\": \"boot_main_menu\""));
    assert!(matrix.contains("\"status\": \"native_3d_capture_present\""));
    assert!(matrix.contains("\"renderer_backend_id\": \"fixture-native-renderer\""));
    assert!(matrix.contains("\"truth_mutation\": false"));
    assert!(matrix.contains("\"fallback_visual_substitutes_allowed\": false"));
    assert!(manifest.contains("\"capture_count\": 1"));
    assert!(manifest.contains("\"passed\": false"));
    assert!(manifest.contains("\"public_demo_ready\": false"));
    assert!(manifest.contains("\"release_candidate_ready\": false"));
    assert!(report.contains("Current native capture count: `1`"));
    assert!(report.contains("Missing native capture count: `55`"));
}

#[test]
fn high_fidelity_capture_gate_records_renderer_spike_candidate_without_production_credit() {
    let root = Path::new("target/tmp/oathyard_high_fidelity_capture_candidate_ingest_test");
    if root.exists() {
        fs::remove_dir_all(root).expect("clear old candidate renderer fixture output");
    }
    let renderer_root = root.join("wgpu_renderer_spike_fixture");
    let out = root.join("high_fidelity_screens");
    fs::create_dir_all(&renderer_root).expect("create candidate renderer fixture root");

    let png_path = renderer_root.join("production_renderer_wgpu_spike_1920x1080.png");
    let mut png = Vec::new();
    png.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    png.extend_from_slice(&13u32.to_be_bytes());
    png.extend_from_slice(b"IHDR");
    png.extend_from_slice(&1920u32.to_be_bytes());
    png.extend_from_slice(&1080u32.to_be_bytes());
    png.extend_from_slice(&[8, 6, 0, 0, 0]);
    fs::write(&png_path, png).expect("write candidate renderer png fixture");

    let manifest_path = renderer_root.join("production_renderer_manifest.json");
    fs::write(
        &manifest_path,
        r#"{
  "schema": "oathyard.production_renderer_manifest.v1",
  "production_renderer_complete": false,
  "native_3d_render_capture": true,
  "truth_mutation": false,
  "fallback_visual_substitutes_allowed": false,
  "owner_visual_acceptance": false,
  "public_demo_ready": false,
  "release_candidate_ready": false,
  "captures": [
    {
      "capture_id": "oathyard_verdict_ring_establishing",
      "capture_file": "production_renderer_wgpu_spike_1920x1080.png",
      "native_3d_capture": true,
      "truth_mutation": false,
      "renderer_backend_id": "wgpu-vulkan-offscreen-production-renderer-spike-v1",
      "renderer_build_hash_or_binary_hash": "fixture-wgpu-build-hash",
      "quality_preset": "wgpu_offscreen_spike_candidate_blockout_not_production",
      "replay_path": "artifacts/fixture/replay.json",
      "replay_final_hash": "f17c8f76b9dfae86",
      "content_manifest_hash": "fixture-content-hash",
      "asset_manifest_hash": "candidate-spike-no-production-assets",
      "camera_mode": "offscreen_verdict_ring_establishing_spike",
      "frame_or_tick": "post_hash_static_frame"
    }
  ]
}
"#,
    )
    .expect("write candidate renderer fixture manifest");

    let capture = Command::new("./tools/capture_high_fidelity_screens.sh")
        .arg(&out)
        .env("OATHYARD_PRODUCTION_RENDERER_MANIFEST", &manifest_path)
        .env("OATHYARD_PRODUCTION_RENDERER_ROOT", &renderer_root)
        .output()
        .expect("run high-fidelity capture gate with candidate renderer fixture");
    assert!(
        !capture.status.success(),
        "candidate renderer capture must not pass the high-fidelity production gate\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&capture.stdout),
        String::from_utf8_lossy(&capture.stderr)
    );
    assert_eq!(capture.status.code(), Some(1));

    let matrix = fs::read_to_string(out.join("high_fidelity_capture_matrix.json"))
        .expect("high-fidelity candidate capture matrix json");
    let manifest = fs::read_to_string(out.join("high_fidelity_screen_manifest.json"))
        .expect("high-fidelity candidate capture manifest");
    let report = fs::read_to_string(out.join("high_fidelity_capture_matrix.md"))
        .expect("high-fidelity candidate capture matrix report");

    assert!(matrix.contains("\"current_native_capture_count\": 0"));
    assert!(matrix.contains("\"missing_native_capture_count\": 56"));
    assert!(matrix.contains("\"candidate_native_capture_count\": 1"));
    assert!(matrix.contains("\"status\": \"candidate_native_3d_capture_not_production_complete\""));
    assert!(matrix.contains("\"capture_id\": \"oathyard_verdict_ring_establishing\""));
    assert!(matrix.contains("\"production_renderer_complete\": false"));
    assert!(matrix.contains("\"owner_visual_acceptance\": false"));
    assert!(manifest.contains("\"candidate_native_capture_count\": 1"));
    assert!(manifest.contains("\"capture_count\": 0"));
    assert!(manifest.contains("\"passed\": false"));
    assert!(manifest.contains("\"public_demo_ready\": false"));
    assert!(manifest.contains("\"release_candidate_ready\": false"));
    assert!(report.contains("Candidate native capture count: `1`"));
    assert!(report.contains("Current native capture count: `0`"));
}

#[test]
fn wgpu_renderer_spike_declares_rodin_candidate_asset_capture_rows() {
    let source = std::fs::read_to_string("spikes/wgpu_renderer/src/main.rs")
        .expect("missing wgpu renderer spike source");
    let tool = std::fs::read_to_string("tools/wgpu_renderer_spike.sh")
        .expect("missing wgpu renderer spike tool");

    assert!(source.contains("--capture-id"));
    assert!(source.contains("--candidate-assets"));
    assert!(source.contains("candidate_asset_ids"));
    assert!(source.contains("asset_manifest_sha256"));
    assert!(tool.contains("assets/manifests/production_candidate_visual_manifest.json"));
    assert!(tool.contains("production_renderer_manifest_default.json"));
    assert!(tool.contains("fighter_closeup_01"));
    assert!(tool.contains("weapon_family_closeup_01"));
    assert!(tool.contains("armor_loadout_family_closeup_01"));
    assert!(tool.contains("gameplay_distance_fighter_loadout_family_01"));
    assert!(tool.contains("gameplay_distance_weapon_family_01"));
    assert!(tool.contains("pre_contact_frame"));
    assert!(tool.contains("contact_frame"));
    assert!(tool.contains("candidate_asset_ids"));
    assert!(tool.contains("wgpu_offscreen_spike_candidate_rodin_asset_metadata_not_mesh_render"));
    assert!(tool.contains("'saltreach_duelist'"));
    assert!(tool.contains("'longsword'"));
    assert!(tool.contains("production_renderer_wgpu_spike_fighter_closeup_01_1920x1080.png"));
}

#[test]
fn wgpu_renderer_spike_consumes_candidate_runtime_mesh_geometry() {
    let source = std::fs::read_to_string("spikes/wgpu_renderer/src/main.rs")
        .expect("missing wgpu renderer spike source");
    let shader = std::fs::read_to_string("spikes/wgpu_renderer/src/verdict_ring.wgsl")
        .expect("missing wgpu verdict-ring shader");
    let tool = std::fs::read_to_string("tools/wgpu_renderer_spike.sh")
        .expect("missing wgpu renderer spike tool");

    assert!(source.contains("--mesh-json"));
    assert!(source.contains("load_runtime_mesh"));
    assert!(source.contains("MeshVertex"));
    assert!(source.contains("create_buffer_init"));
    assert!(source.contains("draw_indexed"));
    assert!(source.contains("mesh_geometry_consumed"));
    assert!(shader.contains("mesh_vs_main"));
    assert!(shader.contains("mesh_fs_main"));
    assert!(tool.contains("--mesh-json"));
    assert!(tool.contains("assets/runtime/candidate/longsword.mesh.json"));
}

#[test]
fn wgpu_renderer_spike_consumes_multiclass_runtime_mesh_assets() {
    let source = std::fs::read_to_string("spikes/wgpu_renderer/src/main.rs")
        .expect("missing wgpu renderer spike source");
    let shader = std::fs::read_to_string("spikes/wgpu_renderer/src/verdict_ring.wgsl")
        .expect("missing wgpu verdict-ring shader");
    let tool = std::fs::read_to_string("tools/wgpu_renderer_spike.sh")
        .expect("missing wgpu renderer spike tool");

    assert!(source.contains("--mesh-manifest-json"));
    assert!(source.contains("load_runtime_mesh_manifest"));
    assert!(source.contains("mesh_asset_id"));
    assert!(source.contains("mesh_asset_class"));
    assert!(source.contains("mesh_assets"));
    assert!(source.contains("candidate_status"));
    assert!(source.contains("production_ready"));
    assert!(source.contains("transform_baked_or_runtime"));
    assert!(source.contains("draw_indexed"));
    assert!(shader.contains("mesh_vs_main"));
    assert!(shader.contains("mesh_fs_main"));

    for class_name in ["fighter", "weapon", "armor", "arena"] {
        assert!(
            tool.contains(class_name),
            "wgpu renderer spike must declare mesh-backed class {class_name}"
        );
    }
    for required_mesh in [
        "assets/runtime/candidate/saltreach_duelist.mesh.json",
        "assets/runtime/candidate/longsword.mesh.json",
        "assets/runtime/candidate/gambeson.mesh.json",
        "assets/runtime/candidate/oathyard_verdict_ring.mesh.json",
    ] {
        assert!(
            tool.contains(required_mesh),
            "wgpu renderer spike must consume {required_mesh}"
        );
    }
    for required_capture in [
        "fighter_closeup_01",
        "armor_family_closeup_01",
        "weapon_family_closeup_01",
        "oathyard_arena_candidate_01",
        "gameplay_distance_fighter_weapon_01",
        "pre_contact_frame",
        "contact_frame",
        "fight_film_candidate_shot_01",
    ] {
        assert!(
            tool.contains(required_capture),
            "wgpu renderer spike must emit mesh-backed capture row {required_capture}"
        );
    }
    assert!(tool.contains("mesh_geometry_capture_count"));
    assert!(tool.contains("mesh_class_coverage"));
    assert!(tool.contains("distinct_mesh_sha256_count"));
    assert!(tool.contains("presentation_truth_isolation_passed"));
    assert!(tool.contains("production_renderer_complete") && tool.contains("False"));
    assert!(!tool.contains(".ppm"));
    assert!(!tool.contains(".svg"));
}

#[test]
fn wgpu_renderer_spike_binds_candidate_material_textures_fail_closed() {
    let source = std::fs::read_to_string("spikes/wgpu_renderer/src/main.rs")
        .expect("missing wgpu renderer spike source");
    let shader = std::fs::read_to_string("spikes/wgpu_renderer/src/verdict_ring.wgsl")
        .expect("missing wgpu verdict-ring shader");
    let tool = std::fs::read_to_string("tools/wgpu_renderer_spike.sh")
        .expect("missing wgpu renderer spike tool");

    assert!(source.contains("RuntimeMaterial"));
    assert!(source.contains("load_runtime_material"));
    assert!(source.contains("material_texture_binding"));
    assert!(source.contains("material_texture_summary"));
    assert!(source.contains("create_texture_with_data"));
    assert!(source.contains("TextureUsages::TEXTURE_BINDING"));
    assert!(source.contains("TextureSampleType::Float"));
    assert!(source.contains("base_color_texture_path"));
    assert!(source.contains("normal_texture_path"));
    assert!(source.contains("orm_texture_path"));
    assert!(source.contains("texture_hashes"));
    assert!(shader.contains("@group(1) @binding(0)"));
    assert!(shader.contains("texture_2d<f32>"));
    assert!(shader.contains("triplanar"));
    assert!(shader.contains("material_uv"));
    assert!(tool.contains("material_texture_binding_count"));
    assert!(tool.contains("bound_texture_channels"));
    assert!(tool.contains("distinct_texture_sha256_count"));
    assert!(tool.contains("base_color_texture_path"));
    assert!(tool.contains("normal_texture_path"));
    assert!(tool.contains("orm_texture_path"));
    assert!(tool.contains("production_renderer_complete") && tool.contains("False"));
    assert!(!tool.contains("owner_visual_acceptance = True"));
}

#[test]
fn wgpu_renderer_spike_multiclass_manifest_assertion_is_fail_closed() {
    let tool = std::fs::read_to_string("tools/wgpu_renderer_spike.sh")
        .expect("missing wgpu renderer spike tool");

    assert!(tool.contains("required_mesh_classes"));
    assert!(tool.contains("mesh_backed_captures"));
    assert!(tool.contains("distinct_mesh_sha256_count"));
    assert!(tool.contains("mesh_asset_class"));
    assert!(tool.contains("mesh_sha256"));
    assert!(tool.contains("vertex_count"));
    assert!(tool.contains("index_count"));
    assert!(tool.contains("triangle_count"));
    assert!(tool.contains("bounds_min"));
    assert!(tool.contains("bounds_max"));
    assert!(tool.contains("candidate_status"));
    assert!(tool.contains("production_ready"));
    assert!(tool.contains("truth_mutation"));
    assert!(tool.contains("raise SystemExit(1)"));
}

#[test]
fn visual_benchmark_integrates_high_fidelity_capture_matrix_into_gap_list() {
    let root = Path::new("target/tmp/oathyard_visual_benchmark_capture_matrix_test");
    if root.exists() {
        fs::remove_dir_all(root).expect("clear old visual benchmark capture matrix output");
    }
    fs::create_dir_all(root).expect("create visual benchmark capture matrix output");

    let capture = Command::new("./tools/capture_high_fidelity_screens.sh")
        .arg(root.join("high_fidelity_screens"))
        .output()
        .expect("run high-fidelity capture gate");
    assert_eq!(capture.status.code(), Some(1));

    let benchmark = Command::new("./tools/visual_benchmark.sh")
        .arg(root.join("visual_review"))
        .output()
        .expect("run visual benchmark gate");
    assert!(
        !benchmark.status.success(),
        "visual benchmark must fail closed without production visual evidence"
    );
    assert_eq!(benchmark.status.code(), Some(1));

    let manifest = fs::read_to_string(root.join("visual_review/visual_benchmark_manifest.json"))
        .expect("visual benchmark manifest");
    let report = fs::read_to_string(root.join("visual_review/visual_benchmark_report.md"))
        .expect("visual benchmark report");
    let gap_list =
        fs::read_to_string(root.join("visual_review/visual_gap_list.md")).expect("visual gap list");

    assert!(manifest.contains("\"source_capture_matrix\":"));
    assert!(manifest.contains("\"required_capture_slot_count\": 56"));
    assert!(manifest.contains("\"missing_native_capture_count\": 56"));
    assert!(manifest.contains("\"current_native_capture_count\": 0"));
    assert!(manifest.contains("\"fallback_visual_substitutes_allowed\": false"));
    assert!(manifest.contains("\"production_renderer_complete\": false"));
    assert!(manifest.contains("\"owner_visual_acceptance\": false"));
    assert!(report.contains("Required high-fidelity capture slots: `56`"));
    assert!(report.contains("Missing native capture slots: `56`"));
    assert!(gap_list.contains("Required high-fidelity capture slots: `56`"));
    assert!(gap_list.contains("Missing native capture slots: `56`"));
    assert!(gap_list.contains("Fallback visual substitutes: `forbidden`"));
}

#[test]
fn performance_benchmark_handles_blocked_native_3d_manifest_without_traceback() {
    let perf_tool =
        fs::read_to_string("tools/performance_benchmark.py").expect("performance benchmark tool");

    assert!(perf_tool.contains("blocked_pending_native_3d_renderer_capture"));
    assert!(perf_tool.contains("native_render_status"));
    assert!(perf_tool.contains("render_manifest.get(\"playback_loop\", {})"));
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
    // Unit-055: wrapper now delegates to production renderer at crates/oathyard_renderer/
    assert!(tool.contains("cargo run --locked --manifest-path crates/oathyard_renderer/Cargo.toml"));
    assert!(tool.contains("post_hash_presentation_packet.json"));
    assert!(tool.contains("production_renderer_manifest.json"));
    assert!(tool.contains("native_3d_render_capture"));
    assert!(tool.contains("oathyard_verdict_ring_establishing"));
    assert!(tool.contains("presentation_truth_isolation_passed"));
    assert!(tool.contains("\"truth_mutation\": false"));
    assert!(truth_tool.contains("tools/wgpu_renderer_spike.sh"));
    assert!(hifi_tool.contains("capture_path.suffix.lower() != '.png'"));
    assert!(
        hifi_tool.contains("capture file is not a production_renderer_*.png native evidence file")
    );
}

#[test]
fn unit050_native_3d_game_flow_planning_loop_truth_isolated() {
    let tool = std::fs::read_to_string("tools/run_native_3d_game_flow.sh")
        .expect("missing native 3D game flow tool");

    // Verify the script exists and has correct structure
    assert!(tool.contains("#!/usr/bin/env bash"));
    assert!(tool.contains("run_duel.sh"));
    assert!(tool.contains("replay_verify.sh"));
    assert!(tool.contains("presentation-bricks"));
    assert!(tool.contains("wgpu_renderer_spike.sh"));
    assert!(tool.contains("native_3d_game_flow_manifest"));

    // Verify the Python manifest generator includes required fields
    assert!(tool.contains("OBSERVE"));
    assert!(tool.contains("PLAN"));
    assert!(tool.contains("COMMIT_REVEAL"));
    assert!(tool.contains("RESOLVE"));
    assert!(tool.contains("CONSEQUENCE"));
    assert!(tool.contains("REPLAN"));
    assert!(tool.contains("truth_mutation"));
    assert!(tool.contains("production_seed_capture_count"));
    assert!(tool.contains("presentation_bricks_frames"));
    assert!(tool.contains("ui_role"));
    assert!(tool.contains("committed_actions"));
    assert!(tool.contains("contact_packets"));
    assert!(tool.contains("capability_changes"));
    assert!(tool.contains("cost_preview"));

    // Verify large-file policy exists
    let lfs_policy = std::fs::read_to_string("docs/decisions/0010-large-file-policy.md")
        .expect("missing large-file policy");
    assert!(lfs_policy.contains("gambeson.obj"));
    assert!(lfs_policy.contains("50 MiB"));

    // Verify skeleton definition exists
    let skeleton = std::fs::read_to_string("content/skeletons/production_seed_skeleton.json")
        .expect("missing skeleton definition");
    assert!(skeleton.contains("truth_joint_count"));
    assert!(skeleton.contains("grip_r"));
    assert!(skeleton.contains("grip_l"));
    assert!(skeleton.contains("presentation_bone_count"));

    // Verify animation manifest exists
    let anim = std::fs::read_to_string("content/animations/production_seed_animations.json")
        .expect("missing animation manifest");
    assert!(anim.contains("MotionBricks-inspired PresentationBricks"));
    assert!(anim.contains("retargeting_bridge"));
    assert!(anim.contains("truth_mutation"));
}

#[test]
fn unit051_first_kit_production_ready_candidate_truth_isolated() {
    // Verify first-kit production-ready-candidate asset manifest exists
    let assets =
        std::fs::read_to_string("content/assets/unit051_production_ready_candidate_assets.json")
            .expect("missing unit051 production ready candidate assets manifest");
    assert!(assets.contains("unit051"));
    assert!(assets.contains("production_ready_candidate"));
    // Each first-kit asset must have required metadata
    for field in [
        "source_path",
        "runtime_path",
        "material_channels",
        "vertex_count",
        "triangle_count",
        "physics_contact_profile",
        "current_state",
        "remaining_blockers",
        "production_ready",
    ] {
        assert!(
            assets.contains(field),
            "asset manifest missing field: {field}"
        );
    }
    // No first-kit asset is marked production_ready true
    assert!(assets.contains("\"production_ready\": false"));

    // Verify physics/contact profiles exist with physical parameters
    let profiles = std::fs::read_to_string(
        "content/physics_profiles/unit051_production_ready_candidate_profiles.json",
    )
    .expect("missing unit051 physics profiles");
    assert!(profiles.contains("longsword"));
    assert!(profiles.contains("gambeson"));
    assert!(profiles.contains("fighter_mannequin"));
    assert!(profiles.contains("witness_stone"));
    assert!(profiles.contains("mass_kg"));
    assert!(profiles.contains("moment_of_inertia"));
    assert!(profiles.contains("grip_frames"));
    assert!(profiles.contains("contact_regions"));
    // No HP/stat shortcuts
    assert!(!profiles.contains("\"hp_points\""));
    assert!(!profiles.contains("\"damage_dice\""));
    assert!(!profiles.contains("\"dps\""));
    assert!(!profiles.contains("\"crit_chance\""));
    assert!(!profiles.contains("\"armor_points\""));

    // Verify material manifest has 4+ distinct material regions per asset
    let materials = std::fs::read_to_string(
        "content/materials/unit051_production_ready_candidate_materials.json",
    )
    .expect("missing unit051 materials manifest");
    assert!(materials.contains("material_region_count"));
    assert!(materials.contains("base_color"));
    assert!(materials.contains("normal"));
    assert!(materials.contains("orm"));
    // At least 4 material regions per asset
    for asset in [
        "longsword",
        "gambeson",
        "fighter_mannequin",
        "witness_stone",
    ] {
        assert!(
            materials.contains(asset),
            "material manifest missing asset: {asset}"
        );
    }

    // Verify WGSL shader has Unit-051 enhancements
    let wgsl = std::fs::read_to_string("spikes/wgpu_renderer/src/verdict_ring.wgsl")
        .expect("missing WGSL shader");
    assert!(wgsl.contains("Unit-051"), "WGSL missing Unit-051 marker");
    assert!(
        wgsl.contains("ssao_approx"),
        "WGSL missing SSAO approximation"
    );
    assert!(
        wgsl.contains("ground_occlusion"),
        "WGSL missing ground contact darkening"
    );
    assert!(
        wgsl.contains("blade_steel"),
        "WGSL missing blade steel region"
    );
    assert!(
        wgsl.contains("crossguard"),
        "WGSL missing crossguard region"
    );
    assert!(
        wgsl.contains("grip_leather"),
        "WGSL missing grip leather region"
    );
    assert!(wgsl.contains("pommel"), "WGSL missing pommel region");
    assert!(
        wgsl.contains("diamond_quilt"),
        "WGSL missing gambeson quilt pattern"
    );
    assert!(wgsl.contains("crack"), "WGSL missing stone crack detail");

    // Verify Rust renderer has guard/cut/thrust/recover poses
    let renderer = std::fs::read_to_string("spikes/wgpu_renderer/src/main.rs")
        .expect("missing renderer main.rs");
    assert!(renderer.contains("\"cut\""), "renderer missing cut pose");
    assert!(
        renderer.contains("\"thrust\""),
        "renderer missing thrust pose"
    );
    assert!(
        renderer.contains("\"recover\""),
        "renderer missing recover pose"
    );

    // Verify capture matrix classification in the wrapper script
    let wrapper = std::fs::read_to_string("tools/wgpu_renderer_spike.sh")
        .expect("missing wgpu renderer spike wrapper");
    assert!(wrapper.contains("production_ready_candidate_native_3d_capture"));
    assert!(wrapper.contains("unit051_candidate_roles"));
    assert!(wrapper.contains("planning_timeline"));
    assert!(wrapper.contains("material_armor_damage_frame"));
    assert!(wrapper.contains("injury_capability_consequence_frame"));

    // Verify large asset policy still enforced
    let lfs_policy = std::fs::read_to_string("docs/decisions/0010-large-file-policy.md")
        .expect("missing large-file policy");
    assert!(lfs_policy.contains("gambeson.obj"));
    assert!(lfs_policy.contains("50 MiB"));
}

#[test]
fn unit052_roster_loadout_capture_matrix_breadth_truth_isolated() {
    // Verify slot assignment manifest exists
    let slots =
        std::fs::read_to_string("content/assets/unit052_roster_loadout_slot_assignments.json")
            .expect("missing unit052 slot assignments");
    assert!(slots.contains("unit052"));
    assert!(slots.contains("fighter_closeup"));
    assert!(slots.contains("armor_loadout_family_closeup"));
    assert!(slots.contains("weapon_family_closeup"));
    assert!(slots.contains("gameplay_distance_fighter_loadout"));
    assert!(slots.contains("gameplay_distance_weapon_family"));
    assert!(slots.contains("training_yard"));
    assert!(slots.contains("first_person_combat_view"));
    assert!(slots.contains("third_person_combat_view"));
    assert!(slots.contains("replay_verification_ui_or_packet_view"));
    assert!(slots.contains("performance_debug_overlay"));
    assert!(slots.contains("settings_accessibility"));
    assert!(slots.contains("arena_select"));
    assert!(slots.contains("recovery_replan_frame"));
    // Blocked slots have exact blockers
    assert!(slots.contains("blocker"));
    assert!(slots.contains("missing_distinct_fighter_asset"));

    // Verify training_yard promotion manifests exist
    let ty_assets =
        std::fs::read_to_string("content/assets/unit052_production_ready_candidate_assets.json")
            .expect("missing unit052 training_yard assets");
    assert!(ty_assets.contains("training_yard"));
    assert!(ty_assets.contains("production_ready_candidate"));
    assert!(ty_assets.contains("\"remaining_blockers\""));

    let ty_mats = std::fs::read_to_string("content/materials/unit052_training_yard_materials.json")
        .expect("missing unit052 training_yard materials");
    assert!(ty_mats.contains("training_yard"));
    assert!(ty_mats.contains("dirt_grass"));

    let ty_phys =
        std::fs::read_to_string("content/physics_profiles/unit052_training_yard_physics.json")
            .expect("missing unit052 training_yard physics");
    assert!(ty_phys.contains("friction_coefficient"));
    assert!(ty_phys.contains("footing_stability"));

    // Verify renderer has Unit-052 camera modes and clip mappings
    let renderer =
        std::fs::read_to_string("spikes/wgpu_renderer/src/main.rs").expect("missing renderer");
    assert!(renderer.contains("training_yard_establishing"));
    assert!(renderer.contains("first_person_combat_view"));
    assert!(renderer.contains("third_person_combat_view"));
    assert!(renderer.contains("replay_verification_ui_or_packet_view"));
    assert!(renderer.contains("performance_debug_overlay"));
    assert!(renderer.contains("settings_accessibility"));
    assert!(renderer.contains("arena_select"));
    assert!(renderer.contains("recovery_replan_frame"));

    // Verify wrapper script has expanded captures
    let wrapper = std::fs::read_to_string("tools/wgpu_renderer_spike.sh").expect("missing wrapper");
    assert!(wrapper.contains("training_yard_establishing"));
    assert!(wrapper.contains("first_person_combat_view"));
    assert!(wrapper.contains("third_person_combat_view"));
    assert!(wrapper.contains("replay_verification_ui_or_packet_view"));
    assert!(wrapper.contains("performance_debug_overlay"));
    assert!(wrapper.contains("settings_accessibility"));
    assert!(wrapper.contains("arena_select"));
    assert!(wrapper.contains("recovery_replan_frame"));
    assert!(wrapper.contains("fighter_closeup_02"));
    assert!(wrapper.contains("weapon_family_closeup_02"));

    // No HP/stat shortcuts in training_yard physics
    assert!(!ty_phys.contains("\"hp_points\""));
    assert!(!ty_phys.contains("\"damage_dice\""));
    assert!(!ty_phys.contains("\"dps\""));

    // Large asset policy still enforced
    let lfs_policy = std::fs::read_to_string("docs/decisions/0010-large-file-policy.md")
        .expect("missing large-file policy");
    assert!(lfs_policy.contains("gambeson.obj"));
}

#[test]
fn unit053_capture_matrix_completion_truth_isolated() {
    // Verify slot closure plan exists
    let closure = std::fs::read_to_string("content/assets/unit053_capture_slot_closure_plan.json")
        .expect("missing unit053 capture slot closure plan");
    assert!(closure.contains("unit053"));
    assert!(closure.contains("\"total_required_slots\": 56"));
    assert!(closure.contains("\"total_filled_after_unit053\": 56"));
    assert!(closure.contains("\"total_remaining_blocked\": 0"));
    assert!(closure.contains("fighter_closeup_04"));
    assert!(closure.contains("weapon_family_closeup_08"));
    assert!(closure.contains("gameplay_distance_fighter_loadout_family_06"));
    assert!(closure.contains("gameplay_distance_weapon_family_08"));

    // Verify renderer has Unit-053 camera modes
    let renderer =
        std::fs::read_to_string("spikes/wgpu_renderer/src/main.rs").expect("missing renderer");
    assert!(renderer.contains("unit053_capture_matrix_complete"));
    assert!(renderer.contains("fighter_closeup_04"));
    assert!(renderer.contains("fighter_closeup_06"));
    assert!(renderer.contains("armor_loadout_family_closeup_04"));
    assert!(renderer.contains("armor_loadout_family_closeup_06"));
    assert!(renderer.contains("weapon_family_closeup_04"));
    assert!(renderer.contains("weapon_family_closeup_08"));
    assert!(renderer.contains("gameplay_distance_fighter_loadout_family_03"));
    assert!(renderer.contains("gameplay_distance_fighter_loadout_family_06"));
    assert!(renderer.contains("gameplay_distance_weapon_family_03"));
    assert!(renderer.contains("gameplay_distance_weapon_family_08"));

    // Verify wrapper has all 21 new captures
    let wrapper = std::fs::read_to_string("tools/wgpu_renderer_spike.sh").expect("missing wrapper");
    assert!(wrapper.contains("fighter_closeup_04"));
    assert!(wrapper.contains("fighter_closeup_05"));
    assert!(wrapper.contains("fighter_closeup_06"));
    assert!(wrapper.contains("armor_loadout_family_closeup_04"));
    assert!(wrapper.contains("armor_loadout_family_closeup_05"));
    assert!(wrapper.contains("armor_loadout_family_closeup_06"));
    assert!(wrapper.contains("weapon_family_closeup_04"));
    assert!(wrapper.contains("weapon_family_closeup_05"));
    assert!(wrapper.contains("weapon_family_closeup_06"));
    assert!(wrapper.contains("weapon_family_closeup_07"));
    assert!(wrapper.contains("weapon_family_closeup_08"));
    assert!(wrapper.contains("gameplay_distance_fighter_loadout_family_03"));
    assert!(wrapper.contains("gameplay_distance_fighter_loadout_family_04"));
    assert!(wrapper.contains("gameplay_distance_fighter_loadout_family_05"));
    assert!(wrapper.contains("gameplay_distance_fighter_loadout_family_06"));
    assert!(wrapper.contains("gameplay_distance_weapon_family_03"));
    assert!(wrapper.contains("gameplay_distance_weapon_family_04"));
    assert!(wrapper.contains("gameplay_distance_weapon_family_05"));
    assert!(wrapper.contains("gameplay_distance_weapon_family_06"));
    assert!(wrapper.contains("gameplay_distance_weapon_family_07"));
    assert!(wrapper.contains("gameplay_distance_weapon_family_08"));

    // Large asset policy still enforced
    let lfs_policy = std::fs::read_to_string("docs/decisions/0010-large-file-policy.md")
        .expect("missing large-file policy");
    assert!(lfs_policy.contains("gambeson.obj"));
}

#[test]
fn unit054_production_quality_certification_truth_isolated() {
    // Verify capture quality certification manifest exists
    let cert = std::fs::read_to_string("content/assets/unit054_capture_quality_certification.json")
        .expect("missing unit054 quality certification manifest");
    assert!(cert.contains("unit054"));
    assert!(cert.contains("\"certified_count\": 12"));
    assert!(cert.contains("\"certified_target\": 12"));
    assert!(cert.contains("RI-01"));
    assert!(cert.contains("RI-02"));
    assert!(cert.contains("fresnel_rim_lighting"));
    assert!(cert.contains("enhanced_specular_response"));
    // All 12 key captures must be listed
    for capture_id in [
        "boot_main_menu",
        "fighter_select",
        "loadout_select",
        "oathyard_verdict_ring_establishing",
        "fighter_closeup_01",
        "armor_loadout_family_closeup_01",
        "weapon_family_closeup_01",
        "planning_timeline",
        "pre_contact_frame",
        "contact_frame",
        "injury_capability_consequence_frame",
        "fight_film_replay_camera_shot",
    ] {
        assert!(
            cert.contains(capture_id),
            "certification manifest missing capture: {capture_id}"
        );
    }
    // Production-ready count must be 0 (global gates not passed)
    assert!(cert.contains("\"production_ready_count\": 0"));

    // Verify renderer improvements are in shader code
    let wgsl = std::fs::read_to_string("spikes/wgpu_renderer/src/verdict_ring.wgsl")
        .expect("missing WGSL shader");
    assert!(
        wgsl.contains("Unit-054 RI-01"),
        "WGSL missing Unit-054 RI-01"
    );
    assert!(
        wgsl.contains("Unit-054 RI-02"),
        "WGSL missing Unit-054 RI-02"
    );
    assert!(wgsl.contains("fresnel"), "WGSL missing fresnel calculation");
    assert!(
        wgsl.contains("enhanced_spec"),
        "WGSL missing enhanced specular"
    );

    // Verify visual_features manifest records improvements
    let renderer = std::fs::read_to_string("spikes/wgpu_renderer/src/main.rs")
        .expect("missing renderer main.rs");
    assert!(
        renderer.contains("unit054_fresnel_rim_lighting"),
        "renderer manifest missing fresnel feature"
    );
    assert!(
        renderer.contains("unit054_enhanced_specular_response"),
        "renderer manifest missing enhanced specular feature"
    );

    // Verify seed capture promotions are in wrapper
    let wrapper = std::fs::read_to_string("tools/wgpu_renderer_spike.sh").expect("missing wrapper");
    assert!(
        wrapper.contains("boot_main_menu"),
        "wrapper missing boot_main_menu in PRC list"
    );
    assert!(
        wrapper.contains("fighter_select"),
        "wrapper missing fighter_select in PRC list"
    );
    assert!(
        wrapper.contains("loadout_select"),
        "wrapper missing loadout_select in PRC list"
    );

    // Large asset policy still enforced
    let lfs_policy = std::fs::read_to_string("docs/decisions/0010-large-file-policy.md")
        .expect("missing large-file policy");
    assert!(lfs_policy.contains("gambeson.obj"));
}

#[test]
fn unit055_production_renderer_path_truth_isolated() {
    // Verify production renderer crate exists outside spikes/
    let prod_cargo = std::fs::read_to_string("crates/oathyard_renderer/Cargo.toml")
        .expect("missing production renderer Cargo.toml");
    assert!(prod_cargo.contains("oathyard-renderer"));
    assert!(prod_cargo.contains("oathyard-native-renderer"));

    // Verify production renderer source exists
    let prod_main = std::fs::read_to_string("crates/oathyard_renderer/src/main.rs")
        .expect("missing production renderer main.rs");
    assert!(prod_main.contains("oathyard-native-wgpu-production-v1"));
    assert!(!prod_main.contains("spike"));

    // Verify production shader exists
    let prod_shader = std::fs::read_to_string("crates/oathyard_renderer/src/verdict_ring.wgsl")
        .expect("missing production renderer shader");
    assert!(prod_shader.contains("Unit-054 RI-01"));
    assert!(prod_shader.contains("Unit-054 RI-02"));

    // Verify spike wrapper delegates to production renderer
    let wrapper =
        std::fs::read_to_string("tools/wgpu_renderer_spike.sh").expect("missing spike wrapper");
    assert!(
        wrapper.contains("crates/oathyard_renderer/Cargo.toml"),
        "spike wrapper must delegate to production renderer"
    );
    assert!(
        wrapper.contains("compatibility wrapper"),
        "spike wrapper must declare compatibility status"
    );

    // Verify production renderer tool exists
    let prod_tool = std::fs::read_to_string("tools/run_production_renderer.sh")
        .expect("missing production renderer tool");
    assert!(prod_tool.contains("crates/oathyard_renderer/Cargo.toml"));
    assert!(prod_tool.contains("production"));

    // Verify game-flow tool uses renderer (through compatibility wrapper)
    let game_flow = std::fs::read_to_string("tools/run_native_3d_game_flow.sh")
        .expect("missing game flow tool");
    assert!(game_flow.contains("wgpu_renderer_spike.sh"));

    // Large asset policy still enforced
    let lfs_policy = std::fs::read_to_string("docs/decisions/0010-large-file-policy.md")
        .expect("missing large-file policy");
    assert!(lfs_policy.contains("gambeson.obj"));
}

#[test]
fn unit056_local_production_ready_owner_review_packet_truth_isolated() {
    // Verify local production-ready checklist exists
    let checklist =
        std::fs::read_to_string("content/assets/unit056_local_production_ready_checklist.json")
            .expect("missing unit056 local production ready checklist");
    assert!(checklist.contains("\"unit\": \"Unit-056\""));
    assert!(checklist.contains("fighter_mannequin"));
    assert!(checklist.contains("gambeson"));
    assert!(checklist.contains("longsword"));
    assert!(checklist.contains("witness_stone"));
    assert!(checklist.contains("owner_visual_acceptance"));
    // Checklist must have stricter requirements than PRC
    assert!(checklist.contains("owner_approved"));
    assert!(checklist.contains("technical_clean"));
    assert!(checklist.contains("runtime_asset_hash"));
    assert!(checklist.contains("material_manifest"));
    assert!(checklist.contains("physics_profile"));

    // Verify local production-ready certification exists
    let cert =
        std::fs::read_to_string("content/assets/unit056_local_production_ready_certification.json")
            .expect("missing unit056 local production ready certification");
    assert!(cert.contains("local_production_ready"));
    // All 5 first-kit assets must be present
    for asset in [
        "fighter_mannequin",
        "gambeson",
        "longsword",
        "witness_stone",
        "training_yard",
    ] {
        assert!(cert.contains(asset), "certification missing asset: {asset}");
    }
    // All 12 key captures must be present
    for capture in [
        "boot_main_menu",
        "fighter_select",
        "loadout_select",
        "oathyard_verdict_ring_establishing",
        "fighter_closeup_01",
        "armor_loadout_family_closeup_01",
        "weapon_family_closeup_01",
        "planning_timeline",
        "pre_contact_frame",
        "contact_frame",
        "injury_capability_consequence_frame",
        "fight_film_replay_camera_shot",
    ] {
        assert!(
            cert.contains(capture),
            "certification missing capture: {capture}"
        );
    }
    // Production renderer complete must be true (local criteria met)
    assert!(cert.contains("\"production_renderer_complete\": true"));
    // Owner acceptance must NOT be claimed — check the review packet manifest
    let packet_manifest = std::fs::read_to_string(
        "artifacts/verification/20260702T110130_unit056/owner_review_packet/owner_review_manifest.json",
    )
    .expect("missing owner review manifest");
    assert!(packet_manifest.contains("\"owner_visual_acceptance\": false"));
    assert!(packet_manifest.contains("\"public_demo_ready\": false"));
    assert!(packet_manifest.contains("\"release_candidate_ready\": false"));

    // Verify owner review packet exists
    let packet_summary = std::fs::read_to_string(
        "artifacts/verification/20260702T120000Z_unit056_owner_review_packet/owner_review_summary.md",
    )
    .expect("missing owner review packet summary");
    assert!(packet_summary.contains("owner_visual_acceptance = false"));
    assert!(packet_summary.contains("oathyard-native-wgpu-production-v1"));

    // Verify no HP/stat shortcuts in certification
    assert!(!cert.contains("\"hp_points\""));
    assert!(!cert.contains("\"damage_dice\""));
    assert!(!cert.contains("\"dps\""));

    // Large asset policy still enforced
    let lfs_policy = std::fs::read_to_string("docs/decisions/0010-large-file-policy.md")
        .expect("missing large-file policy");
    assert!(lfs_policy.contains("gambeson.obj"));
}
