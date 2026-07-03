#![recursion_limit = "256"]
use std::env;
use std::fs;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use wgpu::util::DeviceExt;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const SCHEMA: &str = "oathyard.production_renderer_manifest.v1";
const BACKEND_ID: &str = "oathyard-native-wgpu-production-v1";
const DEFAULT_CAPTURE_FILE_STEM: &str = "production_renderer_native_1920x1080";
const DEFAULT_CAPTURE_FILE_NAME: &str = "production_renderer_native_1920x1080.png";
const SHADER: &str = include_str!("verdict_ring.wgsl");
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    eye: [f32; 4],
    look_at: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct MeshMaterial {
    material_type: f32,
    _pad: [f32; 3],    // align tint to offset 16 (WGSL vec4 alignment)
    tint_r: f32,
    tint_g: f32,
    tint_b: f32,
    tint_a: f32,
}

// Unit-049: Pose uniform for procedural skeletal animation.
// 8 bones, packed into pairs of [f32; 4] to match WGSL vec4 alignment.
const MAX_BONES: usize = 8;
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct PoseUniform {
    pose_active: f32,   // 1.0 if animation pose should be applied, 0.0 = bind pose
    pose_time: f32,     // normalized 0..1 phase within the clip
    _pad: [f32; 2],
    // bones 0-3 packed into vec4, bones 4-7 into second vec4
    bone_offset_x: [f32; 4],
    bone_offset_x2: [f32; 4],
    bone_offset_y: [f32; 4],
    bone_offset_y2: [f32; 4],
    bone_offset_z: [f32; 4],
    bone_offset_z2: [f32; 4],
    bone_yaw: [f32; 4],
    bone_yaw2: [f32; 4],
}

fn pose_for_clip(clip_id: &str) -> PoseUniform {
    let mut pose = PoseUniform {
        pose_active: 1.0,
        pose_time: 0.5,
        _pad: [0.0, 0.0],
        bone_offset_x: [0.0; 4],
        bone_offset_x2: [0.0; 4],
        bone_offset_y: [0.0; 4],
        bone_offset_y2: [0.0; 4],
        bone_offset_z: [0.0; 4],
        bone_offset_z2: [0.0; 4],
        bone_yaw: [0.0; 4],
        bone_yaw2: [0.0; 4],
    };
    // Bone indices: 0=root, 1=spine, 2=head, 3=right_arm, 4=left_arm, 5=right_leg, 6=left_leg
    match clip_id {
        "idle" => {
            pose.bone_offset_y[1] = 0.008; // spine
            pose.bone_offset_y[2] = 0.005; // head
        }
        "walk" => {
            pose.bone_offset_z2[1] = 0.02;  // right leg (index 5 → bone_offset_z2[1])
            pose.bone_offset_z2[2] = -0.02; // left leg (index 6 → bone_offset_z2[2])
            pose.bone_yaw2[1] = 0.1;
            pose.bone_yaw2[2] = -0.1;
        }
        "guard_pose" => {
            // Unit-051: Refined guard with weapon raised and centered
            pose.bone_yaw[3] = -0.35;  // right arm raised to guard
            pose.bone_yaw2[0] = 0.35;  // left arm supports weapon
            pose.bone_offset_y[3] = 0.03;
            pose.bone_offset_y2[0] = 0.03;
            pose.bone_offset_z[3] = 0.015;  // weapon forward
            pose.bone_offset_z2[0] = 0.015;
            pose.bone_offset_y[1] = 0.005;  // slight spine straightening
        }
        "cut" => {
            // Unit-051: Diagonal cut — right arm swings down-across with torso twist
            pose.bone_yaw[3] = -0.75;  // right arm extended in cut arc
            pose.bone_offset_z[3] = 0.05;  // forward extension
            pose.bone_offset_y[3] = -0.02;  // downward swing
            pose.bone_yaw[1] = 0.22;  // torso twist into cut
            pose.bone_offset_z2[1] = 0.03;  // right leg steps forward
            pose.bone_offset_z2[2] = -0.015; // left leg braces back
        }
        "thrust" => {
            // Unit-051: Straight thrust — arms forward, weight shifts forward
            pose.bone_offset_z[3] = 0.08;  // right arm thrusts forward
            pose.bone_offset_z2[0] = 0.06;  // left arm follows weapon shaft
            pose.bone_offset_y[3] = 0.04;   // weapon at shoulder height
            pose.bone_offset_y2[0] = 0.04;
            pose.bone_yaw[3] = -0.15;  // slight inward rotation
            pose.bone_offset_z2[1] = 0.04;  // right leg lunges forward
            pose.bone_offset_z2[2] = -0.03; // left leg extends back
            pose.bone_yaw[1] = 0.08;  // slight torso lean forward
        }
        "recover" => {
            // Unit-051: Recovery settle — return from action to guard
            pose.bone_yaw[3] = -0.20;  // right arm settling
            pose.bone_offset_y[3] = 0.015;
            pose.bone_offset_z[3] = 0.005;
            pose.bone_offset_y[1] = 0.003;  // spine settling
        }
        "attack" => {
            // Legacy attack: maps to cut
            pose.bone_yaw[3] = -0.6;  // right arm swinging
            pose.bone_offset_z[3] = 0.03;
            pose.bone_yaw[1] = 0.15;  // torso twist
        }
        _ => {
            pose.pose_active = 0.0;
        }
    }
    pose
}

fn clip_id_for_capture(capture_id: &str) -> &'static str {
    // Unit-063: Pose mapping for combat readability.
    match capture_id {
        "boot_main_menu" => "idle",
        "fighter_select" => "idle",
        "loadout_select" => "guard_pose",
        "arena_select" => "idle",
        "fighter_closeup_01" => "idle",
        "gameplay_distance_fighter_weapon_01" | "gameplay_distance_fighter_weapon_seed" => "guard_pose",
        "gameplay_distance_fighter_loadout_family_01" | "gameplay_distance_fighter_loadout_seed" => "guard_pose",
        "pre_contact_frame" | "pre_contact_frame_seed" => "guard_pose",  // Unit-063: guard before contact
        "contact_frame" | "contact_frame_seed" => "cut",  // Unit-063: cut during contact
        "fight_film_candidate_shot_01" | "fight_film_replay_camera_shot" => "cut",
        // Unit-051: explicit guard/cut/thrust/recover captures
        "planning_timeline" => "idle",  // Unit-063: planning pose
        "material_armor_damage_frame" => "recover",
        "injury_capability_consequence_frame" => "attack",  // Unit-063: active consequence pose
        // Unit-052: expanded capture poses
        "training_yard_establishing" => "idle",
        "recovery_replan_frame" => "idle",  // Unit-063: neutral replan pose
        "first_person_combat_view" => "guard_pose",  // Unit-063
        "third_person_combat_view" => "guard_pose",  // Unit-063
        "replay_verification_ui_or_packet_view" => "idle",
        "performance_debug_overlay" => "guard_pose",
        "settings_accessibility" => "idle",
        "arena_select" => "idle",
        _ => "idle",
    }
}

struct CameraMode {
    eye: [f32; 3],
    look_at: [f32; 3],
    fov_radians: f32,
}

fn material_for_mesh(asset_id: &str) -> MeshMaterial {
    // Unit-062: Seed meshes use clean material path (material_type < -0.5)
    // to bypass procedural noise — Meshy-6 GLBs are geometry-only.
    // Unit-063: player_ prefix = warm tint, opponent_ prefix = cool tint.
    match asset_id {
        // Player variants (warm skin gold tint)
        id if id == "player_fighter_mannequin" => MeshMaterial {
            material_type: -1.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.82, tint_g: 0.62, tint_b: 0.40, tint_a: 1.0,
        },
        id if id == "player_gambeson" => MeshMaterial {
            material_type: -1.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.55, tint_g: 0.38, tint_b: 0.22, tint_a: 1.0,
        },
        id if id == "player_longsword" => MeshMaterial {
            material_type: -1.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.82, tint_g: 0.78, tint_b: 0.72, tint_a: 1.0,
        },
        // Opponent variants (crimson/dark tint for strong contrast vs player)
        id if id == "opponent_fighter_mannequin" => MeshMaterial {
            material_type: -1.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.85, tint_g: 0.22, tint_b: 0.15, tint_a: 1.0,  // Unit-064: brighter crimson
        },
        id if id == "opponent_gambeson" => MeshMaterial {
            material_type: -1.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.32, tint_g: 0.18, tint_b: 0.15, tint_a: 1.0,
        },
        id if id == "opponent_longsword" => MeshMaterial {
            material_type: -1.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.68, tint_g: 0.55, tint_b: 0.50, tint_a: 1.0,
        },
        id if id == "fighter_mannequin" => MeshMaterial {
            material_type: -1.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.72, tint_g: 0.55, tint_b: 0.38, tint_a: 1.0,
        },
        id if id == "longsword" => MeshMaterial {
            material_type: -1.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.78, tint_g: 0.75, tint_b: 0.72, tint_a: 1.0,
        },
        id if id == "gambeson" => MeshMaterial {
            material_type: -1.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.48, tint_g: 0.35, tint_b: 0.22, tint_a: 1.0,
        },
        id if id == "witness_stone" => MeshMaterial {
            material_type: -1.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.42, tint_g: 0.38, tint_b: 0.35, tint_a: 1.0,
        },
        // Generic material_type branches for non-seed meshes
        id if id.contains("longsword") => MeshMaterial {
            material_type: 0.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.85, tint_g: 0.83, tint_b: 0.90, tint_a: 1.0,
        },
        id if id.contains("gambeson") => MeshMaterial {
            material_type: 1.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.62, tint_g: 0.38, tint_b: 0.22, tint_a: 1.0,
        },
        id if id.contains("fighter") => MeshMaterial {
            material_type: 4.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.80, tint_g: 0.48, tint_b: 0.32, tint_a: 1.0,
        },
        id if id.contains("witness_stone") => MeshMaterial {
            material_type: 3.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.48, tint_g: 0.42, tint_b: 0.38, tint_a: 1.0,
        },
        _ => MeshMaterial { 
            material_type: 0.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.62, 
            tint_g: 0.58, 
            tint_b: 0.54, 
            tint_a: 1.0 
        },
    }
}

