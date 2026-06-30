use std::fs;
use std::path::{Path, PathBuf};

use oathyard::{
    asset_may_declare_combat_truth_authority, evaluate_freeze_conditions,
    may_declare_combat_truth_authority, query_combat_truth_freeze_status, query_freeze_status,
    scoped_asset_may_declare_authority, verify_registry_content_hash, BoundaryFreezeState,
    FreezeConditionName, FreezeState, RegistryEntry,
};

fn test_root(name: &str) -> PathBuf {
    let root = Path::new("target/tmp/freeze_status_tests").join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("test root");
    root
}

fn write_registry_entry(
    root: &Path,
    authority_scope: &str,
    asset_id: &str,
    conditions: BoundaryFreezeState,
) {
    let entry_dir = root
        .join("artifacts/freeze/v1/index/by_scope")
        .join(authority_scope);
    fs::create_dir_all(&entry_dir).expect("registry entry dir");
    let entry = format!(
        "{{\n  \"schema\": \"oathyard.freeze_registry_entry.v1\",\n  \"asset_id\": \"{asset_id}\",\n  \"authority_scope\": \"{authority_scope}\",\n  \"content_hash\": \"sha256:testfixture\",\n  \"conditions\": {{\n    \"frozen\": {},\n    \"deterministic\": {},\n    \"hashed\": {},\n    \"replayable\": {},\n    \"cross_platform_verified\": {}\n  }}\n}}\n",
        conditions.frozen,
        conditions.deterministic,
        conditions.hashed,
        conditions.replayable,
        conditions.cross_platform_verified,
    );
    fs::write(entry_dir.join(format!("{asset_id}.json")), entry).expect("registry entry");
}

#[test]
fn all_five_pass_allows_combat_truth_authority() {
    let root = test_root("all_five_pass");
    write_registry_entry(
        &root,
        "combat_truth",
        "motion_candidate_001",
        BoundaryFreezeState {
            frozen: true,
            deterministic: true,
            hashed: true,
            replayable: true,
            cross_platform_verified: true,
        },
    );

    let status =
        query_combat_truth_freeze_status(&root, "motion_candidate_001").expect("freeze status");

    assert!(status.found_in_registry);
    assert_eq!(status.freeze_state, FreezeState::AuthoritativePostFreeze);
    assert!(status.overall_verdict);
    assert!(status.combat_truth_authority_allowed);
    assert!(may_declare_combat_truth_authority(&status));
    assert_eq!(
        asset_may_declare_combat_truth_authority(&root, "motion_candidate_001")
            .expect("authority query"),
        true
    );
    assert!(status
        .condition_results
        .iter()
        .all(|condition| condition.passed));
}

#[test]
fn partial_pass_reports_failed_conditions_and_blocks_authority() {
    let root = test_root("partial_pass");
    write_registry_entry(
        &root,
        "combat_truth",
        "armor_plate_ai_draft",
        BoundaryFreezeState {
            frozen: true,
            deterministic: true,
            hashed: true,
            replayable: false,
            cross_platform_verified: false,
        },
    );

    let status =
        query_freeze_status(&root, "combat_truth", "armor_plate_ai_draft").expect("freeze status");

    assert!(status.found_in_registry);
    assert_eq!(status.freeze_state, FreezeState::PreFreeze);
    assert!(!status.overall_verdict);
    assert!(!status.combat_truth_authority_allowed);
    assert!(!may_declare_combat_truth_authority(&status));
    assert_eq!(
        asset_may_declare_combat_truth_authority(&root, "armor_plate_ai_draft")
            .expect("asset authority query"),
        false
    );
    assert_eq!(
        scoped_asset_may_declare_authority(&root, "combat_truth", "armor_plate_ai_draft")
            .expect("scoped authority query"),
        false
    );
    assert_eq!(
        status
            .condition_results
            .iter()
            .filter(|condition| !condition.passed)
            .map(|condition| condition.name)
            .collect::<Vec<_>>(),
        vec![
            FreezeConditionName::Replayable,
            FreezeConditionName::CrossPlatformVerified,
        ]
    );
}

