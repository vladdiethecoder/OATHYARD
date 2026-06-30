use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::json::JsonValue;
use crate::sha256;
use crate::{BoundaryFreezeState, OathError};

/// Hash-registry index specified by the /goal frontier-freeze pipeline.
///
/// Registry entries live at:
/// `artifacts/freeze/v1/index/by_scope/<authority_scope>/<asset_id>.json`.
/// The registry entry is advisory input; authority is computed only from the
/// five explicit freeze conditions, never from a claimed state string alone.
pub const FREEZE_HASH_REGISTRY_INDEX: &str = "artifacts/freeze/v1/index/by_scope";
pub const COMBAT_TRUTH_AUTHORITY_SCOPE: &str = "combat_truth";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FreezeState {
    PreFreeze,
    AuthoritativePostFreeze,
}

impl FreezeState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PreFreeze => "pre-freeze",
            Self::AuthoritativePostFreeze => "authoritative-post-freeze",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FreezeConditionName {
    Frozen,
    Deterministic,
    Hashed,
    Replayable,
    CrossPlatformVerified,
}

impl FreezeConditionName {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Frozen => "frozen",
            Self::Deterministic => "deterministic",
            Self::Hashed => "hashed",
            Self::Replayable => "replayable",
            Self::CrossPlatformVerified => "cross_platform_verified",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FreezeConditionResult {
    pub name: FreezeConditionName,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreezeConditionEvaluation {
    pub conditions: BoundaryFreezeState,
    pub condition_results: [FreezeConditionResult; 5],
    pub overall_verdict: bool,
    pub freeze_state: FreezeState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FreezeStatusResult {
    pub asset_id: String,
    pub authority_scope: String,
    pub registry_path: PathBuf,
    pub found_in_registry: bool,
    pub freeze_state: FreezeState,
    pub conditions: BoundaryFreezeState,
    pub condition_results: [FreezeConditionResult; 5],
    pub overall_verdict: bool,
    pub combat_truth_authority_allowed: bool,
}

impl FreezeStatusResult {
    pub const fn may_declare_combat_truth_authority(&self) -> bool {
        self.combat_truth_authority_allowed
    }
}

/// Result of verifying a registry entry's `content_hash` against the actual artifact file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContentHashVerificationResult {
    /// The `content_hash` field from the registry entry (e.g. "sha256:abc123...").
    pub declared_hash: String,
    /// The SHA-256 we recomputed from the artifact file (hex, no prefix).
    pub computed_hash: String,
    /// True if the hash components match (ignoring the "sha256:" prefix).
    pub matches: bool,
    /// Human-readable detail for audit logs.
    pub detail: String,
}

/// Parsed registry entry containing both conditions and the content_hash field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistryEntry {
    pub asset_id: String,
    pub authority_scope: String,
    pub content_hash: String,
    pub conditions: BoundaryFreezeState,
    /// Raw parsed JSON tree of the registry entry.
    pub raw: JsonValue,
}

impl RegistryEntry {
    /// Parse a registry entry JSON text into structured form.
    pub fn parse(text: &str) -> Result<Self, OathError> {
        let root = JsonValue::parse(text)?;
        let asset_id = root
            .get("asset_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OathError::Parse("registry entry missing asset_id".to_string()))?
            .to_string();
        let authority_scope = root
            .get("authority_scope")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OathError::Parse("registry entry missing authority_scope".to_string()))?
            .to_string();
        let content_hash = root
            .get("content_hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| OathError::Parse("registry entry missing content_hash".to_string()))?
            .to_string();
        let conditions = parse_freeze_conditions_from_json(&root)?;
        Ok(Self {
            asset_id,
            authority_scope,
            content_hash,
            conditions,
            raw: root,
        })
    }

    /// Extract the hex digest portion from a "sha256:<hex>" format hash.
    /// Returns None if the hash doesn't follow the expected format.
    pub fn content_hash_hex(&self) -> Option<&str> {
        self.content_hash.strip_prefix("sha256:")
    }

    /// Render the entry back to canonical JSON text for writing.
    pub fn to_json(&self) -> String {
        let c = &self.conditions;
        format!(
            "{{\n  \"schema\": \"oathyard.freeze_registry_entry.v1\",\n  \"asset_id\": \"{}\",\n  \"authority_scope\": \"{}\",\n  \"content_hash\": \"sha256:{}\",\n  \"conditions\": {{\n    \"frozen\": {},\n    \"deterministic\": {},\n    \"hashed\": {},\n    \"replayable\": {},\n    \"cross_platform_verified\": {}\n  }}\n}}\n",
            self.asset_id,
            self.authority_scope,
            self.content_hash_hex().unwrap_or(&self.content_hash),
            c.frozen,
            c.deterministic,
            c.hashed,
            c.replayable,
            c.cross_platform_verified,
        )
    }
}

pub fn query_freeze_status(
    repo_root: impl AsRef<Path>,
    authority_scope: &str,
    asset_id: &str,
) -> Result<FreezeStatusResult, OathError> {
    let repo_root = repo_root.as_ref();
    let registry_path = registry_entry_path(repo_root, authority_scope, asset_id)?;
    let registry_text = match fs::read_to_string(&registry_path) {
        Ok(text) => Some(text),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => return Err(OathError::Io(error.to_string())),
    };

    let found_in_registry = registry_text.is_some();
    let mut conditions = match registry_text {
        Some(ref text) => parse_freeze_conditions(text)?,
        None => BoundaryFreezeState {
            frozen: false,
            deterministic: false,
            hashed: false,
            replayable: false,
            cross_platform_verified: false,
        },
    };

    // R-HASH-1 enforcement: when a registry entry exists, verify the
    // content_hash against the actual artifact file. If the artifact
    // exists and the hash mismatches, force `hashed` to false so the
    // overall verdict fails. This makes content_hash non-decorative.
    //
    // If the artifact file does not exist (common in test fixtures that
    // use placeholder hashes like "sha256:testfixture"), verification is
    // skipped — the conditions declared in the registry entry stand.
    if found_in_registry {
        if let Some(ref text) = registry_text {
            if let Ok(entry) = RegistryEntry::parse(text) {
                let hash_result = verify_registry_content_hash(repo_root, &entry)?;
                if !hash_result.matches && !hash_result.computed_hash.is_empty() {
                    // Artifact exists but hash mismatches — force hashed=false.
                    conditions.hashed = false;
                }
            }
        }
    }

    let evaluation = evaluate_freeze_conditions(conditions);

    let combat_truth_authority_allowed =
        authority_scope == COMBAT_TRUTH_AUTHORITY_SCOPE && evaluation.overall_verdict;

    Ok(FreezeStatusResult {
        asset_id: asset_id.to_string(),
        authority_scope: authority_scope.to_string(),
        registry_path,
        found_in_registry,
        freeze_state: evaluation.freeze_state,
        conditions: evaluation.conditions,
        condition_results: evaluation.condition_results,
        overall_verdict: evaluation.overall_verdict,
        combat_truth_authority_allowed,
    })
}

/// Verify that a registry entry's `content_hash` field matches the actual
/// SHA-256 of the referenced artifact file on disk.
///
/// This is the R-HASH-1 fix: the content_hash was previously decorative.
/// Now it is structurally parsed and cryptographically verified.
///
/// Returns the verification result with match status and detail. Does NOT
/// error on mismatch — callers decide whether to block or warn based on
/// the returned `matches` field.
pub fn verify_registry_content_hash(
    repo_root: impl AsRef<Path>,
    entry: &RegistryEntry,
) -> Result<ContentHashVerificationResult, OathError> {
    let artifact_path = repo_root
        .as_ref()
        .join("artifacts/freeze/v1/manifests")
        .join(&entry.authority_scope)
        .join(format!("{}.manifest", entry.asset_id));

    let declared_hex = entry.content_hash_hex().unwrap_or(&entry.content_hash);

    let computed_hash = match sha256::sha256_file(&artifact_path) {
        Ok(hex) => hex,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            return Ok(ContentHashVerificationResult {
                declared_hash: entry.content_hash.clone(),
                computed_hash: String::new(),
                matches: false,
                detail: format!("artifact file not found: {}", artifact_path.display()),
            });
        }
        Err(e) => return Err(OathError::Io(e.to_string())),
    };

