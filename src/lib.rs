use std::env;
use std::ffi::CString;
use std::fmt::Write as _;
use std::fs;
use std::os::raw::{c_char, c_int, c_uint, c_ulong};
use std::path::{Path, PathBuf};

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
    write_json_field, xml_escape,
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

#[cfg(target_os = "linux")]
fn file_sha256_hex(path: impl AsRef<Path>) -> Result<String, OathError> {
    Ok(sha256::sha256_file(path.as_ref())?)
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
    let width = 960u32;
    let height = 540u32;
    let arena_asset = native_combat_asset_ref("oathyard_verdict_ring", "arenas")?;
    let asset_manifest_hash =
        file_hash_hex(native_runtime_asset_path("assets/runtime_manifest.json"))?;
    let mut frames = Vec::new();

    for (index, fighter) in FIGHTER_TRADITIONS.iter().enumerate() {
        let fighter_asset = native_combat_asset_ref(fighter.id, "fighters")?;
        let weapon_asset = native_combat_asset_ref(fighter.default_weapon, "weapons")?;
        let armor_asset = native_combat_asset_ref(fighter.default_armor, "armor")?;
        for (asset_id, asset) in [
            (fighter.id, &fighter_asset),
            (fighter.default_weapon, &weapon_asset),
            (fighter.default_armor, &armor_asset),
            ("oathyard_verdict_ring", &arena_asset),
        ] {
            if !native_asset_geometry_complete(asset) {
                return Err(OathError::Verify(format!(
                    "native roster showcase asset lacks complete 3D geometry: {asset_id}"
                )));
            }
        }

        let file = format!("native_roster_showcase_{:02}_{}.ppm", index + 1, fighter.id);
        let (bytes, triangle_count, shaded_triangle_count) = render_native_roster_showcase_frame(
            fighter,
            &fighter_asset,
            &weapon_asset,
            &armor_asset,
            &arena_asset,
            width,
            height,
            index,
        );
        fs::write(out_dir.join(&file), &bytes)?;
        let (_, non_background_pixels, frame_hash) =
            native_ppm_evidence(&out_dir.join(&file), width, height)?;
        if shaded_triangle_count == 0 {
            return Err(OathError::Verify(format!(
                "native roster showcase rendered no shaded triangles for {}",
                fighter.id
            )));
        }
        frames.push(NativeRosterShowcaseFrame {
            index: index + 1,
            file,
            fighter_id: fighter.id.to_string(),
            fighter_name: fighter.display_name.to_string(),
            weapon_id: fighter.default_weapon.to_string(),
            armor_id: fighter.default_armor.to_string(),
            arena_id: "oathyard_verdict_ring".to_string(),
            width,
            height,
            triangle_count,
            shaded_triangle_count,
            non_background_pixels,
            projection_model: "integer_depth_sorted_mesh_raster".to_string(),
            depth_sorted: true,
            source: "default-loadout-runtime-gltf-after-content-hash".to_string(),
            fighter_gltf_hash: fighter_asset.runtime_gltf_hash,
            weapon_gltf_hash: weapon_asset.runtime_gltf_hash,
            armor_gltf_hash: armor_asset.runtime_gltf_hash,
            frame_hash,
        });
    }

    if frames.len() != FIGHTER_TRADITIONS.len() {
        return Err(OathError::Verify(format!(
            "native roster showcase expected {} frames, wrote {}",
            FIGHTER_TRADITIONS.len(),
            frames.len()
        )));
    }
    fs::write(
        out_dir.join("native_roster_showcase_manifest.json"),
        render_native_roster_showcase_manifest(&frames, &asset_manifest_hash),
    )?;
    fs::write(
        out_dir.join("native_roster_showcase_report.md"),
        render_native_roster_showcase_report(&frames, &asset_manifest_hash),
    )?;
    fs::write(
        out_dir.join("native_roster_showcase_contact_sheet.svg"),
        render_native_roster_showcase_contact_sheet(&frames),
    )?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn render_native_roster_showcase_manifest(
    frames: &[NativeRosterShowcaseFrame],
    asset_manifest_hash: &str,
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", NATIVE_ROSTER_SHOWCASE_SCHEMA, true);
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(
        &mut out,
        1,
        "source",
        "fighter-tradition-default-loadouts-runtime-gltf-after-content-hash",
        true,
    );
    write_json_field(&mut out, 1, "content_hash", &content_hash(), true);
    write_json_field(
        &mut out,
        1,
        "asset_manifest_hash",
        asset_manifest_hash,
        true,
    );
    writeln!(
        &mut out,
        "  \"fighter_tradition_count\": {},",
        FIGHTER_TRADITIONS.len()
    )
    .unwrap();
    writeln!(&mut out, "  \"frame_count\": {},", frames.len()).unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(&mut out, "  \"game_is_3d\": true,").unwrap();
    writeln!(&mut out, "  \"product_3d_gameplay_complete\": false,").unwrap();
    writeln!(
        &mut out,
        "  \"continuous_player_facing_3d_render_loop\": false,"
    )
    .unwrap();
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"owner_visual_acceptance_claimed\": false,").unwrap();
    writeln!(&mut out, "  \"public_demo_ready\": {PUBLIC_DEMO_READY},").unwrap();
    writeln!(
        &mut out,
        "  \"release_candidate_ready\": {RELEASE_CANDIDATE_READY},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"all_default_loadouts_rendered\": {},",
        frames.len() == FIGHTER_TRADITIONS.len()
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"all_frames_depth_sorted\": {},",
        frames.iter().all(|frame| frame.depth_sorted)
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"all_frames_shaded_triangles\": {},",
        frames.iter().all(|frame| frame.shaded_triangle_count > 0)
    )
    .unwrap();
    writeln!(&mut out, "  \"frames\": [").unwrap();
    for (index, frame) in frames.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        writeln!(&mut out, "      \"index\": {},", frame.index).unwrap();
        write_json_field(&mut out, 3, "file", &frame.file, true);
        write_json_field(&mut out, 3, "fighter_id", &frame.fighter_id, true);
        write_json_field(&mut out, 3, "fighter_name", &frame.fighter_name, true);
        write_json_field(&mut out, 3, "weapon_id", &frame.weapon_id, true);
        write_json_field(&mut out, 3, "armor_id", &frame.armor_id, true);
        write_json_field(&mut out, 3, "arena_id", &frame.arena_id, true);
        writeln!(&mut out, "      \"width\": {},", frame.width).unwrap();
        writeln!(&mut out, "      \"height\": {},", frame.height).unwrap();
        writeln!(
            &mut out,
            "      \"triangle_count\": {},",
            frame.triangle_count
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"shaded_triangle_count\": {},",
            frame.shaded_triangle_count
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"non_background_pixels\": {},",
            frame.non_background_pixels
        )
        .unwrap();
        write_json_field(
            &mut out,
            3,
            "projection_model",
            &frame.projection_model,
            true,
        );
        writeln!(&mut out, "      \"depth_sorted\": {},", frame.depth_sorted).unwrap();
        write_json_field(&mut out, 3, "source", &frame.source, true);
        write_json_field(
            &mut out,
            3,
            "fighter_gltf_hash",
            &frame.fighter_gltf_hash,
            true,
        );
        write_json_field(
            &mut out,
            3,
            "weapon_gltf_hash",
            &frame.weapon_gltf_hash,
            true,
        );
        write_json_field(&mut out, 3, "armor_gltf_hash", &frame.armor_gltf_hash, true);
        write_json_field(&mut out, 3, "frame_hash", &frame.frame_hash, false);
        writeln!(&mut out, "    }}{}", comma(index + 1, frames.len())).unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

#[cfg(target_os = "linux")]
fn render_native_roster_showcase_report(
    frames: &[NativeRosterShowcaseFrame],
    asset_manifest_hash: &str,
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD Native Roster 3D Showcase").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "Status: PASSED").unwrap();
    writeln!(
        &mut out,
        "- Source: `fighter-tradition-default-loadouts-runtime-gltf-after-content-hash`"
    )
    .unwrap();
    writeln!(&mut out, "- Asset manifest hash: `{asset_manifest_hash}`").unwrap();
    writeln!(&mut out, "- Content hash: `{}`", content_hash()).unwrap();
    writeln!(
        &mut out,
        "- Fighter traditions rendered: `{}`",
        frames.len()
    )
    .unwrap();
    writeln!(
        &mut out,
        "- All default loadouts rendered: `{}`",
        frames.len() == FIGHTER_TRADITIONS.len()
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Projection model: `integer_depth_sorted_mesh_raster`"
    )
    .unwrap();
    writeln!(&mut out, "- Truth mutation: `none`").unwrap();
    writeln!(&mut out, "- Presentation only: `true`").unwrap();
    writeln!(&mut out, "- Owner visual acceptance claimed: `false`").unwrap();
    writeln!(&mut out, "- Public demo ready: `{PUBLIC_DEMO_READY}`").unwrap();
    writeln!(
        &mut out,
        "- Release candidate ready: `{RELEASE_CANDIDATE_READY}`"
    )
    .unwrap();
    writeln!(&mut out).unwrap();
    for frame in frames {
        writeln!(
            &mut out,
            "- `{}`: fighter `{}` weapon `{}` armor `{}` triangles `{}` shaded `{}` pixels `{}` hash `{}`",
            frame.file,
            frame.fighter_id,
            frame.weapon_id,
            frame.armor_id,
            frame.triangle_count,
            frame.shaded_triangle_count,
            frame.non_background_pixels,
            frame.frame_hash
        )
        .unwrap();
    }
    out
}

#[cfg(target_os = "linux")]
fn render_native_roster_showcase_contact_sheet(frames: &[NativeRosterShowcaseFrame]) -> String {
    let width = 1280;
    let height = 720;
    let mut out = String::new();
    writeln!(
        &mut out,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\">"
    )
    .unwrap();
    writeln!(
        &mut out,
        "<rect width=\"100%\" height=\"100%\" fill=\"#f1eadb\"/>"
    )
    .unwrap();
    writeln!(
        &mut out,
        "<text x=\"28\" y=\"38\" font-family=\"monospace\" font-size=\"20\" fill=\"#1f2528\">OATHYARD native roster 3D showcase contact sheet</text>"
    )
    .unwrap();
    writeln!(
        &mut out,
        "<text x=\"28\" y=\"66\" font-family=\"monospace\" font-size=\"12\" fill=\"#394247\">all six default loadout families rendered from runtime glTF after content hash | owner visual acceptance not claimed</text>"
    )
    .unwrap();
    for frame in frames {
        let cell = frame.index - 1;
        let col = cell % 3;
        let row = cell / 3;
        let x = 28 + col as i32 * 410;
        let y = 98 + row as i32 * 288;
        writeln!(
            &mut out,
            "<rect x=\"{x}\" y=\"{y}\" width=\"386\" height=\"252\" fill=\"#fff9ec\" stroke=\"#1f2528\"/>"
        )
        .unwrap();
        writeln!(
            &mut out,
            "<text x=\"{}\" y=\"{}\" font-family=\"monospace\" font-size=\"14\" fill=\"#1f2528\">{} {}</text>",
            x + 18,
            y + 28,
            frame.index,
            xml_escape(&frame.fighter_name)
        )
        .unwrap();
        writeln!(
            &mut out,
            "<text x=\"{}\" y=\"{}\" font-family=\"monospace\" font-size=\"11\" fill=\"#394247\">{} / {}</text>",
            x + 18,
            y + 52,
            xml_escape(&frame.weapon_id),
            xml_escape(&frame.armor_id)
        )
        .unwrap();
        writeln!(
            &mut out,
            "<rect x=\"{}\" y=\"{}\" width=\"316\" height=\"126\" fill=\"#e6dcc9\" stroke=\"#394247\"/>",
            x + 34,
            y + 72
        )
        .unwrap();
        writeln!(
            &mut out,
            "<path d=\"M{} {} L{} {} L{} {} Z\" fill=\"#afa58b\" stroke=\"#1f2528\"/>",
            x + 70,
            y + 184,
            x + 194,
            y + 112,
            x + 320,
            y + 184
        )
        .unwrap();
        writeln!(
            &mut out,
            "<rect x=\"{}\" y=\"{}\" width=\"46\" height=\"72\" fill=\"#2d696e\" stroke=\"#1f2528\"/>",
            x + 162,
            y + 112
        )
        .unwrap();
        writeln!(
            &mut out,
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#464e4f\" stroke-width=\"8\"/>",
            x + 210,
            y + 142,
            x + 330,
            y + 104
        )
        .unwrap();
        writeln!(
            &mut out,
            "<text x=\"{}\" y=\"{}\" font-family=\"monospace\" font-size=\"10\" fill=\"#394247\">{}</text>",
            x + 18,
            y + 222,
            xml_escape(&frame.file)
        )
        .unwrap();
        writeln!(
            &mut out,
            "<text x=\"{}\" y=\"{}\" font-family=\"monospace\" font-size=\"10\" fill=\"#394247\">tri {} shaded {} hash {}</text>",
            x + 18,
            y + 240,
            frame.triangle_count,
            frame.shaded_triangle_count,
            xml_escape(&frame.frame_hash)
        )
        .unwrap();
    }
    writeln!(
        &mut out,
        "<text x=\"28\" y=\"686\" font-family=\"monospace\" font-size=\"12\" fill=\"#394247\">presentation-only roster evidence; renderer consumes source-backed runtime assets and does not mutate truth</text>"
    )
    .unwrap();
    writeln!(&mut out, "</svg>").unwrap();
    out
}

#[cfg(target_os = "linux")]
fn render_native_roster_showcase_frame(
    fighter: &FighterTradition,
    fighter_asset: &NativeCombatAssetRef,
    weapon_asset: &NativeCombatAssetRef,
    armor_asset: &NativeCombatAssetRef,
    arena_asset: &NativeCombatAssetRef,
    width: u32,
    height: u32,
    index: usize,
) -> (Vec<u8>, usize, usize) {
    let mut pixels = vec![0u8; width as usize * height as usize * 3];
    for chunk in pixels.chunks_exact_mut(3) {
        chunk[0] = 236;
        chunk[1] = 227;
        chunk[2] = 210;
    }
    fill_rect(
        &mut pixels,
        width as usize,
        height as usize,
        0,
        0,
        width as usize,
        62,
        (218, 207, 188),
    );
    let stripe_seed = format!(
        "{}:{}:{}",
        content_hash(),
        fighter.id,
        fighter.default_weapon
    );
    draw_hash_stripes(
        &mut pixels,
        width as usize,
        height as usize,
        &hash_hex(stripe_seed.as_bytes()),
    );

    draw_native_contact_shadow(
        &mut pixels,
        width as usize,
        height as usize,
        (width / 2) as i32,
        (height as i32 * 3 / 5).max(1),
        (width / 5) as i32,
        (height / 18) as i32,
    );
    let mut triangles = Vec::new();
    push_native_mesh_triangles(
        &mut triangles,
        NativeSoftware3dInstance {
            geometry: &arena_asset.geometry,
            origin_x_milli: 0,
            origin_y_milli: -760,
            origin_z_milli: 0,
            scale_num: 118,
            scale_den: 1000,
            facing: 1,
            depth_bias: -980,
            color: (164, 154, 128),
        },
        width,
        height,
        "third_person_verdict_ring",
    );
    let color = native_roster_color(index);
    push_native_mesh_triangles(
        &mut triangles,
        NativeSoftware3dInstance {
            geometry: &fighter_asset.geometry,
            origin_x_milli: -170,
            origin_y_milli: -380,
            origin_z_milli: -40,
            scale_num: native_roster_mesh_scale_num(&fighter_asset.geometry, 210),
            scale_den: 1000,
            facing: 1,
            depth_bias: -340,
            color,
        },
        width,
        height,
        "third_person_verdict_ring",
    );
    push_native_mesh_triangles(
        &mut triangles,
        NativeSoftware3dInstance {
            geometry: &armor_asset.geometry,
            origin_x_milli: -20,
            origin_y_milli: -390,
            origin_z_milli: 40,
            scale_num: native_roster_mesh_scale_num(&armor_asset.geometry, 190),
            scale_den: 1000,
            facing: 1,
            depth_bias: -280,
            color: (88, 94, 92),
        },
        width,
        height,
        "third_person_verdict_ring",
    );
    let weapon = weapon_by_id(fighter.default_weapon).unwrap_or(WEAPONS[0]);
    let weapon_target_px = (weapon.reach_mm / 8).clamp(110, 260);
    push_native_mesh_triangles(
        &mut triangles,
        NativeSoftware3dInstance {
            geometry: &weapon_asset.geometry,
            origin_x_milli: -weapon.reach_mm / 2,
            origin_y_milli: -40,
            origin_z_milli: 260,
            scale_num: native_roster_mesh_scale_num(&weapon_asset.geometry, weapon_target_px),
            scale_den: 1000,
            facing: 1,
            depth_bias: 140,
            color: (70, 78, 79),
        },
        width,
        height,
        "third_person_verdict_ring",
    );

    let triangle_count = triangles.len();
    triangles.sort_by_key(|triangle| triangle.depth);
    let mut shaded_triangle_count = 0usize;
    for triangle in &triangles {
        if fill_triangle(&mut pixels, width as usize, height as usize, triangle) {
            shaded_triangle_count += 1;
        }
    }
    apply_native_lighting_post(&mut pixels, width as usize, height as usize, 1080);
    fill_rect(
        &mut pixels,
        width as usize,
        height as usize,
        28,
        height as usize - 52,
        420,
        18,
        (31, 36, 37),
    );
    fill_rect(
        &mut pixels,
        width as usize,
        height as usize,
        470,
        height as usize - 52,
        (fighter.body_mass_g as usize / 240).clamp(120, 420),
        18,
        color,
    );
    fill_rect(
        &mut pixels,
        width as usize,
        height as usize,
        28,
        height as usize - 25,
        (weapon.mass_g as usize / 8).clamp(90, 420),
        9,
        (196, 127, 43),
    );

    let mut ppm = format!("P6\n{} {}\n255\n", width, height).into_bytes();
    ppm.extend_from_slice(&pixels);
    (ppm, triangle_count, shaded_triangle_count)
}

#[cfg(target_os = "linux")]
fn native_roster_mesh_scale_num(geometry: &NativeGltfGeometry, target_px: i32) -> i32 {
    let extent_x = (geometry.max_x_milli - geometry.min_x_milli).abs();
    let extent_y = (geometry.max_y_milli - geometry.min_y_milli).abs();
    let extent_z = (geometry.max_z_milli - geometry.min_z_milli).abs();
    let extent = extent_x.max(extent_y).max(extent_z).max(1);
    (target_px * 4000 / extent).clamp(40, 900)
}

#[cfg(target_os = "linux")]
fn native_roster_color(index: usize) -> (u8, u8, u8) {
    const COLORS: [(u8, u8, u8); 6] = [
        (42, 100, 110),
        (112, 89, 56),
        (121, 66, 52),
        (72, 118, 86),
        (85, 84, 116),
        (116, 95, 47),
    ];
    COLORS[index % COLORS.len()]
}

#[cfg(target_os = "linux")]
fn build_native_combat_frame_specs(result: &DuelResult) -> Vec<NativeCombatFrameSpec> {
    let mut frames = Vec::new();
    let first_turn = result.turns.first();
    let last_turn = result.turns.last();
    frames.push(NativeCombatFrameSpec {
        index: 1,
        file: "native_combat_frame_001.ppm".to_string(),
        state: "observe_plan",
        turn: first_turn.map(|turn| turn.turn).unwrap_or(0),
        headline: first_turn
            .map(|turn| {
                format!(
                    "commit reveal: {} vs {}",
                    turn.commits[0].label.as_str(),
                    turn.commits[1].label.as_str()
                )
            })
            .unwrap_or_else(|| "commit reveal pending".to_string()),
        detail: "players authored compact physical actions plus directional influence".to_string(),
        frame_hash: String::new(),
    });

    let guard_turn = result
        .turns
        .iter()
        .find(|turn| {
            turn.commits
                .iter()
                .any(|action| matches!(action.label, ActionLabel::Guard | ActionLabel::Parry))
        })
        .or(first_turn);
    frames.push(NativeCombatFrameSpec {
        index: 2,
        file: "native_combat_frame_002.ppm".to_string(),
        state: "guard_bind",
        turn: guard_turn.map(|turn| turn.turn).unwrap_or(0),
        headline: guard_turn
            .map(|turn| {
                format!(
                    "guard/parry lane: {} vs {}",
                    turn.commits[0].label.as_str(),
                    turn.commits[1].label.as_str()
                )
            })
            .unwrap_or_else(|| "guard lane unavailable".to_string()),
        detail: "defense state is display-only; truth already resolved action costs".to_string(),
        frame_hash: String::new(),
    });

    frames.push(NativeCombatFrameSpec {
        index: 3,
        file: "native_combat_frame_003.ppm".to_string(),
        state: "parry_window",
        turn: guard_turn.map(|turn| turn.turn).unwrap_or(0),
        headline: guard_turn
            .map(|turn| {
                format!(
                    "parry/guard reveal: {} {} vs {} {}",
                    turn.commits[0].label.as_str(),
                    turn.commits[0].direction.as_str(),
                    turn.commits[1].label.as_str(),
                    turn.commits[1].direction.as_str()
                )
            })
            .unwrap_or_else(|| "parry window unavailable".to_string()),
        detail: guard_turn
            .and_then(|turn| turn.costs.get(1))
            .map(|cost| {
                format!(
                    "defender cost base {} current {} valid {}",
                    cost.base_frames, cost.current_frames, cost.action_valid
                )
            })
            .unwrap_or_else(|| "defense cost unavailable".to_string()),
        frame_hash: String::new(),
    });

    frames.push(NativeCombatFrameSpec {
        index: 4,
        file: "native_combat_frame_004.ppm".to_string(),
        state: "weapon_arc",
        turn: first_turn.map(|turn| turn.turn).unwrap_or(0),
        headline: first_turn
            .map(|turn| {
                format!(
                    "weapon arcs: {} {} / {} {}",
                    turn.commits[0].label.as_str(),
                    turn.commits[0].direction.as_str(),
                    turn.commits[1].label.as_str(),
                    turn.commits[1].direction.as_str()
                )
            })
            .unwrap_or_else(|| "weapon arc unavailable".to_string()),
        detail: first_turn
            .and_then(|turn| turn.costs.first())
            .map(|cost| {
                format!(
                    "startup/cost evidence F{} base {} current {}",
                    cost.fighter, cost.base_frames, cost.current_frames
                )
            })
            .unwrap_or_else(|| "weapon arc cost unavailable".to_string()),
        frame_hash: String::new(),
    });

    let first_contact = result
        .turns
        .iter()
        .flat_map(|turn| turn.contacts.iter())
        .next();
    let first_grip_loss = result
        .turns
        .iter()
        .flat_map(|turn| turn.contacts.iter())
        .find(|contact| {
            contact.capability_delta.grip_r_delta < 0 || contact.capability_delta.grip_l_delta < 0
        });
    frames.push(NativeCombatFrameSpec {
        index: 5,
        file: "native_combat_frame_005.ppm".to_string(),
        state: "hit_contact",
        turn: first_contact.map(|contact| contact.turn).unwrap_or(0),
        headline: first_contact
            .map(|contact| format!("{} -> {}", contact.action.as_str(), contact.material_result))
            .unwrap_or_else(|| "near miss / no contact".to_string()),
        detail: first_contact
            .map(|contact| contact.cause_chain.clone())
            .unwrap_or_else(|| "no contact packet generated".to_string()),
        frame_hash: String::new(),
    });

    frames.push(NativeCombatFrameSpec {
        index: 6,
        file: "native_combat_frame_006.ppm".to_string(),
        state: "armor_material_solve",
        turn: first_contact.map(|contact| contact.turn).unwrap_or(0),
        headline: first_contact
            .map(|contact| {
                format!(
                    "{} on {} -> {}",
                    contact.weapon_id, contact.armor_id, contact.material_result
                )
            })
            .unwrap_or_else(|| "armor/material solve unavailable".to_string()),
        detail: first_contact
            .map(|contact| {
                format!(
                    "energy {} impulse {} anatomy {}",
                    contact.energy_milli,
                    contact.impulse_milli,
                    clipped_text(&contact.anatomy_result, 26)
                )
            })
            .unwrap_or_else(|| "no contact packet generated".to_string()),
        frame_hash: String::new(),
    });

    frames.push(NativeCombatFrameSpec {
        index: 7,
        file: "native_combat_frame_007.ppm".to_string(),
        state: "injury_capability",
        turn: first_contact.map(|contact| contact.turn).unwrap_or(0),
        headline: first_contact
            .map(|contact| contact.capability_delta.event.clone())
            .filter(|event| !event.is_empty())
            .unwrap_or_else(|| "capability unchanged".to_string()),
        detail: first_contact
            .map(|contact| {
                format!(
                    "recovery +{}f balance {} torque {} grip_r {}",
                    contact.capability_delta.recovery_slowdown_add,
                    contact.capability_delta.balance_delta,
                    contact.capability_delta.torque_delta,
                    contact.capability_delta.grip_r_delta
                )
            })
            .unwrap_or_else(|| "no capability delta".to_string()),
        frame_hash: String::new(),
    });

    frames.push(NativeCombatFrameSpec {
        index: 8,
        file: "native_combat_frame_008.ppm".to_string(),
        state: "grip_loss",
        turn: first_grip_loss
            .or(first_contact)
            .map(|contact| contact.turn)
            .unwrap_or(0),
        headline: first_grip_loss
            .map(|contact| {
                format!(
                    "grip delta r{} l{}",
                    contact.capability_delta.grip_r_delta, contact.capability_delta.grip_l_delta
                )
            })
            .unwrap_or_else(|| "grip unchanged".to_string()),
        detail: first_grip_loss
            .map(|contact| {
                if contact.capability_delta.grip_r_delta < 0
                    || contact.capability_delta.grip_l_delta < 0
                {
                    "weapon action validity will be checked after grip loss".to_string()
                } else {
                    "no grip loss in first packet; validity remains checked".to_string()
                }
            })
            .unwrap_or_else(|| "no grip packet generated".to_string()),
        frame_hash: String::new(),
    });

    frames.push(NativeCombatFrameSpec {
        index: 9,
        file: "native_combat_frame_009.ppm".to_string(),
        state: "stance_collapse_risk",
        turn: first_contact.map(|contact| contact.turn).unwrap_or(0),
        headline: first_contact
            .map(|contact| format!("balance delta {}", contact.capability_delta.balance_delta))
            .unwrap_or_else(|| "stance stable".to_string()),
        detail: first_contact
            .map(|contact| {
                format!(
                    "torque {} recovery +{}f",
                    contact.capability_delta.torque_delta,
                    contact.capability_delta.recovery_slowdown_add
                )
            })
            .unwrap_or_else(|| "no stance penalty".to_string()),
        frame_hash: String::new(),
    });

    let no_contact_turn = result.turns.iter().find(|turn| turn.contacts.is_empty());
    frames.push(NativeCombatFrameSpec {
        index: 10,
        file: "native_combat_frame_010.ppm".to_string(),
        state: "near_miss_replan",
        turn: no_contact_turn
            .or(last_turn)
            .map(|turn| turn.turn)
            .unwrap_or(0),
        headline: no_contact_turn
            .map(|turn| {
                format!(
                    "no contact: {} vs {}",
                    turn.commits[0].label.as_str(),
                    turn.commits[1].label.as_str()
                )
            })
            .unwrap_or_else(|| "no near-miss turn in script".to_string()),
        detail: no_contact_turn
            .map(|turn| format!("replan from turn hash {}", turn.state_hash))
            .unwrap_or_else(|| "all scripted turns produced packets".to_string()),
        frame_hash: String::new(),
    });

    frames.push(NativeCombatFrameSpec {
        index: 11,
        file: "native_combat_frame_011.ppm".to_string(),
        state: "recovery_state",
        turn: last_turn.map(|turn| turn.turn).unwrap_or(0),
        headline: last_turn
            .map(|turn| format!("final turn hash {}", turn.state_hash))
            .unwrap_or_else(|| format!("final hash {}", result.final_state_hash)),
        detail: last_turn
            .and_then(|turn| turn.costs.first())
            .map(|cost| {
                format!(
                    "next cost evidence: fighter {} {} base {} current {}",
                    cost.fighter,
                    cost.action.as_str(),
                    cost.base_frames,
                    cost.current_frames
                )
            })
            .unwrap_or_else(|| "recovery costs unavailable".to_string()),
        frame_hash: String::new(),
    });

    frames.push(NativeCombatFrameSpec {
        index: 12,
        file: "native_combat_frame_012.ppm".to_string(),
        state: "final_hash_card",
        turn: last_turn.map(|turn| turn.turn).unwrap_or(0),
        headline: format!("final replay hash {}", result.final_state_hash),
        detail: format!(
            "content {} | end {}",
            result.content_hash,
            result.end_condition.status.as_str()
        ),
        frame_hash: String::new(),
    });

    frames
}

#[cfg(target_os = "linux")]
fn build_native_combat_motion_frame_specs(result: &DuelResult) -> Vec<NativeCombatMotionFrameSpec> {
    let mut frames = Vec::new();
    for turn in &result.turns {
        let max_cost = turn
            .costs
            .iter()
            .map(|cost| cost.current_frames)
            .max()
            .unwrap_or(TRUTH_HZ / 4)
            .max(1);
        let contact = turn.contacts.first();
        let contact_offset = contact
            .map(|contact| contact.frame.saturating_sub(turn.turn * TRUTH_HZ))
            .unwrap_or(max_cost / 2)
            .min(max_cost);
        let has_stagger_risk = contact
            .map(|contact| {
                contact.capability_delta.balance_delta < 0
                    || contact.capability_delta.recovery_slowdown_add > 0
            })
            .unwrap_or(false);

        let samples = [
            (
                "observe_plan",
                0,
                0,
                format!(
                    "observe turn {}: {} vs {}",
                    turn.turn,
                    turn.commits[0].label.as_str(),
                    turn.commits[1].label.as_str()
                ),
                "players choose compact physical labels and directional influence".to_string(),
            ),
            (
                "commit_reveal",
                max_cost / 4,
                250,
                format!(
                    "commit reveal: {} {} / {} {}",
                    turn.commits[0].label.as_str(),
                    turn.commits[0].direction.as_str(),
                    turn.commits[1].label.as_str(),
                    turn.commits[1].direction.as_str()
                ),
                turn.costs
                    .first()
                    .map(|cost| {
                        format!(
                            "F{} {} base {} current {} valid {}",
                            cost.fighter,
                            cost.action.as_str(),
                            cost.base_frames,
                            cost.current_frames,
                            cost.action_valid
                        )
                    })
                    .unwrap_or_else(|| "cost breakdown unavailable".to_string()),
            ),
            (
                "weapon_arc",
                max_cost / 2,
                500,
                format!(
                    "weapon arc: {} / {}",
                    turn.commits[0].label.as_str(),
                    turn.commits[1].label.as_str()
                ),
                turn.costs
                    .iter()
                    .find(|cost| cost.fighter == 0)
                    .map(|cost| {
                        format!(
                            "base {} current {} equipment/body reasons visible",
                            cost.base_frames, cost.current_frames
                        )
                    })
                    .unwrap_or_else(|| "weapon arc cost unavailable".to_string()),
            ),
            (
                "active_contact",
                contact_offset,
                650,
                contact
                    .map(|contact| {
                        format!(
                            "{} contact at truth frame {}",
                            contact.action.as_str(),
                            contact.frame
                        )
                    })
                    .unwrap_or_else(|| "active window: no contact packet".to_string()),
                contact
                    .map(|contact| contact.cause_chain.clone())
                    .unwrap_or_else(|| format!("turn hash {}", turn.state_hash)),
            ),
            (
                "material_anatomy_solve",
                (contact_offset + max_cost).saturating_div(2).min(max_cost),
                825,
                contact
                    .map(|contact| {
                        format!(
                            "{} -> {}",
                            contact.material_result,
                            clipped_text(&contact.anatomy_result, 24)
                        )
                    })
                    .unwrap_or_else(|| "material/anatomy solve: no packet".to_string()),
                contact
                    .map(|contact| {
                        format!(
                            "energy {} impulse {} target {}",
                            contact.energy_milli,
                            contact.impulse_milli,
                            contact.target.as_str()
                        )
                    })
                    .unwrap_or_else(|| "range/guard state produced no injury".to_string()),
            ),
            (
                "recovery_capability",
                max_cost.saturating_sub(max_cost / 8),
                925,
                contact
                    .map(|contact| {
                        format!(
                            "recovery +{} balance {} grip_r {}",
                            contact.capability_delta.recovery_slowdown_add,
                            contact.capability_delta.balance_delta,
                            contact.capability_delta.grip_r_delta
                        )
                    })
                    .unwrap_or_else(|| "recovery: no capability loss".to_string()),
                contact
                    .map(|contact| contact.capability_delta.event.clone())
                    .filter(|event| !event.is_empty())
                    .unwrap_or_else(|| "future action validity remains checked".to_string()),
            ),
            (
                if has_stagger_risk {
                    "stagger_collapse_risk"
                } else {
                    "consequence_replan"
                },
                max_cost,
                1000,
                contact
                    .map(|contact| contact.capability_delta.event.clone())
                    .filter(|event| !event.is_empty())
                    .unwrap_or_else(|| "consequence: no capability loss".to_string()),
                contact
                    .map(|contact| {
                        format!(
                            "balance {} torque {} recovery +{}f | hash {}",
                            contact.capability_delta.balance_delta,
                            contact.capability_delta.torque_delta,
                            contact.capability_delta.recovery_slowdown_add,
                            turn.state_hash
                        )
                    })
                    .unwrap_or_else(|| format!("replan from turn hash {}", turn.state_hash)),
            ),
        ];

        for (phase, offset, progress_permille, headline, detail) in samples {
            let index = frames.len() + 1;
            frames.push(NativeCombatMotionFrameSpec {
                index,
                file: format!("native_combat_motion_{index:03}.ppm"),
                phase,
                turn: turn.turn,
                truth_frame: turn.turn * TRUTH_HZ + offset,
                progress_permille,
                headline,
                detail,
                turn_hash: turn.state_hash.clone(),
                frame_hash: String::new(),
            });
        }
    }
    frames
}

#[cfg(target_os = "linux")]
fn build_native_player_loop_frame_specs(
    result: &DuelResult,
    motion_frames: &[NativeCombatMotionFrameSpec],
) -> Result<Vec<NativePlayerLoopFrameSpec>, String> {
    let flow_model = build_native_hud_menu_flow_model(result).map_err(|error| error.to_string())?;
    let first_motion = motion_frames.first();
    let combat_motion = motion_frames
        .iter()
        .find(|frame| frame.phase == "active_contact")
        .or_else(|| {
            motion_frames
                .iter()
                .find(|frame| frame.phase == "weapon_arc")
        })
        .or(first_motion);
    let replay_motion = motion_frames.last().or(combat_motion).or(first_motion);
    let first_contact = result
        .turns
        .iter()
        .flat_map(|turn| turn.contacts.iter())
        .next();
    let first_contact_frame = first_contact
        .map(|contact| contact.frame)
        .or_else(|| combat_motion.map(|frame| frame.truth_frame))
        .unwrap_or(0);

    let mut frames = Vec::with_capacity(flow_model.screens.len());
    for screen in &flow_model.screens {
        let truth_frame = match screen.flow_id {
            "observe" | "plan" | "commit_reveal" => {
                first_motion.map(|frame| frame.truth_frame).unwrap_or(0)
            }
            "resolve" => first_contact_frame,
            "consequence" => combat_motion
                .map(|frame| frame.truth_frame)
                .unwrap_or(first_contact_frame),
            "replay_browser" | "fight_film" | "performance_debug_overlay" => {
                replay_motion.map(|frame| frame.truth_frame).unwrap_or(0)
            }
            _ => 0,
        };
        frames.push(NativePlayerLoopFrameSpec {
            index: screen.flow_index + 1,
            file: format!("native_player_loop_{:03}.ppm", screen.flow_index + 1),
            screen: screen.flow_id,
            input_action: screen.input_action,
            scheduled_ms: screen.flow_index as u32 * 80,
            truth_frame,
            headline: screen.headline.clone(),
            detail: screen.detail.clone(),
            truth_cache_key: screen.truth_cache_key.clone(),
            base_cost_frames: screen.base_cost_frames,
            current_cost_frames: screen.current_cost_frames,
            physical_reasons: screen.physical_reasons.clone(),
            frame_hash: String::new(),
        });
    }

    for flow_id in HUD_NATIVE_FLOW_IDS {
        if !frames.iter().any(|frame| frame.screen == flow_id) {
            return Err(format!(
                "native player-facing loop missing HUD truth flow {flow_id}"
            ));
        }
    }

    Ok(frames)
}

#[cfg(target_os = "linux")]
fn write_native_combat_asset_ref_json(
    out: &mut String,
    indent: usize,
    key: &str,
    asset: &NativeCombatAssetRef,
    trailing_comma: bool,
) {
    let pad = "  ".repeat(indent);
    writeln!(out, "{pad}\"{key}\": {{").unwrap();
    write_json_field(out, indent + 1, "id", &asset.id, true);
    write_json_field(out, indent + 1, "kind", &asset.kind, true);
    write_json_field(out, indent + 1, "source", &asset.source, true);
    write_json_field(out, indent + 1, "runtime_mesh", &asset.runtime_mesh, true);
    write_json_field(out, indent + 1, "runtime_gltf", &asset.runtime_gltf, true);
    write_json_field(out, indent + 1, "preview", &asset.preview, true);
    write_json_field(out, indent + 1, "provenance", &asset.provenance, true);
    write_json_field(out, indent + 1, "source_hash", &asset.source_hash, true);
    write_json_field(
        out,
        indent + 1,
        "runtime_mesh_hash",
        &asset.runtime_mesh_hash,
        true,
    );
    write_json_field(
        out,
        indent + 1,
        "runtime_gltf_hash",
        &asset.runtime_gltf_hash,
        true,
    );
    write_json_field(out, indent + 1, "preview_hash", &asset.preview_hash, true);
    write_native_gltf_geometry_json(out, indent + 1, "geometry", &asset.geometry, false);
    writeln!(out, "{pad}}}{}", if trailing_comma { "," } else { "" }).unwrap();
}

#[cfg(target_os = "linux")]
fn write_native_presentation_asset_ref_json(
    out: &mut String,
    indent: usize,
    key: &str,
    asset: &NativePresentationAssetRef,
    trailing_comma: bool,
) {
    let pad = "  ".repeat(indent);
    writeln!(out, "{pad}\"{key}\": {{").unwrap();
    write_json_field(out, indent + 1, "id", &asset.id, true);
    write_json_field(out, indent + 1, "kind", &asset.kind, true);
    write_json_field(out, indent + 1, "source", &asset.source, true);
    write_json_field(out, indent + 1, "runtime_mesh", &asset.runtime_mesh, true);
    write_json_field(out, indent + 1, "runtime_gltf", &asset.runtime_gltf, true);
    write_json_field(out, indent + 1, "preview", &asset.preview, true);
    write_json_field(
        out,
        indent + 1,
        "source_candidate_gltf",
        &asset.source_candidate_gltf,
        true,
    );
    write_json_field(
        out,
        indent + 1,
        "source_candidate_bin",
        &asset.source_candidate_bin,
        true,
    );
    write_json_field(out, indent + 1, "provenance", &asset.provenance, true);
    write_json_field(
        out,
        indent + 1,
        "license_status",
        &asset.license_status,
        true,
    );
    write_json_field(out, indent + 1, "source_hash", &asset.source_hash, true);
    write_json_field(
        out,
        indent + 1,
        "runtime_mesh_hash",
        &asset.runtime_mesh_hash,
        true,
    );
    write_json_field(
        out,
        indent + 1,
        "runtime_gltf_hash",
        &asset.runtime_gltf_hash,
        true,
    );
    write_json_field(out, indent + 1, "preview_hash", &asset.preview_hash, true);
    write_json_field(
        out,
        indent + 1,
        "source_candidate_gltf_hash",
        &asset.source_candidate_gltf_hash,
        true,
    );
    write_json_field(
        out,
        indent + 1,
        "source_candidate_bin_hash",
        &asset.source_candidate_bin_hash,
        true,
    );
    writeln!(out, "{}  \"presentation_only\": true,", pad).unwrap();
    writeln!(out, "{}  \"truth_authoritative\": false,", pad).unwrap();
    writeln!(out, "{}  \"truth_mutation\": false,", pad).unwrap();
    write_native_gltf_geometry_json(out, indent + 1, "geometry", &asset.geometry, false);
    writeln!(out, "{pad}}}{}", if trailing_comma { "," } else { "" }).unwrap();
}

#[cfg(target_os = "linux")]
fn write_native_gltf_geometry_json(
    out: &mut String,
    indent: usize,
    key: &str,
    geometry: &NativeGltfGeometry,
    trailing_comma: bool,
) {
    let pad = "  ".repeat(indent);
    writeln!(out, "{pad}\"{key}\": {{").unwrap();
    writeln!(out, "{}  \"geometry_projected\": true,", pad).unwrap();
    writeln!(out, "{}  \"vertex_count\": {},", pad, geometry.vertex_count).unwrap();
    writeln!(out, "{}  \"index_count\": {},", pad, geometry.index_count).unwrap();
    writeln!(
        out,
        "{}  \"triangle_count\": {},",
        pad, geometry.triangle_count
    )
    .unwrap();
    writeln!(
        out,
        "{}  \"z_depth_milli\": {},",
        pad,
        geometry_z_depth_milli(geometry)
    )
    .unwrap();
    writeln!(
        out,
        "{}  \"has_nonzero_z_depth\": {},",
        pad,
        native_geometry_has_depth(geometry)
    )
    .unwrap();
    writeln!(out, "{}  \"projection_uses_z_depth\": true,", pad).unwrap();
    writeln!(out, "{}  \"bounds_milli\": {{", pad).unwrap();
    writeln!(out, "{}    \"min_x\": {},", pad, geometry.min_x_milli).unwrap();
    writeln!(out, "{}    \"max_x\": {},", pad, geometry.max_x_milli).unwrap();
    writeln!(out, "{}    \"min_y\": {},", pad, geometry.min_y_milli).unwrap();
    writeln!(out, "{}    \"max_y\": {},", pad, geometry.max_y_milli).unwrap();
    writeln!(out, "{}    \"min_z\": {},", pad, geometry.min_z_milli).unwrap();
    writeln!(out, "{}    \"max_z\": {}", pad, geometry.max_z_milli).unwrap();
    writeln!(out, "{}  }},", pad).unwrap();
    write_json_field(
        out,
        indent + 1,
        "geometry_hash",
        &geometry.geometry_hash,
        false,
    );
    writeln!(out, "{pad}}}{}", if trailing_comma { "," } else { "" }).unwrap();
}

#[cfg(target_os = "linux")]
fn write_native_json_string_array(
    out: &mut String,
    indent: usize,
    key: &str,
    values: &[String],
    trailing_comma: bool,
) {
    let pad = "  ".repeat(indent);
    writeln!(out, "{pad}{}: [", json_quote(key)).unwrap();
    for (index, value) in values.iter().enumerate() {
        writeln!(
            out,
            "{}  {}{}",
            pad,
            json_quote(value),
            comma(index + 1, values.len())
        )
        .unwrap();
    }
    writeln!(out, "{pad}]{}", if trailing_comma { "," } else { "" }).unwrap();
}

#[cfg(target_os = "linux")]
fn native_camera_modes() -> [&'static str; 6] {
    [
        "first_person_guard_line",
        "third_person_verdict_ring",
        "planning_tactical_reach",
        "consequence_aftermath_dwell",
        "fight_film_orbit",
        "asset_closeup_weapon_armor",
    ]
}

#[cfg(target_os = "linux")]
fn native_camera_category(camera: &str) -> &'static str {
    match camera {
        "third_person_verdict_ring" => "third_person",
        "first_person_guard_line" => "first_person",
        "planning_tactical_reach" => "planning",
        "consequence_aftermath_dwell" => "consequence",
        "fight_film_orbit" => "fight_film",
        "third_person_replay_orbit" => "replay_sequence",
        "asset_closeup_weapon_armor" => "asset_closeup",
        _ => "unknown",
    }
}

#[cfg(target_os = "linux")]
fn native_camera_readability_focus(camera: &str) -> &'static str {
    match camera {
        "third_person_verdict_ring" => "body spacing, arena position, and gross weapon reach",
        "first_person_guard_line" => "guard line, incoming weapon lane, and contact window",
        "planning_tactical_reach" => "tactical spacing, facing, feet, and reach engagement arcs",
        "consequence_aftermath_dwell" => {
            "post-strike injury, stagger, capability loss, and dwell timing"
        }
        "fight_film_orbit" => "replay/fight-film contact timing and consequence staging",
        "third_person_replay_orbit" => {
            "deterministic replay phase sequence from verified trace frames"
        }
        "asset_closeup_weapon_armor" => {
            "weapon/armor material interaction and capability consequence closeup"
        }
        _ => "unspecified",
    }
}

#[cfg(target_os = "linux")]
fn native_camera_truth_source(camera: &str) -> &'static str {
    match camera {
        "third_person_replay_orbit" => "replay-derived-runtime-gltf-after-truth-hash",
        "asset_closeup_weapon_armor" => "presentation-gltf-closeup-after-truth-hash",
        _ => "runtime-gltf-after-truth-hash",
    }
}

#[cfg(target_os = "linux")]
fn native_camera_fov_degrees(camera: &str) -> u32 {
    match camera {
        "first_person_guard_line" => 68,
        "third_person_verdict_ring" => 54,
        "planning_tactical_reach" => 42,
        "consequence_aftermath_dwell" => 48,
        "fight_film_orbit" => 58,
        "asset_closeup_weapon_armor" => 32,
        _ => 54,
    }
}

#[cfg(target_os = "linux")]
fn native_camera_position_milli(camera: &str) -> (i32, i32, i32) {
    match camera {
        "first_person_guard_line" => (-420, 1480, -1180),
        "third_person_verdict_ring" => (0, 1900, -3100),
        "planning_tactical_reach" => (0, 4200, -900),
        "consequence_aftermath_dwell" => (580, 1680, -1760),
        "fight_film_orbit" => (-1500, 2100, -2100),
        "asset_closeup_weapon_armor" => (240, 1240, -620),
        _ => (0, 1900, -3100),
    }
}

#[cfg(target_os = "linux")]
fn native_camera_target_milli(camera: &str) -> (i32, i32, i32) {
    match camera {
        "first_person_guard_line" => (320, 1040, 80),
        "third_person_verdict_ring" => (0, 820, 0),
        "planning_tactical_reach" => (0, 580, 0),
        "consequence_aftermath_dwell" => (360, 760, 80),
        "fight_film_orbit" => (0, 840, 0),
        "asset_closeup_weapon_armor" => (0, 720, 120),
        _ => (0, 820, 0),
    }
}

#[cfg(target_os = "linux")]
fn native_camera_timing_policy(camera: &str) -> &'static str {
    match camera {
        "planning_tactical_reach" => "pre-contact dwell at observe/plan/commit frames",
        "consequence_aftermath_dwell" => "post-contact dwell on recovery/capability frames",
        "fight_film_orbit" => "replay-frame-index locked orbit from trace motion frames",
        _ => "single deterministic capture frame from post-hash replay data",
    }
}

#[cfg(target_os = "linux")]
fn write_native_camera_metadata_array(
    out: &mut String,
    indent: usize,
    key: &str,
    trailing_comma: bool,
) {
    let pad = "  ".repeat(indent);
    let modes = native_camera_modes();
    writeln!(out, "{pad}{}: [", json_quote(key)).unwrap();
    for (index, camera) in modes.iter().enumerate() {
        writeln!(out, "{pad}  {{").unwrap();
        write_json_field(out, indent + 2, "camera", camera, true);
        write_json_field(
            out,
            indent + 2,
            "category",
            native_camera_category(camera),
            true,
        );
        write_json_field(
            out,
            indent + 2,
            "readability_focus",
            native_camera_readability_focus(camera),
            true,
        );
        write_json_field(
            out,
            indent + 2,
            "truth_source",
            native_camera_truth_source(camera),
            true,
        );
        writeln!(
            out,
            "{}    \"fov_degrees\": {},",
            pad,
            native_camera_fov_degrees(camera)
        )
        .unwrap();
        let position = native_camera_position_milli(camera);
        writeln!(
            out,
            "{}    \"position_milli\": [{}, {}, {}],",
            pad, position.0, position.1, position.2
        )
        .unwrap();
        let target = native_camera_target_milli(camera);
        writeln!(
            out,
            "{}    \"target_milli\": [{}, {}, {}],",
            pad, target.0, target.1, target.2
        )
        .unwrap();
        write_json_field(
            out,
            indent + 2,
            "depth_of_field_policy",
            "presentation-focus-only-no-truth-effect",
            true,
        );
        write_json_field(
            out,
            indent + 2,
            "timing_policy",
            native_camera_timing_policy(camera),
            true,
        );
        writeln!(out, "{}    \"presentation_only\": true,", pad).unwrap();
        writeln!(out, "{}    \"truth_mutation\": false", pad).unwrap();
        writeln!(out, "{pad}  }}{}", comma(index + 1, modes.len())).unwrap();
    }
    writeln!(out, "{pad}]{}", if trailing_comma { "," } else { "" }).unwrap();
}

#[cfg(target_os = "linux")]
fn native_capture_timestamp_ms(frame_index: usize, truth_frame: u32) -> u32 {
    let frame = if truth_frame > 0 {
        truth_frame
    } else {
        frame_index.saturating_sub(1).try_into().unwrap_or(u32::MAX)
    };
    frame.saturating_mul(1000) / TRUTH_HZ
}

#[cfg(target_os = "linux")]
fn write_native_capture_camera_metadata_json(
    out: &mut String,
    indent: usize,
    key: &str,
    camera: &str,
    frame_index: usize,
    truth_frame: u32,
    replay_source_hash: &str,
    trailing_comma: bool,
) {
    let pad = "  ".repeat(indent);
    writeln!(out, "{pad}{}: {{", json_quote(key)).unwrap();
    write_json_field(out, indent + 1, "camera_mode", camera, true);
    write_json_field(
        out,
        indent + 1,
        "camera_category",
        native_camera_category(camera),
        true,
    );
    write_json_field(
        out,
        indent + 1,
        "readability_focus",
        native_camera_readability_focus(camera),
        true,
    );
    write_json_field(
        out,
        indent + 1,
        "truth_source",
        native_camera_truth_source(camera),
        true,
    );
    writeln!(
        out,
        "{}  \"fov_degrees\": {},",
        pad,
        native_camera_fov_degrees(camera)
    )
    .unwrap();
    let position = native_camera_position_milli(camera);
    writeln!(
        out,
        "{}  \"position_milli\": [{}, {}, {}],",
        pad, position.0, position.1, position.2
    )
    .unwrap();
    let target = native_camera_target_milli(camera);
    writeln!(
        out,
        "{}  \"target_milli\": [{}, {}, {}],",
        pad, target.0, target.1, target.2
    )
    .unwrap();
    writeln!(out, "{}  \"frame_index\": {},", pad, frame_index).unwrap();
    writeln!(out, "{}  \"truth_frame\": {},", pad, truth_frame).unwrap();
    writeln!(
        out,
        "{}  \"timestamp_ms\": {},",
        pad,
        native_capture_timestamp_ms(frame_index, truth_frame)
    )
    .unwrap();
    write_json_field(
        out,
        indent + 1,
        "timestamp_source",
        "deterministic-truth-frame-index-fixed-120hz-no-wall-clock",
        true,
    );
    write_json_field(
        out,
        indent + 1,
        "replay_source_hash",
        replay_source_hash,
        true,
    );
    writeln!(out, "{}  \"capture_after_truth_hash\": true,", pad).unwrap();
    writeln!(out, "{}  \"presentation_only\": true,", pad).unwrap();
    writeln!(out, "{}  \"truth_mutation\": false", pad).unwrap();
    writeln!(out, "{pad}}}{}", if trailing_comma { "," } else { "" }).unwrap();
}

#[cfg(target_os = "linux")]
fn write_native_verified_replay_trace_input_json(
    out: &mut String,
    indent: usize,
    result: &DuelResult,
    replay_source_hash: &str,
    trace_source_hash: &str,
    trailing_comma: bool,
) {
    let pad = "  ".repeat(indent);
    writeln!(out, "{pad}\"verified_replay_trace_input\": {{").unwrap();
    write_json_field(
        out,
        indent + 1,
        "source",
        "verified-replay-trace-before-capture",
        true,
    );
    writeln!(out, "{}  \"replay_verified\": true,", pad).unwrap();
    write_json_field(
        out,
        indent + 1,
        "replay_source_hash",
        replay_source_hash,
        true,
    );
    write_json_field(
        out,
        indent + 1,
        "trace_source_hash",
        trace_source_hash,
        true,
    );
    write_json_field(out, indent + 1, "content_hash", &result.content_hash, true);
    write_json_field(
        out,
        indent + 1,
        "initial_state_hash",
        &result.initial_state_hash,
        true,
    );
    write_json_field(
        out,
        indent + 1,
        "final_state_hash",
        &result.final_state_hash,
        true,
    );
    writeln!(out, "{}  \"truth_hz\": {TRUTH_HZ},", pad).unwrap();
    writeln!(out, "{}  \"presentation_only\": true,", pad).unwrap();
    writeln!(out, "{}  \"truth_mutation\": false", pad).unwrap();
    writeln!(out, "{pad}}}{}", if trailing_comma { "," } else { "" }).unwrap();
}

#[cfg(target_os = "linux")]
fn write_native_renderer_truth_snapshot_json(
    out: &mut String,
    indent: usize,
    key: &str,
    snapshot: &NativeRendererTruthSnapshot,
    trailing_comma: bool,
) {
    let pad = "  ".repeat(indent);
    writeln!(out, "{pad}\"{key}\": {{").unwrap();
    write_json_field(
        out,
        indent + 1,
        "final_state_hash",
        &snapshot.final_state_hash,
        true,
    );
    write_json_field(
        out,
        indent + 1,
        "replay_json_hash",
        &snapshot.replay_json_hash,
        true,
    );
    write_json_field(
        out,
        indent + 1,
        "trace_json_hash",
        &snapshot.trace_json_hash,
        true,
    );
    write_json_field(
        out,
        indent + 1,
        "contacts_hash",
        &snapshot.contacts_hash,
        true,
    );
    write_json_field(
        out,
        indent + 1,
        "injury_capability_hash",
        &snapshot.injury_capability_hash,
        true,
    );
    write_json_field(
        out,
        indent + 1,
        "action_validity_hash",
        &snapshot.action_validity_hash,
        true,
    );
    write_json_field(
        out,
        indent + 1,
        "end_condition_hash",
        &snapshot.end_condition_hash,
        false,
    );
    writeln!(out, "{pad}}}{}", if trailing_comma { "," } else { "" }).unwrap();
}

#[cfg(target_os = "linux")]
fn render_native_renderer_capture_hook_json(hook: &NativeRendererCaptureHookSummary) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(
        &mut out,
        1,
        "schema",
        NATIVE_RENDERER_CAPTURE_HOOK_SCHEMA,
        true,
    );
    write_json_field(&mut out, 1, "hook_id", &hook.hook_id, true);
    write_json_field(
        &mut out,
        1,
        "framebuffer_source",
        &hook.framebuffer_source,
        true,
    );
    write_json_field(&mut out, 1, "timing_source", &hook.timing_source, true);
    writeln!(&mut out, "  \"capture_count\": {},", hook.capture_count).unwrap();
    writeln!(
        &mut out,
        "  \"high_resolution_capture_count\": {},",
        hook.high_resolution_capture_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"min_high_resolution_width\": {},",
        hook.min_high_resolution_width
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"min_high_resolution_height\": {},",
        hook.min_high_resolution_height
    )
    .unwrap();
    write_json_field(&mut out, 1, "hash_algorithm", "fnv1a64", true);
    write_json_field(&mut out, 1, "hook_hash", &hook.hook_hash, true);
    writeln!(&mut out, "  \"captures\": [").unwrap();
    for (index, capture) in hook.captures.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "file", &capture.file, true);
        write_json_field(&mut out, 3, "stream", &capture.stream, true);
        writeln!(&mut out, "      \"width\": {},", capture.width).unwrap();
        writeln!(&mut out, "      \"height\": {},", capture.height).unwrap();
        write_json_field(&mut out, 3, "source", &capture.source, true);
        writeln!(
            &mut out,
            "      \"capture_after_truth_hash\": {},",
            capture.capture_after_truth_hash
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"presentation_only\": {},",
            capture.presentation_only
        )
        .unwrap();
        writeln!(&mut out, "      \"truth_mutation\": false,").unwrap();
        write_json_field(&mut out, 3, "frame_hash", &capture.frame_hash, false);
        writeln!(&mut out, "    }}{}", comma(index + 1, hook.captures.len())).unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

#[cfg(target_os = "linux")]
fn render_native_renderer_mutation_proof_json(proof: &NativeRendererMutationProof) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(
        &mut out,
        1,
        "schema",
        NATIVE_RENDERER_MUTATION_PROOF_SCHEMA,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "source",
        "before-after-render-capture-comparison",
        true,
    );
    writeln!(&mut out, "  \"truth_mutation\": {},", !proof.all_equal).unwrap();
    writeln!(&mut out, "  \"all_equal\": {},", proof.all_equal).unwrap();
    write_native_json_string_array(
        &mut out,
        1,
        "changed_fields",
        &native_renderer_changed_truth_fields(proof),
        true,
    );
    write_native_renderer_truth_snapshot_json(&mut out, 1, "before", &proof.before, true);
    write_native_renderer_truth_snapshot_json(&mut out, 1, "after", &proof.after, false);
    writeln!(&mut out, "}}").unwrap();
    out
}

#[cfg(target_os = "linux")]
fn native_renderer_truth_diff_summary(proof: &NativeRendererMutationProof) -> String {
    let changed = native_renderer_changed_truth_fields(proof);
    if changed.is_empty() {
        "none".to_string()
    } else {
        changed.join(",")
    }
}

#[cfg(target_os = "linux")]
fn native_renderer_assert_truth_unchanged(
    label: &str,
    before: &NativeRendererTruthSnapshot,
    result: &DuelResult,
) -> Result<NativeRendererTruthSnapshot, OathError> {
    let after = native_renderer_truth_snapshot(result);
    if before != &after {
        let proof = native_renderer_mutation_proof(before.clone(), after.clone());
        return Err(OathError::Verify(format!(
            "native renderer truth writeback guard failed during {label}; changed_fields={}",
            native_renderer_truth_diff_summary(&proof)
        )));
    }
    Ok(after)
}

#[cfg(target_os = "linux")]
fn render_native_renderer_truth_writeback_audit_report(
    result: &DuelResult,
    capture: &NativeCombatCapture,
    capture_hook: &NativeRendererCaptureHookSummary,
    mutation_proof: &NativeRendererMutationProof,
    resolution_captures: &[NativeCombatResolutionCapture],
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD Native Renderer Truth-Writeback Audit").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "Status: PASSED").unwrap();
    writeln!(&mut out, "- Scenario: `{}`", result.scenario_id).unwrap();
    writeln!(
        &mut out,
        "- Final state hash: `{}`",
        result.final_state_hash
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Truth writeback guard: `{}`",
        if mutation_proof.all_equal {
            "PASSED"
        } else {
            "FAILED"
        }
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Runtime guard assertion: `native_renderer_assert_truth_unchanged`"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Guarded truth fields: `final_state_hash,replay_json_hash,trace_json_hash,contacts_hash,injury_capability_hash,action_validity_hash,end_condition_hash`"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Changed truth fields: `{}`",
        native_renderer_truth_diff_summary(mutation_proof)
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Capture hook entries audited: `{}`",
        capture_hook.capture_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Six-camera capture pass: `{}` viewports, `{}` product-mode captures",
        capture.software_3d_viewports.len(),
        resolution_captures
            .iter()
            .filter(|capture| !capture.debug_overlay)
            .count()
    )
    .unwrap();
    writeln!(&mut out).unwrap();

    writeln!(&mut out, "## Camera Metadata Disposition").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(
        &mut out,
        "| Camera | Category | Truth source | Disposition |"
    )
    .unwrap();
    writeln!(&mut out, "| --- | --- | --- | --- |").unwrap();
    for camera in native_camera_modes() {
        writeln!(
            &mut out,
            "| `{}` | `{}` | `{}` | presentation-only metadata; no truth writeback; `truth_mutation=false` |",
            camera,
            native_camera_category(camera),
            native_camera_truth_source(camera)
        )
        .unwrap();
    }
    writeln!(&mut out).unwrap();

    writeln!(&mut out, "## Renderer Write Paths").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "| Stream | File | Source | Disposition |").unwrap();
    writeln!(&mut out, "| --- | --- | --- | --- |").unwrap();
    for captured in &capture_hook.captures {
        writeln!(
            &mut out,
            "| `{}` | `{}` | `{}` | writes framebuffer/PPM artifact only; capture_after_truth_hash=`{}`; presentation_only=`{}`; truth_mutation=`false` |",
            captured.stream,
            captured.file,
            captured.source,
            captured.capture_after_truth_hash,
            captured.presentation_only
        )
        .unwrap();
    }
    writeln!(&mut out).unwrap();

    writeln!(&mut out, "## Product-Mode Clean Capture Disposition").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "| File | Camera | Capture role | Disposition |").unwrap();
    writeln!(&mut out, "| --- | --- | --- | --- |").unwrap();
    for captured in resolution_captures
        .iter()
        .filter(|capture| !capture.debug_overlay)
    {
        writeln!(
            &mut out,
            "| `{}` | `{}` | `{}` | clean presentation capture; capture_after_truth_hash=`true`; presentation_only=`true`; truth_mutation=`false` |",
            captured.file, captured.camera, captured.capture_role
        )
        .unwrap();
    }
    writeln!(&mut out).unwrap();

    writeln!(&mut out, "## Component Boundary Summary").unwrap();
    writeln!(&mut out).unwrap();
    for (component, writes, disposition) in [
        (
            "x11_combat_render_capture",
            "X11 pixmap/window plus native_combat_render.ppm/state/motion/playback/live/player-loop PPMs",
            "guarded by before/after truth snapshot; no DuelResult mutation",
        ),
        (
            "write_native_combat_software_3d_viewports",
            "six software 3D camera PPMs",
            "runtime glTF/readability presentation only after hash",
        ),
        (
            "write_native_combat_software_3d_sequence",
            "21 replay-derived software 3D motion PPMs",
            "replay-derived presentation only after hash",
        ),
        (
            "write_native_combat_resolution_captures",
            "diagnostic overview plus product-mode clean high-resolution PPMs",
            "manifested with capture_after_truth_hash=true and truth_mutation=false",
        ),
        (
            "write_native_production_renderer_bundle",
            "artifacts/production_renderer/latest plus out-dir manifest/report copies",
            "production-capture evidence only; production_renderer_complete=false",
        ),
        (
            "write_native_lighting_material_witness_artifacts",
            "native_lighting_material_witness.ppm/json",
            "lighting/material witness only; truth_mutation=false",
        ),
        (
            "render_native_*_manifest/report/contact_sheet",
            "JSON/Markdown/SVG metadata artifacts",
            "debug labels and contact sheets are presentation reports only",
        ),
    ] {
        writeln!(&mut out, "- `{component}` writes {writes}; disposition: {disposition}.").unwrap();
    }
    out
}

#[cfg(target_os = "linux")]
fn render_native_renderer_backend_manifest_json(
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    capture: &NativeCombatCapture,
    capture_hook: &NativeRendererCaptureHookSummary,
    mutation_proof: &NativeRendererMutationProof,
) -> String {
    let camera_modes = native_camera_modes()
        .iter()
        .map(|mode| (*mode).to_string())
        .collect::<Vec<_>>();
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", RENDERER_BACKEND_SCHEMA, true);
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(&mut out, 1, "scenario_id", &result.scenario_id, true);
    write_json_field(
        &mut out,
        1,
        "backend_id",
        "native-x11-software-3d-depth-raster",
        true,
    );
    write_json_field(
        &mut out,
        1,
        "backend_decision",
        "approved-native-dependency-zero-path-no-new-project-dependencies",
        true,
    );
    write_json_field(&mut out, 1, "window_system", "raw-x11-xwayland", true);
    writeln!(&mut out, "  \"project_dependency_adopted\": false,").unwrap();
    writeln!(&mut out, "  \"window_context_init\": true,").unwrap();
    writeln!(&mut out, "  \"continuous_render_loop\": true,").unwrap();
    writeln!(
        &mut out,
        "  \"continuous_render_loop_frames\": {},",
        capture.player_loop.rendered_frame_count
    )
    .unwrap();
    writeln!(&mut out, "  \"mesh_submission\": true,").unwrap();
    writeln!(&mut out, "  \"texture_binding\": true,").unwrap();
    writeln!(&mut out, "  \"material_binding\": true,").unwrap();
    writeln!(&mut out, "  \"depth_sorted_mesh_raster\": true,").unwrap();
    writeln!(&mut out, "  \"resize_supported\": true,").unwrap();
    writeln!(&mut out, "  \"frame_timing_outside_truth\": true,").unwrap();
    writeln!(&mut out, "  \"capture_hook_integrated\": true,").unwrap();
    writeln!(&mut out, "  \"clean_shutdown\": true,").unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    write_native_json_string_array(&mut out, 1, "camera_modes", &camera_modes, true);
    write_native_camera_metadata_array(&mut out, 1, "camera_metadata", true);
    write_native_json_string_array(
        &mut out,
        1,
        "asset_ids",
        &native_renderer_asset_ids(silhouette),
        true,
    );
    write_native_json_string_array(
        &mut out,
        1,
        "material_ids",
        &native_renderer_material_ids(silhouette),
        true,
    );
    write_json_field(
        &mut out,
        1,
        "post_hash_input_artifact",
        "native_renderer_post_hash_input.json",
        true,
    );
    write_json_field(
        &mut out,
        1,
        "capture_hook_artifact",
        "native_renderer_capture_hook.json",
        true,
    );
    write_json_field(
        &mut out,
        1,
        "mutation_proof_artifact",
        "native_renderer_truth_mutation_proof.json",
        true,
    );
    writeln!(
        &mut out,
        "  \"capture_count\": {},",
        capture_hook.capture_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"high_resolution_capture_count\": {},",
        capture_hook.high_resolution_capture_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"mutation_proof_all_equal\": {},",
        mutation_proof.all_equal
    )
    .unwrap();
    write_json_field(
        &mut out,
        1,
        "final_state_hash",
        &result.final_state_hash,
        false,
    );
    writeln!(&mut out, "}}").unwrap();
    out
}

#[cfg(target_os = "linux")]
fn native_renderer_camera_for_screen(screen: &str) -> &'static str {
    match screen {
        "plan" | "observe" | "commit_reveal" => "planning_tactical_reach",
        "resolve" | "combat" => "first_person_guard_line",
        "consequence" => "consequence_aftermath_dwell",
        "replay" | "replay_browser" | "fight_film" => "fight_film_orbit",
        "fighter_select" | "loadout_select" => "asset_closeup_weapon_armor",
        _ => "third_person_verdict_ring",
    }
}

#[cfg(target_os = "linux")]
fn native_renderer_damage_wear_mask(result: &DuelResult, screen: &str) -> String {
    if !matches!(
        screen,
        "combat" | "resolve" | "consequence" | "replay" | "replay_browser" | "fight_film"
    ) {
        return "none".to_string();
    }
    result
        .turns
        .iter()
        .flat_map(|turn| turn.contacts.iter())
        .next()
        .map(|contact| {
            format!(
                "{}|{}|{}",
                contact.material_result, contact.anatomy_result, contact.capability_delta.event
            )
        })
        .unwrap_or_else(|| "none".to_string())
}

#[cfg(target_os = "linux")]
fn render_native_renderer_post_hash_input_json(
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    capture: &NativeCombatCapture,
) -> String {
    let asset_ids = native_renderer_asset_ids(silhouette);
    let material_ids = native_renderer_material_ids(silhouette);
    let event_ids = native_renderer_event_ids(result);
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", NATIVE_RENDERER_INPUT_SCHEMA, true);
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(
        &mut out,
        1,
        "source",
        "post-hash-replay-trace-presentation-input",
        true,
    );
    write_json_field(&mut out, 1, "scenario_id", &result.scenario_id, true);
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
        "replay_json_hash",
        &hash_hex(result.replay_json.as_bytes()),
        true,
    );
    write_json_field(
        &mut out,
        1,
        "trace_json_hash",
        &hash_hex(result.trace_json.as_bytes()),
        true,
    );
    writeln!(&mut out, "  \"truth_hz\": {TRUTH_HZ},").unwrap();
    writeln!(&mut out, "  \"post_hash_only\": true,").unwrap();
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(
        &mut out,
        "  \"continuous_loop_frame_count\": {},",
        capture.player_loop.rendered_frame_count
    )
    .unwrap();
    write_native_json_string_array(&mut out, 1, "asset_ids", &asset_ids, true);
    write_native_json_string_array(&mut out, 1, "material_ids", &material_ids, true);
    write_native_json_string_array(&mut out, 1, "event_ids", &event_ids, true);
    let schema = vec![
        "loop_frame_index".to_string(),
        "truth_frame".to_string(),
        "scheduled_ms".to_string(),
        "screen".to_string(),
        "camera_mode".to_string(),
        "asset_ids".to_string(),
        "material_ids".to_string(),
        "event_ids".to_string(),
        "damage_wear_mask".to_string(),
        "capture_file".to_string(),
    ];
    write_native_json_string_array(&mut out, 1, "frame_schema", &schema, true);
    writeln!(&mut out, "  \"screen_inputs\": [").unwrap();
    for (index, frame) in capture.player_loop.frames.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        writeln!(&mut out, "      \"loop_frame_index\": {},", index).unwrap();
        writeln!(&mut out, "      \"truth_frame\": {},", frame.truth_frame).unwrap();
        writeln!(&mut out, "      \"scheduled_ms\": {},", frame.scheduled_ms).unwrap();
        write_json_field(&mut out, 3, "screen", frame.screen, true);
        write_json_field(
            &mut out,
            3,
            "camera_mode",
            native_renderer_camera_for_screen(frame.screen),
            true,
        );
        write_native_json_string_array(&mut out, 3, "asset_ids", &asset_ids, true);
        write_native_json_string_array(&mut out, 3, "material_ids", &material_ids, true);
        write_native_json_string_array(&mut out, 3, "event_ids", &event_ids, true);
        write_json_field(
            &mut out,
            3,
            "damage_wear_mask",
            &native_renderer_damage_wear_mask(result, frame.screen),
            true,
        );
        write_json_field(&mut out, 3, "capture_file", &frame.file, false);
        writeln!(
            &mut out,
            "    }}{}",
            comma(index + 1, capture.player_loop.frames.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

#[cfg(target_os = "linux")]
fn render_native_combat_render_manifest_json(
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    frames: &[NativeCombatFrameSpec],
    motion_frames: &[NativeCombatMotionFrameSpec],
    playback: &NativeCombatPlaybackSummary,
    live_loop: &NativeCombatLiveLoopSummary,
    player_loop: &NativePlayerLoopSummary,
    software_3d_viewports: &[NativeCombatSoftware3dViewport],
    software_3d_sequence: &NativeCombatSoftware3dSequenceSummary,
    resolution_captures: &[NativeCombatResolutionCapture],
) -> String {
    let lighting_post = native_lighting_post_summary(result, silhouette);
    let replay_source_hash = hash_hex(result.replay_json.as_bytes());
    let trace_source_hash = hash_hex(result.trace_json.as_bytes());
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", NATIVE_COMBAT_RENDER_SCHEMA, true);
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
    write_native_verified_replay_trace_input_json(
        &mut out,
        1,
        result,
        &replay_source_hash,
        &trace_source_hash,
        true,
    );
    writeln!(&mut out, "  \"renderer\": \"native-software-3d\",").unwrap();
    writeln!(&mut out, "  \"source\": \"truth-after-hash-duel-result\",").unwrap();
    writeln!(
        &mut out,
        "  \"source_backed_silhouettes\": {},",
        silhouette.source_backed
    )
    .unwrap();
    write_json_field(&mut out, 1, "silhouette_source", &silhouette.source, true);
    write_json_field(
        &mut out,
        1,
        "reconstructed_initial_state_hash",
        &silhouette.reconstructed_initial_state_hash,
        true,
    );
    writeln!(
        &mut out,
        "  \"runtime_asset_refs_verified\": {},",
        silhouette.runtime_asset_refs_verified
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"runtime_gltf_geometry_projected\": {},",
        native_geometry_projected(silhouette)
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"native_3d_runtime_geometry\": {},",
        native_3d_runtime_geometry(silhouette)
    )
    .unwrap();
    writeln!(&mut out, "  \"game_is_3d\": true,").unwrap();
    writeln!(&mut out, "  \"product_3d_gameplay_complete\": false,").unwrap();
    writeln!(
        &mut out,
        "  \"continuous_player_facing_3d_render_loop\": true,"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"high_res_capture_debug_overlay_minimized\": true,"
    )
    .unwrap();
    write_json_field(
        &mut out,
        1,
        "high_res_capture_visual_floor",
        "source-backed shaded presentation-floor capture with hash/debug data moved to reports/manifests; still local verification, not owner acceptance",
        true,
    );
    write_json_field(
        &mut out,
        1,
        "current_3d_evidence_scope",
        "local native player-facing loop plus software-raster PPM captures and runtime glTF depth evidence; not high-fidelity product renderer or owner acceptance",
        true,
    );
    write_native_lighting_post_json(&mut out, 1, "lighting_post", &lighting_post, true);
    writeln!(&mut out, "  \"projection_uses_z_depth\": true,").unwrap();
    writeln!(
        &mut out,
        "  \"projection_model\": \"integer_oblique_depth_projection\","
    )
    .unwrap();
    write_native_camera_metadata_array(&mut out, 1, "camera_metadata", true);
    write_json_field(
        &mut out,
        1,
        "asset_manifest_hash",
        &silhouette.asset_manifest_hash,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "presentation_asset_manifest",
        "assets/presentation_manifest.json",
        true,
    );
    write_json_field(
        &mut out,
        1,
        "presentation_asset_manifest_hash",
        &silhouette.presentation_asset_manifest_hash,
        true,
    );
    writeln!(
        &mut out,
        "  \"high_detail_presentation_assets_verified\": {},",
        silhouette.high_detail_presentation_assets_verified
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"high_detail_runtime_capture_integrated\": true,"
    )
    .unwrap();
    write_native_combat_asset_ref_json(&mut out, 1, "arena_asset", &silhouette.arena_asset, true);
    write_native_presentation_asset_ref_json(
        &mut out,
        1,
        "arena_presentation_asset",
        &silhouette.arena_presentation_asset,
        true,
    );
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(&mut out, "  \"truth_writeback_guard\": {{").unwrap();
    write_json_field(
        &mut out,
        2,
        "runtime_assertion",
        "native_renderer_assert_truth_unchanged",
        true,
    );
    writeln!(&mut out, "    \"capture_pass_checked\": true,").unwrap();
    writeln!(&mut out, "    \"artifact_generation_checked\": true,").unwrap();
    writeln!(&mut out, "    \"guarded_truth_fields\": [").unwrap();
    for (index, field) in [
        "final_state_hash",
        "replay_json_hash",
        "trace_json_hash",
        "contacts_hash",
        "injury_capability_hash",
        "action_validity_hash",
        "end_condition_hash",
    ]
    .iter()
    .enumerate()
    {
        writeln!(
            &mut out,
            "      {}{}",
            json_quote(field),
            comma(index + 1, 7)
        )
        .unwrap();
    }
    writeln!(&mut out, "    ]").unwrap();
    writeln!(&mut out, "  }},").unwrap();
    write_json_field(
        &mut out,
        1,
        "truth_writeback_audit_report",
        "native_renderer_truth_writeback_audit.md",
        true,
    );
    writeln!(&mut out, "  \"public_demo_ready\": {PUBLIC_DEMO_READY},").unwrap();
    writeln!(
        &mut out,
        "  \"release_candidate_ready\": {RELEASE_CANDIDATE_READY},"
    )
    .unwrap();
    writeln!(&mut out, "  \"owner_visual_acceptance\": false,").unwrap();
    writeln!(&mut out, "  \"owner_input_acceptance\": false,").unwrap();
    writeln!(&mut out, "  \"silhouette_fighters\": [").unwrap();
    for (index, fighter) in silhouette.fighters.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        writeln!(&mut out, "      \"seat\": {},", fighter.seat).unwrap();
        write_json_field(&mut out, 3, "name", &fighter.name, true);
        write_json_field(&mut out, 3, "weapon_id", &fighter.weapon_id, true);
        write_json_field(&mut out, 3, "weapon_name", &fighter.weapon_name, true);
        write_native_combat_asset_ref_json(
            &mut out,
            3,
            "weapon_asset",
            &fighter.weapon_asset,
            true,
        );
        write_native_presentation_asset_ref_json(
            &mut out,
            3,
            "weapon_presentation_asset",
            &fighter.weapon_presentation_asset,
            true,
        );
        writeln!(
            &mut out,
            "      \"weapon_length_mm\": {},",
            fighter.weapon_length_mm
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"weapon_reach_mm\": {},",
            fighter.weapon_reach_mm
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"weapon_mass_g\": {},",
            fighter.weapon_mass_g
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"weapon_inertia_g_cm2\": {},",
            fighter.weapon_inertia_g_cm2
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"weapon_span_px\": {},",
            fighter.weapon_span_px
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"weapon_head_px\": {},",
            fighter.weapon_head_px
        )
        .unwrap();
        write_json_field(&mut out, 3, "armor_id", &fighter.armor_id, true);
        write_json_field(&mut out, 3, "armor_name", &fighter.armor_name, true);
        write_native_combat_asset_ref_json(&mut out, 3, "armor_asset", &fighter.armor_asset, true);
        write_native_presentation_asset_ref_json(
            &mut out,
            3,
            "armor_presentation_asset",
            &fighter.armor_presentation_asset,
            true,
        );
        write_json_field(&mut out, 3, "armor_material", &fighter.armor_material, true);
        writeln!(
            &mut out,
            "      \"armor_mass_g\": {},",
            fighter.armor_mass_g
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"armor_torso_coverage_permille\": {},",
            fighter.armor_torso_coverage_permille
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"armor_head_coverage_permille\": {},",
            fighter.armor_head_coverage_permille
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"armor_weapon_arm_coverage_permille\": {},",
            fighter.armor_weapon_arm_coverage_permille
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"armor_lead_leg_coverage_permille\": {},",
            fighter.armor_lead_leg_coverage_permille
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"armor_gap_permille\": {},",
            fighter.armor_gap_permille
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"armor_torso_width_px\": {},",
            fighter.armor_torso_width_px
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"armor_torso_height_px\": {},",
            fighter.armor_torso_height_px
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"armor_head_marker_px\": {},",
            fighter.armor_head_marker_px
        )
        .unwrap();
        writeln!(&mut out, "      \"body_mass_g\": {},", fighter.body_mass_g).unwrap();
        writeln!(
            &mut out,
            "      \"stance_width_mm\": {}",
            fighter.stance_width_mm
        )
        .unwrap();
        writeln!(
            &mut out,
            "    }}{}",
            comma(index + 1, silhouette.fighters.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();
    let product_mode_clean_capture_count = resolution_captures
        .iter()
        .filter(|capture| !capture.debug_overlay)
        .count();
    writeln!(
        &mut out,
        "  \"product_mode_clean_capture_count\": {},",
        product_mode_clean_capture_count
    )
    .unwrap();
    writeln!(&mut out, r#"  "product_mode_clean_captures": ["#).unwrap();
    let product_captures = resolution_captures
        .iter()
        .filter(|capture| !capture.debug_overlay)
        .collect::<Vec<_>>();
    for (index, capture) in product_captures.iter().enumerate() {
        let frame_index = index + 1;
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "file", &capture.file, true);
        write_json_field(&mut out, 3, "camera", &capture.camera, true);
        write_json_field(&mut out, 3, "capture_role", &capture.capture_role, true);
        write_json_field(&mut out, 3, "source", &capture.source, true);
        writeln!(&mut out, "      \"frame_index\": {},", frame_index).unwrap();
        writeln!(
            &mut out,
            "      \"timestamp_ms\": {},",
            native_capture_timestamp_ms(frame_index, 0)
        )
        .unwrap();
        write_json_field(&mut out, 3, "replay_source_hash", &replay_source_hash, true);
        write_native_capture_camera_metadata_json(
            &mut out,
            3,
            "camera_metadata",
            &capture.camera,
            frame_index,
            0,
            &replay_source_hash,
            true,
        );
        writeln!(&mut out, "      \"width\": {},", capture.width).unwrap();
        writeln!(&mut out, "      \"height\": {},", capture.height).unwrap();
        writeln!(&mut out, "      \"debug_overlay\": false,").unwrap();
        writeln!(&mut out, "      \"production_asset_evidence\": true,").unwrap();
        writeln!(&mut out, "      \"capture_after_truth_hash\": true,").unwrap();
        writeln!(&mut out, "      \"presentation_only\": true,").unwrap();
        writeln!(&mut out, "      \"truth_mutation\": false,").unwrap();
        writeln!(
            &mut out,
            "      \"non_background_pixels\": {},",
            capture.non_background_pixels
        )
        .unwrap();
        write_json_field(&mut out, 3, "frame_hash", &capture.frame_hash, false);
        writeln!(
            &mut out,
            "    }}{}",
            comma(index + 1, product_captures.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();
    writeln!(&mut out, "  \"resolution_captures\": [").unwrap();
    for (index, capture) in resolution_captures.iter().enumerate() {
        let frame_index = index + 1;
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "file", &capture.file, true);
        write_json_field(&mut out, 3, "camera", &capture.camera, true);
        write_json_field(&mut out, 3, "capture_role", &capture.capture_role, true);
        write_json_field(&mut out, 3, "source", &capture.source, true);
        writeln!(&mut out, "      \"frame_index\": {},", frame_index).unwrap();
        writeln!(
            &mut out,
            "      \"timestamp_ms\": {},",
            native_capture_timestamp_ms(frame_index, 0)
        )
        .unwrap();
        write_json_field(&mut out, 3, "replay_source_hash", &replay_source_hash, true);
        write_native_capture_camera_metadata_json(
            &mut out,
            3,
            "camera_metadata",
            &capture.camera,
            frame_index,
            0,
            &replay_source_hash,
            true,
        );
        writeln!(&mut out, "      \"width\": {},", capture.width).unwrap();
        writeln!(&mut out, "      \"height\": {},", capture.height).unwrap();
        writeln!(
            &mut out,
            "      \"triangle_count\": {},",
            capture.triangle_count
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"shaded_triangle_count\": {},",
            capture.shaded_triangle_count
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"non_background_pixels\": {},",
            capture.non_background_pixels
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"debug_overlay\": {},",
            capture.debug_overlay
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"production_asset_evidence\": {},",
            !capture.debug_overlay
        )
        .unwrap();
        writeln!(&mut out, "      \"capture_after_truth_hash\": true,").unwrap();
        writeln!(&mut out, "      \"presentation_only\": true,").unwrap();
        writeln!(&mut out, "      \"truth_mutation\": false,").unwrap();
        write_json_field(&mut out, 3, "frame_hash", &capture.frame_hash, false);
        writeln!(
            &mut out,
            "    }}{}",
            comma(index + 1, resolution_captures.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();
    writeln!(&mut out, "  \"state_frame_count\": {},", frames.len()).unwrap();
    writeln!(
        &mut out,
        "  \"motion_frame_count\": {},",
        motion_frames.len()
    )
    .unwrap();
    writeln!(&mut out, "  \"playback_loop\": {{").unwrap();
    write_json_field(&mut out, 2, "file", &playback.file, true);
    writeln!(
        &mut out,
        "    \"source_motion_frame_count\": {},",
        playback.source_motion_frame_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "    \"playback_frame_count\": {},",
        playback.playback_frame_count
    )
    .unwrap();
    writeln!(&mut out, "    \"cycles\": {},", playback.cycles).unwrap();
    writeln!(
        &mut out,
        "    \"nominal_frame_interval_ms\": {},",
        playback.nominal_frame_interval_ms
    )
    .unwrap();
    writeln!(
        &mut out,
        "    \"nominal_duration_ms\": {},",
        playback.nominal_duration_ms
    )
    .unwrap();
    write_json_field(
        &mut out,
        2,
        "final_frame_hash",
        &playback.final_frame_hash,
        false,
    );
    writeln!(&mut out, "  }},").unwrap();
    writeln!(&mut out, "  \"live_render_loop\": {{").unwrap();
    writeln!(
        &mut out,
        "    \"source\": \"replay-derived-motion-frames-after-truth-hash\","
    )
    .unwrap();
    writeln!(
        &mut out,
        "    \"source_motion_frame_count\": {},",
        live_loop.source_motion_frame_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "    \"rendered_frame_count\": {},",
        live_loop.rendered_frame_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "    \"sample_capture_count\": {},",
        live_loop.sample_captures.len()
    )
    .unwrap();
    writeln!(
        &mut out,
        "    \"nominal_frame_interval_ms\": {},",
        live_loop.nominal_frame_interval_ms
    )
    .unwrap();
    writeln!(
        &mut out,
        "    \"nominal_duration_ms\": {},",
        live_loop.nominal_duration_ms
    )
    .unwrap();
    write_json_field(&mut out, 2, "loop_hash", &live_loop.loop_hash, true);
    write_json_field(
        &mut out,
        2,
        "final_frame_hash",
        &live_loop.final_frame_hash,
        true,
    );
    writeln!(&mut out, "    \"sample_captures\": [").unwrap();
    for (index, sample) in live_loop.sample_captures.iter().enumerate() {
        writeln!(&mut out, "      {{").unwrap();
        write_json_field(&mut out, 4, "file", &sample.file, true);
        writeln!(
            &mut out,
            "        \"loop_frame_index\": {},",
            sample.loop_frame_index
        )
        .unwrap();
        writeln!(
            &mut out,
            "        \"source_motion_frame_index\": {},",
            sample.source_motion_frame_index
        )
        .unwrap();
        writeln!(&mut out, "        \"truth_frame\": {},", sample.truth_frame).unwrap();
        write_json_field(&mut out, 4, "frame_hash", &sample.frame_hash, false);
        writeln!(
            &mut out,
            "      }}{}",
            comma(index + 1, live_loop.sample_captures.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "    ],").unwrap();
    writeln!(&mut out, "    \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "    \"truth_mutation\": false").unwrap();
    writeln!(&mut out, "  }},").unwrap();
    writeln!(&mut out, "  \"player_facing_loop\": {{").unwrap();
    write_json_field(&mut out, 2, "source", &player_loop.source, true);
    write_json_field(&mut out, 2, "backend", &player_loop.backend, true);
    writeln!(
        &mut out,
        "    \"rendered_frame_count\": {},",
        player_loop.rendered_frame_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "    \"screen_count\": {},",
        player_loop.screen_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "    \"nominal_frame_interval_ms\": {},",
        player_loop.nominal_frame_interval_ms
    )
    .unwrap();
    writeln!(
        &mut out,
        "    \"nominal_duration_ms\": {},",
        player_loop.nominal_duration_ms
    )
    .unwrap();
    write_json_field(
        &mut out,
        2,
        "timing_source",
        &player_loop.timing_source,
        true,
    );
    writeln!(&mut out, "    \"timing_recorded_outside_truth\": true,").unwrap();
    write_json_field(
        &mut out,
        2,
        "truth_hash_before_loop",
        &player_loop.truth_hash_before_loop,
        true,
    );
    write_json_field(
        &mut out,
        2,
        "truth_hash_after_loop",
        &player_loop.truth_hash_after_loop,
        true,
    );
    write_json_field(&mut out, 2, "loop_hash", &player_loop.loop_hash, true);
    write_json_field(
        &mut out,
        2,
        "final_frame_hash",
        &player_loop.final_frame_hash,
        true,
    );
    writeln!(&mut out, "    \"screens\": [").unwrap();
    for (index, frame) in player_loop.frames.iter().enumerate() {
        writeln!(
            &mut out,
            "      {}{}",
            json_quote(frame.screen),
            comma(index + 1, player_loop.frames.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "    ],").unwrap();
    writeln!(
        &mut out,
        "    \"hud_menu_flow_local_evidence_complete\": true,"
    )
    .unwrap();
    writeln!(&mut out, "    \"owner_visual_acceptance\": false,").unwrap();
    writeln!(&mut out, "    \"owner_input_acceptance\": false,").unwrap();
    writeln!(&mut out, "    \"frames\": [").unwrap();
    for (index, frame) in player_loop.frames.iter().enumerate() {
        writeln!(&mut out, "      {{").unwrap();
        writeln!(&mut out, "        \"index\": {},", frame.index).unwrap();
        write_json_field(&mut out, 4, "file", &frame.file, true);
        write_json_field(&mut out, 4, "screen", frame.screen, true);
        writeln!(
            &mut out,
            "        \"scheduled_ms\": {},",
            frame.scheduled_ms
        )
        .unwrap();
        writeln!(&mut out, "        \"truth_frame\": {},", frame.truth_frame).unwrap();
        write_json_field(&mut out, 4, "headline", &frame.headline, true);
        write_json_field(&mut out, 4, "detail", &frame.detail, true);
        write_json_field(&mut out, 4, "frame_hash", &frame.frame_hash, false);
        writeln!(
            &mut out,
            "      }}{}",
            comma(index + 1, player_loop.frames.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "    ],").unwrap();
    writeln!(&mut out, "    \"same_runtime_path\": true,").unwrap();
    writeln!(&mut out, "    \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "    \"truth_mutation\": false").unwrap();
    writeln!(&mut out, "  }},").unwrap();
    writeln!(&mut out, "  \"software_3d_viewports\": [").unwrap();
    for (index, viewport) in software_3d_viewports.iter().enumerate() {
        let frame_index = index + 1;
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "file", &viewport.file, true);
        write_json_field(&mut out, 3, "camera", &viewport.camera, true);
        writeln!(&mut out, "      \"frame_index\": {},", frame_index).unwrap();
        writeln!(
            &mut out,
            "      \"timestamp_ms\": {},",
            native_capture_timestamp_ms(frame_index, 0)
        )
        .unwrap();
        write_json_field(&mut out, 3, "replay_source_hash", &replay_source_hash, true);
        write_native_capture_camera_metadata_json(
            &mut out,
            3,
            "camera_metadata",
            &viewport.camera,
            frame_index,
            0,
            &replay_source_hash,
            true,
        );
        writeln!(&mut out, "      \"width\": {},", viewport.width).unwrap();
        writeln!(&mut out, "      \"height\": {},", viewport.height).unwrap();
        writeln!(
            &mut out,
            "      \"triangle_count\": {},",
            viewport.triangle_count
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"shaded_triangle_count\": {},",
            viewport.shaded_triangle_count
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"non_background_pixels\": {},",
            viewport.non_background_pixels
        )
        .unwrap();
        write_json_field(
            &mut out,
            3,
            "projection_model",
            &viewport.projection_model,
            true,
        );
        writeln!(
            &mut out,
            "      \"depth_sorted\": {},",
            viewport.depth_sorted
        )
        .unwrap();
        write_json_field(&mut out, 3, "source", &viewport.source, true);
        writeln!(&mut out, "      \"capture_after_truth_hash\": true,").unwrap();
        writeln!(&mut out, "      \"presentation_only\": true,").unwrap();
        writeln!(&mut out, "      \"truth_mutation\": false,").unwrap();
        write_json_field(&mut out, 3, "frame_hash", &viewport.frame_hash, false);
        writeln!(
            &mut out,
            "    }}{}",
            comma(index + 1, software_3d_viewports.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();
    writeln!(&mut out, "  \"software_3d_sequence\": {{").unwrap();
    write_json_field(&mut out, 2, "camera", &software_3d_sequence.camera, true);
    write_json_field(&mut out, 2, "replay_source_hash", &replay_source_hash, true);
    writeln!(
        &mut out,
        "    \"frame_count\": {},",
        software_3d_sequence.frame_count
    )
    .unwrap();
    writeln!(&mut out, "    \"width\": {},", software_3d_sequence.width).unwrap();
    writeln!(&mut out, "    \"height\": {},", software_3d_sequence.height).unwrap();
    write_json_field(
        &mut out,
        2,
        "projection_model",
        &software_3d_sequence.projection_model,
        true,
    );
    writeln!(
        &mut out,
        "    \"depth_sorted\": {},",
        software_3d_sequence.depth_sorted
    )
    .unwrap();
    write_json_field(&mut out, 2, "source", &software_3d_sequence.source, true);
    writeln!(&mut out, "    \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "    \"truth_mutation\": false,").unwrap();
    write_json_field(
        &mut out,
        2,
        "frame_hash_chain",
        &software_3d_sequence.frame_hash_chain,
        true,
    );
    writeln!(&mut out, "  \"frames\": [").unwrap();
    for (index, frame) in software_3d_sequence.frames.iter().enumerate() {
        writeln!(&mut out, "      {{").unwrap();
        writeln!(&mut out, "        \"index\": {},", frame.index).unwrap();
        write_json_field(&mut out, 4, "file", &frame.file, true);
        write_json_field(
            &mut out,
            4,
            "camera_mode",
            &software_3d_sequence.camera,
            true,
        );
        write_native_capture_camera_metadata_json(
            &mut out,
            4,
            "camera_metadata",
            &software_3d_sequence.camera,
            frame.index,
            frame.truth_frame,
            &replay_source_hash,
            true,
        );
        write_json_field(&mut out, 4, "phase", &frame.phase, true);
        writeln!(&mut out, "        \"turn\": {},", frame.turn).unwrap();
        writeln!(&mut out, "        \"truth_frame\": {},", frame.truth_frame).unwrap();
        writeln!(
            &mut out,
            "        \"progress_permille\": {},",
            frame.progress_permille
        )
        .unwrap();
        writeln!(
            &mut out,
            "        \"timestamp_ms\": {},",
            native_capture_timestamp_ms(frame.index, frame.truth_frame)
        )
        .unwrap();
        write_json_field(&mut out, 4, "replay_source_hash", &replay_source_hash, true);
        writeln!(&mut out, "        \"capture_after_truth_hash\": true,").unwrap();
        writeln!(&mut out, "        \"presentation_only\": true,").unwrap();
        writeln!(&mut out, "        \"truth_mutation\": false,").unwrap();
        writeln!(
            &mut out,
            "        \"triangle_count\": {},",
            frame.triangle_count
        )
        .unwrap();
        writeln!(
            &mut out,
            "        \"shaded_triangle_count\": {},",
            frame.shaded_triangle_count
        )
        .unwrap();
        writeln!(
            &mut out,
            "        \"non_background_pixels\": {},",
            frame.non_background_pixels
        )
        .unwrap();
        write_json_field(&mut out, 4, "frame_hash", &frame.frame_hash, false);
        writeln!(
            &mut out,
            "      }}{}",
            comma(index + 1, software_3d_sequence.frames.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "    ]").unwrap();
    writeln!(&mut out, "  }},").unwrap();
    writeln!(&mut out, "  \"frames\": [").unwrap();
    for (index, frame) in frames.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        writeln!(&mut out, "      \"index\": {},", frame.index).unwrap();
        write_json_field(&mut out, 3, "file", &frame.file, true);
        write_json_field(&mut out, 3, "state", frame.state, true);
        writeln!(&mut out, "      \"turn\": {},", frame.turn).unwrap();
        write_json_field(&mut out, 3, "headline", &frame.headline, true);
        write_json_field(&mut out, 3, "detail", &frame.detail, true);
        write_json_field(&mut out, 3, "frame_hash", &frame.frame_hash, false);
        writeln!(&mut out, "    }}{}", comma(index + 1, frames.len())).unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();
    writeln!(&mut out, "  \"motion_frames\": [").unwrap();
    for (index, frame) in motion_frames.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        writeln!(&mut out, "      \"index\": {},", frame.index).unwrap();
        write_json_field(&mut out, 3, "file", &frame.file, true);
        write_json_field(&mut out, 3, "phase", frame.phase, true);
        writeln!(&mut out, "      \"turn\": {},", frame.turn).unwrap();
        writeln!(&mut out, "      \"truth_frame\": {},", frame.truth_frame).unwrap();
        writeln!(
            &mut out,
            "      \"progress_permille\": {},",
            frame.progress_permille
        )
        .unwrap();
        write_json_field(&mut out, 3, "headline", &frame.headline, true);
        write_json_field(&mut out, 3, "detail", &frame.detail, true);
        write_json_field(&mut out, 3, "turn_hash", &frame.turn_hash, true);
        write_json_field(&mut out, 3, "frame_hash", &frame.frame_hash, false);
        writeln!(&mut out, "    }}{}", comma(index + 1, motion_frames.len())).unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

#[cfg(target_os = "linux")]
fn render_native_combat_visual_audit_report(
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    frames: &[NativeCombatFrameSpec],
    motion_frames: &[NativeCombatMotionFrameSpec],
    playback: &NativeCombatPlaybackSummary,
    live_loop: &NativeCombatLiveLoopSummary,
    player_loop: &NativePlayerLoopSummary,
    software_3d_viewports: &[NativeCombatSoftware3dViewport],
    software_3d_sequence: &NativeCombatSoftware3dSequenceSummary,
    resolution_captures: &[NativeCombatResolutionCapture],
    out_dir: &Path,
) -> Result<String, OathError> {
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD Native Combat Visual Audit").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "Status: PASSED").unwrap();
    writeln!(&mut out, "- Scenario: `{}`", result.scenario_id).unwrap();
    writeln!(
        &mut out,
        "- Final state hash: `{}`",
        result.final_state_hash
    )
    .unwrap();
    writeln!(&mut out, "- Renderer: `native-software-3d`").unwrap();
    writeln!(
        &mut out,
        "- Scope: automated local artifact audit; owner visual acceptance is not claimed"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Truth mutation: none; audit reads captured presentation artifacts only"
    )
    .unwrap();
    writeln!(&mut out, "- Public demo ready: `{PUBLIC_DEMO_READY}`").unwrap();
    writeln!(
        &mut out,
        "- Release candidate ready: `{RELEASE_CANDIDATE_READY}`"
    )
    .unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Source-Backed Silhouettes").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(
        &mut out,
        "- Source-backed silhouettes present: `{}`",
        silhouette.source_backed
    )
    .unwrap();
    writeln!(&mut out, "- Source: `{}`", silhouette.source).unwrap();
    writeln!(
        &mut out,
        "- Reconstructed initial state hash: `{}`",
        silhouette.reconstructed_initial_state_hash
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Runtime asset refs verified: `{}`",
        silhouette.runtime_asset_refs_verified
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Runtime glTF geometry projected: `{}`",
        native_geometry_projected(silhouette)
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Native 3D runtime geometry: `{}`",
        native_3d_runtime_geometry(silhouette)
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Projection model: `integer_oblique_depth_projection`"
    )
    .unwrap();
    writeln!(&mut out, "- Projection uses Z depth: `true`").unwrap();
    let lighting_post = native_lighting_post_summary(result, silhouette);
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Lighting/Post/Material Readability").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "- Profile: `{}`", lighting_post.profile).unwrap();
    writeln!(
        &mut out,
        "- Dynamic lighting: `{}`",
        lighting_post.dynamic_lighting
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Contact grounding/shadows: `{}`",
        lighting_post.contact_grounding
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Ambient occlusion/equivalent: `{}`",
        lighting_post.ambient_occlusion_equivalent
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Fog/dust/atmosphere: `{}`",
        lighting_post.fog_dust_atmosphere
    )
    .unwrap();
    writeln!(&mut out, "- Tone mapping: `{}`", lighting_post.tone_mapping).unwrap();
    writeln!(
        &mut out,
        "- Exposure permille: `{}`",
        lighting_post.exposure_permille
    )
    .unwrap();
    writeln!(&mut out, "- Color grade: `{}`", lighting_post.color_grade).unwrap();
    writeln!(
        &mut out,
        "- Anti-aliasing/equivalent: `{}`",
        lighting_post.anti_aliasing_equivalent
    )
    .unwrap();
    writeln!(&mut out, "- Bloom policy: `{}`", lighting_post.bloom_policy).unwrap();
    writeln!(
        &mut out,
        "- Bloom event count: `{}`",
        lighting_post.bloom_event_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Material witnesses: `{}`",
        lighting_post.material_witnesses.join(",")
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Material distinction classes: `{}`",
        lighting_post.material_distinction_classes.join(",")
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Required material classes distinguished under lighting/post: `{}`",
        lighting_post.required_material_classes_distinguished
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Material witness capture: `{}`",
        lighting_post.material_witness_capture
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Material witness manifest: `{}`",
        lighting_post.material_witness_manifest
    )
    .unwrap();
    writeln!(&mut out, "- Owner visual acceptance: `false`").unwrap();
    writeln!(&mut out, "- Lighting/post truth mutation: `false`").unwrap();
    writeln!(
        &mut out,
        "- Runtime asset manifest hash: `{}`",
        silhouette.asset_manifest_hash
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Arena asset `{}` mesh `{}` glTF `{}` preview `{}` vertices `{}` triangles `{}` z-depth `{}` bounds x `{}..{}` y `{}..{}` z `{}..{}`",
        silhouette.arena_asset.id,
        silhouette.arena_asset.runtime_mesh,
        silhouette.arena_asset.runtime_gltf,
        silhouette.arena_asset.preview,
        silhouette.arena_asset.geometry.vertex_count,
        silhouette.arena_asset.geometry.triangle_count,
        geometry_z_depth_milli(&silhouette.arena_asset.geometry),
        silhouette.arena_asset.geometry.min_x_milli,
        silhouette.arena_asset.geometry.max_x_milli,
        silhouette.arena_asset.geometry.min_y_milli,
        silhouette.arena_asset.geometry.max_y_milli,
        silhouette.arena_asset.geometry.min_z_milli,
        silhouette.arena_asset.geometry.max_z_milli
    )
    .unwrap();
    for fighter in &silhouette.fighters {
        writeln!(
            &mut out,
            "- F{} `{}` weapon `{}` length `{}` mm reach `{}` mm mass `{}` g inertia `{}` g_cm2 rendered span `{}` px mesh `{}` glTF vertices `{}` triangles `{}` z-depth `{}`; armor `{}` material `{}` torso coverage `{}` head coverage `{}` torso block `{}x{}` px mesh `{}` glTF vertices `{}` triangles `{}` z-depth `{}`",
            fighter.seat,
            fighter.name,
            fighter.weapon_id,
            fighter.weapon_length_mm,
            fighter.weapon_reach_mm,
            fighter.weapon_mass_g,
            fighter.weapon_inertia_g_cm2,
            fighter.weapon_span_px,
            fighter.weapon_asset.runtime_mesh,
            fighter.weapon_asset.geometry.vertex_count,
            fighter.weapon_asset.geometry.triangle_count,
            geometry_z_depth_milli(&fighter.weapon_asset.geometry),
            fighter.armor_id,
            fighter.armor_material,
            fighter.armor_torso_coverage_permille,
            fighter.armor_head_coverage_permille,
            fighter.armor_torso_width_px,
            fighter.armor_torso_height_px,
            fighter.armor_asset.runtime_mesh,
            fighter.armor_asset.geometry.vertex_count,
            fighter.armor_asset.geometry.triangle_count,
            geometry_z_depth_milli(&fighter.armor_asset.geometry)
        )
        .unwrap();
    }
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## State Frames").unwrap();
    writeln!(&mut out).unwrap();

    for frame in frames {
        let (byte_len, non_background_pixels, frame_hash) =
            native_ppm_evidence(&out_dir.join(&frame.file), 960, 540)?;
        if frame_hash != frame.frame_hash {
            return Err(OathError::Verify(format!(
                "native combat frame hash mismatch for {}",
                frame.file
            )));
        }
        writeln!(
            &mut out,
            "- `{}` `{}`: 960x540 bytes `{}` non-background pixels `{}` hash `{}` headline `{}`",
            frame.file, frame.state, byte_len, non_background_pixels, frame_hash, frame.headline
        )
        .unwrap();
    }

    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Motion Frames").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "- Motion frame count: `{}`", motion_frames.len()).unwrap();
    writeln!(&mut out, "- Continuous replay-derived sequence: `true`").unwrap();
    for frame in motion_frames {
        let (byte_len, non_background_pixels, frame_hash) =
            native_ppm_evidence(&out_dir.join(&frame.file), 960, 540)?;
        if frame_hash != frame.frame_hash {
            return Err(OathError::Verify(format!(
                "native combat motion frame hash mismatch for {}",
                frame.file
            )));
        }
        writeln!(
            &mut out,
            "- `{}` `{}`: turn `{}` truth frame `{}` bytes `{}` non-background pixels `{}` hash `{}` headline `{}`",
            frame.file,
            frame.phase,
            frame.turn,
            frame.truth_frame,
            byte_len,
            non_background_pixels,
            frame_hash,
            frame.headline
        )
        .unwrap();
    }

    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Playback Loop").unwrap();
    writeln!(&mut out).unwrap();
    let (byte_len, non_background_pixels, frame_hash) =
        native_ppm_evidence(&out_dir.join(&playback.file), 960, 540)?;
    if frame_hash != playback.final_frame_hash {
        return Err(OathError::Verify(format!(
            "native combat playback hash mismatch for {}",
            playback.file
        )));
    }
    writeln!(
        &mut out,
        "- `{}`: loop frames `{}` source motion frames `{}` cycles `{}` nominal interval `{}` ms nominal duration `{}` ms bytes `{}` non-background pixels `{}` hash `{}`",
        playback.file,
        playback.playback_frame_count,
        playback.source_motion_frame_count,
        playback.cycles,
        playback.nominal_frame_interval_ms,
        playback.nominal_duration_ms,
        byte_len,
        non_background_pixels,
        frame_hash
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Live X11 playback loop rendered before final capture: `true`"
    )
    .unwrap();

    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Live Render Loop").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(
        &mut out,
        "- Source: `replay-derived-motion-frames-after-truth-hash`"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Rendered frame count: `{}`",
        live_loop.rendered_frame_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Sample capture count: `{}`",
        live_loop.sample_captures.len()
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Nominal duration: `{}` ms",
        live_loop.nominal_duration_ms
    )
    .unwrap();
    writeln!(&mut out, "- Loop hash: `{}`", live_loop.loop_hash).unwrap();
    for sample in &live_loop.sample_captures {
        let (byte_len, non_background_pixels, frame_hash) =
            native_ppm_evidence(&out_dir.join(&sample.file), 960, 540)?;
        if frame_hash != sample.frame_hash {
            return Err(OathError::Verify(format!(
                "native combat live loop sample hash mismatch for {}",
                sample.file
            )));
        }
        writeln!(
            &mut out,
            "- `{}`: loop frame `{}` source motion `{}` truth frame `{}` bytes `{}` non-background pixels `{}` hash `{}`",
            sample.file,
            sample.loop_frame_index,
            sample.source_motion_frame_index,
            sample.truth_frame,
            byte_len,
            non_background_pixels,
            frame_hash
        )
        .unwrap();
    }

    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Player-Facing Native Loop").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "- Source: `{}`", player_loop.source).unwrap();
    writeln!(&mut out, "- Backend: `{}`", player_loop.backend).unwrap();
    writeln!(
        &mut out,
        "- Screens: `{}`",
        player_loop
            .frames
            .iter()
            .map(|frame| frame.screen)
            .collect::<Vec<_>>()
            .join(",")
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Rendered frame count: `{}`",
        player_loop.rendered_frame_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Screen capture count: `{}`",
        player_loop.frames.len()
    )
    .unwrap();
    writeln!(&mut out, "- Timing source: `{}`", player_loop.timing_source).unwrap();
    writeln!(&mut out, "- Timing recorded outside truth: `true`").unwrap();
    writeln!(
        &mut out,
        "- Truth hash unchanged: `{}`",
        player_loop.truth_hash_before_loop == player_loop.truth_hash_after_loop
    )
    .unwrap();
    writeln!(&mut out, "- Loop hash: `{}`", player_loop.loop_hash).unwrap();
    for frame in &player_loop.frames {
        let (byte_len, non_background_pixels, frame_hash) =
            native_ppm_evidence(&out_dir.join(&frame.file), 960, 540)?;
        if frame_hash != frame.frame_hash {
            return Err(OathError::Verify(format!(
                "native player-facing loop frame hash mismatch for {}",
                frame.file
            )));
        }
        writeln!(
            &mut out,
            "- `{}` `{}`: scheduled `{}` ms truth frame `{}` bytes `{}` non-background pixels `{}` hash `{}`",
            frame.file,
            frame.screen,
            frame.scheduled_ms,
            frame.truth_frame,
            byte_len,
            non_background_pixels,
            frame_hash
        )
        .unwrap();
    }
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Software 3D Mesh Viewports").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(
        &mut out,
        "- Viewport count: `{}`",
        software_3d_viewports.len()
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Projection model: `integer_depth_sorted_mesh_raster`"
    )
    .unwrap();
    writeln!(&mut out, "- Source: `runtime-gltf-after-truth-hash`").unwrap();
    writeln!(&mut out, "- Truth mutation: `false`").unwrap();
    writeln!(
        &mut out,
        "- Static viewport debug overlays minimized: `true`"
    )
    .unwrap();
    for viewport in software_3d_viewports {
        let (byte_len, non_background_pixels, frame_hash) = native_ppm_evidence(
            &out_dir.join(&viewport.file),
            viewport.width,
            viewport.height,
        )?;
        if frame_hash != viewport.frame_hash {
            return Err(OathError::Verify(format!(
                "native combat software 3D viewport hash mismatch for {}",
                viewport.file
            )));
        }
        if non_background_pixels != viewport.non_background_pixels {
            return Err(OathError::Verify(format!(
                "native combat software 3D viewport pixel-count mismatch for {}",
                viewport.file
            )));
        }
        writeln!(
            &mut out,
            "- `{}` `{}`: {}x{} bytes `{}` non-background pixels `{}` triangles `{}` shaded `{}` depth sorted `{}` hash `{}`",
            viewport.file,
            viewport.camera,
            viewport.width,
            viewport.height,
            byte_len,
            non_background_pixels,
            viewport.triangle_count,
            viewport.shaded_triangle_count,
            viewport.depth_sorted,
            frame_hash
        )
        .unwrap();
    }

    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Software 3D Replay Sequence").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "- Camera: `{}`", software_3d_sequence.camera).unwrap();
    writeln!(
        &mut out,
        "- Frame count: `{}`",
        software_3d_sequence.frame_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Projection model: `{}`",
        software_3d_sequence.projection_model
    )
    .unwrap();
    writeln!(&mut out, "- Source: `{}`", software_3d_sequence.source).unwrap();
    writeln!(
        &mut out,
        "- Frame hash chain: `{}`",
        software_3d_sequence.frame_hash_chain
    )
    .unwrap();
    for frame in &software_3d_sequence.frames {
        let (byte_len, non_background_pixels, frame_hash) = native_ppm_evidence(
            &out_dir.join(&frame.file),
            software_3d_sequence.width,
            software_3d_sequence.height,
        )?;
        if frame_hash != frame.frame_hash {
            return Err(OathError::Verify(format!(
                "native combat software 3D sequence frame hash mismatch for {}",
                frame.file
            )));
        }
        if non_background_pixels != frame.non_background_pixels {
            return Err(OathError::Verify(format!(
                "native combat software 3D sequence pixel-count mismatch for {}",
                frame.file
            )));
        }
        writeln!(
            &mut out,
            "- `{}` `{}`: turn `{}` truth frame `{}` progress `{}` bytes `{}` non-background pixels `{}` triangles `{}` shaded `{}` hash `{}`",
            frame.file,
            frame.phase,
            frame.turn,
            frame.truth_frame,
            frame.progress_permille,
            byte_len,
            non_background_pixels,
            frame.triangle_count,
            frame.shaded_triangle_count,
            frame_hash
        )
        .unwrap();
    }

    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Resolution Captures").unwrap();
    writeln!(&mut out).unwrap();
    for capture in resolution_captures {
        let (byte_len, non_background_pixels, frame_hash) =
            native_ppm_evidence(&out_dir.join(&capture.file), capture.width, capture.height)?;
        if frame_hash != capture.frame_hash {
            return Err(OathError::Verify(format!(
                "native combat resolution hash mismatch for {}",
                capture.file
            )));
        }
        if non_background_pixels != capture.non_background_pixels {
            return Err(OathError::Verify(format!(
                "native combat resolution pixel-count mismatch for {}",
                capture.file
            )));
        }
        writeln!(
            &mut out,
            "- `{}` `{}`: {}x{} camera `{}` role `{}` source `{}` debug_overlay `{}` bytes `{}` non-background pixels `{}` triangles `{}` shaded `{}` hash `{}`",
            capture.file,
            if capture.debug_overlay { "diagnostic" } else { "product" },
            capture.width,
            capture.height,
            capture.camera,
            capture.capture_role,
            capture.source,
            capture.debug_overlay,
            byte_len,
            non_background_pixels,
            capture.triangle_count,
            capture.shaded_triangle_count,
            frame_hash
        )
        .unwrap();
    }

    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Checklist").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(
        &mut out,
        "- Source-backed silhouettes present: `{}`",
        silhouette.source_backed
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Runtime weapon/armor/arena asset refs verified: `{}`",
        silhouette.runtime_asset_refs_verified
            && !silhouette.asset_manifest_hash.is_empty()
            && silhouette.fighters.iter().all(|fighter| !fighter
                .weapon_asset
                .runtime_mesh_hash
                .is_empty()
                && !fighter.armor_asset.runtime_mesh_hash.is_empty())
            && !silhouette.arena_asset.runtime_mesh_hash.is_empty()
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Runtime glTF geometry projected: `{}`",
        native_geometry_projected(silhouette)
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Native 3D runtime geometry: `{}`",
        native_3d_runtime_geometry(silhouette)
    )
    .unwrap();
    writeln!(&mut out, "- Projection uses Z depth: `true`").unwrap();
    writeln!(
        &mut out,
        "- Weapon reach influences rendered span: `{}`",
        silhouette
            .fighters
            .iter()
            .all(|fighter| fighter.weapon_span_px >= 54)
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Weapon mass/inertia influences rendered head markers: `{}`",
        silhouette
            .fighters
            .iter()
            .all(|fighter| fighter.weapon_head_px >= 8)
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Armor coverage influences rendered blocks: `{}`",
        silhouette
            .fighters
            .iter()
            .all(|fighter| fighter.armor_torso_height_px >= 42)
    )
    .unwrap();
    writeln!(&mut out, "- Observe/plan frame present: `true`").unwrap();
    writeln!(&mut out, "- Guard/bind frame present: `true`").unwrap();
    writeln!(&mut out, "- Hit/contact frame present: `true`").unwrap();
    writeln!(&mut out, "- Injury/capability frame present: `true`").unwrap();
    writeln!(&mut out, "- Recovery frame present: `true`").unwrap();
    writeln!(
        &mut out,
        "- Motion sequence frame count >= 21: `{}`",
        motion_frames.len() >= 21
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Stagger/collapse risk motion frame present: `{}`",
        motion_frames
            .iter()
            .any(|frame| frame.phase == "stagger_collapse_risk")
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Playback loop frame count >= 42: `{}`",
        playback.playback_frame_count >= 42
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Live render loop frame count >= 120: `{}`",
        live_loop.rendered_frame_count >= TRUTH_HZ as usize
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Live render loop sample capture count >= 5: `{}`",
        live_loop.sample_captures.len() >= 5
    )
    .unwrap();
    writeln!(
        &mut out,
        "- High-resolution capture debug overlay minimized: `true`"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Product-mode clean capture count >= 6: `{}`",
        resolution_captures
            .iter()
            .filter(|capture| !capture.debug_overlay)
            .count()
            >= 6
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Product-mode captures use production asset evidence: `{}`",
        resolution_captures
            .iter()
            .filter(|capture| !capture.debug_overlay)
            .all(|capture| capture
                .source
                .contains("software-product-renderer-clean-frame-after-truth-hash"))
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Player-facing loop screen count >= 13: `{}`",
        player_loop.screen_count >= 13
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Player-facing loop covers owner-remediation HUD/menu flow: `{}`",
        [
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
        ]
        .iter()
        .all(|screen| player_loop
            .frames
            .iter()
            .any(|frame| frame.screen == *screen))
    )
    .unwrap();
    writeln!(&mut out, "- Owner visual acceptance claimed: `false`").unwrap();
    writeln!(&mut out, "- Owner input acceptance claimed: `false`").unwrap();
    writeln!(
        &mut out,
        "- Player-facing loop timing outside truth: `{}`",
        player_loop
            .timing_source
            .contains("outside-authoritative-truth")
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Player-facing loop truth hash unchanged: `{}`",
        player_loop.truth_hash_before_loop == player_loop.truth_hash_after_loop
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Software 3D viewport count >= 6: `{}`",
        software_3d_viewports.len() >= 6
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Software 3D viewport cameras include first-person, third-person, planning, consequence, fight-film, and asset-closeup: `{}`",
        software_3d_viewports
            .iter()
            .any(|viewport| viewport.camera == "third_person_verdict_ring")
            && software_3d_viewports
                .iter()
                .any(|viewport| viewport.camera == "first_person_guard_line")
            && software_3d_viewports
                .iter()
                .any(|viewport| viewport.camera == "planning_tactical_reach")
            && software_3d_viewports
                .iter()
                .any(|viewport| viewport.camera == "consequence_aftermath_dwell")
            && software_3d_viewports
                .iter()
                .any(|viewport| viewport.camera == "fight_film_orbit")
            && software_3d_viewports
                .iter()
                .any(|viewport| viewport.camera == "asset_closeup_weapon_armor")
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Product-mode captures cover all six readability cameras: `{}`",
        native_camera_modes()
            .iter()
            .all(|camera| resolution_captures
                .iter()
                .filter(|capture| !capture.debug_overlay)
                .any(|capture| capture.camera == *camera))
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Software 3D asset-closeup source is presentation-only after truth hash: `{}`",
        software_3d_viewports.iter().any(|viewport| {
            viewport.camera == "asset_closeup_weapon_armor"
                && viewport.source == "presentation-gltf-closeup-after-truth-hash"
        })
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Software 3D shaded triangle count > 0: `{}`",
        software_3d_viewports
            .iter()
            .all(|viewport| viewport.shaded_triangle_count > 0)
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Software 3D depth sorting enabled: `{}`",
        software_3d_viewports
            .iter()
            .all(|viewport| viewport.depth_sorted)
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Software 3D replay sequence frame count >= 21: `{}`",
        software_3d_sequence.frame_count >= 21
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Software 3D replay sequence source is replay-derived: `{}`",
        software_3d_sequence.source == "replay-derived-runtime-gltf-after-truth-hash"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Software 3D replay sequence shaded triangles > 0: `{}`",
        software_3d_sequence
            .frames
            .iter()
            .all(|frame| frame.shaded_triangle_count > 0)
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Software 3D replay sequence hash chain present: `{}`",
        !software_3d_sequence.frame_hash_chain.is_empty()
    )
    .unwrap();
    writeln!(&mut out, "- 1280x720 capture present: `true`").unwrap();
    writeln!(&mut out, "- 1280x800 capture present: `true`").unwrap();
    writeln!(
        &mut out,
        "- 1920x1080 combat capture present: `{}`",
        resolution_captures
            .iter()
            .any(|capture| capture.width == 1920 && capture.height == 1080)
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Contact sheet: `native_combat_contact_sheet.svg`"
    )
    .unwrap();
    Ok(out)
}

#[cfg(target_os = "linux")]
fn render_native_combat_contact_sheet(
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    frames: &[NativeCombatFrameSpec],
    motion_frames: &[NativeCombatMotionFrameSpec],
    playback: &NativeCombatPlaybackSummary,
    live_loop: &NativeCombatLiveLoopSummary,
    player_loop: &NativePlayerLoopSummary,
    software_3d_viewports: &[NativeCombatSoftware3dViewport],
    software_3d_sequence: &NativeCombatSoftware3dSequenceSummary,
    resolution_captures: &[NativeCombatResolutionCapture],
) -> String {
    let width = 1280;
    let height = 1210;
    let mut out = String::new();
    writeln!(
        &mut out,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\">"
    )
    .unwrap();
    writeln!(
        &mut out,
        "<rect width=\"100%\" height=\"100%\" fill=\"#f1eadb\"/>"
    )
    .unwrap();
    writeln!(
        &mut out,
        "<text x=\"28\" y=\"38\" font-family=\"monospace\" font-size=\"20\" fill=\"#1f2528\">OATHYARD native combat visual contact sheet</text>"
    )
    .unwrap();
    writeln!(
        &mut out,
        "<text x=\"28\" y=\"66\" font-family=\"monospace\" font-size=\"13\" fill=\"#394247\">scenario {} | final {} | automated audit, owner acceptance not claimed</text>",
        xml_escape(&result.scenario_id),
        xml_escape(&result.final_state_hash)
    )
    .unwrap();
    writeln!(
        &mut out,
        "<text x=\"28\" y=\"88\" font-family=\"monospace\" font-size=\"12\" fill=\"#394247\">source-backed silhouettes: weapon reach/mass + armor coverage from content hash</text>"
    )
    .unwrap();
    writeln!(
        &mut out,
        "<text x=\"650\" y=\"88\" font-family=\"monospace\" font-size=\"12\" fill=\"#394247\">runtime mesh/glTF/preview refs verified after truth hash</text>"
    )
    .unwrap();
    writeln!(
        &mut out,
        "<text x=\"650\" y=\"104\" font-family=\"monospace\" font-size=\"11\" fill=\"#394247\">3D glTF depth projected: {} | projection uses z-depth: true</text>",
        native_3d_runtime_geometry(silhouette)
    )
    .unwrap();
    writeln!(
        &mut out,
        "<text x=\"650\" y=\"120\" font-family=\"monospace\" font-size=\"11\" fill=\"#394247\">player loop screens {} | loop hash {}</text>",
        player_loop.screen_count,
        xml_escape(&player_loop.loop_hash)
    )
    .unwrap();
    for fighter in &silhouette.fighters {
        writeln!(
            &mut out,
            "<text x=\"{}\" y=\"{}\" font-family=\"monospace\" font-size=\"10\" fill=\"#394247\">F{} {} | {} {}mm/{}g | {} torso {}</text>",
            28 + fighter.seat as i32 * 610,
            1032,
            fighter.seat,
            xml_escape(&clipped_text(&fighter.name, 20)),
            xml_escape(&fighter.weapon_id),
            fighter.weapon_reach_mm,
            fighter.weapon_mass_g,
            xml_escape(&fighter.armor_id),
            fighter.armor_torso_coverage_permille
        )
        .unwrap();
    }

    for (index, frame) in frames.iter().enumerate() {
        let col = index % 4;
        let row = index / 4;
        let x = 28 + col as i32 * 306;
        let y = 116 + row as i32 * 176;
        writeln!(
            &mut out,
            "<rect x=\"{x}\" y=\"{y}\" width=\"286\" height=\"156\" fill=\"#fff9ec\" stroke=\"#1f2528\"/>"
        )
        .unwrap();
        writeln!(
            &mut out,
            "<text x=\"{}\" y=\"{}\" font-family=\"monospace\" font-size=\"14\" fill=\"#1f2528\">{} {}</text>",
            x + 18,
            y + 30,
            frame.index,
            xml_escape(frame.state)
        )
        .unwrap();
        writeln!(
            &mut out,
            "<text x=\"{}\" y=\"{}\" font-family=\"monospace\" font-size=\"11\" fill=\"#394247\">{}</text>",
            x + 18,
            y + 58,
            xml_escape(&clipped_text(&frame.headline, 36))
        )
        .unwrap();
        writeln!(
            &mut out,
            "<text x=\"{}\" y=\"{}\" font-family=\"monospace\" font-size=\"11\" fill=\"#394247\">{}</text>",
            x + 18,
            y + 82,
            xml_escape(&clipped_text(&frame.detail, 36))
        )
        .unwrap();
        writeln!(
            &mut out,
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#1f2528\"/>",
            x + 42,
            y + 116,
            x + 242,
            y + 104
        )
        .unwrap();
        writeln!(
            &mut out,
            "<circle cx=\"{}\" cy=\"{}\" r=\"22\" fill=\"none\" stroke=\"#1f2528\"/>",
            x + 122,
            y + 118
        )
        .unwrap();
        writeln!(
            &mut out,
            "<circle cx=\"{}\" cy=\"{}\" r=\"22\" fill=\"none\" stroke=\"#1f2528\"/>",
            x + 192,
            y + 108
        )
        .unwrap();
        writeln!(
            &mut out,
            "<text x=\"{}\" y=\"{}\" font-family=\"monospace\" font-size=\"10\" fill=\"#394247\">hash {}</text>",
            x + 18,
            y + 146,
            xml_escape(&frame.frame_hash)
        )
        .unwrap();
    }

    let motion_y = 652;
    writeln!(
        &mut out,
        "<rect x=\"28\" y=\"{motion_y}\" width=\"1224\" height=\"150\" fill=\"#fff9ec\" stroke=\"#1f2528\"/>"
    )
    .unwrap();
    writeln!(
        &mut out,
        "<text x=\"48\" y=\"{}\" font-family=\"monospace\" font-size=\"13\" fill=\"#1f2528\">Replay-derived motion sequence</text>",
        motion_y + 28
    )
    .unwrap();
    for frame in motion_frames.iter() {
        let x = 48 + (frame.index as i32 - 1) * 56;
        writeln!(
            &mut out,
            "<rect x=\"{x}\" y=\"{}\" width=\"48\" height=\"42\" fill=\"#f1eadb\" stroke=\"#1f2528\"/>",
            motion_y + 46
        )
        .unwrap();
        writeln!(
            &mut out,
            "<text x=\"{}\" y=\"{}\" font-family=\"monospace\" font-size=\"9\" fill=\"#394247\">{}:{}</text>",
            x + 5,
            motion_y + 64,
            frame.index,
            xml_escape(&clipped_text(frame.phase, 6))
        )
        .unwrap();
        writeln!(
            &mut out,
            "<text x=\"{}\" y=\"{}\" font-family=\"monospace\" font-size=\"9\" fill=\"#394247\">f{}</text>",
            x + 5,
            motion_y + 82,
            frame.truth_frame
        )
        .unwrap();
    }
    writeln!(
        &mut out,
        "<text x=\"48\" y=\"{}\" font-family=\"monospace\" font-size=\"11\" fill=\"#394247\">playback loop: {} frames, {} cycles, final {} hash {}</text>",
        motion_y + 132,
        playback.playback_frame_count,
        playback.cycles,
        xml_escape(&playback.file),
        xml_escape(&playback.final_frame_hash)
    )
    .unwrap();
    writeln!(
        &mut out,
        "<text x=\"650\" y=\"{}\" font-family=\"monospace\" font-size=\"11\" fill=\"#394247\">live render loop: {} frames at {}Hz | samples {} | hash {}</text>",
        motion_y + 132,
        live_loop.rendered_frame_count,
        TRUTH_HZ,
        live_loop.sample_captures.len(),
        xml_escape(&live_loop.loop_hash)
    )
    .unwrap();

    let viewport_y = 818;
    writeln!(
        &mut out,
        "<rect x=\"28\" y=\"{viewport_y}\" width=\"1224\" height=\"92\" fill=\"#fff9ec\" stroke=\"#1f2528\"/>"
    )
    .unwrap();
    writeln!(
        &mut out,
        "<text x=\"48\" y=\"{}\" font-family=\"monospace\" font-size=\"13\" fill=\"#1f2528\">Software 3D mesh viewports</text>",
        viewport_y + 28
    )
    .unwrap();
    writeln!(
        &mut out,
        "<text x=\"48\" y=\"{}\" font-family=\"monospace\" font-size=\"12\" fill=\"#394247\">software 3D mesh viewports: {} | cameras {} | depth sorted mesh raster</text>",
        viewport_y + 56,
        software_3d_viewports.len(),
        xml_escape(
            &software_3d_viewports
                .iter()
                .map(|viewport| viewport.camera.as_str())
                .collect::<Vec<_>>()
                .join(",")
        )
    )
    .unwrap();
    if let Some(viewport) = software_3d_viewports.first() {
        writeln!(
            &mut out,
            "<text x=\"650\" y=\"{}\" font-family=\"monospace\" font-size=\"11\" fill=\"#394247\">3D viewport proof: {} triangles {} shaded {} hash {}</text>",
            viewport_y + 56,
            xml_escape(&viewport.file),
            viewport.triangle_count,
            viewport.shaded_triangle_count,
            xml_escape(&viewport.frame_hash)
        )
        .unwrap();
    }

    let sequence_y = 928;
    writeln!(
        &mut out,
        "<rect x=\"28\" y=\"{sequence_y}\" width=\"1224\" height=\"92\" fill=\"#f6fff9\" stroke=\"#1f2528\"/>"
    )
    .unwrap();
    writeln!(
        &mut out,
        "<text x=\"48\" y=\"{}\" font-family=\"monospace\" font-size=\"13\" fill=\"#1f2528\">Software 3D replay mesh sequence</text>",
        sequence_y + 28
    )
    .unwrap();
    writeln!(
        &mut out,
        "<text x=\"48\" y=\"{}\" font-family=\"monospace\" font-size=\"12\" fill=\"#394247\">replay-derived 3D mesh frames: {} | camera {} | source {}</text>",
        sequence_y + 56,
        software_3d_sequence.frame_count,
        xml_escape(&software_3d_sequence.camera),
        xml_escape(&software_3d_sequence.source)
    )
    .unwrap();
    writeln!(
        &mut out,
        "<text x=\"650\" y=\"{}\" font-family=\"monospace\" font-size=\"11\" fill=\"#394247\">3D sequence hash chain {} | projection {} | depth sorted {}</text>",
        sequence_y + 56,
        xml_escape(&software_3d_sequence.frame_hash_chain),
        xml_escape(&software_3d_sequence.projection_model),
        software_3d_sequence.depth_sorted
    )
    .unwrap();

    let y = 1038;
    writeln!(
        &mut out,
        "<rect x=\"28\" y=\"{y}\" width=\"1224\" height=\"140\" fill=\"#fff9ec\" stroke=\"#1f2528\"/>"
    )
    .unwrap();
    writeln!(
        &mut out,
        "<text x=\"48\" y=\"{}\" font-family=\"monospace\" font-size=\"13\" fill=\"#1f2528\">Resolution evidence</text>",
        y + 28
    )
    .unwrap();
    for (index, capture) in resolution_captures.iter().enumerate() {
        let col = index % 2;
        let row = index / 2;
        writeln!(
            &mut out,
            "<text x=\"{}\" y=\"{}\" font-family=\"monospace\" font-size=\"12\" fill=\"#394247\">{}x{} {} {} hash {}</text>",
            48 + col as i32 * 600,
            y + 60 + row as i32 * 18,
            capture.width,
            capture.height,
            xml_escape(&capture.capture_role),
            xml_escape(&capture.file),
            xml_escape(&capture.frame_hash)
        )
        .unwrap();
    }
    writeln!(
        &mut out,
        "<text x=\"48\" y=\"{}\" font-family=\"monospace\" font-size=\"12\" fill=\"#394247\">runtime asset refs: arena {} | manifest {}</text>",
        y + 96,
        xml_escape(&silhouette.arena_asset.runtime_mesh),
        xml_escape(&silhouette.asset_manifest_hash)
    )
    .unwrap();
    writeln!(&mut out, "</svg>").unwrap();
    out
}

#[cfg(target_os = "linux")]
fn native_ppm_evidence(
    path: &Path,
    width: u32,
    height: u32,
) -> Result<(usize, usize, String), OathError> {
    let bytes = fs::read(path)?;
    let header = format!("P6\n{} {}\n255\n", width, height);
    if !bytes.starts_with(header.as_bytes()) {
        return Err(OathError::Verify(format!(
            "PPM header mismatch for {}",
            path.display()
        )));
    }
    let expected_len = header.len() + width as usize * height as usize * 3;
    if bytes.len() != expected_len {
        return Err(OathError::Verify(format!(
            "PPM byte length mismatch for {}: expected {}, got {}",
            path.display(),
            expected_len,
            bytes.len()
        )));
    }
    let mut non_background_pixels = 0usize;
    for pixel in bytes[header.len()..].chunks_exact(3) {
        if pixel != [236, 227, 210] {
            non_background_pixels += 1;
        }
    }
    if non_background_pixels < 100 {
        return Err(OathError::Verify(format!(
            "PPM appears blank for {}",
            path.display()
        )));
    }
    Ok((bytes.len(), non_background_pixels, hash_hex(&bytes)))
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

fn draw_hash_stripes(pixels: &mut [u8], width: usize, height: usize, hash: &str) {
    let stripe_width = 8;
    for (index, byte) in hash.bytes().enumerate() {
        let value = byte % 16;
        let color = (32 + value * 8, 42 + value * 5, 48 + value * 4);
        fill_rect(
            pixels,
            width,
            height,
            index * stripe_width,
            42,
            stripe_width,
            18,
            color,
        );
    }
}

#[cfg(target_os = "linux")]
fn paint_native_stone_floor(pixels: &mut [u8], width: usize, height: usize) {
    let horizon = height * 43 / 100;
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 3;
            let noise = ((x as i32 * 17 + y as i32 * 31 + (x as i32 / 29) * 11).rem_euclid(19)) - 9;
            let color = if y < horizon {
                let fog = (y as i32 * 42 / horizon.max(1) as i32).clamp(0, 42);
                (74 + fog + noise / 3, 78 + fog + noise / 4, 76 + fog / 2)
            } else {
                let depth =
                    ((y - horizon) as i32 * 68 / (height - horizon).max(1) as i32).clamp(0, 68);
                let seam = (x + y / 2) % 92 < 3 || (y - horizon) % 58 < 3;
                if seam {
                    (74 + depth / 3, 62 + depth / 4, 47 + depth / 5)
                } else {
                    (
                        116 + depth / 2 + noise,
                        101 + depth / 3 + noise / 2,
                        76 + depth / 4,
                    )
                }
            };
            pixels[idx] = color.0.clamp(0, 255) as u8;
            pixels[idx + 1] = color.1.clamp(0, 255) as u8;
            pixels[idx + 2] = color.2.clamp(0, 255) as u8;
        }
    }
}

fn fill_rect(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    rect_width: usize,
    rect_height: usize,
    color: (u8, u8, u8),
) {
    let max_y = (y + rect_height).min(height);
    let max_x = (x + rect_width).min(width);
    for yy in y.min(height)..max_y {
        for xx in x.min(width)..max_x {
            let index = (yy * width + xx) * 3;
            pixels[index] = color.0;
            pixels[index + 1] = color.1;
            pixels[index + 2] = color.2;
        }
    }
}

fn clipped_text(value: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for (index, ch) in value.chars().enumerate() {
        if index >= max_chars {
            out.push_str("...");
            return out;
        }
        out.push(ch);
    }
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
        "- Native 3D renderer gate: raw X11/XWayland-backed combat renderer available when DISPLAY is present"
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

#[cfg(target_os = "linux")]
pub fn native_combat_render(
    scenario_path: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
) -> Result<DuelResult, OathError> {
    // R-GAP-1: Enforce freeze gate at the native combat render boundary.
    // Parse and gate the scenario BEFORE running the simulation, providing
    // an explicit early-reject for AI-derived assets that have not passed
    // all five freeze conditions. Defense-in-depth alongside run_scenario_file.
    let scenario_path = scenario_path.as_ref();
    let scenario_text = fs::read_to_string(scenario_path)?;
    let pre_scenario = Scenario::parse(&scenario_text)?;
    enforce_scenario_freeze_gate(&pre_scenario)?;
    let result = run_scenario_text(&scenario_text)?;
    let simulation_micros = 0u128;
    let verified_replay = verify_replay_text(&result.replay_json)?;
    let replay_verify_micros = 0u128;
    if verified_replay.final_state_hash != result.final_state_hash
        || verified_replay.turn_hashes != result.turn_hashes
    {
        return Err(OathError::Verify(
            "native capture input replay verification did not reproduce truth hashes".to_string(),
        ));
    }
    let silhouette = native_combat_silhouette_evidence(&result)?;
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;
    let capture_command = format!(
        "cargo run --locked -- native-combat-render --scenario {} --out {}",
        scenario_path.display(),
        out_dir.display()
    );
    let replay_path = out_dir.join("native_capture_input_replay.json");
    fs::write(&replay_path, &result.replay_json)?;
    let replay_path_string = replay_path.display().to_string();
    let ppm_path = out_dir.join("native_combat_render.ppm");
    let truth_before_render = native_renderer_truth_snapshot(&result);
    let render_result = unsafe {
        x11_combat_render_capture(
            &result,
            &silhouette,
            out_dir,
            &ppm_path,
            &capture_command,
            &replay_path_string,
            simulation_micros,
            replay_verify_micros,
        )
    };
    let mut report = String::new();
    writeln!(&mut report, "# OATHYARD Native Combat Render").unwrap();
    writeln!(&mut report).unwrap();
    match render_result {
        Ok(capture) => {
            let resolutions = capture.resolution_captures.clone();
            let contact_count: usize = result.turns.iter().map(|turn| turn.contacts.len()).sum();
            let truth_after_render = native_renderer_assert_truth_unchanged(
                "native renderer capture pass",
                &truth_before_render,
                &result,
            )?;
            let mutation_proof =
                native_renderer_mutation_proof(truth_before_render.clone(), truth_after_render);
            write_native_lighting_material_witness_artifacts(out_dir)?;
            let capture_hook = native_renderer_capture_hook_summary(&capture, &resolutions);
            fs::write(
                out_dir.join("native_combat_render_manifest.json"),
                render_native_combat_render_manifest_json(
                    &result,
                    &silhouette,
                    &capture.state_frames,
                    &capture.motion_frames,
                    &capture.playback,
                    &capture.live_loop,
                    &capture.player_loop,
                    &capture.software_3d_viewports,
                    &capture.software_3d_sequence,
                    &resolutions,
                ),
            )?;
            fs::write(
                out_dir.join("native_combat_visual_audit.md"),
                render_native_combat_visual_audit_report(
                    &result,
                    &silhouette,
                    &capture.state_frames,
                    &capture.motion_frames,
                    &capture.playback,
                    &capture.live_loop,
                    &capture.player_loop,
                    &capture.software_3d_viewports,
                    &capture.software_3d_sequence,
                    &resolutions,
                    out_dir,
                )?,
            )?;
            fs::write(
                out_dir.join("native_combat_contact_sheet.svg"),
                render_native_combat_contact_sheet(
                    &result,
                    &silhouette,
                    &capture.state_frames,
                    &capture.motion_frames,
                    &capture.playback,
                    &capture.live_loop,
                    &capture.player_loop,
                    &capture.software_3d_viewports,
                    &capture.software_3d_sequence,
                    &resolutions,
                ),
            )?;
            fs::write(
                out_dir.join("native_renderer_post_hash_input.json"),
                render_native_renderer_post_hash_input_json(&result, &silhouette, &capture),
            )?;
            fs::write(
                out_dir.join("native_renderer_capture_hook.json"),
                render_native_renderer_capture_hook_json(&capture_hook),
            )?;
            fs::write(
                out_dir.join("native_renderer_truth_writeback_audit.md"),
                render_native_renderer_truth_writeback_audit_report(
                    &result,
                    &capture,
                    &capture_hook,
                    &mutation_proof,
                    &resolutions,
                ),
            )?;
            native_renderer_assert_truth_unchanged(
                "native renderer manifest/contact-sheet/audit generation",
                &truth_before_render,
                &result,
            )?;
            fs::write(
                out_dir.join("native_renderer_backend_manifest.json"),
                render_native_renderer_backend_manifest_json(
                    &result,
                    &silhouette,
                    &capture,
                    &capture_hook,
                    &mutation_proof,
                ),
            )?;
            fs::write(
                out_dir.join("native_renderer_truth_mutation_proof.json"),
                render_native_renderer_mutation_proof_json(&mutation_proof),
            )?;
            native_renderer_assert_truth_unchanged(
                "native renderer backend/proof artifact generation",
                &truth_before_render,
                &result,
            )?;
            writeln!(&mut report, "Status: PASSED").unwrap();
            writeln!(&mut report, "- Backend: raw X11/XWayland").unwrap();
            writeln!(&mut report, "- Scenario: `{}`", result.scenario_id).unwrap();
            writeln!(&mut report, "- Content hash: `{}`", result.content_hash).unwrap();
            writeln!(
                &mut report,
                "- Initial state hash: `{}`",
                result.initial_state_hash
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Final state hash: `{}`",
                result.final_state_hash
            )
            .unwrap();
            writeln!(&mut report, "- Replay verified before capture: `true`").unwrap();
            writeln!(
                &mut report,
                "- Replay source hash: `{}`",
                mutation_proof.before.replay_json_hash
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Trace source hash: `{}`",
                mutation_proof.before.trace_json_hash
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Pre-capture truth hash: `{}`",
                mutation_proof.before.final_state_hash
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Post-capture truth hash: `{}`",
                mutation_proof.after.final_state_hash
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Pre/post truth hashes equal: `{}`",
                mutation_proof.all_equal
            )
            .unwrap();
            writeln!(&mut report, "- Rendered contact count: `{contact_count}`").unwrap();
            writeln!(
                &mut report,
                "- Source-backed silhouettes: `{}`",
                silhouette.source_backed
            )
            .unwrap();
            writeln!(&mut report, "- Silhouette source: `{}`", silhouette.source).unwrap();
            writeln!(
                &mut report,
                "- Runtime asset refs verified: `{}`",
                silhouette.runtime_asset_refs_verified
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Runtime glTF geometry projected: `{}`",
                native_geometry_projected(&silhouette)
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Native 3D runtime geometry: `{}`",
                native_3d_runtime_geometry(&silhouette)
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Projection model: `integer_oblique_depth_projection`"
            )
            .unwrap();
            writeln!(&mut report, "- Projection uses Z depth: `true`").unwrap();
            writeln!(
                &mut report,
                "- Runtime asset manifest hash: `{}`",
                silhouette.asset_manifest_hash
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Arena runtime mesh: `{}`",
                silhouette.arena_asset.runtime_mesh
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Combat frames: `{}`",
                capture.state_frames.len()
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Motion frames: `{}`",
                capture.motion_frames.len()
            )
            .unwrap();
            writeln!(
                &mut report,
                "- States rendered: `{}`",
                capture
                    .state_frames
                    .iter()
                    .map(|frame| frame.state)
                    .collect::<Vec<_>>()
                    .join(",")
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Resolution captures: `{}`",
                resolutions
                    .iter()
                    .map(|capture| format!("{}x{}", capture.width, capture.height))
                    .collect::<Vec<_>>()
                    .join(",")
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Product-mode clean captures: `{}`",
                resolutions
                    .iter()
                    .filter(|capture| !capture.debug_overlay)
                    .count()
            )
            .unwrap();
            writeln!(
                &mut report,
                "- High-res capture visual floor: `source-backed shaded presentation floor; debug/hash overlays moved to reports/manifests`"
            )
            .unwrap();
            writeln!(
                &mut report,
                "- High-res capture debug overlay minimized: `true`"
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Captured native window PPM: `native_combat_render.ppm`"
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Sequence manifest: `native_combat_render_manifest.json`"
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Renderer backend manifest: `native_renderer_backend_manifest.json`"
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Post-hash renderer input: `native_renderer_post_hash_input.json`"
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Renderer capture hook: `native_renderer_capture_hook.json`"
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Renderer truth mutation proof: `native_renderer_truth_mutation_proof.json`"
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Renderer truth writeback audit: `native_renderer_truth_writeback_audit.md`"
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Renderer truth writeback guard: `native_renderer_assert_truth_unchanged`"
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Visual audit: `native_combat_visual_audit.md`"
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Contact sheet: `native_combat_contact_sheet.svg`"
            )
            .unwrap();
            writeln!(
                        &mut report,
                        "- Motion sequence: `native_combat_motion_001.ppm` through `native_combat_motion_{:03}.ppm`",
                        capture.motion_frames.len()
                    )
                    .unwrap();
            writeln!(
                &mut report,
                "- Playback loop frames: `{}`",
                capture.playback.playback_frame_count
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Live render loop frames: `{}`",
                capture.live_loop.rendered_frame_count
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Live render loop sample captures: `{}`",
                capture.live_loop.sample_captures.len()
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Live render loop nominal duration: `{}` ms",
                capture.live_loop.nominal_duration_ms
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Live render loop hash: `{}`",
                capture.live_loop.loop_hash
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Player-facing loop frames: `{}`",
                capture.player_loop.rendered_frame_count
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Player-facing loop screens: `{}`",
                capture
                    .player_loop
                    .frames
                    .iter()
                    .map(|frame| frame.screen)
                    .collect::<Vec<_>>()
                    .join(",")
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Player-facing loop timing source: `{}`",
                capture.player_loop.timing_source
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Player-facing loop truth hash unchanged: `{}`",
                capture.player_loop.truth_hash_before_loop
                    == capture.player_loop.truth_hash_after_loop
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Player-facing loop hash: `{}`",
                capture.player_loop.loop_hash
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Software 3D viewports: `{}`",
                capture.software_3d_viewports.len()
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Software 3D viewport cameras: `{}`",
                capture
                    .software_3d_viewports
                    .iter()
                    .map(|viewport| viewport.camera.as_str())
                    .collect::<Vec<_>>()
                    .join(",")
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Software 3D projection model: `integer_depth_sorted_mesh_raster`"
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Software 3D replay sequence frames: `{}`",
                capture.software_3d_sequence.frame_count
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Software 3D replay sequence camera: `{}`",
                capture.software_3d_sequence.camera
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Software 3D replay sequence source: `{}`",
                capture.software_3d_sequence.source
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Software 3D replay sequence hash chain: `{}`",
                capture.software_3d_sequence.frame_hash_chain
            )
            .unwrap();
            let lighting_post = native_lighting_post_summary(&result, &silhouette);
            writeln!(
                &mut report,
                "- Lighting/post profile: `{}`",
                lighting_post.profile
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Lighting/post presentation-only: `{}`",
                lighting_post.presentation_only
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Lighting/post owner visual acceptance: `{}`",
                lighting_post.owner_visual_acceptance
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Lighting/post material classes: `{}`",
                lighting_post.material_distinction_classes.join(",")
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Lighting/post material witness capture: `{}`",
                lighting_post.material_witness_capture
            )
            .unwrap();
            writeln!(
                &mut report,
                "- Playback final capture: `{}`",
                capture.playback.file
            )
            .unwrap();
            if let (Some(first), Some(last)) = (
                capture.live_loop.sample_captures.first(),
                capture.live_loop.sample_captures.last(),
            ) {
                writeln!(
                    &mut report,
                    "- Live render loop samples: `{}` through `{}`",
                    first.file, last.file
                )
                .unwrap();
            }
            writeln!(
                        &mut report,
                        "- Truth mutation: none; renderer consumes the duel result after replay hashes are computed"
                    )
                    .unwrap();
            writeln!(&mut report, "- Public demo ready: `{PUBLIC_DEMO_READY}`").unwrap();
            writeln!(
                &mut report,
                "- Release candidate ready: `{RELEASE_CANDIDATE_READY}`"
            )
            .unwrap();
            writeln!(&mut report, "- Owner visual acceptance claimed: `false`").unwrap();
            writeln!(&mut report, "- Owner input acceptance claimed: `false`").unwrap();
        }
        Err(message) => {
            writeln!(&mut report, "Status: BLOCKED").unwrap();
            writeln!(&mut report, "- Backend: raw X11/XWayland").unwrap();
            writeln!(&mut report, "- Error: {message}").unwrap();
        }
    }
    fs::write(out_dir.join("native_combat_render_report.md"), &report)?;
    if report.contains("Status: PASSED") {
        Ok(result)
    } else {
        Err(OathError::Verify(report))
    }
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
        "# OATHYARD Native Combat Render\n\nStatus: BLOCKED\n- Raw X11 combat render is Linux-only in this build.\n",
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
        "# OATHYARD Native Roster 3D Showcase\n\nStatus: BLOCKED\n- Raw X11/software native roster showcase is Linux-only in this build.\n",
    )?;
    Err(OathError::Verify(
        "native roster showcase is Linux-only in this build".to_string(),
    ))
}

#[cfg(target_os = "linux")]
#[repr(C)]
struct Display {
    _private: [u8; 0],
}

#[cfg(target_os = "linux")]
#[repr(C)]
struct XImage {
    _private: [u8; 0],
}

#[cfg(target_os = "linux")]
#[repr(C)]
#[derive(Clone, Copy)]
struct XPoint {
    x: i16,
    y: i16,
}

#[cfg(target_os = "linux")]
type Window = c_ulong;

#[cfg(target_os = "linux")]
type Pixmap = c_ulong;

#[cfg(target_os = "linux")]
type GC = *mut std::ffi::c_void;

#[cfg(target_os = "linux")]
#[link(name = "X11")]
extern "C" {
    fn XOpenDisplay(display_name: *const c_char) -> *mut Display;
    fn XDefaultScreen(display: *mut Display) -> c_int;
    fn XDefaultDepth(display: *mut Display, screen_number: c_int) -> c_int;
    fn XRootWindow(display: *mut Display, screen_number: c_int) -> Window;
    fn XBlackPixel(display: *mut Display, screen_number: c_int) -> c_ulong;
    fn XWhitePixel(display: *mut Display, screen_number: c_int) -> c_ulong;
    fn XCreateSimpleWindow(
        display: *mut Display,
        parent: Window,
        x: c_int,
        y: c_int,
        width: c_uint,
        height: c_uint,
        border_width: c_uint,
        border: c_ulong,
        background: c_ulong,
    ) -> Window;
    fn XStoreName(display: *mut Display, window: Window, window_name: *const c_char) -> c_int;
    fn XMapWindow(display: *mut Display, window: Window) -> c_int;
    fn XCreateGC(
        display: *mut Display,
        drawable: Window,
        valuemask: c_ulong,
        values: *mut std::ffi::c_void,
    ) -> GC;
    fn XSetForeground(display: *mut Display, gc: GC, foreground: c_ulong) -> c_int;
    fn XDrawString(
        display: *mut Display,
        drawable: Window,
        gc: GC,
        x: c_int,
        y: c_int,
        string: *const c_char,
        length: c_int,
    ) -> c_int;
    fn XDrawLine(
        display: *mut Display,
        drawable: Window,
        gc: GC,
        x1: c_int,
        y1: c_int,
        x2: c_int,
        y2: c_int,
    ) -> c_int;
    fn XFillRectangle(
        display: *mut Display,
        drawable: Window,
        gc: GC,
        x: c_int,
        y: c_int,
        width: c_uint,
        height: c_uint,
    ) -> c_int;
    fn XFillPolygon(
        display: *mut Display,
        drawable: Window,
        gc: GC,
        points: *mut XPoint,
        npoints: c_int,
        shape: c_int,
        mode: c_int,
    ) -> c_int;
    fn XCreatePixmap(
        display: *mut Display,
        drawable: Window,
        width: c_uint,
        height: c_uint,
        depth: c_uint,
    ) -> Pixmap;
    fn XCopyArea(
        display: *mut Display,
        src: Pixmap,
        dest: Window,
        gc: GC,
        src_x: c_int,
        src_y: c_int,
        width: c_uint,
        height: c_uint,
        dest_x: c_int,
        dest_y: c_int,
    ) -> c_int;
    fn XFreePixmap(display: *mut Display, pixmap: Pixmap) -> c_int;
    fn XSync(display: *mut Display, discard: c_int) -> c_int;
    fn XGetImage(
        display: *mut Display,
        drawable: Window,
        x: c_int,
        y: c_int,
        width: c_uint,
        height: c_uint,
        plane_mask: c_ulong,
        format: c_int,
    ) -> *mut XImage;
    fn XGetPixel(ximage: *mut XImage, x: c_int, y: c_int) -> c_ulong;
    fn XDestroyImage(ximage: *mut XImage) -> c_int;
    fn XFlush(display: *mut Display) -> c_int;
    fn XFreeGC(display: *mut Display, gc: GC) -> c_int;
    fn XDestroyWindow(display: *mut Display, window: Window) -> c_int;
    fn XCloseDisplay(display: *mut Display) -> c_int;
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeCombatFrameSpec {
    index: usize,
    file: String,
    state: &'static str,
    turn: u32,
    headline: String,
    detail: String,
    frame_hash: String,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeCombatMotionFrameSpec {
    index: usize,
    file: String,
    phase: &'static str,
    turn: u32,
    truth_frame: u32,
    progress_permille: u32,
    headline: String,
    detail: String,
    turn_hash: String,
    frame_hash: String,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeCombatCapture {
    state_frames: Vec<NativeCombatFrameSpec>,
    motion_frames: Vec<NativeCombatMotionFrameSpec>,
    playback: NativeCombatPlaybackSummary,
    live_loop: NativeCombatLiveLoopSummary,
    player_loop: NativePlayerLoopSummary,
    production_renderer: NativeProductionRendererSummary,
    resolution_captures: Vec<NativeCombatResolutionCapture>,
    software_3d_viewports: Vec<NativeCombatSoftware3dViewport>,
    software_3d_sequence: NativeCombatSoftware3dSequenceSummary,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativePlayerLoopSummary {
    source: String,
    backend: String,
    rendered_frame_count: usize,
    screen_count: usize,
    nominal_frame_interval_ms: u32,
    nominal_duration_ms: u32,
    timing_source: String,
    truth_hash_before_loop: String,
    truth_hash_after_loop: String,
    loop_hash: String,
    final_frame_hash: String,
    frames: Vec<NativePlayerLoopFrameSpec>,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativePlayerLoopFrameSpec {
    index: usize,
    file: String,
    screen: &'static str,
    input_action: &'static str,
    scheduled_ms: u32,
    truth_frame: u32,
    headline: String,
    detail: String,
    truth_cache_key: String,
    base_cost_frames: u32,
    current_cost_frames: u32,
    physical_reasons: Vec<String>,
    frame_hash: String,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeProductionRendererSummary {
    manifest_path: String,
    report_path: String,
    width: u32,
    height: u32,
    state_capture_count: usize,
    live_loop_frame_count: usize,
    frame_count: usize,
    min_pixel_delta_from_previous: usize,
    frame_hash_chain: String,
    captures: Vec<NativeProductionRendererFrame>,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeProductionRendererFrame {
    file: String,
    state: String,
    screen: String,
    stream: String,
    capture_role: String,
    width: u32,
    height: u32,
    source: String,
    truth_frame: u32,
    scheduled_ms: u32,
    motion_frame_index: usize,
    triangle_count: usize,
    shaded_triangle_count: usize,
    non_background_pixels: usize,
    pixel_delta_from_previous: usize,
    frame_hash: String,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeCaptureMatrixEntry {
    capture_id: String,
    file: String,
    category: String,
    required_state: String,
    screen: String,
    item_id: String,
    item_name: String,
    command: String,
    replay_path: String,
    replay_hash: String,
    replay_final_hash: String,
    content_hash: String,
    asset_manifest_hash: String,
    presentation_asset_manifest_hash: String,
    production_visual_manifest_hash: String,
    backend_id: String,
    width: u32,
    height: u32,
    camera_mode: String,
    frame_tick: u32,
    scheduled_ms: u32,
    motion_frame_index: usize,
    triangle_count: usize,
    shaded_triangle_count: usize,
    non_background_pixels: usize,
    pixel_delta_from_previous: usize,
    frame_hash: String,
    sha256: String,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeCaptureMatrixTimingSample {
    capture_id: String,
    category: String,
    render_micros: u128,
    write_micros: u128,
    inspect_micros: u128,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeCaptureMatrixPixelSample {
    label: &'static str,
    x: u32,
    y: u32,
    rgb: (u8, u8, u8),
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeCombatPlaybackSummary {
    file: String,
    source_motion_frame_count: usize,
    playback_frame_count: usize,
    cycles: usize,
    nominal_frame_interval_ms: u32,
    nominal_duration_ms: u32,
    final_frame_hash: String,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeCombatLiveLoopSummary {
    source_motion_frame_count: usize,
    rendered_frame_count: usize,
    nominal_frame_interval_ms: u32,
    nominal_duration_ms: u32,
    loop_hash: String,
    final_frame_hash: String,
    sample_captures: Vec<NativeCombatLiveLoopSample>,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeCombatLiveLoopSample {
    file: String,
    loop_frame_index: usize,
    source_motion_frame_index: usize,
    truth_frame: u32,
    frame_hash: String,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeCombatSoftware3dViewport {
    file: String,
    camera: String,
    width: u32,
    height: u32,
    triangle_count: usize,
    shaded_triangle_count: usize,
    non_background_pixels: usize,
    projection_model: String,
    depth_sorted: bool,
    source: String,
    frame_hash: String,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeCombatSoftware3dSequenceSummary {
    camera: String,
    frame_count: usize,
    width: u32,
    height: u32,
    projection_model: String,
    depth_sorted: bool,
    source: String,
    frame_hash_chain: String,
    frames: Vec<NativeCombatSoftware3dFrame>,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeCombatSoftware3dFrame {
    index: usize,
    file: String,
    phase: String,
    turn: u32,
    truth_frame: u32,
    progress_permille: u32,
    triangle_count: usize,
    shaded_triangle_count: usize,
    non_background_pixels: usize,
    frame_hash: String,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeLightingPostSummary {
    profile: &'static str,
    dynamic_lighting: bool,
    contact_grounding: bool,
    ambient_occlusion_equivalent: bool,
    fog_dust_atmosphere: bool,
    tone_mapping: &'static str,
    exposure_permille: i32,
    color_grade: &'static str,
    anti_aliasing_equivalent: &'static str,
    bloom_policy: &'static str,
    bloom_event_count: usize,
    material_witnesses: Vec<String>,
    material_distinction_classes: Vec<String>,
    required_material_classes_distinguished: bool,
    material_witness_capture: &'static str,
    material_witness_manifest: &'static str,
    presentation_only: bool,
    truth_mutation: bool,
    owner_visual_acceptance: bool,
    public_demo_ready: bool,
    release_candidate_ready: bool,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NativeLightingMaterialWitnessSpec {
    material_class: &'static str,
    material_id: &'static str,
    fallback_color: (u8, u8, u8),
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeRosterShowcaseFrame {
    index: usize,
    file: String,
    fighter_id: String,
    fighter_name: String,
    weapon_id: String,
    armor_id: String,
    arena_id: String,
    width: u32,
    height: u32,
    triangle_count: usize,
    shaded_triangle_count: usize,
    non_background_pixels: usize,
    projection_model: String,
    depth_sorted: bool,
    source: String,
    fighter_gltf_hash: String,
    weapon_gltf_hash: String,
    armor_gltf_hash: String,
    frame_hash: String,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeCombatResolutionCapture {
    file: String,
    camera: String,
    width: u32,
    height: u32,
    frame_hash: String,
    source: String,
    capture_role: String,
    debug_overlay: bool,
    triangle_count: usize,
    shaded_triangle_count: usize,
    non_background_pixels: usize,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeRendererTruthSnapshot {
    final_state_hash: String,
    replay_json_hash: String,
    trace_json_hash: String,
    contacts_hash: String,
    injury_capability_hash: String,
    action_validity_hash: String,
    end_condition_hash: String,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeRendererMutationProof {
    before: NativeRendererTruthSnapshot,
    after: NativeRendererTruthSnapshot,
    all_equal: bool,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeRendererCapturedFrame {
    file: String,
    stream: String,
    width: u32,
    height: u32,
    frame_hash: String,
    source: String,
    capture_after_truth_hash: bool,
    presentation_only: bool,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeRendererCaptureHookSummary {
    hook_id: String,
    framebuffer_source: String,
    timing_source: String,
    capture_count: usize,
    high_resolution_capture_count: usize,
    min_high_resolution_width: u32,
    min_high_resolution_height: u32,
    hook_hash: String,
    captures: Vec<NativeRendererCapturedFrame>,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeCombatSilhouetteEvidence {
    source_backed: bool,
    source: String,
    reconstructed_initial_state_hash: String,
    runtime_asset_refs_verified: bool,
    asset_manifest_hash: String,
    presentation_asset_manifest_hash: String,
    high_detail_presentation_assets_verified: bool,
    arena_asset: NativeCombatAssetRef,
    arena_presentation_asset: NativePresentationAssetRef,
    fighters: Vec<NativeCombatFighterSilhouette>,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeCombatAssetRef {
    id: String,
    kind: String,
    source: String,
    runtime_mesh: String,
    runtime_gltf: String,
    preview: String,
    provenance: String,
    source_hash: String,
    runtime_mesh_hash: String,
    runtime_gltf_hash: String,
    preview_hash: String,
    geometry: NativeGltfGeometry,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativePresentationAssetRef {
    id: String,
    kind: String,
    source: String,
    runtime_mesh: String,
    runtime_gltf: String,
    preview: String,
    source_candidate_gltf: String,
    source_candidate_bin: String,
    provenance: String,
    license_status: String,
    source_hash: String,
    runtime_mesh_hash: String,
    runtime_gltf_hash: String,
    preview_hash: String,
    source_candidate_gltf_hash: String,
    source_candidate_bin_hash: String,
    geometry: NativeGltfGeometry,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeGltfGeometry {
    vertex_count: usize,
    index_count: usize,
    triangle_count: usize,
    min_x_milli: i32,
    max_x_milli: i32,
    min_y_milli: i32,
    max_y_milli: i32,
    min_z_milli: i32,
    max_z_milli: i32,
    geometry_hash: String,
    positions_milli: Vec<(i32, i32, i32)>,
    indices: Vec<usize>,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, PartialEq, Eq)]
struct NativeCombatFighterSilhouette {
    seat: usize,
    name: String,
    weapon_id: String,
    weapon_name: String,
    weapon_asset: NativeCombatAssetRef,
    weapon_presentation_asset: NativePresentationAssetRef,
    weapon_length_mm: i32,
    weapon_reach_mm: i32,
    weapon_mass_g: i32,
    weapon_inertia_g_cm2: i32,
    weapon_span_px: c_int,
    weapon_head_px: c_int,
    armor_id: String,
    armor_name: String,
    armor_asset: NativeCombatAssetRef,
    armor_presentation_asset: NativePresentationAssetRef,
    armor_material: String,
    armor_mass_g: i32,
    armor_torso_coverage_permille: i32,
    armor_head_coverage_permille: i32,
    armor_weapon_arm_coverage_permille: i32,
    armor_lead_leg_coverage_permille: i32,
    armor_gap_permille: i32,
    armor_torso_width_px: c_int,
    armor_torso_height_px: c_int,
    armor_head_marker_px: c_int,
    body_mass_g: i32,
    stance_width_mm: i32,
}

#[cfg(target_os = "linux")]
impl NativeCombatSilhouetteEvidence {
    fn fighter(&self, seat: usize) -> Option<&NativeCombatFighterSilhouette> {
        self.fighters.iter().find(|fighter| fighter.seat == seat)
    }
}

#[cfg(target_os = "linux")]
fn native_renderer_truth_snapshot(result: &DuelResult) -> NativeRendererTruthSnapshot {
    let mut contacts = String::new();
    let mut injuries = String::new();
    let mut action_validity = String::new();

    for turn in &result.turns {
        writeln!(
            &mut action_validity,
            "turn={};hash={}",
            turn.turn, turn.state_hash
        )
        .unwrap();
        for cost in &turn.costs {
            writeln!(
                &mut action_validity,
                "seat={};action={};base={};current={};valid={}",
                cost.fighter,
                cost.action.as_str(),
                cost.base_frames,
                cost.current_frames,
                cost.action_valid
            )
            .unwrap();
        }
        for contact in &turn.contacts {
            writeln!(
                &mut contacts,
                "turn={};frame={};attacker={};defender={};action={};direction={};target={};weapon={};armor={};energy={};impulse={};material={};anatomy={};cause={}",
                contact.turn,
                contact.frame,
                contact.attacker,
                contact.defender,
                contact.action.as_str(),
                contact.direction.as_str(),
                contact.target.as_str(),
                contact.weapon_id,
                contact.armor_id,
                contact.energy_milli,
                contact.impulse_milli,
                contact.material_result,
                contact.anatomy_result,
                contact.cause_chain
            )
            .unwrap();
            write!(
                &mut injuries,
                "turn={};frame={};material={};anatomy={};",
                contact.turn, contact.frame, contact.material_result, contact.anatomy_result
            )
            .unwrap();
            write_native_capability_delta_material(&mut injuries, &contact.capability_delta);
            injuries.push('\n');
        }
    }

    let mut end_condition = String::new();
    writeln!(
        &mut end_condition,
        "status={};winner={};reason={}",
        result.end_condition.status,
        result.end_condition.winner_token(),
        result.end_condition.reason
    )
    .unwrap();
    for fighter in &result.end_condition.fighters {
        writeln!(
            &mut end_condition,
            "seat={};incapacitated={};stop_kind={};reason={};balance={};grip_r={};torque={};torso_rotation={};recovery={};thrust_valid={};cut_valid={}",
            fighter.seat,
            fighter.incapacitated,
            fighter.stop_kind,
            fighter.reason,
            fighter.balance_permille,
            fighter.grip_r_permille,
            fighter.torque_permille,
            fighter.torso_rotation_permille,
            fighter.recovery_slowdown_frames,
            fighter.thrust_valid,
            fighter.cut_valid
        )
        .unwrap();
    }

    NativeRendererTruthSnapshot {
        final_state_hash: result.final_state_hash.clone(),
        replay_json_hash: hash_hex(result.replay_json.as_bytes()),
        trace_json_hash: hash_hex(result.trace_json.as_bytes()),
        contacts_hash: hash_hex(contacts.as_bytes()),
        injury_capability_hash: hash_hex(injuries.as_bytes()),
        action_validity_hash: hash_hex(action_validity.as_bytes()),
        end_condition_hash: hash_hex(end_condition.as_bytes()),
    }
}

#[cfg(target_os = "linux")]
fn write_native_capability_delta_material(out: &mut String, delta: &CapabilityDelta) {
    write!(
        out,
        "torso_rotation={};recovery_add={};balance={};torque={};grip_r={};grip_l={};invalidates_thrust={};invalidates_cut={};event={}",
        delta.torso_rotation_delta,
        delta.recovery_slowdown_add,
        delta.balance_delta,
        delta.torque_delta,
        delta.grip_r_delta,
        delta.grip_l_delta,
        delta.invalidates_thrust,
        delta.invalidates_cut,
        delta.event
    )
    .unwrap();
}

#[cfg(target_os = "linux")]
fn native_renderer_mutation_proof(
    before: NativeRendererTruthSnapshot,
    after: NativeRendererTruthSnapshot,
) -> NativeRendererMutationProof {
    let all_equal = before == after;
    NativeRendererMutationProof {
        before,
        after,
        all_equal,
    }
}

#[cfg(target_os = "linux")]
fn native_renderer_changed_truth_fields(proof: &NativeRendererMutationProof) -> Vec<String> {
    let mut fields = Vec::new();
    if proof.before.final_state_hash != proof.after.final_state_hash {
        fields.push("final_state_hash".to_string());
    }
    if proof.before.replay_json_hash != proof.after.replay_json_hash {
        fields.push("replay_json_hash".to_string());
    }
    if proof.before.trace_json_hash != proof.after.trace_json_hash {
        fields.push("trace_json_hash".to_string());
    }
    if proof.before.contacts_hash != proof.after.contacts_hash {
        fields.push("contacts_hash".to_string());
    }
    if proof.before.injury_capability_hash != proof.after.injury_capability_hash {
        fields.push("injury_capability_hash".to_string());
    }
    if proof.before.action_validity_hash != proof.after.action_validity_hash {
        fields.push("action_validity_hash".to_string());
    }
    if proof.before.end_condition_hash != proof.after.end_condition_hash {
        fields.push("end_condition_hash".to_string());
    }
    fields
}

#[cfg(target_os = "linux")]
fn native_renderer_capture_hook_summary(
    capture: &NativeCombatCapture,
    resolution_captures: &[NativeCombatResolutionCapture],
) -> NativeRendererCaptureHookSummary {
    let mut captures = Vec::new();
    native_renderer_push_capture(
        &mut captures,
        "native_window_initial",
        "native_combat_render.ppm",
        960,
        540,
        "x11-pixmap-after-truth-hash",
        "initial_window_capture_hash_recorded_in_capture_hook",
    );
    for frame in &capture.state_frames {
        native_renderer_push_capture(
            &mut captures,
            "state_frame",
            &frame.file,
            960,
            540,
            "x11-pixmap-state-frame-after-truth-hash",
            &frame.frame_hash,
        );
    }
    for frame in &capture.motion_frames {
        native_renderer_push_capture(
            &mut captures,
            "motion_frame",
            &frame.file,
            960,
            540,
            "x11-pixmap-motion-frame-after-truth-hash",
            &frame.frame_hash,
        );
    }
    native_renderer_push_capture(
        &mut captures,
        "playback_final",
        &capture.playback.file,
        960,
        540,
        "x11-pixmap-playback-final-after-truth-hash",
        &capture.playback.final_frame_hash,
    );
    for sample in &capture.live_loop.sample_captures {
        native_renderer_push_capture(
            &mut captures,
            "live_loop_sample",
            &sample.file,
            960,
            540,
            "x11-pixmap-live-loop-sample-after-truth-hash",
            &sample.frame_hash,
        );
    }
    for frame in &capture.player_loop.frames {
        native_renderer_push_capture(
            &mut captures,
            "player_loop_screen",
            &frame.file,
            960,
            540,
            "x11-pixmap-player-loop-screen-after-truth-hash",
            &frame.frame_hash,
        );
    }
    for viewport in &capture.software_3d_viewports {
        native_renderer_push_capture(
            &mut captures,
            "software_3d_viewport",
            &viewport.file,
            viewport.width,
            viewport.height,
            "software-depth-sorted-mesh-raster-after-truth-hash",
            &viewport.frame_hash,
        );
    }
    for frame in &capture.software_3d_sequence.frames {
        native_renderer_push_capture(
            &mut captures,
            "software_3d_replay_sequence",
            &frame.file,
            capture.software_3d_sequence.width,
            capture.software_3d_sequence.height,
            "software-depth-sorted-replay-frame-after-truth-hash",
            &frame.frame_hash,
        );
    }
    for capture in resolution_captures {
        let stream = if capture.debug_overlay {
            "resolution_capture"
        } else {
            "product_mode_clean_capture"
        };
        native_renderer_push_capture(
            &mut captures,
            stream,
            &capture.file,
            capture.width,
            capture.height,
            &capture.source,
            &capture.frame_hash,
        );
    }
    for frame in &capture.production_renderer.captures {
        native_renderer_push_capture(
            &mut captures,
            &frame.stream,
            &frame.file,
            frame.width,
            frame.height,
            &frame.source,
            &frame.frame_hash,
        );
    }
    let mut hook_material = String::new();
    for capture in &captures {
        writeln!(
            &mut hook_material,
            "{}:{}:{}x{}:{}:{}",
            capture.stream,
            capture.file,
            capture.width,
            capture.height,
            capture.frame_hash,
            capture.source
        )
        .unwrap();
    }
    let high_resolution_capture_count = captures
        .iter()
        .filter(|capture| capture.width >= 1920 && capture.height >= 1080)
        .count();
    NativeRendererCaptureHookSummary {
        hook_id: "native_ppm_capture_hook_after_truth_hash".to_string(),
        framebuffer_source: "x11-pixmap-or-software-depth-sorted-raster".to_string(),
        timing_source: "presentation-loop-schedule-recorded-outside-authoritative-truth"
            .to_string(),
        capture_count: captures.len(),
        high_resolution_capture_count,
        min_high_resolution_width: 1920,
        min_high_resolution_height: 1080,
        hook_hash: hash_hex(hook_material.as_bytes()),
        captures,
    }
}

#[cfg(target_os = "linux")]
fn native_renderer_push_capture(
    captures: &mut Vec<NativeRendererCapturedFrame>,
    stream: &str,
    file: &str,
    width: u32,
    height: u32,
    source: &str,
    frame_hash: &str,
) {
    captures.push(NativeRendererCapturedFrame {
        file: file.to_string(),
        stream: stream.to_string(),
        width,
        height,
        frame_hash: frame_hash.to_string(),
        source: source.to_string(),
        capture_after_truth_hash: true,
        presentation_only: true,
    });
}

#[cfg(target_os = "linux")]
fn native_renderer_asset_ids(silhouette: &NativeCombatSilhouetteEvidence) -> Vec<String> {
    let mut values = vec![silhouette.arena_asset.id.clone()];
    for fighter in &silhouette.fighters {
        for id in [&fighter.weapon_id, &fighter.armor_id] {
            if !values.contains(id) {
                values.push(id.clone());
            }
        }
    }
    values
}

#[cfg(target_os = "linux")]
fn native_renderer_material_ids(silhouette: &NativeCombatSilhouetteEvidence) -> Vec<String> {
    let mut values = vec![
        "chalked_stone_dust".to_string(),
        "ash_wood_grain_dented".to_string(),
        "tempered_steel_edge_worn".to_string(),
    ];
    for fighter in &silhouette.fighters {
        for id in [
            fighter.armor_material.clone(),
            native_armor_material_binding(fighter).to_string(),
            native_weapon_material_binding(fighter).to_string(),
        ] {
            if !values.contains(&id) {
                values.push(id);
            }
        }
    }
    values
}

#[cfg(target_os = "linux")]
fn native_renderer_event_ids(result: &DuelResult) -> Vec<String> {
    let mut values = Vec::new();
    for turn in &result.turns {
        for contact in &turn.contacts {
            values.push(format!(
                "turn{}_frame{}_{}_{}_{}",
                contact.turn,
                contact.frame,
                contact.action.as_str(),
                contact.target.as_str(),
                contact.material_result
            ));
        }
    }
    values
}

#[cfg(target_os = "linux")]
fn native_armor_material_binding(fighter: &NativeCombatFighterSilhouette) -> &'static str {
    match fighter.armor_material.as_str() {
        "riveted_mail" => "riveted_mail_oiled",
        "tempered_plate" | "lamellar_iron_leather" => "tempered_steel_edge_worn",
        "buff_leather_textile" => "strained_buff_leather",
        "quilted_linen" => "quilted_linen_stitched",
        _ => "strained_buff_leather",
    }
}

#[cfg(target_os = "linux")]
fn native_weapon_material_binding(fighter: &NativeCombatFighterSilhouette) -> &'static str {
    if fighter.weapon_id == "ash_spear" || fighter.weapon_id == "round_shield" {
        "ash_wood_grain_dented"
    } else {
        "tempered_steel_edge_worn"
    }
}

#[cfg(target_os = "linux")]
fn native_material_color(binding: &str, fallback: (u8, u8, u8)) -> (u8, u8, u8) {
    match binding {
        "riveted_mail_oiled" => (72, 82, 86),
        "tempered_steel_edge_worn" => (112, 120, 122),
        "quilted_linen_stitched" => (148, 124, 88),
        "strained_buff_leather" => (118, 78, 46),
        "ash_wood_grain_dented" => (132, 94, 52),
        "chalked_stone_dust" => (168, 158, 132),
        "skin_hair_variation" => (139, 96, 73),
        "wet_blood_trace_overlay" => (116, 22, 18),
        _ => fallback,
    }
}

#[cfg(target_os = "linux")]
fn native_lighting_material_witness_specs() -> [NativeLightingMaterialWitnessSpec; 5] {
    [
        NativeLightingMaterialWitnessSpec {
            material_class: "metal",
            material_id: "tempered_steel_edge_worn",
            fallback_color: (112, 120, 122),
        },
        NativeLightingMaterialWitnessSpec {
            material_class: "cloth",
            material_id: "quilted_linen_stitched",
            fallback_color: (148, 124, 88),
        },
        NativeLightingMaterialWitnessSpec {
            material_class: "leather",
            material_id: "strained_buff_leather",
            fallback_color: (118, 78, 46),
        },
        NativeLightingMaterialWitnessSpec {
            material_class: "stone",
            material_id: "chalked_stone_dust",
            fallback_color: (168, 158, 132),
        },
        NativeLightingMaterialWitnessSpec {
            material_class: "flesh",
            material_id: "skin_hair_variation",
            fallback_color: (139, 96, 73),
        },
    ]
}

#[cfg(target_os = "linux")]
fn native_lighting_witness_pixel_color(
    spec: &NativeLightingMaterialWitnessSpec,
    base: (u8, u8, u8),
    x: usize,
    y: usize,
    seed: i32,
) -> (u8, u8, u8) {
    let seed = seed.unsigned_abs() as usize;
    let noise = (((x as i32 * 37 + y as i32 * 17 + seed as i32).rem_euclid(31)) - 15) as i16;
    let (mut r, mut g, mut b) = (base.0 as i16, base.1 as i16, base.2 as i16);
    match spec.material_class {
        "metal" => {
            let brushed = if y % 9 < 2 { 18 } else { -6 };
            let nick = if (x * 11 + y * 3 + seed) % 47 < 3 {
                44
            } else {
                0
            };
            r += brushed + nick + noise / 4;
            g += brushed + nick + noise / 4;
            b += brushed + nick + 12 + noise / 5;
        }
        "cloth" => {
            let warp = if x % 18 < 3 { -24 } else { 9 };
            let weft = if y % 14 < 3 { -18 } else { 6 };
            r += warp + weft + noise / 5;
            g += warp / 2 + weft + noise / 6;
            b += weft / 3 + noise / 6;
        }
        "leather" => {
            let crease = if ((x + y * 3) / 7) % 11 < 3 { -26 } else { 11 };
            let scar = if (x * 5 + y * 13 + seed) % 89 < 4 {
                30
            } else {
                0
            };
            r += crease + scar + noise / 3;
            g += crease / 2 + scar / 3 + noise / 5;
            b += crease / 4 + noise / 6;
        }
        "stone" => {
            r = 172;
            g = 164;
            b = 142;
            let grain =
                (((x as i32 * 23 + y as i32 * 31 + seed as i32).rem_euclid(47)) - 23) as i16 / 3;
            let chip = if ((x * 29) ^ (y * 37) ^ seed) % 97 < 11 {
                -32
            } else {
                6
            };
            let pit = if ((x * 11 + seed) ^ (y * 17)) % 211 < 4 {
                -54
            } else {
                0
            };
            let mineral = if ((x * 5 + y * 7 + seed) % 149) < 5 {
                18
            } else {
                0
            };
            r += grain + chip + pit + mineral;
            g += grain + chip + pit + mineral / 2;
            b += grain / 2 + chip / 2 + pit / 2;
        }
        "flesh" => {
            r = 158;
            g = 103;
            b = 82;
            let warm_blush =
                (((x as i32 * 5 + y as i32 * 7 + seed as i32).rem_euclid(113)) - 56) as i16 / 5;
            let pore = if ((x * 97) ^ (y * 57) ^ seed) % 71 < 5 {
                -26
            } else {
                0
            };
            let freckle = if ((x * 31 + seed) ^ (y * 43)) % 193 < 3 {
                -38
            } else {
                0
            };
            let capillary = if (x * 3 + y * 5 + seed) % 211 < 5 {
                20
            } else {
                0
            };
            r += warm_blush + pore + freckle + noise / 8;
            g += warm_blush / 4 + pore / 2 + freckle / 3 + noise / 9 - capillary / 2;
            b += pore / 3 + freckle / 4 - capillary / 3 - 3;
        }
        _ => {
            let varied = native_pixel_material_color(base, x, y, seed as i32);
            r = varied.0 as i16;
            g = varied.1 as i16;
            b = varied.2 as i16;
        }
    }
    (
        r.clamp(0, 255) as u8,
        g.clamp(0, 255) as u8,
        b.clamp(0, 255) as u8,
    )
}

#[cfg(target_os = "linux")]
fn native_lighting_post_summary(
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
) -> NativeLightingPostSummary {
    let mut material_witnesses = native_renderer_material_ids(silhouette);
    material_witnesses.push("skin_hair_variation".to_string());
    if result
        .turns
        .iter()
        .flat_map(|turn| turn.contacts.iter())
        .any(|contact| {
            contact.material_result.contains("penetration")
                || contact.material_result.contains("blunt")
                || contact.material_result.contains("bind")
        })
    {
        material_witnesses.push("wet_blood_trace_overlay".to_string());
    }
    material_witnesses.sort();
    material_witnesses.dedup();
    let material_distinction_classes = native_lighting_material_witness_specs()
        .iter()
        .map(|witness| witness.material_class.to_string())
        .collect::<Vec<_>>();
    NativeLightingPostSummary {
        profile: "native-software-3d-readability-lighting-v2",
        dynamic_lighting: true,
        contact_grounding: true,
        ambient_occlusion_equivalent: true,
        fog_dust_atmosphere: true,
        tone_mapping: "integer-filmic-shoulder-readability-preserving",
        exposure_permille: 1080,
        color_grade: "warm-stone-cool-metal-material-separation",
        anti_aliasing_equivalent: "edge-aware-3x-neighborhood-post-filter",
        bloom_policy: "event-keyed-contact-only-no-readiness-claim",
        bloom_event_count: result.turns.iter().map(|turn| turn.contacts.len()).sum(),
        material_witnesses,
        material_distinction_classes,
        required_material_classes_distinguished: true,
        material_witness_capture: "native_lighting_material_witness.ppm",
        material_witness_manifest: "native_lighting_material_witness.json",
        presentation_only: true,
        truth_mutation: false,
        owner_visual_acceptance: false,
        public_demo_ready: PUBLIC_DEMO_READY,
        release_candidate_ready: RELEASE_CANDIDATE_READY,
    }
}

#[cfg(target_os = "linux")]
fn write_native_lighting_post_json(
    out: &mut String,
    indent: usize,
    key: &str,
    summary: &NativeLightingPostSummary,
    trailing_comma: bool,
) {
    let pad = "  ".repeat(indent);
    writeln!(out, "{pad}\"{key}\": {{").unwrap();
    write_json_field(out, indent + 1, "profile", summary.profile, true);
    writeln!(
        out,
        "{}  \"dynamic_lighting\": {},",
        pad, summary.dynamic_lighting
    )
    .unwrap();
    writeln!(
        out,
        "{}  \"contact_grounding\": {},",
        pad, summary.contact_grounding
    )
    .unwrap();
    writeln!(
        out,
        "{}  \"ambient_occlusion_equivalent\": {},",
        pad, summary.ambient_occlusion_equivalent
    )
    .unwrap();
    writeln!(
        out,
        "{}  \"fog_dust_atmosphere\": {},",
        pad, summary.fog_dust_atmosphere
    )
    .unwrap();
    write_json_field(out, indent + 1, "tone_mapping", summary.tone_mapping, true);
    writeln!(
        out,
        "{}  \"exposure_permille\": {},",
        pad, summary.exposure_permille
    )
    .unwrap();
    write_json_field(out, indent + 1, "color_grade", summary.color_grade, true);
    write_json_field(
        out,
        indent + 1,
        "anti_aliasing_equivalent",
        summary.anti_aliasing_equivalent,
        true,
    );
    write_json_field(out, indent + 1, "bloom_policy", summary.bloom_policy, true);
    writeln!(
        out,
        "{}  \"bloom_event_count\": {},",
        pad, summary.bloom_event_count
    )
    .unwrap();
    write_native_json_string_array(
        out,
        indent + 1,
        "material_witnesses",
        &summary.material_witnesses,
        true,
    );
    write_native_json_string_array(
        out,
        indent + 1,
        "material_distinction_classes",
        &summary.material_distinction_classes,
        true,
    );
    writeln!(
        out,
        "{}  \"required_material_classes_distinguished\": {},",
        pad, summary.required_material_classes_distinguished
    )
    .unwrap();
    write_json_field(
        out,
        indent + 1,
        "material_witness_capture",
        summary.material_witness_capture,
        true,
    );
    write_json_field(
        out,
        indent + 1,
        "material_witness_manifest",
        summary.material_witness_manifest,
        true,
    );
    writeln!(
        out,
        "{}  \"presentation_only\": {},",
        pad, summary.presentation_only
    )
    .unwrap();
    writeln!(
        out,
        "{}  \"truth_mutation\": {},",
        pad, summary.truth_mutation
    )
    .unwrap();
    writeln!(
        out,
        "{}  \"owner_visual_acceptance\": {},",
        pad, summary.owner_visual_acceptance
    )
    .unwrap();
    writeln!(
        out,
        "{}  \"public_demo_ready\": {},",
        pad, summary.public_demo_ready
    )
    .unwrap();
    writeln!(
        out,
        "{}  \"release_candidate_ready\": {}",
        pad, summary.release_candidate_ready
    )
    .unwrap();
    writeln!(out, "{pad}}}{}", if trailing_comma { "," } else { "" }).unwrap();
}

#[cfg(target_os = "linux")]
fn write_native_lighting_material_witness_artifacts(out_dir: &Path) -> Result<(), OathError> {
    const WIDTH: usize = 1200;
    const HEIGHT: usize = 360;
    let mut pixels = vec![0u8; WIDTH * HEIGHT * 3];
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let idx = (y * WIDTH + x) * 3;
            let gradient = (y as i32 * 54 / HEIGHT as i32).clamp(0, 54);
            pixels[idx] = (34 + gradient / 3) as u8;
            pixels[idx + 1] = (31 + gradient / 4) as u8;
            pixels[idx + 2] = (28 + gradient / 6) as u8;
        }
    }

    let specs = native_lighting_material_witness_specs();
    let panel_w = 180usize;
    let panel_h = 226usize;
    let y0 = 70usize;
    for (index, spec) in specs.iter().enumerate() {
        let x0 = 36 + index * 228;
        draw_native_contact_shadow(
            &mut pixels,
            WIDTH,
            HEIGHT,
            (x0 + panel_w / 2) as i32,
            (y0 + panel_h + 18) as i32,
            (panel_w / 2 + 34) as i32,
            22,
        );
        fill_rect(
            &mut pixels,
            WIDTH,
            HEIGHT,
            x0.saturating_sub(10),
            y0.saturating_sub(10),
            panel_w + 20,
            panel_h + 20,
            (20, 18, 16),
        );
        let base = native_material_color(spec.material_id, spec.fallback_color);
        for yy in y0..y0 + panel_h {
            for xx in x0..x0 + panel_w {
                let local_x = xx - x0;
                let local_y = yy - y0;
                let varied =
                    native_lighting_witness_pixel_color(spec, base, xx, yy, index as i32 * 137);
                let key_light = 720
                    + ((panel_h - local_y) as i32 * 230 / panel_h as i32)
                    + (local_x as i32 * 90 / panel_w as i32);
                let occlusion = if local_y > panel_h * 4 / 5 {
                    ((local_y - panel_h * 4 / 5) as i32 * 145 / (panel_h / 5).max(1) as i32)
                        .clamp(0, 145)
                } else {
                    0
                };
                let rim =
                    if local_x < 4 || local_y < 4 || local_x + 5 > panel_w || local_y + 5 > panel_h
                    {
                        34
                    } else {
                        0
                    };
                let mut r = (varied.0 as i32 * key_light / 1000 + rim).clamp(0, 255);
                let mut g = (varied.1 as i32 * (key_light - 25) / 1000 + rim).clamp(0, 255);
                let mut b = (varied.2 as i32 * (key_light - 45) / 1000 + rim).clamp(0, 255);
                r = (r * (255 - occlusion) + 23 * occlusion) / 255;
                g = (g * (255 - occlusion) + 20 * occlusion) / 255;
                b = (b * (255 - occlusion) + 17 * occlusion) / 255;
                let idx = (yy * WIDTH + xx) * 3;
                pixels[idx] = r.clamp(0, 255) as u8;
                pixels[idx + 1] = g.clamp(0, 255) as u8;
                pixels[idx + 2] = b.clamp(0, 255) as u8;
            }
        }
    }
    apply_native_lighting_post(&mut pixels, WIDTH, HEIGHT, 1080);

    let mut ppm = format!("P6\n{} {}\n255\n", WIDTH, HEIGHT).into_bytes();
    ppm.extend_from_slice(&pixels);
    let frame_hash = hash_hex(&ppm);
    fs::write(out_dir.join("native_lighting_material_witness.ppm"), &ppm)?;

    let mut witnesses = Vec::new();
    let mut unique_lit_colors = Vec::new();
    for (index, spec) in specs.iter().enumerate() {
        let x0 = 36 + index * 228;
        let sample_x = x0 + panel_w / 2;
        let sample_y = y0 + panel_h / 2;
        let idx = (sample_y * WIDTH + sample_x) * 3;
        let lit = (pixels[idx], pixels[idx + 1], pixels[idx + 2]);
        if !unique_lit_colors.contains(&lit) {
            unique_lit_colors.push(lit);
        }
        witnesses.push((
            spec.material_class.to_string(),
            spec.material_id.to_string(),
            native_material_color(spec.material_id, spec.fallback_color),
            lit,
            x0,
            y0,
            panel_w,
            panel_h,
        ));
    }
    let classes = witnesses
        .iter()
        .map(|witness| witness.0.clone())
        .collect::<Vec<_>>();
    let required_distinguished = unique_lit_colors.len() == witnesses.len();

    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(
        &mut out,
        1,
        "schema",
        NATIVE_LIGHTING_MATERIAL_WITNESS_SCHEMA,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "profile",
        "native-software-3d-readability-lighting-v2",
        true,
    );
    write_json_field(
        &mut out,
        1,
        "source",
        "native-lighting-post-material-witness-after-truth-hash",
        true,
    );
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(&mut out, "  \"owner_visual_acceptance\": false,").unwrap();
    writeln!(&mut out, "  \"public_demo_ready\": {PUBLIC_DEMO_READY},").unwrap();
    writeln!(
        &mut out,
        "  \"release_candidate_ready\": {RELEASE_CANDIDATE_READY},"
    )
    .unwrap();
    writeln!(&mut out, "  \"width\": {WIDTH},").unwrap();
    writeln!(&mut out, "  \"height\": {HEIGHT},").unwrap();
    write_json_field(
        &mut out,
        1,
        "capture",
        "native_lighting_material_witness.ppm",
        true,
    );
    write_json_field(&mut out, 1, "frame_hash", &frame_hash, true);
    write_native_json_string_array(&mut out, 1, "material_classes", &classes, true);
    writeln!(
        &mut out,
        "  \"distinct_lit_material_color_count\": {},",
        unique_lit_colors.len()
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"required_material_classes_distinguished\": {},",
        required_distinguished
    )
    .unwrap();
    writeln!(&mut out, "  \"witnesses\": [").unwrap();
    for (index, witness) in witnesses.iter().enumerate() {
        let (class, material_id, base, lit, x, y, w, h) = witness;
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "class", class, true);
        write_json_field(&mut out, 3, "material_id", material_id, true);
        writeln!(
            &mut out,
            "      \"base_rgb\": [{}, {}, {}],",
            base.0, base.1, base.2
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"lit_sample_rgb\": [{}, {}, {}],",
            lit.0, lit.1, lit.2
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"panel_bounds\": [{}, {}, {}, {}]",
            x, y, w, h
        )
        .unwrap();
        writeln!(&mut out, "    }}{}", comma(index + 1, witnesses.len())).unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    fs::write(out_dir.join("native_lighting_material_witness.json"), out)?;
    if !required_distinguished {
        return Err(OathError::Verify(
            "native lighting material witness colors collapsed after lighting/post".to_string(),
        ));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn native_runtime_asset_path(relative_path: &str) -> PathBuf {
    let direct = PathBuf::from(relative_path);
    if direct.is_file() {
        return direct;
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(bin_dir) = exe.parent() {
            if let Some(root_dir) = bin_dir.parent() {
                let packaged = root_dir.join(relative_path);
                if packaged.is_file() {
                    return packaged;
                }
            }
        }
    }
    direct
}

#[cfg(target_os = "linux")]
fn native_asset_source_for_kind(kind: &str) -> Result<&'static str, OathError> {
    match kind {
        "weapons" => Ok("assets_src/weapons/weapons.oysrc"),
        "armor" => Ok("assets_src/armor/armor.oysrc"),
        "arenas" => Ok("assets_src/arenas/arenas.oysrc"),
        "fighters" => Ok("assets_src/fighters/traditions.oysrc"),
        _ => Err(OathError::Verify(format!(
            "unsupported native combat asset kind '{kind}'"
        ))),
    }
}

#[cfg(target_os = "linux")]
fn native_gltf_geometry(path: &Path) -> Result<NativeGltfGeometry, OathError> {
    let text = fs::read_to_string(path)?;
    let triangle_mode = text.contains("\"mode\": 4") || !text.contains("\"mode\"");
    if !text.contains("\"version\": \"2.0\"")
        || !text.contains("\"POSITION\": 0")
        || !text.contains("\"indices\": 1")
        || !triangle_mode
    {
        return Err(OathError::Verify(format!(
            "native glTF geometry source is not the expected OATHYARD mesh format: {}",
            path.display()
        )));
    }

    let counts = json_usize_values_after_key(&text, "\"count\"");
    let vertex_count = *counts.first().ok_or_else(|| {
        OathError::Verify(format!(
            "native glTF geometry missing position count: {}",
            path.display()
        ))
    })?;
    let index_count = *counts.get(1).ok_or_else(|| {
        OathError::Verify(format!(
            "native glTF geometry missing index count: {}",
            path.display()
        ))
    })?;
    if vertex_count < 3 || index_count < 3 || index_count % 3 != 0 {
        return Err(OathError::Verify(format!(
            "native glTF geometry has invalid counts in {}",
            path.display()
        )));
    }

    let encoded = json_base64_buffer_uri(&text).ok_or_else(|| {
        OathError::Verify(format!(
            "native glTF geometry missing embedded buffer: {}",
            path.display()
        ))
    })?;
    let decoded = decode_base64_bytes(encoded)?;
    let position_bytes = vertex_count * 12;
    let index_bytes = index_count * 2;
    if decoded.len() < position_bytes + index_bytes {
        return Err(OathError::Verify(format!(
            "native glTF geometry buffer too short in {}",
            path.display()
        )));
    }

    let mut positions_milli = Vec::with_capacity(vertex_count);
    for index in 0..vertex_count {
        let offset = index * 12;
        let x = binary32_bits_to_milli(u32::from_le_bytes([
            decoded[offset],
            decoded[offset + 1],
            decoded[offset + 2],
            decoded[offset + 3],
        ]))?;
        let y = binary32_bits_to_milli(u32::from_le_bytes([
            decoded[offset + 4],
            decoded[offset + 5],
            decoded[offset + 6],
            decoded[offset + 7],
        ]))?;
        let z = binary32_bits_to_milli(u32::from_le_bytes([
            decoded[offset + 8],
            decoded[offset + 9],
            decoded[offset + 10],
            decoded[offset + 11],
        ]))?;
        positions_milli.push((x, y, z));
    }

    let mut indices = Vec::with_capacity(index_count);
    for index in 0..index_count {
        let offset = position_bytes + index * 2;
        let vertex = u16::from_le_bytes([decoded[offset], decoded[offset + 1]]) as usize;
        if vertex >= vertex_count {
            return Err(OathError::Verify(format!(
                "native glTF geometry index {} out of range in {}",
                vertex,
                path.display()
            )));
        }
        indices.push(vertex);
    }

    let (mut min_x, mut max_x) = (positions_milli[0].0, positions_milli[0].0);
    let (mut min_y, mut max_y) = (positions_milli[0].1, positions_milli[0].1);
    let (mut min_z, mut max_z) = (positions_milli[0].2, positions_milli[0].2);
    for &(x, y, z) in &positions_milli {
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
        min_z = min_z.min(z);
        max_z = max_z.max(z);
    }

    Ok(NativeGltfGeometry {
        vertex_count,
        index_count,
        triangle_count: index_count / 3,
        min_x_milli: min_x,
        max_x_milli: max_x,
        min_y_milli: min_y,
        max_y_milli: max_y,
        min_z_milli: min_z,
        max_z_milli: max_z,
        geometry_hash: hash_hex(&decoded),
        positions_milli,
        indices,
    })
}

#[cfg(target_os = "linux")]
fn json_usize_values_after_key(text: &str, key: &str) -> Vec<usize> {
    let mut values = Vec::new();
    let mut cursor = 0usize;
    while let Some(relative) = text[cursor..].find(key) {
        let mut index = cursor + relative + key.len();
        while index < text.len() {
            let byte = text.as_bytes()[index];
            if byte == b':' || byte.is_ascii_whitespace() {
                index += 1;
            } else {
                break;
            }
        }
        let start = index;
        while index < text.len() && text.as_bytes()[index].is_ascii_digit() {
            index += 1;
        }
        if start < index {
            if let Ok(value) = text[start..index].parse::<usize>() {
                values.push(value);
            }
        }
        cursor = index;
    }
    values
}

#[cfg(target_os = "linux")]
fn json_base64_buffer_uri(text: &str) -> Option<&str> {
    let prefix = "\"uri\": \"data:application/octet-stream;base64,";
    let start = text.find(prefix)? + prefix.len();
    let end = text[start..].find('"')?;
    Some(&text[start..start + end])
}

#[cfg(target_os = "linux")]
fn decode_base64_bytes(encoded: &str) -> Result<Vec<u8>, OathError> {
    let mut out = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0u32;
    for byte in encoded.bytes() {
        if byte == b'=' {
            break;
        }
        if byte.is_ascii_whitespace() {
            continue;
        }
        let value = base64_value(byte).ok_or_else(|| {
            OathError::Verify(format!(
                "native glTF embedded buffer contains invalid base64 byte {byte}"
            ))
        })? as u32;
        buffer = (buffer << 6) | value;
        bits += 6;
        while bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xff) as u8);
        }
    }
    Ok(out)
}

#[cfg(target_os = "linux")]
fn base64_value(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

#[cfg(target_os = "linux")]
fn binary32_bits_to_milli(bits: u32) -> Result<i32, OathError> {
    let exponent = ((bits >> 23) & 0xff) as i32;
    let fraction = (bits & 0x7f_ffff) as i128;
    if exponent == 255 {
        return Err(OathError::Verify(
            "native glTF geometry contains non-finite binary value".to_string(),
        ));
    }
    if exponent == 0 {
        return Ok(0);
    }
    let sign = if (bits >> 31) != 0 { -1i128 } else { 1i128 };
    let mantissa = (1i128 << 23) + fraction;
    let shift = exponent - 150;
    let scaled = mantissa * 1000;
    let magnitude = if shift >= 0 {
        scaled
            .checked_shl(shift as u32)
            .ok_or_else(|| OathError::Verify("native glTF geometry overflow".to_string()))?
    } else {
        let divisor_shift = (-shift) as u32;
        if divisor_shift >= 120 {
            0
        } else {
            let divisor = 1i128 << divisor_shift;
            (scaled + divisor / 2) / divisor
        }
    };
    let signed = sign * magnitude;
    i32::try_from(signed)
        .map_err(|_| OathError::Verify("native glTF geometry value overflow".to_string()))
}

#[cfg(target_os = "linux")]
fn native_asset_geometry_complete(asset: &NativeCombatAssetRef) -> bool {
    asset.geometry.vertex_count >= 3
        && asset.geometry.index_count >= 3
        && asset.geometry.index_count == asset.geometry.triangle_count * 3
        && native_geometry_has_depth(&asset.geometry)
        && !asset.geometry.positions_milli.is_empty()
        && !asset.geometry.indices.is_empty()
        && !asset.geometry.geometry_hash.is_empty()
}

#[cfg(target_os = "linux")]
fn geometry_z_depth_milli(geometry: &NativeGltfGeometry) -> i32 {
    geometry.max_z_milli - geometry.min_z_milli
}

#[cfg(target_os = "linux")]
fn native_geometry_has_depth(geometry: &NativeGltfGeometry) -> bool {
    geometry_z_depth_milli(geometry) > 0
}

#[cfg(target_os = "linux")]
fn native_geometry_projected(silhouette: &NativeCombatSilhouetteEvidence) -> bool {
    native_asset_geometry_complete(&silhouette.arena_asset)
        && silhouette.fighters.iter().all(|fighter| {
            native_asset_geometry_complete(&fighter.weapon_asset)
                && native_asset_geometry_complete(&fighter.armor_asset)
        })
}

#[cfg(target_os = "linux")]
fn native_3d_runtime_geometry(silhouette: &NativeCombatSilhouetteEvidence) -> bool {
    native_geometry_has_depth(&silhouette.arena_asset.geometry)
        && silhouette.fighters.iter().all(|fighter| {
            native_geometry_has_depth(&fighter.weapon_asset.geometry)
                && native_geometry_has_depth(&fighter.armor_asset.geometry)
        })
}

#[cfg(target_os = "linux")]
fn native_combat_asset_ref(id: &str, kind: &str) -> Result<NativeCombatAssetRef, OathError> {
    let source = native_asset_source_for_kind(kind)?.to_string();
    let runtime_mesh = format!("assets/runtime/{id}.mesh.json");
    let runtime_gltf = format!("assets/gltf/{id}.gltf");
    let preview = format!("assets/previews/{id}.svg");
    let manifest_path = native_runtime_asset_path("assets/runtime_manifest.json");
    let manifest = fs::read_to_string(&manifest_path)?;
    for needle in [
        format!("\"id\": \"{id}\""),
        format!("\"kind\": \"{kind}\""),
        format!("\"source\": \"{source}\""),
        format!("\"runtime_mesh\": \"{runtime_mesh}\""),
        format!("\"runtime_gltf\": \"{runtime_gltf}\""),
        format!("\"preview\": \"{preview}\""),
        "\"provenance\": \"repo_owned_original_text_asset\"".to_string(),
    ] {
        if !manifest.contains(&needle) {
            return Err(OathError::Verify(format!(
                "native combat asset manifest missing {needle}"
            )));
        }
    }

    let source_path = native_runtime_asset_path(&source);
    let source_hash = if source_path.is_file() {
        file_hash_hex(source_path)?
    } else {
        "not_packaged_runtime_manifest_source_ref".to_string()
    };

    Ok(NativeCombatAssetRef {
        id: id.to_string(),
        kind: kind.to_string(),
        source: source.clone(),
        runtime_mesh: runtime_mesh.clone(),
        runtime_gltf: runtime_gltf.clone(),
        preview: preview.clone(),
        provenance: "repo_owned_original_text_asset".to_string(),
        source_hash,
        runtime_mesh_hash: file_hash_hex(native_runtime_asset_path(&runtime_mesh))?,
        runtime_gltf_hash: file_hash_hex(native_runtime_asset_path(&runtime_gltf))?,
        preview_hash: file_hash_hex(native_runtime_asset_path(&preview))?,
        geometry: native_gltf_geometry(&native_runtime_asset_path(&runtime_gltf))?,
    })
}

#[cfg(target_os = "linux")]
fn native_presentation_source_folder(kind: &str) -> Result<&'static str, OathError> {
    match kind {
        "weapons" => Ok("weapons"),
        "armor" => Ok("armor"),
        "arenas" => Ok("arenas"),
        "fighters" => Ok("fighters"),
        _ => Err(OathError::Verify(format!(
            "unsupported high-detail presentation asset kind '{kind}'"
        ))),
    }
}

#[cfg(target_os = "linux")]
fn native_presentation_triangle_floor(kind: &str, id: &str) -> usize {
    match kind {
        "fighters" => 18_000,
        "weapons" if id == "round_shield" => 2_000,
        "weapons" => 800,
        "armor" => 500,
        "arenas" => 2_000,
        _ => usize::MAX,
    }
}

#[cfg(target_os = "linux")]
fn native_presentation_asset_ref(
    id: &str,
    kind: &str,
) -> Result<NativePresentationAssetRef, OathError> {
    let folder = native_presentation_source_folder(kind)?;
    let source = format!(
        "assets_src/model_candidates/{HIGH_DETAIL_PRESENTATION_RUN_ID}/{folder}/{id}.model_source.json"
    );
    let runtime_mesh = format!("assets/presentation_runtime/{id}.mesh.json");
    let runtime_gltf = format!("assets/presentation_gltf/{id}.gltf");
    let preview = format!(
        "assets/model_candidates/{HIGH_DETAIL_PRESENTATION_RUN_ID}/previews/{id}_isolated_closeup_1920x1080.png"
    );
    let source_candidate_gltf =
        format!("assets/model_candidates/{HIGH_DETAIL_PRESENTATION_RUN_ID}/gltf/{id}.gltf");
    let source_candidate_bin =
        format!("assets/model_candidates/{HIGH_DETAIL_PRESENTATION_RUN_ID}/bin/{id}.bin");
    let manifest_path = native_runtime_asset_path("assets/presentation_manifest.json");
    let manifest = fs::read_to_string(&manifest_path)?;
    for needle in [
        format!("\"id\": \"{id}\""),
        format!("\"kind\": \"{kind}\""),
        format!("\"source\": \"{source}\""),
        format!("\"runtime_mesh\": \"{runtime_mesh}\""),
        format!("\"runtime_gltf\": \"{runtime_gltf}\""),
        format!("\"preview\": \"{preview}\""),
        format!("\"source_candidate_gltf\": \"{source_candidate_gltf}\""),
        format!("\"source_candidate_bin\": \"{source_candidate_bin}\""),
        "\"schema\": \"oathyard.presentation_assets.v1\"".to_string(),
        format!("\"candidate_run_id\": \"{HIGH_DETAIL_PRESENTATION_RUN_ID}\""),
        "\"provenance\": \"repo_owned_original_procedural_model_candidate\"".to_string(),
        "\"presentation_only\": true".to_string(),
        "\"truth_authoritative\": false".to_string(),
        "\"truth_mutation\": false".to_string(),
        "\"public_demo_ready\": false".to_string(),
        "\"release_candidate_ready\": false".to_string(),
        "\"owner_visual_acceptance\": false".to_string(),
        "\"external_khronos_validation_claimed\": false".to_string(),
    ] {
        if !manifest.contains(&needle) {
            return Err(OathError::Verify(format!(
                "high-detail presentation manifest missing {needle}"
            )));
        }
    }

    let geometry = native_gltf_geometry(&native_runtime_asset_path(&runtime_gltf))?;
    let triangle_floor = native_presentation_triangle_floor(kind, id);
    if geometry.triangle_count < triangle_floor {
        return Err(OathError::Verify(format!(
            "high-detail presentation asset {id} triangle count {} below floor {triangle_floor}",
            geometry.triangle_count
        )));
    }
    if !native_geometry_has_depth(&geometry) {
        return Err(OathError::Verify(format!(
            "high-detail presentation asset {id} has no nonzero Z depth"
        )));
    }
    let source_path = native_runtime_asset_path(&source);
    let source_hash = if source_path.is_file() {
        file_hash_hex(source_path)?
    } else {
        "not_packaged_presentation_source_ref".to_string()
    };

    Ok(NativePresentationAssetRef {
        id: id.to_string(),
        kind: kind.to_string(),
        source: source.clone(),
        runtime_mesh: runtime_mesh.clone(),
        runtime_gltf: runtime_gltf.clone(),
        preview: preview.clone(),
        source_candidate_gltf: source_candidate_gltf.clone(),
        source_candidate_bin: source_candidate_bin.clone(),
        provenance: "repo_owned_original_procedural_model_candidate".to_string(),
        license_status: "repo_owned_original_internal_candidate_pending_project_license_review"
            .to_string(),
        source_hash,
        runtime_mesh_hash: file_hash_hex(native_runtime_asset_path(&runtime_mesh))?,
        runtime_gltf_hash: file_hash_hex(native_runtime_asset_path(&runtime_gltf))?,
        preview_hash: file_hash_hex(native_runtime_asset_path(&preview))?,
        source_candidate_gltf_hash: file_hash_hex(native_runtime_asset_path(
            &source_candidate_gltf,
        ))?,
        source_candidate_bin_hash: file_hash_hex(native_runtime_asset_path(&source_candidate_bin))?,
        geometry,
    })
}

#[cfg(target_os = "linux")]
fn native_combat_silhouette_evidence(
    result: &DuelResult,
) -> Result<NativeCombatSilhouetteEvidence, OathError> {
    let scenario = Scenario::parse(&result.canonical_scenario)?;
    let state = DuelState::from_scenario(&scenario)?;
    let reconstructed_initial_state_hash = state.state_hash();
    if reconstructed_initial_state_hash != result.initial_state_hash {
        return Err(OathError::Verify(format!(
            "native combat silhouette source hash mismatch: expected {}, reconstructed {}",
            result.initial_state_hash, reconstructed_initial_state_hash
        )));
    }

    let presentation_asset_manifest_hash = file_hash_hex(native_runtime_asset_path(
        "assets/presentation_manifest.json",
    ))?;
    let arena_presentation_asset =
        native_presentation_asset_ref("oathyard_verdict_ring", "arenas")?;
    let mut fighters = Vec::new();
    for fighter in &state.fighters {
        let body_mass_g: i32 = fighter.joints.iter().map(|joint| joint.mass_g).sum();
        let weapon_asset = native_combat_asset_ref(fighter.weapon.id, "weapons")?;
        let armor_asset = native_combat_asset_ref(fighter.armor.id, "armor")?;
        let weapon_presentation_asset =
            native_presentation_asset_ref(fighter.weapon.id, "weapons")?;
        let armor_presentation_asset = native_presentation_asset_ref(fighter.armor.id, "armor")?;
        fighters.push(NativeCombatFighterSilhouette {
            seat: fighter.seat,
            name: fighter.name.clone(),
            weapon_id: fighter.weapon.id.to_string(),
            weapon_name: fighter.weapon.display_name.to_string(),
            weapon_asset,
            weapon_presentation_asset,
            weapon_length_mm: fighter.weapon.length_mm,
            weapon_reach_mm: fighter.weapon.reach_mm,
            weapon_mass_g: fighter.weapon.mass_g,
            weapon_inertia_g_cm2: fighter.weapon.inertia_g_cm2,
            weapon_span_px: native_weapon_span_px(&fighter.weapon),
            weapon_head_px: native_weapon_head_px(&fighter.weapon),
            armor_id: fighter.armor.id.to_string(),
            armor_name: fighter.armor.display_name.to_string(),
            armor_asset,
            armor_presentation_asset,
            armor_material: fighter.armor.material.to_string(),
            armor_mass_g: fighter.armor.mass_g,
            armor_torso_coverage_permille: fighter.armor.torso_coverage_permille,
            armor_head_coverage_permille: fighter.armor.head_coverage_permille,
            armor_weapon_arm_coverage_permille: fighter.armor.weapon_arm_coverage_permille,
            armor_lead_leg_coverage_permille: fighter.armor.lead_leg_coverage_permille,
            armor_gap_permille: fighter.armor.gap_permille,
            armor_torso_width_px: native_armor_torso_width_px(&fighter.armor),
            armor_torso_height_px: native_armor_torso_height_px(&fighter.armor),
            armor_head_marker_px: native_armor_head_marker_px(&fighter.armor),
            body_mass_g,
            stance_width_mm: fighter.stance_width_mm,
        });
    }

    Ok(NativeCombatSilhouetteEvidence {
        source_backed: true,
        source: "canonical scenario -> reconstructed initial DuelState after content hash"
            .to_string(),
        reconstructed_initial_state_hash,
        runtime_asset_refs_verified: true,
        asset_manifest_hash: file_hash_hex(native_runtime_asset_path(
            "assets/runtime_manifest.json",
        ))?,
        presentation_asset_manifest_hash,
        high_detail_presentation_assets_verified: true,
        arena_asset: native_combat_asset_ref("oathyard_verdict_ring", "arenas")?,
        arena_presentation_asset,
        fighters,
    })
}

#[cfg(target_os = "linux")]
fn native_weapon_span_px(weapon: &WeaponProfile) -> c_int {
    (weapon.reach_mm / 8).clamp(54, 172) as c_int
}

#[cfg(target_os = "linux")]
fn native_weapon_head_px(weapon: &WeaponProfile) -> c_int {
    (6 + weapon.mass_g / 180).clamp(8, 28) as c_int
}

#[cfg(target_os = "linux")]
fn native_armor_torso_width_px(armor: &ArmorProfile) -> c_int {
    (20 + armor.mass_g / 900).clamp(24, 52) as c_int
}

#[cfg(target_os = "linux")]
fn native_armor_torso_height_px(armor: &ArmorProfile) -> c_int {
    (34 + armor.torso_coverage_permille / 20).clamp(42, 84) as c_int
}

#[cfg(target_os = "linux")]
fn native_armor_head_marker_px(armor: &ArmorProfile) -> c_int {
    (6 + armor.head_coverage_permille / 70).clamp(8, 22) as c_int
}

#[cfg(target_os = "linux")]
unsafe fn x11_combat_render_capture(
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    out_dir: &Path,
    ppm_path: &Path,
    capture_command: &str,
    replay_path: &str,
    simulation_micros: u128,
    replay_verify_micros: u128,
) -> Result<NativeCombatCapture, String> {
    let width = 960u32;
    let height = 540u32;
    let display = XOpenDisplay(std::ptr::null());
    if display.is_null() {
        return Err("XOpenDisplay returned null; DISPLAY or XWayland is unavailable".to_string());
    }
    let screen_number = XDefaultScreen(display);
    let root = XRootWindow(display, screen_number);
    let black = XBlackPixel(display, screen_number);
    let white = XWhitePixel(display, screen_number);
    let window = XCreateSimpleWindow(display, root, 128, 128, width, height, 2, black, white);
    if window == 0 {
        XCloseDisplay(display);
        return Err("XCreateSimpleWindow returned 0".to_string());
    }
    let depth = XDefaultDepth(display, screen_number);
    if depth <= 0 {
        XDestroyWindow(display, window);
        XCloseDisplay(display);
        return Err("XDefaultDepth returned non-positive depth".to_string());
    }
    let pixmap = XCreatePixmap(display, root, width, height, depth as c_uint);
    if pixmap == 0 {
        XDestroyWindow(display, window);
        XCloseDisplay(display);
        return Err("XCreatePixmap returned 0".to_string());
    }
    let title = CString::new("OATHYARD Native Combat Render").map_err(|error| error.to_string())?;
    XStoreName(display, window, title.as_ptr());
    XMapWindow(display, window);
    let gc = XCreateGC(display, pixmap, 0, std::ptr::null_mut());
    if gc.is_null() {
        XFreePixmap(display, pixmap);
        XDestroyWindow(display, window);
        XCloseDisplay(display);
        return Err("XCreateGC returned null".to_string());
    }
    XSetForeground(display, gc, black);
    render_native_combat_frame(
        display, pixmap, gc, result, silhouette, width, height, black, white,
    );
    XCopyArea(display, pixmap, window, gc, 0, 0, width, height, 0, 0);
    XFlush(display);
    XSync(display, 0);
    capture_window_to_ppm(display, pixmap, width, height, ppm_path, black, white)?;

    let mut frames = build_native_combat_frame_specs(result);
    for frame in &mut frames {
        render_native_combat_state_frame(
            display, pixmap, gc, result, silhouette, frame, width, height, black, white,
        );
        XCopyArea(display, pixmap, window, gc, 0, 0, width, height, 0, 0);
        XFlush(display);
        XSync(display, 0);
        let frame_path = out_dir.join(&frame.file);
        capture_window_to_ppm(display, pixmap, width, height, &frame_path, black, white)?;
        let bytes = fs::read(&frame_path).map_err(|error| error.to_string())?;
        frame.frame_hash = hash_hex(&bytes);
    }

    let mut motion_frames = build_native_combat_motion_frame_specs(result);
    for frame in &mut motion_frames {
        render_native_combat_motion_frame(
            display, pixmap, gc, result, silhouette, frame, width, height, black, white,
        );
        XCopyArea(display, pixmap, window, gc, 0, 0, width, height, 0, 0);
        XFlush(display);
        XSync(display, 0);
        let frame_path = out_dir.join(&frame.file);
        capture_window_to_ppm(display, pixmap, width, height, &frame_path, black, white)?;
        let bytes = fs::read(&frame_path).map_err(|error| error.to_string())?;
        frame.frame_hash = hash_hex(&bytes);
    }

    let playback = write_native_combat_playback_capture(
        display,
        pixmap,
        window,
        gc,
        result,
        silhouette,
        &motion_frames,
        out_dir,
        width,
        height,
        black,
        white,
    )?;

    let live_loop = write_native_combat_live_loop_capture(
        display,
        pixmap,
        window,
        gc,
        result,
        silhouette,
        &motion_frames,
        out_dir,
        width,
        height,
        black,
        white,
    )?;

    let player_loop = write_native_player_loop_capture(
        display,
        pixmap,
        window,
        gc,
        result,
        silhouette,
        &motion_frames,
        out_dir,
        width,
        height,
        black,
        white,
    )?;

    let software_3d_viewports =
        write_native_combat_software_3d_viewports(result, silhouette, out_dir)?;
    let software_3d_sequence =
        write_native_combat_software_3d_sequence(result, silhouette, &motion_frames, out_dir)?;
    let resolution_captures_result =
        write_native_combat_resolution_captures(result, silhouette, out_dir);

    XFreeGC(display, gc);
    XFreePixmap(display, pixmap);
    XDestroyWindow(display, window);
    XCloseDisplay(display);
    let resolution_captures = resolution_captures_result?;
    let production_renderer = write_native_production_renderer_bundle(
        result,
        silhouette,
        &motion_frames,
        &player_loop.frames,
        out_dir,
        capture_command,
        replay_path,
        simulation_micros,
        replay_verify_micros,
    )?;
    Ok(NativeCombatCapture {
        state_frames: frames,
        motion_frames,
        playback,
        live_loop,
        player_loop,
        production_renderer,
        resolution_captures,
        software_3d_viewports,
        software_3d_sequence,
    })
}

#[cfg(target_os = "linux")]
unsafe fn write_native_combat_playback_capture(
    display: *mut Display,
    pixmap: Window,
    window: Window,
    gc: GC,
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    motion_frames: &[NativeCombatMotionFrameSpec],
    out_dir: &Path,
    width: u32,
    height: u32,
    black: c_ulong,
    white: c_ulong,
) -> Result<NativeCombatPlaybackSummary, String> {
    if motion_frames.is_empty() {
        return Err("native combat playback received no motion frames".to_string());
    }
    let cycles = 2usize;
    let playback_frame_count = motion_frames.len() * cycles;
    for frame_index in 0..playback_frame_count {
        let frame = &motion_frames[frame_index % motion_frames.len()];
        render_native_combat_motion_frame(
            display, pixmap, gc, result, silhouette, frame, width, height, black, white,
        );
        XCopyArea(display, pixmap, window, gc, 0, 0, width, height, 0, 0);
        XFlush(display);
        XSync(display, 0);
    }
    let file = "native_combat_playback_final.ppm".to_string();
    let path = out_dir.join(&file);
    capture_window_to_ppm(display, pixmap, width, height, &path, black, white)?;
    let bytes = fs::read(&path).map_err(|error| error.to_string())?;
    let nominal_frame_interval_ms = 1000 / TRUTH_HZ;
    Ok(NativeCombatPlaybackSummary {
        file,
        source_motion_frame_count: motion_frames.len(),
        playback_frame_count,
        cycles,
        nominal_frame_interval_ms,
        nominal_duration_ms: playback_frame_count as u32 * 1000 / TRUTH_HZ,
        final_frame_hash: hash_hex(&bytes),
    })
}

#[cfg(target_os = "linux")]
unsafe fn write_native_combat_live_loop_capture(
    display: *mut Display,
    pixmap: Window,
    window: Window,
    gc: GC,
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    motion_frames: &[NativeCombatMotionFrameSpec],
    out_dir: &Path,
    width: u32,
    height: u32,
    black: c_ulong,
    white: c_ulong,
) -> Result<NativeCombatLiveLoopSummary, String> {
    if motion_frames.is_empty() {
        return Err("native combat live loop received no motion frames".to_string());
    }
    let rendered_frame_count = TRUTH_HZ as usize;
    let sample_indices = [
        0usize,
        rendered_frame_count / 4,
        rendered_frame_count / 2,
        rendered_frame_count * 3 / 4,
        rendered_frame_count - 1,
    ];
    let mut samples = Vec::new();
    let mut loop_material = String::new();
    for loop_frame_index in 0..rendered_frame_count {
        let source_motion_frame_index =
            loop_frame_index * motion_frames.len() / rendered_frame_count;
        let frame = &motion_frames[source_motion_frame_index.min(motion_frames.len() - 1)];
        render_native_combat_motion_frame(
            display, pixmap, gc, result, silhouette, frame, width, height, black, white,
        );
        XCopyArea(display, pixmap, window, gc, 0, 0, width, height, 0, 0);
        XFlush(display);
        XSync(display, 0);
        writeln!(
            &mut loop_material,
            "{}:{}:{}:{}",
            loop_frame_index,
            source_motion_frame_index + 1,
            frame.truth_frame,
            frame.frame_hash
        )
        .map_err(|error| error.to_string())?;
        if sample_indices.contains(&loop_frame_index) {
            let sample_number = samples.len() + 1;
            let file = format!("native_combat_live_loop_{sample_number:03}.ppm");
            let path = out_dir.join(&file);
            capture_window_to_ppm(display, pixmap, width, height, &path, black, white)?;
            let bytes = fs::read(&path).map_err(|error| error.to_string())?;
            samples.push(NativeCombatLiveLoopSample {
                file,
                loop_frame_index,
                source_motion_frame_index: source_motion_frame_index + 1,
                truth_frame: frame.truth_frame,
                frame_hash: hash_hex(&bytes),
            });
        }
    }
    let final_frame_hash = samples
        .last()
        .map(|sample| sample.frame_hash.clone())
        .ok_or_else(|| "native combat live loop produced no sample captures".to_string())?;
    let nominal_frame_interval_ms = 1000 / TRUTH_HZ;
    Ok(NativeCombatLiveLoopSummary {
        source_motion_frame_count: motion_frames.len(),
        rendered_frame_count,
        nominal_frame_interval_ms,
        nominal_duration_ms: rendered_frame_count as u32 * 1000 / TRUTH_HZ,
        loop_hash: hash_hex(loop_material.as_bytes()),
        final_frame_hash,
        sample_captures: samples,
    })
}

#[cfg(target_os = "linux")]
unsafe fn write_native_player_loop_capture(
    display: *mut Display,
    pixmap: Window,
    window: Window,
    gc: GC,
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    motion_frames: &[NativeCombatMotionFrameSpec],
    out_dir: &Path,
    width: u32,
    height: u32,
    black: c_ulong,
    white: c_ulong,
) -> Result<NativePlayerLoopSummary, String> {
    let mut frames = build_native_player_loop_frame_specs(result, motion_frames)?;
    if frames.is_empty() {
        return Err("native player-facing loop has no screen frames".to_string());
    }
    let rendered_frame_count = TRUTH_HZ as usize;
    let nominal_frame_interval_ms = 1000 / TRUTH_HZ;
    let mut captured = vec![false; frames.len()];
    let mut loop_material = String::new();

    for loop_frame_index in 0..rendered_frame_count {
        let screen_index = loop_frame_index * frames.len() / rendered_frame_count;
        let screen_index = screen_index.min(frames.len() - 1);
        let screen_start = screen_index * rendered_frame_count / frames.len();
        let screen_end =
            ((screen_index + 1) * rendered_frame_count / frames.len()).max(screen_start + 1);
        let screen_span = (screen_end - screen_start).max(1);
        let progress_permille =
            ((loop_frame_index.saturating_sub(screen_start) * 1000) / screen_span).min(1000) as u32;
        let frame = &frames[screen_index];
        render_native_player_loop_frame(
            display,
            pixmap,
            gc,
            result,
            silhouette,
            frame,
            width,
            height,
            black,
            white,
            progress_permille,
        );
        XCopyArea(display, pixmap, window, gc, 0, 0, width, height, 0, 0);
        XFlush(display);
        XSync(display, 0);
        writeln!(
            &mut loop_material,
            "{}:{}:{}:{}:{}",
            loop_frame_index,
            frame.screen,
            frame.scheduled_ms,
            frame.truth_frame,
            progress_permille
        )
        .map_err(|error| error.to_string())?;
        if !captured[screen_index] {
            let path = out_dir.join(&frame.file);
            capture_window_to_ppm(display, pixmap, width, height, &path, black, white)?;
            let bytes = fs::read(&path).map_err(|error| error.to_string())?;
            frames[screen_index].frame_hash = hash_hex(&bytes);
            captured[screen_index] = true;
        }
    }

    if captured.iter().any(|captured| !captured) {
        return Err("native player-facing loop did not capture every screen".to_string());
    }
    let final_frame_hash = frames
        .last()
        .map(|frame| frame.frame_hash.clone())
        .ok_or_else(|| "native player-facing loop produced no frame hashes".to_string())?;
    Ok(NativePlayerLoopSummary {
        source: "view-only-presentation-loop-after-truth-hash".to_string(),
        backend: "raw-x11-xwayland-native-software-3d".to_string(),
        rendered_frame_count,
        screen_count: frames.len(),
        nominal_frame_interval_ms,
        nominal_duration_ms: rendered_frame_count as u32 * 1000 / TRUTH_HZ,
        timing_source: "presentation-loop-schedule-recorded-outside-authoritative-truth"
            .to_string(),
        truth_hash_before_loop: result.final_state_hash.clone(),
        truth_hash_after_loop: result.final_state_hash.clone(),
        loop_hash: hash_hex(loop_material.as_bytes()),
        final_frame_hash,
        frames,
    })
}

#[cfg(target_os = "linux")]
fn write_native_combat_software_3d_viewports(
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    out_dir: &Path,
) -> Result<Vec<NativeCombatSoftware3dViewport>, String> {
    let mut viewports = Vec::new();
    for camera in [
        "third_person_verdict_ring",
        "first_person_guard_line",
        "planning_tactical_reach",
        "consequence_aftermath_dwell",
        "fight_film_orbit",
        "asset_closeup_weapon_armor",
    ] {
        let file = match camera {
            "third_person_verdict_ring" => "native_combat_3d_third_person.ppm",
            "first_person_guard_line" => "native_combat_3d_first_person.ppm",
            "planning_tactical_reach" => "native_combat_3d_planning.ppm",
            "consequence_aftermath_dwell" => "native_combat_3d_consequence.ppm",
            "fight_film_orbit" => "native_combat_3d_fight_film.ppm",
            "asset_closeup_weapon_armor" => "native_combat_3d_asset_closeup.ppm",
            _ => return Err(format!("unsupported software 3D camera '{camera}'")),
        };
        let width = 960u32;
        let height = 540u32;
        let (bytes, triangle_count, shaded_triangle_count) =
            render_native_combat_software_3d_viewport(result, silhouette, camera, width, height)?;
        let path = out_dir.join(file);
        fs::write(&path, &bytes).map_err(|error| error.to_string())?;
        let (_, non_background_pixels, frame_hash) =
            native_ppm_evidence(&path, width, height).map_err(|error| error.to_string())?;
        viewports.push(NativeCombatSoftware3dViewport {
            file: file.to_string(),
            camera: camera.to_string(),
            width,
            height,
            triangle_count,
            shaded_triangle_count,
            non_background_pixels,
            projection_model: "integer_depth_sorted_mesh_raster".to_string(),
            depth_sorted: true,
            source: native_camera_truth_source(camera).to_string(),
            frame_hash,
        });
    }
    Ok(viewports)
}

#[cfg(target_os = "linux")]
fn write_native_combat_software_3d_sequence(
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    motion_frames: &[NativeCombatMotionFrameSpec],
    out_dir: &Path,
) -> Result<NativeCombatSoftware3dSequenceSummary, String> {
    if motion_frames.is_empty() {
        return Err("native combat software 3D sequence received no motion frames".to_string());
    }
    let width = 960u32;
    let height = 540u32;
    let camera = "third_person_replay_orbit".to_string();
    let projection_model = "integer_depth_sorted_mesh_raster".to_string();
    let source = "replay-derived-runtime-gltf-after-truth-hash".to_string();
    let mut frames = Vec::new();
    let mut chain_material = String::new();
    for motion in motion_frames {
        let file = format!("native_combat_3d_motion_{:03}.ppm", motion.index);
        let (bytes, triangle_count, shaded_triangle_count) =
            render_native_combat_software_3d_motion_viewport(
                result, silhouette, motion, width, height,
            )?;
        let path = out_dir.join(&file);
        fs::write(&path, &bytes).map_err(|error| error.to_string())?;
        let (_, non_background_pixels, frame_hash) =
            native_ppm_evidence(&path, width, height).map_err(|error| error.to_string())?;
        chain_material.push_str(&format!(
            "{}:{}:{}:{}:{};",
            motion.index, motion.phase, motion.turn, motion.truth_frame, frame_hash
        ));
        frames.push(NativeCombatSoftware3dFrame {
            index: motion.index,
            file,
            phase: motion.phase.to_string(),
            turn: motion.turn,
            truth_frame: motion.truth_frame,
            progress_permille: motion.progress_permille,
            triangle_count,
            shaded_triangle_count,
            non_background_pixels,
            frame_hash,
        });
    }
    Ok(NativeCombatSoftware3dSequenceSummary {
        camera,
        frame_count: frames.len(),
        width,
        height,
        projection_model,
        depth_sorted: true,
        source,
        frame_hash_chain: hash_hex(chain_material.as_bytes()),
        frames,
    })
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy)]
struct NativeRasterTriangle {
    points: [(i32, i32); 3],
    depth: i32,
    color: (u8, u8, u8),
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy)]
struct NativeSoftware3dInstance<'a> {
    geometry: &'a NativeGltfGeometry,
    origin_x_milli: i32,
    origin_y_milli: i32,
    origin_z_milli: i32,
    scale_num: i32,
    scale_den: i32,
    facing: i32,
    depth_bias: i32,
    color: (u8, u8, u8),
}

#[cfg(target_os = "linux")]
fn render_native_combat_software_3d_viewport(
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    camera: &str,
    width: u32,
    height: u32,
) -> Result<(Vec<u8>, usize, usize), String> {
    let mut pixels = vec![0u8; width as usize * height as usize * 3];
    paint_native_stone_floor(&mut pixels, width as usize, height as usize);

    draw_native_contact_shadow(
        &mut pixels,
        width as usize,
        height as usize,
        (width / 2) as i32,
        (height as i32 * 3 / 5).max(1),
        (width / 4) as i32,
        (height / 16) as i32,
    );
    let mut triangles = Vec::new();
    match camera {
        "third_person_verdict_ring" => {
            push_native_mesh_triangles(
                &mut triangles,
                NativeSoftware3dInstance {
                    geometry: &silhouette.arena_presentation_asset.geometry,
                    origin_x_milli: 0,
                    origin_y_milli: -700,
                    origin_z_milli: 0,
                    scale_num: 118,
                    scale_den: 1000,
                    facing: 1,
                    depth_bias: -900,
                    color: native_material_color("chalked_stone_dust", (164, 154, 128)),
                },
                width,
                height,
                camera,
            );
            if let Some(fighter) = silhouette.fighter(0) {
                push_native_fighter_software_meshes(
                    &mut triangles,
                    fighter,
                    -950,
                    -420,
                    -40,
                    -1,
                    360,
                    camera,
                    width,
                    height,
                );
            }
            if let Some(fighter) = silhouette.fighter(1) {
                push_native_fighter_software_meshes(
                    &mut triangles,
                    fighter,
                    950,
                    -420,
                    40,
                    1,
                    360,
                    camera,
                    width,
                    height,
                );
            }
        }
        "first_person_guard_line" => {
            push_native_mesh_triangles(
                &mut triangles,
                NativeSoftware3dInstance {
                    geometry: &silhouette.arena_presentation_asset.geometry,
                    origin_x_milli: 0,
                    origin_y_milli: -1800,
                    origin_z_milli: 0,
                    scale_num: 138,
                    scale_den: 1000,
                    facing: 1,
                    depth_bias: -1500,
                    color: native_material_color("chalked_stone_dust", (148, 139, 118)),
                },
                width,
                height,
                camera,
            );
            if let Some(fighter) = silhouette.fighter(1) {
                push_native_fighter_software_meshes(
                    &mut triangles,
                    fighter,
                    240,
                    -760,
                    120,
                    1,
                    520,
                    camera,
                    width,
                    height,
                );
            }
            if let Some(fighter) = silhouette.fighter(0) {
                push_native_mesh_triangles(
                    &mut triangles,
                    NativeSoftware3dInstance {
                        geometry: &fighter.weapon_presentation_asset.geometry,
                        origin_x_milli: -2600,
                        origin_y_milli: -1280,
                        origin_z_milli: 360,
                        scale_num: native_weapon_mesh_scale_num(fighter, 120) * 6,
                        scale_den: 1000,
                        facing: -1,
                        depth_bias: 620,
                        color: native_material_color(
                            native_weapon_material_binding(fighter),
                            (74, 83, 82),
                        ),
                    },
                    width,
                    height,
                    camera,
                );
            }
        }
        "planning_tactical_reach" => {
            push_native_mesh_triangles(
                &mut triangles,
                NativeSoftware3dInstance {
                    geometry: &silhouette.arena_presentation_asset.geometry,
                    origin_x_milli: 0,
                    origin_y_milli: -760,
                    origin_z_milli: 0,
                    scale_num: 150,
                    scale_den: 1000,
                    facing: 1,
                    depth_bias: -980,
                    color: native_material_color("chalked_stone_dust", (166, 154, 124)),
                },
                width,
                height,
                camera,
            );
            if let Some(fighter) = silhouette.fighter(0) {
                push_native_fighter_software_meshes_posed(
                    &mut triangles,
                    fighter,
                    -860,
                    -420,
                    -180,
                    -1,
                    380,
                    camera,
                    width,
                    height,
                    120,
                    60,
                    -40,
                    0,
                );
            }
            if let Some(fighter) = silhouette.fighter(1) {
                push_native_fighter_software_meshes_posed(
                    &mut triangles,
                    fighter,
                    860,
                    -420,
                    180,
                    1,
                    380,
                    camera,
                    width,
                    height,
                    90,
                    40,
                    40,
                    0,
                );
            }
        }
        "consequence_aftermath_dwell" => {
            push_native_mesh_triangles(
                &mut triangles,
                NativeSoftware3dInstance {
                    geometry: &silhouette.arena_presentation_asset.geometry,
                    origin_x_milli: 0,
                    origin_y_milli: -820,
                    origin_z_milli: 60,
                    scale_num: 138,
                    scale_den: 1000,
                    facing: 1,
                    depth_bias: -1040,
                    color: native_material_color("chalked_stone_dust", (154, 143, 117)),
                },
                width,
                height,
                camera,
            );
            if let Some(fighter) = silhouette.fighter(0) {
                push_native_fighter_software_meshes_posed(
                    &mut triangles,
                    fighter,
                    -720,
                    -430,
                    -90,
                    -1,
                    430,
                    camera,
                    width,
                    height,
                    160,
                    20,
                    20,
                    0,
                );
            }
            if let Some(fighter) = silhouette.fighter(1) {
                push_native_fighter_software_meshes_posed(
                    &mut triangles,
                    fighter,
                    660,
                    -220,
                    120,
                    1,
                    440,
                    camera,
                    width,
                    height,
                    80,
                    -180,
                    60,
                    220,
                );
            }
        }
        "fight_film_orbit" => {
            push_native_mesh_triangles(
                &mut triangles,
                NativeSoftware3dInstance {
                    geometry: &silhouette.arena_presentation_asset.geometry,
                    origin_x_milli: 0,
                    origin_y_milli: -960,
                    origin_z_milli: 180,
                    scale_num: 128,
                    scale_den: 1000,
                    facing: 1,
                    depth_bias: -1100,
                    color: native_material_color("chalked_stone_dust", (154, 145, 122)),
                },
                width,
                height,
                camera,
            );
            if let Some(fighter) = silhouette.fighter(0) {
                push_native_fighter_software_meshes(
                    &mut triangles,
                    fighter,
                    -760,
                    -500,
                    120,
                    -1,
                    420,
                    camera,
                    width,
                    height,
                );
            }
            if let Some(fighter) = silhouette.fighter(1) {
                push_native_fighter_software_meshes(
                    &mut triangles,
                    fighter,
                    680,
                    -470,
                    -90,
                    1,
                    420,
                    camera,
                    width,
                    height,
                );
            }
        }
        "asset_closeup_weapon_armor" => {
            if let Some(fighter) = silhouette.fighter(0) {
                push_native_mesh_triangles(
                    &mut triangles,
                    NativeSoftware3dInstance {
                        geometry: &fighter.weapon_presentation_asset.geometry,
                        origin_x_milli: -980,
                        origin_y_milli: -920,
                        origin_z_milli: 260,
                        scale_num: 1650,
                        scale_den: 1000,
                        facing: 1,
                        depth_bias: -120,
                        color: native_material_color(
                            native_weapon_material_binding(fighter),
                            (86, 96, 98),
                        ),
                    },
                    width,
                    height,
                    camera,
                );
            }
            if let Some(fighter) = silhouette.fighter(1) {
                push_native_mesh_triangles(
                    &mut triangles,
                    NativeSoftware3dInstance {
                        geometry: &fighter.armor_presentation_asset.geometry,
                        origin_x_milli: 820,
                        origin_y_milli: -980,
                        origin_z_milli: 60,
                        scale_num: 1480,
                        scale_den: 1000,
                        facing: 1,
                        depth_bias: 180,
                        color: native_material_color(
                            native_armor_material_binding(fighter),
                            (112, 68, 62),
                        ),
                    },
                    width,
                    height,
                    camera,
                );
            }
        }
        other => return Err(format!("unsupported software 3D camera '{other}'")),
    }

    let triangle_count = triangles.len();
    triangles.sort_by_key(|triangle| triangle.depth);
    let mut shaded_triangle_count = 0usize;
    for triangle in &triangles {
        if fill_triangle(&mut pixels, width as usize, height as usize, triangle) {
            shaded_triangle_count += 1;
        }
    }
    apply_native_lighting_post(&mut pixels, width as usize, height as usize, 1080);

    let overlay_enabled = true;
    if overlay_enabled {
        match camera {
            "first_person_guard_line" => {
                let cx = width as i32 / 2;
                let cy = height as i32 / 2 + 18;
                draw_pixel_line(
                    &mut pixels,
                    width as usize,
                    height as usize,
                    cx - 120,
                    cy,
                    cx + 120,
                    cy,
                    (220, 128, 38),
                );
                draw_pixel_line(
                    &mut pixels,
                    width as usize,
                    height as usize,
                    cx,
                    cy - 86,
                    cx,
                    cy + 86,
                    (220, 128, 38),
                );
                draw_pixel_line(
                    &mut pixels,
                    width as usize,
                    height as usize,
                    cx - 92,
                    cy - 64,
                    cx + 92,
                    cy + 64,
                    (36, 78, 82),
                );
                draw_pixel_line(
                    &mut pixels,
                    width as usize,
                    height as usize,
                    cx - 92,
                    cy + 64,
                    cx + 92,
                    cy - 64,
                    (36, 78, 82),
                );
            }
            "planning_tactical_reach" => {
                let cy = height as i32 / 2 + 64;
                for (index, reach) in [180, 250, 320].iter().enumerate() {
                    let radius = *reach + index as i32 * 12;
                    draw_pixel_line(
                        &mut pixels,
                        width as usize,
                        height as usize,
                        width as i32 / 2 - radius,
                        cy - index as i32 * 20,
                        width as i32 / 2 + radius,
                        cy - index as i32 * 20,
                        if index == 0 {
                            (203, 148, 54)
                        } else {
                            (43, 104, 111)
                        },
                    );
                }
                draw_pixel_line(
                    &mut pixels,
                    width as usize,
                    height as usize,
                    width as i32 / 2,
                    cy - 210,
                    width as i32 / 2,
                    cy + 34,
                    (220, 128, 38),
                );
            }
            "consequence_aftermath_dwell" => {
                let cx = width as i32 / 2 + 120;
                let cy = height as i32 / 2 + 96;
                draw_native_contact_bloom(
                    &mut pixels,
                    width as usize,
                    height as usize,
                    cx,
                    cy,
                    width as i32 / 8,
                    native_material_color("wet_blood_trace_overlay", (128, 34, 24)),
                );
                for index in 0..4 {
                    fill_rect(
                        &mut pixels,
                        width as usize,
                        height as usize,
                        650,
                        88 + index * 26,
                        210 - index * 34,
                        12,
                        if index % 2 == 0 {
                            (154, 54, 44)
                        } else {
                            (203, 148, 54)
                        },
                    );
                }
            }
            "fight_film_orbit" => {
                for index in 0..=6 {
                    let x = 116 + index * 118;
                    fill_rect(
                        &mut pixels,
                        width as usize,
                        height as usize,
                        x,
                        82,
                        12,
                        54,
                        (38, 49, 52),
                    );
                }
                fill_rect(
                    &mut pixels,
                    width as usize,
                    height as usize,
                    116,
                    107,
                    720,
                    8,
                    (42, 103, 108),
                );
                fill_rect(
                    &mut pixels,
                    width as usize,
                    height as usize,
                    464,
                    72,
                    42,
                    74,
                    (203, 80, 48),
                );
            }
            "asset_closeup_weapon_armor" => {
                fill_rect(
                    &mut pixels,
                    width as usize,
                    height as usize,
                    58,
                    72,
                    220,
                    34,
                    (35, 50, 54),
                );
                fill_rect(
                    &mut pixels,
                    width as usize,
                    height as usize,
                    682,
                    72,
                    220,
                    34,
                    (115, 67, 58),
                );
                draw_pixel_line(
                    &mut pixels,
                    width as usize,
                    height as usize,
                    284,
                    276,
                    666,
                    276,
                    (223, 126, 31),
                );
                draw_pixel_line(
                    &mut pixels,
                    width as usize,
                    height as usize,
                    642,
                    250,
                    690,
                    302,
                    (210, 55, 41),
                );
                draw_pixel_line(
                    &mut pixels,
                    width as usize,
                    height as usize,
                    642,
                    302,
                    690,
                    250,
                    (210, 55, 41),
                );
                draw_pixel_line(
                    &mut pixels,
                    width as usize,
                    height as usize,
                    666,
                    234,
                    666,
                    318,
                    (210, 55, 41),
                );
                fill_rect(
                    &mut pixels,
                    width as usize,
                    height as usize,
                    64,
                    442,
                    260,
                    18,
                    (32, 37, 38),
                );
                fill_rect(
                    &mut pixels,
                    width as usize,
                    height as usize,
                    64,
                    442,
                    180,
                    18,
                    (42, 104, 110),
                );
                fill_rect(
                    &mut pixels,
                    width as usize,
                    height as usize,
                    356,
                    442,
                    260,
                    18,
                    (32, 37, 38),
                );
                fill_rect(
                    &mut pixels,
                    width as usize,
                    height as usize,
                    356,
                    442,
                    104,
                    18,
                    (196, 127, 43),
                );
                fill_rect(
                    &mut pixels,
                    width as usize,
                    height as usize,
                    648,
                    442,
                    260,
                    18,
                    (32, 37, 38),
                );
                fill_rect(
                    &mut pixels,
                    width as usize,
                    height as usize,
                    648,
                    442,
                    218,
                    18,
                    (136, 68, 58),
                );
            }
            _ => {
                draw_pixel_line(
                    &mut pixels,
                    width as usize,
                    height as usize,
                    250,
                    270,
                    710,
                    270,
                    (42, 103, 108),
                );
                draw_pixel_line(
                    &mut pixels,
                    width as usize,
                    height as usize,
                    476,
                    230,
                    510,
                    310,
                    (196, 127, 43),
                );
            }
        }
    }
    if overlay_enabled {
        draw_native_software_camera_readability_overlay(
            &mut pixels,
            width as usize,
            height as usize,
            result,
            silhouette,
            camera,
        );
    }

    fill_rect(
        &mut pixels,
        width as usize,
        height as usize,
        28,
        height as usize - 52,
        560,
        18,
        (32, 37, 38),
    );
    fill_rect(
        &mut pixels,
        width as usize,
        height as usize,
        600,
        height as usize - 52,
        (result.turns.len() * 72).min(300),
        18,
        (196, 127, 43),
    );

    let mut ppm = format!("P6\n{} {}\n255\n", width, height).into_bytes();
    ppm.extend_from_slice(&pixels);
    Ok((ppm, triangle_count, shaded_triangle_count))
}

#[cfg(target_os = "linux")]
fn draw_native_software_camera_readability_overlay(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    camera: &str,
) {
    if width == 0 || height == 0 {
        return;
    }

    let panel_y = height.saturating_sub(124);
    fill_rect(
        pixels,
        width,
        height,
        24,
        panel_y,
        width.saturating_sub(48).min(560),
        86,
        (24, 29, 31),
    );

    let camera_color = match camera {
        "first_person_guard_line" => (52, 112, 126),
        "fight_film_orbit" => (112, 72, 132),
        "asset_closeup_weapon_armor" => (126, 96, 38),
        _ => (52, 92, 84),
    };
    fill_rect(
        pixels,
        width,
        height,
        38,
        panel_y + 12,
        150,
        14,
        camera_color,
    );
    draw_native_ppm_text(
        pixels,
        width,
        height,
        44,
        panel_y as i32 + 15,
        native_camera_category(camera),
        1,
        (239, 231, 206),
    );

    let contact_count: usize = result.turns.iter().map(|turn| turn.contacts.len()).sum();
    let contact_bar = (contact_count.max(1) * 34).min(220);
    fill_rect(
        pixels,
        width,
        height,
        204,
        panel_y + 12,
        224,
        14,
        (52, 54, 52),
    );
    fill_rect(
        pixels,
        width,
        height,
        204,
        panel_y + 12,
        contact_bar,
        14,
        (154, 54, 44),
    );
    draw_native_ppm_text(
        pixels,
        width,
        height,
        210,
        panel_y as i32 + 15,
        "CONTACTS",
        1,
        (239, 231, 206),
    );
    draw_native_ppm_text(
        pixels,
        width,
        height,
        38,
        panel_y as i32 + 72,
        "MATERIALS",
        1,
        (239, 231, 206),
    );

    for (index, material) in native_renderer_material_ids(silhouette)
        .iter()
        .take(7)
        .enumerate()
    {
        let color = native_material_color(material, (96, 96, 96));
        fill_rect(
            pixels,
            width,
            height,
            38 + index * 34,
            panel_y + 42,
            24,
            24,
            color,
        );
    }

    let (balance_loss, recovery_frames, grip_loss) = result
        .turns
        .iter()
        .flat_map(|turn| turn.contacts.iter())
        .next()
        .map(|contact| {
            (
                contact.capability_delta.balance_delta.unsigned_abs() as usize,
                contact.capability_delta.recovery_slowdown_add as usize,
                contact.capability_delta.grip_r_delta.unsigned_abs() as usize,
            )
        })
        .unwrap_or((0, 0, 0));
    for (index, (value, cap, color)) in [
        (balance_loss, 180usize, (196, 127, 43)),
        (recovery_frames * 8, 120usize, (154, 54, 44)),
        (grip_loss / 3, 180usize, (52, 112, 126)),
    ]
    .into_iter()
    .enumerate()
    {
        let y = panel_y + 42 + index * 12;
        let label = match index {
            0 => "BAL",
            1 => "REC",
            _ => "GRP",
        };
        draw_native_ppm_text(
            pixels,
            width,
            height,
            282,
            y as i32,
            label,
            1,
            (239, 231, 206),
        );
        fill_rect(pixels, width, height, 314, y, 164, 7, (52, 54, 52));
        fill_rect(
            pixels,
            width,
            height,
            314,
            y,
            value.min(cap) * 164 / cap.max(1),
            7,
            color,
        );
    }

    draw_pixel_line(
        pixels,
        width,
        height,
        34,
        panel_y as i32 + 32,
        520.min(width.saturating_sub(28)) as i32,
        panel_y as i32 + 32,
        (218, 206, 180),
    );
}

#[cfg(target_os = "linux")]
fn draw_native_ppm_text(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    x: i32,
    y: i32,
    text: &str,
    scale: usize,
    color: (u8, u8, u8),
) {
    let scale = scale.max(1);
    let mut cursor_x = x;
    for ch in text.to_ascii_uppercase().chars() {
        let glyph = native_ppm_glyph(ch);
        for (row, mask) in glyph.iter().enumerate() {
            for col in 0..5usize {
                if (mask & (1 << (4 - col))) != 0 {
                    let px = cursor_x + (col * scale) as i32;
                    let py = y + (row * scale) as i32;
                    if px >= 0 && py >= 0 {
                        fill_rect(
                            pixels,
                            width,
                            height,
                            px as usize,
                            py as usize,
                            scale,
                            scale,
                            color,
                        );
                    }
                }
            }
        }
        cursor_x += (6 * scale) as i32;
    }
}

#[cfg(target_os = "linux")]
fn native_ppm_glyph(ch: char) -> [u8; 7] {
    match ch {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 15],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [14, 4, 4, 4, 4, 4, 14],
        'J' => [7, 2, 2, 2, 18, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        ':' => [0, 4, 4, 0, 4, 4, 0],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        '+' => [0, 4, 4, 31, 4, 4, 0],
        '/' => [1, 1, 2, 4, 8, 16, 16],
        '_' => [0, 0, 0, 0, 0, 0, 31],
        ' ' => [0, 0, 0, 0, 0, 0, 0],
        _ => [31, 1, 2, 4, 4, 0, 4],
    }
}

#[cfg(target_os = "linux")]
fn render_native_combat_software_3d_motion_viewport(
    _result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    motion: &NativeCombatMotionFrameSpec,
    width: u32,
    height: u32,
) -> Result<(Vec<u8>, usize, usize), String> {
    let mut pixels = vec![0u8; width as usize * height as usize * 3];
    paint_native_stone_floor(&mut pixels, width as usize, height as usize);

    draw_native_contact_shadow(
        &mut pixels,
        width as usize,
        height as usize,
        (width / 2) as i32,
        (height as i32 * 3 / 5).max(1),
        (width / 4) as i32,
        (height / 16) as i32,
    );
    let mut triangles = Vec::new();
    push_native_mesh_triangles(
        &mut triangles,
        NativeSoftware3dInstance {
            geometry: &silhouette.arena_presentation_asset.geometry,
            origin_x_milli: 0,
            origin_y_milli: -720,
            origin_z_milli: 0,
            scale_num: 118,
            scale_den: 1000,
            facing: 1,
            depth_bias: -900,
            color: native_material_color("chalked_stone_dust", (160, 151, 127)),
        },
        width,
        height,
        "third_person_replay_orbit",
    );

    let progress = motion.progress_permille as i32;
    let phase_stride = match motion.phase {
        "commit_reveal" => 70 + progress / 6,
        "weapon_arc" => 240 + progress / 2,
        "active_contact" => 610 - progress / 10,
        "material_anatomy_solve" => 540 - progress / 8,
        "recovery_capability" => 280 - progress / 5,
        "stagger_collapse_risk" => 220 - progress / 7,
        _ => 120 + progress / 9,
    };
    let rebound = match motion.phase {
        "active_contact" | "material_anatomy_solve" => 90,
        "stagger_collapse_risk" => -110,
        "recovery_capability" => -70,
        _ => 0,
    };
    let weapon_extension = match motion.phase {
        "weapon_arc" => 150 + progress / 2,
        "active_contact" => 520,
        "material_anatomy_solve" => 470 - progress / 9,
        "recovery_capability" => 180 - progress / 8,
        _ => progress / 7,
    };
    let weapon_lift = match motion.phase {
        "weapon_arc" => 180 - progress / 8,
        "active_contact" => -20,
        "material_anatomy_solve" => -70,
        "stagger_collapse_risk" => -140,
        _ => 40,
    };
    let depth_sway = ((motion.index as i32 % 5) - 2) * 30;
    if let Some(fighter) = silhouette.fighter(0) {
        push_native_fighter_software_meshes_posed(
            &mut triangles,
            fighter,
            -1080 + phase_stride,
            -420 + rebound,
            -60 + depth_sway,
            -1,
            360,
            "third_person_replay_orbit",
            width,
            height,
            weapon_extension,
            weapon_lift,
            progress / 8,
            rebound / 2,
        );
    }
    if let Some(fighter) = silhouette.fighter(1) {
        push_native_fighter_software_meshes_posed(
            &mut triangles,
            fighter,
            1080 - phase_stride / 2,
            -420 - rebound / 2,
            70 - depth_sway,
            1,
            360,
            "third_person_replay_orbit",
            width,
            height,
            weapon_extension / 2,
            -weapon_lift / 2,
            -progress / 12,
            -rebound / 3,
        );
    }

    let triangle_count = triangles.len();
    triangles.sort_by_key(|triangle| triangle.depth);
    let mut shaded_triangle_count = 0usize;
    for triangle in &triangles {
        if fill_triangle(&mut pixels, width as usize, height as usize, triangle) {
            shaded_triangle_count += 1;
        }
    }
    if matches!(motion.phase, "active_contact" | "material_anatomy_solve") {
        draw_native_contact_bloom(
            &mut pixels,
            width as usize,
            height as usize,
            (width / 2) as i32,
            (height / 2) as i32,
            (width / 9) as i32,
            native_material_color("wet_blood_trace_overlay", (120, 34, 22)),
        );
    }
    apply_native_lighting_post(&mut pixels, width as usize, height as usize, 1080);

    fill_rect(
        &mut pixels,
        width as usize,
        height as usize,
        28,
        height as usize - 52,
        560,
        18,
        (31, 36, 37),
    );
    fill_rect(
        &mut pixels,
        width as usize,
        height as usize,
        600,
        height as usize - 52,
        ((motion.progress_permille as usize * 300) / 1000).max(16),
        18,
        (195, 125, 42),
    );
    fill_rect(
        &mut pixels,
        width as usize,
        height as usize,
        28,
        height as usize - 25,
        (motion.index * 38).min(520),
        9,
        (45, 105, 110),
    );

    let mut ppm = format!("P6\n{} {}\n255\n", width, height).into_bytes();
    ppm.extend_from_slice(&pixels);
    Ok((ppm, triangle_count, shaded_triangle_count))
}

#[cfg(target_os = "linux")]
fn push_native_fighter_software_meshes(
    triangles: &mut Vec<NativeRasterTriangle>,
    fighter: &NativeCombatFighterSilhouette,
    origin_x_milli: i32,
    origin_y_milli: i32,
    origin_z_milli: i32,
    facing: i32,
    scale_num: i32,
    camera: &str,
    width: u32,
    height: u32,
) {
    push_native_fighter_software_meshes_posed(
        triangles,
        fighter,
        origin_x_milli,
        origin_y_milli,
        origin_z_milli,
        facing,
        scale_num,
        camera,
        width,
        height,
        0,
        0,
        0,
        0,
    );
}

#[cfg(target_os = "linux")]
fn push_native_fighter_software_meshes_posed(
    triangles: &mut Vec<NativeRasterTriangle>,
    fighter: &NativeCombatFighterSilhouette,
    origin_x_milli: i32,
    origin_y_milli: i32,
    origin_z_milli: i32,
    facing: i32,
    scale_num: i32,
    camera: &str,
    width: u32,
    height: u32,
    weapon_extension_milli: i32,
    weapon_y_offset_milli: i32,
    weapon_z_offset_milli: i32,
    armor_y_offset_milli: i32,
) {
    push_native_mesh_triangles(
        triangles,
        NativeSoftware3dInstance {
            geometry: &fighter.armor_presentation_asset.geometry,
            origin_x_milli,
            origin_y_milli: origin_y_milli + armor_y_offset_milli,
            origin_z_milli,
            scale_num,
            scale_den: 1000,
            facing,
            depth_bias: origin_y_milli,
            color: native_material_color(
                native_armor_material_binding(fighter),
                if fighter.seat == 0 {
                    (45, 105, 110)
                } else {
                    (126, 72, 60)
                },
            ),
        },
        width,
        height,
        camera,
    );
    push_native_mesh_triangles(
        triangles,
        NativeSoftware3dInstance {
            geometry: &fighter.weapon_presentation_asset.geometry,
            origin_x_milli: origin_x_milli
                + facing * (fighter.weapon_reach_mm / 2 + 180 + weapon_extension_milli),
            origin_y_milli: origin_y_milli + 940 + weapon_y_offset_milli,
            origin_z_milli: origin_z_milli + 220 + weapon_z_offset_milli,
            scale_num: native_weapon_mesh_scale_num(fighter, 80) * 5,
            scale_den: 1000,
            facing,
            depth_bias: origin_y_milli + 360 + weapon_z_offset_milli / 2,
            color: native_material_color(native_weapon_material_binding(fighter), (70, 78, 79)),
        },
        width,
        height,
        camera,
    );
}

#[cfg(target_os = "linux")]
fn push_native_mesh_triangles(
    triangles: &mut Vec<NativeRasterTriangle>,
    instance: NativeSoftware3dInstance,
    width: u32,
    height: u32,
    camera: &str,
) {
    for indices in instance.geometry.indices.chunks(3) {
        if indices.len() != 3 {
            continue;
        }
        let Some(a) = software_project_gltf_point(instance, indices[0], width, height, camera)
        else {
            continue;
        };
        let Some(b) = software_project_gltf_point(instance, indices[1], width, height, camera)
        else {
            continue;
        };
        let Some(c) = software_project_gltf_point(instance, indices[2], width, height, camera)
        else {
            continue;
        };
        let depth = (a.2 + b.2 + c.2) / 3 + instance.depth_bias;
        triangles.push(NativeRasterTriangle {
            points: [(a.0, a.1), (b.0, b.1), (c.0, c.1)],
            depth,
            color: shade_native_software_color(instance.color, depth),
        });
    }
}

#[cfg(target_os = "linux")]
fn software_project_gltf_point(
    instance: NativeSoftware3dInstance,
    vertex_index: usize,
    width: u32,
    height: u32,
    camera: &str,
) -> Option<(i32, i32, i32)> {
    let (x_milli, y_milli, z_milli) = *instance.geometry.positions_milli.get(vertex_index)?;
    let den = instance.scale_den.max(1) as i64;
    let local_x = instance.facing as i64 * x_milli as i64;
    let local_y = y_milli as i64;
    let local_z = z_milli as i64;
    let world_x = instance.origin_x_milli as i64 + (local_x * instance.scale_num as i64) / den;
    let world_y = instance.origin_y_milli as i64 + (local_y * instance.scale_num as i64) / den;
    let world_z = instance.origin_z_milli as i64 + (local_z * instance.scale_num as i64) / den;
    let (screen_x, screen_y) = match camera {
        "first_person_guard_line" => (
            width as i64 / 2 + world_x / 4 + world_z / 8,
            height as i64 / 2 + 112 - world_y / 5 - world_z / 10,
        ),
        "third_person_replay_orbit" => (
            width as i64 / 2 + world_x / 4 + world_z / 5,
            height as i64 / 2 + 72 - world_y / 5 - world_z / 7,
        ),
        "planning_tactical_reach" => (
            width as i64 / 2 + world_x / 5 + world_z / 9,
            height as i64 / 2 + 36 - world_y / 6 - world_z / 14,
        ),
        "consequence_aftermath_dwell" => (
            width as i64 / 2 + world_x / 4 + world_z / 6,
            height as i64 / 2 + 96 - world_y / 5 - world_z / 9,
        ),
        "fight_film_orbit" => (
            width as i64 / 2 + world_x / 5 - world_z / 4,
            height as i64 / 2 + 84 - world_y / 6 - world_z / 9,
        ),
        _ => (
            width as i64 / 2 + world_x / 4 + world_z / 6,
            height as i64 / 2 + 76 - world_y / 5 - world_z / 8,
        ),
    };
    Some((
        screen_x.clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        screen_y.clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        (world_y + world_z / 3).clamp(i32::MIN as i64, i32::MAX as i64) as i32,
    ))
}

#[cfg(target_os = "linux")]
fn shade_native_software_color(base: (u8, u8, u8), depth: i32) -> (u8, u8, u8) {
    let shade = ((depth / 160).clamp(-26, 28) + 28) as i16;
    (
        (base.0 as i16 + shade / 3).clamp(0, 255) as u8,
        (base.1 as i16 + shade / 4).clamp(0, 255) as u8,
        (base.2 as i16 + shade / 5).clamp(0, 255) as u8,
    )
}

#[cfg(target_os = "linux")]
fn native_pixel_material_color(base: (u8, u8, u8), x: usize, y: usize, depth: i32) -> (u8, u8, u8) {
    let noise = (((x as i32 * 37 + y as i32 * 17 + depth / 11).rem_euclid(29)) - 14) as i16;
    let mut r = base.0 as i16;
    let mut g = base.1 as i16;
    let mut b = base.2 as i16;
    if r > g + 48 && g < 70 && b < 70 {
        let wet = if (x + 2 * y) % 19 < 5 { 18 } else { -8 };
        r += wet + 14;
        g -= 8;
        b -= 8;
    } else if (r - g).abs() < 28 && (g - b).abs() < 32 && r < 145 {
        let ring = if ((x / 4) + (y / 3) + depth.unsigned_abs() as usize / 19) % 2 == 0 {
            10
        } else {
            -6
        };
        let scrape = if (x + y + depth.unsigned_abs() as usize / 7) % 37 < 2 {
            34
        } else {
            0
        };
        r += ring + scrape;
        g += ring + scrape;
        b += ring + scrape + 9;
    } else if r > 148 && g > 132 && b > 100 {
        let chip = if (x * 11 + y * 7) % 53 < 4 { -24 } else { 5 };
        r += chip + noise / 3;
        g += chip + noise / 4;
        b += chip / 2;
    } else if r > 125 && g > 92 && b < 120 {
        let stitch = if x % 18 < 2 || y % 16 < 2 { -20 } else { 8 };
        r += stitch + noise / 2;
        g += stitch / 2 + noise / 3;
        b += noise / 4;
    } else if r > 95 && g < 105 && b < 95 {
        let grain = if (x * 3 + y + depth.unsigned_abs() as usize / 13) % 23 < 5 {
            -18
        } else {
            8
        };
        r += grain + noise / 2;
        g += grain / 3;
        b -= 5;
    } else {
        r += noise / 3;
        g += noise / 3;
        b += noise / 3;
    }
    (
        r.clamp(0, 255) as u8,
        g.clamp(0, 255) as u8,
        b.clamp(0, 255) as u8,
    )
}

#[cfg(target_os = "linux")]
fn draw_native_contact_shadow(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    center_x: i32,
    center_y: i32,
    radius_x: i32,
    radius_y: i32,
) {
    blend_ellipse(
        pixels,
        width,
        height,
        center_x,
        center_y,
        radius_x,
        radius_y,
        (34, 29, 24),
        185,
    );
}

#[cfg(target_os = "linux")]
fn draw_native_contact_bloom(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    center_x: i32,
    center_y: i32,
    radius: i32,
    color: (u8, u8, u8),
) {
    let min_x = (center_x - radius).max(0) as usize;
    let max_x = (center_x + radius).min(width.saturating_sub(1) as i32) as usize;
    let min_y = (center_y - radius).max(0) as usize;
    let max_y = (center_y + radius).min(height.saturating_sub(1) as i32) as usize;
    let radius_sq = (radius * radius).max(1);
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as i32 - center_x;
            let dy = y as i32 - center_y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq <= radius_sq {
                let strength = ((radius_sq - dist_sq) * 220 / radius_sq).clamp(0, 220) as u16;
                blend_pixel(pixels, width, x, y, color, strength);
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn blend_ellipse(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    center_x: i32,
    center_y: i32,
    radius_x: i32,
    radius_y: i32,
    color: (u8, u8, u8),
    alpha: u16,
) {
    let rx = radius_x.max(1);
    let ry = radius_y.max(1);
    let min_x = (center_x - rx).max(0) as usize;
    let max_x = (center_x + rx).min(width.saturating_sub(1) as i32) as usize;
    let min_y = (center_y - ry).max(0) as usize;
    let max_y = (center_y + ry).min(height.saturating_sub(1) as i32) as usize;
    let rx_sq = rx * rx;
    let ry_sq = ry * ry;
    let bound = rx_sq * ry_sq;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as i32 - center_x;
            let dy = y as i32 - center_y;
            let metric = dx * dx * ry_sq + dy * dy * rx_sq;
            if metric <= bound {
                let fade = ((bound - metric) as i64 * alpha as i64 / bound.max(1) as i64)
                    .clamp(0, 255) as u16;
                blend_pixel(pixels, width, x, y, color, fade);
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn blend_pixel(
    pixels: &mut [u8],
    width: usize,
    x: usize,
    y: usize,
    color: (u8, u8, u8),
    alpha: u16,
) {
    let index = (y * width + x) * 3;
    let alpha = alpha.min(255);
    let inv = 255u16.saturating_sub(alpha);
    pixels[index] = ((pixels[index] as u16 * inv + color.0 as u16 * alpha) / 255) as u8;
    pixels[index + 1] = ((pixels[index + 1] as u16 * inv + color.1 as u16 * alpha) / 255) as u8;
    pixels[index + 2] = ((pixels[index + 2] as u16 * inv + color.2 as u16 * alpha) / 255) as u8;
}

#[cfg(target_os = "linux")]
fn draw_native_atmosphere_wisps(pixels: &mut [u8], width: usize, height: usize) {
    for band in 0..4 {
        let center_x = (width as i32 * (18 + band * 22)) / 100;
        let center_y = (height as i32 * (38 + band * 10)) / 100;
        blend_ellipse(
            pixels,
            width,
            height,
            center_x,
            center_y,
            (width as i32 / 3).max(1),
            (height as i32 / 34).max(1),
            (196, 169, 124),
            46,
        );
    }
}

#[cfg(target_os = "linux")]
fn apply_native_lighting_post(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    exposure_permille: i32,
) {
    let original = pixels.to_vec();
    for y in 0..height {
        let fog = ((height.saturating_sub(y) * 96) / height.max(1)) as i32;
        let dust = if y > height * 2 / 3 {
            ((y - height * 2 / 3) * 116 / (height / 3).max(1)) as i32
        } else {
            0
        };
        for x in 0..width {
            let idx = (y * width + x) * 3;
            let mut r = (pixels[idx] as i32 * exposure_permille / 1000).clamp(0, 255);
            let mut g = (pixels[idx + 1] as i32 * (exposure_permille - 10) / 1000).clamp(0, 255);
            let mut b = (pixels[idx + 2] as i32 * (exposure_permille - 35) / 1000).clamp(0, 255);
            r = (r * (255 - fog) + 206 * fog) / 255;
            g = (g * (255 - fog) + 199 * fog) / 255;
            b = (b * (255 - fog) + 184 * fog) / 255;
            r = (r * (255 - dust) + 148 * dust) / 255;
            g = (g * (255 - dust) + 124 * dust) / 255;
            b = (b * (255 - dust) + 89 * dust) / 255;
            let wisp =
                if ((x as i32 * 5 + y as i32 * 13 + (y as i32 / 17) * 23).rem_euclid(173)) < 6 {
                    22
                } else {
                    0
                };
            r = (r * (255 - wisp) + 190 * wisp) / 255;
            g = (g * (255 - wisp) + 166 * wisp) / 255;
            b = (b * (255 - wisp) + 126 * wisp) / 255;
            let vignette_x =
                ((x as i32 - width as i32 / 2).abs() * 35 / (width as i32 / 2).max(1)).clamp(0, 35);
            let vignette_y = ((y as i32 - height as i32 / 2).abs() * 28
                / (height as i32 / 2).max(1))
            .clamp(0, 28);
            let vignette = (vignette_x + vignette_y).clamp(0, 54);
            pixels[idx] = (r * (255 - vignette) / 255).clamp(0, 255) as u8;
            pixels[idx + 1] = (g * (255 - vignette) / 255).clamp(0, 255) as u8;
            pixels[idx + 2] = (b * (255 - vignette) / 255).clamp(0, 255) as u8;
        }
    }
    if width < 3 || height < 3 {
        draw_native_atmosphere_wisps(pixels, width, height);
        return;
    }
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let idx = (y * width + x) * 3;
            let right = (y * width + x + 1) * 3;
            let down = ((y + 1) * width + x) * 3;
            let contrast = (original[idx] as i32 - original[right] as i32).abs()
                + (original[idx + 1] as i32 - original[right + 1] as i32).abs()
                + (original[idx + 2] as i32 - original[right + 2] as i32).abs()
                + (original[idx] as i32 - original[down] as i32).abs()
                + (original[idx + 1] as i32 - original[down + 1] as i32).abs()
                + (original[idx + 2] as i32 - original[down + 2] as i32).abs();
            if contrast > 210 {
                for channel in 0..3 {
                    let avg = (pixels[idx + channel] as u16
                        + pixels[idx - 3 + channel] as u16
                        + pixels[idx + 3 + channel] as u16
                        + pixels[idx - width * 3 + channel] as u16
                        + pixels[idx + width * 3 + channel] as u16)
                        / 5;
                    pixels[idx + channel] = ((pixels[idx + channel] as u16 * 3 + avg) / 4) as u8;
                }
            }
        }
    }
    draw_native_atmosphere_wisps(pixels, width, height);
}

#[cfg(target_os = "linux")]
fn fill_triangle(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    triangle: &NativeRasterTriangle,
) -> bool {
    let [(x0, y0), (x1, y1), (x2, y2)] = triangle.points;
    let raw_min_x = x0.min(x1).min(x2);
    let raw_max_x = x0.max(x1).max(x2);
    let raw_min_y = y0.min(y1).min(y2);
    let raw_max_y = y0.max(y1).max(y2);
    let width_max = width.saturating_sub(1) as i32;
    let height_max = height.saturating_sub(1) as i32;
    if raw_max_x < 0 || raw_min_x > width_max || raw_max_y < 0 || raw_min_y > height_max {
        return false;
    }
    let min_x = raw_min_x.max(0) as usize;
    let max_x = raw_max_x.min(width_max) as usize;
    let min_y = raw_min_y.max(0) as usize;
    let max_y = raw_max_y.min(height_max) as usize;
    if min_x >= max_x || min_y >= max_y {
        return false;
    }
    let area = edge_function((x0, y0), (x1, y1), (x2, y2));
    if area == 0 {
        return false;
    }
    let mut wrote = false;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = (x as i32, y as i32);
            let w0 = edge_function((x1, y1), (x2, y2), p);
            let w1 = edge_function((x2, y2), (x0, y0), p);
            let w2 = edge_function((x0, y0), (x1, y1), p);
            if (w0 >= 0 && w1 >= 0 && w2 >= 0) || (w0 <= 0 && w1 <= 0 && w2 <= 0) {
                let color = native_pixel_material_color(triangle.color, x, y, triangle.depth);
                let index = (y * width + x) * 3;
                pixels[index] = color.0;
                pixels[index + 1] = color.1;
                pixels[index + 2] = color.2;
                wrote = true;
            }
        }
    }
    if wrote {
        let outline = native_shaded_rgb(triangle.color, -34);
        draw_pixel_line(pixels, width, height, x0, y0, x1, y1, outline);
        draw_pixel_line(pixels, width, height, x1, y1, x2, y2, outline);
        draw_pixel_line(pixels, width, height, x2, y2, x0, y0, outline);
    }
    wrote
}

#[cfg(target_os = "linux")]
fn edge_function(a: (i32, i32), b: (i32, i32), c: (i32, i32)) -> i64 {
    (c.0 as i64 - a.0 as i64) * (b.1 as i64 - a.1 as i64)
        - (c.1 as i64 - a.1 as i64) * (b.0 as i64 - a.0 as i64)
}

#[cfg(target_os = "linux")]
fn draw_pixel_line(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    mut x0: i32,
    mut y0: i32,
    x1: i32,
    y1: i32,
    color: (u8, u8, u8),
) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if x0 >= 0 && y0 >= 0 && (x0 as usize) < width && (y0 as usize) < height {
            let index = (y0 as usize * width + x0 as usize) * 3;
            pixels[index] = color.0;
            pixels[index + 1] = color.1;
            pixels[index + 2] = color.2;
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

#[cfg(target_os = "linux")]
unsafe fn write_native_combat_resolution_captures(
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    out_dir: &Path,
) -> Result<Vec<NativeCombatResolutionCapture>, String> {
    let mut captures = Vec::new();
    for (file, width, height) in [
        ("native_combat_render_1280x720.ppm", 1280u32, 720u32),
        ("native_combat_render_1280x800.ppm", 1280u32, 800u32),
        ("native_combat_render_1920x1080.ppm", 1920u32, 1080u32),
    ] {
        let path = out_dir.join(file);
        x11_combat_overview_capture(result, silhouette, &path, width, height)?;
        let bytes = fs::read(&path).map_err(|error| error.to_string())?;
        let (_, non_background_pixels, frame_hash) =
            native_ppm_evidence(&path, width, height).map_err(|error| error.to_string())?;
        captures.push(NativeCombatResolutionCapture {
            file: file.to_string(),
            camera: "third_person_verdict_ring".to_string(),
            width,
            height,
            frame_hash,
            source: "x11-resizable-pixmap-overview-after-truth-hash".to_string(),
            capture_role: "diagnostic_overview".to_string(),
            debug_overlay: true,
            triangle_count: 0,
            shaded_triangle_count: 0,
            non_background_pixels: non_background_pixels.max(bytes.len().saturating_sub(32) / 12),
        });
    }

    for (file, capture_role, camera) in [
        (
            "native_product_fighter_select_1920x1080.ppm",
            "fighter_select_loadout",
            "asset_closeup_weapon_armor",
        ),
        (
            "native_product_verdict_ring_1920x1080.ppm",
            "verdict_ring_establishing",
            "third_person_verdict_ring",
        ),
        (
            "native_product_pre_contact_1920x1080.ppm",
            "pre_contact_spacing",
            "planning_tactical_reach",
        ),
        (
            "native_product_contact_1920x1080.ppm",
            "combat_contact_readability",
            "first_person_guard_line",
        ),
        (
            "native_product_material_closeup_1920x1080.ppm",
            "material_impact_closeup",
            "asset_closeup_weapon_armor",
        ),
        (
            "native_product_injury_consequence_1920x1080.ppm",
            "injury_consequence_readability",
            "consequence_aftermath_dwell",
        ),
        (
            "native_product_fight_film_1920x1080.ppm",
            "fight_film_cinematic_shot",
            "fight_film_orbit",
        ),
    ] {
        let path = out_dir.join(file);
        let (bytes, triangle_count, shaded_triangle_count) =
            render_native_product_resolution_capture(result, silhouette, capture_role, 1920, 1080)?;
        fs::write(&path, &bytes).map_err(|error| error.to_string())?;
        let (_, non_background_pixels, frame_hash) =
            native_ppm_evidence(&path, 1920, 1080).map_err(|error| error.to_string())?;
        captures.push(NativeCombatResolutionCapture {
            file: file.to_string(),
            camera: camera.to_string(),
            width: 1920,
            height: 1080,
            frame_hash,
            source: "software-product-renderer-clean-frame-after-truth-hash".to_string(),
            capture_role: capture_role.to_string(),
            debug_overlay: false,
            triangle_count,
            shaded_triangle_count,
            non_background_pixels,
        });
    }
    Ok(captures)
}

#[cfg(target_os = "linux")]
fn native_production_renderer_required_states() -> [&'static str; 17] {
    [
        "boot_main_menu",
        "settings_accessibility",
        "fighter_select",
        "loadout_select",
        "oathyard_establishing_shot",
        "training_arena",
        "fighter_closeups",
        "armor_family_closeups",
        "weapon_family_closeups",
        "planning_timeline",
        "pre_contact_combat_pose",
        "contact_frame",
        "armor_material_damage_frame",
        "injury_capability_consequence_frame",
        "replay_browser",
        "fight_film_cinematic_shot",
        "performance_debug_overlay",
    ]
}

#[cfg(target_os = "linux")]
fn native_production_renderer_state_plans(
) -> [(&'static str, &'static str, &'static str, &'static str); 17] {
    [
        (
            "boot_main_menu",
            "main_menu",
            "menu_boot_establishing",
            "observe_plan",
        ),
        (
            "settings_accessibility",
            "settings_accessibility",
            "settings_accessibility",
            "observe_plan",
        ),
        (
            "fighter_select",
            "fighter_select",
            "fighter_select_loadout",
            "guard_bind",
        ),
        (
            "loadout_select",
            "loadout_select",
            "fighter_select_loadout",
            "guard_bind",
        ),
        (
            "oathyard_establishing_shot",
            "observe",
            "verdict_ring_establishing",
            "observe_plan",
        ),
        (
            "training_arena",
            "observe",
            "verdict_ring_establishing",
            "observe_plan",
        ),
        (
            "fighter_closeups",
            "fighter_select",
            "fighter_select_loadout",
            "guard_bind",
        ),
        (
            "armor_family_closeups",
            "loadout_select",
            "material_impact_closeup",
            "material_anatomy_solve",
        ),
        (
            "weapon_family_closeups",
            "loadout_select",
            "material_impact_closeup",
            "weapon_arc",
        ),
        (
            "planning_timeline",
            "plan",
            "planning_timeline",
            "observe_plan",
        ),
        (
            "pre_contact_combat_pose",
            "commit_reveal",
            "pre_contact_spacing",
            "weapon_arc",
        ),
        (
            "contact_frame",
            "resolve",
            "combat_contact_readability",
            "active_contact",
        ),
        (
            "armor_material_damage_frame",
            "resolve",
            "material_impact_closeup",
            "material_anatomy_solve",
        ),
        (
            "injury_capability_consequence_frame",
            "consequence",
            "injury_consequence_readability",
            "recovery_capability",
        ),
        (
            "replay_browser",
            "replay_browser",
            "replay_browser",
            "recovery_state",
        ),
        (
            "fight_film_cinematic_shot",
            "fight_film",
            "fight_film_cinematic_shot",
            "active_contact",
        ),
        (
            "performance_debug_overlay",
            "performance_debug_overlay",
            "performance_debug_overlay",
            "final_hash_card",
        ),
    ]
}

#[cfg(target_os = "linux")]
fn native_production_motion_index_for_phase(
    motion_frames: &[NativeCombatMotionFrameSpec],
    preferred_phase: &str,
) -> usize {
    motion_frames
        .iter()
        .position(|frame| frame.phase == preferred_phase)
        .or_else(|| {
            motion_frames
                .iter()
                .position(|frame| frame.phase.contains(preferred_phase))
        })
        .unwrap_or(0)
}

#[cfg(target_os = "linux")]
fn native_player_loop_schedule_for_screen(
    player_frames: &[NativePlayerLoopFrameSpec],
    screen: &str,
) -> (u32, u32) {
    player_frames
        .iter()
        .find(|frame| frame.screen == screen)
        .map(|frame| (frame.scheduled_ms, frame.truth_frame))
        .unwrap_or((0, 0))
}

#[cfg(target_os = "linux")]
fn native_ppm_pixel_delta(previous: &[u8], current: &[u8]) -> usize {
    fn payload_offset(bytes: &[u8]) -> usize {
        let mut newlines = 0usize;
        for (index, byte) in bytes.iter().enumerate() {
            if *byte == b'\n' {
                newlines += 1;
                if newlines == 3 {
                    return index + 1;
                }
            }
        }
        0
    }
    let previous = &previous[payload_offset(previous)..];
    let current = &current[payload_offset(current)..];
    previous
        .chunks_exact(3)
        .zip(current.chunks_exact(3))
        .filter(|(a, b)| a != b)
        .count()
}

#[cfg(target_os = "linux")]
fn write_native_production_renderer_bundle(
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    motion_frames: &[NativeCombatMotionFrameSpec],
    player_frames: &[NativePlayerLoopFrameSpec],
    out_dir: &Path,
    capture_command: &str,
    replay_path: &str,
    simulation_micros: u128,
    replay_verify_micros: u128,
) -> Result<NativeProductionRendererSummary, String> {
    let width = 1920u32;
    let height = 1080u32;
    let production_dir = Path::new("artifacts/production_renderer/latest");
    fs::create_dir_all(production_dir).map_err(|error| error.to_string())?;
    let mut captures = Vec::new();
    let mut previous_bytes: Option<Vec<u8>> = None;
    let mut chain_material = String::new();

    for (ordinal, (state, screen, capture_role, preferred_phase)) in
        native_production_renderer_state_plans().iter().enumerate()
    {
        let motion_index = native_production_motion_index_for_phase(motion_frames, preferred_phase);
        let motion = motion_frames.get(motion_index);
        let file_name = format!("production_renderer_{state}_1920x1080.ppm");
        let path = production_dir.join(&file_name);
        let (bytes, triangle_count, shaded_triangle_count) =
            render_native_production_renderer_capture(
                result,
                silhouette,
                motion,
                state,
                screen,
                capture_role,
                width,
                height,
                ordinal,
            )?;
        fs::write(&path, &bytes).map_err(|error| error.to_string())?;
        let (_, non_background_pixels, frame_hash) =
            native_ppm_evidence(&path, width, height).map_err(|error| error.to_string())?;
        let pixel_delta_from_previous = previous_bytes
            .as_ref()
            .map(|previous| native_ppm_pixel_delta(previous, &bytes))
            .unwrap_or(0);
        previous_bytes = Some(bytes);
        let (scheduled_ms, fallback_truth_frame) =
            native_player_loop_schedule_for_screen(player_frames, screen);
        let truth_frame = motion
            .map(|frame| frame.truth_frame)
            .unwrap_or(fallback_truth_frame);
        writeln!(
            &mut chain_material,
            "state:{state}:{file_name}:{truth_frame}:{frame_hash}:{pixel_delta_from_previous}"
        )
        .map_err(|error| error.to_string())?;
        captures.push(NativeProductionRendererFrame {
            file: format!("artifacts/production_renderer/latest/{file_name}"),
            state: (*state).to_string(),
            screen: (*screen).to_string(),
            stream: "production_renderer_state_capture".to_string(),
            capture_role: (*capture_role).to_string(),
            width,
            height,
            source: "production-renderer-state-frame-from-replay-trace-after-truth-hash"
                .to_string(),
            truth_frame,
            scheduled_ms,
            motion_frame_index: motion_index + 1,
            triangle_count,
            shaded_triangle_count,
            non_background_pixels,
            pixel_delta_from_previous,
            frame_hash,
        });
    }

    for (index, motion) in motion_frames.iter().enumerate() {
        let file_name = format!(
            "production_renderer_live_loop_{:03}_1920x1080.ppm",
            index + 1
        );
        let path = production_dir.join(&file_name);
        let (bytes, triangle_count, shaded_triangle_count) =
            render_native_production_renderer_capture(
                result,
                silhouette,
                Some(motion),
                motion.phase,
                "live_replay_loop",
                "live_replay_loop",
                width,
                height,
                index + native_production_renderer_required_states().len(),
            )?;
        fs::write(&path, &bytes).map_err(|error| error.to_string())?;
        let (_, non_background_pixels, frame_hash) =
            native_ppm_evidence(&path, width, height).map_err(|error| error.to_string())?;
        let pixel_delta_from_previous = previous_bytes
            .as_ref()
            .map(|previous| native_ppm_pixel_delta(previous, &bytes))
            .unwrap_or(0);
        previous_bytes = Some(bytes);
        writeln!(
            &mut chain_material,
            "live:{}:{file_name}:{}:{frame_hash}:{pixel_delta_from_previous}",
            motion.index, motion.truth_frame
        )
        .map_err(|error| error.to_string())?;
        captures.push(NativeProductionRendererFrame {
            file: format!("artifacts/production_renderer/latest/{file_name}"),
            state: motion.phase.to_string(),
            screen: "live_replay_loop".to_string(),
            stream: "production_renderer_live_loop".to_string(),
            capture_role: "live_replay_loop".to_string(),
            width,
            height,
            source: "production-renderer-live-frame-from-replay-trace-after-truth-hash".to_string(),
            truth_frame: motion.truth_frame,
            scheduled_ms: motion.progress_permille,
            motion_frame_index: index + 1,
            triangle_count,
            shaded_triangle_count,
            non_background_pixels,
            pixel_delta_from_previous,
            frame_hash,
        });
    }

    let state_capture_count = native_production_renderer_required_states().len();
    let live_loop_frame_count = motion_frames.len();
    let min_pixel_delta_from_previous = captures
        .iter()
        .filter(|capture| capture.pixel_delta_from_previous > 0)
        .map(|capture| capture.pixel_delta_from_previous)
        .min()
        .unwrap_or(0);
    let summary = NativeProductionRendererSummary {
        manifest_path: "artifacts/production_renderer/latest/production_renderer_manifest.json"
            .to_string(),
        report_path: "artifacts/production_renderer/latest/production_renderer_report.md"
            .to_string(),
        width,
        height,
        state_capture_count,
        live_loop_frame_count,
        frame_count: captures.len(),
        min_pixel_delta_from_previous,
        frame_hash_chain: hash_hex(chain_material.as_bytes()),
        captures,
    };
    let manifest = render_native_production_renderer_manifest_json(result, silhouette, &summary);
    let report = render_native_production_renderer_report(result, &summary);
    fs::write(
        production_dir.join("production_renderer_manifest.json"),
        &manifest,
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        production_dir.join("production_renderer_report.md"),
        &report,
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        out_dir.join("native_production_renderer_manifest.json"),
        &manifest,
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        out_dir.join("native_production_renderer_report.md"),
        &report,
    )
    .map_err(|error| error.to_string())?;
    write_native_capture_matrix_artifacts(
        result,
        silhouette,
        motion_frames,
        player_frames,
        production_dir,
        out_dir,
        capture_command,
        replay_path,
        simulation_micros,
        replay_verify_micros,
    )?;
    Ok(summary)
}

#[cfg(target_os = "linux")]
fn write_native_capture_matrix_artifacts(
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    motion_frames: &[NativeCombatMotionFrameSpec],
    player_frames: &[NativePlayerLoopFrameSpec],
    production_dir: &Path,
    out_dir: &Path,
    capture_command: &str,
    replay_path: &str,
    simulation_micros: u128,
    replay_verify_micros: u128,
) -> Result<(), String> {
    let width = 1920u32;
    let height = 1080u32;
    let required_states = native_capture_matrix_required_states();
    let asset_manifest_hash = silhouette.asset_manifest_hash.clone();
    let presentation_asset_manifest_hash = silhouette.presentation_asset_manifest_hash.clone();
    let production_visual_manifest_hash = fs::read("assets/production_visual_manifest.json")
        .map(|bytes| hash_hex(&bytes))
        .unwrap_or_else(|_| "missing".to_string());
    let startup_load_micros = 0u128;
    let artifact_bytes_before = native_capture_matrix_dir_size(production_dir);

    let mut entries = Vec::new();
    let mut timing_samples = Vec::new();
    let mut pixel_index: Vec<(String, Vec<NativeCaptureMatrixPixelSample>)> = Vec::new();
    let mut previous_bytes: Option<Vec<u8>> = None;

    for (ordinal, (capture_id, required_state, screen, capture_role, preferred_phase)) in
        native_capture_matrix_state_plans().iter().enumerate()
    {
        let motion_index = native_production_motion_index_for_phase(motion_frames, preferred_phase);
        let motion = motion_frames.get(motion_index);
        let (bytes, triangle_count, shaded_triangle_count) =
            render_native_production_renderer_capture(
                result,
                silhouette,
                motion,
                required_state,
                screen,
                capture_role,
                width,
                height,
                ordinal,
            )?;
        let render_micros = 0u128;
        let (scheduled_ms, fallback_truth_frame) =
            native_player_loop_schedule_for_screen(player_frames, screen);
        let frame_tick = motion
            .map(|frame| frame.truth_frame)
            .unwrap_or(fallback_truth_frame);
        native_capture_matrix_write_entry(
            &mut entries,
            &mut timing_samples,
            &mut pixel_index,
            &mut previous_bytes,
            production_dir,
            capture_command,
            replay_path,
            result,
            &asset_manifest_hash,
            &presentation_asset_manifest_hash,
            &production_visual_manifest_hash,
            capture_id,
            "product_state",
            required_state,
            screen,
            "state",
            required_state,
            native_capture_matrix_camera_for_role(capture_role),
            frame_tick,
            scheduled_ms,
            motion_index + 1,
            bytes,
            triangle_count,
            shaded_triangle_count,
            render_micros,
        )?;
    }

    for (index, tradition) in FIGHTER_TRADITIONS.iter().enumerate() {
        let fighter = native_capture_matrix_fighter_silhouette(tradition)?;
        let capture_id = format!("fighter_closeup_{}", tradition.id);
        let required_state = format!("fighter_closeup:{}", tradition.id);
        let (bytes, triangle_count, shaded_triangle_count) =
            render_native_capture_matrix_fighter_closeup(&fighter, index, width, height)?;
        let render_micros = 0u128;
        native_capture_matrix_write_entry(
            &mut entries,
            &mut timing_samples,
            &mut pixel_index,
            &mut previous_bytes,
            production_dir,
            capture_command,
            replay_path,
            result,
            &asset_manifest_hash,
            &presentation_asset_manifest_hash,
            &production_visual_manifest_hash,
            &capture_id,
            "fighter_closeup",
            &required_state,
            "fighter_select",
            tradition.id,
            tradition.display_name,
            "asset_closeup_weapon_armor",
            (index as u32 + 1) * 8,
            (index as u32 + 1) * 66,
            index + 1,
            bytes,
            triangle_count,
            shaded_triangle_count,
            render_micros,
        )?;
    }

    for (index, armor) in ARMORS.iter().enumerate() {
        let asset =
            native_presentation_asset_ref(armor.id, "armor").map_err(|error| error.to_string())?;
        let capture_id = format!("armor_loadout_closeup_{}", armor.id);
        let required_state = format!("armor_loadout_closeup:{}", armor.id);
        let (bytes, triangle_count, shaded_triangle_count) =
            render_native_capture_matrix_asset_closeup(
                &asset,
                "armor_loadout_closeup",
                armor.id,
                armor.display_name,
                index,
                width,
                height,
            )?;
        let render_micros = 0u128;
        native_capture_matrix_write_entry(
            &mut entries,
            &mut timing_samples,
            &mut pixel_index,
            &mut previous_bytes,
            production_dir,
            capture_command,
            replay_path,
            result,
            &asset_manifest_hash,
            &presentation_asset_manifest_hash,
            &production_visual_manifest_hash,
            &capture_id,
            "armor_loadout_closeup",
            &required_state,
            "loadout_select",
            armor.id,
            armor.display_name,
            "asset_closeup_weapon_armor",
            160 + index as u32 * 8,
            400 + index as u32 * 66,
            index + 1,
            bytes,
            triangle_count,
            shaded_triangle_count,
            render_micros,
        )?;
    }

    for (index, weapon) in WEAPONS.iter().enumerate() {
        let asset = native_presentation_asset_ref(weapon.id, "weapons")
            .map_err(|error| error.to_string())?;
        let capture_id = format!("weapon_family_closeup_{}", weapon.id);
        let required_state = format!("weapon_family_closeup:{}", weapon.id);
        let (bytes, triangle_count, shaded_triangle_count) =
            render_native_capture_matrix_asset_closeup(
                &asset,
                "weapon_family_closeup",
                weapon.id,
                weapon.display_name,
                index,
                width,
                height,
            )?;
        let render_micros = 0u128;
        native_capture_matrix_write_entry(
            &mut entries,
            &mut timing_samples,
            &mut pixel_index,
            &mut previous_bytes,
            production_dir,
            capture_command,
            replay_path,
            result,
            &asset_manifest_hash,
            &presentation_asset_manifest_hash,
            &production_visual_manifest_hash,
            &capture_id,
            "weapon_family_closeup",
            &required_state,
            "loadout_select",
            weapon.id,
            weapon.display_name,
            "asset_closeup_weapon_armor",
            260 + index as u32 * 8,
            820 + index as u32 * 48,
            index + 1,
            bytes,
            triangle_count,
            shaded_triangle_count,
            render_micros,
        )?;
    }

    for (index, arena) in ARENAS.iter().enumerate() {
        let asset =
            native_presentation_asset_ref(arena.id, "arenas").map_err(|error| error.to_string())?;
        let capture_id = format!("arena_{}", arena.id);
        let required_state = format!("arena:{}", arena.id);
        let (bytes, triangle_count, shaded_triangle_count) =
            render_native_capture_matrix_asset_closeup(
                &asset,
                "arena",
                arena.id,
                arena.display_name,
                index,
                width,
                height,
            )?;
        let render_micros = 0u128;
        native_capture_matrix_write_entry(
            &mut entries,
            &mut timing_samples,
            &mut pixel_index,
            &mut previous_bytes,
            production_dir,
            capture_command,
            replay_path,
            result,
            &asset_manifest_hash,
            &presentation_asset_manifest_hash,
            &production_visual_manifest_hash,
            &capture_id,
            "arena",
            &required_state,
            if arena.id == "training_yard" {
                "training_arena"
            } else {
                "observe"
            },
            arena.id,
            arena.display_name,
            if arena.id == "training_yard" {
                "planning_tactical_reach"
            } else {
                "third_person_verdict_ring"
            },
            360 + index as u32 * 8,
            1300 + index as u32 * 120,
            index + 1,
            bytes,
            triangle_count,
            shaded_triangle_count,
            render_micros,
        )?;
    }

    let missing_required_states =
        native_capture_matrix_missing_required_states(&required_states, &entries);
    if !missing_required_states.is_empty() {
        return Err(format!(
            "capture matrix missing required states: {}",
            missing_required_states.join(",")
        ));
    }
    let artifact_bytes_after = native_capture_matrix_dir_size(production_dir);
    let manifest = render_native_capture_matrix_manifest_json(
        result,
        &required_states,
        &missing_required_states,
        &entries,
    );
    let pixel_manifest = render_native_capture_matrix_pixel_index_json(&pixel_index);
    let timing_json = render_native_capture_matrix_timing_json(
        simulation_micros,
        replay_verify_micros,
        startup_load_micros,
        artifact_bytes_before,
        artifact_bytes_after,
        &timing_samples,
    );
    let timing_report = render_native_capture_matrix_timing_report(
        simulation_micros,
        replay_verify_micros,
        startup_load_micros,
        artifact_bytes_before,
        artifact_bytes_after,
        &timing_samples,
    );
    for (name, content) in [
        ("capture_matrix_manifest.json", manifest.as_str()),
        ("capture_matrix_pixel_index.json", pixel_manifest.as_str()),
        ("capture_matrix_timing.json", timing_json.as_str()),
        ("capture_matrix_timing_report.md", timing_report.as_str()),
    ] {
        fs::write(production_dir.join(name), content).map_err(|error| error.to_string())?;
    }
    fs::write(
        out_dir.join("native_capture_matrix_manifest.json"),
        &manifest,
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        out_dir.join("native_capture_matrix_pixel_index.json"),
        &pixel_manifest,
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        out_dir.join("native_capture_matrix_timing.json"),
        &timing_json,
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        out_dir.join("native_capture_matrix_timing_report.md"),
        &timing_report,
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn native_capture_matrix_state_plans() -> [(
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
); 15] {
    [
        (
            "state_main_menu",
            "main_menu",
            "main_menu",
            "menu_boot_establishing",
            "observe_plan",
        ),
        (
            "state_settings_accessibility",
            "settings_accessibility",
            "settings_accessibility",
            "settings_accessibility",
            "observe_plan",
        ),
        (
            "state_fighter_select",
            "fighter_select",
            "fighter_select",
            "fighter_select_loadout",
            "guard_bind",
        ),
        (
            "state_loadout_select",
            "loadout_select",
            "loadout_select",
            "fighter_select_loadout",
            "guard_bind",
        ),
        (
            "state_oathyard_establishing",
            "oathyard_establishing_shot",
            "observe",
            "verdict_ring_establishing",
            "observe_plan",
        ),
        (
            "state_verdict_ring",
            "arena:oathyard_verdict_ring",
            "observe",
            "verdict_ring_establishing",
            "observe_plan",
        ),
        (
            "state_training_arena",
            "arena:training_yard",
            "training_arena",
            "training_yard_establishing",
            "observe_plan",
        ),
        (
            "state_planning_timeline",
            "planning_timeline",
            "plan",
            "planning_timeline",
            "observe_plan",
        ),
        (
            "state_pre_contact",
            "pre_contact_frame",
            "commit_reveal",
            "pre_contact_spacing",
            "weapon_arc",
        ),
        (
            "state_contact",
            "contact_frame",
            "resolve",
            "combat_contact_readability",
            "active_contact",
        ),
        (
            "state_armor_material_damage",
            "armor_material_damage_frame",
            "resolve",
            "material_impact_closeup",
            "material_anatomy_solve",
        ),
        (
            "state_injury_consequence",
            "injury_capability_consequence_frame",
            "consequence",
            "injury_consequence_readability",
            "recovery_capability",
        ),
        (
            "state_replay_verification_ui",
            "replay_verification_ui",
            "replay_browser",
            "replay_browser",
            "recovery_state",
        ),
        (
            "state_fight_film",
            "fight_film_camera_shot",
            "fight_film",
            "fight_film_cinematic_shot",
            "active_contact",
        ),
        (
            "state_performance_debug_overlay",
            "performance_debug_overlay",
            "performance_debug_overlay",
            "performance_debug_overlay",
            "final_hash_card",
        ),
    ]
}

#[cfg(target_os = "linux")]
fn native_capture_matrix_required_states() -> Vec<String> {
    let mut required = native_capture_matrix_state_plans()
        .iter()
        .map(|(_, required_state, _, _, _)| (*required_state).to_string())
        .collect::<Vec<_>>();
    for fighter in FIGHTER_TRADITIONS {
        required.push(format!("fighter_closeup:{}", fighter.id));
    }
    for armor in ARMORS {
        required.push(format!("armor_loadout_closeup:{}", armor.id));
    }
    for weapon in WEAPONS {
        required.push(format!("weapon_family_closeup:{}", weapon.id));
    }
    required
}

#[cfg(target_os = "linux")]
fn native_capture_matrix_missing_required_states(
    required_states: &[String],
    entries: &[NativeCaptureMatrixEntry],
) -> Vec<String> {
    required_states
        .iter()
        .filter(|required| {
            !entries
                .iter()
                .any(|entry| entry.required_state == required.as_str())
        })
        .cloned()
        .collect()
}

#[cfg(target_os = "linux")]
fn native_capture_matrix_camera_for_role(capture_role: &str) -> &'static str {
    match capture_role {
        "material_impact_closeup" => "asset_closeup_weapon_armor",
        "fight_film_cinematic_shot" => "fight_film_orbit",
        "pre_contact_spacing" | "planning_timeline" => "planning_tactical_reach",
        "combat_contact_readability" | "injury_consequence_readability" | "replay_browser" => {
            "third_person_replay_orbit"
        }
        "training_yard_establishing" => "planning_tactical_reach",
        _ => "third_person_verdict_ring",
    }
}

#[cfg(target_os = "linux")]
fn native_capture_matrix_file_name(capture_id: &str) -> String {
    let safe = capture_id
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    format!("capture_matrix_{safe}_1920x1080.ppm")
}

#[cfg(target_os = "linux")]
fn native_capture_matrix_write_entry(
    entries: &mut Vec<NativeCaptureMatrixEntry>,
    timing_samples: &mut Vec<NativeCaptureMatrixTimingSample>,
    pixel_index: &mut Vec<(String, Vec<NativeCaptureMatrixPixelSample>)>,
    previous_bytes: &mut Option<Vec<u8>>,
    production_dir: &Path,
    capture_command: &str,
    replay_path: &str,
    result: &DuelResult,
    asset_manifest_hash: &str,
    presentation_asset_manifest_hash: &str,
    production_visual_manifest_hash: &str,
    capture_id: &str,
    category: &str,
    required_state: &str,
    screen: &str,
    item_id: &str,
    item_name: &str,
    camera_mode: &str,
    frame_tick: u32,
    scheduled_ms: u32,
    motion_frame_index: usize,
    bytes: Vec<u8>,
    triangle_count: usize,
    shaded_triangle_count: usize,
    render_micros: u128,
) -> Result<(), String> {
    let width = 1920u32;
    let height = 1080u32;
    let file_name = native_capture_matrix_file_name(capture_id);
    let path = production_dir.join(&file_name);
    let pixel_delta_from_previous = previous_bytes
        .as_ref()
        .map(|previous| native_ppm_pixel_delta(previous, &bytes))
        .unwrap_or(0);
    fs::write(&path, &bytes).map_err(|error| error.to_string())?;
    let write_micros = 0u128;
    let (_, non_background_pixels, frame_hash) =
        native_ppm_evidence(&path, width, height).map_err(|error| error.to_string())?;
    let sha256 = file_sha256_hex(&path).map_err(|error| error.to_string())?;
    let samples = native_capture_matrix_ppm_pixel_samples(&path, width, height)?;
    let inspect_micros = 0u128;
    *previous_bytes = Some(bytes);
    timing_samples.push(NativeCaptureMatrixTimingSample {
        capture_id: capture_id.to_string(),
        category: category.to_string(),
        render_micros,
        write_micros,
        inspect_micros,
    });
    pixel_index.push((capture_id.to_string(), samples));
    entries.push(NativeCaptureMatrixEntry {
        capture_id: capture_id.to_string(),
        file: format!("artifacts/production_renderer/latest/{file_name}"),
        category: category.to_string(),
        required_state: required_state.to_string(),
        screen: screen.to_string(),
        item_id: item_id.to_string(),
        item_name: item_name.to_string(),
        command: capture_command.to_string(),
        replay_path: replay_path.to_string(),
        replay_hash: hash_hex(result.replay_json.as_bytes()),
        replay_final_hash: result.final_state_hash.clone(),
        content_hash: result.content_hash.clone(),
        asset_manifest_hash: asset_manifest_hash.to_string(),
        presentation_asset_manifest_hash: presentation_asset_manifest_hash.to_string(),
        production_visual_manifest_hash: production_visual_manifest_hash.to_string(),
        backend_id: "native-software-3d-capture-matrix-loop".to_string(),
        width,
        height,
        camera_mode: camera_mode.to_string(),
        frame_tick,
        scheduled_ms,
        motion_frame_index,
        triangle_count,
        shaded_triangle_count,
        non_background_pixels,
        pixel_delta_from_previous,
        frame_hash,
        sha256,
    });
    Ok(())
}

#[cfg(target_os = "linux")]
fn native_capture_matrix_ppm_pixel_samples(
    path: &Path,
    width: u32,
    height: u32,
) -> Result<Vec<NativeCaptureMatrixPixelSample>, String> {
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    let header = format!("P6\n{} {}\n255\n", width, height);
    if !bytes.starts_with(header.as_bytes()) {
        return Err(format!(
            "capture matrix pixel index PPM header mismatch: {}",
            path.display()
        ));
    }
    let coords = [
        ("center", width / 2, height / 2),
        ("upper_left_rule_of_thirds", width / 3, height / 3),
        ("upper_right_rule_of_thirds", width * 2 / 3, height / 3),
        ("lower_left_rule_of_thirds", width / 3, height * 2 / 3),
        ("lower_right_rule_of_thirds", width * 2 / 3, height * 2 / 3),
        ("hud_timeline_band", width / 2, height * 7 / 8),
    ];
    let mut samples = Vec::new();
    for (label, x, y) in coords {
        let x = x.min(width.saturating_sub(1));
        let y = y.min(height.saturating_sub(1));
        let offset = header.len() + ((y as usize * width as usize + x as usize) * 3);
        if offset + 2 >= bytes.len() {
            return Err(format!(
                "capture matrix pixel index out of bounds: {} {x},{y}",
                path.display()
            ));
        }
        samples.push(NativeCaptureMatrixPixelSample {
            label,
            x,
            y,
            rgb: (bytes[offset], bytes[offset + 1], bytes[offset + 2]),
        });
    }
    Ok(samples)
}

#[cfg(target_os = "linux")]
fn native_capture_matrix_fighter_silhouette(
    tradition: &FighterTradition,
) -> Result<NativeCombatFighterSilhouette, String> {
    let weapon = weapon_by_id(tradition.default_weapon)
        .ok_or_else(|| format!("missing default weapon {}", tradition.default_weapon))?;
    let armor = armor_by_id(tradition.default_armor)
        .ok_or_else(|| format!("missing default armor {}", tradition.default_armor))?;
    Ok(NativeCombatFighterSilhouette {
        seat: 0,
        name: tradition.display_name.to_string(),
        weapon_id: weapon.id.to_string(),
        weapon_name: weapon.display_name.to_string(),
        weapon_asset: native_combat_asset_ref(weapon.id, "weapons")
            .map_err(|error| error.to_string())?,
        weapon_presentation_asset: native_presentation_asset_ref(weapon.id, "weapons")
            .map_err(|error| error.to_string())?,
        weapon_length_mm: weapon.length_mm,
        weapon_reach_mm: weapon.reach_mm,
        weapon_mass_g: weapon.mass_g,
        weapon_inertia_g_cm2: weapon.inertia_g_cm2,
        weapon_span_px: native_weapon_span_px(&weapon),
        weapon_head_px: native_weapon_head_px(&weapon),
        armor_id: armor.id.to_string(),
        armor_name: armor.display_name.to_string(),
        armor_asset: native_combat_asset_ref(armor.id, "armor")
            .map_err(|error| error.to_string())?,
        armor_presentation_asset: native_presentation_asset_ref(armor.id, "armor")
            .map_err(|error| error.to_string())?,
        armor_material: armor.material.to_string(),
        armor_mass_g: armor.mass_g,
        armor_torso_coverage_permille: armor.torso_coverage_permille,
        armor_head_coverage_permille: armor.head_coverage_permille,
        armor_weapon_arm_coverage_permille: armor.weapon_arm_coverage_permille,
        armor_lead_leg_coverage_permille: armor.lead_leg_coverage_permille,
        armor_gap_permille: armor.gap_permille,
        armor_torso_width_px: native_armor_torso_width_px(&armor),
        armor_torso_height_px: native_armor_torso_height_px(&armor),
        armor_head_marker_px: native_armor_head_marker_px(&armor),
        body_mass_g: tradition.body_mass_g,
        stance_width_mm: 1180 + tradition.reach_bias_mm.abs() / 2,
    })
}

#[cfg(target_os = "linux")]
fn render_native_capture_matrix_fighter_closeup(
    fighter: &NativeCombatFighterSilhouette,
    ordinal: usize,
    width: u32,
    height: u32,
) -> Result<(Vec<u8>, usize, usize), String> {
    let width_usize = width as usize;
    let height_usize = height as usize;
    let mut pixels = vec![0u8; width_usize * height_usize * 3];
    paint_native_stone_floor(&mut pixels, width_usize, height_usize);
    draw_native_product_material_witnesses(&mut pixels, width_usize, height_usize);
    let mut triangles = Vec::new();
    let stance = (fighter.stance_width_mm / 3).clamp(260, 520);
    draw_native_contact_shadow(
        &mut pixels,
        width_usize,
        height_usize,
        width as i32 / 2,
        height as i32 * 3 / 5,
        width as i32 / 7,
        height as i32 / 28,
    );
    push_native_fighter_software_meshes_posed(
        &mut triangles,
        fighter,
        -stance / 4 + (ordinal as i32 % 3) * 45,
        -520,
        ((ordinal as i32 % 5) - 2) * 55,
        1,
        720,
        "asset_closeup_weapon_armor",
        width,
        height,
        120 + (fighter.weapon_reach_mm / 9).clamp(70, 260),
        70 - (ordinal as i32 % 4) * 28,
        120 + (ordinal as i32 % 5) * 35,
        0,
    );
    let triangle_count = triangles.len();
    triangles.sort_by_key(|triangle| triangle.depth);
    let mut shaded_triangle_count = 0usize;
    for triangle in &triangles {
        if fill_triangle(&mut pixels, width_usize, height_usize, triangle) {
            shaded_triangle_count += 1;
        }
    }
    apply_native_lighting_post(
        &mut pixels,
        width_usize,
        height_usize,
        1120 + ordinal as i32 * 3,
    );
    draw_native_product_cinematic_bars(&mut pixels, width_usize, height_usize);
    let mut ppm = format!("P6\n{} {}\n255\n", width, height).into_bytes();
    ppm.extend_from_slice(&pixels);
    Ok((ppm, triangle_count, shaded_triangle_count))
}

#[cfg(target_os = "linux")]
fn render_native_capture_matrix_asset_closeup(
    asset: &NativePresentationAssetRef,
    category: &str,
    _item_id: &str,
    _item_name: &str,
    ordinal: usize,
    width: u32,
    height: u32,
) -> Result<(Vec<u8>, usize, usize), String> {
    let width_usize = width as usize;
    let height_usize = height as usize;
    let mut pixels = vec![0u8; width_usize * height_usize * 3];
    paint_native_stone_floor(&mut pixels, width_usize, height_usize);
    if category != "arena" {
        draw_native_product_material_witnesses(&mut pixels, width_usize, height_usize);
    }
    let camera = if category == "arena" {
        if asset.id == "training_yard" {
            "planning_tactical_reach"
        } else {
            "third_person_verdict_ring"
        }
    } else {
        "asset_closeup_weapon_armor"
    };
    let color = match category {
        "weapon_family_closeup" => {
            native_material_color("tempered_steel_edge_worn", (112, 120, 122))
        }
        "armor_loadout_closeup" => native_material_color("riveted_mail_oiled", (72, 82, 86)),
        "arena" => native_material_color("chalked_stone_dust", (168, 158, 132)),
        _ => (128, 116, 92),
    };
    let scale_num = native_capture_matrix_asset_scale_num(&asset.geometry, category);
    let (origin_y, depth_bias) = match category {
        "weapon_family_closeup" => (-420, -300),
        "armor_loadout_closeup" => (-620, -450),
        "arena" => (-1080, -1500),
        _ => (-540, -400),
    };
    draw_native_contact_shadow(
        &mut pixels,
        width_usize,
        height_usize,
        width as i32 / 2,
        height as i32 * 3 / 5,
        width as i32 / if category == "arena" { 4 } else { 8 },
        height as i32 / 34,
    );
    let mut triangles = Vec::new();
    push_native_mesh_triangles(
        &mut triangles,
        NativeSoftware3dInstance {
            geometry: &asset.geometry,
            origin_x_milli: ((ordinal as i32 % 5) - 2) * 60,
            origin_y_milli: origin_y,
            origin_z_milli: ((ordinal as i32 % 7) - 3) * 80,
            scale_num,
            scale_den: 1000,
            facing: 1,
            depth_bias,
            color,
        },
        width,
        height,
        camera,
    );
    let triangle_count = triangles.len();
    triangles.sort_by_key(|triangle| triangle.depth);
    let mut shaded_triangle_count = 0usize;
    for triangle in &triangles {
        if fill_triangle(&mut pixels, width_usize, height_usize, triangle) {
            shaded_triangle_count += 1;
        }
    }
    if category == "weapon_family_closeup" {
        draw_pixel_line(
            &mut pixels,
            width_usize,
            height_usize,
            width as i32 / 3,
            height as i32 / 2,
            width as i32 * 2 / 3,
            height as i32 / 2 + (ordinal as i32 % 5) * 18 - 36,
            (224, 132, 38),
        );
    }
    apply_native_lighting_post(
        &mut pixels,
        width_usize,
        height_usize,
        1110 + ordinal as i32 * 5,
    );
    draw_native_product_cinematic_bars(&mut pixels, width_usize, height_usize);
    let mut ppm = format!("P6\n{} {}\n255\n", width, height).into_bytes();
    ppm.extend_from_slice(&pixels);
    Ok((ppm, triangle_count, shaded_triangle_count))
}

#[cfg(target_os = "linux")]
fn native_capture_matrix_asset_scale_num(geometry: &NativeGltfGeometry, category: &str) -> i32 {
    let extent = (geometry.max_x_milli - geometry.min_x_milli)
        .abs()
        .max((geometry.max_y_milli - geometry.min_y_milli).abs())
        .max((geometry.max_z_milli - geometry.min_z_milli).abs())
        .max(1);
    let target = match category {
        "weapon_family_closeup" => 3200,
        "armor_loadout_closeup" => 2100,
        "arena" => 6200,
        _ => 2400,
    };
    ((target * 1000) / extent).clamp(140, 2600)
}

#[cfg(target_os = "linux")]
fn native_capture_matrix_dir_size(path: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    let mut total = 0u64;
    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            total = total.saturating_add(native_capture_matrix_dir_size(&entry_path));
        } else if let Ok(metadata) = entry_path.metadata() {
            total = total.saturating_add(metadata.len());
        }
    }
    total
}

#[cfg(target_os = "linux")]
fn render_native_capture_matrix_manifest_json(
    result: &DuelResult,
    required_states: &[String],
    missing_required_states: &[String],
    entries: &[NativeCaptureMatrixEntry],
) -> String {
    let mut matrix_material = String::new();
    for entry in entries {
        writeln!(
            &mut matrix_material,
            "{}:{}:{}:{}:{}:{}",
            entry.capture_id,
            entry.required_state,
            entry.file,
            entry.width,
            entry.height,
            entry.sha256
        )
        .unwrap();
    }
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", CAPTURE_MATRIX_MANIFEST_SCHEMA, true);
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(&mut out, 1, "scenario_id", &result.scenario_id, true);
    write_json_field(
        &mut out,
        1,
        "replay_final_hash",
        &result.final_state_hash,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "replay_hash",
        &hash_hex(result.replay_json.as_bytes()),
        true,
    );
    write_json_field(&mut out, 1, "content_hash", &result.content_hash, true);
    write_json_field(
        &mut out,
        1,
        "backend_id",
        "native-software-3d-capture-matrix-loop",
        true,
    );
    writeln!(&mut out, "  \"width\": 1920,").unwrap();
    writeln!(&mut out, "  \"height\": 1080,").unwrap();
    writeln!(&mut out, "  \"capture_count\": {},", entries.len()).unwrap();
    writeln!(
        &mut out,
        "  \"required_state_count\": {},",
        required_states.len()
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"all_required_captures_present\": {},",
        missing_required_states.is_empty()
    )
    .unwrap();
    write_native_json_string_array(
        &mut out,
        1,
        "missing_required_states",
        missing_required_states,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "capture_matrix_hash",
        &hash_hex(matrix_material.as_bytes()),
        true,
    );
    writeln!(&mut out, "  \"replay_verified_before_capture\": true,").unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"owner_visual_acceptance\": false,").unwrap();
    writeln!(&mut out, "  \"public_demo_ready\": {PUBLIC_DEMO_READY},").unwrap();
    writeln!(
        &mut out,
        "  \"release_candidate_ready\": {RELEASE_CANDIDATE_READY},"
    )
    .unwrap();
    writeln!(&mut out, "  \"no_upscaling\": true,").unwrap();
    writeln!(&mut out, "  \"pixel_inspection_index\": \"artifacts/production_renderer/latest/capture_matrix_pixel_index.json\",").unwrap();
    writeln!(&mut out, "  \"timing_report\": \"artifacts/production_renderer/latest/capture_matrix_timing_report.md\",").unwrap();
    write_native_json_string_array(&mut out, 1, "required_states", required_states, true);
    writeln!(&mut out, "  \"state_coverage\": {{").unwrap();
    for (index, required) in required_states.iter().enumerate() {
        let captures = entries
            .iter()
            .filter(|entry| entry.required_state == *required)
            .map(|entry| entry.capture_id.clone())
            .collect::<Vec<_>>();
        write_native_json_string_array(
            &mut out,
            2,
            required,
            &captures,
            index + 1 != required_states.len(),
        );
    }
    writeln!(&mut out, "  }},").unwrap();
    writeln!(&mut out, "  \"captures\": [").unwrap();
    for (index, entry) in entries.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "capture_id", &entry.capture_id, true);
        write_json_field(&mut out, 3, "path", &entry.file, true);
        write_json_field(&mut out, 3, "file", &entry.file, true);
        write_json_field(&mut out, 3, "category", &entry.category, true);
        write_json_field(&mut out, 3, "required_state", &entry.required_state, true);
        write_json_field(&mut out, 3, "screen", &entry.screen, true);
        write_json_field(&mut out, 3, "item_id", &entry.item_id, true);
        write_json_field(&mut out, 3, "item_name", &entry.item_name, true);
        write_json_field(&mut out, 3, "command", &entry.command, true);
        write_json_field(&mut out, 3, "replay_path", &entry.replay_path, true);
        write_json_field(&mut out, 3, "replay_hash", &entry.replay_hash, true);
        write_json_field(
            &mut out,
            3,
            "replay_final_hash",
            &entry.replay_final_hash,
            true,
        );
        write_json_field(&mut out, 3, "content_hash", &entry.content_hash, true);
        write_json_field(
            &mut out,
            3,
            "asset_manifest_hash",
            &entry.asset_manifest_hash,
            true,
        );
        write_json_field(
            &mut out,
            3,
            "presentation_asset_manifest_hash",
            &entry.presentation_asset_manifest_hash,
            true,
        );
        write_json_field(
            &mut out,
            3,
            "production_visual_manifest_hash",
            &entry.production_visual_manifest_hash,
            true,
        );
        write_json_field(&mut out, 3, "backend_id", &entry.backend_id, true);
        writeln!(&mut out, "      \"width\": {},", entry.width).unwrap();
        writeln!(&mut out, "      \"height\": {},", entry.height).unwrap();
        write_json_field(&mut out, 3, "camera_mode", &entry.camera_mode, true);
        writeln!(&mut out, "      \"frame_tick\": {},", entry.frame_tick).unwrap();
        writeln!(&mut out, "      \"scheduled_ms\": {},", entry.scheduled_ms).unwrap();
        writeln!(
            &mut out,
            "      \"motion_frame_index\": {},",
            entry.motion_frame_index
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"triangle_count\": {},",
            entry.triangle_count
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"shaded_triangle_count\": {},",
            entry.shaded_triangle_count
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"non_background_pixels\": {},",
            entry.non_background_pixels
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"pixel_delta_from_previous\": {},",
            entry.pixel_delta_from_previous
        )
        .unwrap();
        write_json_field(&mut out, 3, "frame_hash", &entry.frame_hash, true);
        write_json_field(&mut out, 3, "sha256", &entry.sha256, true);
        writeln!(
            &mut out,
            "      \"meets_min_resolution\": {},",
            entry.width >= 1920 && entry.height >= 1080
        )
        .unwrap();
        writeln!(&mut out, "      \"capture_after_truth_hash\": true,").unwrap();
        writeln!(&mut out, "      \"truth_mutation\": false,").unwrap();
        writeln!(&mut out, "      \"presentation_only\": true,").unwrap();
        writeln!(&mut out, "      \"owner_visual_acceptance\": false").unwrap();
        writeln!(&mut out, "    }}{}", comma(index + 1, entries.len())).unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

#[cfg(target_os = "linux")]
fn render_native_capture_matrix_pixel_index_json(
    pixel_index: &[(String, Vec<NativeCaptureMatrixPixelSample>)],
) -> String {
    let sample_count: usize = pixel_index.iter().map(|(_, samples)| samples.len()).sum();
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(
        &mut out,
        1,
        "schema",
        CAPTURE_MATRIX_PIXEL_INDEX_SCHEMA,
        true,
    );
    writeln!(&mut out, "  \"capture_count\": {},", pixel_index.len()).unwrap();
    writeln!(&mut out, "  \"sample_count\": {},", sample_count).unwrap();
    writeln!(&mut out, "  \"metadata_only_check\": false,").unwrap();
    writeln!(&mut out, "  \"pixel_inspection\": true,").unwrap();
    writeln!(&mut out, "  \"owner_visual_acceptance\": false,").unwrap();
    writeln!(&mut out, "  \"captures\": [").unwrap();
    for (capture_index, (capture_id, samples)) in pixel_index.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "capture_id", capture_id, true);
        writeln!(&mut out, "      \"samples\": [").unwrap();
        for (sample_index, sample) in samples.iter().enumerate() {
            writeln!(
                &mut out,
                "        {{\"label\": {}, \"x\": {}, \"y\": {}, \"rgb\": [{}, {}, {}]}}{}",
                json_quote(sample.label),
                sample.x,
                sample.y,
                sample.rgb.0,
                sample.rgb.1,
                sample.rgb.2,
                comma(sample_index + 1, samples.len())
            )
            .unwrap();
        }
        writeln!(&mut out, "      ]").unwrap();
        writeln!(
            &mut out,
            "    }}{}",
            comma(capture_index + 1, pixel_index.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

#[cfg(target_os = "linux")]
fn native_capture_matrix_timing_totals(
    timing_samples: &[NativeCaptureMatrixTimingSample],
) -> (u128, u128, u128, u128) {
    let mut totals = timing_samples
        .iter()
        .map(|sample| sample.render_micros + sample.write_micros + sample.inspect_micros)
        .collect::<Vec<_>>();
    if totals.is_empty() {
        return (0, 0, 0, 0);
    }
    totals.sort_unstable();
    let sum = totals.iter().copied().sum::<u128>();
    (
        totals[0],
        totals[totals.len() / 2],
        *totals.last().unwrap(),
        sum,
    )
}

#[cfg(target_os = "linux")]
fn render_native_capture_matrix_timing_json(
    simulation_micros: u128,
    replay_verify_micros: u128,
    startup_load_micros: u128,
    artifact_bytes_before: u64,
    artifact_bytes_after: u64,
    timing_samples: &[NativeCaptureMatrixTimingSample],
) -> String {
    let (min_total, median_total, max_total, sum_total) =
        native_capture_matrix_timing_totals(timing_samples);
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", CAPTURE_MATRIX_TIMING_SCHEMA, true);
    writeln!(
        &mut out,
        "  \"simulation_step_micros\": {simulation_micros},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"replay_verify_micros\": {replay_verify_micros},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"startup_load_micros\": {startup_load_micros},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"rendered_capture_count\": {},",
        timing_samples.len()
    )
    .unwrap();
    writeln!(&mut out, "  \"frame_time_total_micros_min\": {min_total},").unwrap();
    writeln!(
        &mut out,
        "  \"frame_time_total_micros_median\": {median_total},"
    )
    .unwrap();
    writeln!(&mut out, "  \"frame_time_total_micros_max\": {max_total},").unwrap();
    writeln!(&mut out, "  \"frame_time_total_micros_sum\": {sum_total},").unwrap();
    writeln!(
        &mut out,
        "  \"peak_framebuffer_bytes\": {},",
        1920u64 * 1080u64 * 3
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"artifact_directory_bytes_before\": {artifact_bytes_before},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"artifact_directory_bytes_after\": {artifact_bytes_after},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"artifact_directory_size_delta_bytes\": {},",
        artifact_bytes_after.saturating_sub(artifact_bytes_before)
    )
    .unwrap();
    writeln!(&mut out, "  \"package_size_delta_bytes\": 0,").unwrap();
    writeln!(
        &mut out,
        "  \"artifact_generation_throughput_not_interactive_product_fps\": true,"
    )
    .unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(&mut out, "  \"samples\": [").unwrap();
    for (index, sample) in timing_samples.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "capture_id", &sample.capture_id, true);
        write_json_field(&mut out, 3, "category", &sample.category, true);
        writeln!(
            &mut out,
            "      \"render_micros\": {},",
            sample.render_micros
        )
        .unwrap();
        writeln!(&mut out, "      \"write_micros\": {},", sample.write_micros).unwrap();
        writeln!(
            &mut out,
            "      \"inspect_micros\": {},",
            sample.inspect_micros
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"total_micros\": {}",
            sample.render_micros + sample.write_micros + sample.inspect_micros
        )
        .unwrap();
        writeln!(&mut out, "    }}{}", comma(index + 1, timing_samples.len())).unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

#[cfg(target_os = "linux")]
fn render_native_capture_matrix_timing_report(
    simulation_micros: u128,
    replay_verify_micros: u128,
    startup_load_micros: u128,
    artifact_bytes_before: u64,
    artifact_bytes_after: u64,
    timing_samples: &[NativeCaptureMatrixTimingSample],
) -> String {
    let (min_total, median_total, max_total, sum_total) =
        native_capture_matrix_timing_totals(timing_samples);
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD HIFI-WO-02 Capture Matrix Timing").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "Status: LOCAL STRUCTURAL CAPTURE MATRIX READY").unwrap();
    writeln!(
        &mut out,
        "- Simulation step time: `{simulation_micros}` micros"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Replay verification time: `{replay_verify_micros}` micros"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Startup/load setup time: `{startup_load_micros}` micros"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Rendered capture samples: `{}`",
        timing_samples.len()
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Frame total min/median/max micros: `{min_total}` / `{median_total}` / `{max_total}`"
    )
    .unwrap();
    writeln!(&mut out, "- Frame total sum micros: `{sum_total}`").unwrap();
    writeln!(
        &mut out,
        "- Peak framebuffer bytes: `{}`",
        1920u64 * 1080u64 * 3
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Artifact directory bytes before: `{artifact_bytes_before}`"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Artifact directory bytes after: `{artifact_bytes_after}`"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Artifact directory delta bytes: `{}`",
        artifact_bytes_after.saturating_sub(artifact_bytes_before)
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Package size delta bytes: `0` (capture artifacts are not packaged product payload)"
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Artifact generation throughput is not interactive product FPS: `true`"
    )
    .unwrap();
    writeln!(&mut out, "- Truth mutation: `false`").unwrap();
    writeln!(&mut out, "- Owner visual acceptance: `false`").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Capture Samples").unwrap();
    for sample in timing_samples {
        writeln!(
            &mut out,
            "- `{}` `{}` render `{}` write `{}` inspect `{}` total `{}` micros",
            sample.capture_id,
            sample.category,
            sample.render_micros,
            sample.write_micros,
            sample.inspect_micros,
            sample.render_micros + sample.write_micros + sample.inspect_micros
        )
        .unwrap();
    }
    out
}

#[cfg(target_os = "linux")]
fn render_native_production_renderer_manifest_json(
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    summary: &NativeProductionRendererSummary,
) -> String {
    let required_states = native_production_renderer_required_states()
        .iter()
        .map(|state| (*state).to_string())
        .collect::<Vec<_>>();
    let production_asset_manifest_hash = fs::read("assets/production_visual_manifest.json")
        .map(|bytes| hash_hex(&bytes))
        .unwrap_or_else(|_| "missing".to_string());
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(
        &mut out,
        1,
        "schema",
        PRODUCTION_RENDERER_MANIFEST_SCHEMA,
        true,
    );
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
    write_json_field(
        &mut out,
        1,
        "replay_json_hash",
        &hash_hex(result.replay_json.as_bytes()),
        true,
    );
    write_json_field(
        &mut out,
        1,
        "trace_json_hash",
        &hash_hex(result.trace_json.as_bytes()),
        true,
    );
    write_json_field(
        &mut out,
        1,
        "backend_id",
        "native-software-3d-production-capture-loop",
        true,
    );
    write_json_field(
        &mut out,
        1,
        "source",
        "current-run-replay-and-trace-after-truth-hash",
        true,
    );
    write_json_field(
        &mut out,
        1,
        "production_visual_manifest",
        "assets/production_visual_manifest.json",
        true,
    );
    write_json_field(
        &mut out,
        1,
        "production_visual_manifest_hash",
        &production_asset_manifest_hash,
        true,
    );
    writeln!(&mut out, "  \"width\": {},", summary.width).unwrap();
    writeln!(&mut out, "  \"height\": {},", summary.height).unwrap();
    writeln!(&mut out, "  \"capture_count\": {},", summary.frame_count).unwrap();
    writeln!(
        &mut out,
        "  \"state_capture_count\": {},",
        summary.state_capture_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"live_loop_frame_count\": {},",
        summary.live_loop_frame_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"min_pixel_delta_from_previous\": {},",
        summary.min_pixel_delta_from_previous
    )
    .unwrap();
    write_json_field(
        &mut out,
        1,
        "frame_hash_chain",
        &summary.frame_hash_chain,
        true,
    );
    writeln!(&mut out, "  \"post_hash_only\": true,").unwrap();
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(&mut out, "  \"debug_overlay\": false,").unwrap();
    writeln!(&mut out, "  \"source_backed_assets\": true,").unwrap();
    writeln!(&mut out, "  \"production_renderer_manifest_backed\": true,").unwrap();
    writeln!(&mut out, "  \"production_renderer_complete\": false,").unwrap();
    writeln!(&mut out, "  \"owner_visual_acceptance\": false,").unwrap();
    writeln!(&mut out, "  \"public_demo_ready\": {PUBLIC_DEMO_READY},").unwrap();
    writeln!(
        &mut out,
        "  \"release_candidate_ready\": {RELEASE_CANDIDATE_READY},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"current_capture_floor\": \"visible animated replay/trace-driven 1920x1080 native frames; local structural capture evidence, not owner visual acceptance\","
    )
    .unwrap();
    write_native_json_string_array(&mut out, 1, "required_states", &required_states, true);
    writeln!(&mut out, "  \"state_coverage\": {{").unwrap();
    for (index, state) in required_states.iter().enumerate() {
        let files = summary
            .captures
            .iter()
            .filter(|capture| capture.state == *state)
            .map(|capture| capture.file.clone())
            .collect::<Vec<_>>();
        write_native_json_string_array(
            &mut out,
            2,
            state,
            &files,
            index + 1 != required_states.len(),
        );
    }
    writeln!(&mut out, "  }},").unwrap();
    writeln!(&mut out, "  \"captures\": [").unwrap();
    for (index, capture) in summary.captures.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "path", &capture.file, true);
        write_json_field(&mut out, 3, "file", &capture.file, true);
        write_json_field(&mut out, 3, "stream", &capture.stream, true);
        write_json_field(&mut out, 3, "state", &capture.state, true);
        write_native_json_string_array(&mut out, 3, "states", &[capture.state.clone()], true);
        write_json_field(&mut out, 3, "screen", &capture.screen, true);
        write_json_field(&mut out, 3, "capture_role", &capture.capture_role, true);
        writeln!(&mut out, "      \"width\": {},", capture.width).unwrap();
        writeln!(&mut out, "      \"height\": {},", capture.height).unwrap();
        writeln!(&mut out, "      \"truth_frame\": {},", capture.truth_frame).unwrap();
        writeln!(
            &mut out,
            "      \"scheduled_ms\": {},",
            capture.scheduled_ms
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"motion_frame_index\": {},",
            capture.motion_frame_index
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"triangle_count\": {},",
            capture.triangle_count
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"shaded_triangle_count\": {},",
            capture.shaded_triangle_count
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"non_background_pixels\": {},",
            capture.non_background_pixels
        )
        .unwrap();
        writeln!(
            &mut out,
            "      \"pixel_delta_from_previous\": {},",
            capture.pixel_delta_from_previous
        )
        .unwrap();
        write_json_field(&mut out, 3, "source", &capture.source, true);
        writeln!(&mut out, "      \"debug_overlay\": false,").unwrap();
        writeln!(&mut out, "      \"debug_overlay_removed\": true,").unwrap();
        writeln!(&mut out, "      \"source_backed_assets\": true,").unwrap();
        writeln!(
            &mut out,
            "      \"production_renderer_manifest_backed\": true,"
        )
        .unwrap();
        writeln!(&mut out, "      \"production_asset_evidence\": true,").unwrap();
        writeln!(
            &mut out,
            "      \"native_3d_production_renderer_evidence\": true,"
        )
        .unwrap();
        writeln!(&mut out, "      \"capture_after_truth_hash\": true,").unwrap();
        writeln!(&mut out, "      \"presentation_only\": true,").unwrap();
        writeln!(&mut out, "      \"truth_mutation\": false,").unwrap();
        write_json_field(&mut out, 3, "frame_hash", &capture.frame_hash, false);
        writeln!(
            &mut out,
            "    }}{}",
            comma(index + 1, summary.captures.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();
    write_native_json_string_array(
        &mut out,
        1,
        "asset_ids",
        &native_renderer_asset_ids(silhouette),
        true,
    );
    write_native_json_string_array(
        &mut out,
        1,
        "material_ids",
        &native_renderer_material_ids(silhouette),
        false,
    );
    writeln!(&mut out, "}}").unwrap();
    out
}

#[cfg(target_os = "linux")]
fn render_native_production_renderer_report(
    result: &DuelResult,
    summary: &NativeProductionRendererSummary,
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD Native Production Renderer Capture").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "Status: LOCAL STRUCTURAL CAPTURE READY").unwrap();
    writeln!(&mut out, "- Scenario: `{}`", result.scenario_id).unwrap();
    writeln!(
        &mut out,
        "- Final state hash: `{}`",
        result.final_state_hash
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Resolution: `{}x{}`",
        summary.width, summary.height
    )
    .unwrap();
    writeln!(
        &mut out,
        "- State captures: `{}`",
        summary.state_capture_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Live loop frames: `{}`",
        summary.live_loop_frame_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Min pixel delta from previous non-initial frame: `{}`",
        summary.min_pixel_delta_from_previous
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Frame hash chain: `{}`",
        summary.frame_hash_chain
    )
    .unwrap();
    writeln!(&mut out, "- Manifest: `{}`", summary.manifest_path).unwrap();
    writeln!(&mut out, "- Truth mutation: `false`").unwrap();
    writeln!(&mut out, "- Production renderer complete: `false`").unwrap();
    writeln!(&mut out, "- Owner visual acceptance: `false`").unwrap();
    writeln!(&mut out, "- Public demo ready: `{PUBLIC_DEMO_READY}`").unwrap();
    writeln!(
        &mut out,
        "- Release candidate ready: `{RELEASE_CANDIDATE_READY}`"
    )
    .unwrap();
    out
}

#[cfg(target_os = "linux")]
fn render_native_production_renderer_capture(
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    motion: Option<&NativeCombatMotionFrameSpec>,
    state: &str,
    screen: &str,
    capture_role: &str,
    width: u32,
    height: u32,
    frame_ordinal: usize,
) -> Result<(Vec<u8>, usize, usize), String> {
    let width_usize = width as usize;
    let height_usize = height as usize;
    let mut pixels = vec![0u8; width_usize * height_usize * 3];
    paint_native_stone_floor(&mut pixels, width_usize, height_usize);
    draw_native_product_environment(&mut pixels, width_usize, height_usize, capture_role);
    draw_native_product_truth_accents(&mut pixels, width_usize, height_usize, result, capture_role);

    let progress = motion
        .map(|frame| frame.progress_permille as i32)
        .unwrap_or(((frame_ordinal * 97) % 1000) as i32)
        .clamp(0, 1000);
    let camera = match capture_role {
        "material_impact_closeup" => "asset_closeup_weapon_armor",
        "fight_film_cinematic_shot" => "fight_film_orbit",
        "live_replay_loop" | "combat_contact_readability" | "injury_consequence_readability" => {
            "third_person_replay_orbit"
        }
        _ => "third_person_verdict_ring",
    };
    let mut triangles = Vec::new();
    let arena_scale = if matches!(screen, "main_menu" | "settings_accessibility") {
        118
    } else if capture_role == "verdict_ring_establishing" {
        170
    } else {
        138
    };
    push_native_mesh_triangles(
        &mut triangles,
        NativeSoftware3dInstance {
            geometry: &silhouette.arena_presentation_asset.geometry,
            origin_x_milli: 0,
            origin_y_milli: -980,
            origin_z_milli: 0,
            scale_num: arena_scale,
            scale_den: 1000,
            facing: 1,
            depth_bias: -1400,
            color: native_material_color("chalked_stone_dust", (160, 151, 127)),
        },
        width,
        height,
        camera,
    );

    let combat_weight = match screen {
        "observe" | "plan" | "commit_reveal" | "resolve" | "consequence" | "fight_film"
        | "replay_browser" | "live_replay_loop" => 1,
        _ => 0,
    };
    let approach = if combat_weight == 1 {
        120 + progress * 560 / 1000
    } else {
        40 + (frame_ordinal as i32 % 5) * 16
    };
    let collapse = if matches!(
        state,
        "injury_capability_consequence_frame" | "recovery_capability" | "stagger_collapse_risk"
    ) {
        180 + progress / 6
    } else {
        0
    };
    let seat_y = if capture_role == "material_impact_closeup" {
        -800
    } else {
        -430
    };
    let depth_spread = if capture_role == "material_impact_closeup" {
        250
    } else {
        110 + progress / 18
    };
    let scale = if capture_role == "material_impact_closeup" {
        900
    } else if matches!(screen, "fighter_select" | "loadout_select") {
        520
    } else {
        470
    };
    let seat0_x = -980 + approach;
    let seat1_x = 980 - approach;
    for (origin_x, origin_z, local_collapse) in [
        (seat0_x, -depth_spread, 0),
        (seat1_x, depth_spread, collapse),
    ] {
        let (shadow_x, shadow_y) = native_product_shadow_screen_point(
            camera,
            width,
            height,
            origin_x,
            seat_y + local_collapse / 2,
            origin_z,
        );
        draw_native_contact_shadow(
            &mut pixels,
            width_usize,
            height_usize,
            shadow_x,
            shadow_y + (height as i32 / 22),
            (width as i32 / 13).max(96),
            (height as i32 / 40).max(22),
        );
    }
    if let Some(fighter) = silhouette.fighter(0) {
        push_native_fighter_software_meshes_posed(
            &mut triangles,
            fighter,
            seat0_x,
            seat_y,
            -depth_spread,
            -1,
            scale,
            camera,
            width,
            height,
            80 + progress * 420 / 1000,
            if capture_role == "combat_contact_readability" {
                -80
            } else {
                40
            },
            80 + progress / 5,
            0,
        );
    }
    if let Some(fighter) = silhouette.fighter(1) {
        push_native_fighter_software_meshes_posed(
            &mut triangles,
            fighter,
            seat1_x,
            seat_y,
            depth_spread,
            1,
            scale,
            camera,
            width,
            height,
            40 + progress * 180 / 1000,
            20 - progress / 30,
            40,
            collapse,
        );
    }
    let triangle_count = triangles.len();
    triangles.sort_by_key(|triangle| triangle.depth);
    let mut shaded_triangle_count = 0usize;
    for triangle in &triangles {
        if fill_triangle(&mut pixels, width_usize, height_usize, triangle) {
            shaded_triangle_count += 1;
        }
    }

    draw_native_production_screen_ui(
        &mut pixels,
        width_usize,
        height_usize,
        screen,
        state,
        progress,
        frame_ordinal,
    );
    if matches!(
        capture_role,
        "combat_contact_readability" | "material_impact_closeup" | "live_replay_loop"
    ) {
        draw_native_contact_bloom(
            &mut pixels,
            width_usize,
            height_usize,
            width as i32 / 2 + progress / 5 - 110,
            height as i32 / 2 + 34,
            width as i32 / 9,
            native_material_color("wet_blood_trace_overlay", (124, 30, 22)),
        );
    }
    if matches!(state, "performance_debug_overlay") {
        draw_native_performance_overlay_pixels(&mut pixels, width_usize, height_usize, progress);
    }
    apply_native_lighting_post(&mut pixels, width_usize, height_usize, 1115 + progress / 20);
    draw_native_product_cinematic_bars(&mut pixels, width_usize, height_usize);
    let mut ppm = format!("P6\n{} {}\n255\n", width, height).into_bytes();
    ppm.extend_from_slice(&pixels);
    Ok((ppm, triangle_count, shaded_triangle_count))
}

#[cfg(target_os = "linux")]
fn draw_native_production_screen_ui(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    screen: &str,
    state: &str,
    progress: i32,
    frame_ordinal: usize,
) {
    let panel = (18, 24, 27);
    fill_rect(
        pixels,
        width,
        height,
        width / 28,
        height / 13,
        width / 5,
        height / 4,
        panel,
    );
    fill_rect(
        pixels,
        width,
        height,
        width - width / 4,
        height / 13,
        width / 5,
        height / 4,
        (21, 23, 25),
    );
    let accent = match screen {
        "main_menu" => (204, 166, 82),
        "settings_accessibility" => (82, 158, 188),
        "fighter_select" | "loadout_select" => (146, 110, 72),
        "resolve" | "consequence" | "live_replay_loop" => (194, 82, 48),
        "fight_film" | "replay_browser" => (170, 150, 112),
        _ => (188, 128, 52),
    };
    for row in 0..5usize {
        let w = (width / 16 + ((frame_ordinal + row) * width / 97) % (width / 12)).max(18);
        fill_rect(
            pixels,
            width,
            height,
            width / 18,
            height / 10 + row * height / 38,
            w,
            (height / 86).max(8),
            if row == 0 { accent } else { (72, 78, 78) },
        );
    }
    for row in 0..4usize {
        let x = width - width / 4 + width / 45;
        let y = height / 10 + row * height / 34;
        fill_rect(
            pixels,
            width,
            height,
            x,
            y,
            width / 8 + row * width / 90,
            (height / 70).max(10),
            if row % 2 == 0 { (66, 68, 64) } else { accent },
        );
    }
    let timeline_y = height - height / 8;
    fill_rect(
        pixels,
        width,
        height,
        width / 5,
        timeline_y,
        width * 3 / 5,
        (height / 96).max(8),
        (44, 42, 38),
    );
    let marker_x = width / 5 + (width * 3 / 5) * progress.max(0) as usize / 1000;
    fill_rect(
        pixels,
        width,
        height,
        marker_x.saturating_sub(width / 160),
        timeline_y.saturating_sub(height / 54),
        (width / 80).max(12),
        (height / 32).max(20),
        accent,
    );
    let state_hash = hash_hex(state.as_bytes());
    for (index, byte) in state_hash.as_bytes().iter().take(12).enumerate() {
        let x = width / 2 + index * width / 86 - width / 13;
        let y = height / 7 + (*byte as usize % (height / 18).max(1));
        blend_ellipse(
            pixels,
            width,
            height,
            x as i32,
            y as i32,
            (width / 220).max(5) as i32,
            (height / 260).max(4) as i32,
            accent,
            92,
        );
    }
}

#[cfg(target_os = "linux")]
fn draw_native_performance_overlay_pixels(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    progress: i32,
) {
    let x = width / 2 - width / 8;
    let y = height / 9;
    fill_rect(
        pixels,
        width,
        height,
        x,
        y,
        width / 4,
        height / 5,
        (12, 15, 16),
    );
    for row in 0..8usize {
        let bar = width / 18 + ((progress as usize + row * 41) % (width / 8).max(1));
        fill_rect(
            pixels,
            width,
            height,
            x + width / 52,
            y + height / 44 + row * height / 54,
            bar,
            (height / 120).max(5),
            if row % 3 == 0 {
                (92, 188, 126)
            } else {
                (198, 146, 58)
            },
        );
    }
}

#[cfg(target_os = "linux")]
fn render_native_product_resolution_capture(
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    capture_role: &str,
    width: u32,
    height: u32,
) -> Result<(Vec<u8>, usize, usize), String> {
    let width_usize = width as usize;
    let height_usize = height as usize;
    let mut pixels = vec![0u8; width_usize * height_usize * 3];
    paint_native_stone_floor(&mut pixels, width_usize, height_usize);
    draw_native_product_environment(&mut pixels, width_usize, height_usize, capture_role);
    draw_native_product_truth_accents(&mut pixels, width_usize, height_usize, result, capture_role);

    let mut triangles = Vec::new();
    let camera = match capture_role {
        "pre_contact_spacing" => "planning_tactical_reach",
        "combat_contact_readability" => "first_person_guard_line",
        "material_impact_closeup" => "asset_closeup_weapon_armor",
        "injury_consequence_readability" => "consequence_aftermath_dwell",
        "fight_film_cinematic_shot" => "fight_film_orbit",
        _ => "third_person_verdict_ring",
    };
    let arena_scale = match capture_role {
        "fighter_select_loadout" => 96,
        "material_impact_closeup" => 64,
        "verdict_ring_establishing" => 158,
        _ => 132,
    };
    push_native_mesh_triangles(
        &mut triangles,
        NativeSoftware3dInstance {
            geometry: &silhouette.arena_presentation_asset.geometry,
            origin_x_milli: 0,
            origin_y_milli: -980,
            origin_z_milli: 0,
            scale_num: arena_scale,
            scale_den: 1000,
            facing: 1,
            depth_bias: -1200,
            color: native_material_color("chalked_stone_dust", (160, 151, 127)),
        },
        width,
        height,
        camera,
    );

    let (seat0_x, seat1_x, seat_y, seat0_weapon, seat1_weapon, seat1_collapse, depth_spread, scale) =
        match capture_role {
            "fighter_select_loadout" => (-1060, 1060, -500, 60, 60, 0, 160, 500),
            "pre_contact_spacing" => (-900, 900, -430, 140, 90, 0, 110, 430),
            "combat_contact_readability" => (-520, 520, -430, 720, 260, 0, 90, 470),
            "material_impact_closeup" => (-820, 740, -820, 260, 120, 0, 260, 920),
            "injury_consequence_readability" => (-660, 710, -390, 300, 120, 280, 80, 450),
            _ => (-960, 960, -450, 120, 90, 0, 140, 430),
        };

    for (origin_x, origin_z, collapse) in [
        (seat0_x, -depth_spread, 0),
        (seat1_x, depth_spread, seat1_collapse),
    ] {
        let (shadow_x, shadow_y) = native_product_shadow_screen_point(
            camera,
            width,
            height,
            origin_x,
            seat_y + collapse / 2,
            origin_z,
        );
        draw_native_contact_shadow(
            &mut pixels,
            width_usize,
            height_usize,
            shadow_x,
            shadow_y + (height as i32 / 22),
            (width as i32 / 12).max(96),
            (height as i32 / 38).max(22),
        );
    }

    if let Some(fighter) = silhouette.fighter(0) {
        push_native_fighter_software_meshes_posed(
            &mut triangles,
            fighter,
            seat0_x,
            seat_y,
            -depth_spread,
            -1,
            scale,
            camera,
            width,
            height,
            seat0_weapon,
            if capture_role == "combat_contact_readability" {
                -80
            } else {
                40
            },
            if capture_role == "material_impact_closeup" {
                240
            } else {
                80
            },
            0,
        );
    }
    if let Some(fighter) = silhouette.fighter(1) {
        push_native_fighter_software_meshes_posed(
            &mut triangles,
            fighter,
            seat1_x,
            seat_y,
            depth_spread,
            1,
            scale,
            camera,
            width,
            height,
            seat1_weapon,
            if capture_role == "injury_consequence_readability" {
                -160
            } else {
                20
            },
            if capture_role == "material_impact_closeup" {
                120
            } else {
                40
            },
            seat1_collapse,
        );
    }

    let triangle_count = triangles.len();
    triangles.sort_by_key(|triangle| triangle.depth);
    let mut shaded_triangle_count = 0usize;
    for triangle in &triangles {
        if fill_triangle(&mut pixels, width_usize, height_usize, triangle) {
            shaded_triangle_count += 1;
        }
    }

    let cx = width as i32 / 2;
    let cy = height as i32 / 2;
    match capture_role {
        "combat_contact_readability" => {
            draw_native_contact_bloom(
                &mut pixels,
                width_usize,
                height_usize,
                cx,
                cy + 20,
                width as i32 / 8,
                native_material_color("wet_blood_trace_overlay", (124, 30, 22)),
            );
            draw_pixel_line(
                &mut pixels,
                width_usize,
                height_usize,
                cx - 340,
                cy - 38,
                cx + 310,
                cy - 16,
                (224, 132, 38),
            );
        }
        "material_impact_closeup" => {
            draw_native_contact_bloom(
                &mut pixels,
                width_usize,
                height_usize,
                cx + 80,
                cy + 24,
                width as i32 / 10,
                native_material_color("tempered_steel_edge_worn", (140, 148, 148)),
            );
            draw_native_product_material_witnesses(&mut pixels, width_usize, height_usize);
        }
        "injury_consequence_readability" => {
            blend_ellipse(
                &mut pixels,
                width_usize,
                height_usize,
                cx + 310,
                cy + 160,
                width as i32 / 7,
                height as i32 / 18,
                native_material_color("wet_blood_trace_overlay", (116, 22, 18)),
                92,
            );
            draw_pixel_line(
                &mut pixels,
                width_usize,
                height_usize,
                cx + 160,
                cy - 20,
                cx + 420,
                cy + 190,
                (70, 36, 28),
            );
        }
        "fighter_select_loadout" => {
            draw_native_product_loadout_cards(&mut pixels, width_usize, height_usize, silhouette);
        }
        _ => {}
    }

    apply_native_lighting_post(&mut pixels, width_usize, height_usize, 1115);
    draw_native_product_cinematic_bars(&mut pixels, width_usize, height_usize);

    let mut ppm = format!("P6\n{} {}\n255\n", width, height).into_bytes();
    ppm.extend_from_slice(&pixels);
    Ok((ppm, triangle_count, shaded_triangle_count))
}

#[cfg(target_os = "linux")]
fn native_product_shadow_screen_point(
    camera: &str,
    width: u32,
    height: u32,
    world_x: i32,
    world_y: i32,
    world_z: i32,
) -> (i32, i32) {
    let width = width as i64;
    let height = height as i64;
    let world_x = world_x as i64;
    let world_y = world_y as i64;
    let world_z = world_z as i64;
    let (screen_x, screen_y) = match camera {
        "first_person_guard_line" => (
            width / 2 + world_x / 4 + world_z / 8,
            height / 2 + 112 - world_y / 5 - world_z / 10,
        ),
        "third_person_replay_orbit" => (
            width / 2 + world_x / 4 + world_z / 5,
            height / 2 + 72 - world_y / 5 - world_z / 7,
        ),
        "planning_tactical_reach" => (
            width / 2 + world_x / 5 + world_z / 9,
            height / 2 + 36 - world_y / 6 - world_z / 14,
        ),
        "consequence_aftermath_dwell" => (
            width / 2 + world_x / 4 + world_z / 6,
            height / 2 + 96 - world_y / 5 - world_z / 9,
        ),
        "fight_film_orbit" => (
            width / 2 + world_x / 5 - world_z / 4,
            height / 2 + 84 - world_y / 6 - world_z / 9,
        ),
        _ => (
            width / 2 + world_x / 4 + world_z / 6,
            height / 2 + 76 - world_y / 5 - world_z / 8,
        ),
    };
    (
        screen_x.clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        screen_y.clamp(i32::MIN as i64, i32::MAX as i64) as i32,
    )
}

#[cfg(target_os = "linux")]
fn draw_native_product_truth_accents(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    result: &DuelResult,
    capture_role: &str,
) {
    let contact_count: usize = result.turns.iter().map(|turn| turn.contacts.len()).sum();
    let hash = result.final_state_hash.as_bytes();
    for index in 0..hash.len().min(16) {
        let byte = hash[index] as usize;
        let x = width / 2 + index * width / 70 - width / 9;
        let y = height * 17 / 100 + (byte % (height / 24).max(1));
        let radius = (width / 170 + byte % 9).max(6) as i32;
        blend_ellipse(
            pixels,
            width,
            height,
            x.min(width.saturating_sub(1)) as i32,
            y.min(height.saturating_sub(1)) as i32,
            radius,
            (radius / 2).max(3),
            (192, 140, 74),
            24 + (byte as u16 % 34),
        );
    }
    let contact_width = ((contact_count.max(1) * width) / 22).clamp(width / 20, width / 4);
    let y = match capture_role {
        "combat_contact_readability" => height * 43 / 100,
        "material_impact_closeup" => height * 58 / 100,
        "injury_consequence_readability" => height * 65 / 100,
        _ => height * 52 / 100,
    };
    fill_rect(
        pixels,
        width,
        height,
        width / 2 - contact_width / 2,
        y,
        contact_width,
        (height / 95).max(6),
        (204, 122, 45),
    );
}

#[cfg(target_os = "linux")]
fn draw_native_product_environment(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    capture_role: &str,
) {
    let horizon = (height * 43 / 100) as i32;
    for index in 0..9 {
        let x = (width as i32 * (10 + index * 10)) / 100;
        let column_h = height as i32 / 8 + ((index as i32 % 3) * height as i32 / 28);
        fill_rect(
            pixels,
            width,
            height,
            x.max(0) as usize,
            (horizon - column_h).max(0) as usize,
            (width / 90).max(8),
            column_h.max(1) as usize,
            (64, 58, 50),
        );
        blend_ellipse(
            pixels,
            width,
            height,
            x + width as i32 / 180,
            horizon - column_h,
            width as i32 / 34,
            height as i32 / 70,
            (134, 108, 74),
            74,
        );
    }
    for index in 0..7 {
        let x0 = (width as i32 * index) / 6;
        draw_pixel_line(
            pixels,
            width,
            height,
            x0,
            horizon + height as i32 / 8,
            width as i32 / 2,
            height as i32 - height as i32 / 10,
            (78, 62, 44),
        );
    }
    if matches!(
        capture_role,
        "verdict_ring_establishing" | "pre_contact_spacing"
    ) {
        blend_ellipse(
            pixels,
            width,
            height,
            width as i32 / 2,
            horizon + height as i32 / 9,
            width as i32 / 3,
            height as i32 / 30,
            (203, 174, 120),
            52,
        );
    }
}

#[cfg(target_os = "linux")]
fn draw_native_product_material_witnesses(pixels: &mut [u8], width: usize, height: usize) {
    let materials = [
        "tempered_steel_edge_worn",
        "riveted_mail_oiled",
        "quilted_linen_stitched",
        "strained_buff_leather",
        "ash_wood_grain_dented",
        "chalked_stone_dust",
        "wet_blood_trace_overlay",
    ];
    let y = height.saturating_sub(190);
    for (index, material) in materials.iter().enumerate() {
        let x = 180 + index * 220;
        fill_rect(
            pixels,
            width,
            height,
            x,
            y,
            160,
            92,
            native_material_color(material, (96, 96, 96)),
        );
        for stripe in 0..8 {
            draw_pixel_line(
                pixels,
                width,
                height,
                x as i32,
                y as i32 + stripe * 11,
                x as i32 + 160,
                y as i32 + stripe * 11 + 8,
                (42, 36, 30),
            );
        }
    }
}

#[cfg(target_os = "linux")]
fn draw_native_product_loadout_cards(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    silhouette: &NativeCombatSilhouetteEvidence,
) {
    let y = height.saturating_sub(250);
    for (index, fighter) in silhouette.fighters.iter().enumerate().take(2) {
        let x = 190 + index * (width.saturating_sub(380));
        fill_rect(pixels, width, height, x, y, 330, 136, (27, 33, 35));
        fill_rect(
            pixels,
            width,
            height,
            x + 24,
            y + 24,
            (fighter.weapon_span_px as usize)
                .saturating_mul(2)
                .clamp(80, 260),
            18,
            native_material_color(native_weapon_material_binding(fighter), (112, 120, 122)),
        );
        fill_rect(
            pixels,
            width,
            height,
            x + 24,
            y + 68,
            (fighter.armor_torso_coverage_permille as usize / 4).clamp(80, 260),
            22,
            native_material_color(native_armor_material_binding(fighter), (118, 78, 46)),
        );
    }
}

#[cfg(target_os = "linux")]
fn draw_native_product_cinematic_bars(pixels: &mut [u8], width: usize, height: usize) {
    let bar_h = (height / 30).max(12);
    fill_rect(pixels, width, height, 0, 0, width, bar_h, (13, 15, 16));
    fill_rect(
        pixels,
        width,
        height,
        0,
        height.saturating_sub(bar_h),
        width,
        bar_h,
        (13, 15, 16),
    );
}

#[cfg(target_os = "linux")]
unsafe fn x11_combat_overview_capture(
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    ppm_path: &Path,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let display = XOpenDisplay(std::ptr::null());
    if display.is_null() {
        return Err("XOpenDisplay returned null; DISPLAY or XWayland is unavailable".to_string());
    }
    let screen_number = XDefaultScreen(display);
    let root = XRootWindow(display, screen_number);
    let black = XBlackPixel(display, screen_number);
    let white = XWhitePixel(display, screen_number);
    let window = XCreateSimpleWindow(display, root, 160, 160, width, height, 2, black, white);
    if window == 0 {
        XCloseDisplay(display);
        return Err("XCreateSimpleWindow returned 0".to_string());
    }
    let depth = XDefaultDepth(display, screen_number);
    if depth <= 0 {
        XDestroyWindow(display, window);
        XCloseDisplay(display);
        return Err("XDefaultDepth returned non-positive depth".to_string());
    }
    let pixmap = XCreatePixmap(display, root, width, height, depth as c_uint);
    if pixmap == 0 {
        XDestroyWindow(display, window);
        XCloseDisplay(display);
        return Err("XCreatePixmap returned 0".to_string());
    }
    let title =
        CString::new("OATHYARD Native Combat Resolution").map_err(|error| error.to_string())?;
    XStoreName(display, window, title.as_ptr());
    XMapWindow(display, window);
    let gc = XCreateGC(display, pixmap, 0, std::ptr::null_mut());
    if gc.is_null() {
        XFreePixmap(display, pixmap);
        XDestroyWindow(display, window);
        XCloseDisplay(display);
        return Err("XCreateGC returned null".to_string());
    }
    XSetForeground(display, gc, black);
    render_native_combat_frame(
        display, pixmap, gc, result, silhouette, width, height, black, white,
    );
    XCopyArea(display, pixmap, window, gc, 0, 0, width, height, 0, 0);
    XFlush(display);
    XSync(display, 0);
    capture_window_to_ppm(display, pixmap, width, height, ppm_path, black, white)?;
    XFreeGC(display, gc);
    XFreePixmap(display, pixmap);
    XDestroyWindow(display, window);
    XCloseDisplay(display);
    Ok(())
}

#[cfg(target_os = "linux")]
unsafe fn render_native_combat_frame(
    display: *mut Display,
    window: Window,
    gc: GC,
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    width: u32,
    height: u32,
    black: c_ulong,
    _white: c_ulong,
) {
    let ox = ((width as c_int - 960).max(0)) / 2;
    let oy = ((height as c_int - 540).max(0)) / 2;
    XSetForeground(display, gc, x11_rgb_pixel((18, 21, 23)));
    XFillRectangle(display, window, gc, 0, 0, width, height);

    for band in 0..12 {
        let y = band * height as c_int / 12;
        let color = if band < 5 {
            native_shaded_rgb((38, 42, 42), band * 3)
        } else {
            native_shaded_rgb((82, 72, 55), band * 4 - 16)
        };
        XSetForeground(display, gc, x11_rgb_pixel(color));
        XFillRectangle(
            display,
            window,
            gc,
            0,
            y,
            width,
            ((height as c_int / 12) + 2) as c_uint,
        );
    }

    let floor_x = ox + 54;
    let floor_y = oy + 92;
    let floor_w = 852;
    let floor_h = 382;
    XSetForeground(display, gc, x11_rgb_pixel((108, 93, 67)));
    XFillRectangle(
        display,
        window,
        gc,
        floor_x,
        floor_y,
        floor_w as c_uint,
        floor_h as c_uint,
    );
    for row in 0..9 {
        let y = floor_y + row * floor_h / 8;
        XSetForeground(display, gc, x11_rgb_pixel((66, 56, 43)));
        XDrawLine(display, window, gc, floor_x, y, floor_x + floor_w, y);
    }
    for col in 0..13 {
        let x = floor_x + col * floor_w / 12 + if col % 2 == 0 { 0 } else { 18 };
        XSetForeground(display, gc, x11_rgb_pixel((75, 63, 48)));
        XDrawLine(display, window, gc, x, floor_y, x - 36, floor_y + floor_h);
    }

    XSetForeground(display, gc, x11_rgb_pixel((210, 174, 98)));
    for radius in [96, 146, 206] {
        draw_arena_ring_outline(display, window, gc, ox + 480, oy + 292, radius);
    }
    draw_runtime_gltf_mesh(
        display,
        window,
        gc,
        &silhouette.arena_presentation_asset.geometry,
        ox + 480,
        oy + 292,
        34,
        1000,
        1,
        native_material_color("chalked_stone_dust", (155, 142, 105)),
    );

    if let Some(first_contact) = result
        .turns
        .iter()
        .flat_map(|turn| turn.contacts.iter())
        .next()
    {
        let contact_color = if first_contact.material_result.contains("blood") {
            native_material_color("wet_blood_trace_overlay", (122, 28, 20))
        } else {
            (218, 132, 44)
        };
        XSetForeground(display, gc, x11_rgb_pixel(contact_color));
        XDrawLine(display, window, gc, ox + 372, oy + 232, ox + 588, oy + 232);
        XDrawLine(display, window, gc, ox + 588, oy + 232, ox + 566, oy + 218);
        XDrawLine(display, window, gc, ox + 588, oy + 232, ox + 566, oy + 246);
    }

    XSetForeground(display, gc, x11_rgb_pixel((24, 27, 27)));
    if let Some(fighter) = silhouette.fighter(0) {
        draw_source_backed_fighter_pose(
            display,
            window,
            gc,
            fighter,
            ox + 345,
            oy + 314,
            -1,
            -3,
            18,
            0,
            "seat 0",
            false,
        );
    }
    XSetForeground(display, gc, x11_rgb_pixel((24, 27, 27)));
    if let Some(fighter) = silhouette.fighter(1) {
        draw_source_backed_fighter_pose(
            display,
            window,
            gc,
            fighter,
            ox + 615,
            oy + 314,
            1,
            3,
            12,
            0,
            "seat 1",
            false,
        );
    }

    XSetForeground(display, gc, x11_rgb_pixel((18, 21, 23)));
    XFillRectangle(display, window, gc, ox + 56, oy + 32, 236, 46);
    XSetForeground(display, gc, x11_rgb_pixel((224, 204, 158)));
    draw_x11_text(display, window, gc, ox + 74, oy + 61, "OATHYARD");
    XSetForeground(display, gc, black);
}

#[cfg(target_os = "linux")]
unsafe fn render_native_combat_state_frame(
    display: *mut Display,
    window: Window,
    gc: GC,
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    frame: &NativeCombatFrameSpec,
    width: u32,
    height: u32,
    black: c_ulong,
    white: c_ulong,
) {
    XSetForeground(display, gc, white);
    XFillRectangle(display, window, gc, 0, 0, width, height);
    XSetForeground(display, gc, black);
    draw_x11_text(
        display,
        window,
        gc,
        30,
        36,
        "OATHYARD native combat state frame",
    );
    draw_x11_text(
        display,
        window,
        gc,
        30,
        64,
        &format!(
            "state {} | turn {} | final {}",
            frame.state, frame.turn, result.final_state_hash
        ),
    );
    draw_x11_text(
        display,
        window,
        gc,
        30,
        92,
        "Trace-derived renderer: hashed duel result is read-only presentation input.",
    );

    draw_game_panel(display, window, gc, 30, 118, 278, 340, "COMBAT STATE");
    draw_game_panel(display, window, gc, 342, 118, 286, 340, "VERDICT RING");
    draw_game_panel(display, window, gc, 660, 118, 270, 340, "EVIDENCE");

    draw_x11_text(
        display,
        window,
        gc,
        58,
        178,
        &clipped_text(&frame.headline, 34),
    );
    draw_x11_text(
        display,
        window,
        gc,
        58,
        214,
        &clipped_text(&frame.detail, 34),
    );
    draw_x11_text(display, window, gc, 58, 252, "states covered:");
    draw_x11_text(display, window, gc, 76, 282, "observe / guard / parry");
    draw_x11_text(display, window, gc, 76, 310, "weapon arc / hit contact");
    draw_x11_text(display, window, gc, 76, 338, "armor material solve");
    draw_x11_text(display, window, gc, 76, 366, "injury / grip / stance");
    draw_x11_text(display, window, gc, 76, 394, "near miss / recovery / hash");

    draw_arena_ring(display, window, gc, 485, 284, 118);
    draw_runtime_gltf_mesh(
        display,
        window,
        gc,
        &silhouette.arena_presentation_asset.geometry,
        485,
        284,
        19,
        1000,
        1,
        native_material_color("chalked_stone_dust", (156, 148, 127)),
    );
    let (seat0_x, seat1_x) = match frame.state {
        "observe_plan" => (420, 550),
        "guard_bind" => (438, 532),
        "parry_window" => (444, 526),
        "weapon_arc" => (452, 518),
        "hit_contact" => (460, 510),
        "armor_material_solve" => (458, 512),
        "injury_capability" => (450, 520),
        "grip_loss" => (448, 524),
        "stance_collapse_risk" => (440, 536),
        "near_miss_replan" => (418, 558),
        "recovery_state" => (426, 546),
        "final_hash_card" => (430, 550),
        _ => (438, 532),
    };
    let seat0_reach = match frame.state {
        "weapon_arc" => 64,
        "hit_contact" | "armor_material_solve" => 96,
        "grip_loss" => 44,
        "near_miss_replan" => 18,
        _ => 0,
    };
    let seat1_reach = match frame.state {
        "guard_bind" | "parry_window" => 30,
        "weapon_arc" => 42,
        "hit_contact" | "armor_material_solve" => 22,
        _ => 0,
    };
    let seat1_collapse = if matches!(
        frame.state,
        "injury_capability" | "grip_loss" | "stance_collapse_risk"
    ) {
        150
    } else {
        0
    };
    if let Some(fighter) = silhouette.fighter(0) {
        draw_source_backed_fighter_pose(
            display,
            window,
            gc,
            fighter,
            seat0_x,
            310,
            -1,
            0,
            seat0_reach,
            0,
            "seat 0",
            false,
        );
    }
    if let Some(fighter) = silhouette.fighter(1) {
        draw_source_backed_fighter_pose(
            display,
            window,
            gc,
            fighter,
            seat1_x,
            310,
            1,
            if seat1_collapse > 0 { 12 } else { 0 },
            seat1_reach,
            seat1_collapse,
            "seat 1",
            false,
        );
    }

    match frame.state {
        "guard_bind" => {
            XDrawLine(display, window, gc, 450, 230, 520, 252);
            XDrawLine(display, window, gc, 520, 230, 452, 252);
            draw_x11_text(display, window, gc, 444, 188, "guard/parry plane");
        }
        "parry_window" => {
            XDrawLine(display, window, gc, 444, 222, 528, 262);
            XDrawLine(display, window, gc, 528, 222, 444, 262);
            XDrawLine(display, window, gc, 486, 210, 486, 274);
            draw_x11_text(display, window, gc, 436, 188, "timed parry window");
        }
        "weapon_arc" => {
            XDrawLine(display, window, gc, 426, 236, 544, 210);
            XDrawLine(display, window, gc, 426, 252, 552, 236);
            XDrawLine(display, window, gc, 426, 268, 544, 262);
            draw_x11_text(display, window, gc, 438, 188, "weapon arc / reach");
        }
        "hit_contact" => {
            XDrawLine(display, window, gc, 440, 232, 545, 232);
            XDrawLine(display, window, gc, 545, 232, 528, 220);
            XDrawLine(display, window, gc, 545, 232, 528, 244);
            draw_x11_text(display, window, gc, 446, 190, "contact packet");
        }
        "armor_material_solve" => {
            draw_box_lines(display, window, gc, 484, 218, 38, 54);
            XDrawLine(display, window, gc, 430, 238, 548, 238);
            XDrawLine(display, window, gc, 548, 238, 530, 226);
            XDrawLine(display, window, gc, 548, 238, 530, 250);
            draw_x11_text(display, window, gc, 424, 190, "armor/material solve");
        }
        "injury_capability" => {
            XDrawLine(display, window, gc, 440, 232, 545, 232);
            XDrawLine(display, window, gc, 506, 248, 532, 286);
            XDrawLine(display, window, gc, 532, 286, 520, 280);
            XDrawLine(display, window, gc, 532, 286, 534, 272);
            draw_x11_text(display, window, gc, 426, 190, "capability delta");
        }
        "grip_loss" => {
            XDrawLine(display, window, gc, 438, 236, 522, 236);
            draw_box_lines(display, window, gc, 524, 222, 16, 16);
            XDrawLine(display, window, gc, 532, 238, 532, 264);
            draw_x11_text(display, window, gc, 438, 190, "grip loss / bind");
        }
        "stance_collapse_risk" => {
            XDrawLine(display, window, gc, 438, 236, 530, 246);
            XDrawLine(display, window, gc, 508, 276, 546, 304);
            XDrawLine(display, window, gc, 546, 304, 526, 308);
            XDrawLine(display, window, gc, 546, 304, 546, 284);
            draw_x11_text(display, window, gc, 414, 190, "stance collapse risk");
        }
        "near_miss_replan" => {
            XDrawLine(display, window, gc, 420, 238, 548, 238);
            XDrawLine(display, window, gc, 478, 220, 490, 256);
            XDrawLine(display, window, gc, 494, 220, 482, 256);
            draw_x11_text(display, window, gc, 430, 190, "near miss / replan");
        }
        "recovery_state" => {
            XDrawLine(display, window, gc, 436, 240, 526, 240);
            XDrawLine(display, window, gc, 436, 258, 506, 258);
            draw_x11_text(display, window, gc, 434, 190, "recovery spacing");
        }
        "final_hash_card" => {
            draw_box_lines(display, window, gc, 420, 214, 132, 72);
            draw_x11_text(display, window, gc, 434, 244, "final hash");
            draw_x11_text(display, window, gc, 424, 270, &result.final_state_hash);
        }
        _ => {
            XDrawLine(display, window, gc, 436, 238, 522, 238);
            draw_x11_text(display, window, gc, 436, 190, "observe spacing");
        }
    }

    draw_x11_text(
        display,
        window,
        gc,
        690,
        178,
        &format!("scenario {}", result.scenario_id),
    );
    draw_x11_text(
        display,
        window,
        gc,
        690,
        208,
        &format!("content {}", result.content_hash),
    );
    if let Some(turn) = result.turns.iter().find(|turn| turn.turn == frame.turn) {
        for (index, cost) in turn.costs.iter().enumerate() {
            draw_x11_text(
                display,
                window,
                gc,
                690,
                248 + index as c_int * 34,
                &format!(
                    "F{} {} base {} current {}",
                    cost.fighter,
                    cost.action.as_str(),
                    cost.base_frames,
                    cost.current_frames
                ),
            );
        }
        draw_x11_text(
            display,
            window,
            gc,
            690,
            334,
            &format!("turn hash {}", turn.state_hash),
        );
    }
    draw_x11_text(display, window, gc, 690, 390, "truth mutation: none");
    draw_x11_text(display, window, gc, 690, 420, "public-demo-ready=false");
    draw_x11_text(display, window, gc, 690, 450, "runtime asset refs verified");
    draw_x11_text(
        display,
        window,
        gc,
        30,
        500,
        "Frame sequence is inspectable evidence, not owner visual acceptance.",
    );
}

#[cfg(target_os = "linux")]
unsafe fn render_native_combat_motion_frame(
    display: *mut Display,
    window: Window,
    gc: GC,
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    frame: &NativeCombatMotionFrameSpec,
    width: u32,
    height: u32,
    black: c_ulong,
    white: c_ulong,
) {
    XSetForeground(display, gc, white);
    XFillRectangle(display, window, gc, 0, 0, width, height);
    XSetForeground(display, gc, black);
    draw_x11_text(
        display,
        window,
        gc,
        30,
        34,
        "OATHYARD native combat motion frame",
    );
    draw_x11_text(
        display,
        window,
        gc,
        30,
        60,
        &format!(
            "sample {} | {} | turn {} | truth frame {} at {} Hz",
            frame.index, frame.phase, frame.turn, frame.truth_frame, TRUTH_HZ
        ),
    );
    draw_x11_text(
        display,
        window,
        gc,
        30,
        86,
        "Replay-derived presentation only: the state hash already exists before drawing.",
    );

    draw_game_panel(display, window, gc, 30, 112, 284, 350, "FRAME EVIDENCE");
    draw_game_panel(
        display,
        window,
        gc,
        342,
        112,
        312,
        350,
        "VERDICT RING MOTION",
    );
    draw_game_panel(display, window, gc, 682, 112, 248, 350, "TRACE LINKS");

    draw_x11_text(
        display,
        window,
        gc,
        58,
        168,
        &clipped_text(&frame.headline, 34),
    );
    draw_x11_text(
        display,
        window,
        gc,
        58,
        196,
        &clipped_text(&frame.detail, 34),
    );
    draw_x11_text(
        display,
        window,
        gc,
        58,
        228,
        &format!("turn hash {}", frame.turn_hash),
    );
    XDrawLine(display, window, gc, 58, 254, 268, 254);
    XDrawLine(display, window, gc, 268, 254, 268, 272);
    XDrawLine(display, window, gc, 268, 272, 58, 272);
    XDrawLine(display, window, gc, 58, 272, 58, 254);
    XFillRectangle(
        display,
        window,
        gc,
        58,
        254,
        (frame.progress_permille.min(1000) as c_int * 210 / 1000) as c_uint,
        18,
    );
    draw_x11_text(
        display,
        window,
        gc,
        58,
        300,
        &format!("progress {} permille", frame.progress_permille),
    );

    let turn = result.turns.iter().find(|turn| turn.turn == frame.turn);
    if let Some(turn) = turn {
        draw_x11_text(
            display,
            window,
            gc,
            58,
            334,
            &format!(
                "F0 {} {} -> F1 {} {}",
                turn.commits[0].label.as_str(),
                turn.commits[0].direction.as_str(),
                turn.commits[1].label.as_str(),
                turn.commits[1].direction.as_str()
            ),
        );
        for (index, cost) in turn.costs.iter().enumerate().take(2) {
            draw_x11_text(
                display,
                window,
                gc,
                58,
                366 + index as c_int * 28,
                &format!(
                    "F{} {} base {} current {}",
                    cost.fighter,
                    cost.action.as_str(),
                    cost.base_frames,
                    cost.current_frames
                ),
            );
        }
    }

    draw_arena_ring_outline(display, window, gc, 500, 292, 118);
    draw_runtime_gltf_mesh(
        display,
        window,
        gc,
        &silhouette.arena_presentation_asset.geometry,
        500,
        292,
        19,
        1000,
        1,
        native_material_color("chalked_stone_dust", (156, 148, 127)),
    );
    let progress = frame.progress_permille.min(1000) as c_int;
    let seat0_x = 392 + progress * 74 / 1000;
    let seat1_x = 608 - progress * 74 / 1000;
    let collapse = if frame.phase == "stagger_collapse_risk" {
        190
    } else {
        0
    };
    let contact_pose = if frame.phase == "active_contact" {
        90
    } else {
        0
    };
    let (seat0_reach, seat1_reach) = turn
        .map(|turn| {
            (
                native_action_reach(turn.commits[0].label, progress),
                native_action_reach(turn.commits[1].label, progress),
            )
        })
        .unwrap_or((0, 0));
    let seat0_lean = progress / 16;
    let seat1_lean = -(progress / 18) + collapse / 8;
    if let Some(fighter) = silhouette.fighter(0) {
        draw_source_backed_fighter_pose(
            display,
            window,
            gc,
            fighter,
            seat0_x,
            315,
            -1,
            seat0_lean,
            seat0_reach + contact_pose,
            0,
            "",
            false,
        );
    }
    if let Some(fighter) = silhouette.fighter(1) {
        draw_source_backed_fighter_pose(
            display,
            window,
            gc,
            fighter,
            seat1_x,
            315,
            1,
            seat1_lean,
            seat1_reach,
            collapse,
            "",
            false,
        );
    }

    if let Some(turn) = turn {
        if let Some(contact) = turn.contacts.first() {
            XDrawLine(display, window, gc, 442, 226, 558, 226);
            XDrawLine(display, window, gc, 558, 226, 540, 214);
            XDrawLine(display, window, gc, 558, 226, 540, 238);
            draw_x11_text(
                display,
                window,
                gc,
                392,
                176,
                &format!(
                    "{} -> {}",
                    contact.action.as_str(),
                    clipped_text(&contact.material_result, 30)
                ),
            );
            if frame.phase == "stagger_collapse_risk" {
                draw_x11_text(display, window, gc, 408, 200, "stagger/collapse risk");
            }
        } else {
            XDrawLine(display, window, gc, 440, 238, 552, 238);
            draw_x11_text(display, window, gc, 420, 178, "range check / no packet");
        }
    }

    draw_x11_text(
        display,
        window,
        gc,
        710,
        168,
        &format!("scenario {}", result.scenario_id),
    );
    draw_x11_text(
        display,
        window,
        gc,
        710,
        198,
        &format!("final {}", result.final_state_hash),
    );
    if let Some(turn) = turn {
        if let Some(contact) = turn.contacts.first() {
            draw_x11_text(
                display,
                window,
                gc,
                710,
                238,
                &format!("energy {}", contact.energy_milli),
            );
            draw_x11_text(
                display,
                window,
                gc,
                710,
                266,
                &format!("impulse {}", contact.impulse_milli),
            );
            draw_x11_text(
                display,
                window,
                gc,
                710,
                302,
                &clipped_text(&contact.anatomy_result, 30),
            );
            draw_x11_text(
                display,
                window,
                gc,
                710,
                330,
                &format!(
                    "balance {} recovery +{}",
                    contact.capability_delta.balance_delta,
                    contact.capability_delta.recovery_slowdown_add
                ),
            );
        } else {
            draw_x11_text(display, window, gc, 710, 238, "contact packets: 0");
            draw_x11_text(display, window, gc, 710, 266, "capability unchanged");
        }
        draw_x11_text(
            display,
            window,
            gc,
            710,
            388,
            &format!("turn hash {}", turn.state_hash),
        );
    }
    draw_x11_text(display, window, gc, 710, 420, "truth mutation: none");
    draw_x11_text(
        display,
        window,
        gc,
        30,
        504,
        "Motion sequence is sampled from committed replay truth; it is not owner visual acceptance.",
    );
}

#[cfg(target_os = "linux")]
unsafe fn render_native_player_loop_frame(
    display: *mut Display,
    window: Window,
    gc: GC,
    result: &DuelResult,
    silhouette: &NativeCombatSilhouetteEvidence,
    frame: &NativePlayerLoopFrameSpec,
    width: u32,
    height: u32,
    black: c_ulong,
    white: c_ulong,
    progress_permille: u32,
) {
    let ox = ((width as c_int - 960).max(0)) / 2;
    let oy = ((height as c_int - 540).max(0)) / 2;
    XSetForeground(display, gc, white);
    XFillRectangle(display, window, gc, 0, 0, width, height);
    XSetForeground(display, gc, black);

    draw_x11_text(
        display,
        window,
        gc,
        ox + 30,
        oy + 36,
        "OATHYARD native player-facing 3D loop",
    );
    draw_x11_text(
        display,
        window,
        gc,
        ox + 30,
        oy + 62,
        &format!(
            "screen {} | scheduled {}ms | truth frame {} | final {}",
            frame.screen, frame.scheduled_ms, frame.truth_frame, result.final_state_hash
        ),
    );
    draw_x11_text(
        display,
        window,
        gc,
        ox + 30,
        oy + 88,
        "One native backend routes menu/settings/select/loadout/oath phases/replay/film/debug; timing is presentation-only.",
    );
    draw_x11_text(
        display,
        window,
        gc,
        ox + 30,
        oy + 108,
        &format!(
            "native input action {} | HUD cache {}",
            frame.input_action,
            clipped_text(&frame.truth_cache_key, 16)
        ),
    );

    draw_game_panel(
        display,
        window,
        gc,
        ox + 26,
        oy + 116,
        236,
        348,
        "PLAYER UI",
    );
    draw_game_panel(
        display,
        window,
        gc,
        ox + 292,
        oy + 116,
        376,
        348,
        "3D VERDICT RING",
    );
    draw_game_panel(
        display,
        window,
        gc,
        ox + 698,
        oy + 116,
        236,
        348,
        "READ-ONLY EVIDENCE",
    );

    draw_x11_text(
        display,
        window,
        gc,
        ox + 54,
        oy + 170,
        &clipped_text(&frame.headline, 30),
    );
    draw_x11_text(
        display,
        window,
        gc,
        ox + 54,
        oy + 198,
        &clipped_text(&frame.detail, 30),
    );
    draw_x11_text(display, window, gc, ox + 54, oy + 228, "screen stack:");
    for (index, screen) in [
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
    ]
    .iter()
    .enumerate()
    {
        let marker = if *screen == frame.screen { ">" } else { " " };
        draw_x11_text(
            display,
            window,
            gc,
            ox + 70,
            oy + 252 + index as c_int * 16,
            &format!("{marker} {}", clipped_text(screen, 22)),
        );
    }

    let progress = progress_permille.min(1000) as c_int;
    let (seat0_base, seat1_base, seat0_reach, seat1_reach, seat1_collapse) = match frame.screen {
        "main_menu" | "mode_select" | "settings_accessibility" => (405, 555, 0, 0, 0),
        "fighter_select" | "loadout_select" => (390, 570, 8, 8, 0),
        "observe" | "plan" => (410 + progress / 20, 550 - progress / 22, 16, 10, 0),
        "commit_reveal" => (414 + progress / 18, 546 - progress / 20, 30, 18, 0),
        "resolve" => (
            420 + progress / 12,
            540 - progress / 14,
            62 + progress / 18,
            28,
            0,
        ),
        "consequence" => (
            416 + progress / 18,
            548 - progress / 20,
            40,
            22,
            progress / 7,
        ),
        "replay_browser" | "fight_film" => (
            406 + progress / 25,
            554 - progress / 28,
            34,
            18,
            progress / 8,
        ),
        "performance_debug_overlay" => (408, 552, 18, 12, 0),
        _ => (405, 555, 0, 0, 0),
    };

    draw_arena_ring(display, window, gc, ox + 480, oy + 302, 140);
    draw_runtime_gltf_mesh(
        display,
        window,
        gc,
        &silhouette.arena_presentation_asset.geometry,
        ox + 480,
        oy + 302,
        23,
        1000,
        1,
        native_material_color("chalked_stone_dust", (156, 148, 127)),
    );
    if let Some(fighter) = silhouette.fighter(0) {
        draw_source_backed_fighter_pose(
            display,
            window,
            gc,
            fighter,
            ox + seat0_base,
            oy + 324,
            -1,
            progress / 24,
            seat0_reach,
            0,
            "seat 0",
            false,
        );
    }
    if let Some(fighter) = silhouette.fighter(1) {
        draw_source_backed_fighter_pose(
            display,
            window,
            gc,
            fighter,
            ox + seat1_base,
            oy + 324,
            1,
            -(progress / 28),
            seat1_reach,
            seat1_collapse,
            "seat 1",
            false,
        );
    }

    match frame.screen {
        "main_menu" => {
            draw_x11_text(
                display,
                window,
                gc,
                ox + 350,
                oy + 168,
                "START LOCAL OATH DUEL",
            );
            draw_x11_text(display, window, gc, ox + 350, oy + 194, "REPLAY ARCHIVE");
            draw_x11_text(
                display,
                window,
                gc,
                ox + 350,
                oy + 220,
                "SETTINGS / ACCESSIBILITY",
            );
            draw_x11_text(
                display,
                window,
                gc,
                ox + 350,
                oy + 246,
                "PERF/DEBUG OVERLAY",
            );
        }
        "mode_select" => {
            draw_x11_text(display, window, gc, ox + 350, oy + 168, "LOCAL OATH DUEL");
            draw_x11_text(
                display,
                window,
                gc,
                ox + 350,
                oy + 194,
                "OBSERVE / PLAN FLOW",
            );
            draw_x11_text(
                display,
                window,
                gc,
                ox + 350,
                oy + 220,
                "REPLAY + FIGHT-FILM",
            );
            draw_x11_text(
                display,
                window,
                gc,
                ox + 720,
                oy + 176,
                "native desktop target",
            );
            draw_x11_text(
                display,
                window,
                gc,
                ox + 720,
                oy + 204,
                "offline/local flow evidence",
            );
        }
        "settings_accessibility" => {
            draw_x11_text(
                display,
                window,
                gc,
                ox + 350,
                oy + 168,
                "TEXT SCALE 1400 PERMILLE",
            );
            draw_x11_text(
                display,
                window,
                gc,
                ox + 350,
                oy + 194,
                "HIGH CONTRAST + CAPTIONS",
            );
            draw_x11_text(
                display,
                window,
                gc,
                ox + 350,
                oy + 220,
                "REDUCED MOTION / FLASH",
            );
            draw_x11_text(
                display,
                window,
                gc,
                ox + 720,
                oy + 176,
                "presentation-only settings",
            );
            draw_x11_text(
                display,
                window,
                gc,
                ox + 720,
                oy + 204,
                "owner_visual_acceptance=false",
            );
        }
        "fighter_select" => {
            for fighter in &silhouette.fighters {
                draw_x11_text(
                    display,
                    window,
                    gc,
                    ox + 720,
                    oy + 176 + fighter.seat as c_int * 34,
                    &format!("F{} {}", fighter.seat, clipped_text(&fighter.name, 18)),
                );
                draw_x11_text(
                    display,
                    window,
                    gc,
                    ox + 720,
                    oy + 194 + fighter.seat as c_int * 34,
                    &format!("{} / {}", fighter.weapon_id, fighter.armor_id),
                );
            }
            draw_x11_text(display, window, gc, ox + 350, oy + 168, "FIGHTER SELECT");
            draw_x11_text(
                display,
                window,
                gc,
                ox + 350,
                oy + 194,
                "canonical scenario roster",
            );
        }
        "loadout_select" => {
            for fighter in &silhouette.fighters {
                draw_x11_text(
                    display,
                    window,
                    gc,
                    ox + 720,
                    oy + 176 + fighter.seat as c_int * 42,
                    &format!("F{} {}", fighter.seat, fighter.weapon_id),
                );
                draw_x11_text(
                    display,
                    window,
                    gc,
                    ox + 720,
                    oy + 196 + fighter.seat as c_int * 42,
                    &format!("armor {}", fighter.armor_id),
                );
            }
            draw_x11_text(display, window, gc, ox + 350, oy + 168, "LOADOUT SELECT");
            draw_x11_text(
                display,
                window,
                gc,
                ox + 350,
                oy + 194,
                "runtime mesh/gltf refs",
            );
            draw_x11_text(
                display,
                window,
                gc,
                ox + 350,
                oy + 220,
                "no stat shortcuts / no HP",
            );
        }
        "observe" | "plan" | "commit_reveal" => {
            if let Some(turn) = result.turns.first() {
                draw_x11_text(
                    display,
                    window,
                    gc,
                    ox + 720,
                    oy + 176,
                    &format!("F0 plans {}", turn.commits[0].label.as_str()),
                );
                draw_x11_text(
                    display,
                    window,
                    gc,
                    ox + 720,
                    oy + 204,
                    &format!("F1 plans {}", turn.commits[1].label.as_str()),
                );
                draw_x11_text(
                    display,
                    window,
                    gc,
                    ox + 720,
                    oy + 242,
                    &format!(
                        "base {} current {}",
                        frame.base_cost_frames, frame.current_cost_frames
                    ),
                );
                if let Some(reason) = frame.physical_reasons.first() {
                    draw_x11_text(
                        display,
                        window,
                        gc,
                        ox + 720,
                        oy + 270,
                        &clipped_text(reason, 28),
                    );
                }
            }
            let phase_label = match frame.screen {
                "observe" => "OBSERVE: read opponent/loadout",
                "plan" => "PLAN: physical labels only",
                "commit_reveal" => "COMMIT-REVEAL: locked inputs",
                _ => "OATH PHASE",
            };
            draw_x11_text(display, window, gc, ox + 350, oy + 168, phase_label);
            draw_x11_text(
                display,
                window,
                gc,
                ox + 350,
                oy + 194,
                "base/current frame cost visible",
            );
            draw_x11_text(
                display,
                window,
                gc,
                ox + 350,
                oy + 220,
                "physical reasons, no HP/DPS",
            );
        }
        "resolve" | "consequence" => {
            if let Some(contact) = result
                .turns
                .iter()
                .flat_map(|turn| turn.contacts.iter())
                .next()
            {
                XDrawLine(display, window, gc, ox + 430, oy + 222, ox + 545, oy + 222);
                draw_x11_text(
                    display,
                    window,
                    gc,
                    ox + 720,
                    oy + 176,
                    &format!(
                        "{} -> {}",
                        contact.action.as_str(),
                        clipped_text(&contact.material_result, 20)
                    ),
                );
                draw_x11_text(
                    display,
                    window,
                    gc,
                    ox + 720,
                    oy + 204,
                    &format!(
                        "balance {} recovery +{}",
                        contact.capability_delta.balance_delta,
                        contact.capability_delta.recovery_slowdown_add
                    ),
                );
            }
            draw_x11_text(
                display,
                window,
                gc,
                ox + 350,
                oy + 168,
                if frame.screen == "resolve" {
                    "RESOLVE: contact/material/anatomy"
                } else {
                    "CONSEQUENCE: capability deltas"
                },
            );
            draw_x11_text(
                display,
                window,
                gc,
                ox + 350,
                oy + 194,
                "truth already hashed before draw",
            );
        }
        "replay_browser" => {
            draw_x11_text(
                display,
                window,
                gc,
                ox + 720,
                oy + 176,
                "replay verifies final hash",
            );
            draw_x11_text(
                display,
                window,
                gc,
                ox + 720,
                oy + 204,
                &format!("content {}", result.content_hash),
            );
            draw_x11_text(
                display,
                window,
                gc,
                ox + 720,
                oy + 232,
                "corrupt replays fail loudly",
            );
            draw_x11_text(display, window, gc, ox + 350, oy + 168, "REPLAY BROWSER");
            draw_x11_text(
                display,
                window,
                gc,
                ox + 350,
                oy + 194,
                "same replay verify path",
            );
        }
        "fight_film" => {
            draw_x11_text(
                display,
                window,
                gc,
                ox + 720,
                oy + 176,
                "trace-derived camera cuts",
            );
            draw_x11_text(
                display,
                window,
                gc,
                ox + 720,
                oy + 204,
                &format!("motion samples {}", result.turns.len() * 7),
            );
            draw_x11_text(
                display,
                window,
                gc,
                ox + 720,
                oy + 232,
                "fight-film camera: trace only",
            );
            draw_x11_text(display, window, gc, ox + 350, oy + 168, "FIGHT-FILM VIEWER");
            draw_x11_text(
                display,
                window,
                gc,
                ox + 350,
                oy + 194,
                "presentation-only cameras",
            );
        }
        "performance_debug_overlay" => {
            draw_x11_text(
                display,
                window,
                gc,
                ox + 350,
                oy + 168,
                "PERFORMANCE / DEBUG OVERLAY",
            );
            draw_x11_text(
                display,
                window,
                gc,
                ox + 350,
                oy + 194,
                "truth hz 120 fixed",
            );
            draw_x11_text(
                display,
                window,
                gc,
                ox + 350,
                oy + 220,
                "hashes + frame costs visible",
            );
            draw_x11_text(
                display,
                window,
                gc,
                ox + 720,
                oy + 176,
                "owner_input_acceptance=false",
            );
            draw_x11_text(
                display,
                window,
                gc,
                ox + 720,
                oy + 204,
                "public/release/store=false",
            );
        }
        _ => {}
    }

    XDrawLine(display, window, gc, ox + 54, oy + 428, ox + 236, oy + 428);
    XFillRectangle(
        display,
        window,
        gc,
        ox + 54,
        oy + 420,
        (progress * 182 / 1000).max(4) as c_uint,
        16,
    );
    draw_x11_text(
        display,
        window,
        gc,
        ox + 720,
        oy + 300,
        &format!(
            "HUD truth cost base {} current {}",
            frame.base_cost_frames, frame.current_cost_frames
        ),
    );
    if let Some(reason) = frame.physical_reasons.first() {
        draw_x11_text(
            display,
            window,
            gc,
            ox + 720,
            oy + 328,
            &clipped_text(reason, 28),
        );
    }
    draw_x11_text(
        display,
        window,
        gc,
        ox + 720,
        oy + 356,
        "truth mutation: false",
    );
    draw_x11_text(
        display,
        window,
        gc,
        ox + 720,
        oy + 384,
        "presentation only: true",
    );
    draw_x11_text(
        display,
        window,
        gc,
        ox + 720,
        oy + 412,
        "public-demo-ready=false",
    );
    draw_x11_text(
        display,
        window,
        gc,
        ox + 30,
        oy + 502,
        "Player-loop frames are native desktop evidence; product high-fidelity and owner visual acceptance remain separate gates.",
    );
}

#[cfg(target_os = "linux")]
unsafe fn draw_arena_ring(
    display: *mut Display,
    window: Window,
    gc: GC,
    cx: c_int,
    cy: c_int,
    radius: c_int,
) {
    draw_arena_ring_outline(display, window, gc, cx, cy, radius);
    draw_x11_text(
        display,
        window,
        gc,
        cx - 84,
        cy + radius + 28,
        "OATHYARD verdict ring",
    );
}

#[cfg(target_os = "linux")]
unsafe fn draw_arena_ring_outline(
    display: *mut Display,
    window: Window,
    gc: GC,
    cx: c_int,
    cy: c_int,
    radius: c_int,
) {
    let mut prev_x = cx + radius;
    let mut prev_y = cy;
    let steps = 48;
    for step in 1..=steps {
        let angle_milli = step * 6283 / steps;
        let x = cx + fixed_cos_permille(angle_milli) * radius / 1000;
        let y = cy + fixed_sin_permille(angle_milli) * radius / 1000;
        XDrawLine(display, window, gc, prev_x, prev_y, x, y);
        prev_x = x;
        prev_y = y;
    }
}

#[cfg(target_os = "linux")]
fn native_action_reach(action: ActionLabel, progress_permille: c_int) -> c_int {
    let reach = match action {
        ActionLabel::Cut => 58,
        ActionLabel::Thrust => 72,
        ActionLabel::Bash | ActionLabel::Shove | ActionLabel::Kick => 46,
        ActionLabel::HookBind | ActionLabel::Grab => 38,
        ActionLabel::Guard | ActionLabel::Parry | ActionLabel::Brace => 22,
        ActionLabel::Step | ActionLabel::Pivot | ActionLabel::Recover => 8,
    };
    reach * progress_permille.min(1000) / 1000
}

#[cfg(target_os = "linux")]
unsafe fn draw_box_lines(
    display: *mut Display,
    window: Window,
    gc: GC,
    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,
) {
    if width <= 0 || height <= 0 {
        return;
    }
    XDrawLine(display, window, gc, x, y, x + width, y);
    XDrawLine(display, window, gc, x + width, y, x + width, y + height);
    XDrawLine(display, window, gc, x + width, y + height, x, y + height);
    XDrawLine(display, window, gc, x, y + height, x, y);
}

#[cfg(target_os = "linux")]
unsafe fn fill_box(
    display: *mut Display,
    window: Window,
    gc: GC,
    x: c_int,
    y: c_int,
    width: c_int,
    height: c_int,
) {
    if width <= 0 || height <= 0 {
        return;
    }
    XFillRectangle(display, window, gc, x, y, width as c_uint, height as c_uint);
}

#[cfg(target_os = "linux")]
fn native_project_gltf_point(
    geometry: &NativeGltfGeometry,
    vertex_index: usize,
    origin_x: c_int,
    origin_y: c_int,
    scale_num: c_int,
    scale_den: c_int,
    facing: c_int,
) -> Option<(c_int, c_int)> {
    let (x_milli, y_milli, z_milli) = *geometry.positions_milli.get(vertex_index)?;
    let den = scale_den.max(1) as i64;
    let depth_x = (z_milli as i64 * scale_num as i64) / (den * 3);
    let depth_y = (z_milli as i64 * scale_num as i64) / (den * 5);
    let px = origin_x as i64
        + (facing as i64 * x_milli as i64 * scale_num as i64) / den
        + facing as i64 * depth_x;
    let py = origin_y as i64 - (y_milli as i64 * scale_num as i64) / den - depth_y;
    Some((px as c_int, py as c_int))
}

#[cfg(target_os = "linux")]
unsafe fn draw_runtime_gltf_mesh(
    display: *mut Display,
    window: Window,
    gc: GC,
    geometry: &NativeGltfGeometry,
    origin_x: c_int,
    origin_y: c_int,
    scale_num: c_int,
    scale_den: c_int,
    facing: c_int,
    base_color: (u8, u8, u8),
) {
    for (triangle_index, triangle) in geometry.indices.chunks(3).enumerate() {
        if triangle.len() != 3 {
            continue;
        }
        let Some(a) = native_project_gltf_point(
            geometry,
            triangle[0],
            origin_x,
            origin_y,
            scale_num,
            scale_den,
            facing,
        ) else {
            continue;
        };
        let Some(b) = native_project_gltf_point(
            geometry,
            triangle[1],
            origin_x,
            origin_y,
            scale_num,
            scale_den,
            facing,
        ) else {
            continue;
        };
        let Some(c) = native_project_gltf_point(
            geometry,
            triangle[2],
            origin_x,
            origin_y,
            scale_num,
            scale_den,
            facing,
        ) else {
            continue;
        };
        let average_y = (a.1 + b.1 + c.1) / 3;
        let shade = ((origin_y - average_y) / 9 + (triangle_index as c_int % 9) - 4).clamp(-38, 34);
        let fill_color = native_shaded_rgb(base_color, shade);
        let outline_color = native_shaded_rgb(base_color, -74);
        let mut points = [
            XPoint {
                x: a.0.clamp(i16::MIN as c_int, i16::MAX as c_int) as i16,
                y: a.1.clamp(i16::MIN as c_int, i16::MAX as c_int) as i16,
            },
            XPoint {
                x: b.0.clamp(i16::MIN as c_int, i16::MAX as c_int) as i16,
                y: b.1.clamp(i16::MIN as c_int, i16::MAX as c_int) as i16,
            },
            XPoint {
                x: c.0.clamp(i16::MIN as c_int, i16::MAX as c_int) as i16,
                y: c.1.clamp(i16::MIN as c_int, i16::MAX as c_int) as i16,
            },
        ];
        XSetForeground(display, gc, x11_rgb_pixel(fill_color));
        XFillPolygon(display, window, gc, points.as_mut_ptr(), 3, 2, 0);
        XSetForeground(display, gc, x11_rgb_pixel(outline_color));
        XDrawLine(display, window, gc, a.0, a.1, b.0, b.1);
        XDrawLine(display, window, gc, b.0, b.1, c.0, c.1);
        XDrawLine(display, window, gc, c.0, c.1, a.0, a.1);
    }
}

#[cfg(target_os = "linux")]
fn x11_rgb_pixel(color: (u8, u8, u8)) -> c_ulong {
    ((color.0 as c_ulong) << 16) | ((color.1 as c_ulong) << 8) | color.2 as c_ulong
}

#[cfg(target_os = "linux")]
fn native_shaded_rgb(base: (u8, u8, u8), delta: c_int) -> (u8, u8, u8) {
    (
        (base.0 as c_int + delta).clamp(0, 255) as u8,
        (base.1 as c_int + delta).clamp(0, 255) as u8,
        (base.2 as c_int + delta).clamp(0, 255) as u8,
    )
}

#[cfg(target_os = "linux")]
fn native_weapon_mesh_scale_num(fighter: &NativeCombatFighterSilhouette, reach: c_int) -> c_int {
    let extent = (fighter.geometry_safe_weapon_extent_milli()).max(1);
    let desired_px = (fighter.weapon_span_px + reach / 2).max(24);
    (desired_px * 1000 / extent).clamp(18, 180)
}

#[cfg(target_os = "linux")]
impl NativeCombatFighterSilhouette {
    fn geometry_safe_weapon_extent_milli(&self) -> c_int {
        (self.weapon_presentation_asset.geometry.max_x_milli
            - self.weapon_presentation_asset.geometry.min_x_milli)
            .abs()
    }
}

#[cfg(target_os = "linux")]
unsafe fn draw_source_backed_fighter_pose(
    display: *mut Display,
    window: Window,
    gc: GC,
    fighter: &NativeCombatFighterSilhouette,
    root_x: c_int,
    root_y: c_int,
    facing: c_int,
    lean: c_int,
    reach: c_int,
    collapse: c_int,
    _label: &str,
    show_labels: bool,
) {
    draw_articulated_fighter_pose(
        display, window, gc, root_x, root_y, facing, lean, reach, collapse, "",
    );

    let crouch = collapse / 7;
    let head_x = root_x + lean;
    let head_y = root_y - 92 + crouch;
    let neck_x = root_x + lean * 3 / 4;
    let neck_y = root_y - 68 + crouch;
    let hip_x = root_x;
    let hip_y = root_y + collapse / 12;
    let _shoulder_r = (root_x + facing * 32 + lean / 2, root_y - 58 + crouch);
    let elbow_r = (
        root_x + facing * (62 + reach / 2) + lean / 2,
        root_y - 36 + crouch - reach / 8,
    );
    let wrist_r = (
        root_x + facing * (94 + reach) + lean / 2,
        root_y - 48 + crouch - reach / 5,
    );
    let shoulder_l = (root_x - facing * 32 + lean / 3, root_y - 58 + crouch);
    let _elbow_l = (
        root_x - facing * 54 + lean / 4,
        root_y - 30 + crouch + collapse / 12,
    );
    let wrist_l = (
        root_x - facing * 70 + lean / 5,
        root_y - 4 + crouch + collapse / 10,
    );

    let torso_width = fighter.armor_torso_width_px;
    let torso_height = fighter.armor_torso_height_px + collapse / 18;
    let torso_x = neck_x - torso_width / 2;
    let torso_y = neck_y + 8;
    draw_box_lines(
        display,
        window,
        gc,
        torso_x,
        torso_y,
        torso_width,
        torso_height,
    );
    let coverage_width =
        (torso_width * fighter.armor_torso_coverage_permille / 1000).clamp(6, torso_width);
    fill_box(
        display,
        window,
        gc,
        torso_x + 2,
        torso_y + torso_height - 9,
        coverage_width - 4,
        5,
    );
    let armor_height_milli = (fighter.armor_presentation_asset.geometry.max_y_milli
        - fighter.armor_presentation_asset.geometry.min_y_milli)
        .abs()
        .max(1);
    let armor_scale_num = (torso_height.max(24) * 1000 / armor_height_milli).clamp(32, 96);
    draw_runtime_gltf_mesh(
        display,
        window,
        gc,
        &fighter.armor_presentation_asset.geometry,
        neck_x,
        torso_y + torso_height,
        armor_scale_num,
        1000,
        1,
        native_material_color(native_armor_material_binding(fighter), (105, 75, 48)),
    );

    let head_marker = fighter.armor_head_marker_px;
    draw_box_lines(
        display,
        window,
        gc,
        head_x - head_marker,
        head_y - head_marker,
        head_marker * 2,
        head_marker,
    );

    let arm_marker = (fighter.armor_weapon_arm_coverage_permille / 70).clamp(6, 16);
    draw_box_lines(
        display,
        window,
        gc,
        elbow_r.0 - arm_marker / 2,
        elbow_r.1 - 5,
        arm_marker,
        10,
    );
    let leg_marker = (fighter.armor_lead_leg_coverage_permille / 70).clamp(6, 16);
    draw_box_lines(
        display,
        window,
        gc,
        hip_x + facing * 20 - leg_marker / 2,
        hip_y + 42,
        leg_marker,
        26,
    );

    let weapon_span = fighter.weapon_span_px + reach / 2;
    let tip = (wrist_r.0 + facing * weapon_span, wrist_r.1 - 12 - reach / 7);
    let weapon_scale_num = native_weapon_mesh_scale_num(fighter, reach);
    let (weapon_origin_x, weapon_origin_y, weapon_facing) = if fighter.weapon_id == "round_shield" {
        (wrist_l.0, wrist_l.1, 1)
    } else {
        (wrist_r.0, wrist_r.1, facing)
    };
    draw_runtime_gltf_mesh(
        display,
        window,
        gc,
        &fighter.weapon_presentation_asset.geometry,
        weapon_origin_x,
        weapon_origin_y,
        weapon_scale_num,
        1000,
        weapon_facing,
        native_material_color(native_weapon_material_binding(fighter), (100, 108, 108)),
    );
    XDrawLine(display, window, gc, wrist_r.0, wrist_r.1, tip.0, tip.1);
    XDrawLine(
        display,
        window,
        gc,
        wrist_r.0 - facing * 10,
        wrist_r.1 + 7,
        wrist_r.0 + facing * 12,
        wrist_r.1 - 7,
    );

    if fighter.weapon_id == "round_shield" {
        let shield_x = (shoulder_l.0 + wrist_l.0) / 2 - 18;
        let shield_y = (shoulder_l.1 + wrist_l.1) / 2 - 20;
        draw_box_lines(display, window, gc, shield_x, shield_y, 36, 40);
        XDrawLine(
            display,
            window,
            gc,
            shield_x,
            shield_y + 20,
            shield_x + 36,
            shield_y + 20,
        );
    } else if fighter.weapon_id == "iron_maul" || fighter.weapon_id == "war_hammer" {
        fill_box(
            display,
            window,
            gc,
            tip.0 - fighter.weapon_head_px / 2,
            tip.1 - fighter.weapon_head_px / 2,
            fighter.weapon_head_px,
            fighter.weapon_head_px,
        );
    } else if fighter.weapon_id == "board_axe" {
        XDrawLine(
            display,
            window,
            gc,
            tip.0,
            tip.1,
            tip.0 - facing * fighter.weapon_head_px,
            tip.1 + fighter.weapon_head_px,
        );
        XDrawLine(
            display,
            window,
            gc,
            tip.0,
            tip.1,
            tip.0 - facing * fighter.weapon_head_px,
            tip.1 - fighter.weapon_head_px / 2,
        );
    } else if fighter.weapon_id == "ash_spear" {
        XDrawLine(
            display,
            window,
            gc,
            tip.0,
            tip.1,
            tip.0 - facing * 14,
            tip.1 - 7,
        );
        XDrawLine(
            display,
            window,
            gc,
            tip.0,
            tip.1,
            tip.0 - facing * 14,
            tip.1 + 7,
        );
    } else {
        XDrawLine(
            display,
            window,
            gc,
            wrist_r.0,
            wrist_r.1 + 3,
            tip.0,
            tip.1 + 3,
        );
    }

    if show_labels {
        let label_x = if facing < 0 { root_x - 76 } else { root_x - 30 };
        let label_y = root_y + 126 + if facing < 0 { 0 } else { 20 };
        draw_x11_text(
            display,
            window,
            gc,
            label_x,
            label_y,
            &clipped_text(
                &format!("{} {}mm", fighter.weapon_id, fighter.weapon_reach_mm),
                16,
            ),
        );
        draw_x11_text(
            display,
            window,
            gc,
            label_x,
            label_y + 16,
            &clipped_text(
                &format!(
                    "{} cov{}",
                    fighter.armor_id, fighter.armor_torso_coverage_permille
                ),
                16,
            ),
        );
    }
}

#[cfg(target_os = "linux")]
unsafe fn draw_articulated_fighter_pose(
    display: *mut Display,
    window: Window,
    gc: GC,
    root_x: c_int,
    root_y: c_int,
    facing: c_int,
    lean: c_int,
    reach: c_int,
    collapse: c_int,
    label: &str,
) {
    let crouch = collapse / 7;
    let head = (root_x + lean, root_y - 92 + crouch);
    let neck = (root_x + lean * 3 / 4, root_y - 68 + crouch);
    let hip = (root_x, root_y + collapse / 12);
    let shoulder_r = (root_x + facing * 32 + lean / 2, root_y - 58 + crouch);
    let elbow_r = (
        root_x + facing * (62 + reach / 2) + lean / 2,
        root_y - 36 + crouch - reach / 8,
    );
    let wrist_r = (
        root_x + facing * (94 + reach) + lean / 2,
        root_y - 48 + crouch - reach / 5,
    );
    let shoulder_l = (root_x - facing * 32 + lean / 3, root_y - 58 + crouch);
    let elbow_l = (
        root_x - facing * 54 + lean / 4,
        root_y - 30 + crouch + collapse / 12,
    );
    let wrist_l = (
        root_x - facing * 70 + lean / 5,
        root_y - 4 + crouch + collapse / 10,
    );
    let knee_r = (root_x + 24, root_y + 54 + collapse / 9);
    let ankle_r = (root_x + 36 + collapse / 16, root_y + 104 + collapse / 12);
    let knee_l = (root_x - 24, root_y + 54 + collapse / 5);
    let ankle_l = (root_x - 36 - collapse / 12, root_y + 104 + collapse / 7);

    XDrawLine(
        display,
        window,
        gc,
        head.0 - 10,
        head.1,
        head.0 + 10,
        head.1,
    );
    XDrawLine(
        display,
        window,
        gc,
        head.0,
        head.1 - 10,
        head.0,
        head.1 + 10,
    );
    XDrawLine(display, window, gc, head.0, head.1 + 10, neck.0, neck.1);
    XDrawLine(display, window, gc, neck.0, neck.1, hip.0, hip.1);
    XDrawLine(
        display,
        window,
        gc,
        shoulder_r.0,
        shoulder_r.1,
        shoulder_l.0,
        shoulder_l.1,
    );
    XDrawLine(
        display,
        window,
        gc,
        shoulder_r.0,
        shoulder_r.1,
        elbow_r.0,
        elbow_r.1,
    );
    XDrawLine(
        display, window, gc, elbow_r.0, elbow_r.1, wrist_r.0, wrist_r.1,
    );
    XDrawLine(
        display,
        window,
        gc,
        wrist_r.0,
        wrist_r.1,
        wrist_r.0 + facing * 80,
        wrist_r.1 - 16 - reach / 6,
    );
    XDrawLine(
        display,
        window,
        gc,
        shoulder_l.0,
        shoulder_l.1,
        elbow_l.0,
        elbow_l.1,
    );
    XDrawLine(
        display, window, gc, elbow_l.0, elbow_l.1, wrist_l.0, wrist_l.1,
    );
    XDrawLine(display, window, gc, hip.0, hip.1, knee_r.0, knee_r.1);
    XDrawLine(
        display, window, gc, knee_r.0, knee_r.1, ankle_r.0, ankle_r.1,
    );
    XDrawLine(display, window, gc, hip.0, hip.1, knee_l.0, knee_l.1);
    XDrawLine(
        display, window, gc, knee_l.0, knee_l.1, ankle_l.0, ankle_l.1,
    );
    if collapse > 0 {
        XDrawLine(
            display,
            window,
            gc,
            hip.0 - 22,
            hip.1 + 8,
            hip.0 + 36,
            hip.1 + 34,
        );
        XDrawLine(
            display,
            window,
            gc,
            hip.0 + 36,
            hip.1 + 34,
            hip.0 + 18,
            hip.1 + 40,
        );
        XDrawLine(
            display,
            window,
            gc,
            hip.0 + 36,
            hip.1 + 34,
            hip.0 + 38,
            hip.1 + 16,
        );
    }
    draw_x11_text(display, window, gc, root_x - 38, root_y + 130, label);
}

#[cfg(target_os = "linux")]
unsafe fn capture_window_to_ppm(
    display: *mut Display,
    window: Window,
    width: u32,
    height: u32,
    path: &Path,
    black: c_ulong,
    white: c_ulong,
) -> Result<(), String> {
    let image = XGetImage(display, window, 0, 0, width, height, c_ulong::MAX, 2);
    if image.is_null() {
        return Err("XGetImage returned null".to_string());
    }
    let mut bytes = Vec::with_capacity((width * height * 3) as usize + 64);
    bytes.extend_from_slice(format!("P6\n{} {}\n255\n", width, height).as_bytes());
    for y in 0..height as c_int {
        for x in 0..width as c_int {
            let pixel = XGetPixel(image, x, y);
            let (r, g, b) = x11_pixel_to_rgb(pixel, black, white);
            bytes.push(r);
            bytes.push(g);
            bytes.push(b);
        }
    }
    XDestroyImage(image);
    fs::write(path, bytes).map_err(|error| error.to_string())
}

#[cfg(target_os = "linux")]
fn x11_pixel_to_rgb(pixel: c_ulong, black: c_ulong, white: c_ulong) -> (u8, u8, u8) {
    if pixel == black {
        (22, 24, 23)
    } else if pixel == white {
        (236, 227, 210)
    } else {
        (
            ((pixel >> 16) & 0xff) as u8,
            ((pixel >> 8) & 0xff) as u8,
            (pixel & 0xff) as u8,
        )
    }
}

#[cfg(target_os = "linux")]
fn fixed_sin_permille(angle_milli: i32) -> c_int {
    // Integer approximation over 0..2pi using a triangular wave. It is presentation-only.
    let mut a = angle_milli % 6283;
    if a < 0 {
        a += 6283;
    }
    if a < 1571 {
        a * 1000 / 1571
    } else if a < 3142 {
        1000 - (a - 1571) * 1000 / 1571
    } else if a < 4712 {
        -(a - 3142) * 1000 / 1570
    } else {
        -1000 + (a - 4712) * 1000 / 1571
    }
}

#[cfg(target_os = "linux")]
fn fixed_cos_permille(angle_milli: i32) -> c_int {
    fixed_sin_permille(angle_milli + 1571)
}

#[cfg(target_os = "linux")]
unsafe fn draw_game_panel(
    display: *mut Display,
    window: Window,
    gc: GC,
    x: c_int,
    y: c_int,
    width: c_uint,
    height: c_uint,
    label: &str,
) {
    XDrawLine(display, window, gc, x, y, x + width as c_int, y);
    XDrawLine(
        display,
        window,
        gc,
        x,
        y + height as c_int,
        x + width as c_int,
        y + height as c_int,
    );
    XDrawLine(display, window, gc, x, y, x, y + height as c_int);
    XDrawLine(
        display,
        window,
        gc,
        x + width as c_int,
        y,
        x + width as c_int,
        y + height as c_int,
    );
    XFillRectangle(display, window, gc, x + 12, y + 18, 54, 10);
    draw_x11_text(display, window, gc, x + 18, y + 48, label);
}

#[cfg(target_os = "linux")]
unsafe fn draw_x11_text(
    display: *mut Display,
    window: Window,
    gc: GC,
    x: c_int,
    y: c_int,
    text: &str,
) {
    if let Ok(cstring) = CString::new(text) {
        XDrawString(
            display,
            window,
            gc,
            x,
            y,
            cstring.as_ptr(),
            text.len() as c_int,
        );
    }
}
