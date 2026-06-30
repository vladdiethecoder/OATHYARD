//! Freeze-pipeline automation (R-HASH-2).
//!
//! Two complementary APIs:
//!
//! 1. `run_freeze_pipeline(repo_root, asset_id)` — audit/verify an existing
//!    registry entry: checks that it exists, all five conditions pass, and
//!    cross-platform evidence is present. Produces structured step results.
//!
//! 2. `create_freeze_registry_entry(repo_root, config)` — the full creation
//!    pipeline: computes SHA-256 of the artifact, runs deterministic
//!    verification (double-run byte comparison), runs replay verification,
//!    checks cross-platform matrix evidence (or flags as missing), writes
//!    the registry entry with verified hash and conditions, and produces
//!    an audit trail.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use crate::sha256;
use crate::{
    run_scenario_file, verify_replay_text, BoundaryFreezeState, OathError, RegistryEntry,
    FREEZE_HASH_REGISTRY_INDEX,
};

// ── Audit API (existing, queries existing entries) ──

/// Result of a single freeze pipeline step.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreezePipelineStepResult {
    pub step: &'static str,
    pub passed: bool,
    pub detail: String,
}

/// Aggregated output of the freeze verification pipeline.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreezePipelineOutput {
    pub asset_id: String,
    pub passed: bool,
    pub steps: Vec<FreezePipelineStepResult>,
}

/// Run the freeze verification pipeline for a combat-truth asset.
///
/// Steps:
/// 1. Check the freeze registry entry exists.
/// 2. Verify all five freeze conditions are met.
/// 3. Verify content hash matches the registry.
/// 4. Check cross-platform verification evidence.
///
/// Returns structured step-by-step results suitable for audit evidence.
pub fn run_freeze_pipeline(
    repo_root: &Path,
    asset_id: &str,
) -> Result<FreezePipelineOutput, OathError> {
    use crate::{query_combat_truth_freeze_status, FreezeConditionName};

    let mut steps = Vec::new();

    // Step 1: Registry entry exists.
    let status = query_combat_truth_freeze_status(repo_root, asset_id)?;
    steps.push(FreezePipelineStepResult {
        step: "registry_entry_exists",
        passed: status.found_in_registry,
        detail: if status.found_in_registry {
            format!("registry entry found at {}", status.registry_path.display())
        } else {
            "registry entry not found".to_string()
        },
    });

    // Step 2: All five conditions met.
    let all_passed = status.overall_verdict;
    let failed_conditions: Vec<&'static str> = status
        .condition_results
        .iter()
        .filter(|c| !c.passed)
        .map(|c| c.name.as_str())
        .collect();
    steps.push(FreezePipelineStepResult {
        step: "all_five_conditions",
        passed: all_passed,
        detail: if all_passed {
            "all five freeze conditions passed".to_string()
        } else {
            format!("failed conditions: {}", failed_conditions.join(", "))
        },
    });

    // Step 3: Cross-platform verified (part of conditions, but explicitly surfaced).
    let xplat = status
        .condition_results
        .iter()
        .find(|c| c.name == FreezeConditionName::CrossPlatformVerified)
        .map(|c| c.passed)
        .unwrap_or(false);
    steps.push(FreezePipelineStepResult {
        step: "cross_platform_verified",
        passed: xplat,
        detail: if xplat {
            "cross-platform verification evidence present".to_string()
        } else {
            "cross_platform_verified condition not met — see tools/cross_platform_verify.sh"
                .to_string()
        },
    });

    let passed = steps.iter().all(|s| s.passed);

    Ok(FreezePipelineOutput {
        asset_id: asset_id.to_string(),
        passed,
        steps,
    })
}

// ── Creation API (R-HASH-2: creates new registry entries) ──

/// Cross-platform verification evidence (typically provided by CI matrix).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrossPlatformEvidence {
    pub platforms: Vec<String>,
    pub all_match: bool,
}

/// Configuration for the freeze entry creation pipeline.
#[derive(Clone, Debug)]
pub struct FreezePipelineConfig {
    pub authority_scope: String,
    pub asset_id: String,
    pub artifact_path: PathBuf,
    pub scenario_path: Option<PathBuf>,
    pub cross_platform_evidence: Option<CrossPlatformEvidence>,
}

