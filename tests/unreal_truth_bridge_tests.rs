use oathyard::{
    verify_unreal_frame_packet_chain, UnrealCapturedFrame, UnrealPostHashBridgeConfig,
    UnrealTruthPacketBridge, TRUTH_HZ, UNREAL_CAPTURE_HOOK_SCHEMA,
    UNREAL_POST_HASH_PACKET_CHAIN_SCHEMA,
};

fn config() -> UnrealPostHashBridgeConfig {
    UnrealPostHashBridgeConfig {
        scenario_id: "unreal_bridge_synthetic_duel".to_string(),
        content_hash: "content_hash_synthetic_001".to_string(),
        final_state_hash: "final_state_hash_synthetic_001".to_string(),
        replay_json_sha256: "11".repeat(32),
        trace_json_sha256: "22".repeat(32),
        presentation_packet_sha256: "33".repeat(32),
        unreal_engine_version: "5.8.0-installed".to_string(),
        unreal_project_id: "OathyardUE".to_string(),
        renderer_backend_id: "unreal-5.8-vulkan-capture-harness".to_string(),
        generated_after_replay_verify: true,
    }
}

fn frame(
    sequence_index: u32,
    truth_frame: u32,
    capture_id: &str,
    capture_sha256: &str,
) -> UnrealCapturedFrame {
    UnrealCapturedFrame {
        sequence_index,
        truth_frame,
        scheduled_ms: truth_frame * 1000 / TRUTH_HZ,
        capture_id: capture_id.to_string(),
        capture_file: format!("artifacts/unreal/captures/{capture_id}.png"),
        width: 1920,
        height: 1080,
        pixel_format: "rgba8_srgb_png".to_string(),
        camera_mode: "third_person_verdict_ring".to_string(),
        render_pass: "lit_nanite_lumen_reference".to_string(),
        capture_sha256: capture_sha256.to_string(),
        source_truth_hash: "final_state_hash_synthetic_001".to_string(),
    }
}

#[test]
fn unreal_bridge_hash_chain_is_reproducible_and_truth_isolated() {
    let frames = [
        frame(0, 0, "observe_plan_000", &"aa".repeat(32)),
        frame(1, 120, "contact_frame_001", &"bb".repeat(32)),
        frame(2, 240, "consequence_002", &"cc".repeat(32)),
    ];

    let mut first = UnrealTruthPacketBridge::new(config()).expect("valid post-hash config");
    for captured in frames.iter().cloned() {
        first
            .record_captured_frame(captured)
            .expect("synthetic Unreal frame accepted");
    }
    let first_json = first.render_packet_chain_json();
    let first_hash = first.final_chain_hash().to_string();
    assert_eq!(
        verify_unreal_frame_packet_chain(&config(), first.packets()).expect("chain verifies"),
        first_hash
    );

    let mut second = UnrealTruthPacketBridge::new(config()).expect("valid post-hash config");
    for captured in frames {
        second
            .record_captured_frame(captured)
            .expect("synthetic Unreal frame accepted");
    }

    assert_eq!(first_hash, second.final_chain_hash());
    assert_eq!(first_json, second.render_packet_chain_json());
    assert!(first_json.contains(UNREAL_POST_HASH_PACKET_CHAIN_SCHEMA));
    assert!(first_json.contains(UNREAL_CAPTURE_HOOK_SCHEMA));
    assert!(first_json.contains("\"post_hash_only\": true"));
    assert!(first_json.contains("\"presentation_only\": true"));
    assert!(first_json.contains("\"truth_mutation\": false"));
    assert!(first_json.contains("\"generated_after_replay_verify\": true"));
    assert!(first_json.contains("\"truth_tick_rate_hz\": 120"));
    assert!(first_json.contains("\"frame_count\": 3"));
    assert!(first_json.contains(
        "\"deterministic_ordering\": \"sequence_index_then_truth_frame_then_capture_id\""
    ));
    assert!(first_json.contains(
        "\"chain_algorithm\": \"sha256(previous_chain_hash + canonical_frame_packet_json)\""
    ));
}

#[test]
fn unreal_bridge_rejects_non_deterministic_hook_inputs() {
    let mut bridge = UnrealTruthPacketBridge::new(config()).expect("valid post-hash config");

    let err = bridge
        .record_captured_frame(frame(1, 120, "wrong_first_index", &"aa".repeat(32)))
        .expect_err("first hook call must start at sequence 0");
    assert!(err
        .to_string()
        .contains("expected Unreal frame sequence_index 0, got 1"));

    let mut absolute = frame(0, 0, "absolute_path", &"aa".repeat(32));
    absolute.capture_file = "/tmp/unreal/absolute_path.png".to_string();
    let err = bridge
        .record_captured_frame(absolute)
        .expect_err("capture paths must be repo-relative for reproducible packets");
    assert!(err
        .to_string()
        .contains("Unreal capture_file must be relative and artifact-scoped"));

    let mut bad_replay_gate = config();
    bad_replay_gate.generated_after_replay_verify = false;
    let err = UnrealTruthPacketBridge::new(bad_replay_gate)
        .expect_err("bridge must not initialize before replay verification");
    assert!(err
        .to_string()
        .contains("Unreal bridge requires generated_after_replay_verify=true"));
}

#[test]
fn unreal_bridge_verifier_rejects_tampered_packet_material() {
    let mut bridge = UnrealTruthPacketBridge::new(config()).expect("valid post-hash config");
    bridge
        .record_captured_frame(frame(0, 0, "observe_plan_000", &"aa".repeat(32)))
        .expect("frame accepted");
    bridge
        .record_captured_frame(frame(1, 120, "contact_frame_001", &"bb".repeat(32)))
        .expect("frame accepted");

    let mut tampered_frame = bridge.packets().to_vec();
    tampered_frame[1].capture_sha256 = "00".repeat(32);
    let err = verify_unreal_frame_packet_chain(&config(), &tampered_frame)
        .expect_err("changed capture bytes must invalidate the packet hash");
    assert!(err
        .to_string()
        .contains("Unreal packet hash mismatch at sequence 1"));

    let mut tampered_link = bridge.packets().to_vec();
    tampered_link[1].previous_chain_hash = "ff".repeat(32);
    let err = verify_unreal_frame_packet_chain(&config(), &tampered_link)
        .expect_err("changed previous chain hash must invalidate the link");
    assert!(err
        .to_string()
        .contains("Unreal previous chain hash mismatch at sequence 1"));
}