#[test]
fn every_single_failed_condition_blocks_combat_truth_authority() {
    let root = test_root("single_failed_conditions");
    let cases = [
        (
            "missing_frozen",
            FreezeConditionName::Frozen,
            BoundaryFreezeState {
                frozen: false,
                deterministic: true,
                hashed: true,
                replayable: true,
                cross_platform_verified: true,
            },
        ),
        (
            "missing_deterministic",
            FreezeConditionName::Deterministic,
            BoundaryFreezeState {
                frozen: true,
                deterministic: false,
                hashed: true,
                replayable: true,
                cross_platform_verified: true,
            },
        ),
        (
            "missing_hashed",
            FreezeConditionName::Hashed,
            BoundaryFreezeState {
                frozen: true,
                deterministic: true,
                hashed: false,
                replayable: true,
                cross_platform_verified: true,
            },
        ),
        (
            "missing_replayable",
            FreezeConditionName::Replayable,
            BoundaryFreezeState {
                frozen: true,
                deterministic: true,
                hashed: true,
                replayable: false,
                cross_platform_verified: true,
            },
        ),
        (
            "missing_cross_platform_verified",
            FreezeConditionName::CrossPlatformVerified,
            BoundaryFreezeState {
                frozen: true,
                deterministic: true,
                hashed: true,
                replayable: true,
                cross_platform_verified: false,
            },
        ),
    ];

    for (asset_id, failed_condition, conditions) in cases {
        write_registry_entry(&root, "combat_truth", asset_id, conditions);

        let status = query_combat_truth_freeze_status(&root, asset_id).expect("freeze status");

        assert!(
            status.found_in_registry,
            "{asset_id} should use registry stub"
        );
        assert_eq!(status.freeze_state, FreezeState::PreFreeze, "{asset_id}");
        assert!(!status.overall_verdict, "{asset_id}");
        assert!(!status.combat_truth_authority_allowed, "{asset_id}");
        assert!(!may_declare_combat_truth_authority(&status), "{asset_id}");
        assert_eq!(
            asset_may_declare_combat_truth_authority(&root, asset_id)
                .expect("asset authority query"),
            false,
            "{asset_id}"
        );
        assert_eq!(
            status
                .condition_results
                .iter()
                .filter(|condition| !condition.passed)
                .map(|condition| condition.name)
                .collect::<Vec<_>>(),
            vec![failed_condition],
            "{asset_id} should report only the failing condition"
        );
    }
}

#[test]
fn non_combat_scope_status_does_not_grant_combat_truth_authority() {
    let root = test_root("non_combat_scope");
    write_registry_entry(
        &root,
        "runtime_presentation",
        "motion_vfx_candidate",
        BoundaryFreezeState {
            frozen: true,
            deterministic: true,
            hashed: true,
            replayable: true,
            cross_platform_verified: true,
        },
    );

    let status = query_freeze_status(&root, "runtime_presentation", "motion_vfx_candidate")
        .expect("non-combat freeze status");

    assert!(status.found_in_registry);
    assert_eq!(status.freeze_state, FreezeState::AuthoritativePostFreeze);
    assert!(status.overall_verdict);
    assert!(!status.combat_truth_authority_allowed);
    assert!(!may_declare_combat_truth_authority(&status));
    assert_eq!(
        scoped_asset_may_declare_authority(&root, "runtime_presentation", "motion_vfx_candidate")
            .expect("scoped authority query"),
        true
    );
}

#[test]
fn not_found_in_registry_returns_structured_pre_freeze_block() {
    let root = test_root("not_found");

    let status = query_freeze_status(&root, "combat_truth", "missing_ai_asset")
        .expect("missing assets produce structured status, not CLI text");

    assert!(!status.found_in_registry);
    assert_eq!(status.asset_id, "missing_ai_asset");
    assert_eq!(status.authority_scope, "combat_truth");
    assert_eq!(status.freeze_state, FreezeState::PreFreeze);
    assert_eq!(
        status.conditions,
        BoundaryFreezeState {
            frozen: false,
            deterministic: false,
            hashed: false,
            replayable: false,
            cross_platform_verified: false,
        }
    );
    assert!(!status.overall_verdict);
    assert!(!status.combat_truth_authority_allowed);
    assert!(!may_declare_combat_truth_authority(&status));
    assert_eq!(
        asset_may_declare_combat_truth_authority(&root, "missing_ai_asset")
            .expect("missing asset authority query"),
        false
    );
    assert!(status
        .condition_results
        .iter()
        .all(|condition| !condition.passed));
}

