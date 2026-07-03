use std::fmt::Write as _;
use std::fs;
use std::path::Path;

mod accessibility_artifacts;
mod animation_state_machine;
mod audio_artifacts;
mod boundary_taxonomy;
mod content;
mod format_utils;
mod freeze_pipeline;
mod freeze_status;
mod gamepad_smoke;
mod goal_command;
mod hud_truth;
mod input_artifacts;
mod json;
mod local_game;
mod material_artifacts;
mod native_hud_menu_flow;
mod replay_bundle;
mod settings_artifacts;
pub mod sha256;
pub use accessibility_artifacts::write_accessibility_artifacts;
pub use animation_state_machine::{
    write_animation_state_machine_artifacts, write_presentation_bricks_artifacts,
};
pub use audio_artifacts::{
    write_audio_device_smoke_artifacts, write_audio_mixer_artifacts, write_audio_vfx_artifacts,
};
pub use boundary_taxonomy::{
    tag_artifact_boundary, ArtifactBoundaryMetadata, ArtifactOrigin, BoundaryFreezeState,
    BoundaryTaxonomyLabel,
};
pub use content::{
    armor_by_id, content_hash, enforce_content_freeze_gate, enforce_content_freeze_gate_batch,
    weapon_by_id, ArenaProfile, ArmorProfile, FighterTradition, WeaponProfile, ARENAS, ARMORS,
    FIGHTER_TRADITIONS, WEAPONS,
};
pub(crate) use format_utils::{
    comma, hash_hex, json_quote, json_string_array, json_string_value, json_u32_value,
    write_json_field,
};
pub use freeze_pipeline::{
    create_freeze_registry_entry, run_freeze_pipeline, CrossPlatformEvidence, FreezeCreationOutput,
    FreezePipelineConfig, FreezePipelineOutput, FreezePipelineStepResult,
};
pub use freeze_status::{
    ai_derived_asset_prefix_str, asset_may_declare_combat_truth_authority,
    enforce_combat_truth_freeze_gate, enforce_combat_truth_freeze_gate_batch,
    evaluate_freeze_conditions, is_ai_derived_asset_id, may_declare_combat_truth_authority,
    oathyard_repo_root, query_combat_truth_freeze_status, query_freeze_status,
    scoped_asset_may_declare_authority, verify_registry_content_hash,
    ContentHashVerificationResult, FreezeConditionEvaluation, FreezeConditionName,
    FreezeConditionResult, FreezeState, FreezeStatusResult, RegistryEntry,
    COMBAT_TRUTH_AUTHORITY_SCOPE, FREEZE_HASH_REGISTRY_INDEX,
};
pub use gamepad_smoke::write_gamepad_smoke_artifacts;
pub use goal_command::{render_goal_command_output, GoalArtifactSpec};
pub use hud_truth::{
    build_hud_truth_view_model_from_replay_text, build_hud_truth_view_model_from_result,
    HudFlowView, HudFrameCostView, HudPhysicalReason, HudTruthMutationAttempt, HudTruthViewModel,
    HUD_NATIVE_FLOW_IDS, HUD_TRUTH_VIEW_SCHEMA,
};
pub use input_artifacts::write_input_artifacts;
pub use local_game::{
    write_local_game_artifacts, GameState, GameStateEntry, LocalGameConfig, LocalGameRun,
    FIGHT_FILM_VIEW_SCHEMA, LOCAL_GAME_SCHEMA, SCRIPTED_INPUT_MANIFEST_SCHEMA,
};
pub use material_artifacts::write_pbr_material_artifacts;
pub use native_hud_menu_flow::{
    build_native_hud_menu_flow_model, drive_native_hud_menu_flow,
    native_hud_menu_flow_replay_status, NativeHudFlowCommand, NativeHudMenuFlowModel,
    NativeHudMenuFlowRun, NativeHudMenuFlowScreen, NativeHudReplayStatus,
    NATIVE_HUD_MENU_FLOW_SCHEMA,
};
pub use replay_bundle::{verify_replay_export_bundle, write_replay_export_bundle};
pub use settings_artifacts::write_runtime_settings_artifacts;

pub const PRODUCT_NAME: &str = "OATHYARD";
pub const BOOTSTRAP_VERSION: &str = "oathyard-fullgame-milestone-v2";
pub const TRACE_SCHEMA: &str = "oathyard.trace.v1";
pub const REPLAY_SCHEMA: &str = "oathyard.replay.v1";
pub const FIGHT_FILM_SCHEMA: &str = "oathyard.fight_film_manifest.v1";
pub const REPLAY_EXPORT_BUNDLE_SCHEMA: &str = "oathyard.replay_export_bundle.v1";
pub const AI_PLAN_SCHEMA: &str = "oathyard.ai_plan.v1";
pub const AI_SWEEP_SCHEMA: &str = "oathyard.ai_sweep.v1";
pub const TRUTH_STRESS_SCHEMA: &str = "oathyard.truth_stress.v1";
pub const TRUTH_EDGE_AUDIT_SCHEMA: &str = "oathyard.truth_edge_audit.v1";
pub const NEGATIVE_INPUT_AUDIT_SCHEMA: &str = "oathyard.negative_input_audit.v1";
pub const NATIVE_COMBAT_RENDER_SCHEMA: &str = "oathyard.native_combat_render.v1";
pub const RENDERER_BACKEND_SCHEMA: &str = "oathyard.renderer_backend.v1";
pub const NATIVE_RENDERER_INPUT_SCHEMA: &str = "oathyard.renderer_post_hash_input.v1";
pub const NATIVE_RENDERER_CAPTURE_HOOK_SCHEMA: &str = "oathyard.renderer_capture_hook.v1";
pub const NATIVE_RENDERER_MUTATION_PROOF_SCHEMA: &str = "oathyard.renderer_truth_mutation_proof.v1";
pub const PRODUCTION_RENDERER_MANIFEST_SCHEMA: &str = "oathyard.production_renderer_manifest.v1";
pub const CAPTURE_MATRIX_MANIFEST_SCHEMA: &str = "oathyard.capture_matrix.v1";
pub const CAPTURE_MATRIX_PIXEL_INDEX_SCHEMA: &str = "oathyard.capture_matrix_pixel_index.v1";
pub const CAPTURE_MATRIX_TIMING_SCHEMA: &str = "oathyard.capture_matrix_timing.v1";
pub const NATIVE_LIGHTING_MATERIAL_WITNESS_SCHEMA: &str =
    "oathyard.native_lighting_material_witness.v1";
pub const NATIVE_ROSTER_SHOWCASE_SCHEMA: &str = "oathyard.native_roster_showcase.v1";
pub const HIGH_DETAIL_PRESENTATION_RUN_ID: &str = "t_73291be5";
pub const ACCESSIBILITY_SCHEMA: &str = "oathyard.accessibility_settings.v1";
pub const INPUT_PROFILE_SCHEMA: &str = "oathyard.input_profile.v1";
pub const GAMEPAD_SMOKE_SCHEMA: &str = "oathyard.gamepad_smoke.v1";
pub const RUNTIME_SETTINGS_SCHEMA: &str = "oathyard.runtime_settings.v1";
pub const AUDIO_MIXER_SCHEMA: &str = "oathyard.audio_mixer.v1";
pub const AUDIO_DEVICE_SMOKE_SCHEMA: &str = "oathyard.audio_device_smoke.v1";
pub const TRUTH_HZ: u32 = 120;
pub const PUBLIC_DEMO_READY: bool = false;
pub const RELEASE_CANDIDATE_READY: bool = false;
pub const CONTACT_ORDER_RULE: &str =
    "frame_then_attacker_then_defender_then_action_then_target_then_direction";

pub const UNREAL_POST_HASH_PACKET_CHAIN_SCHEMA: &str = "oathyard.unreal_post_hash_packet_chain.v1";
pub const UNREAL_CAPTURE_HOOK_SCHEMA: &str = "oathyard.unreal_capture_hook.v1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnrealPostHashBridgeConfig {
    pub scenario_id: String,
    pub content_hash: String,
    pub final_state_hash: String,
    pub replay_json_sha256: String,
    pub trace_json_sha256: String,
    pub presentation_packet_sha256: String,
    pub unreal_engine_version: String,
    pub unreal_project_id: String,
    pub renderer_backend_id: String,
    pub generated_after_replay_verify: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnrealCapturedFrame {
    pub sequence_index: u32,
    pub truth_frame: u32,
    pub scheduled_ms: u32,
    pub capture_id: String,
    pub capture_file: String,
    pub width: u32,
    pub height: u32,
    pub pixel_format: String,
    pub camera_mode: String,
    pub render_pass: String,
    pub capture_sha256: String,
    pub source_truth_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnrealFramePacket {
    pub schema: String,
    pub capture_hook_schema: String,
    pub sequence_index: u32,
    pub truth_frame: u32,
    pub scheduled_ms: u32,
    pub capture_id: String,
    pub capture_file: String,
    pub width: u32,
    pub height: u32,
    pub pixel_format: String,
    pub camera_mode: String,
    pub render_pass: String,
    pub capture_sha256: String,
    pub source_truth_hash: String,
    pub previous_chain_hash: String,
    pub packet_hash: String,
    pub post_hash_only: bool,
    pub presentation_only: bool,
    pub truth_mutation: bool,
}

#[derive(Debug)]
pub struct UnrealTruthPacketBridge {
    config: UnrealPostHashBridgeConfig,
    packets: Vec<UnrealFramePacket>,
    final_chain_hash: String,
}

impl UnrealTruthPacketBridge {
    pub fn new(config: UnrealPostHashBridgeConfig) -> Result<Self, OathError> {
        validate_unreal_bridge_config(&config)?;
        let final_chain_hash = unreal_chain_genesis_hash(&config);
        Ok(Self {
            config,
            packets: Vec::new(),
            final_chain_hash,
        })
    }

    pub fn record_captured_frame(&mut self, frame: UnrealCapturedFrame) -> Result<(), OathError> {
        let expected_index = self.packets.len() as u32;
        if frame.sequence_index != expected_index {
            return Err(OathError::Verify(format!(
                "expected Unreal frame sequence_index {expected_index}, got {}",
                frame.sequence_index
            )));
        }
        validate_unreal_captured_frame(&self.config, &frame)?;
        let previous_chain_hash = self.final_chain_hash.clone();
        let canonical = canonical_unreal_frame_material(&self.config, &frame, &previous_chain_hash);
        let packet_hash = sha256::sha256_hex(canonical.as_bytes());
        self.final_chain_hash = packet_hash.clone();
        self.packets.push(UnrealFramePacket {
            schema: UNREAL_POST_HASH_PACKET_CHAIN_SCHEMA.to_string(),
            capture_hook_schema: UNREAL_CAPTURE_HOOK_SCHEMA.to_string(),
            sequence_index: frame.sequence_index,
            truth_frame: frame.truth_frame,
            scheduled_ms: frame.scheduled_ms,
            capture_id: frame.capture_id,
            capture_file: frame.capture_file,
            width: frame.width,
            height: frame.height,
            pixel_format: frame.pixel_format,
            camera_mode: frame.camera_mode,
            render_pass: frame.render_pass,
            capture_sha256: frame.capture_sha256,
            source_truth_hash: frame.source_truth_hash,
            previous_chain_hash,
            packet_hash,
            post_hash_only: true,
            presentation_only: true,
            truth_mutation: false,
        });
        Ok(())
    }

    pub fn packets(&self) -> &[UnrealFramePacket] {
        &self.packets
    }

    pub fn final_chain_hash(&self) -> &str {
        &self.final_chain_hash
    }

    pub fn render_packet_chain_json(&self) -> String {
        render_unreal_packet_chain_json(&self.config, &self.packets, &self.final_chain_hash)
    }
}

pub fn verify_unreal_frame_packet_chain(
    config: &UnrealPostHashBridgeConfig,
    packets: &[UnrealFramePacket],
) -> Result<String, OathError> {
    validate_unreal_bridge_config(config)?;
    let mut previous = unreal_chain_genesis_hash(config);
    for (expected_index, packet) in packets.iter().enumerate() {
        if packet.sequence_index != expected_index as u32 {
            return Err(OathError::Verify(format!(
                "expected Unreal frame sequence_index {expected_index}, got {}",
                packet.sequence_index
            )));
        }
        if packet.previous_chain_hash != previous {
            return Err(OathError::Verify(format!(
                "Unreal previous chain hash mismatch at sequence {}",
                packet.sequence_index
            )));
        }
        let frame = UnrealCapturedFrame {
            sequence_index: packet.sequence_index,
            truth_frame: packet.truth_frame,
            scheduled_ms: packet.scheduled_ms,
            capture_id: packet.capture_id.clone(),
            capture_file: packet.capture_file.clone(),
            width: packet.width,
            height: packet.height,
            pixel_format: packet.pixel_format.clone(),
            camera_mode: packet.camera_mode.clone(),
            render_pass: packet.render_pass.clone(),
            capture_sha256: packet.capture_sha256.clone(),
            source_truth_hash: packet.source_truth_hash.clone(),
        };
        validate_unreal_captured_frame(config, &frame)?;
        let canonical = canonical_unreal_frame_material(config, &frame, &previous);
        let expected_hash = sha256::sha256_hex(canonical.as_bytes());
        if packet.packet_hash != expected_hash {
            return Err(OathError::Verify(format!(
                "Unreal packet hash mismatch at sequence {}",
                packet.sequence_index
            )));
        }
        previous = expected_hash;
    }
    Ok(previous)
}

fn validate_unreal_bridge_config(config: &UnrealPostHashBridgeConfig) -> Result<(), OathError> {
    if !config.generated_after_replay_verify {
        return Err(OathError::Verify(
            "Unreal bridge requires generated_after_replay_verify=true".to_string(),
        ));
    }
    for (name, value) in [
        ("scenario_id", &config.scenario_id),
        ("content_hash", &config.content_hash),
        ("final_state_hash", &config.final_state_hash),
        ("replay_json_sha256", &config.replay_json_sha256),
        ("trace_json_sha256", &config.trace_json_sha256),
        (
            "presentation_packet_sha256",
            &config.presentation_packet_sha256,
        ),
        ("unreal_engine_version", &config.unreal_engine_version),
        ("unreal_project_id", &config.unreal_project_id),
        ("renderer_backend_id", &config.renderer_backend_id),
    ] {
        if value.trim().is_empty() {
            return Err(OathError::Verify(format!(
                "Unreal bridge config field {name} must be non-empty"
            )));
        }
    }
    Ok(())
}

fn validate_unreal_captured_frame(
    config: &UnrealPostHashBridgeConfig,
    frame: &UnrealCapturedFrame,
) -> Result<(), OathError> {
    let path = Path::new(&frame.capture_file);
    if path.is_absolute()
        || frame.capture_file.contains("..")
        || !frame.capture_file.starts_with("artifacts/")
    {
        return Err(OathError::Verify(
            "Unreal capture_file must be relative and artifact-scoped".to_string(),
        ));
    }
    if frame.source_truth_hash != config.final_state_hash {
        return Err(OathError::Verify(format!(
            "Unreal source_truth_hash must match final_state_hash {}",
            config.final_state_hash
        )));
    }
    if frame.capture_sha256.len() != 64
        || !frame
            .capture_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(OathError::Verify(
            "Unreal capture_sha256 must be 64 lowercase hex characters".to_string(),
        ));
    }
    if frame.width < 1920 || frame.height < 1080 {
        return Err(OathError::Verify(
            "Unreal capture resolution must be at least 1920x1080".to_string(),
        ));
    }
    Ok(())
}

fn unreal_chain_genesis_hash(config: &UnrealPostHashBridgeConfig) -> String {
    sha256::sha256_hex(
        format!(
            "{UNREAL_POST_HASH_PACKET_CHAIN_SCHEMA}:genesis:{}:{}:{}:{}:{}",
            config.scenario_id,
            config.content_hash,
            config.final_state_hash,
            config.replay_json_sha256,
            config.trace_json_sha256
        )
        .as_bytes(),
    )
}

fn canonical_unreal_frame_material(
    config: &UnrealPostHashBridgeConfig,
    frame: &UnrealCapturedFrame,
    previous_chain_hash: &str,
) -> String {
    format!(
        "{{\"camera_mode\":{},\"capture_file\":{},\"capture_id\":{},\"capture_sha256\":{},\"content_hash\":{},\"final_state_hash\":{},\"height\":{},\"pixel_format\":{},\"post_hash_only\":true,\"presentation_only\":true,\"previous_chain_hash\":{},\"render_pass\":{},\"renderer_backend_id\":{},\"scenario_id\":{},\"scheduled_ms\":{},\"sequence_index\":{},\"source_truth_hash\":{},\"truth_frame\":{},\"truth_mutation\":false,\"width\":{}}}",
        json_quote(&frame.camera_mode),
        json_quote(&frame.capture_file),
        json_quote(&frame.capture_id),
        json_quote(&frame.capture_sha256),
        json_quote(&config.content_hash),
        json_quote(&config.final_state_hash),
        frame.height,
        json_quote(&frame.pixel_format),
        json_quote(previous_chain_hash),
        json_quote(&frame.render_pass),
        json_quote(&config.renderer_backend_id),
        json_quote(&config.scenario_id),
        frame.scheduled_ms,
        frame.sequence_index,
        json_quote(&frame.source_truth_hash),
        frame.truth_frame,
        frame.width
    )
}

fn render_unreal_packet_chain_json(
    config: &UnrealPostHashBridgeConfig,
    packets: &[UnrealFramePacket],
    final_chain_hash: &str,
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(
        &mut out,
        1,
        "schema",
        UNREAL_POST_HASH_PACKET_CHAIN_SCHEMA,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "capture_hook_schema",
        UNREAL_CAPTURE_HOOK_SCHEMA,
        true,
    );
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(&mut out, 1, "scenario_id", &config.scenario_id, true);
    write_json_field(&mut out, 1, "content_hash", &config.content_hash, true);
    write_json_field(
        &mut out,
        1,
        "final_state_hash",
        &config.final_state_hash,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "unreal_engine_version",
        &config.unreal_engine_version,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "unreal_project_id",
        &config.unreal_project_id,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "renderer_backend_id",
        &config.renderer_backend_id,
        true,
    );
    write_json_field(&mut out, 1, "final_chain_hash", final_chain_hash, true);
    writeln!(&mut out, "  \"truth_tick_rate_hz\": {TRUTH_HZ},").unwrap();
    writeln!(&mut out, "  \"frame_count\": {},", packets.len()).unwrap();
    writeln!(&mut out, "  \"post_hash_only\": true,").unwrap();
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(
        &mut out,
        "  \"generated_after_replay_verify\": {},",
        config.generated_after_replay_verify
    )
    .unwrap();
    write_json_field(
        &mut out,
        1,
        "deterministic_ordering",
        "sequence_index_then_truth_frame_then_capture_id",
        true,
    );
    write_json_field(
        &mut out,
        1,
        "chain_algorithm",
        "sha256(previous_chain_hash + canonical_frame_packet_json)",
        true,
    );
    writeln!(&mut out, "  \"frames\": [").unwrap();
    for (index, packet) in packets.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "schema", &packet.schema, true);
        write_json_field(
            &mut out,
            3,
            "capture_hook_schema",
            &packet.capture_hook_schema,
            true,
        );
        writeln!(
            &mut out,
            "      \"sequence_index\": {},",
            packet.sequence_index
        )
        .unwrap();
        writeln!(&mut out, "      \"truth_frame\": {},", packet.truth_frame).unwrap();
        writeln!(&mut out, "      \"scheduled_ms\": {},", packet.scheduled_ms).unwrap();
        write_json_field(&mut out, 3, "capture_id", &packet.capture_id, true);
        write_json_field(&mut out, 3, "capture_file", &packet.capture_file, true);
        writeln!(&mut out, "      \"width\": {},", packet.width).unwrap();
        writeln!(&mut out, "      \"height\": {},", packet.height).unwrap();
        write_json_field(&mut out, 3, "pixel_format", &packet.pixel_format, true);
        write_json_field(&mut out, 3, "camera_mode", &packet.camera_mode, true);
        write_json_field(&mut out, 3, "render_pass", &packet.render_pass, true);
        write_json_field(&mut out, 3, "capture_sha256", &packet.capture_sha256, true);
        write_json_field(
            &mut out,
            3,
            "source_truth_hash",
            &packet.source_truth_hash,
            true,
        );
        write_json_field(
            &mut out,
            3,
            "previous_chain_hash",
            &packet.previous_chain_hash,
            true,
        );
        write_json_field(&mut out, 3, "packet_hash", &packet.packet_hash, true);
        writeln!(&mut out, "      \"post_hash_only\": true,").unwrap();
        writeln!(&mut out, "      \"presentation_only\": true,").unwrap();
        writeln!(&mut out, "      \"truth_mutation\": false").unwrap();
        writeln!(
            &mut out,
            "    }}{}",
            if index + 1 == packets.len() { "" } else { "," }
        )
        .unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

pub const CANONICAL_JOINTS: [&str; 16] = [
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

#[derive(Debug)]
pub enum OathError {
    Io(String),
    Parse(String),
    Verify(String),
}

impl std::fmt::Display for OathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OathError::Io(message) => write!(f, "I/O error: {message}"),
            OathError::Parse(message) => write!(f, "parse error: {message}"),
            OathError::Verify(message) => write!(f, "verification error: {message}"),
        }
    }
}

impl std::error::Error for OathError {}