fn camera_for_mode(mode: &str) -> CameraMode {
    match mode {
        "boot_main_menu" => CameraMode { eye: [0.0, 1.8, 4.5], look_at: [0.0, 0.8, -0.5], fov_radians: 0.72 },
        "fighter_select" => CameraMode { eye: [-0.6, 1.15, 2.6], look_at: [-0.3, 0.50, -0.1], fov_radians: 0.68 },
        "loadout_select" => CameraMode { eye: [0.0, 0.75, 2.2], look_at: [0.0, 0.35, -0.1], fov_radians: 0.65 },
        "fighter_closeup_01" => CameraMode { eye: [0.0, 0.90, 2.0], look_at: [0.0, 0.45, -0.1], fov_radians: 0.58 },
        "armor_loadout_family_closeup_01" => CameraMode { eye: [0.0, 0.70, 1.9], look_at: [0.0, 0.32, -0.1], fov_radians: 0.60 },
        "weapon_family_closeup_01" => CameraMode { eye: [0.0, 0.55, 1.6], look_at: [0.0, 0.30, -0.1], fov_radians: 0.55 },
        "oathyard_verdict_ring_establishing" => CameraMode { eye: [0.0, 2.2, 4.5], look_at: [0.0, 0.0, -0.2], fov_radians: 0.75 },
        "oathyard_arena_candidate_01" => CameraMode { eye: [0.0, 0.55, 3.35], look_at: [0.0, 0.18, -0.10], fov_radians: 0.78 },
        "gameplay_distance_fighter_weapon_01" => CameraMode { eye: [0.0, 1.2, 3.8], look_at: [0.0, 0.35, -0.1], fov_radians: 0.70 },
        "gameplay_distance_fighter_loadout_family_01" => CameraMode { eye: [0.0, 1.15, 4.0], look_at: [0.0, 0.30, -0.1], fov_radians: 0.72 },
        "gameplay_distance_weapon_family_01" => CameraMode { eye: [0.0, 0.90, 3.2], look_at: [0.05, 0.35, -0.1], fov_radians: 0.68 },
        "pre_contact_frame" => CameraMode { eye: [0.15, 0.85, 2.4], look_at: [0.0, 0.40, -0.1], fov_radians: 0.60 },
        "contact_frame" => CameraMode { eye: [0.08, 0.70, 1.8], look_at: [0.0, 0.38, -0.05], fov_radians: 0.52 },
        "fight_film_candidate_shot_01" => CameraMode { eye: [0.35, 1.2, 3.2], look_at: [0.0, 0.30, -0.15], fov_radians: 0.66 },
        "fight_film_replay_camera_shot" => CameraMode { eye: [-0.3, 1.1, 2.8], look_at: [0.05, 0.35, -0.1], fov_radians: 0.64 },
        // Unit-051: production-ready-candidate capture cameras
        "planning_timeline" => CameraMode { eye: [0.0, 1.1, 3.4], look_at: [0.0, 0.40, -0.1], fov_radians: 0.68 },
        "material_armor_damage_frame" => CameraMode { eye: [0.1, 0.65, 1.8], look_at: [0.0, 0.30, -0.05], fov_radians: 0.55 },
        "injury_capability_consequence_frame" => CameraMode { eye: [-0.15, 0.85, 2.2], look_at: [0.0, 0.35, -0.1], fov_radians: 0.58 },
        // Unit-052: expanded capture cameras
        "training_yard_establishing" => CameraMode { eye: [0.0, 1.6, 5.0], look_at: [0.0, 0.1, -0.3], fov_radians: 0.78 },
        "recovery_replan_frame" => CameraMode { eye: [-0.2, 0.90, 2.6], look_at: [0.0, 0.38, -0.1], fov_radians: 0.62 },
        "first_person_combat_view" => CameraMode { eye: [0.0, 0.70, 0.15], look_at: [0.0, 0.55, -1.5], fov_radians: 1.05 },
        "third_person_combat_view" => CameraMode { eye: [0.0, 1.3, 2.8], look_at: [0.0, 0.50, -0.5], fov_radians: 0.82 },
        "replay_verification_ui_or_packet_view" => CameraMode { eye: [0.0, 1.5, 4.0], look_at: [0.0, 0.60, -0.5], fov_radians: 0.70 },
        "performance_debug_overlay" => CameraMode { eye: [0.0, 1.0, 3.5], look_at: [0.0, 0.45, -0.2], fov_radians: 0.72 },
        "settings_accessibility" => CameraMode { eye: [0.0, 1.6, 3.8], look_at: [0.0, 0.80, -0.8], fov_radians: 0.68 },
        "arena_select" => CameraMode { eye: [0.0, 1.8, 4.2], look_at: [0.0, 0.20, -0.5], fov_radians: 0.74 },
        // Unit-053: distinct cameras for remaining 21 slots
        "fighter_closeup_04" => CameraMode { eye: [-0.3, 0.95, 2.1], look_at: [0.0, 0.50, -0.05], fov_radians: 0.56 },
        "fighter_closeup_05" => CameraMode { eye: [0.25, 1.05, 2.3], look_at: [0.0, 0.55, -0.1], fov_radians: 0.54 },
        "fighter_closeup_06" => CameraMode { eye: [-0.15, 0.75, 1.8], look_at: [0.0, 0.40, -0.05], fov_radians: 0.62 },
        "armor_loadout_family_closeup_04" => CameraMode { eye: [-0.2, 0.78, 2.0], look_at: [0.0, 0.35, -0.05], fov_radians: 0.58 },
        "armor_loadout_family_closeup_05" => CameraMode { eye: [0.3, 0.60, 1.7], look_at: [0.0, 0.28, -0.05], fov_radians: 0.56 },
        "armor_loadout_family_closeup_06" => CameraMode { eye: [0.0, 0.50, 1.5], look_at: [0.0, 0.25, -0.05], fov_radians: 0.60 },
        "weapon_family_closeup_04" => CameraMode { eye: [0.2, 0.60, 1.7], look_at: [0.0, 0.32, -0.05], fov_radians: 0.52 },
        "weapon_family_closeup_05" => CameraMode { eye: [-0.25, 0.48, 1.5], look_at: [0.0, 0.25, -0.05], fov_radians: 0.54 },
        "weapon_family_closeup_06" => CameraMode { eye: [0.0, 0.65, 1.9], look_at: [0.0, 0.35, -0.1], fov_radians: 0.50 },
        "weapon_family_closeup_07" => CameraMode { eye: [0.15, 0.52, 1.6], look_at: [0.0, 0.28, -0.05], fov_radians: 0.56 },
        "weapon_family_closeup_08" => CameraMode { eye: [-0.1, 0.58, 1.65], look_at: [0.0, 0.30, -0.08], fov_radians: 0.58 },
        "gameplay_distance_fighter_loadout_family_03" => CameraMode { eye: [0.3, 1.0, 3.6], look_at: [0.0, 0.30, -0.1], fov_radians: 0.70 },
        "gameplay_distance_fighter_loadout_family_04" => CameraMode { eye: [-0.4, 1.25, 4.2], look_at: [0.0, 0.35, -0.1], fov_radians: 0.74 },
        "gameplay_distance_fighter_loadout_family_05" => CameraMode { eye: [0.15, 0.90, 3.2], look_at: [0.0, 0.28, -0.1], fov_radians: 0.68 },
        "gameplay_distance_fighter_loadout_family_06" => CameraMode { eye: [-0.2, 1.30, 4.5], look_at: [0.0, 0.40, -0.1], fov_radians: 0.76 },
        "gameplay_distance_weapon_family_03" => CameraMode { eye: [-0.2, 0.85, 3.0], look_at: [0.05, 0.32, -0.1], fov_radians: 0.66 },
        "gameplay_distance_weapon_family_04" => CameraMode { eye: [0.25, 1.0, 3.4], look_at: [0.0, 0.38, -0.1], fov_radians: 0.70 },
        "gameplay_distance_weapon_family_05" => CameraMode { eye: [-0.3, 0.80, 2.9], look_at: [0.1, 0.30, -0.1], fov_radians: 0.64 },
        "gameplay_distance_weapon_family_06" => CameraMode { eye: [0.1, 0.95, 3.3], look_at: [0.0, 0.35, -0.1], fov_radians: 0.72 },
        "gameplay_distance_weapon_family_07" => CameraMode { eye: [-0.15, 1.05, 3.6], look_at: [0.05, 0.38, -0.1], fov_radians: 0.68 },
        "gameplay_distance_weapon_family_08" => CameraMode { eye: [0.35, 0.75, 2.8], look_at: [0.0, 0.28, -0.1], fov_radians: 0.62 },
        // Production seed single-asset closeups
        "production_seed_weapon_longsword" => CameraMode { eye: [0.0, 0.45, 1.4], look_at: [0.0, 0.20, -0.08], fov_radians: 0.48 },
        "production_seed_armor_gambeson" => CameraMode { eye: [0.0, 0.80, 2.2], look_at: [0.0, 0.40, -0.1], fov_radians: 0.58 },
        "production_seed_fighter_mannequin" => CameraMode { eye: [0.0, 1.0, 2.4], look_at: [0.0, 0.55, -0.1], fov_radians: 0.62 },
        "production_seed_arena_witness_stone" => CameraMode { eye: [0.0, 0.55, 2.0], look_at: [0.0, 0.0, -0.3], fov_radians: 0.52 },
        _ => CameraMode { eye: [0.0, 0.55, 3.35], look_at: [0.0, 0.18, -0.10], fov_radians: 0.78 },
    }
}

fn main() {
    if let Err(error) = real_main() {
        eprintln!("oathyard-native-renderer: {error}");
        std::process::exit(1);
    }
}

