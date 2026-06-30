use oathyard::{
    build_native_hud_menu_flow_model, drive_native_hud_menu_flow,
    native_hud_menu_flow_replay_status, run_scenario_text, NativeHudFlowCommand,
    HUD_NATIVE_FLOW_IDS,
};

const BASIC: &str = include_str!("../examples/duels/basic_oathyard.duel");

#[test]
fn native_flow_model_covers_all_required_screens_from_hash_gated_truth() {
    let result = run_scenario_text(BASIC).expect("scenario run");
    let model = build_native_hud_menu_flow_model(&result).expect("native HUD/menu flow model");

    assert_eq!(model.schema, "oathyard.native_hud_menu_flow.v1");
    assert!(model.content_hash_verified);
    assert!(model.presentation_only);
    assert!(!model.truth_mutation);
    assert_eq!(model.content_hash, result.content_hash);
    assert_eq!(model.final_state_hash, result.final_state_hash);
    assert_eq!(model.screens.len(), HUD_NATIVE_FLOW_IDS.len());

    for flow_id in HUD_NATIVE_FLOW_IDS {
        let screen = model.screen(flow_id).expect("screen present");
        assert_eq!(screen.flow_id, flow_id);
        assert!(screen.native_code_path.starts_with("native::"));
        assert!(!screen.input_action.is_empty());
        assert!(!screen.truth_cache_key.is_empty());
        assert!(screen.read_only_truth);
        assert!(!screen.truth_mutation);
        assert!(screen.base_cost_frames > 0, "{flow_id} missing base cost");
        assert!(
            screen.current_cost_frames >= screen.base_cost_frames,
            "{flow_id} current cost regressed below base"
        );
        assert!(
            !screen.physical_reasons.is_empty(),
            "{flow_id} missing physical reasons"
        );
        assert!(
            screen
                .physical_reasons
                .iter()
                .any(|reason| reason.contains("x") && reason.contains(":")),
            "{flow_id} physical reasons must expose factor and explanation"
        );
        let combined = format!("{} {}", screen.headline, screen.detail);
        for forbidden in [
            "hit points",
            "damage bonus",
            "armor points",
            "crit chance",
            "super meter",
        ] {
            assert!(
                !combined.to_ascii_lowercase().contains(forbidden),
                "{flow_id} surfaced forbidden shortcut text {forbidden}"
            );
        }
    }
}

#[test]
fn native_flow_commands_navigate_end_to_end_without_truth_writes() {
    let result = run_scenario_text(BASIC).expect("scenario run");
    let model = build_native_hud_menu_flow_model(&result).expect("native HUD/menu flow model");
    let run = drive_native_hud_menu_flow(
        &model,
        &[
            NativeHudFlowCommand::MainMenuStart,
            NativeHudFlowCommand::OpenSettingsAccessibility,
            NativeHudFlowCommand::ApplySettingsEdit,
            NativeHudFlowCommand::ResetSettings,
            NativeHudFlowCommand::OpenFighterSelect,
            NativeHudFlowCommand::SelectNextFighter,
            NativeHudFlowCommand::OpenLoadoutSelect,
            NativeHudFlowCommand::SelectNextLoadout,
            NativeHudFlowCommand::OpenObserve,
            NativeHudFlowCommand::OpenPlan,
            NativeHudFlowCommand::CommitReveal,
            NativeHudFlowCommand::Resolve,
            NativeHudFlowCommand::Consequence,
            NativeHudFlowCommand::OpenReplayBrowser,
            NativeHudFlowCommand::OpenReplay,
            NativeHudFlowCommand::OpenFightFilm,
            NativeHudFlowCommand::ScrubFightFilmForward,
            NativeHudFlowCommand::TogglePerformanceDebugOverlay,
            NativeHudFlowCommand::BackToMainMenu,
        ],
    )
    .expect("drive native flow");

    for flow_id in HUD_NATIVE_FLOW_IDS {
        assert!(
            run.visited_flow_ids
                .iter()
                .any(|visited| visited == flow_id),
            "command sequence did not visit {flow_id}"
        );
    }
    assert_eq!(run.truth_hash_before, result.final_state_hash);
    assert_eq!(run.truth_hash_after, result.final_state_hash);
    assert!(run.presentation_only);
    assert!(!run.truth_mutation);
    assert!(run.replay_opened);
    assert!(run.fight_film_timeline_position > 0);
    assert!(run.debug_overlay_visible);
    assert!(run.settings_profile_state.contains("reset"));
    assert!(run.selected_fighter_card.contains("seat"));
    assert!(run.selected_loadout_card.contains("weapon="));
}

#[test]
fn replay_browser_status_verifies_and_surfaces_corrupt_replays_loudly() {
    let result = run_scenario_text(BASIC).expect("scenario run");
    let ok = native_hud_menu_flow_replay_status(&result.replay_json);
    assert!(ok.verified);
    assert!(!ok.loud_failure);
    assert_eq!(
        ok.final_state_hash.as_deref(),
        Some(result.final_state_hash.as_str())
    );

    let corrupt =
        result
            .replay_json
            .replacen(&result.final_state_hash, "corrupt-final-state-hash", 1);
    let bad = native_hud_menu_flow_replay_status(&corrupt);
    assert!(!bad.verified);
    assert!(bad.loud_failure);
    assert!(
        bad.error_message
            .as_deref()
            .unwrap_or("")
            .contains("mismatch")
            || bad
                .error_message
                .as_deref()
                .unwrap_or("")
                .contains("content hash")
            || bad
                .error_message
                .as_deref()
                .unwrap_or("")
                .contains("final state")
    );
}
