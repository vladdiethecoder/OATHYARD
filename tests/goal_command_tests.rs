use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const CANON: &str = include_str!("../docs/design/GAME_CANON.md");

#[derive(Clone, Copy, Debug)]
struct FreezeFixture {
    frozen: bool,
    deterministic: bool,
    hashed: bool,
    replayable: bool,
    cross_platform_verified: bool,
}

impl FreezeFixture {
    const fn all_passed() -> Self {
        Self {
            frozen: true,
            deterministic: true,
            hashed: true,
            replayable: true,
            cross_platform_verified: true,
        }
    }

    const fn missing(condition: FreezeConditionForFixture) -> Self {
        match condition {
            FreezeConditionForFixture::Frozen => Self {
                frozen: false,
                deterministic: true,
                hashed: true,
                replayable: true,
                cross_platform_verified: true,
            },
            FreezeConditionForFixture::Deterministic => Self {
                frozen: true,
                deterministic: false,
                hashed: true,
                replayable: true,
                cross_platform_verified: true,
            },
            FreezeConditionForFixture::Hashed => Self {
                frozen: true,
                deterministic: true,
                hashed: false,
                replayable: true,
                cross_platform_verified: true,
            },
            FreezeConditionForFixture::Replayable => Self {
                frozen: true,
                deterministic: true,
                hashed: true,
                replayable: false,
                cross_platform_verified: true,
            },
            FreezeConditionForFixture::CrossPlatformVerified => Self {
                frozen: true,
                deterministic: true,
                hashed: true,
                replayable: true,
                cross_platform_verified: false,
            },
        }
    }

    const fn none_passed() -> Self {
        Self {
            frozen: false,
            deterministic: false,
            hashed: false,
            replayable: false,
            cross_platform_verified: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum FreezeConditionForFixture {
    Frozen,
    Deterministic,
    Hashed,
    Replayable,
    CrossPlatformVerified,
}

impl FreezeConditionForFixture {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Frozen => "frozen",
            Self::Deterministic => "deterministic",
            Self::Hashed => "hashed",
            Self::Replayable => "replayable",
            Self::CrossPlatformVerified => "cross_platform_verified",
        }
    }
}

fn test_root(name: &str) -> PathBuf {
    let root = Path::new("target/tmp/goal_command_tests").join(name);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("docs/design")).expect("docs dir");
    fs::write(root.join("docs/design/GAME_CANON.md"), CANON).expect("canon fixture");
    root
}

fn write_registry_entry(
    root: &Path,
    authority_scope: &str,
    asset_id: &str,
    conditions: FreezeFixture,
) {
    let entry_dir = root
        .join("artifacts/freeze/v1/index/by_scope")
        .join(authority_scope);
    fs::create_dir_all(&entry_dir).expect("registry entry dir");
    let entry = format!(
        "{{\n  \"schema\": \"oathyard.freeze_registry_entry.v1\",\n  \"asset_id\": \"{asset_id}\",\n  \"authority_scope\": \"{authority_scope}\",\n  \"content_hash\": \"sha256:testfixture\",\n  \"conditions\": {{\n    \"frozen\": {frozen},\n    \"deterministic\": {deterministic},\n    \"hashed\": {hashed},\n    \"replayable\": {replayable},\n    \"cross_platform_verified\": {cross_platform_verified}\n  }}\n}}\n",
        frozen = conditions.frozen,
        deterministic = conditions.deterministic,
        hashed = conditions.hashed,
        replayable = conditions.replayable,
        cross_platform_verified = conditions.cross_platform_verified,
    );
    fs::write(entry_dir.join(format!("{asset_id}.json")), entry).expect("registry entry");
}

fn run_goal(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_oathyard"))
        .args(args)
        .output()
        .expect("run oathyard goal command")
}

fn successful_stdout(output: Output) -> String {
    assert!(
        output.status.success(),
        "goal command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("utf8 stdout")
}

fn asset_block<'a>(stdout: &'a str, asset_id: &str) -> &'a str {
    let needle = format!("asset_id={asset_id}\n");
    let start = stdout
        .find(&needle)
        .unwrap_or_else(|| panic!("missing block for asset {asset_id}\n{stdout}"));
    let rest = &stdout[start..];
    let end = rest.find("\n\n").unwrap_or(rest.len());
    &rest[..end]
}