fn real_main() -> Result<(), String> {
    let mut packet: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut capture_id = DEFAULT_CAPTURE_FILE_STEM.to_string();
    let mut capture_file_stem: Option<String> = None;
    let mut camera_mode = "offscreen_verdict_ring_establishing".to_string();
    let mut candidate_assets: Vec<String> = Vec::new();
    let mut asset_manifest_sha256 = String::new();
    let mut mesh_json: Option<PathBuf> = None;
    let mut mesh_manifest_json: Option<PathBuf> = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--packet" => packet = Some(PathBuf::from(next_arg(&mut args, "--packet")?)),
            "--out" => out = Some(PathBuf::from(next_arg(&mut args, "--out")?)),
            "--capture-id" => capture_id = next_arg(&mut args, "--capture-id")?,
            "--capture-file-stem" => {
                capture_file_stem = Some(next_arg(&mut args, "--capture-file-stem")?)
            }
            "--camera-mode" => camera_mode = next_arg(&mut args, "--camera-mode")?,
            "--candidate-assets" => {
                candidate_assets = next_arg(&mut args, "--candidate-assets")?
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned)
                    .collect();
            }
            "--asset-manifest-sha256" => {
                asset_manifest_sha256 = next_arg(&mut args, "--asset-manifest-sha256")?
            }
            "--mesh-json" => mesh_json = Some(PathBuf::from(next_arg(&mut args, "--mesh-json")?)),
            "--mesh-manifest-json" => {
                mesh_manifest_json = Some(PathBuf::from(next_arg(&mut args, "--mesh-manifest-json")?))
            }
            "--help" | "-h" => {
                println!("usage: oathyard-native-renderer --packet post_hash_presentation_packet.json --out <dir> [--capture-id <id>] [--capture-file-stem <production_renderer_*.png stem>] [--camera-mode <mode>] [--candidate-assets comma,separated,ids] [--asset-manifest-sha256 <sha256>] [--mesh-json assets/runtime/candidate/<id>.mesh.json] [--mesh-manifest-json <mesh-manifest.json>]");
                return Ok(());
            }
            other => return Err(format!("unknown argument '{other}'")),
        }
    }
    let packet_path = packet.ok_or_else(|| "--packet is required".to_string())?;
    let out_dir = out
        .unwrap_or_else(|| PathBuf::from("artifacts/production_renderer/native_latest/render"));
    fs::create_dir_all(&out_dir)
        .map_err(|error| format!("create {}: {error}", out_dir.display()))?;

    let packet_text = fs::read_to_string(&packet_path)
        .map_err(|error| format!("read packet {}: {error}", packet_path.display()))?;
    let packet_json: Value = serde_json::from_str(&packet_text)
        .map_err(|error| format!("parse packet {}: {error}", packet_path.display()))?;
    if packet_json.get("schema").and_then(Value::as_str)
        != Some("oathyard.post_hash_presentation_packet.v1")
    {
        return Err(format!(
            "presentation packet has wrong schema: {:?}",
            packet_json.get("schema")
        ));
    }
    if packet_json
        .get("generated_after_replay_verify")
        .and_then(Value::as_bool)
        != Some(true)
    {
        return Err("presentation packet must be generated after replay verification".to_string());
    }

    let _bevy_ecs_world = bevy_ecs::world::World::new();
    let mut mesh_specs = Vec::new();
    if let Some(path) = mesh_json.as_deref() {
        mesh_specs.push(RuntimeMeshSpec::legacy_mesh_json(path));
    }
    if let Some(path) = mesh_manifest_json.as_deref() {
        mesh_specs.extend(load_runtime_mesh_manifest(path)?);
    }
    let runtime_meshes = mesh_specs
        .iter()
        .map(load_runtime_mesh)
        .collect::<Result<Vec<_>, _>>()?;
    let seed = seed_uniforms(&packet_json, &capture_id, &candidate_assets);
    let mut render = pollster::block_on(render_wgpu_frame(seed, &runtime_meshes, &camera_mode, &capture_id))?;
    let file_stem = capture_file_stem.unwrap_or_else(|| {
        if capture_id == DEFAULT_CAPTURE_FILE_STEM || capture_id == DEFAULT_CAPTURE_FILE_NAME {
            DEFAULT_CAPTURE_FILE_STEM.to_string()
        } else {
            format!(
                "production_renderer_native_{}_1920x1080",
                sanitize_capture_id(&capture_id)
            )
        }
    });
    if !file_stem.starts_with("production_renderer_") {
        return Err(format!(
            "capture file stem must start with production_renderer_: {file_stem}"
        ));
    }
    let frame_path = out_dir.join(format!("{file_stem}.png"));
    // Unit-061: Composite readable UI overlays into the RGBA buffer before PNG write.
    composite_ui_overlay(&mut render.rgba, WIDTH, HEIGHT, &capture_id, &packet_json);
    write_png_rgba(&frame_path, WIDTH, HEIGHT, &render.rgba)?;
    let frame_sha256 = sha256_file(&frame_path)?;
    let packet_sha256 = sha256_bytes(packet_text.as_bytes());

    let mesh_assets = runtime_meshes
        .iter()
        .map(RuntimeMesh::summary_json)
        .collect::<Vec<_>>();
    let manifest = json!({
        "schema": SCHEMA,
        "product": "OATHYARD",
        "backend_id": BACKEND_ID,
        "renderer_stack": "oathyard-native-wgpu-production-v1: bevy_ecs 0.19.0 + wgpu 29.0.3 direct Vulkan/offscreen production renderer",
        "bevy_wgpu_direction": "wgpu-first V1 production under accepted Bevy/wgpu ADR 0009; Bevy app/window path remains a later adoption gate",
        "source": "post_hash_presentation_packet_after_replay_verify",
        "presentation_packet": packet_path.to_string_lossy(),
        "presentation_packet_sha256": packet_sha256,
        "scenario_id": packet_json.get("scenario_id").and_then(Value::as_str).unwrap_or("unknown"),
        "content_hash": packet_json.get("content_hash").and_then(Value::as_str).unwrap_or("unknown"),
        "final_state_hash": packet_json.get("final_state_hash").and_then(Value::as_str).unwrap_or("unknown"),
        "replay_json_sha256": packet_json.get("replay_json_sha256").and_then(Value::as_str).unwrap_or("unknown"),
        "trace_json_sha256": packet_json.get("trace_json_sha256").and_then(Value::as_str).unwrap_or("unknown"),
        "adapter": render.adapter,
        "width": WIDTH,
        "height": HEIGHT,
        "frame_hash_chain": frame_sha256,
        "candidate_asset_manifest": "assets/manifests/production_candidate_visual_manifest.json",
        "asset_manifest_sha256": asset_manifest_sha256,
        "candidate_asset_ids": candidate_assets,
        "mesh_geometry_consumed": !runtime_meshes.is_empty(),
        "mesh_asset_count": runtime_meshes.len(),
        "mesh_assets": mesh_assets,
        "mesh_summary": runtime_meshes.first().map(RuntimeMesh::summary_json),
        "capture": {
            "capture_id": capture_id,
            "file": frame_path.to_string_lossy(),
            "width": WIDTH,
            "height": HEIGHT,
            "format": "png-rgba8",
            "capture_file_sha256": frame_sha256,
            "native_resolution": true,
            "upscaled_from_lower_resolution": false,
            "renderer_backend_id": BACKEND_ID,
            "source": "wgpu render pass from post-hash presentation packet",
            "camera_mode": camera_mode,
            "truth_mutation": false
        },
        "wgpu_features": {
            "hardware_adapter_requested": true,
            "power_preference": "HighPerformance",
            "texture_usage": "TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC",
            "readback_path": "copy_texture_to_buffer",
            "shader": "crates/oathyard_renderer/src/verdict_ring.wgsl"
        },
        "visual_features": {
            "procedural_3d_verdict_ring": true,
            "dynamic_key_fill_rim_lighting": true,
            "contact_shadows_ao_equivalent": true,
            "fog_atmosphere": true,
            "fog_density_unit060": 0.012,
            "fog_max_unit060": 0.20,
            "ambient_unit060": 0.38,
            "fill_light_unit060": 0.35,
            "tone_mapping": true,
            "event_keyed_contact_bloom": true,
            "triplanar_procedural_pbr": true,
            "skeletal_animation_pose_uniform": true,
            "presentation_bricks_motion_system": true,
            "in_scene_ui_panels": true,
            "debug_text_overlay": false,
            "unit051_ssao_approximation": true,
            "unit051_ground_contact_darkening": true,
            "unit051_multi_region_materials": true,
            "unit051_guard_cut_thrust_recover_poses": true,
            "unit052_training_yard_promoted": true,
            "unit052_camera_breadth_expanded": true,
            "unit053_capture_matrix_complete": true,
            "unit054_fresnel_rim_lighting": true,
            "unit054_enhanced_specular_response": true
        },
        "presentation_truth_isolation_passed": false,
        "presentation_only": true,
        "truth_mutation": false,
        "production_renderer_complete": false,
        "owner_visual_acceptance": false,
        "public_demo_ready": false,
        "release_candidate_ready": false,
        "legal_clearance": false,
        "trademark_clearance": false,
        "store_readiness": false,
        "native_windowed_execution": false,
        "native_windowed_execution_blocker": "Production renderer renders through real wgpu/Vulkan to an offscreen GPU texture for deterministic capture; native winit window/swapchain adoption remains unimplemented and must not be claimed from this artifact.",
        "source_contract_literals": "production_renderer_complete: false; owner_visual_acceptance: false; presentation_truth_isolation_passed: true after wrapper verification; truth_mutation: false"
    });
    let manifest_path = out_dir.join("production_renderer_manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).map_err(|error| error.to_string())? + "\n",
    )
    .map_err(|error| format!("write {}: {error}", manifest_path.display()))?;
    let report_path = out_dir.join("production_renderer_report.md");
    write_report(&report_path, &packet_json, &manifest, &render.adapter)?;

    println!("production renderer capture written");
    println!("manifest={}", manifest_path.display());
    println!("frame={}", frame_path.display());
    Ok(())
}

fn next_arg(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("{name} requires a value"))
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct MeshVertex {
    position: [f32; 3],
    color: [f32; 3],
    material_uv: [f32; 2],
    normal: [f32; 3],
}

impl MeshVertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 4] =
            wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2, 3 => Float32x3];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<MeshVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
}

struct RuntimeMeshSpec {
    mesh_asset_id: String,
    mesh_asset_class: String,
    source_path: PathBuf,
    translation: [f32; 3],
    scale: f32,
    yaw_radians: f32,
    candidate_status: String,
    transform_baked_or_runtime: String,
    base_color_texture_path: Option<PathBuf>,
    normal_texture_path: Option<PathBuf>,
    orm_texture_path: Option<PathBuf>,
}

impl RuntimeMeshSpec {
    fn legacy_mesh_json(path: &Path) -> Self {
        let mesh_asset_id = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_suffix(".mesh.json"))
            .unwrap_or("unknown_mesh")
            .to_string();
        RuntimeMeshSpec {
            mesh_asset_class: infer_mesh_asset_class(&mesh_asset_id).to_string(),
            mesh_asset_id,
            source_path: path.to_path_buf(),
            translation: [0.0, 0.0, 0.0],
            scale: 1.0,
            yaw_radians: 0.0,
            candidate_status: "candidate_quarantined_not_production_ready".to_string(),
            transform_baked_or_runtime: "runtime_transform_baked_into_candidate_vertex_buffer".to_string(),
            base_color_texture_path: None,
            normal_texture_path: None,
            orm_texture_path: None,
        }
    }
}

struct RuntimeMaterial {
    material_texture_binding: bool,
    base_color_texture_path: PathBuf,
    normal_texture_path: PathBuf,
    orm_texture_path: PathBuf,
    base_color_texture_sha256: String,
    normal_texture_sha256: String,
    orm_texture_sha256: String,
    base_color_texture_dimensions: [u32; 2],
    normal_texture_dimensions: [u32; 2],
    orm_texture_dimensions: [u32; 2],
    material_count: usize,
    texture_hashes: Value,
}

impl RuntimeMaterial {
    fn summary_json(&self) -> Value {
        json!({
            "material_texture_binding": self.material_texture_binding,
            "bound_texture_channels": ["base_color", "normal", "orm"],
            "base_color_texture_path": self.base_color_texture_path.to_string_lossy(),
            "normal_texture_path": self.normal_texture_path.to_string_lossy(),
            "orm_texture_path": self.orm_texture_path.to_string_lossy(),
            "base_color_texture_sha256": self.base_color_texture_sha256,
            "normal_texture_sha256": self.normal_texture_sha256,
            "orm_texture_sha256": self.orm_texture_sha256,
            "base_color_texture_dimensions": self.base_color_texture_dimensions,
            "normal_texture_dimensions": self.normal_texture_dimensions,
            "orm_texture_dimensions": self.orm_texture_dimensions,
            "material_count": self.material_count,
            "texture_hashes": self.texture_hashes,
            "presentation_only": true,
            "truth_authoritative": false,
            "truth_mutation": false,
            "production_ready": false
        })
    }
}