#[test]
fn pure_condition_evaluation_reports_all_five_conditions() {
    let failed = evaluate_freeze_conditions(BoundaryFreezeState {
        frozen: false,
        deterministic: false,
        hashed: false,
        replayable: false,
        cross_platform_verified: false,
    });

    assert_eq!(failed.freeze_state, FreezeState::PreFreeze);
    assert!(!failed.overall_verdict);
    assert_eq!(failed.condition_results.len(), 5);
    assert!(failed
        .condition_results
        .iter()
        .all(|condition| !condition.passed));

    let passed = evaluate_freeze_conditions(BoundaryFreezeState {
        frozen: true,
        deterministic: true,
        hashed: true,
        replayable: true,
        cross_platform_verified: true,
    });

    assert_eq!(passed.freeze_state, FreezeState::AuthoritativePostFreeze);
    assert!(passed.overall_verdict);
    assert_eq!(
        passed
            .condition_results
            .iter()
            .map(|condition| condition.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "frozen",
            "deterministic",
            "hashed",
            "replayable",
            "cross_platform_verified",
        ]
    );
    assert!(passed
        .condition_results
        .iter()
        .all(|condition| condition.passed));
}

// ── R-HASH-1: content_hash verification tests ──

/// Write a registry entry with an arbitrary content_hash string,
/// and optionally write a matching artifact file at the expected manifest path.
fn write_registry_entry_with_hash_and_artifact(
    root: &Path,
    authority_scope: &str,
    asset_id: &str,
    conditions: BoundaryFreezeState,
    content_hash_hex: &str,
    artifact_content: Option<&str>,
) {
    let entry_dir = root
        .join("artifacts/freeze/v1/index/by_scope")
        .join(authority_scope);
    fs::create_dir_all(&entry_dir).expect("registry entry dir");
    let entry = format!(
        concat!(
            "{{\n",
            "  \"schema\": \"oathyard.freeze_registry_entry.v1\",\n",
            "  \"asset_id\": \"{}\",\n",
            "  \"authority_scope\": \"{}\",\n",
            "  \"content_hash\": \"sha256:{}\",\n",
            "  \"conditions\": {{\n",
            "    \"frozen\": {},\n",
            "    \"deterministic\": {},\n",
            "    \"hashed\": {},\n",
            "    \"replayable\": {},\n",
            "    \"cross_platform_verified\": {}\n",
            "  }}\n",
            "}}\n",
        ),
        asset_id,
        authority_scope,
        content_hash_hex,
        conditions.frozen,
        conditions.deterministic,
        conditions.hashed,
        conditions.replayable,
        conditions.cross_platform_verified,
    );
    fs::write(entry_dir.join(format!("{asset_id}.json")), entry).expect("registry entry");

    if let Some(content) = artifact_content {
        let manifest_dir = root
            .join("artifacts/freeze/v1/manifests")
            .join(authority_scope);
        fs::create_dir_all(&manifest_dir).expect("manifest dir");
        fs::write(manifest_dir.join(format!("{asset_id}.manifest")), content)
            .expect("write artifact");
    }
}

#[test]
fn content_hash_mismatch_blocks_freeze_status() {
    let root = test_root("hash_mismatch");
    let all_pass = BoundaryFreezeState {
        frozen: true,
        deterministic: true,
        hashed: true,
        replayable: true,
        cross_platform_verified: true,
    };

    // Registry declares all conditions pass, but content_hash is wrong.
    // The artifact file exists with different content — hash mismatch.
    write_registry_entry_with_hash_and_artifact(
        &root,
        "combat_truth",
        "ai:hash_mismatch_asset",
        all_pass,
        "0000000000000000000000000000000000000000000000000000000000000000",
        Some("actual artifact content that does not match the declared hash\n"),
    );

    let status =
        query_combat_truth_freeze_status(&root, "ai:hash_mismatch_asset").expect("freeze status");

    assert!(status.found_in_registry);
    // Hash mismatch forces hashed=false, blocking the overall verdict.
    assert!(!status.overall_verdict);
    assert!(!status.combat_truth_authority_allowed);
    assert_eq!(status.freeze_state, FreezeState::PreFreeze);

    // The hashed condition should be forced to fail.
    let hashed_result = status
        .condition_results
        .iter()
        .find(|c| c.name == FreezeConditionName::Hashed)
        .expect("hashed condition");
    assert!(!hashed_result.passed);
}

