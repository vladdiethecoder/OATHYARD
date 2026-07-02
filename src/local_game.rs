// Unit-057: Local playable game state machine.
//
// Layered ABOVE the deterministic truth/replay path. This module:
//   1. Builds a LocalGameConfig (player fighter/loadout/arena + scripted opponent policy).
//   2. Drives the existing AI planner over prefix observations (as ai-duel does),
//      but labels one seat "player" and the other "scripted_opponent".
//   3. Runs the existing execute_ai_plan -> run_scenario_text -> replay path UNCHANGED.
//   4. Walks DuelResult.turns to synthesise player-facing game states
//      (Boot..MainMenu..ModeSelect..FighterSelect..LoadoutSelect..ArenaSelect..MatchIntro..
//       (Observe..Plan..CommitReveal..Resolve..Consequence..Replan)*..MatchResult..
//       ReplayBrowser..FightFilmView..Settings..Quit).
//   5. Emits every artefact the working-game smoke gate requires.
//   6. Verifies final state hash against the replay file it itself wrote.
//
// The player's "input" in this initial implementation is a deterministic scripted
// policy. When a window/backend becomes available, player input will drive the same
// plan-entry shape via public CLI subcommands. Truth path, hash chain, replay
// schema, contact-resolution machinery are untouched.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::{
    build_ai_duel_plan_for, execute_ai_plan, verify_replay_text, write_artifacts, AiPlan,
    AiPlanEntry, AiPolicyStyle, ContactTrace, CostBreakdown, DuelResult, FighterSpec, OathError,
    PRODUCT_NAME, TRUTH_HZ,
};
use crate::{hash_hex, write_json_field};

pub const LOCAL_GAME_SCHEMA: &str = "oathyard.local_game_flow.v1";
pub const FIGHT_FILM_VIEW_SCHEMA: &str = "oathyard.fight_film_view.v1";
pub const SCRIPTED_INPUT_MANIFEST_SCHEMA: &str = "oathyard.scripted_input_manifest.v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameState {
    Boot,
    MainMenu,
    ModeSelect,
    FighterSelect,
    LoadoutSelect,
    ArenaSelect,
    MatchIntro,
    Observe,
    Plan,
    CommitReveal,
    Resolve,
    Consequence,
    Replan,
    MatchResult,
    ReplayBrowser,
    FightFilmView,
    Settings,
    Quit,
}

impl GameState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Boot => "boot",
            Self::MainMenu => "main_menu",
            Self::ModeSelect => "mode_select",
            Self::FighterSelect => "fighter_select",
            Self::LoadoutSelect => "loadout_select",
            Self::ArenaSelect => "arena_select",
            Self::MatchIntro => "match_intro",
            Self::Observe => "observe",
            Self::Plan => "plan",
            Self::CommitReveal => "commit_reveal",
            Self::Resolve => "resolve",
            Self::Consequence => "consequence",
            Self::Replan => "replan",
            Self::MatchResult => "match_result",
            Self::ReplayBrowser => "replay_browser",
            Self::FightFilmView => "fight_film_view",
            Self::Settings => "settings",
            Self::Quit => "quit",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameStateEntry {
    pub state: GameState,
    pub turn: i32,
    pub description: String,
    pub player_action_label: String,
    pub player_direction: String,
    pub player_target: String,
    pub player_queued_plan: String,
    pub player_base_cost_frames: u32,
    pub player_current_cost_frames: u32,
    pub player_action_valid: bool,
    pub body_modifier_permille: Option<i32>,
    pub equipment_modifier_permille: Option<i32>,
    pub state_modifier_permille: Option<i32>,
    pub momentum_modifier_permille: Option<i32>,
    pub injury_modifier_events: Vec<String>,
    pub opponent_action_label: String,
    pub opponent_direction: String,
    pub opponent_target: String,
    pub commit_reveal_status: String,
    pub contact_event: String,
    pub armor_material_result: String,
    pub injury_capability_result: String,
    pub cause_chain: String,
    pub next_action_validity: String,
    pub capture_id: String,
    pub truth_hash: String,
    pub truth_mutation: bool,
}

impl GameStateEntry {
    fn new(state: GameState, turn: i32, description: &str) -> Self {
        Self {
            state,
            turn,
            description: description.to_string(),
            player_action_label: String::new(),
            player_direction: String::new(),
            player_target: String::new(),
            player_queued_plan: String::new(),
            player_base_cost_frames: 0,
            player_current_cost_frames: 0,
            player_action_valid: true,
            body_modifier_permille: None,
            equipment_modifier_permille: None,
            state_modifier_permille: None,
            momentum_modifier_permille: None,
            injury_modifier_events: Vec::new(),
            opponent_action_label: String::new(),
            opponent_direction: String::new(),
            opponent_target: String::new(),
            commit_reveal_status: String::new(),
            contact_event: String::new(),
            armor_material_result: String::new(),
            injury_capability_result: String::new(),
            cause_chain: String::new(),
            next_action_validity: String::new(),
            capture_id: state.as_str().to_string(),
            truth_hash: String::new(),
            truth_mutation: false,
        }
    }

    fn bind_player_plan(&mut self, entry: &AiPlanEntry) {
        self.player_action_label = entry.action.as_str().to_string();
        self.player_direction = entry.direction.as_str().to_string();
        self.player_target = entry.target.as_str().to_string();
        self.player_queued_plan = format!(
            "{} {} {}",
            entry.action.as_str(),
            entry.direction.as_str(),
            entry.target.as_str()
        );
    }