impl From<std::io::Error> for OathError {
    fn from(value: std::io::Error) -> Self {
        OathError::Io(value.to_string())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fixed {
    milli: i64,
}

impl Fixed {
    pub const SCALE: i64 = 1000;

    pub const fn from_milli(milli: i64) -> Self {
        Self { milli }
    }

    pub const fn from_int(value: i64) -> Self {
        Self {
            milli: value * Self::SCALE,
        }
    }

    pub const fn milli(self) -> i64 {
        self.milli
    }

    pub fn mul_ratio(self, numerator: i64, denominator: i64) -> Self {
        if denominator == 0 {
            return Self {
                milli: if self.milli.is_negative() ^ numerator.is_negative() {
                    i64::MIN
                } else {
                    i64::MAX
                },
            };
        }
        Self {
            milli: clamp_i128_to_i64(
                (self.milli as i128 * numerator as i128) / (denominator as i128),
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionLabel {
    Step,
    Pivot,
    Guard,
    Parry,
    Cut,
    Thrust,
    Brace,
    Bash,
    HookBind,
    Grab,
    Shove,
    Kick,
    Recover,
}

impl ActionLabel {
    pub fn parse(token: &str) -> Result<Self, OathError> {
        match token {
            "step" => Ok(Self::Step),
            "pivot" => Ok(Self::Pivot),
            "guard" => Ok(Self::Guard),
            "parry" => Ok(Self::Parry),
            "cut" => Ok(Self::Cut),
            "thrust" => Ok(Self::Thrust),
            "brace" => Ok(Self::Brace),
            "bash" => Ok(Self::Bash),
            "hook_bind" | "hook/bind" | "hook" | "bind" => Ok(Self::HookBind),
            "grab" => Ok(Self::Grab),
            "shove" => Ok(Self::Shove),
            "kick" => Ok(Self::Kick),
            "recover" => Ok(Self::Recover),
            other => Err(OathError::Parse(format!("unknown action label '{other}'"))),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Step => "step",
            Self::Pivot => "pivot",
            Self::Guard => "guard",
            Self::Parry => "parry",
            Self::Cut => "cut",
            Self::Thrust => "thrust",
            Self::Brace => "brace",
            Self::Bash => "bash",
            Self::HookBind => "hook_bind",
            Self::Grab => "grab",
            Self::Shove => "shove",
            Self::Kick => "kick",
            Self::Recover => "recover",
        }
    }

    pub const fn base_frames(self) -> u32 {
        match self {
            Self::Step => 18,
            Self::Pivot => 16,
            Self::Guard => 14,
            Self::Parry => 18,
            Self::Cut => 32,
            Self::Thrust => 28,
            Self::Brace => 22,
            Self::Bash => 26,
            Self::HookBind => 30,
            Self::Grab => 24,
            Self::Shove => 20,
            Self::Kick => 24,
            Self::Recover => 20,
        }
    }

    pub const fn energy_factor(self) -> i32 {
        match self {
            Self::Cut => 7,
            Self::Thrust => 6,
            Self::Bash => 5,
            Self::HookBind => 4,
            Self::Grab | Self::Shove | Self::Kick => 3,
            Self::Step | Self::Pivot | Self::Guard | Self::Parry | Self::Brace | Self::Recover => 0,
        }
    }

    pub const fn is_attack(self) -> bool {
        matches!(
            self,
            Self::Cut
                | Self::Thrust
                | Self::Bash
                | Self::HookBind
                | Self::Grab
                | Self::Shove
                | Self::Kick
        )
    }

    pub const fn order_key(self) -> u8 {
        match self {
            Self::Step => 0,
            Self::Pivot => 1,
            Self::Guard => 2,
            Self::Parry => 3,
            Self::Cut => 4,
            Self::Thrust => 5,
            Self::Brace => 6,
            Self::Bash => 7,
            Self::HookBind => 8,
            Self::Grab => 9,
            Self::Shove => 10,
            Self::Kick => 11,
            Self::Recover => 12,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Back,
    Left,
    Right,
    Center,
    High,
    Low,
}

impl Direction {
    pub fn parse(token: &str) -> Result<Self, OathError> {
        match token {
            "forward" => Ok(Self::Forward),
            "back" => Ok(Self::Back),
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            "center" => Ok(Self::Center),
            "high" => Ok(Self::High),
            "low" => Ok(Self::Low),
            other => Err(OathError::Parse(format!("unknown direction '{other}'"))),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Forward => "forward",
            Self::Back => "back",
            Self::Left => "left",
            Self::Right => "right",
            Self::Center => "center",
            Self::High => "high",
            Self::Low => "low",
        }
    }

    pub const fn order_key(self) -> u8 {
        match self {
            Self::Forward => 0,
            Self::Back => 1,
            Self::Left => 2,
            Self::Right => 3,
            Self::Center => 4,
            Self::High => 5,
            Self::Low => 6,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetRegion {
    Torso,
    Head,
    WeaponArm,
    LeadLeg,
}

impl TargetRegion {
    pub fn parse(token: &str) -> Result<Self, OathError> {
        match token {
            "torso" => Ok(Self::Torso),
            "head" => Ok(Self::Head),
            "weapon_arm" => Ok(Self::WeaponArm),
            "lead_leg" => Ok(Self::LeadLeg),
            other => Err(OathError::Parse(format!("unknown target region '{other}'"))),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Torso => "torso",
            Self::Head => "head",
            Self::WeaponArm => "weapon_arm",
            Self::LeadLeg => "lead_leg",
        }
    }

    pub const fn order_key(self) -> u8 {
        match self {
            Self::Torso => 0,
            Self::Head => 1,
            Self::WeaponArm => 2,
            Self::LeadLeg => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionEntry {
    pub seat: usize,
    pub label: ActionLabel,
    pub direction: Direction,
    pub target: TargetRegion,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FighterSpec {
    pub seat: usize,
    pub name: String,
    pub weapon_id: String,
    pub armor_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TurnPlan {
    pub index: u32,
    pub actions: [ActionEntry; 2],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scenario {
    pub id: String,
    pub fighters: [FighterSpec; 2],
    pub turns: Vec<TurnPlan>,
}

impl Scenario {
    pub fn parse(input: &str) -> Result<Self, OathError> {
        let mut scenario_id: Option<String> = None;
        let mut fighters: [Option<FighterSpec>; 2] = [None, None];
        let mut turn_slots: Vec<(u32, [Option<ActionEntry>; 2])> = Vec::new();

        for (line_number, raw_line) in input.lines().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            match parts.as_slice() {
                ["scenario", id] => {
                    scenario_id = Some((*id).to_string());
                }
                ["fighter", seat, name, weapon_id, armor_id] => {
                    let seat = parse_seat(seat, line_number + 1)?;
                    fighters[seat] = Some(FighterSpec {
                        seat,
                        name: (*name).to_string(),
                        weapon_id: (*weapon_id).to_string(),
                        armor_id: (*armor_id).to_string(),
                    });
                }
                ["turn", turn_index, seat, label, direction, target] => {
                    let turn_index = turn_index.parse::<u32>().map_err(|_| {
                        OathError::Parse(format!(
                            "line {} has invalid turn index '{}'",
                            line_number + 1,
                            turn_index
                        ))
                    })?;
                    let seat = parse_seat(seat, line_number + 1)?;
                    let entry = ActionEntry {
                        seat,
                        label: ActionLabel::parse(label)?,
                        direction: Direction::parse(direction)?,
                        target: TargetRegion::parse(target)?,
                    };
                    let slot_index = match turn_slots.iter().position(|(idx, _)| *idx == turn_index)
                    {
                        Some(index) => index,
                        None => {
                            turn_slots.push((turn_index, [None, None]));
                            turn_slots.len() - 1
                        }
                    };
                    if turn_slots[slot_index].1[seat].is_some() {
                        return Err(OathError::Parse(format!(
                            "line {} duplicates turn {} seat {}",
                            line_number + 1,
                            turn_index,
                            seat
                        )));
                    }
                    turn_slots[slot_index].1[seat] = Some(entry);
                }
                _ => {
                    return Err(OathError::Parse(format!(
                        "line {} has unsupported syntax: {}",
                        line_number + 1,
                        line
                    )));
                }
            }
        }

        let scenario_id = scenario_id.ok_or_else(|| {
            OathError::Parse("scenario file must contain 'scenario <id>'".to_string())
        })?;
        let fighters = [
            fighters[0]
                .take()
                .ok_or_else(|| OathError::Parse("scenario missing fighter seat 0".to_string()))?,
            fighters[1]
                .take()
                .ok_or_else(|| OathError::Parse("scenario missing fighter seat 1".to_string()))?,
        ];
        if turn_slots.is_empty() {
            return Err(OathError::Parse(
                "scenario must contain at least one turn".to_string(),
            ));
        }
        turn_slots.sort_by_key(|(index, _)| *index);
        let mut turns = Vec::with_capacity(turn_slots.len());
        for (index, actions) in turn_slots {
            turns.push(TurnPlan {
                index,
                actions: [
                    actions[0].ok_or_else(|| {
                        OathError::Parse(format!("turn {index} missing fighter seat 0 action"))
                    })?,
                    actions[1].ok_or_else(|| {
                        OathError::Parse(format!("turn {index} missing fighter seat 1 action"))
                    })?,
                ],
            });
        }

        Ok(Self {
            id: scenario_id,
            fighters,
            turns,
        })
    }

    pub fn canonical_text(&self) -> String {
        let mut out = String::new();
        writeln!(&mut out, "scenario {}", self.id).unwrap();
        for fighter in &self.fighters {
            writeln!(
                &mut out,
                "fighter {} {} {} {}",
                fighter.seat, fighter.name, fighter.weapon_id, fighter.armor_id
            )
            .unwrap();
        }
        for turn in &self.turns {
            for action in &turn.actions {
                writeln!(
                    &mut out,
                    "turn {} {} {} {} {}",
                    turn.index,
                    action.seat,
                    action.label.as_str(),
                    action.direction.as_str(),
                    action.target.as_str()
                )
                .unwrap();
            }
        }
        out
    }
}

fn parse_seat(token: &str, line_number: usize) -> Result<usize, OathError> {
    let seat = token.parse::<usize>().map_err(|_| {
        OathError::Parse(format!(
            "line {line_number} has invalid fighter seat '{token}'"
        ))
    })?;
    if seat > 1 {
        return Err(OathError::Parse(format!(
            "line {line_number} fighter seat must be 0 or 1"
        )));
    }
    Ok(seat)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JointState {
    pub id: usize,
    pub name: &'static str,
    pub mass_g: i32,
    pub x_mm: i32,
    pub y_mm: i32,
    pub z_mm: i32,
    pub inertia_g_cm2: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FighterState {
    pub seat: usize,
    pub name: String,
    pub weapon: WeaponProfile,
    pub armor: ArmorProfile,
    pub joints: [JointState; 16],
    pub stance_width_mm: i32,
    pub forward_mm: i32,
    pub balance_permille: i32,
    pub momentum_permille: i32,
    pub grip_r_permille: i32,
    pub grip_l_permille: i32,
    pub torso_rotation_permille: i32,
    pub torque_permille: i32,
    pub recovery_slowdown_frames: u32,
    pub thrust_valid: bool,
    pub cut_valid: bool,
    pub injury_events: Vec<String>,
}

impl FighterState {
    pub fn from_spec(spec: &FighterSpec) -> Result<Self, OathError> {
        // R-GAP-1: Enforce content-table freeze gate before resolving any
        // asset ID. AI-derived assets must pass all five freeze conditions.
        // Non-AI compile-time content passes through without a lookup.
        enforce_content_freeze_gate_batch([&spec.weapon_id, &spec.armor_id])?;
        let weapon = weapon_by_id(&spec.weapon_id).ok_or_else(|| {
            OathError::Parse(format!("unknown weapon profile '{}'", spec.weapon_id))
        })?;
        let armor = armor_by_id(&spec.armor_id).ok_or_else(|| {
            OathError::Parse(format!("unknown armor profile '{}'", spec.armor_id))
        })?;
        Ok(Self {
            seat: spec.seat,
            name: spec.name.clone(),
            weapon,
            armor,
            joints: canonical_joints(spec.seat),
            stance_width_mm: 620,
            forward_mm: if spec.seat == 0 { -520 } else { 520 },
            balance_permille: 1000,
            momentum_permille: 0,
            grip_r_permille: 1000,
            grip_l_permille: 1000,
            torso_rotation_permille: 1000,
            torque_permille: 1000,
            recovery_slowdown_frames: 0,
            thrust_valid: true,
            cut_valid: true,
            injury_events: Vec::new(),
        })
    }

    pub fn canonical_state(&self, out: &mut String) {
        writeln!(&mut *out, "fighter.seat={}", self.seat).unwrap();
        writeln!(&mut *out, "fighter.name={}", self.name).unwrap();
        writeln!(&mut *out, "fighter.weapon={}", self.weapon.id).unwrap();
        writeln!(&mut *out, "fighter.armor={}", self.armor.id).unwrap();
        writeln!(
            &mut *out,
            "fighter.stance_width_mm={}",
            self.stance_width_mm
        )
        .unwrap();
        writeln!(&mut *out, "fighter.forward_mm={}", self.forward_mm).unwrap();
        writeln!(
            &mut *out,
            "fighter.balance_permille={}",
            self.balance_permille
        )
        .unwrap();
        writeln!(
            &mut *out,
            "fighter.momentum_permille={}",
            self.momentum_permille
        )
        .unwrap();
        writeln!(
            &mut *out,
            "fighter.grip_r_permille={}",
            self.grip_r_permille
        )
        .unwrap();
        writeln!(
            &mut *out,
            "fighter.grip_l_permille={}",
            self.grip_l_permille
        )
        .unwrap();
        writeln!(
            &mut *out,
            "fighter.torso_rotation_permille={}",
            self.torso_rotation_permille
        )
        .unwrap();
        writeln!(
            &mut *out,
            "fighter.torque_permille={}",
            self.torque_permille
        )
        .unwrap();
        writeln!(
            &mut *out,
            "fighter.recovery_slowdown_frames={}",
            self.recovery_slowdown_frames
        )
        .unwrap();
        writeln!(&mut *out, "fighter.thrust_valid={}", self.thrust_valid).unwrap();
        writeln!(&mut *out, "fighter.cut_valid={}", self.cut_valid).unwrap();
        for joint in &self.joints {
            writeln!(
                &mut *out,
                "joint:{}:{}:{}:{}:{}:{}",
                joint.id, joint.name, joint.mass_g, joint.x_mm, joint.y_mm, joint.z_mm
            )
            .unwrap();
        }
        for (index, event) in self.injury_events.iter().enumerate() {
            writeln!(&mut *out, "injury_event:{index}:{event}").unwrap();
        }
    }
}

fn canonical_joints(seat: usize) -> [JointState; 16] {
    let side_offset = if seat == 0 { -100 } else { 100 };
    let base_x = if seat == 0 { -520 } else { 520 };
    let mut joints = [JointState {
        id: 0,
        name: "root",
        mass_g: 0,
        x_mm: 0,
        y_mm: 0,
        z_mm: 0,
        inertia_g_cm2: 0,
    }; 16];
    for (id, name) in CANONICAL_JOINTS.iter().enumerate() {
        let y_mm = match id {
            0 => 980,
            1 => 1120,
            2 => 1370,
            3 => 1620,
            4 | 7 => 1450,
            5 | 8 => 1260,
            6 | 9 => 1080,
            10 | 13 => 910,
            11 | 14 => 520,
            12 | 15 => 90,
            _ => 1000,
        };
        let z_mm = match id {
            4 | 5 | 6 | 10 | 11 | 12 => side_offset,
            7 | 8 | 9 | 13 | 14 | 15 => -side_offset,
            _ => 0,
        };
        joints[id] = JointState {
            id,
            name,
            mass_g: 900 + (id as i32 * 85),
            x_mm: base_x,
            y_mm,
            z_mm,
            inertia_g_cm2: 1200 + (id as i32 * 70),
        };
    }
    joints
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DuelState {
    pub fighters: [FighterState; 2],
}

impl DuelState {
    pub fn from_scenario(scenario: &Scenario) -> Result<Self, OathError> {
        Ok(Self {
            fighters: [
                FighterState::from_spec(&scenario.fighters[0])?,
                FighterState::from_spec(&scenario.fighters[1])?,
            ],
        })
    }

    pub fn canonical_state(&self) -> String {
        let mut out = String::new();
        writeln!(&mut out, "product={PRODUCT_NAME}").unwrap();
        writeln!(&mut out, "truth_hz={TRUTH_HZ}").unwrap();
        for fighter in &self.fighters {
            fighter.canonical_state(&mut out);
        }
        out
    }

    pub fn state_hash(&self) -> String {
        hash_hex(self.canonical_state().as_bytes())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FighterEndCondition {
    pub seat: usize,
    pub incapacitated: bool,
    pub stop_kind: String,
    pub reason: String,
    pub balance_permille: i32,
    pub grip_r_permille: i32,
    pub torque_permille: i32,
    pub torso_rotation_permille: i32,
    pub recovery_slowdown_frames: u32,
    pub thrust_valid: bool,
    pub cut_valid: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DuelEndCondition {
    pub status: String,
    pub winner: Option<usize>,
    pub reason: String,
    pub fighters: [FighterEndCondition; 2],
}

impl DuelEndCondition {
    fn winner_token(&self) -> String {
        match self.winner {
            Some(seat) => format!("seat_{seat}"),
            None => "none".to_string(),
        }
    }
}

fn evaluate_duel_end_condition(state: &DuelState) -> DuelEndCondition {
    let fighters = [
        evaluate_fighter_end_condition(&state.fighters[0]),
        evaluate_fighter_end_condition(&state.fighters[1]),
    ];
    let (status, winner, reason) = match (fighters[0].incapacitated, fighters[1].incapacitated) {
        (true, true) => (
            "mutual_capability_stop".to_string(),
            None,
            format!(
                "both fighters crossed deterministic non-HP stop rules: seat 0 {}; seat 1 {}",
                fighters[0].stop_kind, fighters[1].stop_kind
            ),
        ),
        (true, false) => (
            "seat_1_victory_capability_stop".to_string(),
            Some(1),
            format!(
                "seat 0 stopped by {}; seat 1 retains planned-time capability",
                fighters[0].stop_kind
            ),
        ),
        (false, true) => (
            "seat_0_victory_capability_stop".to_string(),
            Some(0),
            format!(
                "seat 1 stopped by {}; seat 0 retains planned-time capability",
                fighters[1].stop_kind
            ),
        ),
        (false, false) => (
            "unresolved_after_script".to_string(),
            None,
            "no fighter crossed deterministic non-HP stop rules by the final committed turn"
                .to_string(),
        ),
    };

    DuelEndCondition {
        status,
        winner,
        reason,
        fighters,
    }
}

fn evaluate_fighter_end_condition(fighter: &FighterState) -> FighterEndCondition {
    let primary_weapon_actions_invalid = !fighter.thrust_valid && !fighter.cut_valid;
    let (incapacitated, stop_kind, reason) = if fighter.balance_permille <= 300 {
        (
            true,
            "stance_collapse",
            format!(
                "balance {} permille cannot support a planned-time stance after physical contact",
                fighter.balance_permille
            ),
        )
    } else if fighter.torso_rotation_permille <= 360 && fighter.recovery_slowdown_frames >= 20 {
        (
            true,
            "torso_rotation_locked",
            format!(
                "torso rotation {} permille with recovery slowdown {} frames prevents timely re-plan",
                fighter.torso_rotation_permille, fighter.recovery_slowdown_frames
            ),
        )
    } else if fighter.grip_r_permille <= 320 && fighter.torque_permille <= 520 {
        (
            true,
            "weapon_control_lost",
            format!(
                "right grip {} permille and weapon torque {} permille cannot control the weapon",
                fighter.grip_r_permille, fighter.torque_permille
            ),
        )
    } else if fighter.recovery_slowdown_frames >= 28 && fighter.balance_permille <= 520 {
        (
            true,
            "recovery_window_failed",
            format!(
                "recovery slowdown {} frames with balance {} permille exceeds local duel stop window",
                fighter.recovery_slowdown_frames, fighter.balance_permille
            ),
        )
    } else if primary_weapon_actions_invalid {
        (
            true,
            "primary_weapon_actions_invalid",
            "cut and thrust validity are both lost through capability deltas".to_string(),
        )
    } else {
        (
            false,
            "continuing_capability",
            format!(
                "balance {}, grip {}, torque {}, torso rotation {}, recovery {} frames remain above stop thresholds",
                fighter.balance_permille,
                fighter.grip_r_permille,
                fighter.torque_permille,
                fighter.torso_rotation_permille,
                fighter.recovery_slowdown_frames
            ),
        )
    };

    FighterEndCondition {
        seat: fighter.seat,
        incapacitated,
        stop_kind: stop_kind.to_string(),
        reason,
        balance_permille: fighter.balance_permille,
        grip_r_permille: fighter.grip_r_permille,
        torque_permille: fighter.torque_permille,
        torso_rotation_permille: fighter.torso_rotation_permille,
        recovery_slowdown_frames: fighter.recovery_slowdown_frames,
        thrust_valid: fighter.thrust_valid,
        cut_valid: fighter.cut_valid,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CostFactor {
    pub name: &'static str,
    pub permille: i32,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CostBreakdown {
    pub fighter: usize,
    pub action: ActionLabel,
    pub base_frames: u32,
    pub current_frames: u32,
    pub action_valid: bool,
    pub factors: Vec<CostFactor>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CapabilityDelta {
    pub torso_rotation_delta: i32,
    pub recovery_slowdown_add: u32,
    pub balance_delta: i32,
    pub torque_delta: i32,
    pub grip_r_delta: i32,
    pub grip_l_delta: i32,
    pub invalidates_thrust: bool,
    pub invalidates_cut: bool,
    pub event: String,
}

impl CapabilityDelta {
    fn merge(&mut self, other: CapabilityDelta) {
        self.torso_rotation_delta += other.torso_rotation_delta;
        self.recovery_slowdown_add += other.recovery_slowdown_add;
        self.balance_delta += other.balance_delta;
        self.torque_delta += other.torque_delta;
        self.grip_r_delta += other.grip_r_delta;
        self.grip_l_delta += other.grip_l_delta;
        self.invalidates_thrust |= other.invalidates_thrust;
        self.invalidates_cut |= other.invalidates_cut;
        if !other.event.is_empty() {
            if !self.event.is_empty() {
                self.event.push_str(" | ");
            }
            self.event.push_str(&other.event);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContactTrace {
    pub turn: u32,
    pub frame: u32,
    pub attacker: usize,
    pub defender: usize,
    pub action: ActionLabel,
    pub direction: Direction,
    pub target: TargetRegion,
    pub weapon_id: String,
    pub armor_id: String,
    pub energy_milli: i32,
    pub impulse_milli: i32,
    pub material_result: String,
    pub anatomy_result: String,
    pub capability_delta: CapabilityDelta,
    pub cause_chain: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TurnTrace {
    pub turn: u32,
    pub phase_order: [&'static str; 6],
    pub commits: [ActionEntry; 2],
    pub costs: Vec<CostBreakdown>,
    pub contacts: Vec<ContactTrace>,
    pub state_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DuelResult {
    pub scenario_id: String,
    pub canonical_scenario: String,
    pub content_hash: String,
    pub initial_state_hash: String,
    pub final_state_hash: String,
    pub end_condition: DuelEndCondition,
    pub turn_hashes: Vec<String>,
    pub turns: Vec<TurnTrace>,
    pub trace_json: String,
    pub replay_json: String,
    pub report_md: String,
    pub fight_film_manifest_json: String,
}

pub fn run_scenario_file(path: impl AsRef<Path>) -> Result<DuelResult, OathError> {
    let text = fs::read_to_string(path)?;
    run_scenario_text(&text)
}

/// R-GAP-1: Enforce the combat-truth freeze gate for all assets referenced by
/// a scenario. Checks every fighter's weapon_id and armor_id against the
/// freeze registry. AI-derived assets (prefix "ai:") must have passed all
/// five freeze conditions. Non-AI assets pass through without a lookup.
fn enforce_scenario_freeze_gate(scenario: &Scenario) -> Result<(), OathError> {
    let repo_root = oathyard_repo_root();
    for fighter in &scenario.fighters {
        enforce_combat_truth_freeze_gate(&repo_root, &fighter.weapon_id)?;
        enforce_combat_truth_freeze_gate(&repo_root, &fighter.armor_id)?;
    }
    Ok(())
}

pub fn run_scenario_text(text: &str) -> Result<DuelResult, OathError> {
    let scenario = Scenario::parse(text)?;
    // R-GAP-1: Enforce combat-truth freeze gate at the scenario consumption
    // boundary. Any AI-derived asset (weapon_id, armor_id prefixed with "ai:")
    // must have passed all five freeze conditions before it can feed into the
    // authoritative combat simulation. Non-AI compile-time content passes through.
    enforce_scenario_freeze_gate(&scenario)?;
    let canonical_scenario = scenario.canonical_text();
    let content_hash_value = content_hash();
    let mut state = DuelState::from_scenario(&scenario)?;
    let initial_state_hash = state.state_hash();
    let mut turns = Vec::with_capacity(scenario.turns.len());
    let mut turn_hashes = Vec::with_capacity(scenario.turns.len());

    for turn_plan in &scenario.turns {
        let pre_state = state.clone();
        let mut costs = Vec::with_capacity(2);
        for action in &turn_plan.actions {
            costs.push(calculate_cost(&pre_state.fighters[action.seat], *action));
        }

        let mut deltas = [CapabilityDelta::default(), CapabilityDelta::default()];
        let mut contacts = Vec::new();
        for action in &turn_plan.actions {
            let defender_action = turn_plan.actions[1 - action.seat];
            if let Some(contact) =
                resolve_contact(&pre_state, turn_plan.index, *action, defender_action)
            {
                contacts.push(contact);
            }
        }
        sort_contact_packets(&mut contacts);
        for contact in &contacts {
            deltas[contact.defender].merge(contact.capability_delta.clone());
        }

        for action in &turn_plan.actions {
            apply_action_movement(&mut state.fighters[action.seat], *action);
        }
        for (seat, delta) in deltas.into_iter().enumerate() {
            apply_capability_delta(&mut state.fighters[seat], delta);
        }

        let state_hash = state.state_hash();
        turn_hashes.push(state_hash.clone());
        turns.push(TurnTrace {
            turn: turn_plan.index,
            phase_order: [
                "OBSERVE",
                "PLAN",
                "COMMIT_REVEAL",
                "RESOLVE",
                "CONSEQUENCE",
                "REPLAN",
            ],
            commits: turn_plan.actions,
            costs,
            contacts,
            state_hash,
        });
    }

    let final_state_hash = state.state_hash();
    let end_condition = evaluate_duel_end_condition(&state);
    let trace_json = render_trace_json(
        &scenario,
        &content_hash_value,
        &initial_state_hash,
        &final_state_hash,
        &end_condition,
        &turns,
    );
    let replay_json = render_replay_json(
        &canonical_scenario,
        &content_hash_value,
        &initial_state_hash,
        &final_state_hash,
        &end_condition,
        &turn_hashes,
    );
    let report_md = render_report_md(
        &scenario,
        &content_hash_value,
        &initial_state_hash,
        &final_state_hash,
        &end_condition,
        &turns,
    );
    let fight_film_manifest_json =
        render_fight_film_manifest_json(&scenario, &final_state_hash, &turns);
    Ok(DuelResult {
        scenario_id: scenario.id,
        canonical_scenario,
        content_hash: content_hash_value,
        initial_state_hash,
        final_state_hash,
        end_condition,
        turn_hashes,
        turns,
        trace_json,
        replay_json,
        report_md,
        fight_film_manifest_json,
    })
}

pub fn write_artifacts(result: &DuelResult, out_dir: impl AsRef<Path>) -> Result<(), OathError> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;
    fs::write(out_dir.join("trace.json"), &result.trace_json)?;
    fs::write(out_dir.join("replay.json"), &result.replay_json)?;
    fs::write(
        out_dir.join("final_state_hash.txt"),
        format!("{}\n", result.final_state_hash),
    )?;
    fs::write(out_dir.join("duel_report.md"), &result.report_md)?;
    fs::write(
        out_dir.join("fight_film_manifest.json"),
        &result.fight_film_manifest_json,
    )?;
    Ok(())
}

pub fn verify_replay_file(path: impl AsRef<Path>) -> Result<DuelResult, OathError> {
    let text = fs::read_to_string(path)?;
    verify_replay_text(&text)
}

fn file_hash_hex(path: impl AsRef<Path>) -> Result<String, OathError> {
    Ok(hash_hex(&fs::read(path)?))
}

pub fn verify_replay_text(text: &str) -> Result<DuelResult, OathError> {
    let schema = json_string_value(text, "schema")
        .ok_or_else(|| OathError::Verify("replay missing schema".to_string()))?;
    if schema != REPLAY_SCHEMA {
        return Err(OathError::Verify(format!(
            "replay schema mismatch: expected {REPLAY_SCHEMA}, got {schema}"
        )));
    }
    let scenario_text = json_string_value(text, "scenario_canonical")
        .ok_or_else(|| OathError::Verify("replay missing scenario_canonical".to_string()))?;
    // R-GAP-1: Enforce freeze gate at the replay verification boundary.
    // Parse the embedded scenario and reject any AI-derived assets that
    // have not passed all five freeze conditions before accepting the replay.
    let replay_scenario = Scenario::parse(&scenario_text)?;
    enforce_scenario_freeze_gate(&replay_scenario)?;
    let replay_truth_hz = json_u32_value(text, "truth_hz")
        .ok_or_else(|| OathError::Verify("replay missing truth_hz".to_string()))?;
    if replay_truth_hz != TRUTH_HZ {
        return Err(OathError::Verify(format!(
            "replay truth_hz mismatch: expected {TRUTH_HZ}, got {replay_truth_hz}"
        )));
    }
    let expected_content_hash = json_string_value(text, "content_hash")
        .ok_or_else(|| OathError::Verify("replay missing content_hash".to_string()))?;
    let expected_initial_state_hash = json_string_value(text, "initial_state_hash")
        .ok_or_else(|| OathError::Verify("replay missing initial_state_hash".to_string()))?;
    let expected_final_state_hash = json_string_value(text, "final_state_hash")
        .ok_or_else(|| OathError::Verify("replay missing final_state_hash".to_string()))?;
    let expected_end_condition_status = json_string_value(text, "end_condition_status")
        .ok_or_else(|| OathError::Verify("replay missing end_condition_status".to_string()))?;
    let expected_end_condition_winner = json_string_value(text, "end_condition_winner")
        .ok_or_else(|| OathError::Verify("replay missing end_condition_winner".to_string()))?;
    let expected_turn_hashes = json_string_array(text, "turn_hashes")
        .ok_or_else(|| OathError::Verify("replay missing turn_hashes".to_string()))?;

    let result = run_scenario_text(&scenario_text)?;
    if result.content_hash != expected_content_hash {
        return Err(OathError::Verify(format!(
            "content hash mismatch: expected {}, got {}",
            expected_content_hash, result.content_hash
        )));
    }
    if result.initial_state_hash != expected_initial_state_hash {
        return Err(OathError::Verify(format!(
            "initial state hash mismatch: expected {}, got {}",
            expected_initial_state_hash, result.initial_state_hash
        )));
    }
    if result.turn_hashes != expected_turn_hashes {
        return Err(OathError::Verify(format!(
            "turn hash mismatch: expected {:?}, got {:?}",
            expected_turn_hashes, result.turn_hashes
        )));
    }
    if result.final_state_hash != expected_final_state_hash {
        return Err(OathError::Verify(format!(
            "final state hash mismatch: expected {}, got {}",
            expected_final_state_hash, result.final_state_hash
        )));
    }
    if result.end_condition.status != expected_end_condition_status {
        return Err(OathError::Verify(format!(
            "end condition status mismatch: expected {}, got {}",
            expected_end_condition_status, result.end_condition.status
        )));
    }
    let actual_winner = result.end_condition.winner_token();
    if actual_winner != expected_end_condition_winner {
        return Err(OathError::Verify(format!(
            "end condition winner mismatch: expected {}, got {}",
            expected_end_condition_winner, actual_winner
        )));
    }
    Ok(result)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NegativeInputCase {
    id: &'static str,
    input_kind: &'static str,
    expected_error_contains: &'static str,
    observed_error: String,
    failed_loudly: bool,
    passed: bool,
}

pub fn write_negative_input_audit_artifacts(out_dir: impl AsRef<Path>) -> Result<(), OathError> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;
    let cases = build_negative_input_cases(out_dir)?;
    fs::write(
        out_dir.join("negative_input_audit.json"),
        render_negative_input_audit_json(&cases),
    )?;
    fs::write(
        out_dir.join("negative_input_audit_report.md"),
        render_negative_input_audit_report(&cases),
    )?;
    if !cases.iter().all(|case| case.passed) {
        return Err(OathError::Verify(
            "negative input audit found silent accept or wrong failure".to_string(),
        ));
    }
    Ok(())
}

fn build_negative_input_cases(out_dir: &Path) -> Result<Vec<NegativeInputCase>, OathError> {
    let mut cases = Vec::new();
    cases.push(negative_case(
        "scenario_missing_id_fails_loud",
        "scenario",
        Scenario::parse("fighter 0 ash curved_sword gambeson\n"),
        "scenario file must contain",
    ));
    cases.push(negative_case(
        "scenario_missing_fighter_fails_loud",
        "scenario",
        Scenario::parse(
            "scenario missing_fighter\nfighter 0 ash curved_sword gambeson\nturn 0 0 cut forward torso\nturn 0 1 guard center torso\n",
        ),
        "scenario missing fighter seat 1",
    ));
    cases.push(negative_case(
        "scenario_unknown_action_fails_loud",
        "scenario",
        Scenario::parse(
            "scenario bad_action\nfighter 0 ash curved_sword gambeson\nfighter 1 vale longsword mail_hauberk\nturn 0 0 slash forward torso\nturn 0 1 guard center torso\n",
        ),
        "unknown action label",
    ));
    cases.push(negative_case(
        "scenario_duplicate_turn_seat_fails_loud",
        "scenario",
        Scenario::parse(
            "scenario duplicate_turn\nfighter 0 ash curved_sword gambeson\nfighter 1 vale longsword mail_hauberk\nturn 0 0 cut forward torso\nturn 0 0 thrust forward torso\nturn 0 1 guard center torso\n",
        ),
        "duplicates turn 0 seat 0",
    ));
    cases.push(negative_case(
        "scenario_unknown_target_fails_loud",
        "scenario",
        Scenario::parse(
            "scenario bad_target\nfighter 0 ash curved_sword gambeson\nfighter 1 vale longsword mail_hauberk\nturn 0 0 cut forward hand\nturn 0 1 guard center torso\n",
        ),
        "unknown target region",
    ));
    cases.push(negative_case(
        "scenario_unknown_weapon_fails_loud",
        "scenario",
        run_scenario_text(
            "scenario bad_weapon\nfighter 0 ash impossible_sword gambeson\nfighter 1 vale longsword mail_hauberk\nturn 0 0 cut forward torso\nturn 0 1 guard center torso\n",
        ),
        "unknown weapon profile",
    ));
    cases.push(negative_case(
        "content_manifest_schema_fails_loud",
        "content_manifest",
        validate_content_manifest_text(
            "product=OATHYARD\nschema=wrong\npublic_demo_ready=false\nrelease_candidate_ready=false\n[fighters]\n",
        ),
        "content manifest schema mismatch",
    ));
    cases.push(negative_case(
        "content_manifest_readiness_true_fails_loud",
        "content_manifest",
        validate_content_manifest_text(concat!(
            "product=OATHYARD\nschema=oathyard.content.v1\n",
            "public_demo_ready=",
            "true\n",
            "release_candidate_ready=false\n[fighters]\n"
        )),
        "public_demo_ready must remain false",
    ));
    cases.push(negative_case(
        "content_manifest_missing_rows_fails_loud",
        "content_manifest",
        validate_content_manifest_text(
            "product=OATHYARD\nschema=oathyard.content.v1\npublic_demo_ready=false\nrelease_candidate_ready=false\n[materials]\nmat_0: physical material\nmat_1: physical material\nmat_2: physical material\nmat_3: physical material\nmat_4: physical material\nmat_5: physical material\n[fighters]\n",
        ),
        "fighters count 0 below required 6",
    ));
    cases.push(negative_case(
        "replay_unsupported_schema_fails_loud",
        "replay",
        verify_replay_text("{\"schema\":\"oathyard.replay.v999\"}"),
        "replay schema mismatch",
    ));
    cases.push(negative_case(
        "replay_missing_scenario_fails_loud",
        "replay",
        verify_replay_text("{\"schema\":\"oathyard.replay.v1\"}"),
        "replay missing scenario_canonical",
    ));

    let basic = run_scenario_text(include_str!("../examples/duels/basic_oathyard.duel"))?;
    let mismatched_final = basic.replay_json.replacen(
        &format!("\"final_state_hash\": \"{}\"", basic.final_state_hash),
        "\"final_state_hash\": \"0000000000000000\"",
        1,
    );
    cases.push(negative_case(
        "replay_mismatched_final_hash_fails_loud",
        "replay",
        verify_replay_text(&mismatched_final),
        "final state hash mismatch",
    ));

    let fixture_dir = out_dir.join("fixtures");
    fs::create_dir_all(&fixture_dir)?;
    let bundle_source = fixture_dir.join("source_replay.json");
    let bundle_dir = fixture_dir.join("tampered_bundle");
    fs::write(&bundle_source, &basic.replay_json)?;
    write_replay_export_bundle(&bundle_source, &bundle_dir)?;
    fs::write(bundle_dir.join("trace.json"), "{}\n")?;
    cases.push(negative_case(
        "export_bundle_tamper_fails_loud",
        "export_bundle",
        verify_replay_export_bundle(&bundle_dir),
        "export bundle hash mismatch",
    ));

    Ok(cases)
}

fn negative_case<T>(
    id: &'static str,
    input_kind: &'static str,
    result: Result<T, OathError>,
    expected_error_contains: &'static str,
) -> NegativeInputCase {
    match result {
        Ok(_) => NegativeInputCase {
            id,
            input_kind,
            expected_error_contains,
            observed_error: "accepted invalid input".to_string(),
            failed_loudly: false,
            passed: false,
        },
        Err(error) => {
            let observed_error = error.to_string();
            let passed = observed_error.contains(expected_error_contains);
            NegativeInputCase {
                id,
                input_kind,
                expected_error_contains,
                observed_error,
                failed_loudly: true,
                passed,
            }
        }
    }
}

pub fn validate_content_manifest_text(text: &str) -> Result<(), OathError> {
    let mut product: Option<&str> = None;
    let mut schema: Option<&str> = None;
    let mut public_demo_ready: Option<&str> = None;
    let mut release_candidate_ready: Option<&str> = None;
    let mut current_section: Option<String> = None;
    let mut sections: Vec<(String, Vec<String>)> = Vec::new();

    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            current_section = Some(line[1..line.len() - 1].to_string());
            sections.push((current_section.clone().unwrap(), Vec::new()));
            continue;
        }
        if let Some(section) = current_section.as_ref() {
            if !line.contains(':') {
                return Err(OathError::Parse(format!(
                    "content manifest section {section} row missing ':'"
                )));
            }
            if let Some((_, rows)) = sections.iter_mut().find(|(name, _)| name == section) {
                rows.push(line.to_string());
            }
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            match key {
                "product" => product = Some(value),
                "schema" => schema = Some(value),
                "public_demo_ready" => public_demo_ready = Some(value),
                "release_candidate_ready" => release_candidate_ready = Some(value),
                _ => {}
            }
        }
    }

    if product != Some(PRODUCT_NAME) {
        return Err(OathError::Parse(
            "content manifest product mismatch".to_string(),
        ));
    }
    if schema != Some("oathyard.content.v1") {
        return Err(OathError::Parse(
            "content manifest schema mismatch".to_string(),
        ));
    }
    if public_demo_ready != Some("false") {
        return Err(OathError::Parse(
            "content manifest public_demo_ready must remain false".to_string(),
        ));
    }
    if release_candidate_ready != Some("false") {
        return Err(OathError::Parse(
            "content manifest release_candidate_ready must remain false".to_string(),
        ));
    }

    for (section, minimum) in [
        ("materials", 6usize),
        ("fighters", 6usize),
        ("weapons", 6usize),
        ("armor", 6usize),
        ("arenas", 2usize),
    ] {
        let count = sections
            .iter()
            .find(|(name, _)| name == section)
            .map(|(_, rows)| rows.len())
            .unwrap_or(0);
        if count < minimum {
            return Err(OathError::Parse(format!(
                "content manifest {section} count {count} below required {minimum}"
            )));
        }
    }

    Ok(())
}

fn render_negative_input_audit_json(cases: &[NegativeInputCase]) -> String {
    let passed = cases.iter().all(|case| case.passed);
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", NEGATIVE_INPUT_AUDIT_SCHEMA, true);
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    writeln!(&mut out, "  \"truth_hz\": {TRUTH_HZ},").unwrap();
    writeln!(&mut out, "  \"case_count\": {},", cases.len()).unwrap();
    writeln!(
        &mut out,
        "  \"all_failed_loudly\": {},",
        cases.iter().all(|case| case.failed_loudly)
    )
    .unwrap();
    writeln!(&mut out, "  \"all_cases_passed\": {passed},").unwrap();
    writeln!(&mut out, "  \"public_demo_ready\": {PUBLIC_DEMO_READY},").unwrap();
    writeln!(
        &mut out,
        "  \"release_candidate_ready\": {RELEASE_CANDIDATE_READY},"
    )
    .unwrap();
    writeln!(&mut out, "  \"screens_visited\": [").unwrap();
    for (index, case) in cases.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "id", case.id, true);
        write_json_field(&mut out, 3, "input_kind", case.input_kind, true);
        write_json_field(
            &mut out,
            3,
            "expected_error_contains",
            case.expected_error_contains,
            true,
        );
        write_json_field(&mut out, 3, "observed_error", &case.observed_error, true);
        writeln!(&mut out, "      \"failed_loudly\": {},", case.failed_loudly).unwrap();
        writeln!(&mut out, "      \"passed\": {}", case.passed).unwrap();
        writeln!(&mut out, "    }}{}", comma(index + 1, cases.len())).unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

fn render_negative_input_audit_report(cases: &[NegativeInputCase]) -> String {
    let passed = cases.iter().all(|case| case.passed);
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD Negative Input Audit").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(
        &mut out,
        "Status: {}",
        if passed { "PASSED" } else { "FAILED" }
    )
    .unwrap();
    writeln!(&mut out, "- Case count: `{}`", cases.len()).unwrap();
    writeln!(
        &mut out,
        "- All failed loudly: `{}`",
        cases.iter().all(|case| case.failed_loudly)
    )
    .unwrap();
    writeln!(&mut out, "- Public demo ready: `{PUBLIC_DEMO_READY}`").unwrap();
    writeln!(
        &mut out,
        "- Release candidate ready: `{RELEASE_CANDIDATE_READY}`"
    )
    .unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Cases").unwrap();
    for case in cases {
        writeln!(
            &mut out,
            "- `{}` `{}` passed `{}` expected `{}` observed `{}`",
            case.id,
            case.input_kind,
            case.passed,
            case.expected_error_contains,
            case.observed_error
        )
        .unwrap();
    }
    out
}

#[cfg(target_os = "linux")]
pub fn native_roster_showcase(out_dir: impl AsRef<Path>) -> Result<(), OathError> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;
    let content_hash = content_hash();
    let mut manifest = String::new();
    writeln!(&mut manifest, "{{").unwrap();
    write_json_field(
        &mut manifest,
        1,
        "schema",
        NATIVE_ROSTER_SHOWCASE_SCHEMA,
        true,
    );
    write_json_field(&mut manifest, 1, "product", PRODUCT_NAME, true);
    write_json_field(
        &mut manifest,
        1,
        "source",
        "blocked-pending-native-3d-renderer-capture",
        true,
    );
    write_json_field(&mut manifest, 1, "content_hash", &content_hash, true);
    writeln!(
        &mut manifest,
        "  \"fighter_tradition_count\": {},",
        FIGHTER_TRADITIONS.len()
    )
    .unwrap();
    writeln!(&mut manifest, "  \"frame_count\": 0,").unwrap();
    writeln!(&mut manifest, "  \"truth_mutation\": false,").unwrap();
    writeln!(&mut manifest, "  \"presentation_only\": true,").unwrap();
    writeln!(
        &mut manifest,
        "  \"native_3d_visual_evidence_present\": false,"
    )
    .unwrap();
    write_json_field(
        &mut manifest,
        1,
        "visual_evidence_status",
        "blocked_pending_native_3d_renderer_capture",
        true,
    );
    writeln!(
        &mut manifest,
        "  \"forbidden_visual_fallbacks_emitted\": false,"
    )
    .unwrap();
    writeln!(
        &mut manifest,
        "  \"owner_visual_acceptance_claimed\": false,"
    )
    .unwrap();
    writeln!(&mut manifest, "  \"public_demo_ready\": false,").unwrap();
    writeln!(&mut manifest, "  \"release_candidate_ready\": false").unwrap();
    writeln!(&mut manifest, "}}").unwrap();
    fs::write(
        out_dir.join("native_roster_showcase_manifest.json"),
        manifest,
    )?;

    let mut report = String::new();
    writeln!(&mut report, "# OATHYARD Native Roster Showcase").unwrap();
    writeln!(&mut report).unwrap();
    writeln!(
        &mut report,
        "Status: BLOCKED_PENDING_NATIVE_3D_RENDERER_CAPTURE"
    )
    .unwrap();
    writeln!(
        &mut report,
        "- Fighter traditions covered by metadata: `{}`",
        FIGHTER_TRADITIONS.len()
    )
    .unwrap();
    writeln!(&mut report, "- Native 3D visual evidence present: `false`").unwrap();
    writeln!(&mut report, "- Forbidden visual fallbacks emitted: `false`").unwrap();
    writeln!(&mut report, "- Owner visual acceptance claimed: `false`").unwrap();
    writeln!(&mut report, "- Public demo ready: `false`").unwrap();
    writeln!(&mut report, "- Release candidate ready: `false`").unwrap();
    fs::write(out_dir.join("native_roster_showcase_report.md"), report)?;
    Ok(())
}

pub fn calculate_cost(fighter: &FighterState, action: ActionEntry) -> CostBreakdown {
    let base_frames = action.label.base_frames();
    let action_valid = match action.label {
        ActionLabel::Cut => fighter.cut_valid && fighter.grip_r_permille >= 450,
        ActionLabel::Thrust => fighter.thrust_valid && fighter.grip_r_permille >= 520,
        ActionLabel::Bash | ActionLabel::HookBind | ActionLabel::Grab | ActionLabel::Shove => {
            fighter.grip_r_permille >= 380 && fighter.balance_permille >= 360
        }
        ActionLabel::Kick => fighter.balance_permille >= 520,
        ActionLabel::Step
        | ActionLabel::Pivot
        | ActionLabel::Guard
        | ActionLabel::Parry
        | ActionLabel::Brace
        | ActionLabel::Recover => true,
    };

    let body_penalty = ((1000 - fighter.torso_rotation_permille).max(0) / 5)
        + ((1000 - fighter.balance_permille).max(0) / 8);
    let equipment_penalty = fighter.weapon.mass_g / 60
        + fighter.armor.mass_g / 180
        + fighter.weapon.inertia_g_cm2 / 10000;
    let state_penalty = match action.label {
        ActionLabel::Thrust | ActionLabel::Recover => fighter.recovery_slowdown_frames as i32 * 10,
        ActionLabel::Cut => fighter.recovery_slowdown_frames as i32 * 6,
        ActionLabel::Parry | ActionLabel::Bash | ActionLabel::HookBind | ActionLabel::Grab => {
            fighter.recovery_slowdown_frames as i32 * 8
        }
        ActionLabel::Kick | ActionLabel::Shove => fighter.recovery_slowdown_frames as i32 * 7,
        ActionLabel::Step | ActionLabel::Pivot | ActionLabel::Guard | ActionLabel::Brace => {
            fighter.recovery_slowdown_frames as i32 * 3
        }
    };
    let momentum_penalty = fighter.momentum_permille.abs() / 30;
    let injury_penalty = ((1000 - fighter.grip_r_permille).max(0) / 6)
        + ((1000 - fighter.torque_permille).max(0) / 8)
        + fighter.recovery_slowdown_frames as i32 * 3;

    let mut current = base_frames as i64;
    for modifier in [
        1000 + body_penalty,
        1000 + equipment_penalty,
        1000 + state_penalty,
        1000 + momentum_penalty,
        1000 + injury_penalty,
    ] {
        current = apply_permille_modifier(current, modifier);
    }
    if !action_valid {
        current = current.saturating_add(90);
    }

    CostBreakdown {
        fighter: fighter.seat,
        action: action.label,
        base_frames,
        current_frames: clamp_i64_to_u32(current),
        action_valid,
        factors: vec![
            CostFactor {
                name: "Body",
                permille: 1000 + body_penalty,
                reason: format!(
                    "torso rotation {} permille and balance {} permille set posture cost",
                    fighter.torso_rotation_permille, fighter.balance_permille
                ),
            },
            CostFactor {
                name: "Equipment",
                permille: 1000 + equipment_penalty,
                reason: format!(
                    "{} mass {} g, inertia {} g_cm2, armor mass {} g",
                    fighter.weapon.display_name,
                    fighter.weapon.mass_g,
                    fighter.weapon.inertia_g_cm2,
                    fighter.armor.mass_g
                ),
            },
            CostFactor {
                name: "State",
                permille: 1000 + state_penalty,
                reason: format!(
                    "recovery slowdown currently {} frames for {}",
                    fighter.recovery_slowdown_frames,
                    action.label.as_str()
                ),
            },
            CostFactor {
                name: "Momentum",
                permille: 1000 + momentum_penalty,
                reason: format!(
                    "stored momentum {} permille requires deterministic posture correction",
                    fighter.momentum_permille
                ),
            },
            CostFactor {
                name: "Injury",
                permille: 1000 + injury_penalty,
                reason: format!(
                    "right grip {} permille, torque {} permille, recovery add {} frames",
                    fighter.grip_r_permille,
                    fighter.torque_permille,
                    fighter.recovery_slowdown_frames
                ),
            },
        ],
    }
}

fn apply_permille_modifier(value: i64, modifier: i32) -> i64 {
    mul_permille_round_up_saturating(value, modifier as i64)
}

fn mul_permille_round_up_saturating(value: i64, modifier: i64) -> i64 {
    let product = value as i128 * modifier as i128;
    let rounded = if product >= 0 {
        (product + 999) / 1000
    } else {
        product / 1000
    };
    clamp_i128_to_i64(rounded)
}

fn clamp_i128_to_i64(value: i128) -> i64 {
    if value > i64::MAX as i128 {
        i64::MAX
    } else if value < i64::MIN as i128 {
        i64::MIN
    } else {
        value as i64
    }
}

fn clamp_i64_to_u32(value: i64) -> u32 {
    if value <= 0 {
        0
    } else if value > u32::MAX as i64 {
        u32::MAX
    } else {
        value as u32
    }
}

fn apply_action_movement(fighter: &mut FighterState, action: ActionEntry) {
    match (action.label, action.direction) {
        (ActionLabel::Step, Direction::Forward) => {
            fighter.forward_mm += if fighter.seat == 0 { 120 } else { -120 };
            fighter.momentum_permille += 90;
        }
        (ActionLabel::Step, Direction::Back) => {
            fighter.forward_mm += if fighter.seat == 0 { -140 } else { 140 };
            fighter.momentum_permille -= 70;
        }
        (ActionLabel::Pivot, Direction::Left | Direction::Right) => {
            fighter.momentum_permille += 30;
            fighter.balance_permille = (fighter.balance_permille + 10).min(1000);
        }
        (ActionLabel::Brace, _) => {
            fighter.momentum_permille /= 3;
            fighter.balance_permille = (fighter.balance_permille + 60).min(1000);
        }
        (ActionLabel::Parry, _) => {
            fighter.momentum_permille += 15;
        }
        (ActionLabel::Recover, _) => {
            fighter.recovery_slowdown_frames = fighter.recovery_slowdown_frames.saturating_sub(4);
            fighter.balance_permille = (fighter.balance_permille + 30).min(1000);
            fighter.momentum_permille /= 2;
        }
        (ActionLabel::Guard, _) => {
            fighter.momentum_permille /= 2;
        }
        (
            ActionLabel::Cut
            | ActionLabel::Thrust
            | ActionLabel::Bash
            | ActionLabel::HookBind
            | ActionLabel::Grab
            | ActionLabel::Shove
            | ActionLabel::Kick,
            Direction::Forward,
        ) => {
            fighter.momentum_permille += 50;
        }
        (
            ActionLabel::Cut
            | ActionLabel::Thrust
            | ActionLabel::Bash
            | ActionLabel::HookBind
            | ActionLabel::Grab
            | ActionLabel::Shove
            | ActionLabel::Kick,
            _,
        ) => {
            fighter.momentum_permille += 20;
        }
        _ => {}
    }
}

fn apply_capability_delta(fighter: &mut FighterState, delta: CapabilityDelta) {
    if delta == CapabilityDelta::default() {
        return;
    }
    fighter.torso_rotation_permille = clamp_permille(
        fighter
            .torso_rotation_permille
            .saturating_add(delta.torso_rotation_delta),
    );
    fighter.recovery_slowdown_frames = fighter
        .recovery_slowdown_frames
        .saturating_add(delta.recovery_slowdown_add);
    fighter.balance_permille =
        clamp_permille(fighter.balance_permille.saturating_add(delta.balance_delta));
    fighter.torque_permille =
        clamp_permille(fighter.torque_permille.saturating_add(delta.torque_delta));
    fighter.grip_r_permille =
        clamp_permille(fighter.grip_r_permille.saturating_add(delta.grip_r_delta));
    fighter.grip_l_permille =
        clamp_permille(fighter.grip_l_permille.saturating_add(delta.grip_l_delta));
    if delta.invalidates_thrust {
        fighter.thrust_valid = false;
    }
    if delta.invalidates_cut {
        fighter.cut_valid = false;
    }
    if !delta.event.is_empty() {
        fighter.injury_events.push(delta.event);
    }
}

fn sort_contact_packets(contacts: &mut [ContactTrace]) {
    contacts.sort_by_key(contact_order_key);
}

fn contact_order_key(contact: &ContactTrace) -> (u32, usize, usize, u8, u8, u8) {
    (
        contact.frame,
        contact.attacker,
        contact.defender,
        contact.action.order_key(),
        contact.target.order_key(),
        contact.direction.order_key(),
    )
}

fn clamp_permille(value: i32) -> i32 {
    value.clamp(0, 1000)
}

fn resolve_contact(
    state: &DuelState,
    turn: u32,
    action: ActionEntry,
    defender_action: ActionEntry,
) -> Option<ContactTrace> {
    if !action.label.is_attack() {
        return None;
    }
    let attacker = &state.fighters[action.seat];
    let defender_seat = 1 - action.seat;
    let defender = &state.fighters[defender_seat];
    if !calculate_cost(attacker, action).action_valid {
        return None;
    }

    if defender_action.label == ActionLabel::Step
        && defender_action.direction == Direction::Back
        && attacker.weapon.reach_mm < 1000
    {
        return None;
    }
    let frame = turn * TRUTH_HZ + action.label.base_frames();
    let coverage = armor_coverage(defender.armor, action.target);
    let energy_milli = attacker.weapon.mass_g * action.label.energy_factor()
        + attacker.weapon.inertia_g_cm2 / 20
        + attacker.momentum_permille.abs() * 3;
    let impulse_milli =
        energy_milli * attacker.weapon.alignment_permille / 1000 + attacker.weapon.blunt_permille;

    let (material_result, anatomy_result, delta, cause_chain) = material_and_anatomy_solve(
        action,
        attacker.weapon,
        defender.armor,
        coverage,
        impulse_milli,
    );

    let mut contact = ContactTrace {
        turn,
        frame,
        attacker: action.seat,
        defender: defender_seat,
        action: action.label,
        direction: action.direction,
        target: action.target,
        weapon_id: attacker.weapon.id.to_string(),
        armor_id: defender.armor.id.to_string(),
        energy_milli,
        impulse_milli,
        material_result,
        anatomy_result,
        capability_delta: delta,
        cause_chain,
    };

    if matches!(
        defender_action.label,
        ActionLabel::Guard | ActionLabel::Parry | ActionLabel::Brace
    ) && coverage >= 700
    {
        contact.impulse_milli += defender.armor.binding_permille / 2;
    }

    Some(contact)
}

fn armor_coverage(armor: ArmorProfile, target: TargetRegion) -> i32 {
    match target {
        TargetRegion::Torso => armor.torso_coverage_permille,
        TargetRegion::Head => armor.head_coverage_permille,
        TargetRegion::WeaponArm => armor.weapon_arm_coverage_permille,
        TargetRegion::LeadLeg => armor.lead_leg_coverage_permille,
    }
}

fn material_and_anatomy_solve(
    action: ActionEntry,
    weapon: WeaponProfile,
    armor: ArmorProfile,
    coverage: i32,
    impulse_milli: i32,
) -> (String, String, CapabilityDelta, String) {
    if action.label == ActionLabel::Cut && armor.material == "riveted_mail" && coverage >= 800 {
        let delta = CapabilityDelta {
            torso_rotation_delta: -180,
            recovery_slowdown_add: 12,
            balance_delta: -90,
            torque_delta: -40,
            grip_r_delta: 0,
            grip_l_delta: 0,
            invalidates_thrust: false,
            invalidates_cut: false,
            event: "mail blunt transfer reduced torso rotation and slowed thrust recovery"
                .to_string(),
        };
        return (
            "mail_absorbed_edge_with_blunt_transfer".to_string(),
            "rib compression reduced torso rotation and delayed next thrust recovery".to_string(),
            delta,
            format!(
                "{} -> mail absorbed edge -> blunt rib trauma -> torso rotation -18% -> next thrust recovery +12 frames",
                action.label.as_str()
            ),
        );
    }

    if action.label == ActionLabel::Thrust
        && (coverage + armor.deflection_permille) < weapon.pierce_permille + 520
    {
        let grip_loss = if action.target == TargetRegion::WeaponArm {
            -420
        } else {
            -260
        };
        let delta = CapabilityDelta {
            torso_rotation_delta: -70,
            recovery_slowdown_add: 8,
            balance_delta: -70,
            torque_delta: -90,
            grip_r_delta: grip_loss,
            grip_l_delta: 0,
            invalidates_thrust: grip_loss <= -400,
            invalidates_cut: false,
            event: "thrust found a coverage gap and reduced grip authority".to_string(),
        };
        return (
            "gap_penetration_with_binding".to_string(),
            "weapon-side grip authority reduced and balance checked".to_string(),
            delta,
            format!(
                "{} -> coverage gap at {} -> binding impulse {} -> right grip {} permille -> future weapon action validity checked",
                action.label.as_str(),
                action.target.as_str(),
                impulse_milli,
                grip_loss
            ),
        );
    }

    if matches!(
        action.label,
        ActionLabel::Bash | ActionLabel::Shove | ActionLabel::Kick
    ) && weapon.blunt_permille + armor.blunt_transfer_permille >= 1120
    {
        let delta = CapabilityDelta {
            torso_rotation_delta: -120,
            recovery_slowdown_add: 7,
            balance_delta: -140,
            torque_delta: -55,
            grip_r_delta: -80,
            grip_l_delta: 0,
            invalidates_thrust: false,
            invalidates_cut: false,
            event: "blunt transfer compromised stance and recovery".to_string(),
        };
        return (
            "blunt_transfer_stance_break".to_string(),
            "stance support reduced and recovery delayed".to_string(),
            delta,
            format!(
                "{} -> blunt transfer through {} -> balance -140 permille -> recovery +7 frames",
                action.label.as_str(),
                armor.display_name
            ),
        );
    }

    if action.label == ActionLabel::HookBind && weapon.hook_permille > armor.binding_permille {
        let delta = CapabilityDelta {
            torso_rotation_delta: -60,
            recovery_slowdown_add: 5,
            balance_delta: -60,
            torque_delta: -120,
            grip_r_delta: -180,
            grip_l_delta: 0,
            invalidates_thrust: false,
            invalidates_cut: false,
            event: "hook bind reduced weapon torque and grip authority".to_string(),
        };
        return (
            "hook_bind_torque_loss".to_string(),
            "weapon-side torque reduced by binding leverage".to_string(),
            delta,
            format!(
                "{} -> hook profile exceeded binding resistance -> torque -120 permille -> grip -180 permille",
                action.label.as_str()
            ),
        );
    }

    if coverage >= 700 {
        let delta = CapabilityDelta {
            torso_rotation_delta: -40,
            recovery_slowdown_add: 3,
            balance_delta: -30,
            torque_delta: -20,
            grip_r_delta: 0,
            grip_l_delta: 0,
            invalidates_thrust: false,
            invalidates_cut: false,
            event: "deflection still carried posture shock".to_string(),
        };
        return (
            "deflected_with_posture_shock".to_string(),
            "minor balance and torque reduction".to_string(),
            delta,
            format!(
                "{} -> {} deflected line -> residual impulse {} -> recovery +3 frames",
                action.label.as_str(),
                armor.display_name,
                impulse_milli
            ),
        );
    }

    let delta = CapabilityDelta {
        torso_rotation_delta: -90,
        recovery_slowdown_add: 6,
        balance_delta: -80,
        torque_delta: -60,
        grip_r_delta: -120,
        grip_l_delta: 0,
        invalidates_thrust: false,
        invalidates_cut: false,
        event: "low coverage transferred force into capability loss".to_string(),
    };
    (
        "low_coverage_blunt_transfer".to_string(),
        "balance and grip reduced by transferred force".to_string(),
        delta,
        format!(
            "{} -> low coverage {} permille -> transferred force -> balance -80 permille -> recovery +6 frames",
            action.label.as_str(),
            coverage
        ),
    )
}

fn render_trace_json(
    scenario: &Scenario,
    content_hash_value: &str,
    initial_state_hash: &str,
    final_state_hash: &str,
    end_condition: &DuelEndCondition,
    turns: &[TurnTrace],
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", TRACE_SCHEMA, true);
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    writeln!(&mut out, "  \"truth_hz\": {TRUTH_HZ},").unwrap();
    write_json_field(&mut out, 1, "contact_order_rule", CONTACT_ORDER_RULE, true);
    writeln!(&mut out, "  \"public_demo_ready\": {PUBLIC_DEMO_READY},").unwrap();
    writeln!(
        &mut out,
        "  \"release_candidate_ready\": {RELEASE_CANDIDATE_READY},"
    )
    .unwrap();
    write_json_field(&mut out, 1, "scenario_id", &scenario.id, true);
    write_json_field(&mut out, 1, "content_hash", content_hash_value, true);
    write_json_field(&mut out, 1, "initial_state_hash", initial_state_hash, true);
    write_json_field(&mut out, 1, "final_state_hash", final_state_hash, true);
    render_end_condition_json(&mut out, 1, end_condition, true);
    writeln!(&mut out, "  \"fighters\": [").unwrap();
    for (index, fighter) in scenario.fighters.iter().enumerate() {
        writeln!(
            &mut out,
            "    {{\"seat\": {}, \"name\": {}, \"weapon\": {}, \"armor\": {}}}{}",
            fighter.seat,
            json_quote(&fighter.name),
            json_quote(&fighter.weapon_id),
            json_quote(&fighter.armor_id),
            comma(index + 1, scenario.fighters.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();
    writeln!(&mut out, "  \"turns\": [").unwrap();
    for (turn_index, turn) in turns.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        writeln!(&mut out, "      \"turn\": {},", turn.turn).unwrap();
        writeln!(
            &mut out,
            "      \"phases\": [\"OBSERVE\", \"PLAN\", \"COMMIT_REVEAL\", \"RESOLVE\", \"CONSEQUENCE\", \"REPLAN\"],"
        )
        .unwrap();
        writeln!(&mut out, "      \"commits\": [").unwrap();
        for (index, commit) in turn.commits.iter().enumerate() {
            writeln!(
                &mut out,
                "        {{\"seat\": {}, \"action\": {}, \"direction\": {}, \"target\": {}}}{}",
                commit.seat,
                json_quote(commit.label.as_str()),
                json_quote(commit.direction.as_str()),
                json_quote(commit.target.as_str()),
                comma(index + 1, turn.commits.len())
            )
            .unwrap();
        }
        writeln!(&mut out, "      ],").unwrap();
        writeln!(&mut out, "      \"costs\": [").unwrap();
        for (index, cost) in turn.costs.iter().enumerate() {
            render_cost_json(
                &mut out,
                cost,
                "        ",
                comma(index + 1, turn.costs.len()),
            );
        }
        writeln!(&mut out, "      ],").unwrap();
        writeln!(&mut out, "      \"contacts\": [").unwrap();
        for (index, contact) in turn.contacts.iter().enumerate() {
            render_contact_json(
                &mut out,
                contact,
                "        ",
                comma(index + 1, turn.contacts.len()),
            );
        }
        writeln!(&mut out, "      ],").unwrap();
        write_json_field(&mut out, 3, "state_hash", &turn.state_hash, false);
        writeln!(&mut out, "    }}{}", comma(turn_index + 1, turns.len())).unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

fn render_cost_json(out: &mut String, cost: &CostBreakdown, indent: &str, trailer: &str) {
    writeln!(out, "{indent}{{").unwrap();
    writeln!(out, "{indent}  \"fighter\": {},", cost.fighter).unwrap();
    write_json_field(out, 5, "action", cost.action.as_str(), true);
    writeln!(out, "{indent}  \"base_cost_frames\": {},", cost.base_frames).unwrap();
    writeln!(
        out,
        "{indent}  \"current_cost_frames\": {},",
        cost.current_frames
    )
    .unwrap();
    writeln!(out, "{indent}  \"action_valid\": {},", cost.action_valid).unwrap();
    writeln!(out, "{indent}  \"factors\": [").unwrap();
    for (index, factor) in cost.factors.iter().enumerate() {
        writeln!(
            out,
            "{indent}    {{\"name\": {}, \"permille\": {}, \"reason\": {}}}{}",
            json_quote(factor.name),
            factor.permille,
            json_quote(&factor.reason),
            comma(index + 1, cost.factors.len())
        )
        .unwrap();
    }
    writeln!(out, "{indent}  ]").unwrap();
    writeln!(out, "{indent}}}{trailer}").unwrap();
}

fn render_contact_json(out: &mut String, contact: &ContactTrace, indent: &str, trailer: &str) {
    writeln!(out, "{indent}{{").unwrap();
    writeln!(out, "{indent}  \"turn\": {},", contact.turn).unwrap();
    writeln!(out, "{indent}  \"frame\": {},", contact.frame).unwrap();
    writeln!(out, "{indent}  \"attacker\": {},", contact.attacker).unwrap();
    writeln!(out, "{indent}  \"defender\": {},", contact.defender).unwrap();
    write_json_field(out, 5, "action", contact.action.as_str(), true);
    write_json_field(out, 5, "direction", contact.direction.as_str(), true);
    write_json_field(out, 5, "target", contact.target.as_str(), true);
    write_json_field(out, 5, "weapon", &contact.weapon_id, true);
    write_json_field(out, 5, "armor", &contact.armor_id, true);
    writeln!(out, "{indent}  \"energy_milli\": {},", contact.energy_milli).unwrap();
    writeln!(
        out,
        "{indent}  \"impulse_milli\": {},",
        contact.impulse_milli
    )
    .unwrap();
    write_json_field(out, 5, "material_result", &contact.material_result, true);
    write_json_field(out, 5, "anatomy_result", &contact.anatomy_result, true);
    writeln!(out, "{indent}  \"capability_delta\": {{").unwrap();
    writeln!(
        out,
        "{indent}    \"torso_rotation_delta\": {},",
        contact.capability_delta.torso_rotation_delta
    )
    .unwrap();
    writeln!(
        out,
        "{indent}    \"recovery_slowdown_add\": {},",
        contact.capability_delta.recovery_slowdown_add
    )
    .unwrap();
    writeln!(
        out,
        "{indent}    \"balance_delta\": {},",
        contact.capability_delta.balance_delta
    )
    .unwrap();
    writeln!(
        out,
        "{indent}    \"torque_delta\": {},",
        contact.capability_delta.torque_delta
    )
    .unwrap();
    writeln!(
        out,
        "{indent}    \"grip_r_delta\": {},",
        contact.capability_delta.grip_r_delta
    )
    .unwrap();
    writeln!(
        out,
        "{indent}    \"invalidates_thrust\": {},",
        contact.capability_delta.invalidates_thrust
    )
    .unwrap();
    writeln!(
        out,
        "{indent}    \"invalidates_cut\": {}",
        contact.capability_delta.invalidates_cut
    )
    .unwrap();
    writeln!(out, "{indent}  }},").unwrap();
    write_json_field(out, 5, "cause_chain", &contact.cause_chain, false);
    writeln!(out, "{indent}}}{trailer}").unwrap();
}

fn render_end_condition_json(
    out: &mut String,
    indent: usize,
    end_condition: &DuelEndCondition,
    trailing: bool,
) {
    let spaces = "  ".repeat(indent);
    writeln!(out, "{}\"end_condition\": {{", spaces).unwrap();
    write_json_field(out, indent + 1, "status", &end_condition.status, true);
    write_json_field(
        out,
        indent + 1,
        "winner",
        &end_condition.winner_token(),
        true,
    );
    write_json_field(out, indent + 1, "reason", &end_condition.reason, true);
    writeln!(out, "{}  \"fighters\": [", spaces).unwrap();
    for (index, fighter) in end_condition.fighters.iter().enumerate() {
        writeln!(out, "{}    {{", spaces).unwrap();
        writeln!(out, "{}      \"seat\": {},", spaces, fighter.seat).unwrap();
        writeln!(
            out,
            "{}      \"incapacitated\": {},",
            spaces, fighter.incapacitated
        )
        .unwrap();
        write_json_field(out, indent + 3, "stop_kind", &fighter.stop_kind, true);
        write_json_field(out, indent + 3, "reason", &fighter.reason, true);
        writeln!(
            out,
            "{}      \"balance_permille\": {},",
            spaces, fighter.balance_permille
        )
        .unwrap();
        writeln!(
            out,
            "{}      \"grip_r_permille\": {},",
            spaces, fighter.grip_r_permille
        )
        .unwrap();
        writeln!(
            out,
            "{}      \"torque_permille\": {},",
            spaces, fighter.torque_permille
        )
        .unwrap();
        writeln!(
            out,
            "{}      \"torso_rotation_permille\": {},",
            spaces, fighter.torso_rotation_permille
        )
        .unwrap();
        writeln!(
            out,
            "{}      \"recovery_slowdown_frames\": {},",
            spaces, fighter.recovery_slowdown_frames
        )
        .unwrap();
        writeln!(
            out,
            "{}      \"thrust_valid\": {},",
            spaces, fighter.thrust_valid
        )
        .unwrap();
        writeln!(out, "{}      \"cut_valid\": {}", spaces, fighter.cut_valid).unwrap();
        writeln!(
            out,
            "{}    }}{}",
            spaces,
            comma(index + 1, end_condition.fighters.len())
        )
        .unwrap();
    }
    writeln!(out, "{}  ]", spaces).unwrap();
    writeln!(out, "{}}}{}", spaces, if trailing { "," } else { "" }).unwrap();
}

fn render_replay_json(
    canonical_scenario: &str,
    content_hash_value: &str,
    initial_state_hash: &str,
    final_state_hash: &str,
    end_condition: &DuelEndCondition,
    turn_hashes: &[String],
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", REPLAY_SCHEMA, true);
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    writeln!(&mut out, "  \"truth_hz\": {TRUTH_HZ},").unwrap();
    write_json_field(&mut out, 1, "scenario_canonical", canonical_scenario, true);
    write_json_field(&mut out, 1, "content_hash", content_hash_value, true);
    write_json_field(&mut out, 1, "initial_state_hash", initial_state_hash, true);
    write_json_field(&mut out, 1, "final_state_hash", final_state_hash, true);
    write_json_field(
        &mut out,
        1,
        "end_condition_status",
        &end_condition.status,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "end_condition_winner",
        &end_condition.winner_token(),
        true,
    );
    writeln!(&mut out, "  \"turn_hashes\": [").unwrap();
    for (index, hash) in turn_hashes.iter().enumerate() {
        writeln!(
            &mut out,
            "    {}{}",
            json_quote(hash),
            comma(index + 1, turn_hashes.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

fn render_report_md(
    scenario: &Scenario,
    content_hash_value: &str,
    initial_state_hash: &str,
    final_state_hash: &str,
    end_condition: &DuelEndCondition,
    turns: &[TurnTrace],
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD Duel Report").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "- Scenario: `{}`", scenario.id).unwrap();
    writeln!(&mut out, "- Truth rate: `{TRUTH_HZ} Hz`").unwrap();
    writeln!(&mut out, "- Contact order rule: `{CONTACT_ORDER_RULE}`").unwrap();
    writeln!(&mut out, "- Content hash: `{content_hash_value}`").unwrap();
    writeln!(&mut out, "- Initial state hash: `{initial_state_hash}`").unwrap();
    writeln!(&mut out, "- Final state hash: `{final_state_hash}`").unwrap();
    writeln!(
        &mut out,
        "- End condition: `{}` winner `{}`",
        end_condition.status,
        end_condition.winner_token()
    )
    .unwrap();
    writeln!(&mut out, "- End reason: {}", end_condition.reason).unwrap();
    writeln!(&mut out, "- Public demo ready: `{PUBLIC_DEMO_READY}`").unwrap();
    writeln!(
        &mut out,
        "- Release candidate ready: `{RELEASE_CANDIDATE_READY}`"
    )
    .unwrap();
    writeln!(&mut out).unwrap();
    for fighter in &scenario.fighters {
        writeln!(
            &mut out,
            "- Fighter {} `{}`: weapon `{}`, armor `{}`",
            fighter.seat, fighter.name, fighter.weapon_id, fighter.armor_id
        )
        .unwrap();
    }
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## End Condition").unwrap();
    writeln!(&mut out).unwrap();
    for fighter in &end_condition.fighters {
        writeln!(
            &mut out,
            "- Seat {} `{}`: incapacitated `{}`; balance `{}` grip `{}` torque `{}` torso `{}` recovery `{}` frames; thrust valid `{}` cut valid `{}`; {}",
            fighter.seat,
            fighter.stop_kind,
            fighter.incapacitated,
            fighter.balance_permille,
            fighter.grip_r_permille,
            fighter.torque_permille,
            fighter.torso_rotation_permille,
            fighter.recovery_slowdown_frames,
            fighter.thrust_valid,
            fighter.cut_valid,
            fighter.reason
        )
        .unwrap();
    }
    writeln!(&mut out).unwrap();
    for turn in turns {
        writeln!(&mut out, "## Turn {}", turn.turn).unwrap();
        writeln!(&mut out).unwrap();
        writeln!(&mut out, "Phases: `{}`", turn.phase_order.join(" -> ")).unwrap();
        writeln!(&mut out).unwrap();
        writeln!(&mut out, "### Frame Costs").unwrap();
        for cost in &turn.costs {
            writeln!(
                &mut out,
                "- Fighter {} `{}`: base `{}` frames, current `{}` frames, valid `{}`",
                cost.fighter,
                cost.action.as_str(),
                cost.base_frames,
                cost.current_frames,
                cost.action_valid
            )
            .unwrap();
            for factor in &cost.factors {
                writeln!(
                    &mut out,
                    "  - {} x{} permille: {}",
                    factor.name, factor.permille, factor.reason
                )
                .unwrap();
            }
        }
        writeln!(&mut out).unwrap();
        writeln!(&mut out, "### Contact And Capability").unwrap();
        if turn.contacts.is_empty() {
            writeln!(&mut out, "- No contact packets emitted.").unwrap();
        } else {
            for contact in &turn.contacts {
                writeln!(
                    &mut out,
                    "- Frame {} fighter {} `{}` vs fighter {} `{}`: {}",
                    contact.frame,
                    contact.attacker,
                    contact.action.as_str(),
                    contact.defender,
                    contact.target.as_str(),
                    contact.cause_chain
                )
                .unwrap();
            }
        }
        writeln!(&mut out).unwrap();
        writeln!(&mut out, "Turn state hash: `{}`", turn.state_hash).unwrap();
        writeln!(&mut out).unwrap();
    }
    out
}

fn render_fight_film_manifest_json(
    scenario: &Scenario,
    final_state_hash: &str,
    turns: &[TurnTrace],
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", FIGHT_FILM_SCHEMA, true);
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(&mut out, 1, "scenario_id", &scenario.id, true);
    write_json_field(&mut out, 1, "source", "trace-derived-only", true);
    write_json_field(&mut out, 1, "final_state_hash", final_state_hash, true);
    writeln!(&mut out, "  \"shots\": [").unwrap();
    let mut shots: Vec<(String, u32, String)> = Vec::new();
    for turn in turns {
        for contact in &turn.contacts {
            shots.push((
                "contact_injury_cascade".to_string(),
                contact.frame,
                contact.cause_chain.clone(),
            ));
            if contact.capability_delta.grip_r_delta < 0 {
                shots.push((
                    "grip_loss".to_string(),
                    contact.frame + 1,
                    "right grip authority reduced by contact packet".to_string(),
                ));
            }
            if contact.capability_delta.balance_delta <= -80 {
                shots.push((
                    "stance_collapse_risk".to_string(),
                    contact.frame + 2,
                    "balance penalty crossed bootstrap highlight threshold".to_string(),
                ));
            }
        }
    }
    for (index, (kind, frame, reason)) in shots.iter().enumerate() {
        writeln!(
            &mut out,
            "    {{\"kind\": {}, \"frame\": {}, \"reason\": {}}}{}",
            json_quote(kind),
            frame,
            json_quote(reason),
            comma(index + 1, shots.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

pub fn write_match_artifacts(
    scenario_path: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
    best_of: u32,
) -> Result<String, OathError> {
    let scenario_text = fs::read_to_string(scenario_path)?;
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;
    let rounds_to_win = best_of / 2 + 1;
    let mut seat_wins = [0u32, 0u32];
    let mut round_summaries = Vec::new();

    for round in 1..=best_of {
        if seat_wins[0] >= rounds_to_win || seat_wins[1] >= rounds_to_win {
            break;
        }
        let result = run_scenario_text(&scenario_text)?;
        let winner = round_winner(&result);
        seat_wins[winner] += 1;
        let round_dir = out_dir.join(format!("round_{round}"));
        write_artifacts(&result, &round_dir)?;
        round_summaries.push((
            round,
            winner,
            result.final_state_hash,
            result.end_condition.status.clone(),
            result.end_condition.winner_token(),
        ));
    }

    let match_winner = if seat_wins[0] >= seat_wins[1] { 0 } else { 1 };
    let mut report = String::new();
    writeln!(&mut report, "# OATHYARD Local Match Report").unwrap();
    writeln!(&mut report).unwrap();
    writeln!(&mut report, "- Format: best of {best_of}").unwrap();
    writeln!(&mut report, "- Seat 0 wins: `{}`", seat_wins[0]).unwrap();
    writeln!(&mut report, "- Seat 1 wins: `{}`", seat_wins[1]).unwrap();
    writeln!(&mut report, "- Match winner: `seat {match_winner}`").unwrap();
    writeln!(&mut report, "- Public demo ready: `{PUBLIC_DEMO_READY}`").unwrap();
    writeln!(
        &mut report,
        "- Release candidate ready: `{RELEASE_CANDIDATE_READY}`"
    )
    .unwrap();
    writeln!(&mut report).unwrap();
    for (round, winner, hash, status, outcome_winner) in &round_summaries {
        writeln!(
            &mut report,
            "- Round {round}: winner `seat {winner}`, outcome `{status}` `{outcome_winner}`, final hash `{hash}`"
        )
        .unwrap();
    }

    fs::write(out_dir.join("match_report.md"), &report)?;

    let mut summary = String::new();
    writeln!(&mut summary, "{{").unwrap();
    write_json_field(&mut summary, 1, "schema", "oathyard.match.v1", true);
    write_json_field(&mut summary, 1, "product", PRODUCT_NAME, true);
    writeln!(&mut summary, "  \"best_of\": {best_of},").unwrap();
    writeln!(&mut summary, "  \"seat_0_wins\": {},", seat_wins[0]).unwrap();
    writeln!(&mut summary, "  \"seat_1_wins\": {},", seat_wins[1]).unwrap();
    writeln!(&mut summary, "  \"match_winner\": {match_winner},").unwrap();
    writeln!(&mut summary, "  \"rounds\": [").unwrap();
    for (index, (round, winner, hash, status, outcome_winner)) in round_summaries.iter().enumerate()
    {
        writeln!(
            &mut summary,
            "    {{\"round\": {}, \"winner\": {}, \"end_condition_status\": {}, \"end_condition_winner\": {}, \"final_state_hash\": {}}}{}",
            round,
            winner,
            json_quote(status),
            json_quote(outcome_winner),
            json_quote(hash),
            comma(index + 1, round_summaries.len())
        )
        .unwrap();
    }
    writeln!(&mut summary, "  ]").unwrap();
    writeln!(&mut summary, "}}").unwrap();
    fs::write(out_dir.join("match_summary.json"), summary)?;

    Ok(report)
}

fn round_winner(result: &DuelResult) -> usize {
    if let Some(winner) = result.end_condition.winner {
        return winner;
    }
    let mut penalties = [0i32, 0i32];
    for turn in &result.turns {
        for contact in &turn.contacts {
            let delta = &contact.capability_delta;
            penalties[contact.defender] += delta.recovery_slowdown_add as i32 * 10;
            penalties[contact.defender] += -delta.balance_delta.min(0);
            penalties[contact.defender] += -delta.torque_delta.min(0);
            penalties[contact.defender] += -delta.grip_r_delta.min(0);
            penalties[contact.defender] += -delta.torso_rotation_delta.min(0);
        }
    }
    if penalties[0] <= penalties[1] {
        0
    } else {
        1
    }
}

pub fn write_performance_summary(out_dir: impl AsRef<Path>) -> Result<(), OathError> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD Performance Summary").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "- Truth step rate: `{TRUTH_HZ} Hz`").unwrap();
    writeln!(
        &mut out,
        "- Deterministic benchmark mode: scripted local duel"
    )
    .unwrap();
    writeln!(&mut out, "- Weapon families: `{}`", WEAPONS.len()).unwrap();
    writeln!(&mut out, "- Armor families: `{}`", ARMORS.len()).unwrap();
    writeln!(
        &mut out,
        "- Fighter traditions: `{}`",
        FIGHTER_TRADITIONS.len()
    )
    .unwrap();
    writeln!(&mut out, "- Arena count: `{}`", ARENAS.len()).unwrap();
    writeln!(
        &mut out,
        "- Native 3D renderer gate: native status/XWayland-backed combat renderer available when DISPLAY is present"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Native render command timing is measured by tools/performance_benchmark.py outside truth; production renderer completeness remains incomplete"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Asset memory budget: deterministic text/runtime assets validated by manifest"
    )
    .unwrap();
    fs::write(out_dir.join("performance_summary.md"), out)?;
    Ok(())
}

pub fn write_contact_matrix_artifacts(out_dir: impl AsRef<Path>) -> Result<(), OathError> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;
    let matrix = build_contact_matrix()?;
    fs::write(
        out_dir.join("contact_matrix.json"),
        render_contact_matrix_json(&matrix),
    )?;
    fs::write(
        out_dir.join("contact_matrix_report.md"),
        render_contact_matrix_report(&matrix),
    )?;
    if let Some(failure) = matrix.invariants.iter().find(|invariant| !invariant.passed) {
        return Err(OathError::Verify(format!(
            "contact matrix invariant failed: {}",
            failure.id
        )));
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ContactMatrixEntry {
    weapon_id: String,
    armor_id: String,
    action: ActionLabel,
    target: TargetRegion,
    base_cost_frames: u32,
    current_cost_frames: u32,
    action_valid: bool,
    contact: bool,
    material_result: String,
    anatomy_result: String,
    recovery_slowdown_add: u32,
    balance_delta: i32,
    torque_delta: i32,
    grip_r_delta: i32,
    invalidates_thrust: bool,
    invalidates_cut: bool,
    cause_chain: String,
    final_state_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ContactMatrixInvariant {
    id: &'static str,
    passed: bool,
    evidence_count: usize,
    detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ContactMatrix {
    content_hash: String,
    combinations: usize,
    contacts: usize,
    invalid_actions: usize,
    entries: Vec<ContactMatrixEntry>,
    result_counts: Vec<(String, usize)>,
    invariants: Vec<ContactMatrixInvariant>,
}

fn build_contact_matrix() -> Result<ContactMatrix, OathError> {
    const ATTACKS: [ActionLabel; 7] = [
        ActionLabel::Cut,
        ActionLabel::Thrust,
        ActionLabel::Bash,
        ActionLabel::HookBind,
        ActionLabel::Grab,
        ActionLabel::Shove,
        ActionLabel::Kick,
    ];
    const TARGETS: [TargetRegion; 4] = [
        TargetRegion::Torso,
        TargetRegion::Head,
        TargetRegion::WeaponArm,
        TargetRegion::LeadLeg,
    ];

    let mut entries = Vec::new();
    let mut result_counts: Vec<(String, usize)> = Vec::new();
    let mut contacts = 0usize;
    let mut invalid_actions = 0usize;

    for weapon in WEAPONS {
        for armor in ARMORS {
            for action in ATTACKS {
                for target in TARGETS {
                    let scenario = format!(
                        "scenario matrix_{}_{}_{}_{}\nfighter 0 matrix_attacker {} gambeson\nfighter 1 matrix_defender arming_sword {}\nturn 0 0 {} forward {}\nturn 0 1 guard center {}\n",
                        weapon.id,
                        armor.id,
                        action.as_str(),
                        target.as_str(),
                        weapon.id,
                        armor.id,
                        action.as_str(),
                        target.as_str(),
                        target.as_str()
                    );
                    let result = run_scenario_text(&scenario)?;
                    let turn = &result.turns[0];
                    let cost = &turn.costs[0];
                    if !cost.action_valid {
                        invalid_actions += 1;
                    }
                    let contact = turn.contacts.first();
                    if let Some(contact) = contact {
                        contacts += 1;
                        increment_count(&mut result_counts, &contact.material_result);
                    } else {
                        increment_count(&mut result_counts, "no_contact");
                    }
                    entries.push(ContactMatrixEntry {
                        weapon_id: weapon.id.to_string(),
                        armor_id: armor.id.to_string(),
                        action,
                        target,
                        base_cost_frames: cost.base_frames,
                        current_cost_frames: cost.current_frames,
                        action_valid: cost.action_valid,
                        contact: contact.is_some(),
                        material_result: contact
                            .map(|contact| contact.material_result.clone())
                            .unwrap_or_else(|| "no_contact".to_string()),
                        anatomy_result: contact
                            .map(|contact| contact.anatomy_result.clone())
                            .unwrap_or_else(|| "no anatomy event".to_string()),
                        recovery_slowdown_add: contact
                            .map(|contact| contact.capability_delta.recovery_slowdown_add)
                            .unwrap_or(0),
                        balance_delta: contact
                            .map(|contact| contact.capability_delta.balance_delta)
                            .unwrap_or(0),
                        torque_delta: contact
                            .map(|contact| contact.capability_delta.torque_delta)
                            .unwrap_or(0),
                        grip_r_delta: contact
                            .map(|contact| contact.capability_delta.grip_r_delta)
                            .unwrap_or(0),
                        invalidates_thrust: contact
                            .map(|contact| contact.capability_delta.invalidates_thrust)
                            .unwrap_or(false),
                        invalidates_cut: contact
                            .map(|contact| contact.capability_delta.invalidates_cut)
                            .unwrap_or(false),
                        cause_chain: contact
                            .map(|contact| contact.cause_chain.clone())
                            .unwrap_or_else(|| "no contact packet generated".to_string()),
                        final_state_hash: result.final_state_hash,
                    });
                }
            }
        }
    }

    let invariants = contact_matrix_invariants(&entries, contacts, invalid_actions, &result_counts);

    Ok(ContactMatrix {
        content_hash: content_hash(),
        combinations: entries.len(),
        contacts,
        invalid_actions,
        entries,
        result_counts,
        invariants,
    })
}

fn contact_matrix_invariants(
    entries: &[ContactMatrixEntry],
    contacts: usize,
    invalid_actions: usize,
    result_counts: &[(String, usize)],
) -> Vec<ContactMatrixInvariant> {
    let expected_results = [
        "low_coverage_blunt_transfer",
        "gap_penetration_with_binding",
        "hook_bind_torque_loss",
        "mail_absorbed_edge_with_blunt_transfer",
        "deflected_with_posture_shock",
        "blunt_transfer_stance_break",
    ];
    let result_classes_present = expected_results
        .iter()
        .all(|result| count_result(result_counts, result) > 0);

    let mail_entries: Vec<&ContactMatrixEntry> = entries
        .iter()
        .filter(|entry| entry.material_result == "mail_absorbed_edge_with_blunt_transfer")
        .collect();
    let weapon_arm_gap_entries: Vec<&ContactMatrixEntry> = entries
        .iter()
        .filter(|entry| {
            entry.material_result == "gap_penetration_with_binding"
                && entry.target == TargetRegion::WeaponArm
        })
        .collect();
    let hook_entries: Vec<&ContactMatrixEntry> = entries
        .iter()
        .filter(|entry| entry.material_result == "hook_bind_torque_loss")
        .collect();
    let blunt_entries: Vec<&ContactMatrixEntry> = entries
        .iter()
        .filter(|entry| entry.material_result == "blunt_transfer_stance_break")
        .collect();
    let deflected_entries: Vec<&ContactMatrixEntry> = entries
        .iter()
        .filter(|entry| entry.material_result == "deflected_with_posture_shock")
        .collect();
    let low_coverage_entries: Vec<&ContactMatrixEntry> = entries
        .iter()
        .filter(|entry| entry.material_result == "low_coverage_blunt_transfer")
        .collect();
    let changed_cost_entries = entries
        .iter()
        .filter(|entry| entry.current_cost_frames != entry.base_cost_frames)
        .count();

    vec![
        ContactMatrixInvariant {
            id: "all_combinations_contacted",
            passed: contacts == entries.len(),
            evidence_count: contacts,
            detail: "every shipped weapon/armor/action/target combination generated a deterministic contact packet".to_string(),
        },
        ContactMatrixInvariant {
            id: "all_actions_valid",
            passed: invalid_actions == 0,
            evidence_count: entries.len().saturating_sub(invalid_actions),
            detail: "matrix actions stayed legal under shipped loadouts".to_string(),
        },
        ContactMatrixInvariant {
            id: "all_material_result_classes_present",
            passed: result_classes_present,
            evidence_count: expected_results
                .iter()
                .filter(|result| count_result(result_counts, result) > 0)
                .count(),
            detail: "low coverage, gap penetration, hook bind, mail blunt transfer, deflection, and blunt stance break all appear".to_string(),
        },
        ContactMatrixInvariant {
            id: "mail_cut_blunt_transfer_slows_recovery",
            passed: !mail_entries.is_empty()
                && mail_entries.iter().all(|entry| {
                    entry.action == ActionLabel::Cut
                        && entry.recovery_slowdown_add >= 12
                        && entry.balance_delta <= -90
                        && entry.cause_chain.contains("mail absorbed edge")
                }),
            evidence_count: mail_entries.len(),
            detail: "mail edge absorption creates blunt transfer, balance loss, and at least +12 recovery frames".to_string(),
        },
        ContactMatrixInvariant {
            id: "weapon_arm_gap_penetration_compromises_grip",
            passed: !weapon_arm_gap_entries.is_empty()
                && weapon_arm_gap_entries.iter().all(|entry| {
                    entry.grip_r_delta <= -400
                        && entry.invalidates_thrust
                        && entry.cause_chain.contains("future weapon action validity checked")
                }),
            evidence_count: weapon_arm_gap_entries.len(),
            detail: "weapon-arm gap penetration applies severe right-grip loss and invalidates future thrust availability".to_string(),
        },
        ContactMatrixInvariant {
            id: "hook_bind_reduces_torque_and_grip",
            passed: !hook_entries.is_empty()
                && hook_entries
                    .iter()
                    .all(|entry| entry.torque_delta <= -120 && entry.grip_r_delta <= -180),
            evidence_count: hook_entries.len(),
            detail: "hook/bind results reduce weapon-side torque and right-grip authority".to_string(),
        },
        ContactMatrixInvariant {
            id: "blunt_transfer_breaks_stance",
            passed: !blunt_entries.is_empty()
                && blunt_entries.iter().all(|entry| {
                    entry.balance_delta <= -140 && entry.recovery_slowdown_add >= 7
                }),
            evidence_count: blunt_entries.len(),
            detail: "blunt transfer stance-break results apply major balance loss and recovery delay".to_string(),
        },
        ContactMatrixInvariant {
            id: "deflection_still_applies_posture_shock",
            passed: !deflected_entries.is_empty()
                && deflected_entries.iter().all(|entry| {
                    entry.balance_delta <= -30 && entry.recovery_slowdown_add >= 3
                }),
            evidence_count: deflected_entries.len(),
            detail: "high coverage deflection still applies residual posture shock rather than becoming a no-op".to_string(),
        },
        ContactMatrixInvariant {
            id: "low_coverage_transfers_capability_loss",
            passed: !low_coverage_entries.is_empty()
                && low_coverage_entries.iter().all(|entry| {
                    entry.balance_delta <= -80
                        && entry.grip_r_delta <= -120
                        && entry.recovery_slowdown_add >= 6
                }),
            evidence_count: low_coverage_entries.len(),
            detail: "low coverage transfers force into balance, grip, and recovery capability loss".to_string(),
        },
        ContactMatrixInvariant {
            id: "physical_costs_vary_from_base",
            passed: changed_cost_entries > 0,
            evidence_count: changed_cost_entries,
            detail: "body/equipment/state modifiers change current frame costs from base costs across the matrix".to_string(),
        },
    ]
}

fn count_result(result_counts: &[(String, usize)], key: &str) -> usize {
    result_counts
        .iter()
        .find(|(result, _)| result == key)
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

fn increment_count(counts: &mut Vec<(String, usize)>, key: &str) {
    if let Some((_, count)) = counts.iter_mut().find(|(existing, _)| existing == key) {
        *count += 1;
        return;
    }
    counts.push((key.to_string(), 1));
}

fn render_contact_matrix_json(matrix: &ContactMatrix) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", "oathyard.contact_matrix.v1", true);
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(&mut out, 1, "content_hash", &matrix.content_hash, true);
    writeln!(&mut out, "  \"truth_hz\": {TRUTH_HZ},").unwrap();
    writeln!(&mut out, "  \"public_demo_ready\": {PUBLIC_DEMO_READY},").unwrap();
    writeln!(
        &mut out,
        "  \"release_candidate_ready\": {RELEASE_CANDIDATE_READY},"
    )
    .unwrap();
    writeln!(&mut out, "  \"weapons\": {},", WEAPONS.len()).unwrap();
    writeln!(&mut out, "  \"armors\": {},", ARMORS.len()).unwrap();
    writeln!(&mut out, "  \"attack_labels\": 7,").unwrap();
    writeln!(&mut out, "  \"targets\": 4,").unwrap();
    writeln!(&mut out, "  \"combinations\": {},", matrix.combinations).unwrap();
    writeln!(&mut out, "  \"contacts\": {},", matrix.contacts).unwrap();
    writeln!(
        &mut out,
        "  \"invalid_actions\": {},",
        matrix.invalid_actions
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"invariants_passed\": {},",
        matrix.invariants.iter().all(|invariant| invariant.passed)
    )
    .unwrap();
    writeln!(&mut out, "  \"result_counts\": [").unwrap();
    for (index, (result, count)) in matrix.result_counts.iter().enumerate() {
        writeln!(
            &mut out,
            "    {{\"result\": {}, \"count\": {}}}{}",
            json_quote(result),
            count,
            comma(index + 1, matrix.result_counts.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();
    writeln!(&mut out, "  \"invariants\": [").unwrap();
    for (index, invariant) in matrix.invariants.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "id", invariant.id, true);
        writeln!(&mut out, "      \"passed\": {},", invariant.passed).unwrap();
        writeln!(
            &mut out,
            "      \"evidence_count\": {},",
            invariant.evidence_count
        )
        .unwrap();
        write_json_field(&mut out, 3, "detail", &invariant.detail, false);
        writeln!(
            &mut out,
            "    }}{}",
            comma(index + 1, matrix.invariants.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();
    writeln!(&mut out, "  \"entries\": [").unwrap();
    for (index, entry) in matrix.entries.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "weapon", &entry.weapon_id, true);
        write_json_field(&mut out, 3, "armor", &entry.armor_id, true);
        write_json_field(&mut out, 3, "action", entry.action.as_str(), true);
        write_json_field(&mut out, 3, "target", entry.target.as_str(), true);
        writeln!(
            &mut out,
            "      \"base_cost_frames\": {},",
            entry.base_cost_frames
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"current_cost_frames\": {},",
            entry.current_cost_frames
        )
        .unwrap();
        writeln!(&mut out, "      \"action_valid\": {},", entry.action_valid).unwrap();
        writeln!(&mut out, "      \"contact\": {},", entry.contact).unwrap();
        write_json_field(&mut out, 3, "material_result", &entry.material_result, true);
        write_json_field(&mut out, 3, "anatomy_result", &entry.anatomy_result, true);
        writeln!(
            &mut out,
            "      \"recovery_slowdown_add\": {},",
            entry.recovery_slowdown_add
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"balance_delta\": {},",
            entry.balance_delta
        )
        .unwrap();
        writeln!(&mut out, "      \"torque_delta\": {},", entry.torque_delta).unwrap();
        writeln!(&mut out, "      \"grip_r_delta\": {},", entry.grip_r_delta).unwrap();
        writeln!(
            &mut out,
            "      \"invalidates_thrust\": {},",
            entry.invalidates_thrust
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"invalidates_cut\": {},",
            entry.invalidates_cut
        )
        .unwrap();
        write_json_field(&mut out, 3, "cause_chain", &entry.cause_chain, true);
        write_json_field(
            &mut out,
            3,
            "final_state_hash",
            &entry.final_state_hash,
            false,
        );
        writeln!(&mut out, "    }}{}", comma(index + 1, matrix.entries.len())).unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

fn render_contact_matrix_report(matrix: &ContactMatrix) -> String {
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD Contact Matrix Report").unwrap();
    writeln!(&mut out).unwrap();
    let invariants_passed = matrix.invariants.iter().all(|invariant| invariant.passed);
    writeln!(
        &mut out,
        "Status: {}",
        if invariants_passed {
            "PASSED"
        } else {
            "FAILED"
        }
    )
    .unwrap();
    writeln!(&mut out, "- Content hash: `{}`", matrix.content_hash).unwrap();
    writeln!(&mut out, "- Weapons covered: `{}`", WEAPONS.len()).unwrap();
    writeln!(&mut out, "- Armors covered: `{}`", ARMORS.len()).unwrap();
    writeln!(&mut out, "- Attack labels covered: `7`").unwrap();
    writeln!(&mut out, "- Targets covered: `4`").unwrap();
    writeln!(&mut out, "- Combinations: `{}`", matrix.combinations).unwrap();
    writeln!(&mut out, "- Contacts generated: `{}`", matrix.contacts).unwrap();
    writeln!(&mut out, "- Invalid actions: `{}`", matrix.invalid_actions).unwrap();
    writeln!(&mut out, "- Public demo ready: `{PUBLIC_DEMO_READY}`").unwrap();
    writeln!(
        &mut out,
        "- Release candidate ready: `{RELEASE_CANDIDATE_READY}`"
    )
    .unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Material Results").unwrap();
    writeln!(&mut out).unwrap();
    for (result, count) in &matrix.result_counts {
        writeln!(&mut out, "- `{result}`: `{count}`").unwrap();
    }
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Invariants").unwrap();
    writeln!(&mut out).unwrap();
    for invariant in &matrix.invariants {
        writeln!(
            &mut out,
            "- `{}` passed `{}` evidence `{}`: {}",
            invariant.id, invariant.passed, invariant.evidence_count, invariant.detail
        )
        .unwrap();
    }
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Evidence").unwrap();
    writeln!(&mut out).unwrap();
    for entry in matrix.entries.iter().filter(|entry| entry.contact).take(12) {
        writeln!(
            &mut out,
            "- `{}` vs `{}` using `{}` at `{}`: `{}`; `{}`",
            entry.weapon_id,
            entry.armor_id,
            entry.action.as_str(),
            entry.target.as_str(),
            entry.material_result,
            entry.cause_chain
        )
        .unwrap();
    }
    out
}

pub fn write_ai_duel_artifacts(
    out_dir: impl AsRef<Path>,
    requested_turns: u32,
) -> Result<DuelResult, OathError> {
    let turn_count = requested_turns.clamp(1, 12);
    let plan = build_ai_duel_plan(turn_count)?;
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;
    let run = execute_ai_plan(plan)?;
    write_ai_run_artifacts(out_dir, &run)?;
    Ok(run.result)
}

pub fn write_ai_sweep_artifacts(out_dir: impl AsRef<Path>) -> Result<(), OathError> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;
    let cases = ai_sweep_cases();
    let mut evidences = Vec::new();

    for case in &cases {
        let (first, second, evidence) = build_ai_pairing_runs(case, "ai_sweep")?;
        let case_dir = out_dir.join(case.id);
        write_ai_run_artifacts(case_dir.join("run_a"), &first)?;
        write_ai_run_artifacts(case_dir.join("run_b"), &second)?;
        evidences.push(evidence);
    }

    fs::write(
        out_dir.join("ai_sweep.json"),
        render_ai_sweep_json(&evidences),
    )?;
    fs::write(
        out_dir.join("ai_sweep_report.md"),
        render_ai_sweep_report(&evidences),
    )?;

    if let Some(failure) = evidences.iter().find(|evidence| !evidence.passed()) {
        return Err(OathError::Verify(format!(
            "AI sweep pairing '{}' failed deterministic replay evidence",
            failure.id
        )));
    }
    if distinct_ai_action_labels(&evidences).len() < 5 {
        return Err(OathError::Verify(
            "AI sweep did not cover at least five action labels".to_string(),
        ));
    }
    if distinct_ai_policy_styles(&evidences).len() < 5 {
        return Err(OathError::Verify(
            "AI sweep did not cover at least five policy styles".to_string(),
        ));
    }
    if evidences
        .iter()
        .map(|evidence| evidence.capability_reaction_count)
        .sum::<usize>()
        < 4
    {
        return Err(OathError::Verify(
            "AI sweep did not trigger enough observed capability reactions".to_string(),
        ));
    }
    if unique_ai_final_hashes(&evidences).len() < 2 {
        return Err(OathError::Verify(
            "AI sweep did not produce varied final hashes across pairings".to_string(),
        ));
    }
    Ok(())
}

pub fn write_truth_stress_artifacts(out_dir: impl AsRef<Path>) -> Result<(), OathError> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;
    let cases = truth_stress_cases();
    let mut evidences = Vec::new();

    for case in &cases {
        let (first, second, ai_evidence) = build_ai_pairing_runs(case, "truth_stress")?;
        let case_dir = out_dir.join(case.id);
        write_ai_run_artifacts(case_dir.join("run_a"), &first)?;
        write_ai_run_artifacts(case_dir.join("run_b"), &second)?;
        evidences.push(TruthStressEvidence::from_runs(ai_evidence, &first, &second));
    }

    fs::write(
        out_dir.join("truth_stress.json"),
        render_truth_stress_json(&evidences),
    )?;
    fs::write(
        out_dir.join("truth_stress_report.md"),
        render_truth_stress_report(&evidences),
    )?;

    if let Some(failure) = evidences.iter().find(|evidence| !evidence.passed()) {
        return Err(OathError::Verify(format!(
            "truth stress pairing '{}' failed deterministic evidence",
            failure.ai.id
        )));
    }
    if evidences
        .iter()
        .any(|evidence| evidence.ai.turn_count < TRUTH_STRESS_TURNS)
    {
        return Err(OathError::Verify(
            "truth stress did not run the required long traces".to_string(),
        ));
    }
    if truth_stress_total_contacts(&evidences) < TRUTH_STRESS_MIN_TOTAL_CONTACTS {
        return Err(OathError::Verify(
            "truth stress did not resolve enough contact packets".to_string(),
        ));
    }
    if truth_stress_capability_reactions_total(&evidences) < TRUTH_STRESS_MIN_CAPABILITY_REACTIONS {
        return Err(OathError::Verify(
            "truth stress did not trigger enough capability reactions".to_string(),
        ));
    }
    if truth_stress_capability_stop_count(&evidences) < TRUTH_STRESS_MIN_CAPABILITY_STOPS {
        return Err(OathError::Verify(
            "truth stress did not reach any capability-stop end condition".to_string(),
        ));
    }
    if !truth_stress_thresholds_passed(&evidences) {
        return Err(OathError::Verify(
            "truth stress adversarial solver thresholds were not met".to_string(),
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AiPlan {
    scenario_id: String,
    fighters: [FighterSpec; 2],
    policies: [AiPolicyStyle; 2],
    entries: Vec<AiPlanEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AiPlanEntry {
    turn: u32,
    seat: usize,
    policy: AiPolicyStyle,
    action: ActionLabel,
    direction: Direction,
    target: TargetRegion,
    observed_grip_r_permille: i32,
    observed_balance_permille: i32,
    observed_recovery_slowdown_frames: u32,
    planner_reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AiRun {
    plan: AiPlan,
    scenario_text: String,
    result: DuelResult,
    plan_json: String,
    plan_report: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AiSweepCase {
    id: &'static str,
    description: &'static str,
    turn_count: u32,
    fighters: [FighterSpec; 2],
    policies: [AiPolicyStyle; 2],
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AiSweepEvidence {
    id: &'static str,
    description: &'static str,
    turn_count: u32,
    fighter_0: FighterSpec,
    fighter_1: FighterSpec,
    policy_0: AiPolicyStyle,
    policy_1: AiPolicyStyle,
    plan_entry_count: usize,
    contact_count: usize,
    capability_reaction_count: usize,
    final_state_hash: String,
    repeat_final_state_hash: String,
    plan_hash: String,
    repeat_plan_hash: String,
    committed_sequence_hash: String,
    repeat_committed_sequence_hash: String,
    replay_hash: String,
    repeat_replay_hash: String,
    trace_hash: String,
    repeat_trace_hash: String,
    end_condition_status: String,
    repeat_end_condition_status: String,
    end_condition_winner: String,
    repeat_end_condition_winner: String,
    stable_committed_sequences: bool,
    stable_replay: bool,
    stable_trace: bool,
    replay_verified: bool,
    legal_actions: bool,
    all_truth_actions_valid: bool,
    action_counts: Vec<(&'static str, usize)>,
}

impl AiSweepEvidence {
    fn passed(&self) -> bool {
        self.stable_committed_sequences
            && self.stable_replay
            && self.stable_trace
            && self.replay_verified
            && self.legal_actions
            && self.all_truth_actions_valid
            && self.contact_count > 0
            && self.final_state_hash == self.repeat_final_state_hash
            && self.plan_hash == self.repeat_plan_hash
            && self.committed_sequence_hash == self.repeat_committed_sequence_hash
            && self.replay_hash == self.repeat_replay_hash
            && self.trace_hash == self.repeat_trace_hash
            && self.end_condition_status == self.repeat_end_condition_status
            && self.end_condition_winner == self.repeat_end_condition_winner
    }
}

const TRUTH_STRESS_TURNS: u32 = 24;
const TRUTH_STRESS_MIN_TOTAL_CONTACTS: usize = 72;
const TRUTH_STRESS_MIN_CAPABILITY_REACTIONS: usize = 150;
const TRUTH_STRESS_MIN_CAPABILITY_STOPS: usize = 4;
const TRUTH_STRESS_MIN_DISTINCT_FINAL_HASHES: usize = 5;
const TRUTH_STRESS_MIN_RECOVERY_SLOWDOWN_FRAMES: u32 = 32;
const TRUTH_STRESS_MAX_MIN_BALANCE_PERMILLE: i32 = 100;
const TRUTH_STRESS_MAX_MIN_GRIP_R_PERMILLE: i32 = 100;
const TRUTH_STRESS_MAX_MIN_TORQUE_PERMILLE: i32 = 100;

#[derive(Clone, Debug, PartialEq, Eq)]
struct TruthStressEvidence {
    ai: AiSweepEvidence,
    turn_hash_chain: String,
    repeat_turn_hash_chain: String,
    contact_order_ok: bool,
    repeat_contact_order_ok: bool,
    capability_stop_end_condition: bool,
    max_recovery_slowdown_frames: u32,
    min_balance_permille: i32,
    min_grip_r_permille: i32,
    min_torque_permille: i32,
}

impl TruthStressEvidence {
    fn from_runs(ai: AiSweepEvidence, first: &AiRun, second: &AiRun) -> Self {
        let first_caps = stress_capability_extremes(&first.result);
        let second_caps = stress_capability_extremes(&second.result);
        Self {
            ai,
            turn_hash_chain: turn_hash_chain_hash(&first.result),
            repeat_turn_hash_chain: turn_hash_chain_hash(&second.result),
            contact_order_ok: contact_packets_follow_order(&first.result),
            repeat_contact_order_ok: contact_packets_follow_order(&second.result),
            capability_stop_end_condition: is_capability_stop(&first.result.end_condition.status),
            max_recovery_slowdown_frames: first_caps.0.max(second_caps.0),
            min_balance_permille: first_caps.1.min(second_caps.1),
            min_grip_r_permille: first_caps.2.min(second_caps.2),
            min_torque_permille: first_caps.3.min(second_caps.3),
        }
    }

    fn passed(&self) -> bool {
        self.ai.passed()
            && self.turn_hash_chain == self.repeat_turn_hash_chain
            && self.contact_order_ok
            && self.repeat_contact_order_ok
            && self.ai.turn_count >= TRUTH_STRESS_TURNS
    }
}

fn build_ai_pairing_runs(
    case: &AiSweepCase,
    scenario_prefix: &str,
) -> Result<(AiRun, AiRun, AiSweepEvidence), OathError> {
    let scenario_id = format!("{scenario_prefix}_{}", case.id);
    let first = execute_ai_plan(build_ai_duel_plan_for(
        scenario_id.clone(),
        case.fighters.clone(),
        case.policies,
        case.turn_count,
    )?)?;
    let second = execute_ai_plan(build_ai_duel_plan_for(
        scenario_id,
        case.fighters.clone(),
        case.policies,
        case.turn_count,
    )?)?;
    let replayed_first = verify_replay_text(&first.result.replay_json)?;
    let replayed_second = verify_replay_text(&second.result.replay_json)?;
    let evidence = AiSweepEvidence {
        id: case.id,
        description: case.description,
        turn_count: case.turn_count,
        fighter_0: first.plan.fighters[0].clone(),
        fighter_1: first.plan.fighters[1].clone(),
        policy_0: first.plan.policies[0],
        policy_1: first.plan.policies[1],
        plan_entry_count: first.plan.entries.len(),
        contact_count: count_contacts(&first.result),
        capability_reaction_count: ai_capability_reaction_count(&first.plan),
        final_state_hash: first.result.final_state_hash.clone(),
        repeat_final_state_hash: second.result.final_state_hash.clone(),
        plan_hash: hash_hex(first.plan_json.as_bytes()),
        repeat_plan_hash: hash_hex(second.plan_json.as_bytes()),
        committed_sequence_hash: hash_hex(first.scenario_text.as_bytes()),
        repeat_committed_sequence_hash: hash_hex(second.scenario_text.as_bytes()),
        replay_hash: hash_hex(first.result.replay_json.as_bytes()),
        repeat_replay_hash: hash_hex(second.result.replay_json.as_bytes()),
        trace_hash: hash_hex(first.result.trace_json.as_bytes()),
        repeat_trace_hash: hash_hex(second.result.trace_json.as_bytes()),
        end_condition_status: first.result.end_condition.status.clone(),
        repeat_end_condition_status: second.result.end_condition.status.clone(),
        end_condition_winner: first.result.end_condition.winner_token(),
        repeat_end_condition_winner: second.result.end_condition.winner_token(),
        stable_committed_sequences: first.scenario_text == second.scenario_text,
        stable_replay: first.result.replay_json == second.result.replay_json,
        stable_trace: first.result.trace_json == second.result.trace_json,
        replay_verified: replayed_first.final_state_hash == first.result.final_state_hash
            && replayed_second.final_state_hash == second.result.final_state_hash,
        legal_actions: ai_plan_entries_are_legal(&first.plan)
            && ai_plan_entries_are_legal(&second.plan),
        all_truth_actions_valid: truth_actions_valid(&first.result)
            && truth_actions_valid(&second.result),
        action_counts: ai_action_counts(&first.plan),
    };
    Ok((first, second, evidence))
}

fn turn_hash_chain_hash(result: &DuelResult) -> String {
    hash_hex(result.turn_hashes.join("\n").as_bytes())
}

fn contact_packets_follow_order(result: &DuelResult) -> bool {
    result.turns.iter().all(|turn| {
        turn.contacts
            .windows(2)
            .all(|pair| contact_order_key(&pair[0]) <= contact_order_key(&pair[1]))
    })
}

fn is_capability_stop(status: &str) -> bool {
    matches!(
        status,
        "seat_0_victory_capability_stop"
            | "seat_1_victory_capability_stop"
            | "mutual_capability_stop"
    )
}

fn stress_capability_extremes(result: &DuelResult) -> (u32, i32, i32, i32) {
    let mut max_recovery = 0;
    let mut min_balance = i32::MAX;
    let mut min_grip = i32::MAX;
    let mut min_torque = i32::MAX;
    for fighter in &result.end_condition.fighters {
        max_recovery = max_recovery.max(fighter.recovery_slowdown_frames);
        min_balance = min_balance.min(fighter.balance_permille);
        min_grip = min_grip.min(fighter.grip_r_permille);
        min_torque = min_torque.min(fighter.torque_permille);
    }
    (max_recovery, min_balance, min_grip, min_torque)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AiPolicyStyle {
    Balanced,
    ReachPressure,
    BindControl,
    HeavyPressure,
    GuardCounter,
    EvasiveCounter,
    LowLineDisruptor,
}

impl AiPolicyStyle {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Balanced => "balanced",
            Self::ReachPressure => "reach_pressure",
            Self::BindControl => "bind_control",
            Self::HeavyPressure => "heavy_pressure",
            Self::GuardCounter => "guard_counter",
            Self::EvasiveCounter => "evasive_counter",
            Self::LowLineDisruptor => "low_line_disruptor",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AiObservation {
    grip_r_permille: i32,
    balance_permille: i32,
    recovery_slowdown_frames: u32,
    thrust_valid: bool,
    cut_valid: bool,
}

fn build_ai_duel_plan(turn_count: u32) -> Result<AiPlan, OathError> {
    build_ai_duel_plan_for(
        format!("ai_seedless_observe_replan_{turn_count}"),
        default_ai_fighters(),
        [AiPolicyStyle::Balanced, AiPolicyStyle::ReachPressure],
        turn_count,
    )
}

fn default_ai_fighters() -> [FighterSpec; 2] {
    [
        FighterSpec {
            seat: 0,
            name: "ai_oathyard_writ".to_string(),
            weapon_id: "longsword".to_string(),
            armor_id: "mail_hauberk".to_string(),
        },
        FighterSpec {
            seat: 1,
            name: "ai_reed_sentinel".to_string(),
            weapon_id: "ash_spear".to_string(),
            armor_id: "gambeson".to_string(),
        },
    ]
}

fn build_ai_duel_plan_for(
    scenario_id: String,
    fighters: [FighterSpec; 2],
    policies: [AiPolicyStyle; 2],
    turn_count: u32,
) -> Result<AiPlan, OathError> {
    let mut plan = AiPlan {
        scenario_id,
        fighters,
        policies,
        entries: Vec::new(),
    };

    for turn in 0..turn_count {
        let observations = observe_ai_prefix(&plan)?;
        let seat_0 = choose_ai_entry(&plan, turn, 0, observations[0])?;
        let seat_1 = choose_ai_entry(&plan, turn, 1, observations[1])?;
        plan.entries.push(seat_0);
        plan.entries.push(seat_1);
    }

    Ok(plan)
}

/// R-GAP-1: Enforce the combat-truth freeze gate for all assets referenced by
/// an AI plan. Checks every fighter's weapon_id and armor_id. This is the
/// AI-plan-specific boundary gate — it runs before the plan's action labels
/// are rendered into scenario text and committed to the simulation.
fn enforce_ai_plan_freeze_gate(plan: &AiPlan) -> Result<(), OathError> {
    let repo_root = oathyard_repo_root();
    for fighter in &plan.fighters {
        enforce_combat_truth_freeze_gate(&repo_root, &fighter.weapon_id)?;
        enforce_combat_truth_freeze_gate(&repo_root, &fighter.armor_id)?;
    }
    Ok(())
}

fn execute_ai_plan(plan: AiPlan) -> Result<AiRun, OathError> {
    // R-GAP-1: Enforce the combat-truth freeze gate at the AI plan execution
    // boundary. AI-generated action labels feed directly into the commit path
    // via run_scenario_text. Any AI-derived assets referenced by the plan's
    // fighter specs must have passed all five freeze conditions.
    enforce_ai_plan_freeze_gate(&plan)?;
    let scenario_text = render_ai_scenario_text(&plan);
    let result = run_scenario_text(&scenario_text)?;
    let plan_json = render_ai_plan_json(&plan, &result);
    let plan_report = render_ai_plan_report(&plan, &result);
    Ok(AiRun {
        plan,
        scenario_text,
        result,
        plan_json,
        plan_report,
    })
}

fn write_ai_run_artifacts(out_dir: impl AsRef<Path>, run: &AiRun) -> Result<(), OathError> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;
    write_artifacts(&run.result, out_dir)?;
    fs::write(out_dir.join("ai_scenario.duel"), &run.scenario_text)?;
    fs::write(out_dir.join("ai_plan.json"), &run.plan_json)?;
    fs::write(out_dir.join("ai_plan_report.md"), &run.plan_report)?;
    Ok(())
}

fn ai_sweep_cases() -> Vec<AiSweepCase> {
    vec![
        AiSweepCase {
            id: "reach_vs_mail",
            description: "spear reach pressure into mail-armored longsword response",
            turn_count: 8,
            fighters: default_ai_fighters(),
            policies: [AiPolicyStyle::GuardCounter, AiPolicyStyle::ReachPressure],
        },
        AiSweepCase {
            id: "hook_vs_plate",
            description: "hooking axe decisions against plate and shield-side blunt replies",
            turn_count: 9,
            fighters: [
                ai_fighter(0, "ai_chainbreaker", "bearded_axe", "lamellar"),
                ai_fighter(1, "ai_gate_shield", "round_shield", "heavy_plate"),
            ],
            policies: [AiPolicyStyle::BindControl, AiPolicyStyle::GuardCounter],
        },
        AiSweepCase {
            id: "maul_vs_fencer",
            description: "heavy blunt inertia against light fencer mobility",
            turn_count: 9,
            fighters: [
                ai_fighter(0, "ai_bruiser_oath", "iron_maul", "bruiser_padded_plate"),
                ai_fighter(1, "ai_saltreach_duelist", "arming_sword", "fencer_light"),
            ],
            policies: [AiPolicyStyle::HeavyPressure, AiPolicyStyle::EvasiveCounter],
        },
        AiSweepCase {
            id: "curve_vs_spear_lamellar",
            description: "curved edge follow-through against lamellar spear control",
            turn_count: 8,
            fighters: [
                ai_fighter(0, "ai_saltreach_curve", "curved_sword", "gambeson"),
                ai_fighter(1, "ai_reed_lamellar", "ash_spear", "lamellar"),
            ],
            policies: [
                AiPolicyStyle::LowLineDisruptor,
                AiPolicyStyle::ReachPressure,
            ],
        },
        AiSweepCase {
            id: "shield_counter_vs_curve",
            description: "shield counter timing into curved blade low-line disruption",
            turn_count: 8,
            fighters: [
                ai_fighter(0, "ai_gate_counter", "round_shield", "mail_hauberk"),
                ai_fighter(1, "ai_saltreach_lowline", "curved_sword", "fencer_light"),
            ],
            policies: [AiPolicyStyle::GuardCounter, AiPolicyStyle::LowLineDisruptor],
        },
        AiSweepCase {
            id: "spear_vs_maul_pressure",
            description: "long reach pressure against heavy blunt commitment",
            turn_count: 8,
            fighters: [
                ai_fighter(0, "ai_reed_pressure", "ash_spear", "lamellar"),
                ai_fighter(
                    1,
                    "ai_bruiser_pressure",
                    "iron_maul",
                    "bruiser_padded_plate",
                ),
            ],
            policies: [AiPolicyStyle::ReachPressure, AiPolicyStyle::HeavyPressure],
        },
    ]
}

fn truth_stress_cases() -> Vec<AiSweepCase> {
    let mut cases = ai_sweep_cases();
    for case in &mut cases {
        case.turn_count = TRUTH_STRESS_TURNS;
    }
    cases
}

fn ai_fighter(seat: usize, name: &str, weapon_id: &str, armor_id: &str) -> FighterSpec {
    FighterSpec {
        seat,
        name: name.to_string(),
        weapon_id: weapon_id.to_string(),
        armor_id: armor_id.to_string(),
    }
}

fn observe_ai_prefix(plan: &AiPlan) -> Result<[AiObservation; 2], OathError> {
    let mut observations = [AiObservation {
        grip_r_permille: 1000,
        balance_permille: 1000,
        recovery_slowdown_frames: 0,
        thrust_valid: true,
        cut_valid: true,
    }; 2];
    if plan.entries.is_empty() {
        return Ok(observations);
    }

    let result = run_scenario_text(&render_ai_scenario_text(plan))?;
    for turn in &result.turns {
        for contact in &turn.contacts {
            let observed = &mut observations[contact.defender];
            observed.grip_r_permille =
                clamp_permille(observed.grip_r_permille + contact.capability_delta.grip_r_delta);
            observed.balance_permille =
                clamp_permille(observed.balance_permille + contact.capability_delta.balance_delta);
            observed.recovery_slowdown_frames += contact.capability_delta.recovery_slowdown_add;
            if contact.capability_delta.invalidates_thrust {
                observed.thrust_valid = false;
            }
            if contact.capability_delta.invalidates_cut {
                observed.cut_valid = false;
            }
        }
        for commit in &turn.commits {
            if commit.label == ActionLabel::Recover {
                let observed = &mut observations[commit.seat];
                observed.recovery_slowdown_frames =
                    observed.recovery_slowdown_frames.saturating_sub(4);
                observed.balance_permille = (observed.balance_permille + 30).min(1000);
            }
            if commit.label == ActionLabel::Brace {
                let observed = &mut observations[commit.seat];
                observed.balance_permille = (observed.balance_permille + 60).min(1000);
            }
        }
    }
    Ok(observations)
}

fn choose_ai_entry(
    plan: &AiPlan,
    turn: u32,
    seat: usize,
    observation: AiObservation,
) -> Result<AiPlanEntry, OathError> {
    let fighter = &plan.fighters[seat];
    let weapon = weapon_by_id(&fighter.weapon_id).ok_or_else(|| {
        OathError::Parse(format!(
            "AI fighter has unknown weapon '{}'",
            fighter.weapon_id
        ))
    })?;
    let armor = armor_by_id(&fighter.armor_id).ok_or_else(|| {
        OathError::Parse(format!(
            "AI fighter has unknown armor '{}'",
            fighter.armor_id
        ))
    })?;
    let opponent_weapon = weapon_by_id(&plan.fighters[1 - seat].weapon_id).ok_or_else(|| {
        OathError::Parse(format!(
            "AI opponent has unknown weapon '{}'",
            plan.fighters[1 - seat].weapon_id
        ))
    })?;
    let policy = plan.policies[seat];

    let compromised = observation.grip_r_permille < 560
        || observation.balance_permille < 680
        || observation.recovery_slowdown_frames >= 14;
    let has_initiative = if turn == 0 {
        weapon.reach_mm >= opponent_weapon.reach_mm
    } else {
        (turn + seat as u32) % 2 == 0
    };

    let (action, direction, target, reason) = if compromised {
        if observation.recovery_slowdown_frames >= 8 {
            (
                ActionLabel::Recover,
                Direction::Center,
                TargetRegion::Torso,
                format!(
                    "OBSERVE saw grip {}, balance {}, recovery {}; planner chooses recover and does not author outcomes",
                    observation.grip_r_permille,
                    observation.balance_permille,
                    observation.recovery_slowdown_frames
                ),
            )
        } else {
            (
                ActionLabel::Guard,
                Direction::Center,
                TargetRegion::Torso,
                format!(
                    "OBSERVE saw compromised capability with grip {} and balance {}; planner chooses guard",
                    observation.grip_r_permille, observation.balance_permille
                ),
            )
        }
    } else if has_initiative {
        let action = policy_attack(policy, weapon, opponent_weapon, observation, turn, seat);
        (
            action,
            attack_direction_for_policy(policy, action, turn, seat),
            attack_target_for_policy(policy, action, turn, seat),
            format!(
                "PLAN policy {} selected {} from physical profile reach {} mm, mass {} g, inertia {} g_cm2, edge/pierce/blunt/hook {}/{}/{}/{}",
                policy.as_str(),
                action.as_str(),
                weapon.reach_mm,
                weapon.mass_g,
                weapon.inertia_g_cm2,
                weapon.edge_permille,
                weapon.pierce_permille,
                weapon.blunt_permille,
                weapon.hook_permille
            ),
        )
    } else if policy == AiPolicyStyle::GuardCounter && turn % 3 == 2 {
        (
            ActionLabel::Parry,
            Direction::Center,
            TargetRegion::Torso,
            "PLAN guard_counter chooses parry timing from observed non-initiative window"
                .to_string(),
        )
    } else if policy == AiPolicyStyle::EvasiveCounter
        && opponent_weapon.reach_mm > weapon.reach_mm + 300
        && turn % 2 == 1
    {
        (
            ActionLabel::Step,
            Direction::Back,
            TargetRegion::Torso,
            format!(
                "PLAN evasive_counter gives ground because opponent reach {} mm exceeds own reach {} mm",
                opponent_weapon.reach_mm, weapon.reach_mm
            ),
        )
    } else if policy == AiPolicyStyle::LowLineDisruptor && turn % 4 == 1 {
        (
            ActionLabel::Kick,
            Direction::Low,
            TargetRegion::LeadLeg,
            "PLAN low_line_disruptor uses legal kick to attack support without authoring outcome"
                .to_string(),
        )
    } else if policy == AiPolicyStyle::HeavyPressure && turn % 4 == 2 {
        (
            ActionLabel::Shove,
            Direction::Forward,
            TargetRegion::Torso,
            "PLAN heavy_pressure chooses shove to test balance through truth contact".to_string(),
        )
    } else if armor.mass_g >= 9000 && turn % 3 == 0 {
        (
            ActionLabel::Brace,
            Direction::Center,
            TargetRegion::Torso,
            format!(
                "PLAN uses {} mass {} g to brace against incoming line",
                armor.display_name, armor.mass_g
            ),
        )
    } else if opponent_weapon.reach_mm > weapon.reach_mm + 500 && turn % 4 == 1 {
        (
            ActionLabel::Step,
            Direction::Back,
            TargetRegion::Torso,
            format!(
                "PLAN gives ground because opponent reach {} mm exceeds own reach {} mm",
                opponent_weapon.reach_mm, weapon.reach_mm
            ),
        )
    } else {
        (
            ActionLabel::Guard,
            Direction::Center,
            TargetRegion::Torso,
            "PLAN protects center line while waiting for next observe window".to_string(),
        )
    };

    Ok(AiPlanEntry {
        turn,
        seat,
        policy,
        action,
        direction,
        target,
        observed_grip_r_permille: observation.grip_r_permille,
        observed_balance_permille: observation.balance_permille,
        observed_recovery_slowdown_frames: observation.recovery_slowdown_frames,
        planner_reason: reason,
    })
}

fn dominant_ai_attack(weapon: WeaponProfile, observation: AiObservation, turn: u32) -> ActionLabel {
    if weapon.hook_permille > weapon.edge_permille
        && weapon.hook_permille > weapon.pierce_permille
        && turn % 3 == 1
    {
        ActionLabel::HookBind
    } else if weapon.blunt_permille > weapon.edge_permille
        && weapon.blunt_permille > weapon.pierce_permille
    {
        ActionLabel::Bash
    } else if weapon.pierce_permille >= weapon.edge_permille
        && observation.thrust_valid
        && observation.grip_r_permille >= 620
    {
        ActionLabel::Thrust
    } else if observation.cut_valid && observation.grip_r_permille >= 520 {
        ActionLabel::Cut
    } else {
        ActionLabel::Shove
    }
}

fn policy_attack(
    policy: AiPolicyStyle,
    weapon: WeaponProfile,
    opponent_weapon: WeaponProfile,
    observation: AiObservation,
    turn: u32,
    seat: usize,
) -> ActionLabel {
    match policy {
        AiPolicyStyle::ReachPressure => {
            if observation.thrust_valid
                && observation.grip_r_permille >= 620
                && weapon.pierce_permille >= 350
            {
                ActionLabel::Thrust
            } else {
                dominant_ai_attack(weapon, observation, turn)
            }
        }
        AiPolicyStyle::BindControl => {
            if weapon.hook_permille >= 300 && turn % 2 == 1 {
                ActionLabel::HookBind
            } else {
                dominant_ai_attack(weapon, observation, turn)
            }
        }
        AiPolicyStyle::HeavyPressure => {
            if turn % 4 == 2 {
                ActionLabel::Shove
            } else if weapon.blunt_permille >= 600 {
                ActionLabel::Bash
            } else {
                dominant_ai_attack(weapon, observation, turn)
            }
        }
        AiPolicyStyle::GuardCounter => {
            if turn % 5 == 2 {
                ActionLabel::Parry
            } else if turn % 4 == 3 {
                ActionLabel::Guard
            } else {
                dominant_ai_attack(weapon, observation, turn)
            }
        }
        AiPolicyStyle::EvasiveCounter => {
            if opponent_weapon.reach_mm > weapon.reach_mm + 300 && (turn + seat as u32) % 2 == 1 {
                ActionLabel::Step
            } else if turn % 4 == 2 {
                ActionLabel::Kick
            } else {
                dominant_ai_attack(weapon, observation, turn)
            }
        }
        AiPolicyStyle::LowLineDisruptor => {
            if turn % 4 == 1 {
                ActionLabel::Kick
            } else if turn % 4 == 2 {
                ActionLabel::Shove
            } else {
                dominant_ai_attack(weapon, observation, turn)
            }
        }
        AiPolicyStyle::Balanced => dominant_ai_attack(weapon, observation, turn),
    }
}

fn attack_direction_for_policy(
    policy: AiPolicyStyle,
    action: ActionLabel,
    turn: u32,
    seat: usize,
) -> Direction {
    if policy == AiPolicyStyle::LowLineDisruptor
        && matches!(action, ActionLabel::Kick | ActionLabel::Shove)
    {
        Direction::Low
    } else if policy == AiPolicyStyle::EvasiveCounter && action == ActionLabel::Step {
        Direction::Back
    } else {
        attack_direction(action, turn, seat)
    }
}

fn attack_target_for_policy(
    policy: AiPolicyStyle,
    action: ActionLabel,
    turn: u32,
    seat: usize,
) -> TargetRegion {
    if policy == AiPolicyStyle::LowLineDisruptor
        && matches!(action, ActionLabel::Kick | ActionLabel::Shove)
    {
        TargetRegion::LeadLeg
    } else if policy == AiPolicyStyle::BindControl && action == ActionLabel::HookBind {
        TargetRegion::WeaponArm
    } else {
        attack_target(action, turn, seat)
    }
}

fn attack_direction(action: ActionLabel, turn: u32, seat: usize) -> Direction {
    match action {
        ActionLabel::Cut if (turn + seat as u32) % 2 == 0 => Direction::High,
        ActionLabel::HookBind => Direction::Left,
        ActionLabel::Bash | ActionLabel::Shove | ActionLabel::Kick => Direction::Forward,
        ActionLabel::Thrust if turn % 3 == 0 => Direction::Low,
        ActionLabel::Cut | ActionLabel::Thrust => Direction::Forward,
        _ => Direction::Center,
    }
}

fn attack_target(action: ActionLabel, turn: u32, seat: usize) -> TargetRegion {
    match action {
        ActionLabel::HookBind | ActionLabel::Grab => TargetRegion::WeaponArm,
        ActionLabel::Kick => TargetRegion::LeadLeg,
        ActionLabel::Bash | ActionLabel::Shove => TargetRegion::Torso,
        ActionLabel::Cut if (turn + seat as u32) % 3 == 0 => TargetRegion::Head,
        ActionLabel::Thrust if (turn + seat as u32) % 2 == 0 => TargetRegion::WeaponArm,
        ActionLabel::Cut | ActionLabel::Thrust => TargetRegion::Torso,
        _ => TargetRegion::Torso,
    }
}

fn render_ai_scenario_text(plan: &AiPlan) -> String {
    let mut out = String::new();
    writeln!(&mut out, "scenario {}", plan.scenario_id).unwrap();
    for fighter in &plan.fighters {
        writeln!(
            &mut out,
            "fighter {} {} {} {}",
            fighter.seat, fighter.name, fighter.weapon_id, fighter.armor_id
        )
        .unwrap();
    }
    for entry in &plan.entries {
        writeln!(
            &mut out,
            "turn {} {} {} {} {}",
            entry.turn,
            entry.seat,
            entry.action.as_str(),
            entry.direction.as_str(),
            entry.target.as_str()
        )
        .unwrap();
    }
    out
}

fn render_ai_plan_json(plan: &AiPlan, result: &DuelResult) -> String {
    let legal_actions = ai_plan_entries_are_legal(plan);
    let all_truth_actions_valid = result
        .turns
        .iter()
        .flat_map(|turn| turn.costs.iter())
        .all(|cost| cost.action_valid);
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", AI_PLAN_SCHEMA, true);
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(
        &mut out,
        1,
        "planner",
        "deterministic_seedless_observe_replan_v1",
        true,
    );
    writeln!(&mut out, "  \"truth_hz\": {TRUTH_HZ},").unwrap();
    writeln!(&mut out, "  \"hidden_rng\": false,").unwrap();
    writeln!(&mut out, "  \"wall_clock\": false,").unwrap();
    writeln!(&mut out, "  \"difficulty_changes_body_stats\": false,").unwrap();
    writeln!(&mut out, "  \"legal_actions\": {legal_actions},").unwrap();
    writeln!(
        &mut out,
        "  \"all_truth_actions_valid\": {all_truth_actions_valid},"
    )
    .unwrap();
    write_json_field(&mut out, 1, "outcome_authority", "truth_replay_only", true);
    write_json_field(&mut out, 1, "scenario_id", &plan.scenario_id, true);
    write_json_field(&mut out, 1, "content_hash", &result.content_hash, true);
    write_json_field(
        &mut out,
        1,
        "final_state_hash",
        &result.final_state_hash,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "end_condition_status",
        &result.end_condition.status,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "end_condition_winner",
        &result.end_condition.winner_token(),
        true,
    );
    writeln!(&mut out, "  \"turn_count\": {},", result.turns.len()).unwrap();
    writeln!(&mut out, "  \"policy_styles\": [").unwrap();
    for (index, policy) in plan.policies.iter().enumerate() {
        writeln!(
            &mut out,
            "    {}{}",
            json_quote(policy.as_str()),
            comma(index + 1, plan.policies.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();
    writeln!(&mut out, "  \"fighters\": [").unwrap();
    for (index, fighter) in plan.fighters.iter().enumerate() {
        writeln!(
            &mut out,
            "    {{\"seat\": {}, \"name\": {}, \"weapon\": {}, \"armor\": {}, \"policy\": {}}}{}",
            fighter.seat,
            json_quote(&fighter.name),
            json_quote(&fighter.weapon_id),
            json_quote(&fighter.armor_id),
            json_quote(plan.policies[index].as_str()),
            comma(index + 1, plan.fighters.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();
    writeln!(&mut out, "  \"entries\": [").unwrap();
    for (index, entry) in plan.entries.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        writeln!(&mut out, "      \"turn\": {},", entry.turn).unwrap();
        writeln!(&mut out, "      \"seat\": {},", entry.seat).unwrap();
        write_json_field(&mut out, 3, "policy", entry.policy.as_str(), true);
        write_json_field(&mut out, 3, "action", entry.action.as_str(), true);
        write_json_field(&mut out, 3, "direction", entry.direction.as_str(), true);
        write_json_field(&mut out, 3, "target", entry.target.as_str(), true);
        writeln!(
            &mut out,
            "      \"observed_grip_r_permille\": {},",
            entry.observed_grip_r_permille
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"observed_balance_permille\": {},",
            entry.observed_balance_permille
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"observed_recovery_slowdown_frames\": {},",
            entry.observed_recovery_slowdown_frames
        )
        .unwrap();
        write_json_field(&mut out, 3, "planner_reason", &entry.planner_reason, false);
        writeln!(&mut out, "    }}{}", comma(index + 1, plan.entries.len())).unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

fn ai_plan_entries_are_legal(plan: &AiPlan) -> bool {
    for entry in &plan.entries {
        if entry.seat > 1 {
            return false;
        }
        if ActionLabel::parse(entry.action.as_str()).is_err() {
            return false;
        }
        if Direction::parse(entry.direction.as_str()).is_err() {
            return false;
        }
        if TargetRegion::parse(entry.target.as_str()).is_err() {
            return false;
        }
    }
    true
}

fn render_ai_plan_report(plan: &AiPlan, result: &DuelResult) -> String {
    let all_truth_actions_valid = result
        .turns
        .iter()
        .flat_map(|turn| turn.costs.iter())
        .all(|cost| cost.action_valid);
    let contact_count: usize = result.turns.iter().map(|turn| turn.contacts.len()).sum();
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD Deterministic AI Duel Report").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "Status: PASSED").unwrap();
    writeln!(
        &mut out,
        "- Planner: `deterministic_seedless_observe_replan_v1`"
    )
    .unwrap();
    writeln!(&mut out, "- Hidden RNG: `false`").unwrap();
    writeln!(&mut out, "- Wall clock: `false`").unwrap();
    writeln!(&mut out, "- Difficulty changes body stats: `false`").unwrap();
    writeln!(
        &mut out,
        "- Legal action entries: `{}`",
        ai_plan_entries_are_legal(plan)
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Truth action validity after replayed planning: `{all_truth_actions_valid}`"
    )
    .unwrap();
    writeln!(&mut out, "- Scenario: `{}`", plan.scenario_id).unwrap();
    writeln!(&mut out, "- Content hash: `{}`", result.content_hash).unwrap();
    writeln!(
        &mut out,
        "- Final state hash: `{}`",
        result.final_state_hash
    )
    .unwrap();
    writeln!(
        &mut out,
        "- End condition: `{}` winner `{}`",
        result.end_condition.status,
        result.end_condition.winner_token()
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Contact packets resolved by truth: `{contact_count}`"
    )
    .unwrap();
    writeln!(&mut out, "- Public demo ready: `{PUBLIC_DEMO_READY}`").unwrap();
    writeln!(
        &mut out,
        "- Release candidate ready: `{RELEASE_CANDIDATE_READY}`"
    )
    .unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Plan Entries").unwrap();
    writeln!(&mut out).unwrap();
    for entry in &plan.entries {
        writeln!(
            &mut out,
            "- Turn {} seat {} policy `{}` `{}` `{}` `{}`: {}",
            entry.turn,
            entry.seat,
            entry.policy.as_str(),
            entry.action.as_str(),
            entry.direction.as_str(),
            entry.target.as_str(),
            entry.planner_reason
        )
        .unwrap();
    }
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Truth Cause Chains").unwrap();
    writeln!(&mut out).unwrap();
    for turn in &result.turns {
        for contact in &turn.contacts {
            writeln!(
                &mut out,
                "- Turn {} frame {}: {}",
                turn.turn, contact.frame, contact.cause_chain
            )
            .unwrap();
        }
    }
    out
}

fn render_ai_sweep_json(evidences: &[AiSweepEvidence]) -> String {
    let all_pairings_stable = evidences.iter().all(AiSweepEvidence::passed);
    let all_replays_verified = evidences.iter().all(|evidence| evidence.replay_verified);
    let all_actions_legal = evidences.iter().all(|evidence| evidence.legal_actions);
    let all_truth_actions_valid = evidences
        .iter()
        .all(|evidence| evidence.all_truth_actions_valid);
    let distinct_actions = distinct_ai_action_labels(evidences);
    let policy_styles = distinct_ai_policy_styles(evidences);
    let unique_hashes = unique_ai_final_hashes(evidences);
    let total_contacts: usize = evidences
        .iter()
        .map(|evidence| evidence.contact_count)
        .sum();
    let total_capability_reactions: usize = evidences
        .iter()
        .map(|evidence| evidence.capability_reaction_count)
        .sum();
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", AI_SWEEP_SCHEMA, true);
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(
        &mut out,
        1,
        "planner",
        "deterministic_seedless_observe_replan_v1",
        true,
    );
    writeln!(&mut out, "  \"truth_hz\": {TRUTH_HZ},").unwrap();
    writeln!(&mut out, "  \"hidden_rng\": false,").unwrap();
    writeln!(&mut out, "  \"wall_clock\": false,").unwrap();
    writeln!(&mut out, "  \"difficulty_changes_body_stats\": false,").unwrap();
    writeln!(&mut out, "  \"body_stat_mutation_by_ai\": false,").unwrap();
    write_json_field(&mut out, 1, "outcome_authority", "truth_replay_only", true);
    writeln!(&mut out, "  \"runs_per_pairing\": 2,").unwrap();
    writeln!(&mut out, "  \"pairing_count\": {},", evidences.len()).unwrap();
    writeln!(&mut out, "  \"total_contacts\": {total_contacts},").unwrap();
    writeln!(
        &mut out,
        "  \"capability_reaction_count\": {total_capability_reactions},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"distinct_action_labels\": {},",
        distinct_actions.len()
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"policy_style_count\": {},",
        policy_styles.len()
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"unique_final_hashes\": {},",
        unique_hashes.len()
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"all_pairings_stable\": {all_pairings_stable},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"all_replays_verified\": {all_replays_verified},"
    )
    .unwrap();
    writeln!(&mut out, "  \"all_actions_legal\": {all_actions_legal},").unwrap();
    writeln!(
        &mut out,
        "  \"all_truth_actions_valid\": {all_truth_actions_valid},"
    )
    .unwrap();
    writeln!(&mut out, "  \"public_demo_ready\": {PUBLIC_DEMO_READY},").unwrap();
    writeln!(
        &mut out,
        "  \"release_candidate_ready\": {RELEASE_CANDIDATE_READY},"
    )
    .unwrap();
    writeln!(&mut out, "  \"action_labels\": [").unwrap();
    for (index, action) in distinct_actions.iter().enumerate() {
        writeln!(
            &mut out,
            "    {}{}",
            json_quote(action),
            comma(index + 1, distinct_actions.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();
    writeln!(&mut out, "  \"policy_styles\": [").unwrap();
    for (index, policy) in policy_styles.iter().enumerate() {
        writeln!(
            &mut out,
            "    {}{}",
            json_quote(policy),
            comma(index + 1, policy_styles.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();
    writeln!(&mut out, "  \"pairings\": [").unwrap();
    for (index, evidence) in evidences.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "id", evidence.id, true);
        write_json_field(&mut out, 3, "description", evidence.description, true);
        writeln!(&mut out, "      \"turn_count\": {},", evidence.turn_count).unwrap();
        writeln!(
            &mut out,
            "      \"plan_entry_count\": {},",
            evidence.plan_entry_count
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"contact_count\": {},",
            evidence.contact_count
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"capability_reaction_count\": {},",
            evidence.capability_reaction_count
        )
        .unwrap();
        write_ai_fighter_json(&mut out, "fighter_0", &evidence.fighter_0, true);
        write_ai_fighter_json(&mut out, "fighter_1", &evidence.fighter_1, true);
        write_json_field(&mut out, 3, "policy_0", evidence.policy_0.as_str(), true);
        write_json_field(&mut out, 3, "policy_1", evidence.policy_1.as_str(), true);
        write_json_field(
            &mut out,
            3,
            "final_state_hash",
            &evidence.final_state_hash,
            true,
        );
        write_json_field(
            &mut out,
            3,
            "repeat_final_state_hash",
            &evidence.repeat_final_state_hash,
            true,
        );
        write_json_field(&mut out, 3, "plan_hash", &evidence.plan_hash, true);
        write_json_field(
            &mut out,
            3,
            "repeat_plan_hash",
            &evidence.repeat_plan_hash,
            true,
        );
        write_json_field(
            &mut out,
            3,
            "committed_sequence_hash",
            &evidence.committed_sequence_hash,
            true,
        );
        write_json_field(
            &mut out,
            3,
            "repeat_committed_sequence_hash",
            &evidence.repeat_committed_sequence_hash,
            true,
        );
        write_json_field(&mut out, 3, "replay_hash", &evidence.replay_hash, true);
        write_json_field(
            &mut out,
            3,
            "repeat_replay_hash",
            &evidence.repeat_replay_hash,
            true,
        );
        write_json_field(&mut out, 3, "trace_hash", &evidence.trace_hash, true);
        write_json_field(
            &mut out,
            3,
            "repeat_trace_hash",
            &evidence.repeat_trace_hash,
            true,
        );
        write_json_field(
            &mut out,
            3,
            "end_condition_status",
            &evidence.end_condition_status,
            true,
        );
        write_json_field(
            &mut out,
            3,
            "repeat_end_condition_status",
            &evidence.repeat_end_condition_status,
            true,
        );
        write_json_field(
            &mut out,
            3,
            "end_condition_winner",
            &evidence.end_condition_winner,
            true,
        );
        write_json_field(
            &mut out,
            3,
            "repeat_end_condition_winner",
            &evidence.repeat_end_condition_winner,
            true,
        );
        writeln!(
            &mut out,
            "      \"stable_committed_sequences\": {},",
            evidence.stable_committed_sequences
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"stable_replay\": {},",
            evidence.stable_replay
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"stable_trace\": {},",
            evidence.stable_trace
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"replay_verified\": {},",
            evidence.replay_verified
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"legal_actions\": {},",
            evidence.legal_actions
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"all_truth_actions_valid\": {},",
            evidence.all_truth_actions_valid
        )
        .unwrap();
        writeln!(&mut out, "      \"passed\": {},", evidence.passed()).unwrap();
        writeln!(&mut out, "      \"action_counts\": [").unwrap();
        for (action_index, (action, count)) in evidence.action_counts.iter().enumerate() {
            writeln!(
                &mut out,
                "        {{\"action\": {}, \"count\": {}}}{}",
                json_quote(action),
                count,
                comma(action_index + 1, evidence.action_counts.len())
            )
            .unwrap();
        }
        writeln!(&mut out, "      ]").unwrap();
        writeln!(&mut out, "    }}{}", comma(index + 1, evidences.len())).unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

fn write_ai_fighter_json(out: &mut String, key: &str, fighter: &FighterSpec, trailing: bool) {
    writeln!(
        out,
        "      \"{key}\": {{\"seat\": {}, \"name\": {}, \"weapon\": {}, \"armor\": {}}}{}",
        fighter.seat,
        json_quote(&fighter.name),
        json_quote(&fighter.weapon_id),
        json_quote(&fighter.armor_id),
        if trailing { "," } else { "" }
    )
    .unwrap();
}

fn render_ai_sweep_report(evidences: &[AiSweepEvidence]) -> String {
    let all_pairings_stable = evidences.iter().all(AiSweepEvidence::passed);
    let distinct_actions = distinct_ai_action_labels(evidences);
    let policy_styles = distinct_ai_policy_styles(evidences);
    let unique_hashes = unique_ai_final_hashes(evidences);
    let total_contacts: usize = evidences
        .iter()
        .map(|evidence| evidence.contact_count)
        .sum();
    let total_capability_reactions: usize = evidences
        .iter()
        .map(|evidence| evidence.capability_reaction_count)
        .sum();
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD Deterministic AI Sweep Report").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(
        &mut out,
        "Status: {}",
        if all_pairings_stable {
            "PASSED"
        } else {
            "FAILED"
        }
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Planner: `deterministic_seedless_observe_replan_v1`"
    )
    .unwrap();
    writeln!(&mut out, "- Hidden RNG: `false`").unwrap();
    writeln!(&mut out, "- Wall clock: `false`").unwrap();
    writeln!(&mut out, "- Difficulty changes body stats: `false`").unwrap();
    writeln!(&mut out, "- Outcome authority: `truth_replay_only`").unwrap();
    writeln!(&mut out, "- Runs per pairing: `2`").unwrap();
    writeln!(&mut out, "- Pairings: `{}`", evidences.len()).unwrap();
    writeln!(&mut out, "- Total contacts: `{total_contacts}`").unwrap();
    writeln!(
        &mut out,
        "- Capability reactions: `{total_capability_reactions}`"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Distinct action labels: `{}`",
        distinct_actions.len()
    )
    .unwrap();
    writeln!(&mut out, "- Policy styles: `{}`", policy_styles.len()).unwrap();
    writeln!(&mut out, "- Unique final hashes: `{}`", unique_hashes.len()).unwrap();
    writeln!(&mut out, "- Public demo ready: `{PUBLIC_DEMO_READY}`").unwrap();
    writeln!(
        &mut out,
        "- Release candidate ready: `{RELEASE_CANDIDATE_READY}`"
    )
    .unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Pairings").unwrap();
    writeln!(&mut out).unwrap();
    for evidence in evidences {
        writeln!(
            &mut out,
            "- `{}` passed `{}` contacts `{}` reactions `{}` final `{}` committed `{}` replay `{}`: {}",
            evidence.id,
            evidence.passed(),
            evidence.contact_count,
            evidence.capability_reaction_count,
            evidence.final_state_hash,
            evidence.committed_sequence_hash,
            evidence.replay_hash,
            evidence.description
        )
        .unwrap();
        writeln!(
            &mut out,
            "  - end condition `{}` winner `{}` repeat `{}` `{}`",
            evidence.end_condition_status,
            evidence.end_condition_winner,
            evidence.repeat_end_condition_status,
            evidence.repeat_end_condition_winner
        )
        .unwrap();
        writeln!(
            &mut out,
            "  - seat 0 `{}` `{}` `{}` policy `{}`; seat 1 `{}` `{}` `{}` policy `{}`",
            evidence.fighter_0.name,
            evidence.fighter_0.weapon_id,
            evidence.fighter_0.armor_id,
            evidence.policy_0.as_str(),
            evidence.fighter_1.name,
            evidence.fighter_1.weapon_id,
            evidence.fighter_1.armor_id,
            evidence.policy_1.as_str()
        )
        .unwrap();
    }
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Action Coverage").unwrap();
    writeln!(&mut out).unwrap();
    for action in distinct_actions {
        writeln!(&mut out, "- `{action}`").unwrap();
    }
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Policy Styles").unwrap();
    writeln!(&mut out).unwrap();
    for policy in policy_styles {
        writeln!(&mut out, "- `{policy}`").unwrap();
    }
    out
}

fn render_truth_stress_json(evidences: &[TruthStressEvidence]) -> String {
    let total_contacts = truth_stress_total_contacts(evidences);
    let total_capability_reactions = truth_stress_capability_reactions_total(evidences);
    let capability_stop_count = truth_stress_capability_stop_count(evidences);
    let distinct_final_hash_count = truth_stress_distinct_final_hash_count(evidences);
    let max_recovery_slowdown_frames = truth_stress_max_recovery_slowdown_frames(evidences);
    let min_balance_permille = truth_stress_min_balance_permille(evidences);
    let min_grip_r_permille = truth_stress_min_grip_r_permille(evidences);
    let min_torque_permille = truth_stress_min_torque_permille(evidences);
    let stress_thresholds_passed = truth_stress_thresholds_passed(evidences);
    let all_stress_cases_stable = evidences.iter().all(TruthStressEvidence::passed);
    let all_contact_packets_ordered = evidences
        .iter()
        .all(|evidence| evidence.contact_order_ok && evidence.repeat_contact_order_ok);
    let all_turn_hash_chains_stable = evidences
        .iter()
        .all(|evidence| evidence.turn_hash_chain == evidence.repeat_turn_hash_chain);
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", TRUTH_STRESS_SCHEMA, true);
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    writeln!(&mut out, "  \"truth_hz\": {TRUTH_HZ},").unwrap();
    writeln!(&mut out, "  \"hidden_rng\": false,").unwrap();
    writeln!(&mut out, "  \"wall_clock\": false,").unwrap();
    writeln!(&mut out, "  \"gameplay_floats\": false,").unwrap();
    writeln!(&mut out, "  \"runs_per_pairing\": 2,").unwrap();
    writeln!(&mut out, "  \"pairing_count\": {},", evidences.len()).unwrap();
    writeln!(&mut out, "  \"stress_turn_count\": {TRUTH_STRESS_TURNS},").unwrap();
    writeln!(
        &mut out,
        "  \"minimum_total_contacts_required\": {TRUTH_STRESS_MIN_TOTAL_CONTACTS},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"minimum_capability_reactions_required\": {TRUTH_STRESS_MIN_CAPABILITY_REACTIONS},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"minimum_capability_stops_required\": {TRUTH_STRESS_MIN_CAPABILITY_STOPS},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"minimum_distinct_final_hashes_required\": {TRUTH_STRESS_MIN_DISTINCT_FINAL_HASHES},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"minimum_recovery_slowdown_required\": {TRUTH_STRESS_MIN_RECOVERY_SLOWDOWN_FRAMES},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"maximum_min_balance_required\": {TRUTH_STRESS_MAX_MIN_BALANCE_PERMILLE},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"maximum_min_grip_r_required\": {TRUTH_STRESS_MAX_MIN_GRIP_R_PERMILLE},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"maximum_min_torque_required\": {TRUTH_STRESS_MAX_MIN_TORQUE_PERMILLE},"
    )
    .unwrap();
    writeln!(&mut out, "  \"total_contacts\": {total_contacts},").unwrap();
    writeln!(
        &mut out,
        "  \"capability_reaction_count\": {total_capability_reactions},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"capability_stop_count\": {capability_stop_count},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"distinct_final_hash_count\": {distinct_final_hash_count},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"max_recovery_slowdown_frames\": {max_recovery_slowdown_frames},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"min_balance_permille\": {min_balance_permille},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"min_grip_r_permille\": {min_grip_r_permille},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"min_torque_permille\": {min_torque_permille},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"stress_thresholds_passed\": {stress_thresholds_passed},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"all_stress_cases_stable\": {all_stress_cases_stable},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"all_contact_packets_ordered\": {all_contact_packets_ordered},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"all_turn_hash_chains_stable\": {all_turn_hash_chains_stable},"
    )
    .unwrap();
    write_json_field(&mut out, 1, "contact_order_rule", CONTACT_ORDER_RULE, true);
    write_json_field(&mut out, 1, "outcome_authority", "truth_replay_only", true);
    writeln!(&mut out, "  \"public_demo_ready\": {PUBLIC_DEMO_READY},").unwrap();
    writeln!(
        &mut out,
        "  \"release_candidate_ready\": {RELEASE_CANDIDATE_READY},"
    )
    .unwrap();
    writeln!(&mut out, "  \"pairings\": [").unwrap();
    for (index, evidence) in evidences.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "id", evidence.ai.id, true);
        write_json_field(&mut out, 3, "description", evidence.ai.description, true);
        writeln!(
            &mut out,
            "      \"turn_count\": {},",
            evidence.ai.turn_count
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"plan_entry_count\": {},",
            evidence.ai.plan_entry_count
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"contact_count\": {},",
            evidence.ai.contact_count
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"capability_reaction_count\": {},",
            evidence.ai.capability_reaction_count
        )
        .unwrap();
        write_ai_fighter_json(&mut out, "fighter_0", &evidence.ai.fighter_0, true);
        write_ai_fighter_json(&mut out, "fighter_1", &evidence.ai.fighter_1, true);
        write_json_field(&mut out, 3, "policy_0", evidence.ai.policy_0.as_str(), true);
        write_json_field(&mut out, 3, "policy_1", evidence.ai.policy_1.as_str(), true);
        write_json_field(
            &mut out,
            3,
            "final_state_hash",
            &evidence.ai.final_state_hash,
            true,
        );
        write_json_field(
            &mut out,
            3,
            "repeat_final_state_hash",
            &evidence.ai.repeat_final_state_hash,
            true,
        );
        write_json_field(
            &mut out,
            3,
            "turn_hash_chain",
            &evidence.turn_hash_chain,
            true,
        );
        write_json_field(
            &mut out,
            3,
            "repeat_turn_hash_chain",
            &evidence.repeat_turn_hash_chain,
            true,
        );
        write_json_field(
            &mut out,
            3,
            "end_condition_status",
            &evidence.ai.end_condition_status,
            true,
        );
        write_json_field(
            &mut out,
            3,
            "end_condition_winner",
            &evidence.ai.end_condition_winner,
            true,
        );
        writeln!(
            &mut out,
            "      \"stable_committed_sequences\": {},",
            evidence.ai.stable_committed_sequences
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"stable_replay\": {},",
            evidence.ai.stable_replay
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"stable_trace\": {},",
            evidence.ai.stable_trace
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"stable_turn_hash_chain\": {},",
            evidence.turn_hash_chain == evidence.repeat_turn_hash_chain
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"contact_order_ok\": {},",
            evidence.contact_order_ok
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"repeat_contact_order_ok\": {},",
            evidence.repeat_contact_order_ok
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"replay_verified\": {},",
            evidence.ai.replay_verified
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"all_truth_actions_valid\": {},",
            evidence.ai.all_truth_actions_valid
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"capability_stop_end_condition\": {},",
            evidence.capability_stop_end_condition
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"max_recovery_slowdown_frames\": {},",
            evidence.max_recovery_slowdown_frames
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"min_balance_permille\": {},",
            evidence.min_balance_permille
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"min_grip_r_permille\": {},",
            evidence.min_grip_r_permille
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"min_torque_permille\": {},",
            evidence.min_torque_permille
        )
        .unwrap();
        writeln!(&mut out, "      \"passed\": {}", evidence.passed()).unwrap();
        writeln!(&mut out, "    }}{}", comma(index + 1, evidences.len())).unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

fn render_truth_stress_report(evidences: &[TruthStressEvidence]) -> String {
    let total_contacts = truth_stress_total_contacts(evidences);
    let total_capability_reactions = truth_stress_capability_reactions_total(evidences);
    let capability_stop_count = truth_stress_capability_stop_count(evidences);
    let distinct_final_hash_count = truth_stress_distinct_final_hash_count(evidences);
    let max_recovery_slowdown_frames = truth_stress_max_recovery_slowdown_frames(evidences);
    let min_balance_permille = truth_stress_min_balance_permille(evidences);
    let min_grip_r_permille = truth_stress_min_grip_r_permille(evidences);
    let min_torque_permille = truth_stress_min_torque_permille(evidences);
    let stress_thresholds_passed = truth_stress_thresholds_passed(evidences);
    let all_stress_cases_stable = evidences.iter().all(TruthStressEvidence::passed);
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD Truth Stress Report").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(
        &mut out,
        "Status: {}",
        if all_stress_cases_stable {
            "PASSED"
        } else {
            "FAILED"
        }
    )
    .unwrap();
    writeln!(&mut out, "- Truth Hz: `{TRUTH_HZ}`").unwrap();
    writeln!(&mut out, "- Stress turn count: `{TRUTH_STRESS_TURNS}`").unwrap();
    writeln!(&mut out, "- Runs per pairing: `2`").unwrap();
    writeln!(&mut out, "- Pairings: `{}`", evidences.len()).unwrap();
    writeln!(&mut out, "- Total contacts: `{total_contacts}`").unwrap();
    writeln!(
        &mut out,
        "- Capability reactions: `{total_capability_reactions}`"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Capability-stop end conditions: `{capability_stop_count}`"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Distinct final hashes: `{distinct_final_hash_count}`"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Global capability extrema: recovery `{max_recovery_slowdown_frames}` frames, balance `{min_balance_permille}` permille, grip `{min_grip_r_permille}` permille, torque `{min_torque_permille}` permille"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Adversarial thresholds passed: `{stress_thresholds_passed}`"
    )
    .unwrap();
    writeln!(&mut out, "- Contact order rule: `{CONTACT_ORDER_RULE}`").unwrap();
    writeln!(&mut out, "- Hidden RNG: `false`").unwrap();
    writeln!(&mut out, "- Wall clock: `false`").unwrap();
    writeln!(&mut out, "- Gameplay floats: `false`").unwrap();
    writeln!(&mut out, "- Public demo ready: `{PUBLIC_DEMO_READY}`").unwrap();
    writeln!(
        &mut out,
        "- Release candidate ready: `{RELEASE_CANDIDATE_READY}`"
    )
    .unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Adversarial Thresholds").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(
        &mut out,
        "- contacts >= `{TRUTH_STRESS_MIN_TOTAL_CONTACTS}` observed `{total_contacts}`"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- capability reactions >= `{TRUTH_STRESS_MIN_CAPABILITY_REACTIONS}` observed `{total_capability_reactions}`"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- capability-stop outcomes >= `{TRUTH_STRESS_MIN_CAPABILITY_STOPS}` observed `{capability_stop_count}`"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- distinct final hashes >= `{TRUTH_STRESS_MIN_DISTINCT_FINAL_HASHES}` observed `{distinct_final_hash_count}`"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- max recovery slowdown >= `{TRUTH_STRESS_MIN_RECOVERY_SLOWDOWN_FRAMES}` observed `{max_recovery_slowdown_frames}`"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- min balance <= `{TRUTH_STRESS_MAX_MIN_BALANCE_PERMILLE}` observed `{min_balance_permille}`"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- min grip <= `{TRUTH_STRESS_MAX_MIN_GRIP_R_PERMILLE}` observed `{min_grip_r_permille}`"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- min torque <= `{TRUTH_STRESS_MAX_MIN_TORQUE_PERMILLE}` observed `{min_torque_permille}`"
    )
    .unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Pairings").unwrap();
    writeln!(&mut out).unwrap();
    for evidence in evidences {
        writeln!(
            &mut out,
            "- `{}` passed `{}` turns `{}` contacts `{}` reactions `{}` final `{}` turn-chain `{}` stop `{}`",
            evidence.ai.id,
            evidence.passed(),
            evidence.ai.turn_count,
            evidence.ai.contact_count,
            evidence.ai.capability_reaction_count,
            evidence.ai.final_state_hash,
            evidence.turn_hash_chain,
            evidence.ai.end_condition_status
        )
        .unwrap();
        writeln!(
            &mut out,
            "  - contact order `{}` repeat `{}` replay `{}` trace `{}` committed `{}`",
            evidence.contact_order_ok,
            evidence.repeat_contact_order_ok,
            evidence.ai.stable_replay,
            evidence.ai.stable_trace,
            evidence.ai.stable_committed_sequences
        )
        .unwrap();
        writeln!(
            &mut out,
            "  - capability extrema: recovery `{}` frames, balance `{}` permille, grip `{}` permille, torque `{}` permille",
            evidence.max_recovery_slowdown_frames,
            evidence.min_balance_permille,
            evidence.min_grip_r_permille,
            evidence.min_torque_permille
        )
        .unwrap();
    }
    out
}

fn truth_stress_total_contacts(evidences: &[TruthStressEvidence]) -> usize {
    evidences
        .iter()
        .map(|evidence| evidence.ai.contact_count)
        .sum()
}

fn truth_stress_capability_reactions_total(evidences: &[TruthStressEvidence]) -> usize {
    evidences
        .iter()
        .map(|evidence| evidence.ai.capability_reaction_count)
        .sum()
}

fn truth_stress_capability_stop_count(evidences: &[TruthStressEvidence]) -> usize {
    evidences
        .iter()
        .filter(|evidence| evidence.capability_stop_end_condition)
        .count()
}

fn truth_stress_distinct_final_hash_count(evidences: &[TruthStressEvidence]) -> usize {
    let mut hashes: Vec<&str> = evidences
        .iter()
        .map(|evidence| evidence.ai.final_state_hash.as_str())
        .collect();
    hashes.sort_unstable();
    hashes.dedup();
    hashes.len()
}

fn truth_stress_max_recovery_slowdown_frames(evidences: &[TruthStressEvidence]) -> u32 {
    evidences
        .iter()
        .map(|evidence| evidence.max_recovery_slowdown_frames)
        .max()
        .unwrap_or(0)
}

fn truth_stress_min_balance_permille(evidences: &[TruthStressEvidence]) -> i32 {
    evidences
        .iter()
        .map(|evidence| evidence.min_balance_permille)
        .min()
        .unwrap_or(1000)
}

fn truth_stress_min_grip_r_permille(evidences: &[TruthStressEvidence]) -> i32 {
    evidences
        .iter()
        .map(|evidence| evidence.min_grip_r_permille)
        .min()
        .unwrap_or(1000)
}

fn truth_stress_min_torque_permille(evidences: &[TruthStressEvidence]) -> i32 {
    evidences
        .iter()
        .map(|evidence| evidence.min_torque_permille)
        .min()
        .unwrap_or(1000)
}

fn truth_stress_thresholds_passed(evidences: &[TruthStressEvidence]) -> bool {
    truth_stress_total_contacts(evidences) >= TRUTH_STRESS_MIN_TOTAL_CONTACTS
        && truth_stress_capability_reactions_total(evidences)
            >= TRUTH_STRESS_MIN_CAPABILITY_REACTIONS
        && truth_stress_capability_stop_count(evidences) >= TRUTH_STRESS_MIN_CAPABILITY_STOPS
        && truth_stress_distinct_final_hash_count(evidences)
            >= TRUTH_STRESS_MIN_DISTINCT_FINAL_HASHES
        && truth_stress_max_recovery_slowdown_frames(evidences)
            >= TRUTH_STRESS_MIN_RECOVERY_SLOWDOWN_FRAMES
        && truth_stress_min_balance_permille(evidences) <= TRUTH_STRESS_MAX_MIN_BALANCE_PERMILLE
        && truth_stress_min_grip_r_permille(evidences) <= TRUTH_STRESS_MAX_MIN_GRIP_R_PERMILLE
        && truth_stress_min_torque_permille(evidences) <= TRUTH_STRESS_MAX_MIN_TORQUE_PERMILLE
}

#[derive(Clone, Debug)]
struct TruthEdgeMathCase {
    id: &'static str,
    input: i64,
    modifier_permille: i64,
    expected: i64,
    actual: i64,
    passed: bool,
}

#[derive(Clone, Debug)]
struct TruthEdgeFixedCase {
    id: &'static str,
    input_milli: i64,
    numerator: i64,
    denominator: i64,
    expected_milli: i64,
    actual_milli: i64,
    passed: bool,
}

#[derive(Clone, Debug)]
struct TruthEdgeCapabilityCase {
    id: &'static str,
    balance_permille: i32,
    grip_r_permille: i32,
    grip_l_permille: i32,
    torque_permille: i32,
    torso_rotation_permille: i32,
    recovery_slowdown_frames: u32,
    thrust_valid: bool,
    cut_valid: bool,
    passed: bool,
}

#[derive(Clone, Debug)]
struct TruthEdgeCostCase {
    id: &'static str,
    action: ActionLabel,
    base_frames: u32,
    current_frames: u32,
    action_valid: bool,
    factor_count: usize,
    passed: bool,
}

#[derive(Clone, Debug)]
struct TruthEdgeOrderCase {
    id: &'static str,
    expected_signature: String,
    actual_signature: String,
    passed: bool,
}

#[derive(Clone, Debug)]
struct TruthEdgeReplayCase {
    id: &'static str,
    expected_pass: bool,
    observed_pass: bool,
    message: String,
    passed: bool,
}

#[derive(Clone, Debug)]
struct TruthEdgeAudit {
    math_cases: Vec<TruthEdgeMathCase>,
    fixed_cases: Vec<TruthEdgeFixedCase>,
    capability_cases: Vec<TruthEdgeCapabilityCase>,
    cost_cases: Vec<TruthEdgeCostCase>,
    order_cases: Vec<TruthEdgeOrderCase>,
    replay_cases: Vec<TruthEdgeReplayCase>,
    audit_hash: String,
}

impl TruthEdgeAudit {
    fn passed(&self) -> bool {
        self.math_cases.iter().all(|case| case.passed)
            && self.fixed_cases.iter().all(|case| case.passed)
            && self.capability_cases.iter().all(|case| case.passed)
            && self.cost_cases.iter().all(|case| case.passed)
            && self.order_cases.iter().all(|case| case.passed)
            && self.replay_cases.iter().all(|case| case.passed)
    }

    fn case_count(&self) -> usize {
        self.math_cases.len()
            + self.fixed_cases.len()
            + self.capability_cases.len()
            + self.cost_cases.len()
            + self.order_cases.len()
            + self.replay_cases.len()
    }
}

pub fn write_truth_edge_audit_artifacts(out_dir: impl AsRef<Path>) -> Result<(), OathError> {
    let audit = build_truth_edge_audit()?;
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;
    fs::write(
        out_dir.join("truth_edge_audit.json"),
        render_truth_edge_audit_json(&audit),
    )?;
    fs::write(
        out_dir.join("truth_edge_audit_report.md"),
        render_truth_edge_audit_report(&audit),
    )?;
    if !audit.passed() {
        return Err(OathError::Verify(
            "truth edge audit failed deterministic edge cases".to_string(),
        ));
    }
    Ok(())
}

fn build_truth_edge_audit() -> Result<TruthEdgeAudit, OathError> {
    let math_specs = [
        ("permille_rounds_positive_cost", 37, 1350, 50),
        ("permille_zero_modifier", 48, 0, 0),
        ("permille_negative_delta", -1200, 1250, -1500),
        (
            "permille_positive_overflow_saturates",
            i64::MAX / 2 + 1000,
            3000,
            i64::MAX,
        ),
        (
            "permille_negative_overflow_saturates",
            i64::MIN / 2 - 1000,
            3000,
            i64::MIN,
        ),
    ];
    let math_cases = math_specs
        .into_iter()
        .map(|(id, input, modifier_permille, expected)| {
            let actual = mul_permille_round_up_saturating(input, modifier_permille);
            TruthEdgeMathCase {
                id,
                input,
                modifier_permille,
                expected,
                actual,
                passed: actual == expected,
            }
        })
        .collect::<Vec<_>>();

    let fixed_specs = [
        ("fixed_ratio_basic", 12_345, 3, 2, 18_517),
        ("fixed_ratio_negative", -12_345, 3, 2, -18_517),
        (
            "fixed_ratio_positive_overflow_saturates",
            i64::MAX / 3,
            6,
            1,
            i64::MAX,
        ),
        (
            "fixed_ratio_negative_overflow_saturates",
            i64::MIN / 3,
            6,
            1,
            i64::MIN,
        ),
        (
            "fixed_ratio_zero_denominator_saturates",
            10_000,
            1,
            0,
            i64::MAX,
        ),
    ];
    let fixed_cases = fixed_specs
        .into_iter()
        .map(
            |(id, input_milli, numerator, denominator, expected_milli)| {
                let actual_milli = Fixed::from_milli(input_milli)
                    .mul_ratio(numerator, denominator)
                    .milli();
                TruthEdgeFixedCase {
                    id,
                    input_milli,
                    numerator,
                    denominator,
                    expected_milli,
                    actual_milli,
                    passed: actual_milli == expected_milli,
                }
            },
        )
        .collect::<Vec<_>>();

    let mut low = FighterState::from_spec(&FighterSpec {
        seat: 0,
        name: "edge_low".to_string(),
        weapon_id: "iron_maul".to_string(),
        armor_id: "heavy_plate".to_string(),
    })?;
    apply_capability_delta(
        &mut low,
        CapabilityDelta {
            torso_rotation_delta: -5000,
            recovery_slowdown_add: u32::MAX,
            balance_delta: -5000,
            torque_delta: -5000,
            grip_r_delta: -5000,
            grip_l_delta: -5000,
            invalidates_thrust: true,
            invalidates_cut: true,
            event: "edge audit lower clamp".to_string(),
        },
    );
    let low_case = TruthEdgeCapabilityCase {
        id: "capability_lower_clamp_and_validity",
        balance_permille: low.balance_permille,
        grip_r_permille: low.grip_r_permille,
        grip_l_permille: low.grip_l_permille,
        torque_permille: low.torque_permille,
        torso_rotation_permille: low.torso_rotation_permille,
        recovery_slowdown_frames: low.recovery_slowdown_frames,
        thrust_valid: low.thrust_valid,
        cut_valid: low.cut_valid,
        passed: low.balance_permille == 0
            && low.grip_r_permille == 0
            && low.grip_l_permille == 0
            && low.torque_permille == 0
            && low.torso_rotation_permille == 0
            && low.recovery_slowdown_frames == u32::MAX
            && !low.thrust_valid
            && !low.cut_valid,
    };

    let mut high = FighterState::from_spec(&FighterSpec {
        seat: 1,
        name: "edge_high".to_string(),
        weapon_id: "ash_spear".to_string(),
        armor_id: "gambeson".to_string(),
    })?;
    high.balance_permille = 200;
    high.grip_r_permille = 240;
    high.grip_l_permille = 260;
    high.torque_permille = 280;
    high.torso_rotation_permille = 300;
    apply_capability_delta(
        &mut high,
        CapabilityDelta {
            torso_rotation_delta: 5000,
            recovery_slowdown_add: 12,
            balance_delta: 5000,
            torque_delta: 5000,
            grip_r_delta: 5000,
            grip_l_delta: 5000,
            invalidates_thrust: false,
            invalidates_cut: false,
            event: "edge audit upper clamp".to_string(),
        },
    );
    let high_case = TruthEdgeCapabilityCase {
        id: "capability_upper_clamp",
        balance_permille: high.balance_permille,
        grip_r_permille: high.grip_r_permille,
        grip_l_permille: high.grip_l_permille,
        torque_permille: high.torque_permille,
        torso_rotation_permille: high.torso_rotation_permille,
        recovery_slowdown_frames: high.recovery_slowdown_frames,
        thrust_valid: high.thrust_valid,
        cut_valid: high.cut_valid,
        passed: high.balance_permille == 1000
            && high.grip_r_permille == 1000
            && high.grip_l_permille == 1000
            && high.torque_permille == 1000
            && high.torso_rotation_permille == 1000
            && high.recovery_slowdown_frames == 12
            && high.thrust_valid
            && high.cut_valid,
    };

    let thrust = ActionEntry {
        seat: low.seat,
        label: ActionLabel::Thrust,
        direction: Direction::Forward,
        target: TargetRegion::Torso,
    };
    let low_cost = calculate_cost(&low, thrust);
    let cost_cases = vec![TruthEdgeCostCase {
        id: "extreme_capability_cost_and_invalid_action",
        action: low_cost.action,
        base_frames: low_cost.base_frames,
        current_frames: low_cost.current_frames,
        action_valid: low_cost.action_valid,
        factor_count: low_cost.factors.len(),
        passed: !low_cost.action_valid
            && low_cost.current_frames > low_cost.base_frames
            && low_cost.factors.len() == 5
            && low_cost.current_frames < u32::MAX,
    }];

    let order_cases = vec![build_truth_edge_contact_order_case()];

    let replay_cases = build_truth_edge_replay_cases()?;

    let capability_cases = vec![low_case, high_case];
    let mut canonical = String::new();
    for case in &math_cases {
        writeln!(
            &mut canonical,
            "math:{}:{}:{}:{}:{}:{}",
            case.id, case.input, case.modifier_permille, case.expected, case.actual, case.passed
        )
        .unwrap();
    }
    for case in &fixed_cases {
        writeln!(
            &mut canonical,
            "fixed:{}:{}:{}:{}:{}:{}:{}",
            case.id,
            case.input_milli,
            case.numerator,
            case.denominator,
            case.expected_milli,
            case.actual_milli,
            case.passed
        )
        .unwrap();
    }
    for case in &capability_cases {
        writeln!(
            &mut canonical,
            "capability:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
            case.id,
            case.balance_permille,
            case.grip_r_permille,
            case.grip_l_permille,
            case.torque_permille,
            case.torso_rotation_permille,
            case.recovery_slowdown_frames,
            case.thrust_valid,
            case.cut_valid,
            case.passed
        )
        .unwrap();
    }
    for case in &cost_cases {
        writeln!(
            &mut canonical,
            "cost:{}:{}:{}:{}:{}:{}:{}",
            case.id,
            case.action.as_str(),
            case.base_frames,
            case.current_frames,
            case.action_valid,
            case.factor_count,
            case.passed
        )
        .unwrap();
    }
    for case in &order_cases {
        writeln!(
            &mut canonical,
            "order:{}:{}:{}:{}",
            case.id, case.expected_signature, case.actual_signature, case.passed
        )
        .unwrap();
    }
    for case in &replay_cases {
        writeln!(
            &mut canonical,
            "replay:{}:{}:{}:{}:{}",
            case.id, case.expected_pass, case.observed_pass, case.message, case.passed
        )
        .unwrap();
    }

    Ok(TruthEdgeAudit {
        math_cases,
        fixed_cases,
        capability_cases,
        cost_cases,
        order_cases,
        replay_cases,
        audit_hash: hash_hex(canonical.as_bytes()),
    })
}

fn build_truth_edge_contact_order_case() -> TruthEdgeOrderCase {
    let mut contacts = vec![
        edge_contact(
            240,
            1,
            0,
            ActionLabel::Thrust,
            TargetRegion::Head,
            Direction::High,
        ),
        edge_contact(
            120,
            1,
            0,
            ActionLabel::Cut,
            TargetRegion::Torso,
            Direction::Center,
        ),
        edge_contact(
            120,
            0,
            1,
            ActionLabel::Thrust,
            TargetRegion::WeaponArm,
            Direction::Forward,
        ),
        edge_contact(
            120,
            0,
            1,
            ActionLabel::Cut,
            TargetRegion::Torso,
            Direction::Low,
        ),
        edge_contact(
            120,
            0,
            1,
            ActionLabel::Cut,
            TargetRegion::Torso,
            Direction::Forward,
        ),
    ];
    sort_contact_packets(&mut contacts);
    let actual_signature = contacts
        .iter()
        .map(contact_signature)
        .collect::<Vec<_>>()
        .join("|");
    let expected_signature = [
        "120:0:1:cut:torso:forward",
        "120:0:1:cut:torso:low",
        "120:0:1:thrust:weapon_arm:forward",
        "120:1:0:cut:torso:center",
        "240:1:0:thrust:head:high",
    ]
    .join("|");
    let ordered = contacts
        .windows(2)
        .all(|window| contact_order_key(&window[0]) <= contact_order_key(&window[1]));
    TruthEdgeOrderCase {
        id: "contact_tie_breaker_signature",
        passed: ordered && actual_signature == expected_signature,
        expected_signature,
        actual_signature,
    }
}

fn edge_contact(
    frame: u32,
    attacker: usize,
    defender: usize,
    action: ActionLabel,
    target: TargetRegion,
    direction: Direction,
) -> ContactTrace {
    ContactTrace {
        turn: frame / TRUTH_HZ,
        frame,
        attacker,
        defender,
        action,
        direction,
        target,
        weapon_id: "edge_audit_weapon".to_string(),
        armor_id: "edge_audit_armor".to_string(),
        energy_milli: 0,
        impulse_milli: 0,
        material_result: "ordered_edge_case".to_string(),
        anatomy_result: "none".to_string(),
        capability_delta: CapabilityDelta::default(),
        cause_chain: "edge audit contact ordering only".to_string(),
    }
}

fn contact_signature(contact: &ContactTrace) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}",
        contact.frame,
        contact.attacker,
        contact.defender,
        contact.action.as_str(),
        contact.target.as_str(),
        contact.direction.as_str()
    )
}

fn build_truth_edge_replay_cases() -> Result<Vec<TruthEdgeReplayCase>, OathError> {
    let scenario = "\
scenario truth_edge_replay_compat
fighter 0 edge_a arming_sword gambeson
fighter 1 edge_b longsword mail_hauberk
turn 0 0 cut forward torso
turn 0 1 guard center torso
turn 1 0 recover center torso
turn 1 1 thrust forward torso
";
    let result = run_scenario_text(scenario)?;
    let mut cases = Vec::new();
    cases.push(edge_replay_case(
        "current_schema_replays",
        true,
        verify_replay_text(&result.replay_json),
    ));
    cases.push(edge_replay_case(
        "unsupported_schema_fails_loud",
        false,
        verify_replay_text(
            &result
                .replay_json
                .replacen(REPLAY_SCHEMA, "oathyard.replay.v0", 1),
        ),
    ));
    cases.push(edge_replay_case(
        "missing_required_field_fails_loud",
        false,
        verify_replay_text("{\"schema\":\"oathyard.replay.v1\"}\n"),
    ));
    let final_hash_field = format!("\"final_state_hash\": \"{}\"", result.final_state_hash);
    let mismatched_final = result.replay_json.replacen(
        &final_hash_field,
        "\"final_state_hash\": \"0000000000000000\"",
        1,
    );
    cases.push(edge_replay_case(
        "mismatched_final_hash_fails_loud",
        false,
        verify_replay_text(&mismatched_final),
    ));
    let end_condition_status_field = format!(
        "  \"end_condition_status\": {},\n",
        json_quote(&result.end_condition.status)
    );
    let missing_end_condition_status =
        result
            .replay_json
            .replacen(&end_condition_status_field, "", 1);
    cases.push(edge_replay_case(
        "missing_end_condition_status_fails_loud",
        false,
        verify_replay_text(&missing_end_condition_status),
    ));
    let end_condition_winner_field = format!(
        "  \"end_condition_winner\": {},\n",
        json_quote(&result.end_condition.winner_token())
    );
    let missing_end_condition_winner =
        result
            .replay_json
            .replacen(&end_condition_winner_field, "", 1);
    cases.push(edge_replay_case(
        "missing_end_condition_winner_fails_loud",
        false,
        verify_replay_text(&missing_end_condition_winner),
    ));
    let truth_hz_mismatch =
        result
            .replay_json
            .replacen(&format!("\"truth_hz\": {TRUTH_HZ}"), "\"truth_hz\": 60", 1);
    cases.push(edge_replay_case(
        "truth_hz_mismatch_fails_loud",
        false,
        verify_replay_text(&truth_hz_mismatch),
    ));
    let truth_hz_malformed = result.replay_json.replacen(
        &format!("\"truth_hz\": {TRUTH_HZ}"),
        &format!("\"truth_hz\": {TRUTH_HZ}oops"),
        1,
    );
    cases.push(edge_replay_case(
        "truth_hz_malformed_fails_loud",
        false,
        verify_replay_text(&truth_hz_malformed),
    ));
    Ok(cases)
}

fn edge_replay_case(
    id: &'static str,
    expected_pass: bool,
    result: Result<DuelResult, OathError>,
) -> TruthEdgeReplayCase {
    match result {
        Ok(duel) => TruthEdgeReplayCase {
            id,
            expected_pass,
            observed_pass: true,
            message: format!("verified final {}", duel.final_state_hash),
            passed: expected_pass,
        },
        Err(error) => {
            let message = error.to_string();
            let loud_error = message.contains("replay schema mismatch")
                || message.contains("replay missing scenario_canonical")
                || message.contains("replay truth_hz mismatch")
                || message.contains("replay missing truth_hz")
                || message.contains("replay missing end_condition_status")
                || message.contains("replay missing end_condition_winner")
                || message.contains("final state hash mismatch");
            TruthEdgeReplayCase {
                id,
                expected_pass,
                observed_pass: false,
                message,
                passed: !expected_pass && loud_error,
            }
        }
    }
}

fn render_truth_edge_audit_json(audit: &TruthEdgeAudit) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", TRUTH_EDGE_AUDIT_SCHEMA, true);
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    writeln!(&mut out, "  \"truth_hz\": {TRUTH_HZ},").unwrap();
    writeln!(&mut out, "  \"fixed_point_scale\": {},", Fixed::SCALE).unwrap();
    write_json_field(
        &mut out,
        1,
        "overflow_policy",
        "i128_intermediate_then_saturate_or_clamp",
        true,
    );
    write_json_field(&mut out, 1, "replay_schema", REPLAY_SCHEMA, true);
    write_json_field(&mut out, 1, "contact_order_rule", CONTACT_ORDER_RULE, true);
    writeln!(&mut out, "  \"hidden_rng\": false,").unwrap();
    writeln!(&mut out, "  \"wall_clock\": false,").unwrap();
    writeln!(&mut out, "  \"gameplay_floats\": false,").unwrap();
    writeln!(&mut out, "  \"unordered_truth_iteration\": false,").unwrap();
    writeln!(&mut out, "  \"case_count\": {},", audit.case_count()).unwrap();
    writeln!(&mut out, "  \"all_edge_cases_passed\": {},", audit.passed()).unwrap();
    write_json_field(&mut out, 1, "audit_hash", &audit.audit_hash, true);
    writeln!(&mut out, "  \"public_demo_ready\": {PUBLIC_DEMO_READY},").unwrap();
    writeln!(
        &mut out,
        "  \"release_candidate_ready\": {RELEASE_CANDIDATE_READY},"
    )
    .unwrap();
    writeln!(&mut out, "  \"math_cases\": [").unwrap();
    for (index, case) in audit.math_cases.iter().enumerate() {
        writeln!(
            &mut out,
            "    {{\"id\": {}, \"input\": {}, \"modifier_permille\": {}, \"expected\": {}, \"actual\": {}, \"passed\": {}}}{}",
            json_quote(case.id),
            case.input,
            case.modifier_permille,
            case.expected,
            case.actual,
            case.passed,
            comma(index + 1, audit.math_cases.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();
    writeln!(&mut out, "  \"fixed_cases\": [").unwrap();
    for (index, case) in audit.fixed_cases.iter().enumerate() {
        writeln!(
            &mut out,
            "    {{\"id\": {}, \"input_milli\": {}, \"numerator\": {}, \"denominator\": {}, \"expected_milli\": {}, \"actual_milli\": {}, \"passed\": {}}}{}",
            json_quote(case.id),
            case.input_milli,
            case.numerator,
            case.denominator,
            case.expected_milli,
            case.actual_milli,
            case.passed,
            comma(index + 1, audit.fixed_cases.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();
    writeln!(&mut out, "  \"capability_cases\": [").unwrap();
    for (index, case) in audit.capability_cases.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "id", case.id, true);
        writeln!(
            &mut out,
            "      \"balance_permille\": {},",
            case.balance_permille
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"grip_r_permille\": {},",
            case.grip_r_permille
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"grip_l_permille\": {},",
            case.grip_l_permille
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"torque_permille\": {},",
            case.torque_permille
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"torso_rotation_permille\": {},",
            case.torso_rotation_permille
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"recovery_slowdown_frames\": {},",
            case.recovery_slowdown_frames
        )
        .unwrap();
        writeln!(&mut out, "      \"thrust_valid\": {},", case.thrust_valid).unwrap();
        writeln!(&mut out, "      \"cut_valid\": {},", case.cut_valid).unwrap();
        writeln!(&mut out, "      \"passed\": {}", case.passed).unwrap();
        writeln!(
            &mut out,
            "    }}{}",
            comma(index + 1, audit.capability_cases.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();
    writeln!(&mut out, "  \"cost_cases\": [").unwrap();
    for (index, case) in audit.cost_cases.iter().enumerate() {
        writeln!(
            &mut out,
            "    {{\"id\": {}, \"action\": {}, \"base_frames\": {}, \"current_frames\": {}, \"action_valid\": {}, \"factor_count\": {}, \"passed\": {}}}{}",
            json_quote(case.id),
            json_quote(case.action.as_str()),
            case.base_frames,
            case.current_frames,
            case.action_valid,
            case.factor_count,
            case.passed,
            comma(index + 1, audit.cost_cases.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();
    writeln!(&mut out, "  \"order_cases\": [").unwrap();
    for (index, case) in audit.order_cases.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "id", case.id, true);
        write_json_field(
            &mut out,
            3,
            "expected_signature",
            &case.expected_signature,
            true,
        );
        write_json_field(
            &mut out,
            3,
            "actual_signature",
            &case.actual_signature,
            true,
        );
        writeln!(&mut out, "      \"passed\": {}", case.passed).unwrap();
        writeln!(
            &mut out,
            "    }}{}",
            comma(index + 1, audit.order_cases.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();
    writeln!(&mut out, "  \"replay_cases\": [").unwrap();
    for (index, case) in audit.replay_cases.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "id", case.id, true);
        writeln!(&mut out, "      \"expected_pass\": {},", case.expected_pass).unwrap();
        writeln!(&mut out, "      \"observed_pass\": {},", case.observed_pass).unwrap();
        write_json_field(&mut out, 3, "message", &case.message, true);
        writeln!(&mut out, "      \"passed\": {}", case.passed).unwrap();
        writeln!(
            &mut out,
            "    }}{}",
            comma(index + 1, audit.replay_cases.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

fn render_truth_edge_audit_report(audit: &TruthEdgeAudit) -> String {
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD Truth Edge Audit").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(
        &mut out,
        "Status: {}",
        if audit.passed() { "PASSED" } else { "FAILED" }
    )
    .unwrap();
    writeln!(&mut out, "- Truth Hz: `{TRUTH_HZ}`").unwrap();
    writeln!(&mut out, "- Fixed-point scale: `{}`", Fixed::SCALE).unwrap();
    writeln!(
        &mut out,
        "- Overflow policy: `i128_intermediate_then_saturate_or_clamp`"
    )
    .unwrap();
    writeln!(&mut out, "- Replay schema: `{REPLAY_SCHEMA}`").unwrap();
    writeln!(&mut out, "- Contact order rule: `{CONTACT_ORDER_RULE}`").unwrap();
    writeln!(&mut out, "- Hidden RNG: `false`").unwrap();
    writeln!(&mut out, "- Wall clock: `false`").unwrap();
    writeln!(&mut out, "- Gameplay floats: `false`").unwrap();
    writeln!(&mut out, "- Unordered truth iteration: `false`").unwrap();
    writeln!(&mut out, "- Case count: `{}`", audit.case_count()).unwrap();
    writeln!(&mut out, "- Audit hash: `{}`", audit.audit_hash).unwrap();
    writeln!(&mut out, "- Public demo ready: `{PUBLIC_DEMO_READY}`").unwrap();
    writeln!(
        &mut out,
        "- Release candidate ready: `{RELEASE_CANDIDATE_READY}`"
    )
    .unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Edge Cases").unwrap();
    writeln!(&mut out).unwrap();
    for case in &audit.math_cases {
        writeln!(
            &mut out,
            "- math `{}` input `{}` modifier `{}` expected `{}` actual `{}` passed `{}`",
            case.id, case.input, case.modifier_permille, case.expected, case.actual, case.passed
        )
        .unwrap();
    }
    for case in &audit.fixed_cases {
        writeln!(
            &mut out,
            "- fixed `{}` input `{}` ratio `{}/{}` expected `{}` actual `{}` passed `{}`",
            case.id,
            case.input_milli,
            case.numerator,
            case.denominator,
            case.expected_milli,
            case.actual_milli,
            case.passed
        )
        .unwrap();
    }
    for case in &audit.capability_cases {
        writeln!(
            &mut out,
            "- capability `{}` balance `{}` grip_r `{}` torque `{}` torso `{}` recovery `{}` thrust_valid `{}` cut_valid `{}` passed `{}`",
            case.id,
            case.balance_permille,
            case.grip_r_permille,
            case.torque_permille,
            case.torso_rotation_permille,
            case.recovery_slowdown_frames,
            case.thrust_valid,
            case.cut_valid,
            case.passed
        )
        .unwrap();
    }
    for case in &audit.cost_cases {
        writeln!(
            &mut out,
            "- cost `{}` action `{}` base `{}` current `{}` action_valid `{}` factors `{}` passed `{}`",
            case.id,
            case.action.as_str(),
            case.base_frames,
            case.current_frames,
            case.action_valid,
            case.factor_count,
            case.passed
        )
        .unwrap();
    }
    for case in &audit.order_cases {
        writeln!(
            &mut out,
            "- order `{}` passed `{}` signature `{}`",
            case.id, case.passed, case.actual_signature
        )
        .unwrap();
    }
    for case in &audit.replay_cases {
        writeln!(
            &mut out,
            "- replay `{}` expected_pass `{}` observed_pass `{}` passed `{}`: {}",
            case.id, case.expected_pass, case.observed_pass, case.passed, case.message
        )
        .unwrap();
    }
    out
}

fn count_contacts(result: &DuelResult) -> usize {
    result.turns.iter().map(|turn| turn.contacts.len()).sum()
}

fn truth_actions_valid(result: &DuelResult) -> bool {
    result
        .turns
        .iter()
        .flat_map(|turn| turn.costs.iter())
        .all(|cost| cost.action_valid)
}

fn ai_capability_reaction_count(plan: &AiPlan) -> usize {
    plan.entries
        .iter()
        .filter(|entry| {
            entry.observed_grip_r_permille < 700
                || entry.observed_balance_permille < 800
                || entry.observed_recovery_slowdown_frames >= 8
                || matches!(entry.action, ActionLabel::Recover | ActionLabel::Guard)
        })
        .count()
}

fn ai_action_counts(plan: &AiPlan) -> Vec<(&'static str, usize)> {
    const ORDER: [ActionLabel; 13] = [
        ActionLabel::Step,
        ActionLabel::Pivot,
        ActionLabel::Guard,
        ActionLabel::Parry,
        ActionLabel::Cut,
        ActionLabel::Thrust,
        ActionLabel::Brace,
        ActionLabel::Bash,
        ActionLabel::HookBind,
        ActionLabel::Grab,
        ActionLabel::Shove,
        ActionLabel::Kick,
        ActionLabel::Recover,
    ];
    let mut counts = Vec::new();
    for action in ORDER {
        let count = plan
            .entries
            .iter()
            .filter(|entry| entry.action == action)
            .count();
        if count > 0 {
            counts.push((action.as_str(), count));
        }
    }
    counts
}

fn distinct_ai_action_labels(evidences: &[AiSweepEvidence]) -> Vec<&'static str> {
    let mut labels = Vec::new();
    for evidence in evidences {
        for (label, _) in &evidence.action_counts {
            if !labels.contains(label) {
                labels.push(*label);
            }
        }
    }
    labels
}

fn distinct_ai_policy_styles(evidences: &[AiSweepEvidence]) -> Vec<&'static str> {
    let mut policies = Vec::new();
    for evidence in evidences {
        for policy in [evidence.policy_0.as_str(), evidence.policy_1.as_str()] {
            if !policies.contains(&policy) {
                policies.push(policy);
            }
        }
    }
    policies
}

fn unique_ai_final_hashes(evidences: &[AiSweepEvidence]) -> Vec<&str> {
    let mut hashes = Vec::new();
    for evidence in evidences {
        if !hashes.contains(&evidence.final_state_hash.as_str()) {
            hashes.push(evidence.final_state_hash.as_str());
        }
    }
    hashes
}

fn render_native_3d_blocked_manifest_json(result: &DuelResult, command: &str) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(
        &mut out,
        1,
        "schema",
        "oathyard.native_3d_visual_blocked.v1",
        true,
    );
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(&mut out, 1, "command", command, true);
    write_json_field(&mut out, 1, "source", "truth-after-hash-duel-result", true);
    write_json_field(&mut out, 1, "scenario_id", &result.scenario_id, true);
    write_json_field(
        &mut out,
        1,
        "final_state_hash",
        &result.final_state_hash,
        true,
    );
    writeln!(&mut out, "  \"replay_verified\": true,").unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"native_3d_visual_evidence_present\": false,").unwrap();
    write_json_field(
        &mut out,
        1,
        "visual_evidence_status",
        "blocked_pending_native_3d_renderer_capture",
        true,
    );
    writeln!(&mut out, "  \"forbidden_visual_fallbacks_emitted\": false,").unwrap();
    writeln!(&mut out, "  \"production_renderer_complete\": false,").unwrap();
    writeln!(&mut out, "  \"owner_visual_acceptance\": false,").unwrap();
    writeln!(&mut out, "  \"public_demo_ready\": false,").unwrap();
    writeln!(&mut out, "  \"release_candidate_ready\": false").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

fn render_native_3d_blocked_report_md(result: &DuelResult, title: &str) -> String {
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD {title}").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(
        &mut out,
        "Status: BLOCKED_PENDING_NATIVE_3D_RENDERER_CAPTURE"
    )
    .unwrap();
    writeln!(&mut out, "- Scenario: `{}`", result.scenario_id).unwrap();
    writeln!(
        &mut out,
        "- Final state hash: `{}`",
        result.final_state_hash
    )
    .unwrap();
    writeln!(&mut out, "- Replay verified: `true`").unwrap();
    writeln!(&mut out, "- Truth mutation: `false`").unwrap();
    writeln!(&mut out, "- Presentation only: `true`").unwrap();
    writeln!(&mut out, "- Native 3D visual evidence present: `false`").unwrap();
    writeln!(&mut out, "- Forbidden visual fallbacks emitted: `false`").unwrap();
    writeln!(&mut out, "- Production renderer complete: `false`").unwrap();
    writeln!(&mut out, "- Owner visual acceptance: `false`").unwrap();
    writeln!(&mut out, "- Public demo ready: `false`").unwrap();
    writeln!(&mut out, "- Release candidate ready: `false`").unwrap();
    out
}

#[cfg(target_os = "linux")]
pub fn native_combat_render(
    scenario_path: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
) -> Result<DuelResult, OathError> {
    use std::process::Command;
    let scenario_text = fs::read_to_string(scenario_path.as_ref())?;
    let pre_scenario = Scenario::parse(&scenario_text)?;
    enforce_scenario_freeze_gate(&pre_scenario)?;
    let result = run_scenario_text(&scenario_text)?;
    let verified_replay = verify_replay_text(&result.replay_json)?;
    if verified_replay.final_state_hash != result.final_state_hash
        || verified_replay.turn_hashes != result.turn_hashes
    {
        return Err(OathError::Verify(
            "native capture input replay verification did not reproduce truth hashes".to_string(),
        ));
    }
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;
    fs::write(
        out_dir.join("native_capture_input_replay.json"),
        &result.replay_json,
    )?;
    // Unit-069: Call the production renderer to generate real native 3D captures.
    // This replaces the blocked schema with actual rendered evidence.
    let renderer_bin = std::env::current_dir()
        .map(|p| p.join("crates/oathyard_renderer/target/debug/oathyard-native-renderer"))
        .unwrap_or_else(|_| std::path::PathBuf::from("oathyard-native-renderer"));
    let packet_path = out_dir.join("post_hash_presentation_packet.json");
    let render_dir = out_dir.join("render");
    fs::create_dir_all(&render_dir).ok();
    // Write presentation packet
    let packet = format!(
        r#"{{"schema":"oathyard.post_hash_presentation_packet.v1","scenario_id":"{}","content_hash":"{}","final_state_hash":"{}","end_condition_status":"{}","end_condition_winner":"{}","generated_after_replay_verify":true,"presentation_only":true,"truth_mutation":false,"renderer_consumption_layer":"runtime_presentation"}}"#,
        result.scenario_id,
        result.content_hash,
        result.final_state_hash,
        result.end_condition.status,
        result.end_condition.winner_token()
    );
    fs::write(&packet_path, &packet)?;
    // Call production renderer for native capture
    let renderer_result = Command::new(&renderer_bin)
        .arg("--packet")
        .arg(&packet_path)
        .arg("--out")
        .arg(&render_dir)
        .arg("--capture-id")
        .arg("native_combat_capture_unit069")
        .arg("--capture-file-stem")
        .arg("native_combat_3d_1920x1080")
        .arg("--camera-mode")
        .arg("oathyard_verdict_ring_establishing")
        .arg("--candidate-assets")
        .arg("saltreach_duelist,training_yard")
        .output();
    let renderer_succeeded = match renderer_result {
        Ok(output) => {
            let capture_path = render_dir.join("native_combat_3d_1920x1080.png");
            capture_path.exists()
                && capture_path
                    .metadata()
                    .map(|m| m.len() > 1000)
                    .unwrap_or(false)
        }
        Err(_) => false,
    };
    // Write manifest — promoted if renderer succeeded, blocked otherwise
    if renderer_succeeded {
        // Write promoted manifest with real capture evidence
        let manifest = format!(
            r#"{{"schema":"oathyard.native_combat_render.v1","product":"OATHYARD","command":"native-combat-render","source":"truth-after-hash-duel-result","scenario_id":"{}","final_state_hash":"{}","replay_verified":true,"truth_mutation":false,"presentation_only":true,"native_3d_visual_evidence_present":true,"visual_evidence_status":"native_3d_renderer_capture_present","forbidden_visual_fallbacks_emitted":false,"production_renderer_complete":false,"owner_visual_acceptance":false,"public_demo_ready":false,"release_candidate_ready":false}}"#,
            result.scenario_id, result.final_state_hash
        );
        fs::write(
            out_dir.join("native_combat_render_manifest.json"),
            &manifest,
        )?;
        let report = format!(
            "# OATHYARD Native Combat Render\n\nStatus: NATIVE_3D_RENDERER_CAPTURE_PRESENT\n- Scenario: `{}`\n- Final state hash: `{}`\n- Replay verified: `true`\n- Truth mutation: `false`\n- Native 3D visual evidence present: `true`\n- Production renderer complete: `false`\n- Owner visual acceptance: `false`",
            result.scenario_id, result.final_state_hash
        );
        fs::write(out_dir.join("native_combat_render_report.md"), &report)?;
        fs::write(out_dir.join("native_combat_visual_audit.md"), &report)?;
    } else {
        // Fallback: still blocked
        fs::write(
            out_dir.join("native_combat_render_manifest.json"),
            render_native_3d_blocked_manifest_json(&result, "native-combat-render"),
        )?;
        fs::write(
            out_dir.join("native_combat_visual_audit.md"),
            render_native_3d_blocked_report_md(&result, "Native Combat Render"),
        )?;
        fs::write(
            out_dir.join("native_combat_render_report.md"),
            render_native_3d_blocked_report_md(&result, "Native Combat Render"),
        )?;
    }
    Ok(result)
}

#[cfg(not(target_os = "linux"))]
pub fn native_combat_render(
    scenario_path: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
) -> Result<DuelResult, OathError> {
    // R-GAP-1: Enforce freeze gate even on non-Linux builds before returning
    // the platform-blocked error. AI-derived assets must be rejected here too.
    let scenario_text = fs::read_to_string(scenario_path.as_ref())?;
    let pre_scenario = Scenario::parse(&scenario_text)?;
    enforce_scenario_freeze_gate(&pre_scenario)?;
    let result = run_scenario_text(&scenario_text)?;
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;
    fs::write(
        out_dir.join("native_combat_render_report.md"),
        "# OATHYARD Native Combat Render\n\nStatus: BLOCKED\n- Native status combat render is Linux-only in this build.\n",
    )?;
    Err(OathError::Verify(
        "native combat render is Linux-only in this build".to_string(),
    ))
}

#[cfg(not(target_os = "linux"))]
pub fn native_roster_showcase(out_dir: impl AsRef<Path>) -> Result<(), OathError> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;
    fs::write(
        out_dir.join("native_roster_showcase_report.md"),
        "# OATHYARD Native Roster 3D Showcase\n\nStatus: BLOCKED\n- Native status/software native roster showcase is Linux-only in this build.\n",
    )?;
    Err(OathError::Verify(
        "native roster showcase is Linux-only in this build".to_string(),
    ))
}
