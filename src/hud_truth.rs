use crate::{content_hash, hash_hex, verify_replay_text, DuelResult, OathError};

pub const HUD_TRUTH_VIEW_SCHEMA: &str = "oathyard.hud_truth_view.v1";

pub const HUD_NATIVE_FLOW_IDS: [&str; 13] = [
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
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HudTruthViewModel {
    pub schema: &'static str,
    pub source: &'static str,
    pub cache_key: String,
    pub content_hash_verified: bool,
    pub presentation_only: bool,
    pub truth_mutation: bool,
    pub scenario_id: String,
    pub content_hash: String,
    pub initial_state_hash: String,
    pub final_state_hash: String,
    pub replay_json_hash: String,
    pub trace_json_hash: String,
    pub frame_cost_hash: String,
    pub flows: Vec<HudFlowView>,
    pub frame_costs: Vec<HudFrameCostView>,
}

impl HudTruthViewModel {
    pub fn reject_truth_mutation_attempt(
        &mut self,
        attempt: HudTruthMutationAttempt,
    ) -> Result<(), OathError> {
        let flow_known = HUD_NATIVE_FLOW_IDS
            .iter()
            .any(|flow_id| *flow_id == attempt.flow_id.as_str());
        let flow_detail = if flow_known {
            "known HUD flow"
        } else {
            "unknown HUD flow"
        };
        Err(OathError::Verify(format!(
            "HUD truth view is read-only: {flow_detail} '{}' attempted '{}' after content hash verification; truth mutation rejected as a no-op",
            attempt.flow_id, attempt.requested_change
        )))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HudFlowView {
    pub flow_id: &'static str,
    pub cache_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HudFrameCostView {
    pub turn: u32,
    pub fighter: usize,
    pub action: &'static str,
    pub base_cost_frames: u32,
    pub current_cost_frames: u32,
    pub action_valid: bool,
    pub physical_reasons: Vec<HudPhysicalReason>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HudPhysicalReason {
    pub category: &'static str,
    pub permille: i32,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HudTruthMutationAttempt {
    pub flow_id: String,
    pub requested_change: String,
}

pub fn build_hud_truth_view_model_from_replay_text(
    replay_text: &str,
) -> Result<HudTruthViewModel, OathError> {
    let verified = verify_replay_text(replay_text)?;
    build_hud_truth_view_model_from_verified_result(&verified)
}

pub fn build_hud_truth_view_model_from_result(
    result: &DuelResult,
) -> Result<HudTruthViewModel, OathError> {
    let verified = verify_replay_text(&result.replay_json)?;
    require_equal_hash("content_hash", &result.content_hash, &verified.content_hash)?;
    require_equal_hash(
        "initial_state_hash",
        &result.initial_state_hash,
        &verified.initial_state_hash,
    )?;
    require_equal_hash(
        "final_state_hash",
        &result.final_state_hash,
        &verified.final_state_hash,
    )?;
    if result.turn_hashes != verified.turn_hashes {
        return Err(OathError::Verify(
            "HUD truth view rejected mismatched turn hash chain".to_string(),
        ));
    }
    build_hud_truth_view_model_from_verified_result(&verified)
}

fn require_equal_hash(label: &str, expected: &str, actual: &str) -> Result<(), OathError> {
    if expected != actual {
        return Err(OathError::Verify(format!(
            "HUD truth view rejected mismatched {label}: expected {expected}, got {actual}"
        )));
    }
    Ok(())
}

fn build_hud_truth_view_model_from_verified_result(
    verified: &DuelResult,
) -> Result<HudTruthViewModel, OathError> {
    require_equal_hash(
        "verified content_hash",
        &content_hash(),
        &verified.content_hash,
    )?;

    let frame_costs = hud_frame_cost_views(verified)?;
    let replay_json_hash = hash_hex(verified.replay_json.as_bytes());
    let trace_json_hash = hash_hex(verified.trace_json.as_bytes());
    let frame_cost_hash = hash_hex(hud_frame_cost_cache_material(&frame_costs).as_bytes());
    let turn_hash_chain = verified.turn_hashes.join("|");
    let cache_material = format!(
        "{HUD_TRUTH_VIEW_SCHEMA}|{}|{}|{}|{}|{}|{}",
        verified.scenario_id,
        verified.content_hash,
        verified.initial_state_hash,
        verified.final_state_hash,
        turn_hash_chain,
        frame_cost_hash
    );
    let cache_key = hash_hex(cache_material.as_bytes());
    let flows = HUD_NATIVE_FLOW_IDS
        .iter()
        .map(|flow_id| HudFlowView {
            flow_id: *flow_id,
            cache_key: hash_hex(format!("{cache_key}|{flow_id}").as_bytes()),
        })
        .collect();

    Ok(HudTruthViewModel {
        schema: HUD_TRUTH_VIEW_SCHEMA,
        source: "verified-replay-after-content-hash",
        cache_key,
        content_hash_verified: true,
        presentation_only: true,
        truth_mutation: false,
        scenario_id: verified.scenario_id.clone(),
        content_hash: verified.content_hash.clone(),
        initial_state_hash: verified.initial_state_hash.clone(),
        final_state_hash: verified.final_state_hash.clone(),
        replay_json_hash,
        trace_json_hash,
        frame_cost_hash,
        flows,
        frame_costs,
    })
}

fn hud_frame_cost_views(verified: &DuelResult) -> Result<Vec<HudFrameCostView>, OathError> {
    let mut frame_costs = Vec::new();
    for turn in &verified.turns {
        for cost in &turn.costs {
            let physical_reasons = cost
                .factors
                .iter()
                .map(|factor| HudPhysicalReason {
                    category: factor.name,
                    permille: factor.permille,
                    reason: factor.reason.clone(),
                })
                .collect::<Vec<_>>();
            if physical_reasons.is_empty() {
                return Err(OathError::Verify(format!(
                    "HUD truth view cost for turn {} fighter {} has no physical reasons",
                    turn.turn, cost.fighter
                )));
            }
            frame_costs.push(HudFrameCostView {
                turn: turn.turn,
                fighter: cost.fighter,
                action: cost.action.as_str(),
                base_cost_frames: cost.base_frames,
                current_cost_frames: cost.current_frames,
                action_valid: cost.action_valid,
                physical_reasons,
            });
        }
    }
    if frame_costs.is_empty() {
        return Err(OathError::Verify(
            "HUD truth view has no frame-cost rows to expose".to_string(),
        ));
    }
    Ok(frame_costs)
}

fn hud_frame_cost_cache_material(frame_costs: &[HudFrameCostView]) -> String {
    let mut out = String::new();
    for cost in frame_costs {
        out.push_str(&format!(
            "{}:{}:{}:{}:{}:{}",
            cost.turn,
            cost.fighter,
            cost.action,
            cost.base_cost_frames,
            cost.current_cost_frames,
            cost.action_valid
        ));
        for reason in &cost.physical_reasons {
            out.push_str(&format!(
                ":{}:{}:{}",
                reason.category, reason.permille, reason.reason
            ));
        }
        out.push('\n');
    }
    out
}