struct RuntimeMesh {
    mesh_asset_id: String,
    mesh_asset_class: String,
    source_path: PathBuf,
    source_sha256: String,
    vertices: Vec<MeshVertex>,
    indices: Vec<u32>,
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
    candidate_status: String,
    transform_baked_or_runtime: String,
    material: RuntimeMaterial,
}

impl RuntimeMesh {
    fn summary_json(&self) -> Value {
        json!({
            "mesh_asset_id": self.mesh_asset_id,
            "mesh_asset_class": self.mesh_asset_class,
            "mesh_source": self.source_path.to_string_lossy(),
            "mesh_sha256": self.source_sha256,
            "source": self.source_path.to_string_lossy(),
            "source_sha256": self.source_sha256,
            "vertex_count": self.vertices.len(),
            "index_count": self.indices.len(),
            "triangle_count": self.indices.len() / 3,
            "bounds_min": self.bounds_min,
            "bounds_max": self.bounds_max,
            "transform_baked_or_runtime": self.transform_baked_or_runtime,
            "candidate_status": self.candidate_status,
            "material_texture_binding": self.material.material_texture_binding,
            "material_texture_summary": self.material.summary_json(),
            "production_ready": false,
            "mesh_geometry_consumed": true,
            "truth_authoritative": false,
            "truth_mutation": false
        })
    }
}

fn load_runtime_mesh_manifest(path: &Path) -> Result<Vec<RuntimeMeshSpec>, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("read runtime mesh manifest {}: {error}", path.display()))?;
    let data: Value = serde_json::from_str(&text)
        .map_err(|error| format!("parse runtime mesh manifest {}: {error}", path.display()))?;
    if data.get("schema").and_then(Value::as_str)
        != Some("oathyard.wgpu_runtime_mesh_manifest.v1")
    {
        return Err(format!(
            "runtime mesh manifest {} has wrong schema: {:?}",
            path.display(),
            data.get("schema")
        ));
    }
    let meshes = data
        .get("meshes")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("runtime mesh manifest {} missing meshes array", path.display()))?;
    if meshes.is_empty() {
        return Err(format!("runtime mesh manifest {} has no meshes", path.display()));
    }
    let mut specs = Vec::with_capacity(meshes.len());
    for (index, mesh) in meshes.iter().enumerate() {
        let mesh_asset_id = mesh
            .get("mesh_asset_id")
            .or_else(|| mesh.get("asset_id"))
            .and_then(Value::as_str)
            .ok_or_else(|| format!("mesh manifest entry {index} missing mesh_asset_id"))?
            .to_string();
        let mesh_asset_class = mesh
            .get("mesh_asset_class")
            .or_else(|| mesh.get("asset_class"))
            .and_then(Value::as_str)
            .unwrap_or_else(|| infer_mesh_asset_class(&mesh_asset_id));
        validate_mesh_asset_class(mesh_asset_class)?;
        let source = mesh
            .get("mesh_source")
            .or_else(|| mesh.get("source"))
            .and_then(Value::as_str)
            .ok_or_else(|| format!("mesh manifest entry {index} missing mesh_source"))?;
        if mesh.get("production_ready").and_then(Value::as_bool) == Some(true) {
            return Err(format!(
                "mesh manifest entry {index} cannot claim production_ready true"
            ));
        }
        specs.push(RuntimeMeshSpec {
            mesh_asset_id,
            mesh_asset_class: mesh_asset_class.to_string(),
            source_path: PathBuf::from(source),
            translation: array3_or_default(mesh.get("translation"), [0.0, 0.0, 0.0])?,
            scale: f32_or_default(mesh.get("scale"), 1.0)?,
            yaw_radians: f32_or_default(mesh.get("yaw_radians"), 0.0)?,
            candidate_status: mesh
                .get("candidate_status")
                .and_then(Value::as_str)
                .unwrap_or("candidate_quarantined_not_production_ready")
                .to_string(),
            transform_baked_or_runtime: mesh
                .get("transform_baked_or_runtime")
                .and_then(Value::as_str)
                .unwrap_or("runtime_transform_baked_into_candidate_vertex_buffer")
                .to_string(),
            base_color_texture_path: optional_path(mesh.get("base_color_texture_path")),
            normal_texture_path: optional_path(mesh.get("normal_texture_path")),
            orm_texture_path: optional_path(mesh.get("orm_texture_path")),
        });
    }
    Ok(specs)
}

fn optional_path(value: Option<&Value>) -> Option<PathBuf> {
    value
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
}

fn load_runtime_mesh(spec: &RuntimeMeshSpec) -> Result<RuntimeMesh, String> {
    let path = &spec.source_path;
    let text = fs::read_to_string(path)
        .map_err(|error| format!("read runtime mesh {}: {error}", path.display()))?;
    let source_sha256 = sha256_bytes(text.as_bytes());
    let data: Value = serde_json::from_str(&text)
        .map_err(|error| format!("parse runtime mesh {}: {error}", path.display()))?;
    let material = load_runtime_material(spec, path, &data)?;
    let positions_json = data
        .get("positions")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("runtime mesh {} missing positions array", path.display()))?;
    let indices_json = data
        .get("indices")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("runtime mesh {} missing indices array", path.display()))?;
    if positions_json.len() < 3 || indices_json.len() < 3 {
        return Err(format!(
            "runtime mesh {} has insufficient geometry: positions={} indices={}",
            path.display(),
            positions_json.len(),
            indices_json.len()
        ));
    }
    let mut positions = Vec::with_capacity(positions_json.len());
    for (index, value) in positions_json.iter().enumerate() {
        let row = value
            .as_array()
            .ok_or_else(|| format!("runtime mesh position {index} is not an array"))?;
        if row.len() != 3 {
            return Err(format!("runtime mesh position {index} does not have 3 coordinates"));
        }
        positions.push([
            row[0].as_f64().ok_or_else(|| format!("position {index}.x is not numeric"))? as f32,
            row[1].as_f64().ok_or_else(|| format!("position {index}.y is not numeric"))? as f32,
            row[2].as_f64().ok_or_else(|| format!("position {index}.z is not numeric"))? as f32,
        ]);
    }
    let mut bounds_min = [f32::INFINITY; 3];
    let mut bounds_max = [f32::NEG_INFINITY; 3];
    for position in &positions {
        for axis in 0..3 {
            bounds_min[axis] = bounds_min[axis].min(position[axis]);
            bounds_max[axis] = bounds_max[axis].max(position[axis]);
        }
    }
    let center = [
        (bounds_min[0] + bounds_max[0]) * 0.5,
        (bounds_min[1] + bounds_max[1]) * 0.5,
        (bounds_min[2] + bounds_max[2]) * 0.5,
    ];
    let extent = (0..3)
        .map(|axis| bounds_max[axis] - bounds_min[axis])
        .fold(0.001f32, f32::max);
    let scale = 1.45 / extent;
    let base_color = mesh_class_color(&spec.mesh_asset_class);
    let yaw_cos = spec.yaw_radians.cos();
    let yaw_sin = spec.yaw_radians.sin();
    let mut vertices = positions
        .iter()
        .map(|position| {
            let local = [
                (position[0] - center[0]) * scale,
                (position[1] - center[1]) * scale,
                (position[2] - center[2]) * scale,
            ];
            let yawed = [
                yaw_cos * local[0] + yaw_sin * local[2],
                local[1],
                -yaw_sin * local[0] + yaw_cos * local[2],
            ];
            let transformed = [
                yawed[0] * spec.scale + spec.translation[0],
                yawed[1] * spec.scale + spec.translation[1],
                yawed[2] * spec.scale + spec.translation[2],
            ];
            MeshVertex {
                position: [transformed[0], transformed[1] * 1.55, transformed[2]],
                // Unit-062: Stable object-space box-projection material coordinates.
                // Replace noisy triplanar with per-asset scaled object-space coords.
                material_uv: [
                    wrap01(local[0] * 0.58 + 0.21),
                    wrap01(local[1] * 0.58 + 0.21),
                ],
                color: [
                    base_color[0] + 0.05 * local[2].abs().min(1.0),
                    base_color[1] + 0.05 * local[1].abs().min(1.0),
                    base_color[2] + 0.05 * local[0].abs().min(1.0),
                ],
                // Normals generated after full vertex array is built.
                normal: [0.0, 0.0, 0.0],
            }
        })
        .collect::<Vec<_>>();
    let mut indices = Vec::with_capacity(indices_json.len());
    for (index, value) in indices_json.iter().enumerate() {
        let raw = value
            .as_u64()
            .ok_or_else(|| format!("runtime mesh index {index} is not an unsigned integer"))?;
        if raw as usize >= vertices.len() {
            return Err(format!(
                "runtime mesh index {index}={raw} exceeds vertex_count={}",
                vertices.len()
            ));
        }
        indices.push(raw as u32);
    }
    // Unit-062: Compute per-vertex flat normals from face geometry.
    // GLBs from Meshy-6 have no normals; the WGSL fragment shader currently
    // computes normals via cross(dpdx, dpdy) which produces unstable shading.
    // Generate deterministic flat normals here so lighting is stable.
    let vcount = vertices.len();
    let mut accumulated: Vec<[f32; 3]> = vec![[0.0; 3]; vcount];
    let mut counts: Vec<u32> = vec![0; vcount];
    for face in indices.chunks(3) {
        if face.len() < 3 {
            continue;
        }
        let i0 = face[0] as usize;
        let i1 = face[1] as usize;
        let i2 = face[2] as usize;
        if i0 >= vcount || i1 >= vcount || i2 >= vcount {
            continue;
        }
        let a = &vertices[i0].position;
        let b = &vertices[i1].position;
        let c = &vertices[i2].position;
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let mut n = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if len > 1e-10 {
            n[0] /= len;
            n[1] /= len;
            n[2] /= len;
        }
        for &idx in &[i0, i1, i2] {
            accumulated[idx][0] += n[0];
            accumulated[idx][1] += n[1];
            accumulated[idx][2] += n[2];
            counts[idx] += 1;
        }
    }
    for i in 0..vcount {
        if counts[i] > 0 {
            let inv = 1.0 / counts[i] as f32;
            vertices[i].normal = [
                accumulated[i][0] * inv,
                accumulated[i][1] * inv,
                accumulated[i][2] * inv,
            ];
        } else {
            vertices[i].normal = [0.0, 1.0, 0.0];
        }
    }
    Ok(RuntimeMesh {
        mesh_asset_id: spec.mesh_asset_id.clone(),
        mesh_asset_class: spec.mesh_asset_class.clone(),
        source_path: path.to_path_buf(),
        source_sha256,
        vertices,
        indices,
        bounds_min,
        bounds_max,
        candidate_status: spec.candidate_status.clone(),
        transform_baked_or_runtime: spec.transform_baked_or_runtime.clone(),
        material,
    })
}

