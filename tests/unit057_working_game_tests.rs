// Unit-057: Working-game integration tests.
//
// Verify the local playable game path: state machine order, plan cycles,
// truth isolation, replay verify, opponent legality, UI data binding.
//
// AiPolicyStyle/AiPlan are pub(crate) and not accessible here; we verify
// their effects through the public LocalGameRun/GameStateEntry API.

use oathyard::{write_local_game_artifacts, GameState, GameStateEntry, LocalGameConfig};

fn required_states() -> Vec<GameState> {
    vec![
        GameState::Boot,
        GameState::MainMenu,
        GameState::ModeSelect,
        GameState::FighterSelect,
        GameState::LoadoutSelect,
        GameState::ArenaSelect,
        GameState::MatchIntro,
        GameState::Observe,
        GameState::Plan,
        GameState::CommitReveal,
        GameState::Resolve,
        GameState::Consequence,
        GameState::Replan,
        GameState::MatchResult,
        GameState::ReplayBrowser,
        GameState::FightFilmView,
        GameState::Settings,
        GameState::Quit,
    ]
}

struct AssetGenerationTestLock {
    path: std::path::PathBuf,
}

impl Drop for AssetGenerationTestLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn acquire_asset_generation_test_lock() -> AssetGenerationTestLock {
    let path =
        std::path::Path::new("target/tmp/oathyard_unit057_working_game_test.lock").to_path_buf();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    AssetGenerationTestLock { path }
}

#[test]
fn unit057_working_local_game_loop_truth_isolated() {
    let _lock = acquire_asset_generation_test_lock();
    let tmp = std::path::Path::new("target/tmp/unit057_working_local_game_loop_truth_isolated");
    let _ = std::fs::remove_dir_all(tmp);
    std::fs::create_dir_all(tmp).unwrap();

    let config = LocalGameConfig::default();
    let run = write_local_game_artifacts(tmp, config).expect("working local game failed");

    // 1. Game state machine visits required states.
    let required = required_states();
    for state in &required {
        assert!(
            run.states.iter().any(|e| e.state == *state),
            "required state {:?} not visited",
            state
        );
    }

    // 2. Scripted local game completes a match.
    assert!(
        run.result.end_condition.status.len() > 0,
        "end_condition.status empty"
    );
    assert!(
        run.result.final_state_hash.len() > 0,
        "final_state_hash empty"
    );

    // 3. At least two plan/resolve cycles.
    assert!(run.plan_cycles >= 2, "plan_cycles={} < 2", run.plan_cycles);

    // 4. UI data comes from trace/timeline/cost data — Plan states have non-empty action labels.
    let plan_states: Vec<&GameStateEntry> = run
        .states
        .iter()
        .filter(|s| s.state == GameState::Plan)
        .collect();
    assert!(
        plan_states.len() >= 2,
        "expected >=2 Plan states, got {}",
        plan_states.len()
    );
    for ps in &plan_states {
        assert!(
            ps.player_action_label.len() > 0,
            "Plan state missing player_action_label (UI data not bound to trace)"
        );
        assert!(
            ps.player_direction.len() > 0,
            "Plan state missing player_direction"
        );
        assert!(
            ps.player_target.len() > 0,
            "Plan state missing player_target"
        );
        assert!(
            ps.player_base_cost_frames > 0,
            "Plan state missing player_base_cost_frames"
        );
    }

    // 5. Opponent emits legal planned actions only (verified by AI planner in lib.rs;
    //    here we confirm the manifest records non-empty opponent_action_label for Plan states).
    for ps in &plan_states {
        assert!(
            ps.opponent_action_label.len() > 0,
            "Plan state missing opponent_action_label"
        );
        assert!(
            ps.opponent_direction.len() > 0,
            "Plan state missing opponent_direction"
        );
        assert!(
            ps.opponent_target.len() > 0,
            "Plan state missing opponent_target"
        );
    }

    // 6. Replay after match verifies the same final hash.
    assert!(run.replay_verified, "replay not verified");
    assert!(
        run.replay_hash_matches,
        "replay final_state_hash does not match observed"
    );

    // 7. Fight-film view consumes trace/replay data — fight_film_view_manifest.json exists
    //    and has shots array referencing turn hashes.
    let fight_film_view_manifest = tmp.join("fight_film_view_manifest.json");
    assert!(
        fight_film_view_manifest.exists(),
        "fight_film_view_manifest.json missing"
    );
    let fight_film_view_text = std::fs::read_to_string(fight_film_view_manifest).unwrap();
    assert!(
        fight_film_view_text.contains("\"shots\""),
        "fight_film_view_manifest.json missing shots array"
    );
    assert!(
        fight_film_view_text.contains("\"truth_hash\""),
        "fight_film_view_manifest.json shots missing truth_hash binding"
    );

    // 8. Presentation/UI does not mutate truth — every manifest asserts truth_mutation=false.
    let game_flow_manifest = tmp.join("game_flow_manifest.json");
    let scripted_input_manifest = tmp.join("scripted_input_manifest.json");
    assert!(
        game_flow_manifest.exists(),
        "game_flow_manifest.json missing"
    );
    assert!(
        scripted_input_manifest.exists(),
        "scripted_input_manifest.json missing"
    );

    let game_flow_text = std::fs::read_to_string(game_flow_manifest).unwrap();
    let scripted_input_text = std::fs::read_to_string(scripted_input_manifest).unwrap();

    for (path, text) in [
        ("game_flow_manifest.json", &game_flow_text),
        ("scripted_input_manifest.json", &scripted_input_text),
        ("fight_film_view_manifest.json", &fight_film_view_text),
    ] {
        assert!(
            text.contains("\"truth_mutation\": false"),
            "{} does not assert truth_mutation=false",
            path
        );
    }

    // 9. local_playable_game_ready is true when all gates pass.
    assert!(
        run.local_playable_game_ready,
        "local_playable_game_ready should be true when all gates pass"
    );

    // 10. No forbidden 2D visual fallback formats in manifests.
    for (path, text) in [
        ("game_flow_manifest.json", &game_flow_text),
        ("scripted_input_manifest.json", &scripted_input_text),
        ("fight_film_view_manifest.json", &fight_film_view_text),
    ] {
        assert!(
            !text.contains(".svg"),
            "{} contains .svg (forbidden 2D fallback)",
            path
        );
        assert!(
            !text.contains("image/svg"),
            "{} contains image/svg (forbidden 2D fallback)",
            path
        );
        assert!(
            !text.contains("<canvas"),
            "{} contains <canvas (forbidden 2D fallback)",
            path
        );
        assert!(
            !text.contains("data:text/html"),
            "{} contains data:text/html (forbidden browser fallback)",
            path
        );
    }

    let _ = std::fs::remove_dir_all(tmp);
}
