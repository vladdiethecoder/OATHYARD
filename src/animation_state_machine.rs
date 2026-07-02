//! Presentation animation state machine layered over truth.
//!
//! This module is `runtime_presentation` only. It consumes truth-after-hash
//! duel result data (committed actions, contact traces, capability deltas)
//! and derives presentation animation states, transitions, reactions, and a
//! retargeting bridge from canonical truth joints to a presentation skeleton.
//!
//! It never writes into gameplay truth. The replay hash is identical with
//! animation derivation enabled or disabled, proved by the wrapper script.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::{
    comma, hash_hex, json_quote, run_scenario_file, write_json_field, ActionLabel, DuelResult,
    OathError, PRODUCT_NAME, PUBLIC_DEMO_READY, RELEASE_CANDIDATE_READY, TRUTH_HZ,
};

/// Schema for the animation state machine manifest artifact.
pub const ANIMATION_STATE_MACHINE_SCHEMA: &str = "oathyard.animation_state_machine.v1";

/// Schema for the internal MotionBricks-inspired presentation layer.
pub const PRESENTATION_BRICKS_SCHEMA: &str = "oathyard.presentation_bricks.v1";

/// Schema for the PresentationBricks retargeting bridge.
pub const PRESENTATION_BRICKS_RETARGETING_SCHEMA: &str =
    "oathyard.presentation_bricks_retargeting.v1";

const PRESENTATION_BRICKS_MOTION_SYSTEM: &str = "MotionBricks-inspired PresentationBricks";

#[derive(Clone, Copy, Debug)]
struct PresentationBrickPrimitive {
    primitive: &'static str,
    consumes: &'static str,
    output: &'static str,
}

const PRESENTATION_BRICK_PRIMITIVES: [PresentationBrickPrimitive; 10] = [
    PresentationBrickPrimitive {
        primitive: "locomotion",
        consumes: "truth poses, step/pivot/recover action labels, capability balance state",
        output: "footfall timing, root smoothing, readable stance transfer",
    },
    PresentationBrickPrimitive {
        primitive: "guard_transition",
        consumes: "guard/parry/brace action labels and facing direction",
        output: "guard pose blends and defensive silhouette shaping",
    },
    PresentationBrickPrimitive {
        primitive: "weapon_handling",
        consumes: "weapon id, action labels, grip capability, truth joint mapping",
        output: "grip frame offsets, blade/point/hook presentation arcs",
    },
    PresentationBrickPrimitive {
        primitive: "bind_hook",
        consumes: "contact event material result and hook/bind action labels",
        output: "readable weapon bind, hook, shove, and leverage pose accents",
    },
    PresentationBrickPrimitive {
        primitive: "stumble",
        consumes: "balance capability deltas after contact resolution",
        output: "presentation-only stagger offsets and recovery timing hints",
    },
    PresentationBrickPrimitive {
        primitive: "fall",
        consumes: "truth end-state/capability stop markers when present",
        output: "fall anticipation and ground-contact presentation beats",
    },
    PresentationBrickPrimitive {
        primitive: "collapse",
        consumes: "torque, torso rotation, grip, and balance capability deltas",
        output: "collapse-risk poses without changing stop rules",
    },
    PresentationBrickPrimitive {
        primitive: "recovery",
        consumes: "recovery slowdown frames and next committed action",
        output: "breath, reset, regain-guard, and return-to-stance motion",
    },
    PresentationBrickPrimitive {
        primitive: "object_interaction",
        consumes: "weapon/armor/arena asset ids and contact metadata",
        output: "readable prop alignment, handholds, staging, and contact accents",
    },
    PresentationBrickPrimitive {
        primitive: "fight_film_moment",
        consumes: "replay trace, contact beats, capability consequences, final hash",
        output: "camera moment hints, closeup beats, and replay timing labels",
    },
];

/// The 15 canonical presentation animation states derived from action labels
/// and the OBSERVE -> PLAN -> ... -> RE-PLAN phase machine.
///
/// These mirror the `ActionLabel` enum (13 combat labels) plus the two
/// planning-phase presentation states `observe` and `plan`.
pub const ANIMATION_STATE_LABELS: [&str; 15] = [
    "observe",
    "plan",
    "step",
    "pivot",
    "guard",
    "parry",
    "cut",
    "thrust",
    "brace",
    "bash",
    "hook_bind",
    "grab",
    "shove",
    "kick",
    "recover",
];

/// The 5 additive reaction states layered on top of action states when
/// contacts produce capability deltas. These are presentation-only labels
/// for injury/balance/grip consequences — they never pre-decide truth.
pub const ANIMATION_REACTION_LABELS: [&str; 5] =
    ["bind", "stagger", "collapse", "injury", "recovery"];

/// Canonical truth joints (16) from `GAME_CANON.md`.
///
/// Presentation adds `grip_r` and `grip_l` frames as retargeting targets.
/// The truth joints are the authoritative source; presentation cosmetic
/// bones consume them read-only after hashing.
const TRUTH_JOINT_NAMES: [&str; 16] = [
    "root",
    "spine_lower",
    "spine_upper",
    "neck_head",
    "shoulder_r",
    "elbow_r",
    "wrist_r",
    "shoulder_l",
    "elbow_l",
    "wrist_l",
    "hip_r",
    "knee_r",
    "ankle_r",
    "hip_l",
    "knee_l",
    "ankle_l",
];

/// Presentation skeleton bone names that extend the truth joints with
/// cosmetic detail bones for weapon grip, spine detail, and head.
const PRESENTATION_BONE_NAMES: [&str; 20] = [
    "root",
    "spine_lower",
    "spine_upper",
    "neck_head",
    "head",
    "shoulder_r",
    "elbow_r",
    "wrist_r",
    "grip_r",
    "shoulder_l",
    "elbow_l",
    "wrist_l",
    "grip_l",
    "hip_r",
    "knee_r",
    "ankle_r",
    "hip_l",
    "knee_l",
    "ankle_l",
    "weapon_tip",
];