#[test]
fn content_hash_match_allows_freeze_when_all_conditions_pass() {
    let root = test_root("hash_match");

    // Write an artifact, compute its real hash, then create a registry entry
    // with the correct hash.
    let artifact_content = "verified artifact content\n";
    let real_hash = oathyard::sha256::sha256_hex(artifact_content.as_bytes());

    let all_pass = BoundaryFreezeState {
        frozen: true,
        deterministic: true,
        hashed: true,
        replayable: true,
        cross_platform_verified: true,
    };

    write_registry_entry_with_hash_and_artifact(
        &root,
        "combat_truth",
        "ai:hash_match_asset",
        all_pass,
        &real_hash,
        Some(artifact_content),
    );

    let status =
        query_combat_truth_freeze_status(&root, "ai:hash_match_asset").expect("freeze status");

    assert!(status.found_in_registry);
    assert!(status.overall_verdict, "should pass with matching hash");
    assert!(status.combat_truth_authority_allowed);
    assert_eq!(status.freeze_state, FreezeState::AuthoritativePostFreeze);
}

#[test]
fn verify_registry_content_hash_detects_mismatch_directly() {
    let root = test_root("hash_verify_direct");
    let artifact_content = "some artifact data\n";
    let real_hash = oathyard::sha256::sha256_hex(artifact_content.as_bytes());
    let wrong_hash = "deadbeef".repeat(8);

    write_registry_entry_with_hash_and_artifact(
        &root,
        "combat_truth",
        "ai:direct_verify",
        BoundaryFreezeState {
            frozen: true,
            deterministic: true,
            hashed: true,
            replayable: true,
            cross_platform_verified: true,
        },
        &wrong_hash,
        Some(artifact_content),
    );

    let registry_text = fs::read_to_string(
        root.join("artifacts/freeze/v1/index/by_scope/combat_truth/ai:direct_verify.json"),
    )
    .expect("read registry");
    let entry = RegistryEntry::parse(&registry_text).expect("parse entry");

    let result = verify_registry_content_hash(&root, &entry).expect("verify");
    assert!(!result.matches);
    assert_eq!(result.computed_hash, real_hash);
    assert!(result.detail.contains("MISMATCH"));

    // Now test with correct hash
    write_registry_entry_with_hash_and_artifact(
        &root,
        "combat_truth",
        "ai:direct_verify",
        BoundaryFreezeState {
            frozen: true,
            deterministic: true,
            hashed: true,
            replayable: true,
            cross_platform_verified: true,
        },
        &real_hash,
        Some(artifact_content),
    );

    let registry_text = fs::read_to_string(
        root.join("artifacts/freeze/v1/index/by_scope/combat_truth/ai:direct_verify.json"),
    )
    .expect("read registry");
    let entry = RegistryEntry::parse(&registry_text).expect("parse entry");

    let result = verify_registry_content_hash(&root, &entry).expect("verify");
    assert!(result.matches);
    assert!(result.detail.contains("verified"));
}

// ── R-HASH-3: JSON string injection test ──

#[test]
fn json_string_injection_cannot_bypass_boolean_parser() {
    let root = test_root("json_injection");

    // Craft a registry entry where a string value contains the text
    // '"frozen": true' inside the notes field. The structural JSON parser
    // must NOT interpret this as the actual conditions.frozen boolean.
    let entry_dir = root.join("artifacts/freeze/v1/index/by_scope/combat_truth");
    fs::create_dir_all(&entry_dir).expect("registry dir");

    // The real conditions.frozen is false, but a string field contains
    // '"frozen": true' to try to fool a naive string-search parser.
    let malicious_entry = r#"{
  "schema": "oathyard.freeze_registry_entry.v1",
  "asset_id": "ai:injection_test",
  "authority_scope": "combat_truth",
  "content_hash": "sha256:placeholder",
  "notes": "this string contains \"frozen\": true to trick naive parsers",
  "decoy": "\"conditions\": { \"frozen\": true, \"deterministic\": true, \"hashed\": true, \"replayable\": true, \"cross_platform_verified\": true }",
  "conditions": {
    "frozen": false,
    "deterministic": false,
    "hashed": false,
    "replayable": false,
    "cross_platform_verified": false
  }
}
"#;
    fs::write(entry_dir.join("ai:injection_test.json"), malicious_entry)
        .expect("write malicious entry");

    let status =
        query_combat_truth_freeze_status(&root, "ai:injection_test").expect("freeze status");

    assert!(status.found_in_registry);
    // All real conditions are false — the injection must not bypass them.
    assert!(!status.overall_verdict);
    assert!(!status.combat_truth_authority_allowed);
    assert_eq!(status.freeze_state, FreezeState::PreFreeze);
    assert!(status.condition_results.iter().all(|c| !c.passed));
}

