use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::{
    query_freeze_status, tag_artifact_boundary, ArtifactBoundaryMetadata, ArtifactOrigin,
    OathError, COMBAT_TRUTH_AUTHORITY_SCOPE,
};

/// User-facing `/goal` report input for one artifact referenced by the goal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GoalArtifactSpec {
    pub asset_id: String,
    pub authority_scope: String,
    pub origin: ArtifactOrigin,
}

impl GoalArtifactSpec {
    pub fn combat_truth_ai_assisted(asset_id: impl Into<String>) -> Self {
        Self {
            asset_id: asset_id.into(),
            authority_scope: COMBAT_TRUTH_AUTHORITY_SCOPE.to_string(),
            origin: ArtifactOrigin::AiAssisted,
        }
    }

    pub fn combat_truth_deterministic_never_ai(asset_id: impl Into<String>) -> Self {
        Self {
            asset_id: asset_id.into(),
            authority_scope: COMBAT_TRUTH_AUTHORITY_SCOPE.to_string(),
            origin: ArtifactOrigin::DeterministicNeverAi,
        }
    }
}

/// Render the `/goal` command report.
///
/// Canon source: `docs/design/GAME_CANON.md` section
/// "Frontier Research Leverage and Authoritative Combat Truth".
/// Implementation source: `src/freeze_status.rs` defines the five freeze
/// conditions, authority scopes, and the structured status API consumed here.
/// This command is display glue only; it does not generate, repair, promote,
/// or mutate freeze records.
pub fn render_goal_command_output(
    repo_root: impl AsRef<Path>,
    artifacts: &[GoalArtifactSpec],
) -> Result<String, OathError> {
    let repo_root = repo_root.as_ref();
    let policy = load_frontier_policy_summary(repo_root)?;

    let mut out = String::new();
    writeln!(&mut out, "OATHYARD /goal frontier policy summary").unwrap();
    writeln!(&mut out, "source=docs/design/GAME_CANON.md").unwrap();
    writeln!(&mut out, "summary={}", policy.summary).unwrap();
    writeln!(&mut out, "layers:").unwrap();
    for layer in &policy.layers {
        writeln!(&mut out, "- {layer}").unwrap();
    }
    writeln!(
        &mut out,
        "promotion_gates={}",
        policy.promotion_gates.join(", ")
    )
    .unwrap();
    writeln!(&mut out, "combat_truth_rule={}", policy.combat_truth_rule).unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "OATHYARD /goal artifact boundary report").unwrap();

    if artifacts.is_empty() {
        writeln!(
            &mut out,
            "no_goal_artifacts_supplied=true; pass --asset <asset_id> or --non-ai-asset <asset_id> to query freeze status"
        )
        .unwrap();
        return Ok(out);
    }

    for artifact in artifacts {
        let status = query_freeze_status(repo_root, &artifact.authority_scope, &artifact.asset_id)?;
        let label = tag_artifact_boundary(&ArtifactBoundaryMetadata {
            origin: artifact.origin,
            freeze: status.conditions,
        });
        let failed_conditions = status
            .condition_results
            .iter()
            .filter(|condition| !condition.passed)
            .map(|condition| condition.name.as_str())
            .collect::<Vec<_>>();

        writeln!(&mut out, "asset_id={}", status.asset_id).unwrap();
        writeln!(&mut out, "authority_scope={}", status.authority_scope).unwrap();
        writeln!(&mut out, "registry_path={}", status.registry_path.display()).unwrap();
        writeln!(&mut out, "found_in_registry={}", status.found_in_registry).unwrap();
        writeln!(&mut out, "freeze_state={}", status.freeze_state.as_str()).unwrap();
        writeln!(&mut out, "taxonomy_label={}", label.as_str()).unwrap();
        writeln!(
            &mut out,
            "combat_truth_authority_allowed={}",
            status.combat_truth_authority_allowed
        )
        .unwrap();
        writeln!(
            &mut out,
            "conditions={}",
            condition_status_line(&status.condition_results)
        )
        .unwrap();

        if !status.combat_truth_authority_allowed {
            writeln!(
                &mut out,
                "WARNING: combat-truth authority blocked for {}; failed_conditions={}",
                status.asset_id,
                failed_conditions.join(",")
            )
            .unwrap();
        }
        writeln!(&mut out).unwrap();
    }

    Ok(out)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FrontierPolicySummary {
    summary: String,
    layers: Vec<String>,
    promotion_gates: Vec<String>,
    combat_truth_rule: String,
}