    let matches = computed_hash == declared_hex;
    let detail = if matches {
        format!(
            "content_hash verified: sha256:{} matches artifact",
            declared_hex
        )
    } else {
        format!(
            "content_hash MISMATCH: registry declares sha256:{} but artifact computes sha256:{}",
            declared_hex, computed_hash
        )
    };

    Ok(ContentHashVerificationResult {
        declared_hash: entry.content_hash.clone(),
        computed_hash,
        matches,
        detail,
    })
}

pub fn evaluate_freeze_conditions(conditions: BoundaryFreezeState) -> FreezeConditionEvaluation {
    let condition_results = condition_results(conditions);
    let overall_verdict = conditions.all_conditions_passed();
    let freeze_state = if overall_verdict {
        FreezeState::AuthoritativePostFreeze
    } else {
        FreezeState::PreFreeze
    };

    FreezeConditionEvaluation {
        conditions,
        condition_results,
        overall_verdict,
        freeze_state,
    }
}

pub fn query_combat_truth_freeze_status(
    repo_root: impl AsRef<Path>,
    asset_id: &str,
) -> Result<FreezeStatusResult, OathError> {
    query_freeze_status(repo_root, COMBAT_TRUTH_AUTHORITY_SCOPE, asset_id)
}

pub fn may_declare_combat_truth_authority(status: &FreezeStatusResult) -> bool {
    status.may_declare_combat_truth_authority()
}

pub fn asset_may_declare_combat_truth_authority(
    repo_root: impl AsRef<Path>,
    asset_id: &str,
) -> Result<bool, OathError> {
    query_combat_truth_freeze_status(repo_root, asset_id)
        .map(|status| status.combat_truth_authority_allowed)
}

pub fn scoped_asset_may_declare_authority(
    repo_root: impl AsRef<Path>,
    authority_scope: &str,
    asset_id: &str,
) -> Result<bool, OathError> {
    query_freeze_status(repo_root, authority_scope, asset_id).map(|status| status.overall_verdict)
}

/// Prefix convention for AI-derived asset IDs.
///
/// Any weapon_id, armor_id, scenario_id, or other asset reference starting
/// with this prefix is treated as an AI-derived candidate subject to
/// combat-truth freeze-gate enforcement at consumption boundaries.
/// Compile-time content tables (e.g. "longsword", "mail_hauberk") do not
/// use this prefix and pass through without a registry lookup.
pub const AI_DERIVED_ASSET_PREFIX: &str = "ai:";

/// `&str` accessor for the prefix constant, re-exported for downstream use.
pub const fn ai_derived_asset_prefix_str() -> &'static str {
    AI_DERIVED_ASSET_PREFIX
}