// ── R-HASH-2: freeze CLI command integration test ──

#[test]
fn freeze_cli_command_creates_valid_registry_entry() {
    use std::process::Command;

    let root = test_root("cli_freeze_command");

    // Create an artifact file at the conventional path
    // (manifests/<scope>/<asset_id>.manifest)
    let artifact_dir = root.join("artifacts/freeze/v1/manifests/combat_truth");
    fs::create_dir_all(&artifact_dir).expect("artifact dir");
    let artifact_path = artifact_dir.join("ai:cli_test.manifest");
    let artifact_content = "cli test artifact content for freeze pipeline\n";
    fs::write(&artifact_path, artifact_content).expect("write artifact");
    let expected_hash = oathyard::sha256::sha256_hex(artifact_content.as_bytes());

    // Run: oathyard freeze --repo-root <root> --asset-id ai:cli_test --artifact <path> --scenario <duel> --xplat-platform linux --xplat-platform windows
    let output = Command::new(env!("CARGO_BIN_EXE_oathyard"))
        .args([
            "freeze",
            "--repo-root",
            root.to_str().expect("utf8 root"),
            "--asset-id",
            "ai:cli_test",
            "--artifact",
            artifact_path.to_str().expect("utf8 artifact path"),
            "--scenario",
            "examples/duels/basic_oathyard.duel",
            "--xplat-platform",
            "linux",
            "--xplat-platform",
            "windows",
        ])
        .output()
        .expect("run oathyard freeze command");

    assert!(
        output.status.success(),
        "freeze command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify CLI output structure
    assert!(stdout.contains("OATHYARD freeze pipeline complete"));
    assert!(stdout.contains("asset_id=ai:cli_test"));
    assert!(stdout.contains("authority_scope=combat_truth"));
    assert!(stdout.contains(&format!("content_hash=sha256:{expected_hash}")));
    assert!(stdout.contains("overall_passed=true"));

    // Each step should pass
    assert!(stdout.contains("[PASS] content_hash"));
    assert!(stdout.contains("[PASS] deterministic_double_run"));
    assert!(stdout.contains("[PASS] replay_verification"));
    assert!(stdout.contains("[PASS] cross_platform_verified"));
    assert!(stdout.contains("[PASS] registry_write"));

    // Verify the registry entry was actually written to disk
    let registry_path = root
        .join("artifacts/freeze/v1/index/by_scope/combat_truth")
        .join("ai:cli_test.json");
    let entry_text = fs::read_to_string(&registry_path)
        .expect("registry entry should exist after freeze command");

    // Parse it back and verify integrity
    let entry = RegistryEntry::parse(&entry_text).expect("parse written registry entry");
    assert_eq!(entry.asset_id, "ai:cli_test");
    assert_eq!(entry.authority_scope, "combat_truth");
    assert_eq!(entry.content_hash, format!("sha256:{expected_hash}"));
    assert!(entry.conditions.all_conditions_passed());

    // Verify the hash matches the artifact
    let hash_result = verify_registry_content_hash(&root, &entry).expect("verify hash");
    assert!(hash_result.matches, "content hash should match artifact");

    // Verify the entry is queryable via query_freeze_status
    let status = query_combat_truth_freeze_status(&root, "ai:cli_test").expect("query");
    assert!(status.found_in_registry);
    assert!(status.overall_verdict);
    assert!(status.combat_truth_authority_allowed);
    assert_eq!(status.freeze_state, FreezeState::AuthoritativePostFreeze);
}