#[test]
fn goal_command_displays_policy_freeze_enforcement_and_all_taxonomy_tags() {
    let root = test_root("goal_output_contract_all_taxonomy_tags");
    write_registry_entry(
        &root,
        "combat_truth",
        "motion_candidate_001",
        FreezeFixture::all_passed(),
    );
    write_registry_entry(
        &root,
        "combat_truth",
        "armor_plate_ai_draft",
        FreezeFixture::missing(FreezeConditionForFixture::Replayable),
    );
    write_registry_entry(
        &root,
        "combat_truth",
        "hand_authored_rule_table",
        FreezeFixture::none_passed(),
    );

    let stdout = successful_stdout(run_goal(&[
        "goal",
        "--repo-root",
        root.to_str().expect("utf8 root"),
        "--asset",
        "motion_candidate_001",
        "--asset",
        "armor_plate_ai_draft",
        "--non-ai-asset",
        "hand_authored_rule_table",
    ]));

    assert!(stdout.contains("OATHYARD /goal frontier policy summary"));
    assert!(stdout.contains("source=docs/design/GAME_CANON.md"));
    assert!(stdout.contains("OATHYARD should actively use frontier AI"));
    assert!(stdout.contains("runtime_authoritative_truth: forbidden by default"));
    assert!(stdout.contains("Frozen, Deterministic, Hashed, Replayable, Cross-platform verified"));

    let frozen_asset = asset_block(&stdout, "motion_candidate_001");
    assert!(frozen_asset.contains("found_in_registry=true"));
    assert!(frozen_asset.contains("freeze_state=authoritative-post-freeze"));
    assert!(frozen_asset.contains("combat_truth_authority_allowed=true"));
    assert!(frozen_asset.contains("taxonomy_label=authoritative-post-freeze"));
    assert!(frozen_asset.contains(
        "conditions=frozen=pass,deterministic=pass,hashed=pass,replayable=pass,cross_platform_verified=pass"
    ));

    let partial_asset = asset_block(&stdout, "armor_plate_ai_draft");
    assert!(partial_asset.contains("found_in_registry=true"));
    assert!(partial_asset.contains("freeze_state=pre-freeze"));
    assert!(partial_asset.contains("combat_truth_authority_allowed=false"));
    assert!(partial_asset.contains("taxonomy_label=AI-assisted-pre-freeze"));
    assert!(partial_asset.contains("replayable=fail"));
    assert!(
        partial_asset.contains("WARNING: combat-truth authority blocked for armor_plate_ai_draft")
    );
    assert!(partial_asset.contains("failed_conditions=replayable"));

    let non_ai_asset = asset_block(&stdout, "hand_authored_rule_table");
    assert!(non_ai_asset.contains("found_in_registry=true"));
    assert!(non_ai_asset.contains("freeze_state=pre-freeze"));
    assert!(non_ai_asset.contains("combat_truth_authority_allowed=false"));
    assert!(non_ai_asset.contains("taxonomy_label=purely-deterministic-never-AI"));
    assert!(non_ai_asset.contains(
        "conditions=frozen=fail,deterministic=fail,hashed=fail,replayable=fail,cross_platform_verified=fail"
    ));
}

#[test]
fn goal_command_warns_and_blocks_each_individual_missing_freeze_condition() {
    let root = test_root("goal_missing_each_individual_condition");
    let missing_cases = [
        (
            "candidate_missing_frozen",
            FreezeConditionForFixture::Frozen,
        ),
        (
            "candidate_missing_deterministic",
            FreezeConditionForFixture::Deterministic,
        ),
        (
            "candidate_missing_hashed",
            FreezeConditionForFixture::Hashed,
        ),
        (
            "candidate_missing_replayable",
            FreezeConditionForFixture::Replayable,
        ),
        (
            "candidate_missing_cross_platform_verified",
            FreezeConditionForFixture::CrossPlatformVerified,
        ),
    ];

    for (asset_id, missing_condition) in missing_cases {
        write_registry_entry(
            &root,
            "combat_truth",
            asset_id,
            FreezeFixture::missing(missing_condition),
        );
    }

    let stdout = successful_stdout(run_goal(&[
        "goal",
        "--repo-root",
        root.to_str().expect("utf8 root"),
        "--asset",
        "candidate_missing_frozen",
        "--asset",
        "candidate_missing_deterministic",
        "--asset",
        "candidate_missing_hashed",
        "--asset",
        "candidate_missing_replayable",
        "--asset",
        "candidate_missing_cross_platform_verified",
    ]));

    for (asset_id, missing_condition) in missing_cases {
        let block = asset_block(&stdout, asset_id);
        let condition_name = missing_condition.as_str();
        assert!(block.contains("found_in_registry=true"));
        assert!(block.contains("freeze_state=pre-freeze"));
        assert!(block.contains("taxonomy_label=AI-assisted-pre-freeze"));
        assert!(block.contains("combat_truth_authority_allowed=false"));
        assert!(block.contains(&format!("{condition_name}=fail")));
        assert!(block.contains(&format!(
            "WARNING: combat-truth authority blocked for {asset_id}"
        )));
        assert!(block.contains(&format!("failed_conditions={condition_name}")));
    }
}

#[test]
fn slash_goal_alias_uses_same_handler_and_blocks_missing_registry_assets() {
    let root = test_root("slash_goal_missing_registry");

    let stdout = successful_stdout(run_goal(&[
        "/goal",
        "--repo-root",
        root.to_str().expect("utf8 root"),
        "--asset",
        "missing_ai_asset",
    ]));

    let missing_asset = asset_block(&stdout, "missing_ai_asset");
    assert!(missing_asset.contains("found_in_registry=false"));
    assert!(missing_asset.contains("freeze_state=pre-freeze"));
    assert!(missing_asset.contains("combat_truth_authority_allowed=false"));
    assert!(missing_asset.contains("frozen=fail"));
    assert!(missing_asset.contains("deterministic=fail"));
    assert!(missing_asset.contains("hashed=fail"));
    assert!(missing_asset.contains("replayable=fail"));
    assert!(missing_asset.contains("cross_platform_verified=fail"));
    assert!(missing_asset.contains("WARNING: combat-truth authority blocked for missing_ai_asset"));
    assert!(missing_asset.contains(
        "failed_conditions=frozen,deterministic,hashed,replayable,cross_platform_verified"
    ));
}