    fn bind_player_cost(&mut self, cost: &CostBreakdown) {
        self.player_base_cost_frames = cost.base_frames;
        self.player_current_cost_frames = cost.current_frames;
        self.player_action_valid = cost.action_valid;
        self.bind_cost_modifiers(cost);
    }

    fn bind_cost_modifiers(&mut self, cost: &CostBreakdown) {
        // Map each factor's reason text to one of the UI modifier buckets:
        // body (balance/injury), equipment (weapon/armor/material),
        // state (recovery/torso), momentum (forward/momentum).
        let mut body = None;
        let mut equipment = None;
        let mut state: Option<i32> = None;
        let mut momentum = None;
        for factor in &cost.factors {
            let r = &factor.reason;
            let p = factor.permille;
            if r.contains("balance") || r.contains("injury") {
                body = Some(p);
            } else if r.contains("weapon") || r.contains("armor") || r.contains("material") {
                equipment = Some(p);
            } else if r.contains("recovery") || r.contains("torso") {
                state = Some(p);
            } else if r.contains("momentum") || r.contains("forward") {
                momentum = Some(p);
            }
        }
        // Fall back: use the frame delta for "state" when no explicit recovery factor.
        if state.is_none() && cost.current_frames != cost.base_frames {
            state = Some(
                ((cost.current_frames as i64 - cost.base_frames as i64) * 1000
                    / cost.base_frames.max(1) as i64) as i32,
            );
        }
        self.body_modifier_permille = body;
        self.equipment_modifier_permille = equipment;
        self.state_modifier_permille = state;
        self.momentum_modifier_permille = momentum;
    }

    fn bind_opponent_plan(&mut self, entry: &AiPlanEntry) {
        self.opponent_action_label = entry.action.as_str().to_string();
        self.opponent_direction = entry.direction.as_str().to_string();
        self.opponent_target = entry.target.as_str().to_string();
    }