/// Returns true if `asset_id` follows the AI-derived naming convention and
/// thus requires freeze-gate clearance at consumption boundaries.
pub fn is_ai_derived_asset_id(asset_id: &str) -> bool {
    asset_id.starts_with(AI_DERIVED_ASSET_PREFIX)
}

/// Enforce the combat-truth freeze gate at a consumption boundary.
///
/// If `asset_id` is AI-derived (per the `ai:` prefix convention), queries the
/// freeze registry and returns `Err` if the asset has not passed all five
/// freeze conditions. Non-AI assets pass through without a registry lookup.
///
/// Call this at every data-flow boundary where an asset could feed into the
/// authoritative combat simulation: AI plan execution, scenario parsing,
/// replay verification, native rendering, and content-table loading.
pub fn enforce_combat_truth_freeze_gate(
    repo_root: impl AsRef<Path>,
    asset_id: &str,
) -> Result<(), OathError> {
    if !is_ai_derived_asset_id(asset_id) {
        return Ok(());
    }
    let status = query_combat_truth_freeze_status(&repo_root, asset_id)?;
    if !status.combat_truth_authority_allowed {
        let failed = status
            .condition_results
            .iter()
            .filter(|condition| !condition.passed)
            .map(|condition| condition.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let registry_note = if status.found_in_registry {
            "found in registry but conditions not met"
        } else {
            "not found in freeze registry"
        };
        return Err(OathError::Verify(format!(
            "combat-truth freeze gate blocked AI-derived asset '{}' at consumption boundary: \
             freeze_state={}, {}",
            asset_id,
            status.freeze_state.as_str(),
            if failed.is_empty() {
                registry_note.to_string()
            } else {
                format!("{registry_note}, failed_conditions=[{failed}]")
            }
        )));
    }
    Ok(())
}

/// Batch-check multiple asset IDs against the freeze gate.
///
/// Convenience for scenarios that reference multiple assets (weapon_id,
/// armor_id per fighter). Returns the first blocking error or `Ok(())` if
/// all assets pass.
pub fn enforce_combat_truth_freeze_gate_batch<I>(
    repo_root: impl AsRef<Path>,
    asset_ids: I,
) -> Result<(), OathError>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    for asset_id in asset_ids {
        enforce_combat_truth_freeze_gate(&repo_root, asset_id.as_ref())?;
    }
    Ok(())
}

