use oathyard::{
    build_hud_truth_view_model_from_replay_text, build_hud_truth_view_model_from_result,
    run_scenario_text, verify_replay_text, HudTruthMutationAttempt, HUD_NATIVE_FLOW_IDS,
};

const BASIC: &str = include_str!("../examples/duels/basic_oathyard.duel");

#[test]
fn hud_view_rejects_stale_content_hash_before_exposing_truth() {
    let result = run_scenario_text(BASIC).expect("scenario run");
    let expected = format!("\"content_hash\": \"{}\"", result.content_hash);
    let stale =
        result
            .replay_json
            .replacen(&expected, "\"content_hash\": \"stale-content-hash\"", 1);

    assert_ne!(
        result.replay_json, stale,
        "test fixture must alter the replay content hash"
    );
    let error = build_hud_truth_view_model_from_replay_text(&stale)
        .expect_err("HUD truth view must reject stale content before exposing UI truth");
    assert!(
        error.to_string().contains("content hash mismatch"),
        "expected content-hash verification failure, got {error}"
    );
}

#[test]
fn hud_view_exposes_frame_cost_physical_reasons_for_native_flows() {
    let result = run_scenario_text(BASIC).expect("scenario run");
    let view = build_hud_truth_view_model_from_result(&result).expect("hash-gated HUD view");

    assert_eq!(view.schema, "oathyard.hud_truth_view.v1");
    assert!(view.content_hash_verified);
    assert!(!view.truth_mutation);
    assert_eq!(view.content_hash, result.content_hash);
    assert_eq!(view.final_state_hash, result.final_state_hash);
    assert!(!view.cache_key.is_empty());

    assert!(
        view.flows.len() >= 9,
        "view model must cover all requested native UI flows"
    );
    for flow_id in HUD_NATIVE_FLOW_IDS {
        assert!(
            view.flows.iter().any(|flow| flow.flow_id == flow_id),
            "missing HUD flow view for {flow_id}"
        );
    }

    assert!(!view.frame_costs.is_empty());
    assert!(
        view.frame_costs
            .iter()
            .any(|cost| cost.current_cost_frames != cost.base_cost_frames),
        "at least one physical modifier must change current frame cost from base"
    );
    for cost in &view.frame_costs {
        assert!(cost.base_cost_frames > 0);
        assert!(cost.current_cost_frames >= cost.base_cost_frames);
        assert!(
            !cost.physical_reasons.is_empty(),
            "cost entry must expose physical reasons"
        );
        for reason in &cost.physical_reasons {
            assert!(!reason.category.is_empty());
            assert!(reason.permille > 0);
            assert!(!reason.reason.is_empty());
        }
    }
}

#[test]
fn hud_view_is_read_only_noop_for_ui_truth_mutation_attempts() {
    let result = run_scenario_text(BASIC).expect("scenario run");
    let mut view = build_hud_truth_view_model_from_result(&result).expect("hash-gated HUD view");
    let before = view.clone();

    let error = view
        .reject_truth_mutation_attempt(HudTruthMutationAttempt {
            flow_id: "plan".to_string(),
            requested_change: "rewrite_final_state_hash".to_string(),
        })
        .expect_err("UI truth mutation attempts must hard-error");

    assert!(
        error.to_string().contains("read-only"),
        "expected read-only mutation error, got {error}"
    );
    assert_eq!(
        view, before,
        "mutation rejection must leave HUD view unchanged"
    );

    let replayed = verify_replay_text(&result.replay_json).expect("replay still verifies");
    assert_eq!(replayed.final_state_hash, result.final_state_hash);
}