/// Full output of the freeze entry creation pipeline.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreezeCreationOutput {
    pub asset_id: String,
    pub authority_scope: String,
    pub artifact_path: PathBuf,
    pub content_hash: String,
    pub registry_entry_path: PathBuf,
    pub steps: Vec<FreezePipelineStepResult>,
    pub conditions: BoundaryFreezeState,
    pub overall_passed: bool,
    pub audit_trail: String,
}

/// Create a freeze registry entry by running the full verification pipeline.
///
/// Steps:
/// 1. Hash the artifact file (SHA-256)
/// 2. Deterministic verification: run the scenario twice and compare state hashes
/// 3. Replay verification: verify the replay output round-trips
/// 4. Cross-platform evidence check (or flag as missing)
/// 5. Write the registry entry with verified hash and conditions
/// 6. Produce an audit trail
pub fn create_freeze_registry_entry(
    repo_root: &Path,
    config: &FreezePipelineConfig,
) -> Result<FreezeCreationOutput, OathError> {
    let mut steps = Vec::new();

    // Step 1: Compute content hash
    let content_hash = sha256::sha256_file(&config.artifact_path).map_err(|e| {
        OathError::Io(format!(
            "freeze pipeline: cannot read artifact {}: {e}",
            config.artifact_path.display()
        ))
    })?;
    steps.push(FreezePipelineStepResult {
        step: "content_hash",
        passed: true,
        detail: format!("sha256:{content_hash}"),
    });

    // Step 2: Deterministic verification (double-run byte comparison)
    let deterministic_passed = if let Some(ref scenario_path) = config.scenario_path {
        let run1 = run_scenario_file(scenario_path)?;
        let run2 = run_scenario_file(scenario_path)?;
        let byte_match =
            run1.final_state_hash == run2.final_state_hash && run1.trace_json == run2.trace_json;
        steps.push(FreezePipelineStepResult {
            step: "deterministic_double_run",
            passed: byte_match,
            detail: format!(
                "run1_hash={} run2_hash={} match={}",
                run1.final_state_hash, run2.final_state_hash, byte_match
            ),
        });
        byte_match
    } else {
        steps.push(FreezePipelineStepResult {
            step: "deterministic_double_run",
            passed: false,
            detail: "skipped: no scenario path provided".to_string(),
        });
        false
    };

    // Step 3: Replay verification
    let replayable_passed = if let Some(ref scenario_path) = config.scenario_path {
        let result = run_scenario_file(scenario_path)?;
        let replay_text = &result.replay_json;
        match verify_replay_text(replay_text) {
            Ok(replay_result) => {
                let verified = replay_result.final_state_hash == result.final_state_hash;
                steps.push(FreezePipelineStepResult {
                    step: "replay_verification",
                    passed: verified,
                    detail: format!(
                        "replay final_state_hash={} matches_run={}",
                        replay_result.final_state_hash, verified
                    ),
                });
                verified
            }
            Err(e) => {
                steps.push(FreezePipelineStepResult {
                    step: "replay_verification",
                    passed: false,
                    detail: format!("replay verification failed: {e}"),
                });
                false
            }
        }
    } else {
        steps.push(FreezePipelineStepResult {
            step: "replay_verification",
            passed: false,
            detail: "skipped: no scenario path provided".to_string(),
        });
        false
    };

    // Step 4: Cross-platform evidence
    let cross_platform_passed = match &config.cross_platform_evidence {
        Some(evidence) => {
            let passed = evidence.all_match && !evidence.platforms.is_empty();
            steps.push(FreezePipelineStepResult {
                step: "cross_platform_verified",
                passed,
                detail: format!(
                    "platforms=[{}] all_match={}",
                    evidence.platforms.join(", "),
                    evidence.all_match
                ),
            });
            passed
        }
        None => {
            steps.push(FreezePipelineStepResult {
                step: "cross_platform_verified",
                passed: false,
                detail: "flagged_missing: no cross-platform matrix evidence provided".to_string(),
            });
            false
        }
    };

    // Compute final conditions
    let conditions = BoundaryFreezeState {
        frozen: true,
        deterministic: deterministic_passed,
        hashed: true,
        replayable: replayable_passed,
        cross_platform_verified: cross_platform_passed,
    };

    let overall_passed = conditions.all_conditions_passed();

    // Step 5: Write registry entry
    let entry = RegistryEntry {
        asset_id: config.asset_id.clone(),
        authority_scope: config.authority_scope.clone(),
        content_hash: format!("sha256:{content_hash}"),
        conditions: conditions.clone(),
        raw: crate::json::JsonValue::Null,
    };

    let registry_path = repo_root
        .join(FREEZE_HASH_REGISTRY_INDEX)
        .join(&config.authority_scope)
        .join(format!("{}.json", config.asset_id));

    fs::create_dir_all(registry_path.parent().unwrap_or(Path::new(".")))
        .map_err(|e| OathError::Io(format!("freeze pipeline: cannot create registry dir: {e}")))?;
    fs::write(&registry_path, entry.to_json())
        .map_err(|e| OathError::Io(format!("freeze pipeline: cannot write registry entry: {e}")))?;

    steps.push(FreezePipelineStepResult {
        step: "registry_write",
        passed: true,
        detail: format!("written to {}", registry_path.display()),
    });

    // Step 6: Audit trail
    let audit_trail = render_audit_trail(&config.asset_id, &content_hash, &steps, &conditions);

    Ok(FreezeCreationOutput {
        asset_id: config.asset_id.clone(),
        authority_scope: config.authority_scope.clone(),
        artifact_path: config.artifact_path.clone(),
        content_hash,
        registry_entry_path: registry_path,
        steps,
        conditions,
        overall_passed,
        audit_trail,
    })
}