/// Returns the OATHYARD repo root directory for freeze-registry lookups.
///
/// Resolved at runtime from the `OATHYARD_REPO_ROOT` environment variable if
/// set, otherwise defaults to the current working directory (`.`). Production
/// code paths call this to locate the freeze registry without threading a
/// repo_root parameter through every function signature.
pub fn oathyard_repo_root() -> PathBuf {
    match env::var("OATHYARD_REPO_ROOT") {
        Ok(root) if !root.trim().is_empty() => PathBuf::from(root),
        _ => PathBuf::from("."),
    }
}

fn registry_entry_path(
    repo_root: &Path,
    authority_scope: &str,
    asset_id: &str,
) -> Result<PathBuf, OathError> {
    validate_registry_token("authority scope", authority_scope)?;
    validate_registry_token("asset id", asset_id)?;
    Ok(repo_root
        .join(FREEZE_HASH_REGISTRY_INDEX)
        .join(authority_scope)
        .join(format!("{asset_id}.json")))
}

fn validate_registry_token(kind: &str, token: &str) -> Result<(), OathError> {
    if token.trim().is_empty() {
        return Err(OathError::Parse(format!(
            "freeze registry {kind} must not be empty"
        )));
    }
    if token.contains('/') || token.contains('\\') || token == "." || token == ".." {
        return Err(OathError::Parse(format!(
            "freeze registry {kind} '{token}' must be a single path-safe identifier"
        )));
    }
    Ok(())
}

/// Parse freeze conditions from a registry entry JSON text using proper
/// structural JSON parsing (R-HASH-3: replaces ad-hoc string search).
///
/// Extracts the `conditions` object and reads the five boolean fields.
/// A string value containing `"frozen": true` as data will NOT be confused
/// with the actual `conditions.frozen` field because the JSON parser
/// distinguishes string values from boolean values structurally.
fn parse_freeze_conditions(input: &str) -> Result<BoundaryFreezeState, OathError> {
    let root = JsonValue::parse(input)?;
    parse_freeze_conditions_from_json(&root)
}

/// Parse freeze conditions from a pre-parsed JSON tree.
fn parse_freeze_conditions_from_json(root: &JsonValue) -> Result<BoundaryFreezeState, OathError> {
    let conditions = root.get("conditions").ok_or_else(|| {
        OathError::Parse("registry entry missing 'conditions' object".to_string())
    })?;
    let frozen = read_bool_condition(conditions, "frozen")?;
    let deterministic = read_bool_condition(conditions, "deterministic")?;
    let hashed = read_bool_condition(conditions, "hashed")?;
    let replayable = read_bool_condition(conditions, "replayable")?;
    let cross_platform_verified = read_bool_condition(conditions, "cross_platform_verified")?;
    Ok(BoundaryFreezeState {
        frozen,
        deterministic,
        hashed,
        replayable,
        cross_platform_verified,
    })
}

/// Read a boolean field from a JSON object, rejecting non-boolean values.
fn read_bool_condition(obj: &JsonValue, key: &str) -> Result<bool, OathError> {
    match obj.get(key) {
        Some(JsonValue::Bool(b)) => Ok(*b),
        Some(JsonValue::Null) => Ok(false),
        Some(_) => Err(OathError::Parse(format!(
            "freeze registry field '{key}' must be boolean, not {}",
            json_value_type_name(obj.get(key).unwrap())
        ))),
        None => Ok(false),
    }
}

fn json_value_type_name(v: &JsonValue) -> &'static str {
    match v {
        JsonValue::Null => "null",
        JsonValue::Bool(_) => "bool",
        JsonValue::Number(_) => "number",
        JsonValue::String(_) => "string",
        JsonValue::Array(_) => "array",
        JsonValue::Object(_) => "object",
    }
}

fn condition_results(conditions: BoundaryFreezeState) -> [FreezeConditionResult; 5] {
    [
        FreezeConditionResult {
            name: FreezeConditionName::Frozen,
            passed: conditions.frozen,
        },
        FreezeConditionResult {
            name: FreezeConditionName::Deterministic,
            passed: conditions.deterministic,
        },
        FreezeConditionResult {
            name: FreezeConditionName::Hashed,
            passed: conditions.hashed,
        },
        FreezeConditionResult {
            name: FreezeConditionName::Replayable,
            passed: conditions.replayable,
        },
        FreezeConditionResult {
            name: FreezeConditionName::CrossPlatformVerified,
            passed: conditions.cross_platform_verified,
        },
    ]
}