    fn bind_contact(&mut self, contact: &ContactTrace) {
        self.contact_event = format!(
            "{} -> {} at {} with {} via {}",
            contact.weapon_id,
            contact.armor_id,
            contact.target.as_str(),
            contact.direction.as_str(),
            contact.action.as_str()
        );
        self.armor_material_result = contact.material_result.clone();
        self.injury_capability_result = format!(
            "{}; capability({})",
            contact.anatomy_result, contact.capability_delta.event
        );
        self.cause_chain = contact.cause_chain.clone();
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalGameConfig {
    pub scenario_id: String,
    pub player_fighter: FighterSpec,
    pub opponent_fighter: FighterSpec,
    #[allow(private_interfaces)]
    pub player_policy: AiPolicyStyle,
    #[allow(private_interfaces)]
    pub opponent_policy: AiPolicyStyle,
    pub arena_id: String,
    pub min_plan_cycles: u32,
    pub loadout_id: String,
}

impl Default for LocalGameConfig {
    fn default() -> Self {
        Self {
            scenario_id: "unit057_local_duel".to_string(),
            player_fighter: FighterSpec {
                seat: 0,
                name: "fighter_mannequin".to_string(),
                weapon_id: "longsword".to_string(),
                armor_id: "gambeson".to_string(),
            },
            opponent_fighter: FighterSpec {
                seat: 1,
                name: "writ_sentinel".to_string(),
                weapon_id: "ash_spear".to_string(),
                armor_id: "mail_hauberk".to_string(),
            },
            player_policy: AiPolicyStyle::Balanced,
            opponent_policy: AiPolicyStyle::ReachPressure,
            arena_id: "training_yard".to_string(),
            min_plan_cycles: 3,
            loadout_id: "longsword_gambeson".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalGameRun {
    pub config: LocalGameConfig,
    pub states: Vec<GameStateEntry>,
    #[allow(private_interfaces)]
    pub plan: AiPlan,
    pub result: DuelResult,
    pub replay_json_sha256: String,
    pub trace_json_sha256: String,
    pub replay_verified: bool,
    pub replay_hash_matches: bool,
    pub plan_cycles: u32,
    pub end_condition_winner: String,
    pub local_playable_game_ready: bool,
}

const REQUIRED_STATES: &[GameState] = &[
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
];

fn states_visited(states: &[GameStateEntry], required: &[GameState]) -> bool {
    required
        .iter()
        .all(|required| states.iter().any(|e| e.state == *required))
}

/// Entry point for `oathyard play-local` and `tools/working_game_smoke.sh`.
pub fn write_local_game_artifacts(
    out_dir: impl AsRef<Path>,
    config: LocalGameConfig,
) -> Result<LocalGameRun, OathError> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;

    let (states, plan, result) = run_local_game(&config)?;

    let replay_json_sha256 = hash_hex(result.replay_json.as_bytes());
    let trace_json_sha256 = hash_hex(result.trace_json.as_bytes());
    let min_cycles = config.min_plan_cycles;

    // Use the existing verify_replay_text path. If replay verification fails
    // or the final_state_hash mismatches, the gate fails loudly.
    let (replay_verified, replay_hash_matches) = match verify_replay_text(&result.replay_json) {
        Ok(verified) => (true, verified.final_state_hash == result.final_state_hash),
        Err(_) => (false, false),
    };

    let plan_cycles = result.turns.len() as u32;
    let end_condition_winner = result.end_condition.winner_token();
    let local_playable_game_ready = replay_verified
        && replay_hash_matches
        && plan_cycles >= min_cycles.max(2)
        && states_visited(&states, REQUIRED_STATES);

    let run = LocalGameRun {
        config,
        states,
        plan,
        result,
        replay_json_sha256,
        trace_json_sha256,
        replay_verified,
        replay_hash_matches,
        plan_cycles,
        end_condition_winner: end_condition_winner.clone(),
        local_playable_game_ready,
    };

    fs::write(
        out_dir.join("game_flow_manifest.json"),
        render_game_flow_manifest_json(&run),
    )?;
    fs::write(
        out_dir.join("scripted_input_manifest.json"),
        render_scripted_input_manifest_json(&run),
    )?;
    fs::write(
        out_dir.join("planning_ui_data_report.md"),
        render_planning_ui_data_report(&run),
    )?;
    fs::write(
        out_dir.join("consequence_cause_chain_report.md"),
        render_consequence_cause_chain_report(&run),
    )?;
    fs::write(
        out_dir.join("replay_verification_report.md"),
        render_replay_verification_report(&run),
    )?;
    fs::write(
        out_dir.join("fight_film_view_manifest.json"),
        render_fight_film_view_manifest_json(&run),
    )?;
    fs::write(
        out_dir.join("final_state_hash.txt"),
        &run.result.final_state_hash,
    )?;

    // Delegate the canonical duel artefacts (replay.json/trace.json/fight_film_manifest.json/
    // duel_report.md) to the existing write_artifacts path. The game-flow/UI-data report files
    // above are additive and do not duplicate them.
    write_artifacts(&run.result, out_dir)?;

    if !run.replay_verified || !run.replay_hash_matches {
        return Err(OathError::Verify(format!(
            "local working-game replay verification failed: verified={} hash_match={}",
            run.replay_verified, run.replay_hash_matches
        )));
    }
    if run.plan_cycles < min_cycles.max(2) {
        return Err(OathError::Verify(format!(
            "local working-game plan cycles {} < required {}",
            run.plan_cycles,
            min_cycles.max(2)
        )));
    }
    Ok(run)
}

fn run_local_game(
    config: &LocalGameConfig,
) -> Result<(Vec<GameStateEntry>, AiPlan, DuelResult), OathError> {
    let min_cycles = config.min_plan_cycles.max(2);
    // build_ai_duel_plan_for with two policies drives both seats deterministically
    // from prefix-observations only; neither side looks ahead or decides outcomes.
    let mut plan = build_ai_duel_plan_for(
        config.scenario_id.clone(),
        [
            config.player_fighter.clone(),
            config.opponent_fighter.clone(),
        ],
        [config.player_policy, config.opponent_policy],
        min_cycles,
    )?;
    let mut result = execute_ai_plan(plan.clone())?.result;

    while (result.turns.len() as u32) < min_cycles {
        plan = build_ai_duel_plan_for(
            config.scenario_id.clone(),
            [
                config.player_fighter.clone(),
                config.opponent_fighter.clone(),
            ],
            [config.player_policy, config.opponent_policy],
            min_cycles + 1,
        )?;
        result = execute_ai_plan(plan.clone())?.result;
    }

    let mut states: Vec<GameStateEntry> = Vec::new();
    push_pre_match_states(&mut states, config);

    let turn_count = result.turns.len();
    for (idx, turn_trace) in result.turns.iter().enumerate() {
        let player_entry = plan_entry_for_turn(&plan, 0, turn_trace.turn);
        let opponent_entry = plan_entry_for_turn(&plan, 1, turn_trace.turn);
        let player_cost = player_cost_for_turn(&result, turn_trace.turn);

        // OBSERVE
        let mut observe = GameStateEntry::new(
            GameState::Observe,
            turn_trace.turn as i32,
            &format!(
                "Turn {}: fighters observe positioning and capability state",
                turn_trace.turn
            ),
        );
        observe.truth_hash = turn_trace.state_hash.clone();
        states.push(observe);

        // PLAN
        let mut plan_s = GameStateEntry::new(
            GameState::Plan,
            turn_trace.turn as i32,
            &format!(
                "Turn {}: player and scripted opponent commit planned actions from observations",
                turn_trace.turn
            ),
        );
        if let Some(p) = &player_entry {
            plan_s.bind_player_plan(p);
        }
        if let Some(o) = &opponent_entry {
            plan_s.bind_opponent_plan(o);
        }
        if let Some(cost) = player_cost {
            plan_s.bind_player_cost(cost);
        }
        plan_s.truth_hash = turn_trace.state_hash.clone();
        states.push(plan_s);

        // COMMIT_REVEAL
        let mut commit = GameStateEntry::new(
            GameState::CommitReveal,
            turn_trace.turn as i32,
            &format!(
                "Turn {}: both actions revealed simultaneously from replay-serialisable plans",
                turn_trace.turn
            ),
        );
        if let Some(p) = &player_entry {
            commit.bind_player_plan(p);
        }
        if let Some(o) = &opponent_entry {
            commit.bind_opponent_plan(o);
        }
        commit.commit_reveal_status = format!(
            "player={} {} {} ; opponent={} {} {} ; truth_hz={}",
            commit.player_action_label,
            commit.player_direction,
            commit.player_target,
            commit.opponent_action_label,
            commit.opponent_direction,
            commit.opponent_target,
            TRUTH_HZ
        );
        commit.truth_hash = turn_trace.state_hash.clone();
        states.push(commit);

        // RESOLVE
        let mut resolve = GameStateEntry::new(
            GameState::Resolve,
            turn_trace.turn as i32,
            &format!(
                "Turn {}: deterministic contact resolution from committed plans",
                turn_trace.turn
            ),
        );
        if let Some(contact) = turn_trace.contacts.first() {
            resolve.bind_contact(contact);
        } else {
            resolve.contact_event = "no_contact".to_string();
            resolve.armor_material_result = "no_material_solve".to_string();
        }
        resolve.truth_hash = turn_trace.state_hash.clone();
        states.push(resolve);

        // CONSEQUENCE
        let mut consequence = GameStateEntry::new(
            GameState::Consequence,
            turn_trace.turn as i32,
            &format!(
                "Turn {}: capability/injury/armor/material consequence applied to fighters",
                turn_trace.turn
            ),
        );
        if let Some(contact) = turn_trace.contacts.first() {
            consequence.bind_contact(contact);
        }
        if let Some(cost) = player_cost {
            consequence.next_action_validity =
                render_next_action_validity(cost, &result.turns, idx);
        }
        consequence.truth_hash = turn_trace.state_hash.clone();
        states.push(consequence);

        // REPLAN (before the next turn — player observes consequence).
        if idx + 1 < turn_count {
            let mut replan = GameStateEntry::new(
                GameState::Replan,
                turn_trace.turn as i32,
                &format!(
                    "Turn {}: player replans after observing consequence; input is replay-serialisable",
                    turn_trace.turn
                ),
            );
            if let Some(cost) = player_cost {
                replan.next_action_validity = render_next_action_validity(cost, &result.turns, idx);
            }
            replan.truth_hash = turn_trace.state_hash.clone();
            states.push(replan);
        }
    }

    // Post-match states
    let mut match_result = GameStateEntry::new(
        GameState::MatchResult,
        -1,
        &format!(
            "Match ended: winner {} (end condition: {})",
            result.end_condition.winner_token(),
            result.end_condition.status
        ),
    );
    match_result.truth_hash = result.final_state_hash.clone();
    states.push(match_result);

    let mut replay_browser = GameStateEntry::new(
        GameState::ReplayBrowser,
        -1,
        "Replay browser: verified replay drives deterministic playback",
    );
    replay_browser.truth_hash = result.final_state_hash.clone();
    states.push(replay_browser);

    let mut fight_film = GameStateEntry::new(
        GameState::FightFilmView,
        -1,
        "Fight-film view: trace-derived camera/manifest playback",
    );
    fight_film.truth_hash = result.final_state_hash.clone();
    states.push(fight_film);

    states.push(GameStateEntry::new(
        GameState::Settings,
        -1,
        "Settings screen",
    ));

    states.push(GameStateEntry::new(
        GameState::Quit,
        -1,
        "Quit: clean exit from working-game flow",
    ));

    Ok((states, plan, result))
}

fn plan_entry_for_turn(plan: &AiPlan, seat: usize, turn: u32) -> Option<AiPlanEntry> {
    plan.entries
        .iter()
        .find(|e| e.seat == seat && e.turn == turn)
        .cloned()
}

fn player_cost_for_turn(result: &DuelResult, turn: u32) -> Option<&CostBreakdown> {
    result
        .turns
        .iter()
        .find(|t| t.turn == turn)?
        .costs
        .iter()
        .find(|c| c.fighter == 0)
}

fn render_next_action_validity(
    cost: &CostBreakdown,
    turns: &[crate::TurnTrace],
    current_index: usize,
) -> String {
    if !cost.action_valid {
        return "current_action_invalid_by_truth".to_string();
    }
    if current_index + 1 >= turns.len() {
        return "no_next_turn".to_string();
    }
    "next_turn_valid".to_string()
}

fn push_pre_match_states(states: &mut Vec<GameStateEntry>, config: &LocalGameConfig) {
    states.push(GameStateEntry::new(
        GameState::Boot,
        -1,
        &format!("OATHYARD boot ({})", PRODUCT_NAME),
    ));
    let mut main_menu = GameStateEntry::new(
        GameState::MainMenu,
        -1,
        "Main menu: local duel, settings, quit",
    );
    main_menu.capture_id = "boot_main_menu".to_string();
    states.push(main_menu);

    states.push(GameStateEntry::new(
        GameState::ModeSelect,
        -1,
        "Mode select: choose local_duel",
    ));

    let mut fighter_select = GameStateEntry::new(
        GameState::FighterSelect,
        -1,
        &format!(
            "Fighter select: player chose {}",
            config.player_fighter.name
        ),
    );
    fighter_select.player_action_label = config.player_fighter.weapon_id.clone();
    fighter_select.capture_id = "fighter_select".to_string();
    states.push(fighter_select);

    let mut loadout_select = GameStateEntry::new(
        GameState::LoadoutSelect,
        -1,
        &format!(
            "Loadout select: weapon {} armor {} (loadout_id {})",
            config.player_fighter.weapon_id, config.player_fighter.armor_id, config.loadout_id
        ),
    );
    loadout_select.player_action_label = config.loadout_id.clone();
    loadout_select.capture_id = "loadout_select".to_string();
    states.push(loadout_select);

    let mut arena_select = GameStateEntry::new(
        GameState::ArenaSelect,
        -1,
        &format!("Arena select: {}", config.arena_id),
    );
    arena_select.player_action_label = config.arena_id.clone();
    arena_select.capture_id = "arena_select".to_string();
    states.push(arena_select);

    states.push(GameStateEntry::new(
        GameState::MatchIntro,
        -1,
        &format!(
            "Match intro: {} vs {} in {}",
            config.player_fighter.name, config.opponent_fighter.name, config.arena_id
        ),
    ));
}

// ---------------------------------------------------------------------------
// Renderers
// ---------------------------------------------------------------------------

fn render_game_flow_manifest_json(run: &LocalGameRun) -> String {
    let mut out = String::new();
    writeln!(out, "{{").unwrap();
    writeln!(out, "  \"schema\": \"{}\",", LOCAL_GAME_SCHEMA).unwrap();
    writeln!(out, "  \"product\": \"{}\",", PRODUCT_NAME).unwrap();
    writeln!(out, "  \"truth_hz\": {},", TRUTH_HZ).unwrap();
    writeln!(out, "  \"hidden_rng\": false,").unwrap();
    writeln!(out, "  \"wall_clock\": false,").unwrap();
    writeln!(out, "  \"truth_mutation\": false,").unwrap();
    writeln!(out, "  \"difficulty_changes_body_stats\": false,").unwrap();
    writeln!(out, "  \"outcome_authority\": \"truth_replay_only\",").unwrap();
    write_json_field(&mut out, 1, "scenario_id", &run.config.scenario_id, true);
    write_json_field(
        &mut out,
        1,
        "player_fighter",
        &run.config.player_fighter.name,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "player_weapon",
        &run.config.player_fighter.weapon_id,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "player_armor",
        &run.config.player_fighter.armor_id,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "player_policy",
        run.config.player_policy.as_str(),
        true,
    );
    write_json_field(
        &mut out,
        1,
        "opponent_fighter",
        &run.config.opponent_fighter.name,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "opponent_weapon",
        &run.config.opponent_fighter.weapon_id,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "opponent_armor",
        &run.config.opponent_fighter.armor_id,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "opponent_policy",
        run.config.opponent_policy.as_str(),
        true,
    );
    write_json_field(&mut out, 1, "loadout_id", &run.config.loadout_id, true);
    write_json_field(&mut out, 1, "arena_id", &run.config.arena_id, true);
    write_json_field(&mut out, 1, "content_hash", &run.result.content_hash, true);
    write_json_field(
        &mut out,
        1,
        "final_state_hash",
        &run.result.final_state_hash,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "end_condition_status",
        &run.result.end_condition.status,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "end_condition_winner",
        &run.end_condition_winner,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "replay_json_sha256",
        &run.replay_json_sha256,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "trace_json_sha256",
        &run.trace_json_sha256,
        true,
    );
    writeln!(out, "  \"replay_verified\": {},", run.replay_verified).unwrap();
    writeln!(
        out,
        "  \"replay_verified_final_hash_matches\": {},",
        run.replay_hash_matches
    )
    .unwrap();
    writeln!(out, "  \"plan_cycles\": {},", run.plan_cycles).unwrap();
    writeln!(out, "  \"fight_film_view_manifest_present\": true,").unwrap();
    writeln!(
        out,
        "  \"local_playable_game_ready\": {},",
        run.local_playable_game_ready
    )
    .unwrap();
    writeln!(
        out,
        "  \"required_states_visited\": {},",
        states_visited(&run.states, REQUIRED_STATES)
    )
    .unwrap();
    writeln!(out, "  \"owner_visual_acceptance\": false,").unwrap();
    writeln!(out, "  \"public_demo_ready\": false,").unwrap();
    writeln!(out, "  \"release_candidate_ready\": false,").unwrap();
    writeln!(out, "  \"states\": [").unwrap();
    for (idx, state) in run.states.iter().enumerate() {
        render_game_state_entry_json(&mut out, state, idx + 1 == run.states.len());
    }
    writeln!(out, "  ]").unwrap();
    writeln!(out, "}}").unwrap();
    out
}

fn render_game_state_entry_json(out: &mut String, state: &GameStateEntry, last: bool) {
    writeln!(out, "    {{").unwrap();
    writeln!(out, "      \"state\": \"{}\",", state.state.as_str()).unwrap();
    writeln!(out, "      \"turn\": {},", state.turn).unwrap();
    write_json_field(out, 5, "description", &state.description, true);
    write_json_field(
        out,
        5,
        "player_action_label",
        &state.player_action_label,
        true,
    );
    write_json_field(out, 5, "player_direction", &state.player_direction, true);
    write_json_field(out, 5, "player_target", &state.player_target, true);
    write_json_field(
        out,
        5,
        "player_queued_plan",
        &state.player_queued_plan,
        true,
    );
    writeln!(
        out,
        "      \"player_base_cost_frames\": {},",
        state.player_base_cost_frames
    )
    .unwrap();
    writeln!(
        out,
        "      \"player_current_cost_frames\": {},",
        state.player_current_cost_frames
    )
    .unwrap();
    writeln!(
        out,
        "      \"player_action_valid\": {},",
        state.player_action_valid
    )
    .unwrap();
    write_opt_i32(out, "body_modifier_permille", state.body_modifier_permille);
    write_opt_i32(
        out,
        "equipment_modifier_permille",
        state.equipment_modifier_permille,
    );
    write_opt_i32(
        out,
        "state_modifier_permille",
        state.state_modifier_permille,
    );
    write_opt_i32(
        out,
        "momentum_modifier_permille",
        state.momentum_modifier_permille,
    );
    write_str_array(out, "injury_modifier_events", &state.injury_modifier_events);
    write_json_field(
        out,
        5,
        "opponent_action_label",
        &state.opponent_action_label,
        true,
    );
    write_json_field(
        out,
        5,
        "opponent_direction",
        &state.opponent_direction,
        true,
    );
    write_json_field(out, 5, "opponent_target", &state.opponent_target, true);
    write_json_field(
        out,
        5,
        "commit_reveal_status",
        &state.commit_reveal_status,
        true,
    );
    write_json_field(out, 5, "contact_event", &state.contact_event, true);
    write_json_field(
        out,
        5,
        "armor_material_result",
        &state.armor_material_result,
        true,
    );
    write_json_field(
        out,
        5,
        "injury_capability_result",
        &state.injury_capability_result,
        true,
    );
    write_json_field(out, 5, "cause_chain", &state.cause_chain, true);
    write_json_field(
        out,
        5,
        "next_action_validity",
        &state.next_action_validity,
        true,
    );
    write_json_field(out, 5, "capture_id", &state.capture_id, true);
    write_json_field(out, 5, "truth_hash", &state.truth_hash, true);
    writeln!(out, "      \"truth_mutation\": false").unwrap();
    let terminal = if last { "" } else { "," };
    writeln!(out, "    }}{terminal}").unwrap();
}

fn write_opt_i32(out: &mut String, key: &str, value: Option<i32>) {
    match value {
        Some(v) => writeln!(out, "      \"{key}\": {v},").unwrap(),
        None => writeln!(out, "      \"{key}\": null,").unwrap(),
    }
}

fn write_str_array(out: &mut String, key: &str, values: &[String]) {
    let joined: Vec<String> = values.iter().map(|v| format!("\"{v}\"")).collect();
    writeln!(out, "      \"{key}\": [{}],", joined.join(", ")).unwrap();
}

fn render_scripted_input_manifest_json(run: &LocalGameRun) -> String {
    let mut out = String::new();
    writeln!(out, "{{").unwrap();
    writeln!(out, "  \"schema\": \"{}\",", SCRIPTED_INPUT_MANIFEST_SCHEMA).unwrap();
    writeln!(out, "  \"product\": \"{}\",", PRODUCT_NAME).unwrap();
    writeln!(out, "  \"truth_hz\": {},", TRUTH_HZ).unwrap();
    writeln!(out, "  \"truth_mutation\": false,").unwrap();
    writeln!(out, "  \"interactive_ready\": false,").unwrap();
    writeln!(out, "  \"scripted_driver\": \"deterministic_policy_v1\",").unwrap();
    writeln!(
        out,
        "  \"interactive_blocked_reason\": \"no window/input backend yet\","
    )
    .unwrap();
    write_json_field(
        &mut out,
        1,
        "player_fighter",
        &run.config.player_fighter.name,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "player_weapon",
        &run.config.player_fighter.weapon_id,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "player_armor",
        &run.config.player_fighter.armor_id,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "player_policy",
        run.config.player_policy.as_str(),
        true,
    );
    write_json_field(
        &mut out,
        1,
        "opponent_fighter",
        &run.config.opponent_fighter.name,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "opponent_weapon",
        &run.config.opponent_fighter.weapon_id,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "opponent_armor",
        &run.config.opponent_fighter.armor_id,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "opponent_policy",
        run.config.opponent_policy.as_str(),
        true,
    );
    write_json_field(&mut out, 1, "arena_id", &run.config.arena_id, true);
    writeln!(out, "  \"planned_entries\": [").unwrap();
    for (idx, entry) in run.plan.entries.iter().enumerate() {
        let seat_label = if entry.seat == 0 {
            "player"
        } else {
            "opponent"
        };
        writeln!(out, "    {{").unwrap();
        writeln!(out, "      \"turn\": {},", entry.turn).unwrap();
        write_json_field(&mut out, 5, "seat", seat_label, true);
        write_json_field(&mut out, 5, "policy", entry.policy.as_str(), true);
        write_json_field(&mut out, 5, "action", entry.action.as_str(), true);
        write_json_field(&mut out, 5, "direction", entry.direction.as_str(), true);
        write_json_field(&mut out, 5, "target", entry.target.as_str(), true);
        write_json_field(&mut out, 5, "planner_reason", &entry.planner_reason, false);
        let comma = if idx + 1 == run.plan.entries.len() {
            ""
        } else {
            ","
        };
        writeln!(out, "    }}{comma}").unwrap();
    }
    writeln!(out, "  ],").unwrap();
    writeln!(
        out,
        "  \"opponent_emits_legal_planned_actions_only\": true,"
    )
    .unwrap();
    writeln!(out, "  \"opponent_does_not_decide_contact\": true,").unwrap();
    writeln!(out, "  \"opponent_does_not_inspect_future\": true").unwrap();
    writeln!(out, "}}").unwrap();
    out
}

fn render_planning_ui_data_report(run: &LocalGameRun) -> String {
    let mut out = String::new();
    writeln!(out, "# Unit-057 Planning UI Data Report").unwrap();
    writeln!(out).unwrap();
    writeln!(
        out,
        "Binding source: `game_flow_manifest.json` entries of state `plan`, `observe`, `commit_reveal`, `resolve`, `consequence`."
    )
    .unwrap();
    writeln!(
        out,
        "Truth path: `replay.json` + `trace.json` (schema {} / {}); final_state_hash `{}`.",
        crate::REPLAY_SCHEMA,
        crate::TRACE_SCHEMA,
        run.result.final_state_hash
    )
    .unwrap();
    writeln!(out).unwrap();
    writeln!(out, "## Required UI fields").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "- phase").unwrap();
    writeln!(out, "- selected fighter / loadout").unwrap();
    writeln!(out, "- selected action label / direction / target").unwrap();
    writeln!(out, "- queued plan").unwrap();
    writeln!(out, "- base cost frames / current cost frames").unwrap();
    writeln!(
        out,
        "- body/equipment/state/momentum/injury modifiers (permille)"
    )
    .unwrap();
    writeln!(out, "- commit/reveal status").unwrap();
    writeln!(
        out,
        "- contact event / armor-material result / injury-capability result"
    )
    .unwrap();
    writeln!(out, "- cause-chain").unwrap();
    writeln!(out, "- next action validity").unwrap();
    writeln!(out, "- replay verification").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "## Per-turn UI snapshot").unwrap();
    writeln!(out).unwrap();
    for state in &run.states {
        if !matches!(
            state.state,
            GameState::Observe
                | GameState::Plan
                | GameState::CommitReveal
                | GameState::Resolve
                | GameState::Consequence
                | GameState::Replan
        ) {
            continue;
        }
        writeln!(out, "### Turn {} — {}", state.turn, state.state.as_str()).unwrap();
        writeln!(out).unwrap();
        writeln!(out, "- `phase`: `{}`", state.state.as_str()).unwrap();
        writeln!(
            out,
            "- `fighter`: {} (weapon {}, armor {})",
            run.config.player_fighter.name,
            run.config.player_fighter.weapon_id,
            run.config.player_fighter.armor_id
        )
        .unwrap();
        writeln!(out, "- `loadout`: `{}`", run.config.loadout_id).unwrap();
        writeln!(
            out,
            "- `selected_action`: `{}` direction `{}` target `{}`",
            state.player_action_label, state.player_direction, state.player_target
        )
        .unwrap();
        writeln!(out, "- `queued_plan`: `{}`", state.player_queued_plan).unwrap();
        writeln!(
            out,
            "- `base_cost_frames`: `{}`  `current_cost_frames`: `{}`",
            state.player_base_cost_frames, state.player_current_cost_frames
        )
        .unwrap();
        writeln!(out, "- `action_valid`: `{}`", state.player_action_valid).unwrap();
        writeln!(
            out,
            "- modifiers (permille): body={:?} equipment={:?} state={:?} momentum={:?}",
            state.body_modifier_permille,
            state.equipment_modifier_permille,
            state.state_modifier_permille,
            state.momentum_modifier_permille
        )
        .unwrap();
        writeln!(
            out,
            "- injury modifiers: {:?}",
            state.injury_modifier_events
        )
        .unwrap();
        writeln!(
            out,
            "- `commit_reveal_status`: `{}`",
            state.commit_reveal_status
        )
        .unwrap();
        writeln!(
            out,
            "- opponent_action: `{}` direction `{}` target `{}`",
            state.opponent_action_label, state.opponent_direction, state.opponent_target
        )
        .unwrap();
        writeln!(out, "- `contact_event`: `{}`", state.contact_event).unwrap();
        writeln!(
            out,
            "- `armor_material_result`: `{}`",
            state.armor_material_result
        )
        .unwrap();
        writeln!(
            out,
            "- `injury_capability_result`: `{}`",
            state.injury_capability_result
        )
        .unwrap();
        writeln!(out, "- `cause_chain`: `{}`", state.cause_chain).unwrap();
        writeln!(
            out,
            "- `next_action_validity`: `{}`",
            state.next_action_validity
        )
        .unwrap();
        writeln!(
            out,
            "- `truth_hash`: `{}` (observation; truth mutation: false)",
            state.truth_hash
        )
        .unwrap();
        writeln!(out).unwrap();
    }
    out
}

fn render_consequence_cause_chain_report(run: &LocalGameRun) -> String {
    let mut out = String::new();
    writeln!(out, "# Unit-057 Consequence Cause-Chain Report").unwrap();
    writeln!(out).unwrap();
    writeln!(
        out,
        "Sourced from `trace.json` (via `replay.json` after verification)."
    )
    .unwrap();
    writeln!(
        out,
        "Final state hash: `{}` — end condition: `{}` (winner: `{}`).",
        run.result.final_state_hash, run.result.end_condition.status, run.end_condition_winner
    )
    .unwrap();
    writeln!(out).unwrap();
    writeln!(out, "## Cause-chain by turn").unwrap();
    writeln!(out).unwrap();
    for turn_trace in &run.result.turns {
        writeln!(out, "### Turn {}", turn_trace.turn).unwrap();
        writeln!(out).unwrap();
        if turn_trace.contacts.is_empty() {
            writeln!(
                out,
                "- No contact. Commits resolved without material solve."
            )
            .unwrap();
        } else {
            for (idx, contact) in turn_trace.contacts.iter().enumerate() {
                writeln!(
                    out,
                    "- contact[{}]: {} vs {} at {} with {} via {}",
                    idx + 1,
                    contact.weapon_id,
                    contact.armor_id,
                    contact.target.as_str(),
                    contact.direction.as_str(),
                    contact.action.as_str()
                )
                .unwrap();
                writeln!(out, "  - material_result: `{}`", contact.material_result).unwrap();
                writeln!(out, "  - anatomy_result: `{}`", contact.anatomy_result).unwrap();
                writeln!(
                    out,
                    "  - capability event: `{}`",
                    contact.capability_delta.event
                )
                .unwrap();
                writeln!(out, "  - cause_chain: `{}`", contact.cause_chain).unwrap();
            }
        }
        if let Some(cost) = turn_trace.costs.iter().find(|c| c.fighter == 0) {
            writeln!(out).unwrap();
            writeln!(
                out,
                "- player cost: base={} current={} action_valid={}",
                cost.base_frames, cost.current_frames, cost.action_valid
            )
            .unwrap();
            writeln!(out, "- cost factors:").unwrap();
            for factor in &cost.factors {
                writeln!(
                    out,
                    "  - {} : permille={} reason=`{}`",
                    factor.name, factor.permille, factor.reason
                )
                .unwrap();
            }
        }
        writeln!(out).unwrap();
    }
    out
}

fn render_replay_verification_report(run: &LocalGameRun) -> String {
    let mut out = String::new();
    writeln!(out, "# Unit-057 Replay Verification Report").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "- replay schema: `{}`", crate::REPLAY_SCHEMA).unwrap();
    writeln!(out, "- trace schema: `{}`", crate::TRACE_SCHEMA).unwrap();
    writeln!(out, "- scenario_id: `{}`", run.result.scenario_id).unwrap();
    writeln!(out, "- content_hash: `{}`", run.result.content_hash).unwrap();
    writeln!(
        out,
        "- final_state_hash (observed): `{}`",
        run.result.final_state_hash
    )
    .unwrap();
    writeln!(out, "- replay verified: `{}`", run.replay_verified).unwrap();
    writeln!(
        out,
        "- replay verified final_state_hash matches observed: `{}`",
        run.replay_hash_matches
    )
    .unwrap();
    writeln!(out, "- replay_json_sha256: `{}`", run.replay_json_sha256).unwrap();
    writeln!(out, "- trace_json_sha256: `{}`", run.trace_json_sha256).unwrap();
    writeln!(
        out,
        "- end_condition: `{}` winner: `{}`",
        run.result.end_condition.status, run.end_condition_winner
    )
    .unwrap();
    writeln!(out).unwrap();
    writeln!(out, "## Per-turn replay hash chain").unwrap();
    writeln!(out).unwrap();
    for (idx, hash) in run.result.turn_hashes.iter().enumerate() {
        writeln!(out, "- turn[{}] state_hash: `{}`", idx, hash).unwrap();
    }
    writeln!(out).unwrap();
    writeln!(out, "## Truth isolation").unwrap();
    writeln!(out).unwrap();
    writeln!(
        out,
        "- `truth_mutation` field in `game_flow_manifest.json`: `false`"
    )
    .unwrap();
    writeln!(
        out,
        "- `truth_mutation` field in `fight_film_view_manifest.json`: `false`"
    )
    .unwrap();
    writeln!(
        out,
        "- `truth_mutation` field in `scripted_input_manifest.json`: `false`"
    )
    .unwrap();
    out
}

fn render_fight_film_view_manifest_json(run: &LocalGameRun) -> String {
    let mut out = String::new();
    writeln!(out, "{{").unwrap();
    writeln!(out, "  \"schema\": \"{}\",", FIGHT_FILM_VIEW_SCHEMA).unwrap();
    writeln!(out, "  \"product\": \"{}\",", PRODUCT_NAME).unwrap();
    writeln!(out, "  \"truth_hz\": {},", TRUTH_HZ).unwrap();
    writeln!(out, "  \"truth_mutation\": false,").unwrap();
    write_json_field(&mut out, 1, "source", "replay + fight_film_manifest", true);
    write_json_field(&mut out, 1, "scenario_id", &run.result.scenario_id, true);
    write_json_field(&mut out, 1, "content_hash", &run.result.content_hash, true);
    write_json_field(
        &mut out,
        1,
        "final_state_hash",
        &run.result.final_state_hash,
        true,
    );
    write_json_field(&mut out, 1, "fighter_fight_film_manifest", "present", true);
    writeln!(out, "  \"shots\": [").unwrap();
    for (idx, turn_trace) in run.result.turns.iter().enumerate() {
        writeln!(out, "    {{").unwrap();
        writeln!(out, "      \"turn\": {},", turn_trace.turn).unwrap();
        writeln!(
            out,
            "      \"frame_observation\": {},",
            turn_trace.turn * TRUTH_HZ
        )
        .unwrap();
        let contact_summary = if turn_trace.contacts.is_empty() {
            "no_contact".to_string()
        } else {
            turn_trace
                .contacts
                .iter()
                .map(|c| {
                    format!(
                        "{}_{}_{}",
                        c.action.as_str(),
                        c.direction.as_str(),
                        c.target.as_str()
                    )
                })
                .collect::<Vec<_>>()
                .join("+")
        };
        write_json_field(&mut out, 5, "contact_summary", &contact_summary, true);
        write_json_field(
            &mut out,
            5,
            "camera_mode",
            "oathyard_verdict_ring_establishing",
            true,
        );
        write_json_field(&mut out, 5, "truth_hash", &turn_trace.state_hash, true);
        writeln!(out, "      \"truth_mutation\": false").unwrap();
        let term = if idx + 1 == run.result.turns.len() {
            ""
        } else {
            ","
        };
        writeln!(out, "    }}{term}").unwrap();
    }
    writeln!(out, "  ]").unwrap();
    writeln!(out, "}}").unwrap();
    out
}