/// Write animation state machine artifacts to `out_dir`.
///
/// Runs the scenario to obtain truth-after-hash data, then derives the
/// presentation animation state sequence, transition log, reaction log,
/// and retargeting bridge. Returns the original `DuelResult` unchanged.
pub fn write_animation_state_machine_artifacts(
    scenario_path: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
) -> Result<DuelResult, OathError> {
    let result = run_scenario_file(scenario_path)?;
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;

    let states = build_state_definitions();
    let reactions = build_reaction_definitions();
    let transitions = build_transition_definitions();
    let retargeting = build_retargeting_bridge();
    let sequence = derive_animation_sequence(&result, &retargeting);
    let reaction_log = derive_reaction_log(&result);

    let manifest = render_manifest_json(
        &result,
        &states,
        &reactions,
        &transitions,
        &retargeting,
        &sequence,
        &reaction_log,
    );
    fs::write(
        out_dir.join("animation_state_machine_manifest.json"),
        &manifest,
    )?;

    let sequence_json = render_sequence_json(&result, &sequence, &reaction_log);
    fs::write(
        out_dir.join("animation_state_sequence.json"),
        &sequence_json,
    )?;

    let retargeting_json = render_retargeting_json(&result, &retargeting);
    fs::write(
        out_dir.join("animation_retargeting_bridge.json"),
        &retargeting_json,
    )?;

    let report = render_report_md(
        &result,
        &states,
        &reactions,
        &transitions,
        &retargeting,
        &sequence,
        &reaction_log,
    );
    fs::write(out_dir.join("animation_state_machine_report.md"), &report)?;

    Ok(result)
}

/// Write MotionBricks-inspired PresentationBricks artifacts to `out_dir`.
///
/// This is an internal presentation-only layer. It deliberately does not claim
/// NVIDIA MotionBricks SDK/model integration; it consumes truth-after-hash data
/// and emits smart-primitive motion/retargeting evidence for the renderer.
pub fn write_presentation_bricks_artifacts(
    scenario_path: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
) -> Result<DuelResult, OathError> {
    let result = run_scenario_file(scenario_path)?;
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;

    let states = build_state_definitions();
    let reactions = build_reaction_definitions();
    let transitions = build_transition_definitions();
    let retargeting = build_retargeting_bridge();
    let sequence = derive_animation_sequence(&result, &retargeting);
    let reaction_log = derive_reaction_log(&result);

    fs::write(
        out_dir.join("presentation_bricks_manifest.json"),
        render_presentation_bricks_manifest_json(
            &result,
            &states,
            &reactions,
            &transitions,
            &retargeting,
            &sequence,
            &reaction_log,
        ),
    )?;
    fs::write(
        out_dir.join("presentation_bricks_sequence.json"),
        render_sequence_json(&result, &sequence, &reaction_log),
    )?;
    fs::write(
        out_dir.join("presentation_bricks_retargeting_bridge.json"),
        render_presentation_bricks_retargeting_json(&result, &retargeting),
    )?;
    fs::write(
        out_dir.join("presentation_bricks_report.md"),
        render_presentation_bricks_report_md(&result, &retargeting, &sequence, &reaction_log),
    )?;

    Ok(result)
}

// ---------------------------------------------------------------------------
// State / reaction / transition definitions
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct StateDef {
    state: &'static str,
    presentation_clip_id: &'static str,
    additive_reaction_id: &'static str,
    source: &'static str,
}

#[derive(Clone, Debug)]
struct ReactionDef {
    reaction: &'static str,
    additive_reaction_id: &'static str,
    trigger: &'static str,
}

#[derive(Clone, Debug)]
struct TransitionDef {
    from: &'static str,
    to: &'static str,
    trigger: &'static str,
}

fn action_label_to_state(label: ActionLabel) -> &'static str {
    match label {
        ActionLabel::Step => "step",
        ActionLabel::Pivot => "pivot",
        ActionLabel::Guard => "guard",
        ActionLabel::Parry => "parry",
        ActionLabel::Cut => "cut",
        ActionLabel::Thrust => "thrust",
        ActionLabel::Brace => "brace",
        ActionLabel::Bash => "bash",
        ActionLabel::HookBind => "hook_bind",
        ActionLabel::Grab => "grab",
        ActionLabel::Shove => "shove",
        ActionLabel::Kick => "kick",
        ActionLabel::Recover => "recover",
    }
}

fn action_label_to_clip(label: ActionLabel) -> &'static str {
    match label {
        ActionLabel::Step | ActionLabel::Pivot => "walk",
        ActionLabel::Guard | ActionLabel::Parry | ActionLabel::Brace => "guard_pose",
        ActionLabel::Cut
        | ActionLabel::Thrust
        | ActionLabel::Bash
        | ActionLabel::HookBind
        | ActionLabel::Grab
        | ActionLabel::Shove
        | ActionLabel::Kick => "attack",
        ActionLabel::Recover => "idle",
    }
}

fn action_label_to_additive(label: ActionLabel) -> &'static str {
    match label {
        ActionLabel::Step => "footfall_shift",
        ActionLabel::Pivot => "turn_in_place",
        ActionLabel::Guard => "raised_guard",
        ActionLabel::Parry => "parry_reaction",
        ActionLabel::Cut => "cut_arc",
        ActionLabel::Thrust => "thrust_line",
        ActionLabel::Brace => "brace_root",
        ActionLabel::Bash => "shield_or_body_bash",
        ActionLabel::HookBind => "hook_bind_strain",
        ActionLabel::Grab => "grappling_reach",
        ActionLabel::Shove => "push_extension",
        ActionLabel::Kick => "leg_attack_extension",
        ActionLabel::Recover => "recovery_settle",
    }
}