fn load_frontier_policy_summary(repo_root: &Path) -> Result<FrontierPolicySummary, OathError> {
    let canon_path = repo_root.join("docs/design/GAME_CANON.md");
    let canon = fs::read_to_string(&canon_path).map_err(|error| {
        OathError::Io(format!(
            "failed to read canonical policy summary from {}: {error}",
            canon_path.display()
        ))
    })?;
    let section = frontier_policy_section(&canon)?;

    let summary = section
        .lines()
        .map(str::trim)
        .find(|line| {
            !line.is_empty()
                && !line.starts_with("## ")
                && !line.starts_with("**Authoritative combat truth**")
        })
        .ok_or_else(|| OathError::Parse("frontier policy summary paragraph missing".to_string()))?
        .to_string();

    let layers = section
        .lines()
        .filter_map(frontier_layer_line)
        .collect::<Vec<_>>();
    if layers.len() != 3 {
        return Err(OathError::Parse(format!(
            "expected 3 frontier policy layers in GAME_CANON.md, found {}",
            layers.len()
        )));
    }

    let promotion_gates = section
        .lines()
        .filter_map(promotion_gate_name)
        .collect::<Vec<_>>();
    if promotion_gates.len() != 5 {
        return Err(OathError::Parse(format!(
            "expected 5 frontier promotion gates in GAME_CANON.md, found {}",
            promotion_gates.len()
        )));
    }

    let combat_truth_rule = section
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("- Missing evidence means"))
        .map(|line| line.trim_start_matches("- ").to_string())
        .ok_or_else(|| {
            OathError::Parse("missing evidence combat-truth rule not found".to_string())
        })?;

    Ok(FrontierPolicySummary {
        summary,
        layers,
        promotion_gates,
        combat_truth_rule,
    })
}

fn frontier_policy_section(canon: &str) -> Result<&str, OathError> {
    let heading = "## Frontier Research Leverage and Authoritative Combat Truth";
    let start = canon
        .find(heading)
        .ok_or_else(|| OathError::Parse(format!("canon heading '{heading}' not found")))?;
    let after_heading = start + heading.len();
    let tail = &canon[after_heading..];
    let end = tail.find("\n## ").unwrap_or(tail.len());
    Ok(&canon[start..after_heading + end])
}

fn frontier_layer_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let after_number = trimmed
        .strip_prefix("1. `")
        .or_else(|| trimmed.strip_prefix("2. `"))
        .or_else(|| trimmed.strip_prefix("3. `"))?;
    let (name, rest) = after_number.split_once("`: ")?;
    if !matches!(
        name,
        "offline_research_authoring" | "runtime_presentation" | "runtime_authoritative_truth"
    ) {
        return None;
    }
    Some(format!("{name}: {rest}"))
}

fn promotion_gate_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let after_number = trimmed
        .strip_prefix("1. **")
        .or_else(|| trimmed.strip_prefix("2. **"))
        .or_else(|| trimmed.strip_prefix("3. **"))
        .or_else(|| trimmed.strip_prefix("4. **"))
        .or_else(|| trimmed.strip_prefix("5. **"))?;
    let (name, _) = after_number.split_once("**")?;
    Some(name.to_string())
}

fn condition_status_line(condition_results: &[crate::FreezeConditionResult; 5]) -> String {
    condition_results
        .iter()
        .map(|condition| {
            format!(
                "{}={}",
                condition.name.as_str(),
                if condition.passed { "pass" } else { "fail" }
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}