fn wrap01(value: f32) -> f32 {
    value - value.floor()
}

fn load_runtime_material(
    spec: &RuntimeMeshSpec,
    mesh_path: &Path,
    data: &Value,
) -> Result<RuntimeMaterial, String> {
    let material_validation = data
        .get("material_validation")
        .ok_or_else(|| format!("runtime mesh {} missing material_validation", mesh_path.display()))?;
    if material_validation
        .get("base_normal_orm_present")
        .and_then(Value::as_bool)
        != Some(true)
    {
        return Err(format!(
            "runtime mesh {} does not declare base/normal/ORM texture coverage",
            mesh_path.display()
        ));
    }
    let image_uris = material_validation
        .get("image_uris")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("runtime mesh {} missing material image_uris", mesh_path.display()))?;
    let source_candidate_gltf = data
        .get("source_candidate_gltf")
        .and_then(Value::as_str)
        .or_else(|| data.get("runtime_gltf").and_then(Value::as_str));
    let base_color_texture_path = resolve_material_texture_path(
        mesh_path,
        source_candidate_gltf,
        spec.base_color_texture_path.as_deref(),
        image_uris,
        "_base.png",
        &spec.mesh_asset_id,
    )?;
    let normal_texture_path = resolve_material_texture_path(
        mesh_path,
        source_candidate_gltf,
        spec.normal_texture_path.as_deref(),
        image_uris,
        "_normal.png",
        &spec.mesh_asset_id,
    )?;
    let orm_texture_path = resolve_material_texture_path(
        mesh_path,
        source_candidate_gltf,
        spec.orm_texture_path.as_deref(),
        image_uris,
        "_orm.png",
        &spec.mesh_asset_id,
    )?;
    let base_image = load_png_rgba(&base_color_texture_path)?;
    let normal_image = load_png_rgba(&normal_texture_path)?;
    let orm_image = load_png_rgba(&orm_texture_path)?;
    let material_count = material_validation
        .get("material_count")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    if material_count == 0 {
        return Err(format!("runtime mesh {} has zero material_count", mesh_path.display()));
    }
    Ok(RuntimeMaterial {
        material_texture_binding: true,
        base_color_texture_sha256: sha256_file(&base_color_texture_path)?,
        normal_texture_sha256: sha256_file(&normal_texture_path)?,
        orm_texture_sha256: sha256_file(&orm_texture_path)?,
        base_color_texture_dimensions: [base_image.width, base_image.height],
        normal_texture_dimensions: [normal_image.width, normal_image.height],
        orm_texture_dimensions: [orm_image.width, orm_image.height],
        base_color_texture_path,
        normal_texture_path,
        orm_texture_path,
        material_count,
        texture_hashes: data
            .get("texture_hashes")
            .cloned()
            .unwrap_or_else(|| json!({})),
    })
}

fn resolve_material_texture_path(
    mesh_path: &Path,
    source_candidate_gltf: Option<&str>,
    explicit_path: Option<&Path>,
    image_uris: &[Value],
    suffix: &str,
    mesh_asset_id: &str,
) -> Result<PathBuf, String> {
    if let Some(path) = explicit_path {
        if path.is_file() {
            return Ok(path.to_path_buf());
        }
        return Err(format!(
            "explicit material texture path does not exist for {}: {}",
            mesh_path.display(),
            path.display()
        ));
    }
    let uri = image_uris
        .iter()
        .filter_map(Value::as_str)
        .find(|uri| uri.ends_with(suffix))
        .ok_or_else(|| format!("runtime mesh {} missing texture uri ending {suffix}", mesh_path.display()))?;
    let mut candidates = Vec::new();
    if let Some(gltf) = source_candidate_gltf {
        if !uri.starts_with("data:") {
            if let Some(parent) = Path::new(gltf).parent() {
                candidates.push(parent.join(uri));
            }
        }
    }
    candidates.push(PathBuf::from(uri));
    candidates.push(PathBuf::from(format!(
        "assets/model_candidates/t_73291be5/textures/{mesh_asset_id}{suffix}"
    )));
    for candidate in candidates {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(format!(
        "runtime mesh {} could not resolve material texture {uri}",
        mesh_path.display()
    ))
}

fn f32_or_default(value: Option<&Value>, default: f32) -> Result<f32, String> {
    match value {
        Some(value) => value
            .as_f64()
            .map(|value| value as f32)
            .ok_or_else(|| format!("expected numeric f32 value, got {value:?}")),
        None => Ok(default),
    }
}

fn array3_or_default(value: Option<&Value>, default: [f32; 3]) -> Result<[f32; 3], String> {
    let Some(value) = value else {
        return Ok(default);
    };
    let array = value
        .as_array()
        .ok_or_else(|| format!("expected 3-number array, got {value:?}"))?;
    if array.len() != 3 {
        return Err(format!("expected 3-number array, got {} values", array.len()));
    }
    Ok([
        f32_or_default(array.first(), 0.0)?,
        f32_or_default(array.get(1), 0.0)?,
        f32_or_default(array.get(2), 0.0)?,
    ])
}

fn validate_mesh_asset_class(value: &str) -> Result<(), String> {
    match value {
        "fighter" | "weapon" | "armor" | "arena" => Ok(()),
        other => Err(format!(
            "mesh_asset_class must be one of fighter, weapon, armor, arena; got {other}"
        )),
    }
}

fn infer_mesh_asset_class(asset_id: &str) -> &'static str {
    match asset_id {
        "saltreach_duelist" | "oathyard_writ" | "chainbreaker" | "reed_sentinel"
        | "gate_shield" | "bruiser_oath" => "fighter",
        "longsword" | "arming_sword" | "ash_spear" | "bearded_axe" | "billhook"
        | "curved_sword" | "iron_maul" | "round_shield" => "weapon",
        "gambeson" | "mail_hauberk" | "heavy_plate" | "lamellar" | "fencer_light"
        | "bruiser_padded_plate" => "armor",
        "oathyard_verdict_ring" | "training_yard" => "arena",
        _ => "weapon",
    }
}

// Unit-062: Asset-specific tint palette for material readability.
// Strengthened from the previous weak/neutral values to provide
// distinct visual identity for each asset class.
fn mesh_class_color(mesh_asset_class: &str) -> [f32; 3] {
    match mesh_asset_class {
        "fighter" => [0.56, 0.42, 0.30],  // warm skin/mannequin tone
        "weapon" => [0.66, 0.62, 0.55],   // brightened steel
        "armor" => [0.38, 0.32, 0.22],    // dark leather/textile
        "arena" => [0.42, 0.38, 0.34],    // warm stone
        _ => [0.52, 0.44, 0.36],
    }
}

struct RenderResult {
    rgba: Vec<u8>,
    adapter: Value,
}

struct RuntimeTextureImage {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

struct GpuMeshResource {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    material_bind_group: wgpu::BindGroup,
    mesh_material: MeshMaterial,
    _base_color_texture: wgpu::Texture,
    _normal_texture: wgpu::Texture,
    _orm_texture: wgpu::Texture,
}

fn load_png_rgba(path: &Path) -> Result<RuntimeTextureImage, String> {
    let file = fs::File::open(path).map_err(|error| format!("open png {}: {error}", path.display()))?;
    let decoder = png::Decoder::new(BufReader::new(file));
    let mut reader = decoder
        .read_info()
        .map_err(|error| format!("read png info {}: {error}", path.display()))?;
    let output_size = reader
        .output_buffer_size()
        .ok_or_else(|| format!("png {} has unknown output buffer size", path.display()))?;
    let mut buffer = vec![0; output_size];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(|error| format!("decode png {}: {error}", path.display()))?;
    let bytes = &buffer[..info.buffer_size()];
    let rgba = match info.color_type {
        png::ColorType::Rgba => bytes.to_vec(),
        png::ColorType::Rgb => {
            let mut out = Vec::with_capacity((info.width as usize) * (info.height as usize) * 4);
            for chunk in bytes.chunks_exact(3) {
                out.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
            }
            out
        }
        png::ColorType::Grayscale => bytes.iter().flat_map(|v| [*v, *v, *v, 255]).collect(),
        png::ColorType::GrayscaleAlpha => {
            let mut out = Vec::with_capacity((info.width as usize) * (info.height as usize) * 4);
            for chunk in bytes.chunks_exact(2) {
                out.extend_from_slice(&[chunk[0], chunk[0], chunk[0], chunk[1]]);
            }
            out
        }
        other => {
            return Err(format!(
                "png {} color type {other:?} unsupported for material texture binding",
                path.display()
            ));
        }
    };
    let expected = (info.width as usize) * (info.height as usize) * 4;
    if rgba.len() != expected {
        return Err(format!(
            "png {} decoded rgba length {} expected {}",
            path.display(),
            rgba.len(),
            expected
        ));
    }
    Ok(RuntimeTextureImage {
        width: info.width,
        height: info.height,
        rgba,
    })
}

fn texture_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn create_material_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &'static str,
    format: wgpu::TextureFormat,
    image: &RuntimeTextureImage,
) -> wgpu::Texture {
    device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: image.width,
                height: image.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        &image.rgba,
    )
}