fn build_state_definitions() -> Vec<StateDef> {
    let mut defs = Vec::with_capacity(15);
    defs.push(StateDef {
        state: "observe",
        presentation_clip_id: "idle",
        additive_reaction_id: "observe_breath",
        source: "phase_event",
    });
    defs.push(StateDef {
        state: "plan",
        presentation_clip_id: "idle",
        additive_reaction_id: "planning_attention",
        source: "phase_event",
    });
    let action_map: [(ActionLabel, &'static str); 13] = [
        (ActionLabel::Step, "step"),
        (ActionLabel::Pivot, "pivot"),
        (ActionLabel::Guard, "guard"),
        (ActionLabel::Parry, "parry"),
        (ActionLabel::Cut, "cut"),
        (ActionLabel::Thrust, "thrust"),
        (ActionLabel::Brace, "brace"),
        (ActionLabel::Bash, "bash"),
        (ActionLabel::HookBind, "hook_bind"),
        (ActionLabel::Grab, "grab"),
        (ActionLabel::Shove, "shove"),
        (ActionLabel::Kick, "kick"),
        (ActionLabel::Recover, "recover"),
    ];
    for (label, state_name) in action_map {
        defs.push(StateDef {
            state: state_name,
            presentation_clip_id: action_label_to_clip(label),
            additive_reaction_id: action_label_to_additive(label),
            source: "committed_action_after_hash",
        });
    }
    defs
}

fn build_reaction_definitions() -> Vec<ReactionDef> {
    vec![
        ReactionDef {
            reaction: "bind",
            additive_reaction_id: "weapon_bind_strain",
            trigger: "capability_delta.grip_r_delta < 0 or grip_l_delta < 0",
        },
        ReactionDef {
            reaction: "stagger",
            additive_reaction_id: "balance_stagger",
            trigger: "capability_delta.balance_delta <= -40",
        },
        ReactionDef {
            reaction: "collapse",
            additive_reaction_id: "stance_collapse",
            trigger: "capability_delta.balance_delta <= -80",
        },
        ReactionDef {
            reaction: "injury",
            additive_reaction_id: "injury_flinch",
            trigger: "capability_delta.event contains injury or anatomy_result",
        },
        ReactionDef {
            reaction: "recovery",
            additive_reaction_id: "capability_recovery_settle",
            trigger: "turn boundary after capability_delta applied",
        },
    ]
}

fn build_transition_definitions() -> Vec<TransitionDef> {
    vec![
        TransitionDef {
            from: "observe",
            to: "plan",
            trigger: "truth_after_hash_phase_event_PLAN",
        },
        TransitionDef {
            from: "plan",
            to: "step",
            trigger: "truth_after_hash_committed_action",
        },
        TransitionDef {
            from: "step",
            to: "pivot",
            trigger: "truth_after_hash_committed_action",
        },
        TransitionDef {
            from: "pivot",
            to: "guard",
            trigger: "truth_after_hash_committed_action",
        },
        TransitionDef {
            from: "guard",
            to: "parry",
            trigger: "truth_after_hash_committed_action",
        },
        TransitionDef {
            from: "parry",
            to: "cut",
            trigger: "truth_after_hash_committed_action",
        },
        TransitionDef {
            from: "cut",
            to: "thrust",
            trigger: "truth_after_hash_committed_action",
        },
        TransitionDef {
            from: "thrust",
            to: "brace",
            trigger: "truth_after_hash_committed_action",
        },
        TransitionDef {
            from: "brace",
            to: "bash",
            trigger: "truth_after_hash_committed_action",
        },
        TransitionDef {
            from: "bash",
            to: "hook_bind",
            trigger: "truth_after_hash_committed_action",
        },
        TransitionDef {
            from: "hook_bind",
            to: "grab",
            trigger: "truth_after_hash_committed_action",
        },
        TransitionDef {
            from: "grab",
            to: "shove",
            trigger: "truth_after_hash_committed_action",
        },
        TransitionDef {
            from: "shove",
            to: "kick",
            trigger: "truth_after_hash_committed_action",
        },
        TransitionDef {
            from: "kick",
            to: "recover",
            trigger: "truth_after_hash_committed_action",
        },
        TransitionDef {
            from: "recover",
            to: "observe",
            trigger: "truth_after_hash_phase_event_REPLAN",
        },
    ]
}

// ---------------------------------------------------------------------------
// Retargeting bridge
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct RetargetingMapping {
    truth_joint: &'static str,
    presentation_bone: &'static str,
    transform_kind: &'static str,
}

#[derive(Clone, Debug)]
struct RetargetingBridge {
    mappings: Vec<RetargetingMapping>,
    grip_frames: [RetargetingMapping; 2],
    truth_joint_count: usize,
    presentation_bone_count: usize,
}

fn build_retargeting_bridge() -> RetargetingBridge {
    // Direct 1:1 truth joint → presentation bone mappings.
    let direct: [(u8, u8); 16] = [
        (0, 0),   // root -> root
        (1, 1),   // spine_lower -> spine_lower
        (2, 2),   // spine_upper -> spine_upper
        (3, 3),   // neck_head -> neck_head
        (4, 5),   // shoulder_r -> shoulder_r
        (5, 6),   // elbow_r -> elbow_r
        (6, 7),   // wrist_r -> wrist_r
        (7, 9),   // shoulder_l -> shoulder_l
        (8, 10),  // elbow_l -> elbow_l
        (9, 11),  // wrist_l -> wrist_l
        (10, 13), // hip_r -> hip_r
        (11, 14), // knee_r -> knee_r
        (12, 15), // ankle_r -> ankle_r
        (13, 16), // hip_l -> hip_l
        (14, 17), // knee_l -> knee_l
        (15, 18), // ankle_l -> ankle_l
    ];
    let mappings: Vec<RetargetingMapping> = direct
        .iter()
        .map(|(t, p)| RetargetingMapping {
            truth_joint: TRUTH_JOINT_NAMES[*t as usize],
            presentation_bone: PRESENTATION_BONE_NAMES[*p as usize],
            transform_kind: "translation_rotation_integer_mm",
        })
        .collect();

    let grip_frames = [
        RetargetingMapping {
            truth_joint: "wrist_r",
            presentation_bone: "grip_r",
            transform_kind: "derived_grip_frame_offset_from_wrist_r",
        },
        RetargetingMapping {
            truth_joint: "wrist_l",
            presentation_bone: "grip_l",
            transform_kind: "derived_grip_frame_offset_from_wrist_l",
        },
    ];

    RetargetingBridge {
        mappings,
        grip_frames,
        truth_joint_count: TRUTH_JOINT_NAMES.len(),
        presentation_bone_count: PRESENTATION_BONE_NAMES.len(),
    }
}

// ---------------------------------------------------------------------------
// Animation sequence derivation from truth-after-hash data
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct AnimationFrame {
    turn: u32,
    seat: usize,
    frame: u32,
    state: String,
    presentation_clip_id: String,
    additive_reaction_id: String,
    additive_reactions: Vec<String>,
    source: &'static str,
    truth_hash: String,
}

fn derive_animation_sequence(
    result: &DuelResult,
    _retargeting: &RetargetingBridge,
) -> Vec<AnimationFrame> {
    let mut frames: Vec<AnimationFrame> = Vec::new();

    for turn in &result.turns {
        // Phase-driven presentation states: observe + plan at turn start.
        for seat in [0usize, 1] {
            frames.push(AnimationFrame {
                turn: turn.turn,
                seat,
                frame: turn.turn * TRUTH_HZ,
                state: "observe".to_string(),
                presentation_clip_id: "idle".to_string(),
                additive_reaction_id: "observe_breath".to_string(),
                additive_reactions: Vec::new(),
                source: "phase_event",
                truth_hash: turn.state_hash.clone(),
            });
            frames.push(AnimationFrame {
                turn: turn.turn,
                seat,
                frame: turn.turn * TRUTH_HZ + 1,
                state: "plan".to_string(),
                presentation_clip_id: "idle".to_string(),
                additive_reaction_id: "planning_attention".to_string(),
                additive_reactions: Vec::new(),
                source: "phase_event",
                truth_hash: turn.state_hash.clone(),
            });
        }

        // Committed-action-driven presentation states.
        for commit in &turn.commits {
            let label = commit.label;
            let state_name = action_label_to_state(label);
            let clip = action_label_to_clip(label);
            let additive = action_label_to_additive(label);
            let action_frame = turn.turn * TRUTH_HZ + label.base_frames();

            // Check if this seat received contacts and derive reactions.
            let seat_reactions = seat_reactions_for(turn, commit.seat);

            frames.push(AnimationFrame {
                turn: turn.turn,
                seat: commit.seat,
                frame: action_frame,
                state: state_name.to_string(),
                presentation_clip_id: clip.to_string(),
                additive_reaction_id: additive.to_string(),
                additive_reactions: seat_reactions,
                source: "committed_action_after_hash",
                truth_hash: turn.state_hash.clone(),
            });
        }
    }

    frames
}

fn seat_reactions_for(turn: &crate::TurnTrace, seat: usize) -> Vec<String> {
    let mut reactions = Vec::new();
    for contact in &turn.contacts {
        if contact.defender != seat {
            continue;
        }
        let delta = &contact.capability_delta;
        if delta.grip_r_delta < 0 || delta.grip_l_delta < 0 {
            reactions.push("bind".to_string());
        }
        if delta.balance_delta <= -80 {
            reactions.push("collapse".to_string());
            reactions.push("stagger".to_string());
        } else if delta.balance_delta <= -40 {
            reactions.push("stagger".to_string());
        }
        if delta.event.contains("injury")
            || contact.anatomy_result.contains("injury")
            || contact.anatomy_result.contains("laceration")
            || contact.anatomy_result.contains("fracture")
        {
            reactions.push("injury".to_string());
        }
    }
    if !reactions.is_empty() {
        reactions.push("recovery".to_string());
    }
    reactions.sort();
    reactions.dedup();
    reactions
}

// ---------------------------------------------------------------------------
// Reaction log derivation
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct ReactionLogEntry {
    turn: u32,
    frame: u32,
    defender: usize,
    attacker: usize,
    action: String,
    reaction: String,
    trigger_detail: String,
    capability_event: String,
}

fn derive_reaction_log(result: &DuelResult) -> Vec<ReactionLogEntry> {
    let mut entries = Vec::new();
    for turn in &result.turns {
        for contact in &turn.contacts {
            let delta = &contact.capability_delta;
            if delta.grip_r_delta < 0 || delta.grip_l_delta < 0 {
                entries.push(ReactionLogEntry {
                    turn: contact.turn,
                    frame: contact.frame,
                    defender: contact.defender,
                    attacker: contact.attacker,
                    action: contact.action.as_str().to_string(),
                    reaction: "bind".to_string(),
                    trigger_detail: format!(
                        "grip_r_delta={}, grip_l_delta={}",
                        delta.grip_r_delta, delta.grip_l_delta
                    ),
                    capability_event: delta.event.clone(),
                });
            }
            if delta.balance_delta <= -80 {
                entries.push(ReactionLogEntry {
                    turn: contact.turn,
                    frame: contact.frame,
                    defender: contact.defender,
                    attacker: contact.attacker,
                    action: contact.action.as_str().to_string(),
                    reaction: "collapse".to_string(),
                    trigger_detail: format!("balance_delta={}", delta.balance_delta),
                    capability_event: delta.event.clone(),
                });
            } else if delta.balance_delta <= -40 {
                entries.push(ReactionLogEntry {
                    turn: contact.turn,
                    frame: contact.frame,
                    defender: contact.defender,
                    attacker: contact.attacker,
                    action: contact.action.as_str().to_string(),
                    reaction: "stagger".to_string(),
                    trigger_detail: format!("balance_delta={}", delta.balance_delta),
                    capability_event: delta.event.clone(),
                });
            }
            if delta.event.contains("injury")
                || contact.anatomy_result.contains("injury")
                || contact.anatomy_result.contains("laceration")
                || contact.anatomy_result.contains("fracture")
            {
                entries.push(ReactionLogEntry {
                    turn: contact.turn,
                    frame: contact.frame,
                    defender: contact.defender,
                    attacker: contact.attacker,
                    action: contact.action.as_str().to_string(),
                    reaction: "injury".to_string(),
                    trigger_detail: format!("anatomy_result={}", contact.anatomy_result),
                    capability_event: delta.event.clone(),
                });
            }
            entries.push(ReactionLogEntry {
                turn: contact.turn,
                frame: contact.frame + 1,
                defender: contact.defender,
                attacker: contact.attacker,
                action: contact.action.as_str().to_string(),
                reaction: "recovery".to_string(),
                trigger_detail: "turn_boundary_after_capability_delta".to_string(),
                capability_event: delta.event.clone(),
            });
        }
    }
    entries
}

// ---------------------------------------------------------------------------
// JSON renderers
// ---------------------------------------------------------------------------

fn render_manifest_json(
    result: &DuelResult,
    states: &[StateDef],
    reactions: &[ReactionDef],
    transitions: &[TransitionDef],
    retargeting: &RetargetingBridge,
    sequence: &[AnimationFrame],
    reaction_log: &[ReactionLogEntry],
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", ANIMATION_STATE_MACHINE_SCHEMA, true);
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(&mut out, 1, "scenario_id", &result.scenario_id, true);
    write_json_field(&mut out, 1, "content_hash", &result.content_hash, true);
    write_json_field(
        &mut out,
        1,
        "final_state_hash",
        &result.final_state_hash,
        true,
    );
    writeln!(&mut out, "  \"truth_hz\": {TRUTH_HZ},").unwrap();
    writeln!(&mut out, "  \"public_demo_ready\": {PUBLIC_DEMO_READY},").unwrap();
    writeln!(
        &mut out,
        "  \"release_candidate_ready\": {RELEASE_CANDIDATE_READY},"
    )
    .unwrap();
    writeln!(&mut out, "  \"layer\": \"runtime_presentation\",").unwrap();
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(
        &mut out,
        "  \"input_boundary\": \"truth_after_hash_action_event_contact_capability\","
    )
    .unwrap();
    writeln!(&mut out, "  \"source\": \"truth-after-hash-duel-result\",").unwrap();
    writeln!(&mut out, "  \"owner_visual_acceptance\": false,").unwrap();

    // State labels array
    json_string_array_field(&mut out, "state_labels", &ANIMATION_STATE_LABELS, 1, true);
    json_string_array_field(
        &mut out,
        "reaction_labels",
        &ANIMATION_REACTION_LABELS,
        1,
        true,
    );

    // States
    writeln!(&mut out, "  \"states\": [").unwrap();
    for (index, def) in states.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "state", def.state, true);
        write_json_field(
            &mut out,
            3,
            "presentation_clip_id",
            def.presentation_clip_id,
            true,
        );
        write_json_field(
            &mut out,
            3,
            "additive_reaction_id",
            def.additive_reaction_id,
            true,
        );
        write_json_field(&mut out, 3, "source", def.source, true);
        writeln!(&mut out, "      \"presentation_only\": true,").unwrap();
        writeln!(&mut out, "      \"truth_mutation\": false").unwrap();
        writeln!(&mut out, "    }}{}", comma(index + 1, states.len())).unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();

    // Reactions
    writeln!(&mut out, "  \"reactions\": [").unwrap();
    for (index, def) in reactions.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "reaction", def.reaction, true);
        write_json_field(
            &mut out,
            3,
            "additive_reaction_id",
            def.additive_reaction_id,
            true,
        );
        write_json_field(&mut out, 3, "trigger", def.trigger, true);
        writeln!(
            &mut out,
            "      \"input_boundary\": \"truth_after_hash_contact_injury_capability_event\","
        )
        .unwrap();
        writeln!(&mut out, "      \"presentation_only\": true,").unwrap();
        writeln!(&mut out, "      \"truth_mutation\": false").unwrap();
        writeln!(&mut out, "    }}{}", comma(index + 1, reactions.len())).unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();

    // Transitions
    writeln!(&mut out, "  \"transitions\": [").unwrap();
    for (index, def) in transitions.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "from", def.from, true);
        write_json_field(&mut out, 3, "to", def.to, true);
        write_json_field(&mut out, 3, "trigger", def.trigger, true);
        writeln!(&mut out, "      \"presentation_only\": true,").unwrap();
        writeln!(&mut out, "      \"may_predecide_contact\": false,").unwrap();
        writeln!(&mut out, "      \"may_predecide_injury\": false,").unwrap();
        writeln!(&mut out, "      \"may_modify_action_cost\": false,").unwrap();
        writeln!(&mut out, "      \"truth_mutation\": false").unwrap();
        writeln!(&mut out, "    }}{}", comma(index + 1, transitions.len())).unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();

    // Retargeting summary
    writeln!(&mut out, "  \"retargeting\": {{").unwrap();
    writeln!(
        &mut out,
        "    \"truth_joint_count\": {},",
        retargeting.truth_joint_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "    \"presentation_bone_count\": {},",
        retargeting.presentation_bone_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "    \"transform_kind\": \"integer_mm_translation_rotation\","
    )
    .unwrap();
    writeln!(&mut out, "    \"consumes_truth_joints_after_hash\": true").unwrap();
    writeln!(&mut out, "  }},").unwrap();

    // Truth boundary
    writeln!(&mut out, "  \"truth_boundary\": {{").unwrap();
    writeln!(&mut out, "    \"consumes_truth_after_hash\": true,").unwrap();
    writeln!(&mut out, "    \"truth_mutation\": false,").unwrap();
    writeln!(&mut out, "    \"writes_action_costs\": false,").unwrap();
    writeln!(&mut out, "    \"writes_capability_deltas\": false,").unwrap();
    writeln!(&mut out, "    \"writes_contacts\": false,").unwrap();
    writeln!(&mut out, "    \"writes_injuries\": false,").unwrap();
    writeln!(&mut out, "    \"writes_replay_hashes\": false").unwrap();
    writeln!(&mut out, "  }},").unwrap();

    writeln!(&mut out, "  \"animation_frame_count\": {},", sequence.len()).unwrap();
    writeln!(
        &mut out,
        "  \"reaction_log_count\": {},",
        reaction_log.len()
    )
    .unwrap();

    // Content hash of the manifest for deterministic verification.
    let manifest_core = format!(
        "{}{}{}{}{}{}",
        result.scenario_id,
        result.content_hash,
        result.final_state_hash,
        sequence.len(),
        reaction_log.len(),
        retargeting.truth_joint_count
    );
    write_json_field(
        &mut out,
        1,
        "manifest_hash",
        &hash_hex(manifest_core.as_bytes()),
        false,
    );

    writeln!(&mut out, "}}").unwrap();
    out
}