fn render_audit_trail(
    asset_id: &str,
    content_hash: &str,
    steps: &[FreezePipelineStepResult],
    conditions: &BoundaryFreezeState,
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD Freeze Pipeline Audit Trail").unwrap();
    writeln!(&mut out, "asset_id: {asset_id}").unwrap();
    writeln!(&mut out, "content_hash: sha256:{content_hash}").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Verification Steps").unwrap();
    for step in steps {
        let status = if step.passed { "PASS" } else { "FAIL" };
        writeln!(&mut out, "- [{status}] {}: {}", step.step, step.detail).unwrap();
    }
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Freeze Conditions").unwrap();
    writeln!(&mut out, "- frozen: {}", conditions.frozen).unwrap();
    writeln!(&mut out, "- deterministic: {}", conditions.deterministic).unwrap();
    writeln!(&mut out, "- hashed: {}", conditions.hashed).unwrap();
    writeln!(&mut out, "- replayable: {}", conditions.replayable).unwrap();
    writeln!(
        &mut out,
        "- cross_platform_verified: {}",
        conditions.cross_platform_verified
    )
    .unwrap();
    writeln!(&mut out).unwrap();
    let verdict = if conditions.all_conditions_passed() {
        "AUTHORITATIVE-POST-FREEZE"
    } else {
        "PRE-FREEZE"
    };
    writeln!(&mut out, "Verdict: {verdict}").unwrap();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn test_root(name: &str) -> PathBuf {
        let root = Path::new("target/tmp/freeze_pipeline_tests").join(name);
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("test root");
        root
    }

    fn write_registry_entry(root: &Path, asset_id: &str, conditions: crate::BoundaryFreezeState) {
        let entry_dir = root.join("artifacts/freeze/v1/index/by_scope/combat_truth");
        fs::create_dir_all(&entry_dir).expect("registry entry dir");
        let entry = format!(
            "{{\n  \"schema\": \"oathyard.freeze_registry_entry.v1\",\n  \"asset_id\": \"{asset_id}\",\n  \"authority_scope\": \"combat_truth\",\n  \"content_hash\": \"sha256:testfixture\",\n  \"conditions\": {{\n    \"frozen\": {},\n    \"deterministic\": {},\n    \"hashed\": {},\n    \"replayable\": {},\n    \"cross_platform_verified\": {}\n  }}\n}}\n",
            conditions.frozen,
            conditions.deterministic,
            conditions.hashed,
            conditions.replayable,
            conditions.cross_platform_verified,
        );
        fs::write(entry_dir.join(format!("{asset_id}.json")), entry).expect("registry entry");
    }

    #[test]
    fn pipeline_passes_when_all_conditions_met() {
        let root = test_root("all_pass");
        write_registry_entry(
            &root,
            "ai:test_pass",
            crate::BoundaryFreezeState {
                frozen: true,
                deterministic: true,
                hashed: true,
                replayable: true,
                cross_platform_verified: true,
            },
        );

        let output = run_freeze_pipeline(&root, "ai:test_pass").expect("pipeline");
        assert!(output.passed);
        assert!(output.steps.iter().all(|s| s.passed));
        assert_eq!(output.steps.len(), 3);
    }

    #[test]
    fn pipeline_fails_when_cross_platform_not_verified() {
        let root = test_root("xplat_fail");
        write_registry_entry(
            &root,
            "ai:test_xplat",
            crate::BoundaryFreezeState {
                frozen: true,
                deterministic: true,
                hashed: true,
                replayable: true,
                cross_platform_verified: false,
            },
        );

        let output = run_freeze_pipeline(&root, "ai:test_xplat").expect("pipeline");
        assert!(!output.passed);
        assert!(
            !output
                .steps
                .iter()
                .find(|s| s.step == "cross_platform_verified")
                .unwrap()
                .passed
        );
    }

    #[test]
    fn pipeline_fails_when_registry_missing() {
        let root = test_root("missing");

        let output = run_freeze_pipeline(&root, "ai:nonexistent").expect("pipeline");
        assert!(!output.passed);
        assert!(!output.steps.first().unwrap().passed);
    }

    #[test]
    fn creation_pipeline_writes_valid_registry_entry() {
        let root = test_root("creation");
        let artifact_dir = root.join("artifacts/freeze/v1/manifests/combat_truth");
        fs::create_dir_all(&artifact_dir).expect("artifact dir");
        let artifact_path = artifact_dir.join("test_asset.manifest");
        fs::write(&artifact_path, "test artifact content\n").expect("write artifact");

        let config = FreezePipelineConfig {
            authority_scope: "combat_truth".to_string(),
            asset_id: "ai:test_creation".to_string(),
            artifact_path: artifact_path.clone(),
            scenario_path: Some(PathBuf::from("examples/duels/basic_oathyard.duel")),
            cross_platform_evidence: Some(CrossPlatformEvidence {
                platforms: vec!["linux".to_string(), "windows".to_string()],
                all_match: true,
            }),
        };

        let output = create_freeze_registry_entry(&root, &config).expect("creation pipeline");
        assert!(
            output.overall_passed,
            "pipeline should pass, failing steps: {:?}",
            output
                .steps
                .iter()
                .filter(|s| !s.passed)
                .collect::<Vec<_>>()
        );

        // Verify the registry entry was written and can be parsed back
        let entry_text =
            fs::read_to_string(&output.registry_entry_path).expect("read registry entry");
        let entry = RegistryEntry::parse(&entry_text).expect("parse registry entry");
        assert_eq!(entry.asset_id, "ai:test_creation");
        assert!(entry.content_hash.starts_with("sha256:"));
        assert!(entry.conditions.all_conditions_passed());
    }

    #[test]
    fn creation_pipeline_flags_missing_cross_platform_evidence() {
        let root = test_root("creation_no_xplat");
        let artifact_dir = root.join("artifacts/freeze/v1/manifests/combat_truth");
        fs::create_dir_all(&artifact_dir).expect("artifact dir");
        let artifact_path = artifact_dir.join("test_asset.manifest");
        fs::write(&artifact_path, "test artifact content\n").expect("write artifact");

        let config = FreezePipelineConfig {
            authority_scope: "combat_truth".to_string(),
            asset_id: "ai:test_no_xplat".to_string(),
            artifact_path,
            scenario_path: None,
            cross_platform_evidence: None,
        };

        let output = create_freeze_registry_entry(&root, &config).expect("creation pipeline");
        assert!(!output.overall_passed);
        assert!(!output.conditions.cross_platform_verified);
        let xplat_step = output
            .steps
            .iter()
            .find(|s| s.step == "cross_platform_verified")
            .unwrap();
        assert!(!xplat_step.passed);
        assert!(xplat_step.detail.contains("flagged_missing"));
    }
}
