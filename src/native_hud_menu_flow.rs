use crate::{
    armor_by_id, build_hud_truth_view_model_from_result, hash_hex, verify_replay_text,
    weapon_by_id, DuelResult, OathError, Scenario, ARMORS, FIGHTER_TRADITIONS, HUD_NATIVE_FLOW_IDS,
    HUD_TRUTH_VIEW_SCHEMA, RUNTIME_SETTINGS_SCHEMA, TRUTH_HZ, WEAPONS,
};

pub const NATIVE_HUD_MENU_FLOW_SCHEMA: &str = "oathyard.native_hud_menu_flow.v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeHudMenuFlowModel {
    pub schema: &'static str,
    pub source: &'static str,
    pub hud_truth_schema: &'static str,
    pub content_hash_verified: bool,
    pub presentation_only: bool,
    pub truth_mutation: bool,
    pub scenario_id: String,
    pub content_hash: String,
    pub final_state_hash: String,
    pub replay_json_hash: String,
    pub trace_json_hash: String,
    pub frame_cost_hash: String,
    pub hud_cache_key: String,
    pub screens: Vec<NativeHudMenuFlowScreen>,
}

impl NativeHudMenuFlowModel {
    pub fn screen(&self, flow_id: &str) -> Option<&NativeHudMenuFlowScreen> {
        self.screens.iter().find(|screen| screen.flow_id == flow_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeHudMenuFlowScreen {
    pub flow_id: &'static str,
    pub flow_index: usize,
    pub input_action: &'static str,
    pub native_code_path: &'static str,
    pub headline: String,
    pub detail: String,
    pub base_cost_frames: u32,
    pub current_cost_frames: u32,
    pub physical_reasons: Vec<String>,
    pub truth_cache_key: String,
    pub read_only_truth: bool,
    pub presentation_only: bool,
    pub truth_mutation: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeHudFlowCommand {
    Next,
    Previous,
    MainMenuStart,
    OpenSettingsAccessibility,
    ApplySettingsEdit,
    ResetSettings,
    OpenFighterSelect,
    SelectNextFighter,
    OpenLoadoutSelect,
    SelectNextLoadout,
    OpenObserve,
    OpenPlan,
    CommitReveal,
    Resolve,
    Consequence,
    OpenReplayBrowser,
    OpenReplay,
    OpenFightFilm,
    ScrubFightFilmForward,
    TogglePerformanceDebugOverlay,
    BackToMainMenu,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeHudMenuFlowRun {
    pub schema: &'static str,
    pub source: &'static str,
    pub command_count: usize,
    pub visited_flow_ids: Vec<String>,
    pub truth_hash_before: String,
    pub truth_hash_after: String,
    pub presentation_only: bool,
    pub truth_mutation: bool,
    pub settings_profile_state: String,
    pub selected_fighter_card: String,
    pub selected_loadout_card: String,
    pub replay_opened: bool,
    pub fight_film_timeline_position: u32,
    pub debug_overlay_visible: bool,
    pub run_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeHudReplayStatus {
    pub schema: &'static str,
    pub verified: bool,
    pub loud_failure: bool,
    pub final_state_hash: Option<String>,
    pub content_hash: Option<String>,
    pub error_message: Option<String>,
}

pub fn build_native_hud_menu_flow_model(
    result: &DuelResult,
) -> Result<NativeHudMenuFlowModel, OathError> {
    let hud = build_hud_truth_view_model_from_result(result)?;
    let scenario = Scenario::parse(&result.canonical_scenario)?;
    let first_turn = result.turns.first();
    let first_contact = result
        .turns
        .iter()
        .flat_map(|turn| turn.contacts.iter())
        .next();
    let first_cost = hud.frame_costs.first().ok_or_else(|| {
        OathError::Verify("native HUD/menu flow has no HUD cost rows".to_string())
    })?;
    let frame_cost_summary = format!(
        "base {} current {}",
        first_cost.base_cost_frames, first_cost.current_cost_frames
    );
    let first_reason = first_cost
        .physical_reasons
        .iter()
        .find(|reason| reason.permille != 1000)
        .or_else(|| first_cost.physical_reasons.first())
        .map(format_physical_reason)
        .ok_or_else(|| {
            OathError::Verify("native HUD/menu flow has no physical cost reason".to_string())
        })?;
    let selected_weapon = scenario.fighters[0].weapon_id.as_str();
    let selected_armor = scenario.fighters[0].armor_id.as_str();
    let weapon = weapon_by_id(selected_weapon)
        .map(|weapon| weapon.display_name)
        .unwrap_or(selected_weapon);
    let armor = armor_by_id(selected_armor)
        .map(|armor| armor.display_name)
        .unwrap_or(selected_armor);
    let commit_summary = first_turn
        .map(|turn| {
            format!(
                "{} {} vs {} {}",
                turn.commits[0].label.as_str(),
                turn.commits[0].direction.as_str(),
                turn.commits[1].label.as_str(),
                turn.commits[1].direction.as_str()
            )
        })
        .unwrap_or_else(|| "commit pending".to_string());
    let resolve_summary = first_contact
        .map(|contact| format!("{} -> {}", contact.action.as_str(), contact.material_result))
        .unwrap_or_else(|| "no contact packet this turn".to_string());
    let consequence_summary = first_contact
        .map(|contact| {
            format!(
                "{} | balance {} recovery +{}f",
                contact.capability_delta.event,
                contact.capability_delta.balance_delta,
                contact.capability_delta.recovery_slowdown_add
            )
        })
        .unwrap_or_else(|| "capability state unchanged; re-plan remains legal".to_string());

    let mut screens = Vec::with_capacity(HUD_NATIVE_FLOW_IDS.len());
    for (index, flow_id) in HUD_NATIVE_FLOW_IDS.iter().enumerate() {
        let cost = hud
            .frame_costs
            .get(index % hud.frame_costs.len())
            .unwrap_or(first_cost);
        let physical_reasons = cost
            .physical_reasons
            .iter()
            .map(format_physical_reason)
            .collect::<Vec<_>>();
        if physical_reasons.is_empty() {
            return Err(OathError::Verify(format!(
                "native HUD/menu flow {flow_id} has no physical cost reasons"
            )));
        }
        let flow_cache_key = hud
            .flows
            .iter()
            .find(|flow| flow.flow_id == *flow_id)
            .map(|flow| flow.cache_key.clone())
            .ok_or_else(|| {
                OathError::Verify(format!(
                    "native HUD/menu flow missing HUD truth cache key for {flow_id}"
                ))
            })?;
        let (headline, detail) = screen_text(
            flow_id,
            result,
            &scenario,
            &commit_summary,
            &resolve_summary,
            &consequence_summary,
            &frame_cost_summary,
            &first_reason,
            weapon,
            armor,
        );
        screens.push(NativeHudMenuFlowScreen {
            flow_id,
            flow_index: index,
            input_action: input_action_for_flow(flow_id),
            native_code_path: native_code_path_for_flow(flow_id),
            headline,
            detail,
            base_cost_frames: cost.base_cost_frames,
            current_cost_frames: cost.current_cost_frames,
            physical_reasons,
            truth_cache_key: flow_cache_key,
            read_only_truth: true,
            presentation_only: true,
            truth_mutation: false,
        });
    }

    for flow_id in HUD_NATIVE_FLOW_IDS {
        if !screens.iter().any(|screen| screen.flow_id == flow_id) {
            return Err(OathError::Verify(format!(
                "native HUD/menu flow model missing required flow {flow_id}"
            )));
        }
    }

    Ok(NativeHudMenuFlowModel {
        schema: NATIVE_HUD_MENU_FLOW_SCHEMA,
        source: "native-command-state-model-consuming-hash-gated-hud-truth",
        hud_truth_schema: HUD_TRUTH_VIEW_SCHEMA,
        content_hash_verified: hud.content_hash_verified,
        presentation_only: true,
        truth_mutation: false,
        scenario_id: hud.scenario_id,
        content_hash: hud.content_hash,
        final_state_hash: hud.final_state_hash,
        replay_json_hash: hud.replay_json_hash,
        trace_json_hash: hud.trace_json_hash,
        frame_cost_hash: hud.frame_cost_hash,
        hud_cache_key: hud.cache_key,
        screens,
    })
}

pub fn drive_native_hud_menu_flow(
    model: &NativeHudMenuFlowModel,
    commands: &[NativeHudFlowCommand],
) -> Result<NativeHudMenuFlowRun, OathError> {
    let mut current_index = model
        .screens
        .iter()
        .position(|screen| screen.flow_id == "main_menu")
        .ok_or_else(|| OathError::Verify("native flow missing main_menu".to_string()))?;
    let mut visited = vec![model.screens[current_index].flow_id.to_string()];
    let mut settings_profile_state = "runtime_settings:default_accessible_local".to_string();
    let mut selected_fighter_index = 0usize;
    let mut selected_weapon_index = WEAPONS
        .iter()
        .position(|weapon| weapon.id == FIGHTER_TRADITIONS[0].default_weapon)
        .unwrap_or(0);
    let mut selected_armor_index = ARMORS
        .iter()
        .position(|armor| armor.id == FIGHTER_TRADITIONS[0].default_armor)
        .unwrap_or(0);
    let mut replay_opened = false;
    let mut fight_film_timeline_position = 0u32;
    let mut debug_overlay_visible = false;
    let mut run_material = String::new();

    for command in commands {
        match command {
            NativeHudFlowCommand::Next => {
                current_index = (current_index + 1) % model.screens.len();
            }
            NativeHudFlowCommand::Previous => {
                current_index = if current_index == 0 {
                    model.screens.len() - 1
                } else {
                    current_index - 1
                };
            }
            NativeHudFlowCommand::MainMenuStart => {
                current_index = screen_index(model, "mode_select")?;
            }
            NativeHudFlowCommand::OpenSettingsAccessibility => {
                current_index = screen_index(model, "settings_accessibility")?;
            }
            NativeHudFlowCommand::ApplySettingsEdit => {
                current_index = screen_index(model, "settings_accessibility")?;
                settings_profile_state = format!(
                    "edited:{RUNTIME_SETTINGS_SCHEMA}:text_scale=1400;captions=true;truth_mutation=false"
                );
            }
            NativeHudFlowCommand::ResetSettings => {
                current_index = screen_index(model, "settings_accessibility")?;
                settings_profile_state = format!(
                    "reset:{RUNTIME_SETTINGS_SCHEMA}:default_accessible_local;truth_mutation=false"
                );
            }
            NativeHudFlowCommand::OpenFighterSelect => {
                current_index = screen_index(model, "fighter_select")?;
            }
            NativeHudFlowCommand::SelectNextFighter => {
                current_index = screen_index(model, "fighter_select")?;
                selected_fighter_index = (selected_fighter_index + 1) % FIGHTER_TRADITIONS.len();
            }
            NativeHudFlowCommand::OpenLoadoutSelect => {
                current_index = screen_index(model, "loadout_select")?;
            }
            NativeHudFlowCommand::SelectNextLoadout => {
                current_index = screen_index(model, "loadout_select")?;
                selected_weapon_index = (selected_weapon_index + 1) % WEAPONS.len();
                selected_armor_index = (selected_armor_index + 1) % ARMORS.len();
            }
            NativeHudFlowCommand::OpenObserve => current_index = screen_index(model, "observe")?,
            NativeHudFlowCommand::OpenPlan => current_index = screen_index(model, "plan")?,
            NativeHudFlowCommand::CommitReveal => {
                current_index = screen_index(model, "commit_reveal")?;
            }
            NativeHudFlowCommand::Resolve => current_index = screen_index(model, "resolve")?,
            NativeHudFlowCommand::Consequence => {
                current_index = screen_index(model, "consequence")?;
            }
            NativeHudFlowCommand::OpenReplayBrowser => {
                current_index = screen_index(model, "replay_browser")?;
            }
            NativeHudFlowCommand::OpenReplay => {
                current_index = screen_index(model, "replay_browser")?;
                replay_opened = true;
            }
            NativeHudFlowCommand::OpenFightFilm => {
                current_index = screen_index(model, "fight_film")?;
            }
            NativeHudFlowCommand::ScrubFightFilmForward => {
                current_index = screen_index(model, "fight_film")?;
                fight_film_timeline_position =
                    fight_film_timeline_position.saturating_add(TRUTH_HZ / 2);
            }
            NativeHudFlowCommand::TogglePerformanceDebugOverlay => {
                current_index = screen_index(model, "performance_debug_overlay")?;
                debug_overlay_visible = !debug_overlay_visible;
            }
            NativeHudFlowCommand::BackToMainMenu => {
                current_index = screen_index(model, "main_menu")?;
            }
        }
        let screen = &model.screens[current_index];
        visited.push(screen.flow_id.to_string());
        run_material.push_str(&format!(
            "{:?}:{}:{}:{}:{};",
            command,
            screen.flow_id,
            screen.truth_cache_key,
            screen.base_cost_frames,
            screen.current_cost_frames
        ));
    }

    let selected_fighter = &FIGHTER_TRADITIONS[selected_fighter_index];
    let selected_weapon = &WEAPONS[selected_weapon_index];
    let selected_armor = &ARMORS[selected_armor_index];
    Ok(NativeHudMenuFlowRun {
        schema: NATIVE_HUD_MENU_FLOW_SCHEMA,
        source: "native-input-command-navigation-presentation-only",
        command_count: commands.len(),
        visited_flow_ids: visited,
        truth_hash_before: model.final_state_hash.clone(),
        truth_hash_after: model.final_state_hash.clone(),
        presentation_only: true,
        truth_mutation: false,
        settings_profile_state,
        selected_fighter_card: format!(
            "seat {selected_fighter_index}: {} ({})",
            selected_fighter.display_name, selected_fighter.affordance
        ),
        selected_loadout_card: format!(
            "weapon={} armor={} source=content_manifest",
            selected_weapon.id, selected_armor.id
        ),
        replay_opened,
        fight_film_timeline_position,
        debug_overlay_visible,
        run_hash: hash_hex(run_material.as_bytes()),
    })
}

pub fn native_hud_menu_flow_replay_status(replay_text: &str) -> NativeHudReplayStatus {
    match verify_replay_text(replay_text) {
        Ok(result) => NativeHudReplayStatus {
            schema: NATIVE_HUD_MENU_FLOW_SCHEMA,
            verified: true,
            loud_failure: false,
            final_state_hash: Some(result.final_state_hash),
            content_hash: Some(result.content_hash),
            error_message: None,
        },
        Err(error) => NativeHudReplayStatus {
            schema: NATIVE_HUD_MENU_FLOW_SCHEMA,
            verified: false,
            loud_failure: true,
            final_state_hash: None,
            content_hash: None,
            error_message: Some(error.to_string()),
        },
    }
}

fn screen_index(model: &NativeHudMenuFlowModel, flow_id: &str) -> Result<usize, OathError> {
    model
        .screens
        .iter()
        .position(|screen| screen.flow_id == flow_id)
        .ok_or_else(|| OathError::Verify(format!("native flow missing screen {flow_id}")))
}

fn screen_text(
    flow_id: &str,
    result: &DuelResult,
    scenario: &Scenario,
    commit_summary: &str,
    resolve_summary: &str,
    consequence_summary: &str,
    frame_cost_summary: &str,
    first_reason: &str,
    weapon: &str,
    armor: &str,
) -> (String, String) {
    match flow_id {
        "main_menu" => (
            "Main menu: start local oath duel".to_string(),
            format!("native input routes to settings/select/replay/debug; {frame_cost_summary}; {first_reason}"),
        ),
        "mode_select" => (
            format!("Mode select: local scenario {}", result.scenario_id),
            "offline native duel flow; public/release gates remain false".to_string(),
        ),
        "settings_accessibility" => (
            "Settings/accessibility: edit/apply/cancel/reset".to_string(),
            format!("presentation-only profile over {RUNTIME_SETTINGS_SCHEMA}; {frame_cost_summary}; {first_reason}"),
        ),
        "fighter_select" => (
            format!(
                "Fighter select: {} vs {}",
                scenario.fighters[0].name, scenario.fighters[1].name
            ),
            "roster cards source legal fighter families without stat bonuses".to_string(),
        ),
        "loadout_select" => (
            format!("Loadout select: {weapon} / {armor}"),
            "weapon/armor cards are physical content refs, not shortcut resources".to_string(),
        ),
        "observe" => (
            format!("Observe: content hash {} verified", result.content_hash),
            format!("read-only loadout, turn hash, and {frame_cost_summary}; {first_reason}"),
        ),
        "plan" => (
            format!("Plan compact physical labels: {commit_summary}"),
            format!("{frame_cost_summary}; physical reason {first_reason}"),
        ),
        "commit_reveal" => (
            format!("Commit reveal: {commit_summary}"),
            format!("locked inputs display {frame_cost_summary}; {first_reason}"),
        ),
        "resolve" => (
            format!("Resolve: {resolve_summary}"),
            "contact/material/anatomy packets are verified replay evidence".to_string(),
        ),
        "consequence" => (
            format!("Consequence: {consequence_summary}"),
            "capability deltas update future legality and costs without health bars".to_string(),
        ),
        "replay_browser" => (
            format!("Replay browser: verified final hash {}", result.final_state_hash),
            "open path reruns authoritative replay verification; corrupt files fail loudly".to_string(),
        ),
        "fight_film" => (
            format!("Fight-film: {} trace turns", result.turns.len()),
            "timeline/camera bookmarks consume trace data after replay hash".to_string(),
        ),
        "performance_debug_overlay" => (
            "Performance/debug overlay: hashes, costs, presentation timing".to_string(),
            format!("truth hz {TRUTH_HZ}; {frame_cost_summary}; {first_reason}"),
        ),
        _ => (
            format!("Unknown flow {flow_id}"),
            "unknown native HUD/menu flow".to_string(),
        ),
    }
}

fn format_physical_reason(reason: &crate::HudPhysicalReason) -> String {
    format!(
        "{} x{}: {}",
        reason.category, reason.permille, reason.reason
    )
}

fn input_action_for_flow(flow_id: &str) -> &'static str {
    match flow_id {
        "main_menu" => "main_menu_start",
        "mode_select" => "mode_select",
        "settings_accessibility" => "settings_accessibility",
        "fighter_select" => "fighter_select",
        "loadout_select" => "loadout_select",
        "observe" => "observe",
        "plan" => "plan",
        "commit_reveal" => "commit_reveal",
        "resolve" => "resolve",
        "consequence" => "consequence_readout",
        "replay_browser" => "replay_browser",
        "fight_film" => "fight_film",
        "performance_debug_overlay" => "performance_debug_overlay",
        _ => "unknown",
    }
}

fn native_code_path_for_flow(flow_id: &str) -> &'static str {
    match flow_id {
        "main_menu" => "native::hud_menu_flow::main_menu",
        "mode_select" => "native::hud_menu_flow::mode_select",
        "settings_accessibility" => "native::hud_menu_flow::settings_accessibility",
        "fighter_select" => "native::hud_menu_flow::fighter_select",
        "loadout_select" => "native::hud_menu_flow::loadout_select",
        "observe" => "native::hud_menu_flow::observe",
        "plan" => "native::hud_menu_flow::plan",
        "commit_reveal" => "native::hud_menu_flow::commit_reveal",
        "resolve" => "native::hud_menu_flow::resolve",
        "consequence" => "native::hud_menu_flow::consequence",
        "replay_browser" => "native::hud_menu_flow::replay_browser",
        "fight_film" => "native::hud_menu_flow::fight_film",
        "performance_debug_overlay" => "native::hud_menu_flow::performance_debug_overlay",
        _ => "native::hud_menu_flow::unknown",
    }
}