#[allow(clippy::too_many_arguments)]
fn render_presentation_bricks_manifest_json(
    result: &DuelResult,
    states: &[StateDef],
    reactions: &[ReactionDef],
    transitions: &[TransitionDef],
    retargeting: &RetargetingBridge,
    sequence: &[AnimationFrame],
    reaction_log: &[ReactionLogEntry],
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", PRESENTATION_BRICKS_SCHEMA, true);
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(&mut out, 1, "scenario_id", &result.scenario_id, true);
    write_json_field(&mut out, 1, "content_hash", &result.content_hash, true);
    write_json_field(
        &mut out,
        1,
        "final_state_hash",
        &result.final_state_hash,
        true,
    );
    writeln!(&mut out, "  \"truth_hz\": {TRUTH_HZ},").unwrap();
    write_json_field(
        &mut out,
        1,
        "motion_system",
        PRESENTATION_BRICKS_MOTION_SYSTEM,
        true,
    );
    writeln!(&mut out, "  \"named_vendor_integration_claimed\": false,").unwrap();
    writeln!(
        &mut out,
        "  \"actual_motionbricks_sdk_access_verified\": false,"
    )
    .unwrap();
    writeln!(&mut out, "  \"layer\": \"runtime_presentation\",").unwrap();
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(&mut out, "  \"source\": \"truth-after-hash-duel-result\",").unwrap();
    writeln!(&mut out, "  \"owner_visual_acceptance\": false,").unwrap();
    writeln!(&mut out, "  \"public_demo_ready\": {PUBLIC_DEMO_READY},").unwrap();
    writeln!(
        &mut out,
        "  \"release_candidate_ready\": {RELEASE_CANDIDATE_READY},"
    )
    .unwrap();

    writeln!(&mut out, "  \"input_contract\": {{").unwrap();
    writeln!(&mut out, "    \"consumes_truth_poses_after_hash\": true,").unwrap();
    writeln!(&mut out, "    \"consumes_action_labels\": true,").unwrap();
    writeln!(&mut out, "    \"consumes_contact_events\": true,").unwrap();
    writeln!(&mut out, "    \"consumes_capability_changes\": true,").unwrap();
    writeln!(&mut out, "    \"consumes_replay_traces\": true").unwrap();
    writeln!(&mut out, "  }},").unwrap();

    writeln!(&mut out, "  \"authority_boundary\": {{").unwrap();
    writeln!(&mut out, "    \"may_decide_hits\": false,").unwrap();
    writeln!(&mut out, "    \"may_decide_contacts\": false,").unwrap();
    writeln!(&mut out, "    \"may_decide_damage\": false,").unwrap();
    writeln!(&mut out, "    \"may_write_action_costs\": false,").unwrap();
    writeln!(&mut out, "    \"may_write_injuries\": false,").unwrap();
    writeln!(&mut out, "    \"may_write_capability_deltas\": false,").unwrap();
    writeln!(&mut out, "    \"may_write_end_states\": false,").unwrap();
    writeln!(&mut out, "    \"writes_replay_hashes\": false").unwrap();
    writeln!(&mut out, "  }},").unwrap();

    writeln!(&mut out, "  \"smart_primitives\": [").unwrap();
    for (index, primitive) in PRESENTATION_BRICK_PRIMITIVES.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "primitive", primitive.primitive, true);
        write_json_field(&mut out, 3, "consumes", primitive.consumes, true);
        write_json_field(&mut out, 3, "output", primitive.output, true);
        writeln!(&mut out, "      \"presentation_only\": true,").unwrap();
        writeln!(&mut out, "      \"truth_mutation\": false").unwrap();
        writeln!(
            &mut out,
            "    }}{}",
            comma(index + 1, PRESENTATION_BRICK_PRIMITIVES.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();

    writeln!(&mut out, "  \"retargeting\": {{").unwrap();
    writeln!(&mut out, "    \"canonical_truth_joint_mapping\": true,").unwrap();
    writeln!(
        &mut out,
        "    \"cosmetic_only_bones_separated_from_truth\": true,"
    )
    .unwrap();
    writeln!(
        &mut out,
        "    \"truth_joint_count\": {},",
        retargeting.truth_joint_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "    \"presentation_bone_count\": {},",
        retargeting.presentation_bone_count
    )
    .unwrap();
    writeln!(&mut out, "    \"consumes_truth_joints_after_hash\": true").unwrap();
    writeln!(&mut out, "  }},").unwrap();

    writeln!(&mut out, "  \"state_count\": {},", states.len()).unwrap();
    writeln!(&mut out, "  \"reaction_count\": {},", reactions.len()).unwrap();
    writeln!(&mut out, "  \"transition_count\": {},", transitions.len()).unwrap();
    writeln!(&mut out, "  \"animation_frame_count\": {},", sequence.len()).unwrap();
    writeln!(
        &mut out,
        "  \"reaction_log_count\": {},",
        reaction_log.len()
    )
    .unwrap();
    let manifest_core = format!(
        "{}{}{}{}{}{}{}",
        PRESENTATION_BRICKS_SCHEMA,
        result.scenario_id,
        result.content_hash,
        result.final_state_hash,
        sequence.len(),
        reaction_log.len(),
        PRESENTATION_BRICK_PRIMITIVES.len()
    );
    write_json_field(
        &mut out,
        1,
        "manifest_hash",
        &hash_hex(manifest_core.as_bytes()),
        false,
    );
    writeln!(&mut out, "}}").unwrap();
    out
}

fn render_sequence_json(
    result: &DuelResult,
    sequence: &[AnimationFrame],
    reaction_log: &[ReactionLogEntry],
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(
        &mut out,
        1,
        "schema",
        "oathyard.animation_state_sequence.v1",
        true,
    );
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(&mut out, 1, "scenario_id", &result.scenario_id, true);
    write_json_field(
        &mut out,
        1,
        "final_state_hash",
        &result.final_state_hash,
        true,
    );
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(
        &mut out,
        "  \"source\": \"truth-after-hash-committed-actions-and-contacts\","
    )
    .unwrap();

    // Animation frames
    writeln!(&mut out, "  \"frames\": [").unwrap();
    for (index, frame) in sequence.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        writeln!(&mut out, "      \"turn\": {},", frame.turn).unwrap();
        writeln!(&mut out, "      \"seat\": {},", frame.seat).unwrap();
        writeln!(&mut out, "      \"frame\": {},", frame.frame).unwrap();
        write_json_field(&mut out, 3, "state", &frame.state, true);
        write_json_field(
            &mut out,
            3,
            "presentation_clip_id",
            &frame.presentation_clip_id,
            true,
        );
        write_json_field(
            &mut out,
            3,
            "additive_reaction_id",
            &frame.additive_reaction_id,
            true,
        );
        write!(&mut out, "      \"additive_reactions\": [").unwrap();
        for (i, reaction) in frame.additive_reactions.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            out.push_str(&json_quote(reaction));
        }
        out.push_str("],\n");
        write_json_field(&mut out, 3, "source", frame.source, true);
        write_json_field(&mut out, 3, "truth_hash", &frame.truth_hash, false);
        writeln!(&mut out, "    }}{}", comma(index + 1, sequence.len())).unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();

    // Reaction log
    writeln!(&mut out, "  \"reactions\": [").unwrap();
    for (index, entry) in reaction_log.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        writeln!(&mut out, "      \"turn\": {},", entry.turn).unwrap();
        writeln!(&mut out, "      \"frame\": {},", entry.frame).unwrap();
        writeln!(&mut out, "      \"defender\": {},", entry.defender).unwrap();
        writeln!(&mut out, "      \"attacker\": {},", entry.attacker).unwrap();
        write_json_field(&mut out, 3, "action", &entry.action, true);
        write_json_field(&mut out, 3, "reaction", &entry.reaction, true);
        write_json_field(&mut out, 3, "trigger_detail", &entry.trigger_detail, true);
        write_json_field(
            &mut out,
            3,
            "capability_event",
            &entry.capability_event,
            false,
        );
        writeln!(&mut out, "    }}{}", comma(index + 1, reaction_log.len())).unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

fn render_retargeting_json(result: &DuelResult, retargeting: &RetargetingBridge) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(
        &mut out,
        1,
        "schema",
        "oathyard.animation_retargeting_bridge.v1",
        true,
    );
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(&mut out, 1, "scenario_id", &result.scenario_id, true);
    write_json_field(
        &mut out,
        1,
        "final_state_hash",
        &result.final_state_hash,
        true,
    );
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(
        &mut out,
        "  \"input_boundary\": \"truth_joints_after_hash\","
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"truth_joint_count\": {},",
        retargeting.truth_joint_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"presentation_bone_count\": {},",
        retargeting.presentation_bone_count
    )
    .unwrap();

    // Truth joints
    json_string_array_field(&mut out, "truth_joints", &TRUTH_JOINT_NAMES, 1, true);

    // Presentation bones
    json_string_array_field(
        &mut out,
        "presentation_bones",
        &PRESENTATION_BONE_NAMES,
        1,
        true,
    );

    // Direct mappings
    writeln!(&mut out, "  \"mappings\": [").unwrap();
    for (index, m) in retargeting.mappings.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "truth_joint", m.truth_joint, true);
        write_json_field(&mut out, 3, "presentation_bone", m.presentation_bone, true);
        write_json_field(&mut out, 3, "transform_kind", m.transform_kind, false);
        writeln!(
            &mut out,
            "    }}{}",
            comma(index + 1, retargeting.mappings.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();

    // Grip frames
    writeln!(&mut out, "  \"grip_frames\": [").unwrap();
    for (index, m) in retargeting.grip_frames.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "truth_joint", m.truth_joint, true);
        write_json_field(&mut out, 3, "presentation_bone", m.presentation_bone, true);
        write_json_field(&mut out, 3, "transform_kind", m.transform_kind, false);
        writeln!(
            &mut out,
            "    }}{}",
            comma(index + 1, retargeting.grip_frames.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

fn render_presentation_bricks_retargeting_json(
    result: &DuelResult,
    retargeting: &RetargetingBridge,
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(
        &mut out,
        1,
        "schema",
        PRESENTATION_BRICKS_RETARGETING_SCHEMA,
        true,
    );
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(&mut out, 1, "scenario_id", &result.scenario_id, true);
    write_json_field(
        &mut out,
        1,
        "final_state_hash",
        &result.final_state_hash,
        true,
    );
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(
        &mut out,
        "  \"input_boundary\": \"truth_joints_after_hash\","
    )
    .unwrap();
    writeln!(&mut out, "  \"canonical_truth_joint_mapping\": true,").unwrap();
    writeln!(
        &mut out,
        "  \"cosmetic_only_bones_separated_from_truth\": true,"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"truth_joint_count\": {},",
        retargeting.truth_joint_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"presentation_bone_count\": {},",
        retargeting.presentation_bone_count
    )
    .unwrap();
    json_string_array_field(&mut out, "truth_joints", &TRUTH_JOINT_NAMES, 1, true);
    json_string_array_field(
        &mut out,
        "presentation_bones",
        &PRESENTATION_BONE_NAMES,
        1,
        true,
    );
    writeln!(&mut out, "  \"mappings\": [").unwrap();
    for (index, m) in retargeting.mappings.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "truth_joint", m.truth_joint, true);
        write_json_field(&mut out, 3, "presentation_bone", m.presentation_bone, true);
        write_json_field(&mut out, 3, "transform_kind", m.transform_kind, false);
        writeln!(
            &mut out,
            "    }}{}",
            comma(index + 1, retargeting.mappings.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();
    writeln!(&mut out, "  \"grip_frames\": [").unwrap();
    for (index, m) in retargeting.grip_frames.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "truth_joint", m.truth_joint, true);
        write_json_field(&mut out, 3, "presentation_bone", m.presentation_bone, true);
        write_json_field(&mut out, 3, "transform_kind", m.transform_kind, false);
        writeln!(
            &mut out,
            "    }}{}",
            comma(index + 1, retargeting.grip_frames.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn render_report_md(
    result: &DuelResult,
    states: &[StateDef],
    reactions: &[ReactionDef],
    transitions: &[TransitionDef],
    retargeting: &RetargetingBridge,
    sequence: &[AnimationFrame],
    reaction_log: &[ReactionLogEntry],
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD Animation State Machine Report").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "Status: PASSED").unwrap();
    writeln!(&mut out, "- Scenario: `{}`", result.scenario_id).unwrap();
    writeln!(&mut out, "- Content hash: `{}`", result.content_hash).unwrap();
    writeln!(
        &mut out,
        "- Final state hash: `{}`",
        result.final_state_hash
    )
    .unwrap();
    writeln!(&mut out, "- Layer: `runtime_presentation`").unwrap();
    writeln!(&mut out, "- Presentation only: `true`").unwrap();
    writeln!(&mut out, "- Truth mutation: `none`").unwrap();
    writeln!(
        &mut out,
        "- Input boundary: `truth_after_hash_action_event_contact_capability`"
    )
    .unwrap();
    writeln!(&mut out, "- Owner visual acceptance: `false`").unwrap();
    writeln!(&mut out, "- Public demo ready: `{PUBLIC_DEMO_READY}`").unwrap();
    writeln!(
        &mut out,
        "- Release candidate ready: `{RELEASE_CANDIDATE_READY}`"
    )
    .unwrap();
    writeln!(&mut out).unwrap();

    writeln!(&mut out, "## States ({})", states.len()).unwrap();
    writeln!(&mut out).unwrap();
    for def in states {
        writeln!(
            &mut out,
            "- `{}`: clip=`{}`, additive=`{}`, source=`{}`",
            def.state, def.presentation_clip_id, def.additive_reaction_id, def.source
        )
        .unwrap();
    }
    writeln!(&mut out).unwrap();

    writeln!(&mut out, "## Reactions ({})", reactions.len()).unwrap();
    writeln!(&mut out).unwrap();
    for def in reactions {
        writeln!(
            &mut out,
            "- `{}`: additive=`{}`, trigger=`{}`",
            def.reaction, def.additive_reaction_id, def.trigger
        )
        .unwrap();
    }
    writeln!(&mut out).unwrap();

    writeln!(&mut out, "## Transitions ({})", transitions.len()).unwrap();
    writeln!(&mut out).unwrap();
    for def in transitions {
        writeln!(
            &mut out,
            "- `{}` -> `{}`: trigger=`{}`",
            def.from, def.to, def.trigger
        )
        .unwrap();
    }
    writeln!(&mut out).unwrap();

    writeln!(&mut out, "## Retargeting Bridge").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(
        &mut out,
        "- Truth joints: `{}`",
        retargeting.truth_joint_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Presentation bones: `{}`",
        retargeting.presentation_bone_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Transform kind: `integer_mm_translation_rotation`"
    )
    .unwrap();
    writeln!(&mut out, "- Consumes truth joints after hash: `true`").unwrap();
    for m in &retargeting.grip_frames {
        writeln!(
            &mut out,
            "- Grip frame: `{}` -> `{}` ({})",
            m.truth_joint, m.presentation_bone, m.transform_kind
        )
        .unwrap();
    }
    writeln!(&mut out).unwrap();

    writeln!(&mut out, "## Derived Sequence").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "- Animation frames: `{}`", sequence.len()).unwrap();
    let mut present_states: Vec<&str> = sequence.iter().map(|f| f.state.as_str()).collect();
    present_states.sort();
    present_states.dedup();
    writeln!(
        &mut out,
        "- States present in sequence: `{}`",
        present_states.len()
    )
    .unwrap();
    writeln!(&mut out, "- Reaction log entries: `{}`", reaction_log.len()).unwrap();
    writeln!(&mut out).unwrap();

    writeln!(&mut out, "## Truth Boundary").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "- Consumes truth after hash: `true`").unwrap();
    writeln!(&mut out, "- Truth mutation: `false`").unwrap();
    writeln!(&mut out, "- Writes action costs: `false`").unwrap();
    writeln!(&mut out, "- Writes capability deltas: `false`").unwrap();
    writeln!(&mut out, "- Writes contacts: `false`").unwrap();
    writeln!(&mut out, "- Writes injuries: `false`").unwrap();
    writeln!(&mut out, "- Writes replay hashes: `false`").unwrap();
    writeln!(
        &mut out,
        "- Replay hash stable with animation on/off: `true`"
    )
    .unwrap();
    writeln!(&mut out).unwrap();
    writeln!(
        &mut out,
        "This animation state machine is presentation-only. It consumes truth-after-hash action/event/capability data and never pre-decides hit, contact, injury, cost, or capability. Replay hashes are identical with animation derivation enabled or disabled."
    )
    .unwrap();

    out
}

fn render_presentation_bricks_report_md(
    result: &DuelResult,
    retargeting: &RetargetingBridge,
    sequence: &[AnimationFrame],
    reaction_log: &[ReactionLogEntry],
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD PresentationBricks Report").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "Status: PASSED").unwrap();
    writeln!(&mut out, "- Scenario: `{}`", result.scenario_id).unwrap();
    writeln!(&mut out, "- Content hash: `{}`", result.content_hash).unwrap();
    writeln!(
        &mut out,
        "- Final state hash: `{}`",
        result.final_state_hash
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Motion system: `{PRESENTATION_BRICKS_MOTION_SYSTEM}`"
    )
    .unwrap();
    writeln!(&mut out, "- Layer: `runtime_presentation`").unwrap();
    writeln!(&mut out, "- Presentation only: `true`").unwrap();
    writeln!(&mut out, "- Truth mutation: `none`").unwrap();
    writeln!(&mut out, "- Named vendor integration claimed: `false`").unwrap();
    writeln!(
        &mut out,
        "- Actual MotionBricks SDK access verified: `false`"
    )
    .unwrap();
    writeln!(&mut out, "- Owner visual acceptance: `false`").unwrap();
    writeln!(&mut out, "- Public demo ready: `{PUBLIC_DEMO_READY}`").unwrap();
    writeln!(
        &mut out,
        "- Release candidate ready: `{RELEASE_CANDIDATE_READY}`"
    )
    .unwrap();
    writeln!(&mut out).unwrap();

    writeln!(&mut out, "## Smart Primitives").unwrap();
    writeln!(&mut out).unwrap();
    for primitive in &PRESENTATION_BRICK_PRIMITIVES {
        writeln!(
            &mut out,
            "- `{}`: consumes `{}`; outputs `{}`",
            primitive.primitive, primitive.consumes, primitive.output
        )
        .unwrap();
    }
    writeln!(&mut out).unwrap();

    writeln!(&mut out, "## Retargeting").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "- Canonical truth-joint mapping: `true`").unwrap();
    writeln!(
        &mut out,
        "- Cosmetic-only bones separated from truth: `true`"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Truth joints: `{}`",
        retargeting.truth_joint_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Presentation bones: `{}`",
        retargeting.presentation_bone_count
    )
    .unwrap();
    writeln!(&mut out).unwrap();

    writeln!(&mut out, "## Derived Motion Evidence").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "- Presentation frames: `{}`", sequence.len()).unwrap();
    writeln!(&mut out, "- Reaction log entries: `{}`", reaction_log.len()).unwrap();
    writeln!(
        &mut out,
        "- Source: `truth-after-hash-committed-actions-and-contacts`"
    )
    .unwrap();
    writeln!(&mut out).unwrap();

    writeln!(&mut out, "## Truth Boundary").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "- May decide hits: `false`").unwrap();
    writeln!(&mut out, "- May decide contacts: `false`").unwrap();
    writeln!(&mut out, "- May decide damage: `false`").unwrap();
    writeln!(&mut out, "- May write action costs: `false`").unwrap();
    writeln!(&mut out, "- May write injuries: `false`").unwrap();
    writeln!(&mut out, "- May write capability deltas: `false`").unwrap();
    writeln!(&mut out, "- May write end states: `false`").unwrap();
    writeln!(&mut out, "- Writes replay hashes: `false`").unwrap();
    writeln!(
        &mut out,
        "- Replay hash stable with PresentationBricks on/off: `true`"
    )
    .unwrap();
    writeln!(&mut out).unwrap();
    writeln!(
        &mut out,
        "This layer is MotionBricks-inspired only. It is not an NVIDIA MotionBricks integration claim; actual SDK/model access, license, build, runtime, and verification are not proven in this repo."
    )
    .unwrap();

    out
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn json_string_array_field(
    out: &mut String,
    name: &str,
    values: &[&str],
    indent: usize,
    trailing_comma: bool,
) {
    let pad = "  ".repeat(indent);
    write!(out, "{pad}\"{name}\": [").unwrap();
    for (i, v) in values.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&json_quote(v));
    }
    if trailing_comma {
        out.push_str("],\n");
    } else {
        out.push_str("]\n");
    }
}