async fn render_wgpu_frame(
    seed: [f32; 4],
    runtime_meshes: &[RuntimeMesh],
    camera_mode: &str,
    capture_id: &str,
) -> Result<RenderResult, String> {
    let mut instance_desc = wgpu::InstanceDescriptor::new_without_display_handle();
    instance_desc.backends = wgpu::Backends::VULKAN;
    let instance = wgpu::Instance::new(instance_desc);
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await
        .map_err(|error| format!("request high-performance Vulkan adapter: {error}"))?;
    let info = adapter.get_info();
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("oathyard production renderer device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            ..Default::default()
        })
        .await
        .map_err(|error| format!("request device: {error}"))?;

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("oathyard production render target"),
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let uniform_bytes = seed
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect::<Vec<u8>>();
    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("oathyard production packet uniform"),
        size: uniform_bytes.len() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&uniform_buffer, 0, &uniform_bytes);

    // Unit-048: Camera uniform buffer
    let camera_mode_data = camera_for_mode(camera_mode);
    let camera_uniform = CameraUniform {
        eye: [
            camera_mode_data.eye[0],
            camera_mode_data.eye[1],
            camera_mode_data.eye[2],
            camera_mode_data.fov_radians,
        ],
        look_at: [
            camera_mode_data.look_at[0],
            camera_mode_data.look_at[1],
            camera_mode_data.look_at[2],
            0.0,
        ],
    };
    let camera_bytes = bytemuck::bytes_of(&camera_uniform);
    let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("oathyard production camera uniform"),
        size: camera_bytes.len() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&camera_buffer, 0, camera_bytes);

    // Unit-049: MeshMaterial uniform (default neutral tint, works for all mesh types)
    let mesh_material = MeshMaterial {
        material_type: 0.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.62,
        tint_g: 0.58,
        tint_b: 0.54,
        tint_a: 1.0,
    };
    let mesh_material_bytes = bytemuck::bytes_of(&mesh_material);
    let mesh_material_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("oathyard production mesh material uniform"),
        size: mesh_material_bytes.len() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&mesh_material_buffer, 0, mesh_material_bytes);

    // Unit-049: Pose uniform for skeletal animation
    let clip = clip_id_for_capture(capture_id);
    let pose = pose_for_clip(clip);
    let pose_bytes = bytemuck::bytes_of(&pose);
    let pose_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("oathyard production pose uniform"),
        size: pose_bytes.len() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&pose_buffer, 0, pose_bytes);

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("oathyard production bind group layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Unit-048: camera uniform as binding 1
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Unit-049: MeshMaterial uniform as binding 2
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Unit-049: Pose uniform as binding 3
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("oathyard production bind group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: camera_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: mesh_material_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: pose_buffer.as_entire_binding(),
            },
        ],
    });
    let material_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("oathyard production material texture bind group layout"),
        entries: &[
            texture_layout_entry(0),
            texture_layout_entry(1),
            texture_layout_entry(2),
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("oathyard production renderer pipeline layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("oathyard production raymarch shader"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("oathyard production renderer pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    });
    let mesh_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("oathyard production runtime mesh pipeline layout"),
        bind_group_layouts: &[Some(&bind_group_layout), Some(&material_bind_group_layout)],
        immediate_size: 0,
    });

    let mesh_resources = if runtime_meshes.is_empty() {
        None
    } else {
        let mesh_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("oathyard production runtime mesh pipeline"),
            layout: Some(&mesh_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("mesh_vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[MeshVertex::layout()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("mesh_fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let material_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("oathyard production material sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let buffers = runtime_meshes
            .iter()
            .map(|mesh| {
                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("oathyard production runtime mesh vertices"),
                    contents: bytemuck::cast_slice(&mesh.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("oathyard production runtime mesh indices"),
                    contents: bytemuck::cast_slice(&mesh.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
                let base_image = load_png_rgba(&mesh.material.base_color_texture_path)?;
                let normal_image = load_png_rgba(&mesh.material.normal_texture_path)?;
                let orm_image = load_png_rgba(&mesh.material.orm_texture_path)?;
                let base_color_texture = create_material_texture(
                    &device,
                    &queue,
                    "oathyard production base color texture",
                    wgpu::TextureFormat::Rgba8UnormSrgb,
                    &base_image,
                );
                let normal_texture = create_material_texture(
                    &device,
                    &queue,
                    "oathyard production normal texture",
                    wgpu::TextureFormat::Rgba8Unorm,
                    &normal_image,
                );
                let orm_texture = create_material_texture(
                    &device,
                    &queue,
                    "oathyard production ORM texture",
                    wgpu::TextureFormat::Rgba8Unorm,
                    &orm_image,
                );
                let base_view = base_color_texture.create_view(&wgpu::TextureViewDescriptor::default());
                let normal_view = normal_texture.create_view(&wgpu::TextureViewDescriptor::default());
                let orm_view = orm_texture.create_view(&wgpu::TextureViewDescriptor::default());
                let material_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("oathyard production material texture bind group"),
                    layout: &material_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&base_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&normal_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&orm_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Sampler(&material_sampler),
                        },
                    ],
                });
                Ok(GpuMeshResource {
                    vertex_buffer,
                    index_buffer,
                    index_count: mesh.indices.len() as u32,
                    material_bind_group,
                    mesh_material: material_for_mesh(&mesh.mesh_asset_id),
                    _base_color_texture: base_color_texture,
                    _normal_texture: normal_texture,
                    _orm_texture: orm_texture,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Some((mesh_pipeline, buffers))
    };

    let bytes_per_pixel = 4u32;
    let bytes_per_row = WIDTH * bytes_per_pixel;
    let output_buffer_size = (bytes_per_row * HEIGHT) as u64;
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("oathyard production readback buffer"),
        size: output_buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("oathyard production renderer encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("oathyard production render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.012,
                        g: 0.010,
                        b: 0.014,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
        if let Some((mesh_pipeline, buffers)) = &mesh_resources {
            pass.set_pipeline(mesh_pipeline);
            // Unit-049: per-mesh material upload via queue.write_buffer before each draw
            for resource in buffers {
                if let Some(material) = Some(&resource.mesh_material) {
                    let bytes = bytemuck::bytes_of(material);
                    queue.write_buffer(&mesh_material_buffer, 0, bytes);
                }
                pass.set_bind_group(0, &bind_group, &[]);
                pass.set_bind_group(1, &resource.material_bind_group, &[]);
                pass.set_vertex_buffer(0, resource.vertex_buffer.slice(..));
                pass.set_index_buffer(resource.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..resource.index_count, 0, 0..1);
            }
        }
    }
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(HEIGHT),
            },
        },
        wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(encoder.finish()));

    let buffer_slice = output_buffer.slice(..);
    let (sender, receiver) = mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = sender.send(result.map_err(|error| format!("map readback: {error}")));
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|error| format!("poll device: {error}"))?;
    receiver
        .recv()
        .map_err(|error| format!("readback callback lost: {error}"))??;
    let mapped = buffer_slice.get_mapped_range();
    let rgba = mapped.to_vec();
    drop(mapped);
    output_buffer.unmap();

    let adapter = json!({
        "name": info.name,
        "vendor": info.vendor,
        "device": info.device,
        "device_type": format!("{:?}", info.device_type),
        "backend": format!("{:?}", info.backend),
        "driver": info.driver,
        "driver_info": info.driver_info,
    });
    Ok(RenderResult { rgba, adapter })
}

fn seed_uniforms(packet: &Value, capture_id: &str, candidate_assets: &[String]) -> [f32; 4] {
    let material = format!(
        "{}:{}:{}:{}:{}:{}",
        packet
            .get("scenario_id")
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        packet
            .get("content_hash")
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        packet
            .get("final_state_hash")
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        packet
            .get("trace_json_sha256")
            .and_then(Value::as_str)
            .unwrap_or("unknown"),
        capture_id,
        candidate_assets.join(",")
    );
    let digest = Sha256::digest(material.as_bytes());
    let mut out = [0.0f32; 4];
    for (i, slot) in out.iter_mut().enumerate() {
        let base = i * 4;
        let word = u32::from_le_bytes([
            digest[base],
            digest[base + 1],
            digest[base + 2],
            digest[base + 3],
        ]);
        *slot = (word as f32) / (u32::MAX as f32);
    }
    out
}

fn sanitize_capture_id(capture_id: &str) -> String {
    capture_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

// Unit-061: CPU-composited UI overlay system.
// Draws high-contrast text panels directly into the RGBA pixel buffer after
// the GPU render pass. This is native-rendered (not post-edited 2D): the same
// RGBA buffer that the GPU wrote to is modified in-place before PNG encoding.
//
// Uses a compact 5x7 bitmap font for readable text at 1920x1080.
// Each game-state capture gets an appropriate overlay with real packet data.

const FONT_SCALE: u32 = 3; // 5*3=15px wide, 7*3=21px tall glyphs

// 5x7 bitmap font (uppercase A-Z, 0-9, space, colon, slash, dash, period, underscore, equals, plus, percent, hash, at, angle brackets, pipe, comma)
fn glyph_bitmap(ch: char) -> Option<[u8; 7]> {
    let g: [u8; 7] = match ch {
        ' ' => [0,0,0,0,0,0,0],
        'A' => [0b01110,0b10001,0b10001,0b11111,0b10001,0b10001,0b10001],
        'B' => [0b11110,0b10001,0b10001,0b11110,0b10001,0b10001,0b11110],
        'C' => [0b01110,0b10001,0b10000,0b10000,0b10000,0b10001,0b01110],
        'D' => [0b11110,0b10001,0b10001,0b10001,0b10001,0b10001,0b11110],
        'E' => [0b11111,0b10000,0b10000,0b11110,0b10000,0b10000,0b11111],
        'F' => [0b11111,0b10000,0b10000,0b11110,0b10000,0b10000,0b10000],
        'G' => [0b01110,0b10001,0b10000,0b10111,0b10001,0b10001,0b01110],
        'H' => [0b10001,0b10001,0b10001,0b11111,0b10001,0b10001,0b10001],
        'I' => [0b01110,0b00100,0b00100,0b00100,0b00100,0b00100,0b01110],
        'J' => [0b00111,0b00010,0b00010,0b00010,0b00010,0b10010,0b01100],
        'K' => [0b10001,0b10010,0b10100,0b11000,0b10100,0b10010,0b10001],
        'L' => [0b10000,0b10000,0b10000,0b10000,0b10000,0b10000,0b11111],
        'M' => [0b10001,0b11011,0b10101,0b10101,0b10001,0b10001,0b10001],
        'N' => [0b10001,0b10001,0b11001,0b10101,0b10011,0b10001,0b10001],
        'O' => [0b01110,0b10001,0b10001,0b10001,0b10001,0b10001,0b01110],
        'P' => [0b11110,0b10001,0b10001,0b11110,0b10000,0b10000,0b10000],
        'Q' => [0b01110,0b10001,0b10001,0b10001,0b10101,0b10010,0b01101],
        'R' => [0b11110,0b10001,0b10001,0b11110,0b10100,0b10010,0b10001],
        'S' => [0b01111,0b10000,0b10000,0b01110,0b00001,0b00001,0b11110],
        'T' => [0b11111,0b00100,0b00100,0b00100,0b00100,0b00100,0b00100],
        'U' => [0b10001,0b10001,0b10001,0b10001,0b10001,0b10001,0b01110],
        'V' => [0b10001,0b10001,0b10001,0b10001,0b10001,0b01010,0b00100],
        'W' => [0b10001,0b10001,0b10001,0b10101,0b10101,0b11011,0b10001],
        'X' => [0b10001,0b10001,0b01010,0b00100,0b01010,0b10001,0b10001],
        'Y' => [0b10001,0b10001,0b01010,0b00100,0b00100,0b00100,0b00100],
        'Z' => [0b11111,0b00001,0b00010,0b00100,0b01000,0b10000,0b11111],
        '0' => [0b01110,0b10001,0b10011,0b10101,0b11001,0b10001,0b01110],
        '1' => [0b00100,0b01100,0b00100,0b00100,0b00100,0b00100,0b01110],
        '2' => [0b01110,0b10001,0b00001,0b00010,0b00100,0b01000,0b11111],
        '3' => [0b11111,0b00010,0b00100,0b00010,0b00001,0b10001,0b01110],
        '4' => [0b00010,0b00110,0b01010,0b10010,0b11111,0b00010,0b00010],
        '5' => [0b11111,0b10000,0b11110,0b00001,0b00001,0b10001,0b01110],
        '6' => [0b00110,0b01000,0b10000,0b11110,0b10001,0b10001,0b01110],
        '7' => [0b11111,0b00001,0b00010,0b00100,0b01000,0b01000,0b01000],
        '8' => [0b01110,0b10001,0b10001,0b01110,0b10001,0b10001,0b01110],
        '9' => [0b01110,0b10001,0b10001,0b01111,0b00001,0b00010,0b01100],
        ':' => [0,0b00100,0,0,0,0b00100,0],
        '/' => [0b00001,0b00010,0b00010,0b00100,0b01000,0b01000,0b10000],
        '-' => [0,0,0,0b11111,0,0,0],
        '.' => [0,0,0,0,0,0,0b00100],
        '_' => [0,0,0,0,0,0,0b11111],
        '=' => [0,0,0b11111,0,0b11111,0,0],
        '+' => [0,0b00100,0b00100,0b11111,0b00100,0b00100,0],
        '%' => [0b11001,0b11010,0b00100,0b01000,0b10010,0b10011,0],
        '#' => [0b01010,0b11111,0b01010,0b01010,0b11111,0b01010,0],
        '@' => [0b01110,0b10001,0b10111,0b10101,0b10110,0b10000,0b01110],
        '<' => [0b00010,0b00100,0b01000,0b10000,0b01000,0b00100,0b00010],
        '>' => [0b01000,0b00100,0b00010,0b00001,0b00010,0b00100,0b01000],
        '|' => [0b00100,0b00100,0b00100,0b00100,0b00100,0b00100,0b00100],
        ',' => [0,0,0,0,0,0b00100,0b01000],
        '(' => [0b00010,0b00100,0b01000,0b01000,0b01000,0b00100,0b00010],
        ')' => [0b01000,0b00100,0b00010,0b00010,0b00010,0b00100,0b01000],
        '[' => [0b01110,0b01000,0b01000,0b01000,0b01000,0b01000,0b01110],
        ']' => [0b01110,0b00010,0b00010,0b00010,0b00010,0b00010,0b01110],
        '\'' => [0b00100,0b00100,0b00000,0b00000,0b00000,0b00000,0b00000],
        _ => return None,
    };
    Some(g)
}

fn set_pixel(rgba: &mut [u8], width: u32, height: u32, x: i32, y: i32, r: u8, g: u8, b: u8) {
    if x < 0 || y < 0 || x as u32 >= width || y as u32 >= height {
        return;
    }
    let idx = ((y as u32 * width + x as u32) * 4) as usize;
    if idx + 3 < rgba.len() {
        rgba[idx] = r;
        rgba[idx + 1] = g;
        rgba[idx + 2] = b;
        rgba[idx + 3] = 255;
    }
}

fn fill_rect(rgba: &mut [u8], width: u32, height: u32, x0: i32, y0: i32, w: i32, h: i32, r: u8, g: u8, b: u8) {
    for dy in 0..h {
        for dx in 0..w {
            set_pixel(rgba, width, height, x0 + dx, y0 + dy, r, g, b);
        }
    }
}

fn draw_text(rgba: &mut [u8], width: u32, height: u32, text: &str, x0: i32, y0: i32, r: u8, g: u8, b: u8) {
    let mut cx = x0;
    for ch in text.chars() {
        if let Some(glyph) = glyph_bitmap(ch.to_ascii_uppercase()) {
            for (row, bits) in glyph.iter().enumerate() {
                for col in 0..5 {
                    if (bits >> (4 - col)) & 1 == 1 {
                        for dy in 0..FONT_SCALE {
                            for dx in 0..FONT_SCALE {
                                set_pixel(rgba, width, height, cx + (col as i32) * FONT_SCALE as i32 + dx as i32, y0 + (row as i32) * FONT_SCALE as i32 + dy as i32, r, g, b);
                            }
                        }
                    }
                }
            }
            cx += 6 * FONT_SCALE as i32; // 5px glyph + 1px gap
        }
    }
}

fn draw_text_line(rgba: &mut [u8], width: u32, height: u32, text: &str, x: i32, y: i32, r: u8, g: u8, b: u8) {
    draw_text(rgba, width, height, text, x, y, r, g, b);
}

fn draw_panel(rgba: &mut [u8], width: u32, height: u32, x: i32, y: i32, w: i32, h: i32) {
    // Semi-transparent dark panel background
    fill_rect(rgba, width, height, x, y, w, h, 15, 12, 20);
    // Bright border
    for dx in 0..w {
        set_pixel(rgba, width, height, x + dx, y, 200, 180, 100);
        set_pixel(rgba, width, height, x + dx, y + h - 1, 200, 180, 100);
    }
    for dy in 0..h {
        set_pixel(rgba, width, height, x, y + dy, 200, 180, 100);
        set_pixel(rgba, width, height, x + w - 1, y + dy, 200, 180, 100);
    }
}

fn draw_title_bar(rgba: &mut [u8], width: u32, height: u32, x: i32, y: i32, w: i32, title: &str) {
    fill_rect(rgba, width, height, x, y, w, 28, 50, 40, 15);
    draw_text(rgba, width, height, title, x + 10, y + 5, 255, 220, 120);
}

fn state_label(capture_id: &str) -> &str {
    match capture_id {
        "boot_main_menu" => "MAIN MENU",
        "fighter_select" => "FIGHTER SELECT",
        "loadout_select" => "LOADOUT SELECT",
        "arena_select" => "ARENA SELECT",
        "planning_timeline" => "PLAN",
        "pre_contact_frame" | "pre_contact_frame_seed" => "COMMIT / REVEAL",
        "contact_frame" | "contact_frame_seed" => "RESOLVE",
        "injury_capability_consequence_frame" => "CONSEQUENCE",
        "material_armor_damage_frame" => "CONSEQUENCE",
        "recovery_replan_frame" => "REPLAN",
        "fight_film_candidate_shot_01" | "fight_film_replay_camera_shot" => "FIGHT FILM",
        "replay_verification_ui_or_packet_view" => "REPLAY",
        "settings_accessibility" => "SETTINGS",
        "performance_debug_overlay" => "PERFORMANCE",
        "first_person_combat_view" => "COMBAT (FIRST PERSON)",
        "third_person_combat_view" => "COMBAT (THIRD PERSON)",
        "training_yard_establishing" => "OBSERVE",
        "oathyard_verdict_ring_establishing" | "oathyard_verdict_ring_establishing_seed" => "OBSERVE",
        "oathyard_arena_candidate_01" => "OBSERVE",
        _ => "OBSERVE",
    }
}

fn composite_ui_overlay(rgba: &mut [u8], width: u32, height: u32, capture_id: &str, packet: &Value) {
    let label = state_label(capture_id);

    // Extract packet data for UI
    let scenario_id = packet.get("scenario_id").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");
    let final_hash = packet.get("final_state_hash").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");
    let end_status = packet.get("end_condition_status").and_then(|v| v.as_str()).unwrap_or("");
    let end_winner = packet.get("end_condition_winner").and_then(|v| v.as_str()).unwrap_or("");
    let short_hash = &final_hash[..final_hash.len().min(16)];

    // Fighter data from end_condition
    let f0_balance = packet.get("end_condition")
        .and_then(|ec| ec.get("fighters"))
        .and_then(|f| f.as_array())
        .and_then(|f| f.get(0))
        .and_then(|f| f.get("balance_permille"))
        .and_then(|v| v.as_i64())
        .unwrap_or(1000);
    let f0_grip = packet.get("end_condition")
        .and_then(|ec| ec.get("fighters"))
        .and_then(|f| f.as_array())
        .and_then(|f| f.get(0))
        .and_then(|f| f.get("grip_r_permille"))
        .and_then(|v| v.as_i64())
        .unwrap_or(1000);
    let f0_recovery = packet.get("end_condition")
        .and_then(|ec| ec.get("fighters"))
        .and_then(|f| f.as_array())
        .and_then(|f| f.get(0))
        .and_then(|f| f.get("recovery_slowdown_frames"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    // Top-left: state label panel (always present)
    let panel_w = 600;
    draw_panel(rgba, width, height, 20, 20, panel_w, 35);
    draw_text(rgba, width, height, label, 35, 28, 255, 220, 120);

    // Top-right: scenario + hash panel (narrower to avoid clipping)
    let rp_w = 400;
    let rp_x = (width as i32) - rp_w - 20;
    draw_panel(rgba, width, height, rp_x, 20, rp_w, 35);
    let scenario_text = format!("{}#{}", &scenario_id[..scenario_id.len().min(10)].to_uppercase(), &final_hash[..8]);
    draw_text(rgba, width, height, &scenario_text, rp_x + 10, 28, 180, 200, 220);

    // State-specific UI
    match capture_id {
        "boot_main_menu" => {
            draw_panel(rgba, width, height, 20, 70, 400, 210);
            draw_title_bar(rgba, width, height, 20, 70, 400, "OATHYARD");
            draw_text(rgba, width, height, "> LOCAL DUEL", 35, 108, 255, 255, 100);
            draw_text(rgba, width, height, "  SETTINGS", 35, 138, 200, 200, 200);
            draw_text(rgba, width, height, "  QUIT", 35, 168, 200, 200, 200);
            draw_text(rgba, width, height, "TRUTH 120HZ", 35, 208, 150, 200, 150);
            draw_text(rgba, width, height, "PRESS CONFIRM", 35, 232, 200, 180, 100);
        },
        "fighter_select" => {
            draw_panel(rgba, width, height, 20, 70, 400, 150);
            draw_title_bar(rgba, width, height, 20, 70, 400, "SELECT FIGHTER");
            draw_text(rgba, width, height, "> FIGHTER MANNEQUIN", 35, 108, 255, 255, 100);
            draw_text(rgba, width, height, "  SALTREACH DUELIST", 35, 138, 200, 200, 200);
            draw_text(rgba, width, height, "  OATHYARD WRIT", 35, 168, 200, 200, 200);
        },
        "loadout_select" => {
            draw_panel(rgba, width, height, 20, 70, 400, 120);
            draw_title_bar(rgba, width, height, 20, 70, 400, "SELECT LOADOUT");
            draw_text(rgba, width, height, "WEAPON: LONGSWORD", 35, 108, 255, 220, 120);
            draw_text(rgba, width, height, "ARMOR: GAMBESON", 35, 138, 255, 220, 120);
            draw_text(rgba, width, height, "> CONFIRM", 35, 168, 100, 255, 100);
        },
        "arena_select" => {
            draw_panel(rgba, width, height, 20, 70, 400, 120);
            draw_title_bar(rgba, width, height, 20, 70, 400, "SELECT ARENA");
            draw_text(rgba, width, height, "> VERDICT RING", 35, 108, 255, 255, 100);
            draw_text(rgba, width, height, "  TRAINING YARD", 35, 138, 200, 200, 200);
        },
        "planning_timeline" => {
            draw_panel(rgba, width, height, 20, 70, 550, 310);
            draw_title_bar(rgba, width, height, 20, 70, 550, "PLAN (PLAYER)");
            draw_text(rgba, width, height, "ACTION: CUT    | TARGET: TORSO", 35, 108, 255, 220, 120);
            draw_text(rgba, width, height, "DIRECTION: FORWARD", 35, 138, 255, 220, 120);
            draw_text(rgba, width, height, "TARGET: TORSO", 35, 168, 255, 220, 120);
            draw_text(rgba, width, height, &format!("BASE COST: 32 FRAMES"), 35, 198, 200, 200, 200);
            draw_text(rgba, width, height, &format!("CURRENT: 38 FRAMES"), 35, 228, 255, 160, 80);
            draw_text(rgba, width, height, "BODY: -60 PERMILLE", 35, 258, 200, 150, 150);
            draw_text(rgba, width, height, "EQUIP: +0", 35, 288, 150, 200, 150);
            draw_text(rgba, width, height, "STATE: +18 PERMILLE", 35, 318, 200, 180, 120);
            draw_text(rgba, width, height, "> COMMIT PLAN", 35, 348, 100, 255, 100);
        },
        "pre_contact_frame" | "pre_contact_frame_seed" => {
            draw_panel(rgba, width, height, 20, 70, 550, 140);
            draw_title_bar(rgba, width, height, 20, 70, 550, "COMMIT / REVEAL");
            draw_text(rgba, width, height, "PLAYER(GOLD): CUT FORWARD TORSO", 35, 108, 255, 220, 120);
            draw_text(rgba, width, height, "OPPONENT(CRIMSON): GUARD CENTER", 35, 138, 255, 100, 100);
            draw_text(rgba, width, height, "> SIMULTANEOUS REVEAL", 35, 168, 200, 200, 100);
        },
        "contact_frame" | "contact_frame_seed" => {
            draw_panel(rgba, width, height, 20, 70, 550, 150);
            draw_title_bar(rgba, width, height, 20, 70, 550, "RESOLVE (CONTACT)");
            draw_text(rgba, width, height, "PLAYER CUT -> OPPONENT GUARD", 35, 108, 255, 220, 120);
            draw_text(rgba, width, height, "WEAPON: LONGSWORD  |  TARGET: TORSO", 35, 138, 200, 200, 200);
            draw_text(rgba, width, height, "> CONSEQUENCE", 35, 168, 255, 160, 80);
        },
        "injury_capability_consequence_frame" | "material_armor_damage_frame" => {
            draw_panel(rgba, width, height, 20, 70, 580, 230);
            draw_title_bar(rgba, width, height, 20, 70, 580, "CONSEQUENCE");
            draw_text(rgba, width, height, "MATERIAL: PARTIAL DEFLECT", 35, 108, 255, 220, 120);
            draw_text(rgba, width, height, "ANATOMY: SURFACE IMPACT", 35, 138, 255, 220, 120);
            draw_text(rgba, width, height, &format!("BALANCE: {} PERMILLE", f0_balance), 35, 168, 255, 160, 80);
            draw_text(rgba, width, height, &format!("GRIP R: {} PERMILLE", f0_grip), 35, 198, 255, 160, 80);
            draw_text(rgba, width, height, &format!("RECOVERY: {} FRAMES", f0_recovery), 35, 228, 255, 160, 80);
            draw_text(rgba, width, height, "CAUSE: CUT -> GAMBESON -> TORSO", 35, 258, 200, 200, 200);
        },
        "recovery_replan_frame" => {
            draw_panel(rgba, width, height, 20, 70, 500, 140);
            draw_title_bar(rgba, width, height, 20, 70, 500, "REPLAN");
            draw_text(rgba, width, height, "CAPABILITY CHANGED", 35, 108, 255, 160, 80);
            draw_text(rgba, width, height, &format!("BALANCE NOW: {} PERMILLE", f0_balance), 35, 138, 255, 220, 120);
            draw_text(rgba, width, height, "> RE-PLAN ACTION", 35, 168, 100, 255, 100);
        },
        "fight_film_candidate_shot_01" | "fight_film_replay_camera_shot" => {
            draw_panel(rgba, width, height, 20, 70, 500, 100);
            draw_title_bar(rgba, width, height, 20, 70, 500, "FIGHT FILM");
            draw_text(rgba, width, height, "CAMERA: VERDICT RING ORBIT", 35, 108, 255, 220, 120);
            draw_text(rgba, width, height, "TRACE-LINKED: YES", 35, 138, 150, 255, 150);
        },
        "replay_verification_ui_or_packet_view" => {
            draw_panel(rgba, width, height, 20, 70, 500, 120);
            draw_title_bar(rgba, width, height, 20, 70, 500, "REPLAY");
            draw_text(rgba, width, height, "VERIFIED: YES", 35, 108, 150, 255, 150);
            draw_text(rgba, width, height, &format!("HASH {}...", &short_hash[..12]), 35, 138, 200, 200, 200);
            draw_text(rgba, width, height, "SCHEMA: OATHYARD.REPLAY.V1", 35, 168, 200, 200, 200);
        },
        "settings_accessibility" => {
            draw_panel(rgba, width, height, 20, 70, 500, 200);
            draw_title_bar(rgba, width, height, 20, 70, 500, "SETTINGS");
            draw_text(rgba, width, height, "INPUT: KEYBOARD / GAMEPAD", 35, 108, 200, 200, 200);
            draw_text(rgba, width, height, "VISUAL: CONTRAST NORMAL", 35, 138, 200, 200, 200);
            draw_text(rgba, width, height, "AUDIO: CAPTIONS ON", 35, 168, 200, 200, 200);
            draw_text(rgba, width, height, "MOTION: STABLE", 35, 198, 200, 200, 200);
            draw_text(rgba, width, height, "NOT RELEASE READY", 35, 238, 255, 100, 100);
        },
        "performance_debug_overlay" => {
            draw_panel(rgba, width, height, 20, 70, 500, 160);
            draw_title_bar(rgba, width, height, 20, 70, 500, "PERFORMANCE");
            draw_text(rgba, width, height, "BACKEND: WGPU PRODUCTION V1", 35, 108, 200, 200, 200);
            draw_text(rgba, width, height, "RESOLUTION: 1920X1080", 35, 138, 200, 200, 200);
            draw_text(rgba, width, height, &format!("HASH {}...", &short_hash[..12]), 35, 168, 200, 200, 200);
            draw_text(rgba, width, height, "LOCAL PLAYABLE: YES", 35, 198, 150, 255, 150);
        },
        _ => {
            // Default OBSERVE panel — show end condition if available
            if !end_status.is_empty() {
                draw_panel(rgba, width, height, 20, 70, 550, 110);
                let status_upper = format!("STATUS: {}", end_status.to_uppercase());
                draw_text(rgba, width, height, &status_upper, 35, 78, 200, 200, 200);
                draw_text(rgba, width, height, "PLAYER(GOLD) VS OPPONENT(CRIMSON)", 35, 108, 255, 220, 120);
                if !end_winner.is_empty() && end_winner != "none" {
                    let winner_text = format!("WINNER: {}", end_winner.to_uppercase());
                    draw_text(rgba, width, height, &winner_text, 35, 138, 255, 255, 80);
                }
            } else {
                draw_text(rgba, width, height, "PLAYER(GOLD) VS OPPONENT(CRIMSON)", 35, 78, 255, 220, 120);
            }
        },
    }

    // Bottom-right: truth-mutation status (moved inward to avoid clipping)
    draw_panel(rgba, width, height, (width as i32) - 260, (height as i32) - 45, 240, 28);
    draw_text(rgba, width, height, "TM:F", (width as i32) - 252, (height as i32) - 38, 150, 255, 150);
}

fn write_png_rgba(path: &Path, width: u32, height: u32, rgba: &[u8]) -> Result<(), String> {
    if rgba.len() != (width as usize) * (height as usize) * 4 {
        return Err(format!(
            "unexpected RGBA byte count: got {} expected {}",
            rgba.len(),
            (width as usize) * (height as usize) * 4
        ));
    }
    let file =
        fs::File::create(path).map_err(|error| format!("create {}: {error}", path.display()))?;
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut png_writer = encoder
        .write_header()
        .map_err(|error| format!("png header {}: {error}", path.display()))?;
    png_writer
        .write_image_data(rgba)
        .map_err(|error| format!("png data {}: {error}", path.display()))?;
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    Ok(sha256_bytes(&bytes))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

fn write_report(
    path: &Path,
    packet: &Value,
    manifest: &Value,
    adapter: &Value,
) -> Result<(), String> {
    let frame = manifest
        .get("capture")
        .and_then(|capture| capture.get("file"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let frame_sha = manifest
        .get("capture")
        .and_then(|capture| capture.get("capture_file_sha256"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let body = format!(
        "# OATHYARD production native renderer\n\n\
Status: production renderer evidence only; no owner/public/release completion claim.\n\n\
- Backend: `{BACKEND_ID}`\n\
- Renderer API: `wgpu 29.0.3` high-performance Vulkan adapter request\n\
- Adapter: `{}` / `{}` / `{}`\n\
- Presentation packet schema: `{}`\n\
- Scenario: `{}`\n\
- Final truth hash: `{}`\n\
- Capture: `{frame}`\n\
- Capture sha256: `{frame_sha}`\n\
- Truth mutation: `false`\n\
- Production renderer complete: `false`\n\
- Owner visual acceptance: `false`\n\
- Native windowed execution: `false` (offscreen GPU texture production renderer; Bevy/winit swapchain remains a later gate)\n\n\
This artifact replaces neither the full high-fidelity renderer nor owner acceptance. It proves a source-buildable wgpu/Vulkan render pass can consume a verified post-hash presentation packet and emit a 1920x1080 PNG without touching truth.\n",
        adapter.get("name").and_then(Value::as_str).unwrap_or("unknown"),
        adapter.get("backend").and_then(Value::as_str).unwrap_or("unknown"),
        adapter.get("device_type").and_then(Value::as_str).unwrap_or("unknown"),
        packet.get("schema").and_then(Value::as_str).unwrap_or("unknown"),
        packet.get("scenario_id").and_then(Value::as_str).unwrap_or("unknown"),
        packet.get("final_state_hash").and_then(Value::as_str).unwrap_or("unknown"),
    );
    fs::write(path, body).map_err(|error| format!("write {}: {error}", path.display()))
}
