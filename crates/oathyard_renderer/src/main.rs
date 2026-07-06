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

// Unit-104: MotionBricks — procedural animation interpolation engine.
// Transitions between poses smoothly over a configurable number of frames.
struct MotionBrick {
    prev_pose: PoseUniform,
    target_pose: PoseUniform,
    elapsed_frames: u32,
    duration_frames: u32,
    clip_name: String,
}

impl MotionBrick {
    fn new() -> Self {
        let neutral = PoseUniform {
            pose_active: 1.0,
            pose_time: 0.5,
            _pad: [0.0; 2],
            bone_offset_x: [0.0; 4],
            bone_offset_x2: [0.0; 4],
            bone_offset_y: [0.0; 4],
            bone_offset_y2: [0.0; 4],
            bone_offset_z: [0.0; 4],
            bone_offset_z2: [0.0; 4],
            bone_yaw: [0.0; 4],
            bone_yaw2: [0.0; 4],
        };
        MotionBrick {
            prev_pose: neutral,
            target_pose: neutral,
            elapsed_frames: 0,
            duration_frames: 30, // ~0.5s at 60fps
            clip_name: String::new(),
        }
    }

    fn start_transition(&mut self, target: PoseUniform, new_clip: &str, duration: u32) {
        self.prev_pose = self.current_pose();
        self.target_pose = target;
        self.elapsed_frames = 0;
        self.duration_frames = duration;
        self.clip_name = new_clip.to_string();
    }

    fn advance(&mut self, frames: u32) {
        self.elapsed_frames = self.elapsed_frames.saturating_add(frames);
        if self.elapsed_frames >= self.duration_frames {
            self.elapsed_frames = self.duration_frames;
        }
        self.target_pose.pose_time = (self.elapsed_frames as f32) / (self.duration_frames as f32);
    }

    fn is_complete(&self) -> bool {
        self.elapsed_frames >= self.duration_frames
    }

    fn current_pose(&self) -> PoseUniform {
        let t = if self.duration_frames > 0 {
            // Smoothstep for ease-in-out
            let raw = (self.elapsed_frames as f32) / (self.duration_frames as f32);
            raw * raw * (3.0 - 2.0 * raw)
        } else {
            1.0
        };
        blend_pose(&self.prev_pose, &self.target_pose, t)
    }
}

fn blend_pose(a: &PoseUniform, b: &PoseUniform, t: f32) -> PoseUniform {
    fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }
    fn lerp_arr(a: &[f32; 4], b: &[f32; 4], t: f32) -> [f32; 4] {
        [lerp(a[0], b[0], t), lerp(a[1], b[1], t), lerp(a[2], b[2], t), lerp(a[3], b[3], t)]
    }
    PoseUniform {
        pose_active: lerp(a.pose_active, b.pose_active, t),
        pose_time: lerp(a.pose_time, b.pose_time, t),
        _pad: [0.0; 2],
        bone_offset_x: lerp_arr(&a.bone_offset_x, &b.bone_offset_x, t),
        bone_offset_x2: lerp_arr(&a.bone_offset_x2, &b.bone_offset_x2, t),
        bone_offset_y: lerp_arr(&a.bone_offset_y, &b.bone_offset_y, t),
        bone_offset_y2: lerp_arr(&a.bone_offset_y2, &b.bone_offset_y2, t),
        bone_offset_z: lerp_arr(&a.bone_offset_z, &b.bone_offset_z, t),
        bone_offset_z2: lerp_arr(&a.bone_offset_z2, &b.bone_offset_z2, t),
        bone_yaw: lerp_arr(&a.bone_yaw, &b.bone_yaw, t),
        bone_yaw2: lerp_arr(&a.bone_yaw2, &b.bone_yaw2, t),
    }
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
    // Unit-098: All 13 actions have distinct visible poses
    match clip_id {
        "idle" => {
            pose.bone_offset_y[1] = 0.016; // spine
            pose.bone_offset_y[2] = 0.010; // head
        }
        "walk" => {
            pose.bone_offset_z2[1] = 0.04;  // right leg
            pose.bone_offset_z2[2] = -0.04; // left leg
            pose.bone_yaw2[1] = 0.20;
            pose.bone_yaw2[2] = -0.20;
        }
        "guard_pose" => {
            pose.bone_yaw[3] = -0.70;  // right arm raised to guard
            pose.bone_yaw2[0] = 0.70;  // left arm supports weapon
            pose.bone_offset_y[3] = 0.06;
            pose.bone_offset_y2[0] = 0.06;
            pose.bone_offset_z[3] = 0.03;
            pose.bone_offset_z2[0] = 0.03;
            pose.bone_offset_y[1] = 0.010;
        }
        "cut" => {
            // Diagonal cut — right arm swings down-across with torso twist
            pose.bone_yaw[3] = -1.50;
            pose.bone_offset_z[3] = 0.10;
            pose.bone_offset_y[3] = -0.04;
            pose.bone_yaw[1] = 0.44;
            pose.bone_offset_z2[1] = 0.06;
            pose.bone_offset_z2[2] = -0.03;
        }
        "thrust" => {
            // Straight thrust — arms forward, weight shifts forward
            pose.bone_offset_z[3] = 0.16;
            pose.bone_offset_z2[0] = 0.12;
            pose.bone_offset_y[3] = 0.08;
            pose.bone_offset_y2[0] = 0.08;
            pose.bone_yaw[3] = -0.30;
            pose.bone_offset_z2[1] = 0.08;
            pose.bone_offset_z2[2] = -0.06;
            pose.bone_yaw[1] = 0.16;
        }
        "recover" => {
            pose.bone_yaw[3] = -0.40;
            pose.bone_offset_y[3] = 0.030;
            pose.bone_offset_z[3] = 0.010;
            pose.bone_offset_y[1] = 0.006;
        }
        "attack" => {
            pose.bone_yaw[3] = -1.20;
            pose.bone_offset_z[3] = 0.06;
            pose.bone_yaw[1] = 0.30;
        }
        // Unit-098: New distinct poses for remaining actions
        "step" => {
            // Lateral step — weight shift to right, legs spread
            pose.bone_offset_x[1] = 0.08;     // root shifts right
            pose.bone_offset_x2[1] = 0.10;    // right leg steps out
            pose.bone_offset_x2[2] = -0.04;   // left leg trails
            pose.bone_offset_y[1] = 0.012;    // slight bob
            pose.bone_yaw[1] = 0.08;          // slight torso turn
        }
        "pivot" => {
            // Pivot turn — torso rotates, weight on back foot
            pose.bone_yaw[1] = 0.90;          // strong torso rotation
            pose.bone_yaw[2] = 0.40;          // head follows
            pose.bone_offset_x2[2] = -0.06;   // left leg pivots back
            pose.bone_offset_y[1] = 0.008;
        }
        "parry" => {
            // Deflection — weapon up and angled, body slightly back
            pose.bone_yaw[3] = -1.10;         // right arm raises weapon high
            pose.bone_offset_y[3] = 0.12;     // weapon up
            pose.bone_offset_z[3] = -0.04;    // slightly back
            pose.bone_yaw2[0] = 0.90;         // left arm mirrors
            pose.bone_offset_y2[0] = 0.10;
            pose.bone_offset_z[1] = -0.03;    // lean back slightly
        }
        "brace" => {
            // Low brace — crouch with weapon low and forward
            pose.bone_offset_y[1] = -0.06;    // spine lowers (crouch)
            pose.bone_offset_y[2] = -0.04;    // head drops
            pose.bone_offset_z[3] = 0.04;     // weapon forward low
            pose.bone_offset_y[3] = -0.04;
            pose.bone_offset_z2[1] = -0.04;   // legs brace wide
            pose.bone_offset_z2[2] = 0.04;
        }
        "bash" => {
            // Shield/weapon bash — arms thrust forward together, body lunges
            pose.bone_offset_z[3] = 0.14;     // right arm forward
            pose.bone_offset_z2[0] = 0.14;    // left arm forward
            pose.bone_offset_y[3] = 0.04;     // chest height
            pose.bone_offset_y2[0] = 0.04;
            pose.bone_offset_z[1] = 0.08;     // torso leans into bash
            pose.bone_offset_z2[1] = 0.10;    // right leg lunges
            pose.bone_offset_z2[2] = -0.04;
        }
        "hook_bind" => {
            // Hook/bind — arms crossed close, weapon entangled
            pose.bone_yaw[3] = 0.60;          // right arm crosses inward
            pose.bone_yaw2[0] = -0.60;        // left arm crosses inward
            pose.bone_offset_z[3] = 0.06;     // close to body
            pose.bone_offset_z2[0] = 0.06;
            pose.bone_offset_y[1] = 0.014;    // slight rise from tension
        }
        "grab" => {
            // Grab — both arms reach forward, claws open
            pose.bone_offset_z[3] = 0.12;     // right arm reaches
            pose.bone_offset_z2[0] = 0.12;    // left arm reaches
            pose.bone_offset_y[3] = 0.02;     // low reach
            pose.bone_offset_y2[0] = 0.02;
            pose.bone_offset_z[1] = 0.06;     // torso leans forward
            pose.bone_offset_z2[1] = 0.06;    // lunge
        }
        "shove" => {
            // Shove — palms forward pushing, full body extension
            pose.bone_offset_z[3] = 0.10;     // arms extend forward
            pose.bone_offset_z2[0] = 0.10;
            pose.bone_offset_y[3] = 0.06;     // chest height
            pose.bone_offset_y2[0] = 0.06;
            pose.bone_offset_z[1] = 0.10;     // torso pushes
            pose.bone_offset_z2[1] = 0.08;
        }
        "kick" => {
            // Kick — right leg extends forward, arms back for balance
            pose.bone_offset_z2[1] = 0.18;    // right leg kicks forward hard
            pose.bone_offset_y2[1] = 0.08;    // leg raised
            pose.bone_offset_z[3] = -0.06;    // arms back
            pose.bone_offset_z2[0] = -0.06;
            pose.bone_offset_y[1] = -0.04;    // torso leans back for balance
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
    let result = match asset_id {
        // Player variants (warm skin gold tint)
        id if id == "player_fighter_mannequin" => MeshMaterial {
            material_type: -1.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.82, tint_g: 0.62, tint_b: 0.40, tint_a: 1.0,
        },
        id if id == "player_gambeson" => MeshMaterial {
            material_type: 1.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.72, tint_g: 0.46, tint_b: 0.24, tint_a: 1.0,
        },
        id if id == "player_longsword" => MeshMaterial {
            material_type: 0.0,
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
            material_type: 1.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.48, tint_g: 0.22, tint_b: 0.18, tint_a: 1.0,
        },
        id if id == "opponent_longsword" => MeshMaterial {
            material_type: 0.0,
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
            material_type: 1.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.66, tint_g: 0.42, tint_b: 0.24, tint_a: 1.0,
        },
        id if id == "witness_stone" => MeshMaterial {
            material_type: 3.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.42, tint_g: 0.38, tint_b: 0.35, tint_a: 1.0,
        },
        // Unit-066: Rigged saltreach_duelist — has real PBR materials/textures
        id if id == "player_saltreach" || id == "player_saltreach_duelist" => MeshMaterial {
            material_type: 4.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.95, tint_g: 0.72, tint_b: 0.22, tint_a: 1.0,  // stronger gold
        },
        id if id == "opponent_saltreach" || id == "opponent_saltreach_duelist" => MeshMaterial {
            material_type: 4.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.92, tint_g: 0.18, tint_b: 0.12, tint_a: 1.0,  // stronger crimson
        },
        // Unit-095: AAA Meshy asset material overrides
        id if id == "player_duelist_gold_aaa" => MeshMaterial {
            material_type: 4.0, _pad: [0.0, 0.0, 0.0],
            tint_r: 0.95, tint_g: 0.70, tint_b: 0.20, tint_a: 1.0,  // player gold
        },
        id if id == "opponent_heavy_crimson_aaa" => MeshMaterial {
            material_type: 4.0, _pad: [0.0, 0.0, 0.0],
            tint_r: 0.90, tint_g: 0.15, tint_b: 0.10, tint_a: 1.0,  // opponent crimson
        },
        id if id == "verdict_ring_aaa" => MeshMaterial {
            material_type: 3.0, _pad: [0.0, 0.0, 0.0],
            tint_r: 0.48, tint_g: 0.42, tint_b: 0.38, tint_a: 1.0,  // arena stone
        },
        // Generic material_type branches for the full local Meshy/Rodin candidate family.
        id if id.contains("saltreach_duelist")
            || id.contains("oathyard_writ")
            || id.contains("chainbreaker")
            || id.contains("reed_sentinel")
            || id.contains("gate_shield")
            || id.contains("bruiser_oath") => MeshMaterial {
            material_type: 4.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: if id.contains("opponent_") { 0.90 } else { 0.95 },
            tint_g: if id.contains("opponent_") { 0.15 } else { 0.70 },
            tint_b: if id.contains("opponent_") { 0.10 } else { 0.20 },
            tint_a: 1.0,
        },
        id if id.contains("gambeson")
            || id.contains("mail_hauberk")
            || id.contains("heavy_plate")
            || id.contains("lamellar")
            || id.contains("fencer_light")
            || id.contains("bruiser_padded_plate") => MeshMaterial {
            material_type: 2.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: if id.contains("opponent_") { 0.62 } else { 0.82 },
            tint_g: if id.contains("opponent_") { 0.36 } else { 0.56 },
            tint_b: if id.contains("opponent_") { 0.30 } else { 0.38 },
            tint_a: 1.0,
        },
        id if id.contains("longsword")
            || id.contains("curved_sword")
            || id.contains("bearded_axe")
            || id.contains("ash_spear")
            || id.contains("round_shield")
            || id.contains("iron_maul")
            || id.contains("arming_sword")
            || id.contains("billhook") => MeshMaterial {
            material_type: 0.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.85, tint_g: 0.83, tint_b: 0.90, tint_a: 1.0,
        },
        id if id.contains("witness_stone")
            || id.contains("oathyard_verdict_ring") => MeshMaterial {
            material_type: 3.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.48, tint_g: 0.42, tint_b: 0.38, tint_a: 1.0,
        },
        id if id.contains("training_yard") => MeshMaterial {
            material_type: -1.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.72, tint_g: 0.68, tint_b: 0.58, tint_a: 1.0,
        },
        _ => MeshMaterial { 
            material_type: 0.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.62, 
            tint_g: 0.58, 
            tint_b: 0.54, 
            tint_a: 1.0 
        },
    };
    result
}

// Unit-095: Minimal dummy bind_group0 for the initial GpuMeshResource construction.
// Replaced by per-mesh bind groups after collection.
fn create_dummy_bind_group0(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    uniform_buffer: &wgpu::Buffer,
    camera_buffer: &wgpu::Buffer,
    pose_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    let dummy_mat = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("dummy per-mesh material buffer"),
        size: 32,
        usage: wgpu::BufferUsages::UNIFORM,
        mapped_at_creation: true,
    });
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("dummy per-mesh bind group 0"),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: camera_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: dummy_mat.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: pose_buffer.as_entire_binding() },
        ],
    })
}

fn camera_for_mode(mode: &str) -> CameraMode {
    match mode {
        "boot_main_menu" => CameraMode { eye: [0.0, 1.8, 4.5], look_at: [0.0, 0.8, -0.5], fov_radians: 0.72 },
        "fighter_select" => CameraMode { eye: [-0.6, 1.15, 2.6], look_at: [-0.3, 0.50, -0.1], fov_radians: 0.68 },
        "loadout_select" => CameraMode { eye: [0.0, 0.75, 2.2], look_at: [0.0, 0.35, -0.1], fov_radians: 0.65 },
        "fighter_closeup_01" => CameraMode { eye: [0.0, 0.90, 2.0], look_at: [0.0, 0.45, -0.1], fov_radians: 0.58 },
        "armor_loadout_family_closeup_01" => CameraMode { eye: [0.0, 0.70, 1.9], look_at: [0.0, 0.32, -0.1], fov_radians: 0.60 },
        "weapon_family_closeup_01" => CameraMode { eye: [0.0, 0.55, 1.6], look_at: [0.0, 0.30, -0.1], fov_radians: 0.55 },
        "oathyard_verdict_ring_establishing" => CameraMode { eye: [0.0, 1.2, 3.8], look_at: [0.0, 0.25, -0.1], fov_radians: 0.78 },
        "oathyard_arena_candidate_01" => CameraMode { eye: [0.0, 0.55, 3.35], look_at: [0.0, 0.18, -0.10], fov_radians: 0.78 },
        "gameplay_distance_fighter_weapon_01" => CameraMode { eye: [0.0, 1.2, 3.8], look_at: [0.0, 0.35, -0.1], fov_radians: 0.70 },
        "gameplay_distance_fighter_loadout_family_01" => CameraMode { eye: [0.0, 1.15, 4.0], look_at: [0.0, 0.30, -0.1], fov_radians: 0.72 },
        "gameplay_distance_weapon_family_01" => CameraMode { eye: [0.0, 0.90, 3.2], look_at: [0.05, 0.35, -0.1], fov_radians: 0.68 },
        "pre_contact_frame" => CameraMode { eye: [0.0, 1.0, 3.8], look_at: [0.0, 0.30, -0.1], fov_radians: 0.75 },
        "contact_frame" => CameraMode { eye: [0.0, 0.90, 3.2], look_at: [0.0, 0.28, -0.05], fov_radians: 0.72 },
        "fight_film_candidate_shot_01" => CameraMode { eye: [0.35, 1.2, 3.2], look_at: [0.0, 0.30, -0.15], fov_radians: 0.66 },
        "fight_film_replay_camera_shot" => CameraMode { eye: [-0.3, 1.1, 2.8], look_at: [0.05, 0.35, -0.1], fov_radians: 0.64 },
        // Unit-051: production-ready-candidate capture cameras
        "planning_timeline" => CameraMode { eye: [0.0, 1.1, 3.4], look_at: [0.0, 0.40, -0.1], fov_radians: 0.68 },
        "material_armor_damage_frame" => CameraMode { eye: [0.1, 0.65, 1.8], look_at: [0.0, 0.30, -0.05], fov_radians: 0.55 },
        "injury_capability_consequence_frame" => CameraMode { eye: [-0.15, 0.85, 2.2], look_at: [0.0, 0.35, -0.1], fov_radians: 0.58 },
        // Unit-052: expanded capture cameras
        "training_yard_establishing" => CameraMode { eye: [0.0, 1.6, 5.0], look_at: [0.0, 0.1, -0.3], fov_radians: 0.78 },
        "recovery_replan_frame" => CameraMode { eye: [-0.2, 0.90, 2.6], look_at: [0.0, 0.38, -0.1], fov_radians: 0.62 },
        "first_person_combat_view" => CameraMode { eye: [0.0, 1.1, 3.5], look_at: [0.0, 0.35, -0.1], fov_radians: 0.75 },
        "third_person_combat_view" => CameraMode { eye: [0.0, 1.2, 4.0], look_at: [0.0, 0.30, -0.1], fov_radians: 0.78 },
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
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--windowed") {
        if let Err(error) = windowed_main() {
            eprintln!("oathyard-native-renderer (windowed): {error}");
            std::process::exit(1);
        }
        return;
    }
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
    let mut no_aaa_override = false;
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
            "--no-aaa-override" => {
                no_aaa_override = true;
            }
            "--help" | "-h" => {
                println!("usage: oathyard-native-renderer --packet post_hash_presentation_packet.json --out <dir> [--capture-id <id>] [--capture-file-stem <production_renderer_*.png stem>] [--camera-mode <mode>] [--candidate-assets comma,separated,ids] [--asset-manifest-sha256 <sha256>] [--mesh-json assets/runtime/candidate/<id>.mesh.json] [--mesh-manifest-json <mesh-manifest.json>] [--no-aaa-override]");
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
    // Unit-095: Always load AAA Meshy assets when the manifest exists
    // Unit-103: Skip AAA override when --no-aaa-override is passed (per-asset capture)
    let aaa_manifest = std::env::current_dir()
        .map(|p| p.join("assets/manifests/aaa_mesh_manifest.json"))
        .unwrap_or_else(|_| std::path::PathBuf::from("assets/manifests/aaa_mesh_manifest.json"));
    if !no_aaa_override && aaa_manifest.exists() {
        // Unit-096: Replace old fighters/arena with AAA, keep weapons/armor
        mesh_specs.retain(|s| {
            let cls = infer_mesh_asset_class(&s.mesh_asset_id);
            cls == "weapon" || cls == "armor"
        });
        mesh_specs.extend(load_runtime_mesh_manifest(&aaa_manifest)?);
    }
    let runtime_meshes = mesh_specs
        .iter()
        .map(|s| load_runtime_mesh_with_clip(s, clip_id_for_capture(&capture_id)))
        .collect::<Result<Vec<_>, _>>()?;
    let mut seed = seed_uniforms(&packet_json, &capture_id, &candidate_assets);
    // Unit-095: Set seed.z = 1.0 when runtime meshes are loaded.
    // This tells the SDF shader to hide procedural fighters so the
    // mesh-rendered team-colored fighters are visible.
    if !runtime_meshes.is_empty() {
        seed[2] = 1.0;
    }
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
    composite_ui_overlay(&mut render.rgba, WIDTH, HEIGHT, &capture_id, &packet_json, &candidate_assets);
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

fn load_runtime_mesh_with_clip(spec: &RuntimeMeshSpec, clip_id: &str) -> Result<RuntimeMesh, String> {
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
    // Unit-068: Load normals from JSON if available (skinned meshes have source normals)
    let mut mesh_normals: Vec<[f32; 3]> = Vec::new();
    if let Some(normals_json) = data.get("normals").and_then(Value::as_array) {
        for n_val in normals_json {
            if let Some(n_arr) = n_val.as_array() {
                if n_arr.len() >= 3 {
                    mesh_normals.push([
                        n_arr[0].as_f64().unwrap_or(0.0) as f32,
                        n_arr[1].as_f64().unwrap_or(0.0) as f32,
                        n_arr[2].as_f64().unwrap_or(0.0) as f32,
                    ]);
                }
            }
        }
    }
    let mut mesh_texcoords: Vec<[f32; 2]> = Vec::new();
    let texcoord_json = data
        .get("texcoords")
        .or_else(|| data.get("uvs"))
        .and_then(Value::as_array);
    if let Some(texcoords_json) = texcoord_json {
        for uv_val in texcoords_json {
            if let Some(uv_arr) = uv_val.as_array() {
                if uv_arr.len() >= 2 {
                    mesh_texcoords.push([
                        uv_arr[0].as_f64().unwrap_or(0.0) as f32,
                        uv_arr[1].as_f64().unwrap_or(0.0) as f32,
                    ]);
                }
            }
        }
    }
    let mut mesh_material_colors: Vec<[f32; 3]> = Vec::new();
    if let Some(colors_json) = data.get("material_colors").and_then(Value::as_array) {
        for color_val in colors_json {
            if let Some(color_arr) = color_val.as_array() {
                if color_arr.len() >= 3 {
                    mesh_material_colors.push([
                        color_arr[0].as_f64().unwrap_or(1.0) as f32,
                        color_arr[1].as_f64().unwrap_or(1.0) as f32,
                        color_arr[2].as_f64().unwrap_or(1.0) as f32,
                    ]);
                }
            }
        }
    }

    // Unit-068: Apply CPU-side skinning deformation from glTF animation data
    if mesh_normals.len() == positions.len() && !clip_id.is_empty() {
        let _ = apply_skinned_deformation(&mut positions, &mut mesh_normals, &data, clip_id);
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
        .enumerate()
        .map(|(vi, position)| {
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
                // Unit-100: Set vertex color to team tint for fighter meshes.
                // This bypasses the uniform buffer pitfall and ensures each
                // fighter mesh gets its correct team color via vertex attribute.
                material_uv: if vi < mesh_texcoords.len() {
                    [wrap01(mesh_texcoords[vi][0]), wrap01(mesh_texcoords[vi][1])]
                } else {
                    [
                        wrap01(local[0] * 0.58 + 0.21),
                        wrap01(local[1] * 0.58 + 0.21),
                    ]
                },
                color: {
                    // Unit-100: Use team tint as vertex color for fighter body meshes
                    let mat = material_for_mesh(&spec.mesh_asset_id);
                    if mat.material_type > 3.5 && mat.material_type < 4.5 {
                        [mat.tint_r, mat.tint_g, mat.tint_b]
                    } else if vi < mesh_material_colors.len() {
                        [
                            (mesh_material_colors[vi][0] + 0.04 * local[2].abs().min(1.0)).min(1.25),
                            (mesh_material_colors[vi][1] + 0.04 * local[1].abs().min(1.0)).min(1.25),
                            (mesh_material_colors[vi][2] + 0.04 * local[0].abs().min(1.0)).min(1.25),
                        ]
                    } else {
                        [
                            base_color[0] + 0.05 * local[2].abs().min(1.0),
                            base_color[1] + 0.05 * local[1].abs().min(1.0),
                            base_color[2] + 0.05 * local[0].abs().min(1.0),
                        ]
                    }
                },
                // Normals: use source normals if available (set later), otherwise computed.
                normal: if vi < mesh_normals.len() { mesh_normals[vi] } else { [0.0, 0.0, 0.0] },
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
    // Unit-062/068: Compute per-vertex flat normals only if no source normals were used.
    if mesh_normals.is_empty() {
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
    } // end if mesh_normals.is_empty()
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


// Unit-068: CPU-side glTF animation sampling and skinning deformation.

fn quat_slerp(q0: [f32; 4], q1: [f32; 4], t: f32) -> [f32; 4] {
    let dot = q0[0]*q1[0] + q0[1]*q1[1] + q0[2]*q1[2] + q0[3]*q1[3];
    let (q1x, q1y, q1z, q1w) = if dot < 0.0 {
        (-q1[0], -q1[1], -q1[2], -q1[3])
    } else {
        (q1[0], q1[1], q1[2], q1[3])
    };
    let dot_abs = dot.abs();
    if dot_abs > 0.9995 {
        // Linear interpolation for nearly-parallel quaternions
        return [
            q0[0] + t * (q1x - q0[0]),
            q0[1] + t * (q1y - q0[1]),
            q0[2] + t * (q1z - q0[2]),
            q0[3] + t * (q1w - q0[3]),
        ];
    }
    let theta = dot_abs.acos();
    let sin_theta = theta.sin();
    let sin_t = (t * theta).sin();
    let sin_1mt = ((1.0 - t) * theta).sin();
    let s0 = sin_1mt / sin_theta;
    let s1 = if dot < 0.0 { -sin_t / sin_theta } else { sin_t / sin_theta };
    [
        s0 * q0[0] + s1 * q1x,
        s0 * q0[1] + s1 * q1y,
        s0 * q0[2] + s1 * q1z,
        s0 * q0[3] + s1 * q1w,
    ]
}

fn quat_to_mat4(q: [f32; 4]) -> [f32; 16] {
    let (x, y, z, w) = (q[0], q[1], q[2], q[3]);
    let xx = x * x; let yy = y * y; let zz = z * z;
    let xy = x * y; let xz = x * z; let yz = y * z;
    let wx = w * x; let wy = w * y; let wz = w * z;
    [
        1.0 - 2.0 * (yy + zz), 2.0 * (xy + wz), 2.0 * (xz - wy), 0.0,
        2.0 * (xy - wz), 1.0 - 2.0 * (xx + zz), 2.0 * (yz + wx), 0.0,
        2.0 * (xz + wy), 2.0 * (yz - wx), 1.0 - 2.0 * (xx + yy), 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
}

fn mat4_mul(a: [f32; 16], b: [f32; 16]) -> [f32; 16] {
    let mut r = [0.0f32; 16];
    for i in 0..4 {
        for j in 0..4 {
            let mut sum = 0.0;
            for k in 0..4 {
                sum += a[i * 4 + k] * b[k * 4 + j];
            }
            r[i * 4 + j] = sum;
        }
    }
    r
}

fn mat4_from_trs(t: [f32; 3], r: [f32; 4], s: [f32; 3]) -> [f32; 16] {
    let rot_mat = quat_to_mat4(r);
    let mut result = [
        rot_mat[0] * s[0], rot_mat[1] * s[1], rot_mat[2] * s[2], 0.0,
        rot_mat[4] * s[0], rot_mat[5] * s[1], rot_mat[6] * s[2], 0.0,
        rot_mat[8] * s[0], rot_mat[9] * s[1], rot_mat[10] * s[2], 0.0,
        t[0], t[1], t[2], 1.0,
    ];
    let _ = &mut result;
    result
}

fn transform_point(m: [f32; 16], p: [f32; 3]) -> [f32; 3] {
    [
        m[0] * p[0] + m[4] * p[1] + m[8] * p[2] + m[12],
        m[1] * p[0] + m[5] * p[1] + m[9] * p[2] + m[13],
        m[2] * p[0] + m[6] * p[1] + m[10] * p[2] + m[14],
    ]
}

fn transform_direction(m: [f32; 16], d: [f32; 3]) -> [f32; 3] {
    [
        m[0] * d[0] + m[4] * d[1] + m[8] * d[2],
        m[1] * d[0] + m[5] * d[1] + m[9] * d[2],
        m[2] * d[0] + m[6] * d[1] + m[10] * d[2],
    ]
}

/// Apply CPU-side skinning deformation using glTF animation data.
/// Modifies vertex positions and normals in-place based on joint weights.
fn apply_skinned_deformation(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    data: &Value,
    clip_id: &str,
) -> Result<bool, String> {
    // Check if this mesh has skinning data
    let joints_arr = match data.get("joints").and_then(Value::as_array) {
        Some(j) if !j.is_empty() => j,
        _ => return Ok(false), // No skinning data
    };
    let weights_arr = data.get("weights").and_then(Value::as_array)
        .ok_or("skinned mesh missing weights")?;
    let ibms_arr = data.get("inverse_bind_matrices").and_then(Value::as_array)
        .ok_or("skinned mesh missing inverse_bind_matrices")?;
    let joint_names_arr = data.get("joint_names").and_then(Value::as_array)
        .ok_or("skinned mesh missing joint_names")?;
    let node_transforms_arr = data.get("node_transforms").and_then(Value::as_array)
        .ok_or("skinned mesh missing node_transforms")?;
    let animations_arr = data.get("animations").and_then(Value::as_array)
        .ok_or("skinned mesh missing animations")?;

    let joint_count = joint_names_arr.len();
    if joint_count == 0 {
        return Ok(false);
    }

    // Parse IBMs (each is 16 floats in row-major order)
    let mut ibms = Vec::with_capacity(joint_count);
    for i in 0..joint_count {
        let row = ibms_arr[i].as_array().ok_or("IBM not array")?;
        let mut m = [0.0f32; 16];
        for j in 0..16.min(row.len()) {
            m[j] = row[j].as_f64().unwrap_or(0.0) as f32;
        }
        ibms.push(m);
    }

    // Parse bind-pose node local transforms
    let mut bind_transforms = Vec::with_capacity(joint_count);
    for i in 0..joint_count {
        let node = &node_transforms_arr[i];
        let t: [f32; 3] = {
            let arr = node.get("translation").and_then(Value::as_array);
            match arr {
                Some(a) if a.len() >= 3 => [a[0].as_f64().unwrap_or(0.0) as f32, a[1].as_f64().unwrap_or(0.0) as f32, a[2].as_f64().unwrap_or(0.0) as f32],
                _ => [0.0, 0.0, 0.0],
            }
        };
        let r: [f32; 4] = {
            let arr = node.get("rotation").and_then(Value::as_array);
            match arr {
                Some(a) if a.len() >= 4 => [a[0].as_f64().unwrap_or(0.0) as f32, a[1].as_f64().unwrap_or(0.0) as f32, a[2].as_f64().unwrap_or(0.0) as f32, a[3].as_f64().unwrap_or(1.0) as f32],
                _ => [0.0, 0.0, 0.0, 1.0],
            }
        };
        let s: [f32; 3] = {
            let arr = node.get("scale").and_then(Value::as_array);
            match arr {
                Some(a) if a.len() >= 3 => [a[0].as_f64().unwrap_or(1.0) as f32, a[1].as_f64().unwrap_or(1.0) as f32, a[2].as_f64().unwrap_or(1.0) as f32],
                _ => [1.0, 1.0, 1.0],
            }
        };
        bind_transforms.push((t, r, s));
    }

    // Parse joint hierarchy (parent-child relationships)
    let mut joint_parents = vec![-1i32; joint_count];
    for i in 0..joint_count {
        let node = &node_transforms_arr[i];
        let children = node.get("children").and_then(Value::as_array);
        if let Some(children) = children {
            for child_val in children {
                let child_idx = child_val.as_i64().unwrap_or(-1);
                if child_idx >= 0 && (child_idx as usize) < joint_count {
                    joint_parents[child_idx as usize] = i as i32;
                }
            }
        }
    }

    // Apply animation if clip exists, otherwise use bind pose
    let mut animated_rotations: std::collections::HashMap<String, [f32; 4]> = std::collections::HashMap::new();
    let mut animated_translations: std::collections::HashMap<String, [f32; 3]> = std::collections::HashMap::new();

    // Find the matching animation clip
    let clip_to_find = match clip_id {
        "idle" => "idle",
        "walk" => "walk",
        "attack" | "cut" | "thrust" | "guard_pose" | "recover" => "attack",
        _ => "idle",
    };

    let sample_time = match clip_id {
        "idle" => 0.5,
        "walk" => 0.3,
        "attack" => 0.3,
        "cut" | "thrust" => 0.4,
        "guard_pose" => 0.1,
        "recover" => 0.7,
        _ => 0.5,
    };

    for anim_val in animations_arr {
        let anim_name = anim_val.get("name").and_then(Value::as_str).unwrap_or("");
        if anim_name != clip_to_find {
            continue;
        }
        let channels = anim_val.get("channels").and_then(Value::as_array).ok_or("anim missing channels")?;
        let samplers = anim_val.get("samplers").and_then(Value::as_array).ok_or("anim missing samplers")?;

        for channel in channels {
            let node_name = channel.get("node").and_then(Value::as_str).unwrap_or("");
            let path_type = channel.get("path").and_then(Value::as_str).unwrap_or("");
            let sampler_idx = channel.get("sampler").and_then(Value::as_u64).unwrap_or(0) as usize;

            if sampler_idx >= samplers.len() {
                continue;
            }
            let sampler = &samplers[sampler_idx];
            let times = sampler.get("times").and_then(Value::as_array).ok_or("sampler missing times")?;
            let values = sampler.get("values").and_then(Value::as_array).ok_or("sampler missing values")?;

            if times.len() < 2 {
                continue;
            }

            // Find the keyframe interval
            let mut seg_idx = 0;
            for (i, t_val) in times.iter().enumerate() {
                let t = t_val.as_f64().unwrap_or(0.0) as f32;
                if t <= sample_time {
                    seg_idx = i;
                } else {
                    break;
                }
            }
            let next_idx = (seg_idx + 1).min(times.len() - 1);
            let t0 = times[seg_idx].as_f64().unwrap_or(0.0) as f32;
            let t1 = times[next_idx].as_f64().unwrap_or(1.0) as f32;
            let alpha = if t1 > t0 { ((sample_time - t0) / (t1 - t0)).clamp(0.0, 1.0) } else { 0.0 };

            if path_type == "rotation" {
                let v0: [f32; 4] = {
                    let arr = values[seg_idx].as_array().ok_or("rot value not array")?;
                    [arr[0].as_f64().unwrap_or(0.0) as f32, arr[1].as_f64().unwrap_or(0.0) as f32, arr[2].as_f64().unwrap_or(0.0) as f32, arr[3].as_f64().unwrap_or(1.0) as f32]
                };
                let v1: [f32; 4] = {
                    let arr = values[next_idx].as_array().ok_or("rot value not array")?;
                    [arr[0].as_f64().unwrap_or(0.0) as f32, arr[1].as_f64().unwrap_or(0.0) as f32, arr[2].as_f64().unwrap_or(0.0) as f32, arr[3].as_f64().unwrap_or(1.0) as f32]
                };
                let interpolated = quat_slerp(v0, v1, alpha);
                animated_rotations.insert(node_name.to_string(), interpolated);
            } else if path_type == "translation" {
                let v0: [f32; 3] = {
                    let arr = values[seg_idx].as_array().ok_or("trans value not array")?;
                    [arr[0].as_f64().unwrap_or(0.0) as f32, arr[1].as_f64().unwrap_or(0.0) as f32, arr[2].as_f64().unwrap_or(0.0) as f32]
                };
                let v1: [f32; 3] = {
                    let arr = values[next_idx].as_array().ok_or("trans value not array")?;
                    [arr[0].as_f64().unwrap_or(0.0) as f32, arr[1].as_f64().unwrap_or(0.0) as f32, arr[2].as_f64().unwrap_or(0.0) as f32]
                };
                let interpolated = [
                    v0[0] + alpha * (v1[0] - v0[0]),
                    v0[1] + alpha * (v1[1] - v0[1]),
                    v0[2] + alpha * (v1[2] - v0[2]),
                ];
                animated_translations.insert(node_name.to_string(), interpolated);
            }
        }
        break; // Only use the first matching clip
    }

    // Compute world-space joint matrices by traversing the hierarchy
    let mut joint_world = vec![[1.0f32; 16]; joint_count];
    for i in 0..joint_count {
        let joint_name = joint_names_arr[i].as_str().unwrap_or("");
        // Get local transform (use animated value if available, otherwise bind pose)
        let (bind_t, bind_r, bind_s) = &bind_transforms[i];
        let t = animated_translations.get(joint_name).unwrap_or(bind_t);
        let r = animated_rotations.get(joint_name).unwrap_or(bind_r);
        let local_mat = mat4_from_trs(*t, *r, *bind_s);

        let parent = joint_parents[i];
        if parent >= 0 {
            joint_world[i] = mat4_mul(joint_world[parent as usize], local_mat);
        } else {
            joint_world[i] = local_mat;
        }
    }

    // Compute final skinning matrices: joint_world * IBM
    let mut skin_matrices = vec![[0.0f32; 16]; joint_count];
    for i in 0..joint_count {
        skin_matrices[i] = mat4_mul(joint_world[i], ibms[i]);
    }

    // Deform vertices using skinning matrices
    let vcount = positions.len();
    for vi in 0..vcount {
        let joint_vals = joints_arr[vi].as_array().ok_or("joint not array")?;
        let weight_vals = weights_arr[vi].as_array().ok_or("weight not array")?;
        if joint_vals.len() < 4 || weight_vals.len() < 4 {
            continue;
        }

        let j0 = joint_vals[0].as_u64().unwrap_or(0) as usize;
        let j1 = joint_vals[1].as_u64().unwrap_or(0) as usize;
        let j2 = joint_vals[2].as_u64().unwrap_or(0) as usize;
        let j3 = joint_vals[3].as_u64().unwrap_or(0) as usize;
        let w0 = weight_vals[0].as_f64().unwrap_or(0.0) as f32;
        let w1 = weight_vals[1].as_f64().unwrap_or(0.0) as f32;
        let w2 = weight_vals[2].as_f64().unwrap_or(0.0) as f32;
        let w3 = weight_vals[3].as_f64().unwrap_or(0.0) as f32;

        let original_pos = positions[vi];
        let mut deformed_pos = [0.0f32; 3];
        if j0 < joint_count && w0 > 0.001 {
            let p = transform_point(skin_matrices[j0], original_pos);
            deformed_pos[0] += w0 * p[0];
            deformed_pos[1] += w0 * p[1];
            deformed_pos[2] += w0 * p[2];
        }
        if j1 < joint_count && w1 > 0.001 {
            let p = transform_point(skin_matrices[j1], original_pos);
            deformed_pos[0] += w1 * p[0];
            deformed_pos[1] += w1 * p[1];
            deformed_pos[2] += w1 * p[2];
        }
        if j2 < joint_count && w2 > 0.001 {
            let p = transform_point(skin_matrices[j2], original_pos);
            deformed_pos[0] += w2 * p[0];
            deformed_pos[1] += w2 * p[1];
            deformed_pos[2] += w2 * p[2];
        }
        if j3 < joint_count && w3 > 0.001 {
            let p = transform_point(skin_matrices[j3], original_pos);
            deformed_pos[0] += w3 * p[0];
            deformed_pos[1] += w3 * p[1];
            deformed_pos[2] += w3 * p[2];
        }

        // Only apply deformation if the result is valid (not NaN)
        if deformed_pos[0].is_finite() && deformed_pos[1].is_finite() && deformed_pos[2].is_finite() {
            positions[vi] = deformed_pos;
        }

        // Deform normals similarly (using direction transform, not point transform)
        let original_norm = normals[vi];
        let mut deformed_norm = [0.0f32; 3];
        if j0 < joint_count && w0 > 0.001 {
            let d = transform_direction(skin_matrices[j0], original_norm);
            deformed_norm[0] += w0 * d[0];
            deformed_norm[1] += w0 * d[1];
            deformed_norm[2] += w0 * d[2];
        }
        if j1 < joint_count && w1 > 0.001 {
            let d = transform_direction(skin_matrices[j1], original_norm);
            deformed_norm[0] += w1 * d[0];
            deformed_norm[1] += w1 * d[1];
            deformed_norm[2] += w1 * d[2];
        }
        if j2 < joint_count && w2 > 0.001 {
            let d = transform_direction(skin_matrices[j2], original_norm);
            deformed_norm[0] += w2 * d[0];
            deformed_norm[1] += w2 * d[1];
            deformed_norm[2] += w2 * d[2];
        }
        if j3 < joint_count && w3 > 0.001 {
            let d = transform_direction(skin_matrices[j3], original_norm);
            deformed_norm[0] += w3 * d[0];
            deformed_norm[1] += w3 * d[1];
            deformed_norm[2] += w3 * d[2];
        }
        // Normalize
        let len = (deformed_norm[0]*deformed_norm[0] + deformed_norm[1]*deformed_norm[1] + deformed_norm[2]*deformed_norm[2]).sqrt();
        if len > 1e-10 && deformed_norm[0].is_finite() {
            normals[vi] = [deformed_norm[0]/len, deformed_norm[1]/len, deformed_norm[2]/len];
        }
    }

    Ok(true)
}


fn wrap01(value: f32) -> f32 {
    value - value.floor()
}

fn load_runtime_material(
    spec: &RuntimeMeshSpec,
    mesh_path: &Path,
    data: &Value,
) -> Result<RuntimeMaterial, String> {
    if let (Some(base_path), Some(normal_path), Some(orm_path)) = (
        spec.base_color_texture_path.clone(),
        spec.normal_texture_path.clone(),
        spec.orm_texture_path.clone(),
    ) {
        if base_path.exists() && normal_path.exists() && orm_path.exists() {
            let base_sha = sha256_file(&base_path).unwrap_or_default();
            let normal_sha = sha256_file(&normal_path).unwrap_or_default();
            let orm_sha = sha256_file(&orm_path).unwrap_or_default();
            let base_img = load_png_rgba(&base_path).ok();
            let normal_img = load_png_rgba(&normal_path).ok();
            let orm_img = load_png_rgba(&orm_path).ok();
            return Ok(RuntimeMaterial {
                material_texture_binding: true,
                base_color_texture_path: base_path,
                normal_texture_path: normal_path,
                orm_texture_path: orm_path,
                base_color_texture_sha256: base_sha.clone(),
                normal_texture_sha256: normal_sha.clone(),
                orm_texture_sha256: orm_sha.clone(),
                base_color_texture_dimensions: base_img.as_ref().map(|i| [i.width, i.height]).unwrap_or([0, 0]),
                normal_texture_dimensions: normal_img.as_ref().map(|i| [i.width, i.height]).unwrap_or([0, 0]),
                orm_texture_dimensions: orm_img.as_ref().map(|i| [i.width, i.height]).unwrap_or([0, 0]),
                material_count: 1,
                texture_hashes: json!({
                    "source": "explicit_runtime_mesh_manifest_paths",
                    "base_color": base_sha,
                    "normal": normal_sha,
                    "orm": orm_sha,
                }),
            });
        }
    }
    let material_validation = data
        .get("material_validation");
    let mat_val = match material_validation {
        Some(mv) => mv,
        None => {
            // Unit-068: Skinned meshes from generated JSON may not have material_validation.
            // Try to find textures from the candidate texture directory using the mesh asset ID.
            let tex_base = format!("assets/model_candidates/t_73291be5/textures/{}", spec.mesh_asset_id);
            let base_path = PathBuf::from(format!("{}_base.png", tex_base));
            let normal_path = PathBuf::from(format!("{}_normal.png", tex_base));
            let orm_path = PathBuf::from(format!("{}_orm.png", tex_base));
            if base_path.exists() && normal_path.exists() && orm_path.exists() {
                let base_sha = sha256_file(&base_path).unwrap_or_default();
                let normal_sha = sha256_file(&normal_path).unwrap_or_default();
                let orm_sha = sha256_file(&orm_path).unwrap_or_default();
                let base_img = load_png_rgba(&base_path).ok();
                let normal_img = load_png_rgba(&normal_path).ok();
                let orm_img = load_png_rgba(&orm_path).ok();
                return Ok(RuntimeMaterial {
                    material_texture_binding: true,
                    base_color_texture_path: base_path,
                    normal_texture_path: normal_path,
                    orm_texture_path: orm_path,
                    base_color_texture_sha256: base_sha,
                    normal_texture_sha256: normal_sha,
                    orm_texture_sha256: orm_sha,
                    base_color_texture_dimensions: base_img.as_ref().map(|i| [i.width, i.height]).unwrap_or([1024, 1024]),
                    normal_texture_dimensions: normal_img.as_ref().map(|i| [i.width, i.height]).unwrap_or([1024, 1024]),
                    orm_texture_dimensions: orm_img.as_ref().map(|i| [i.width, i.height]).unwrap_or([1024, 1024]),
                    material_count: 1,
                    texture_hashes: Value::Null,
                });
            }
            return Ok(RuntimeMaterial {
                material_texture_binding: false,
                base_color_texture_path: PathBuf::new(),
                normal_texture_path: PathBuf::new(),
                orm_texture_path: PathBuf::new(),
                base_color_texture_sha256: String::new(),
                normal_texture_sha256: String::new(),
                orm_texture_sha256: String::new(),
                base_color_texture_dimensions: [0, 0],
                normal_texture_dimensions: [0, 0],
                orm_texture_dimensions: [0, 0],
                material_count: 0,
                texture_hashes: Value::Null,
            });
        }
    };
    if mat_val
        .get("base_normal_orm_present")
        .and_then(Value::as_bool)
        != Some(true)
    {
        return Err(format!(
            "runtime mesh {} does not declare base/normal/ORM texture coverage",
            mesh_path.display()
        ));
    }
    let image_uris = mat_val
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
    let material_count = mat_val
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
    // Unit-096: Recognize AAA Meshy asset names
    if asset_id.contains("duelist_gold") || asset_id.contains("heavy_crimson") {
        return "fighter";
    }
    if asset_id.contains("verdict_ring") && asset_id.contains("aaa") {
        return "arena";
    }
    // Unit-102: Use contains() for prefixed IDs (player_saltreach_duelist, etc.)
    // Previous exact-match caused play-path fighters to be misclassified as "weapon"
    // and retained alongside AAA fighters, creating duplicate geometry.
    if asset_id.contains("saltreach_duelist")
        || asset_id.contains("oathyard_writ")
        || asset_id.contains("chainbreaker")
        || asset_id.contains("reed_sentinel")
        || asset_id.contains("gate_shield")
        || asset_id.contains("bruiser_oath")
    {
        return "fighter";
    }
    if asset_id.contains("longsword")
        || asset_id.contains("arming_sword")
        || asset_id.contains("ash_spear")
        || asset_id.contains("bearded_axe")
        || asset_id.contains("billhook")
        || asset_id.contains("curved_sword")
        || asset_id.contains("iron_maul")
        || asset_id.contains("round_shield")
    {
        return "weapon";
    }
    if asset_id.contains("gambeson")
        || asset_id.contains("mail_hauberk")
        || asset_id.contains("heavy_plate")
        || asset_id.contains("lamellar")
        || asset_id.contains("fencer_light")
        || asset_id.contains("bruiser_padded_plate")
    {
        return "armor";
    }
    if asset_id.contains("verdict_ring") || asset_id.contains("training_yard") {
        return "arena";
    }
    "_unknown"
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
    // Unit-095: Per-mesh material uniform buffer + bind_group0
    material_uniform_buffer: wgpu::Buffer,
    bind_group0: wgpu::BindGroup,
    _base_color_texture: wgpu::Texture,
    _normal_texture: wgpu::Texture,
    _orm_texture: wgpu::Texture,
}

fn depth_stencil_state(depth_write_enabled: bool) -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth32Float,
        depth_write_enabled: Some(depth_write_enabled),
        depth_compare: if depth_write_enabled {
            Some(wgpu::CompareFunction::LessEqual)
        } else {
            Some(wgpu::CompareFunction::Always)
        },
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    }
}

fn create_depth_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    label: &'static str,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    })
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
    let depth_texture = create_depth_texture(
        &device,
        WIDTH,
        HEIGHT,
        "oathyard production render depth target",
    );
    let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

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
            // Unit-049: MeshMaterial uniform as binding 2 (fixed, per-per-mesh via separate bind groups)
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
        depth_stencil: Some(depth_stencil_state(false)),
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
            depth_stencil: Some(depth_stencil_state(true)),
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
        let mut buffers = runtime_meshes
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
                // Unit-071: Only load textures when material_texture_binding is true.
                // Skinned meshes without material_validation produce empty texture paths.
                let (base_color_texture, normal_texture, orm_texture, material_texture_bound) =
                    if mesh.material.material_texture_binding {
                        let base_image = load_png_rgba(&mesh.material.base_color_texture_path)?;
                        let normal_image = load_png_rgba(&mesh.material.normal_texture_path)?;
                        let orm_image = load_png_rgba(&mesh.material.orm_texture_path)?;
                        let bt = create_material_texture(
                            &device,
                            &queue,
                            "oathyard production base color texture",
                            wgpu::TextureFormat::Rgba8UnormSrgb,
                            &base_image,
                        );
                        let nt = create_material_texture(
                            &device,
                            &queue,
                            "oathyard production normal texture",
                            wgpu::TextureFormat::Rgba8Unorm,
                            &normal_image,
                        );
                        let ot = create_material_texture(
                            &device,
                            &queue,
                            "oathyard production ORM texture",
                            wgpu::TextureFormat::Rgba8Unorm,
                            &orm_image,
                        );
                        (bt, nt, ot, true)
                    } else {
                        // Fallback: create 1x1 dummy textures for meshes without material textures.
                        let dummy = create_material_texture(
                            &device,
                            &queue,
                            "oathyard production dummy texture",
                            wgpu::TextureFormat::Rgba8UnormSrgb,
                            &RuntimeTextureImage { width: 1, height: 1, rgba: vec![255, 255, 255, 255] },
                        );
                        let dummy_norm = create_material_texture(
                            &device,
                            &queue,
                            "oathyard production dummy normal texture",
                            wgpu::TextureFormat::Rgba8Unorm,
                            &RuntimeTextureImage { width: 1, height: 1, rgba: vec![128, 128, 255, 255] },
                        );
                        let dummy_orm = create_material_texture(
                            &device,
                            &queue,
                            "oathyard production dummy ORM texture",
                            wgpu::TextureFormat::Rgba8Unorm,
                            &RuntimeTextureImage { width: 1, height: 1, rgba: vec![255, 255, 255, 255] },
                        );
                        (dummy, dummy_norm, dummy_orm, false)
                    };
                let _ = material_texture_bound;
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
                    // Unit-095: Per-mesh fields filled in later, set to dummy here
                    material_uniform_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some("dummy per-mesh init"),
                        size: 32,
                        usage: wgpu::BufferUsages::UNIFORM,
                        mapped_at_creation: true,
                    }),
                    bind_group0: create_dummy_bind_group0(&device, &bind_group_layout, &uniform_buffer, &camera_buffer, &pose_buffer),
                    _base_color_texture: base_color_texture,
                    _normal_texture: normal_texture,
                    _orm_texture: orm_texture,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        // Unit-095: Per-mesh bind groups — each with its own material uniform buffer.
        // Shared buffers (seed, camera, pose) are reused across all bind groups.
        use std::sync::Arc;
        let uniform_arc = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("oathyard shared seed"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&uniform_arc, 0, bytemuck::bytes_of(&[seed]));
        let mut per_mesh_bgs: Vec<wgpu::BindGroup> = Vec::new();
        for resource in &buffers {
            let mb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("per-mesh material buf"),
                contents: bytemuck::bytes_of(&resource.mesh_material),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("per-mesh bg0"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: camera_buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: mb.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 3, resource: pose_buffer.as_entire_binding() },
                ],
            });
            per_mesh_bgs.push(bg);
        }
        Some((mesh_pipeline, buffers, per_mesh_bgs))
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
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        // Unit-095: SDF renders arena background first; meshes draw on top.
        // SDF fighters are hidden via seed.z when meshes are loaded.
        // Both passes share a depth buffer so mesh fragments pass depth test.
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
        if let Some((mesh_pipeline, buffers, per_mesh_bind_groups)) = &mesh_resources {
            pass.set_pipeline(mesh_pipeline);
            for (idx, resource) in buffers.iter().enumerate() {
                pass.set_bind_group(0, &per_mesh_bind_groups[idx], &[]);
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


// Unit-065: Draw a line from (x0,y0) to (x1,y1) using Bresenham-like stepping.
fn draw_line(rgba: &mut [u8], width: u32, height: u32, x0: i32, y0: i32, x1: i32, y1: i32, r: u8, g: u8, b: u8) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1i32 } else { -1i32 };
    let sy = if y0 < y1 { 1i32 } else { -1i32 };
    let mut err = dx + dy;
    let mut x = x0;
    let mut y = y0;
    loop {
        set_pixel(rgba, width, height, x, y, r, g, b);
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x += sx; }
        if e2 <= dx { err += dx; y += sy; }
    }
}

// Unit-065: Draw shape markers for non-color identity cues.
fn draw_marker_shape(rgba: &mut [u8], width: u32, height: u32, cx: i32, cy: i32, size: i32, r: u8, g: u8, b: u8, is_player: bool) {
    if is_player {
        // Triangle for player (pointing up — aggressive, forward-facing)
        draw_line(rgba, width, height, cx, cy - size, cx - size, cy + size/2, r, g, b);
        draw_line(rgba, width, height, cx, cy - size, cx + size, cy + size/2, r, g, b);
        draw_line(rgba, width, height, cx - size, cy + size/2, cx + size, cy + size/2, r, g, b);
    } else {
        // Diamond for opponent (stable, defensive)
        draw_line(rgba, width, height, cx, cy - size, cx + size, cy, r, g, b);
        draw_line(rgba, width, height, cx + size, cy, cx, cy + size, r, g, b);
        draw_line(rgba, width, height, cx, cy + size, cx - size, cy, r, g, b);
        draw_line(rgba, width, height, cx - size, cy, cx, cy - size, r, g, b);
    }
}

// Unit-065: Draw action affordance cue based on action label.
fn draw_action_cue(rgba: &mut [u8], width: u32, height: u32, cx: i32, cy: i32, action: &str, r: u8, g: u8, b: u8) {
    let s = 40i32;
    match action {
        "guard_pose" | "guard" => {
            // Vertical shield line
            draw_line(rgba, width, height, cx, cy - s, cx, cy + s, r, g, b);
            draw_line(rgba, width, height, cx - s/2, cy - s/2, cx + s/2, cy - s/2, r, g, b);
        },
        "cut" => {
            // Arc slash — diagonal line from upper-left to lower-right
            draw_line(rgba, width, height, cx - s, cy - s, cx + s, cy + s, r, g, b);
            draw_line(rgba, width, height, cx - s, cy - s, cx - s + s/3, cy - s, r, g, b);
        },
        "thrust" => {
            // Arrow pointing forward (right)
            draw_line(rgba, width, height, cx - s, cy, cx + s, cy, r, g, b);
            draw_line(rgba, width, height, cx + s, cy, cx + s - s/3, cy - s/3, r, g, b);
            draw_line(rgba, width, height, cx + s, cy, cx + s - s/3, cy + s/3, r, g, b);
        },
        "recover" => {
            // Return arc — curved arrow back
            draw_line(rgba, width, height, cx - s, cy - s/2, cx, cy + s, r, g, b);
            draw_line(rgba, width, height, cx, cy + s, cx + s, cy - s/2, r, g, b);
        },
        _ => {}
    }
}

// Unit-104: Combat impact VFX — dramatic hit/block/whiff visualization.
// All presentation only; no truth mutation.
fn draw_hit_impact(rgba: &mut [u8], width: u32, height: u32, cx: i32, cy: i32, is_hit: bool) {
    if is_hit {
        // Hit impact — bright radial burst with large flash
        let colors = [(255, 220, 60), (255, 160, 30), (255, 100, 20)];
        for (i, &(r, g, b)) in colors.iter().enumerate() {
            let radius = 60 + (i as i32) * 25;
            // Draw 12 radial lines at 30-degree intervals
            for angle_i in 0..12 {
                let angle = angle_i as f64 * std::f64::consts::PI / 6.0;
                let x1 = cx + (angle.cos() * radius as f64) as i32;
                let y1 = cy + (angle.sin() * radius as f64) as i32;
                let inner_radius = radius - 15;
                let x0 = cx + (angle.cos() * inner_radius as f64) as i32;
                let y0 = cy + (angle.sin() * inner_radius as f64) as i32;
                draw_line(rgba, width, height, x0, y0, x1, y1, r, g, b);
            }
        }
        // Central flash — concentric bright rectangles
        fill_rect(rgba, width, height, cx - 20, cy - 20, 40, 40, 255, 255, 255);
        fill_rect(rgba, width, height, cx - 12, cy - 12, 24, 24, 255, 220, 80);
        fill_rect(rgba, width, height, cx - 5, cy - 5, 10, 10, 255, 180, 40);
    } else {
        // Block/whiff — dimmer spark pattern
        for angle_i in 0..8 {
            let angle = angle_i as f64 * std::f64::consts::PI / 4.0;
            let radius = 35.0;
            let x1 = cx + (angle.cos() * radius) as i32;
            let y1 = cy + (angle.sin() * radius) as i32;
            let x0 = cx + (angle.cos() * radius * 0.4) as i32;
            let y0 = cy + (angle.sin() * radius * 0.4) as i32;
            draw_line(rgba, width, height, x0, y0, x1, y1, 180, 200, 220);
        }
        fill_rect(rgba, width, height, cx - 8, cy - 8, 16, 16, 200, 220, 240);
    }
}

// Unit-065: Draw contact marker — line from weapon grip to target.
fn draw_contact_marker(rgba: &mut [u8], width: u32, height: u32, x0: i32, y0: i32, x1: i32, y1: i32) {
    // Bright red-orange impact line
    draw_line(rgba, width, height, x0, y0, x1, y1, 255, 100, 30);
    // Impact flash at target
    fill_rect(rgba, width, height, x1 - 8, y1 - 8, 16, 16, 255, 200, 50);
    fill_rect(rgba, width, height, x1 - 4, y1 - 4, 8, 8, 255, 255, 100);
}

// Unit-094: Map game state and action data to animation clip for skeletal motion
fn action_clip_for_state(
    state: &InteractiveState,
    timeline_slots: &[String],
    combat_contacts: &[ResolvedContact],
) -> &'static str {
    match state {
        InteractiveState::Observe => "idle",
        InteractiveState::Timeline => {
            // Show the action at the current cursor position
            "guard_pose" // default to guard while planning
        }
        InteractiveState::Plan => "guard_pose",
        InteractiveState::CommitReveal => {
            if let Some(slot) = timeline_slots.first() {
                match slot.as_str() {
                    "step" => "step",
                    "pivot" => "pivot",
                    "guard" => "guard_pose",
                    "parry" => "parry",
                    "cut" => "cut",
                    "thrust" => "thrust",
                    "brace" => "brace",
                    "bash" => "bash",
                    "hook_bind" => "hook_bind",
                    "grab" => "grab",
                    "shove" => "shove",
                    "kick" => "kick",
                    "recover" => "recover",
                    _ => "guard_pose",
                }
            } else {
                "guard_pose"
            }
        }
        InteractiveState::Resolve => {
            if let Some(contact) = combat_contacts.first() {
                match contact.player_action.as_str() {
                    "step" => "step",
                    "pivot" => "pivot",
                    "guard" => "guard_pose",
                    "parry" => "parry",
                    "cut" => "cut",
                    "thrust" => "thrust",
                    "brace" => "brace",
                    "bash" => "bash",
                    "hook_bind" => "hook_bind",
                    "grab" => "grab",
                    "shove" => "shove",
                    "kick" => "kick",
                    "recover" => "recover",
                    _ => "attack",
                }
            } else {
                "attack"
            }
        }
        InteractiveState::Consequence => "recover",
        InteractiveState::Replan => "idle",
        InteractiveState::MatchResult => "idle",
        InteractiveState::Replay => {
            if let Some(contact) = combat_contacts.first() {
                match contact.player_action.as_str() {
                    "step" => "step",
                    "pivot" => "pivot",
                    "guard" => "guard_pose",
                    "parry" => "parry",
                    "cut" => "cut",
                    "thrust" => "thrust",
                    "brace" => "brace",
                    "bash" => "bash",
                    "hook_bind" => "hook_bind",
                    "grab" => "grab",
                    "shove" => "shove",
                    "kick" => "kick",
                    "recover" => "recover",
                    _ => "guard_pose",
                }
            } else {
                "idle"
            }
        }
        InteractiveState::FightFilm => {
            if let Some(contact) = combat_contacts.first() {
                match contact.player_action.as_str() {
                    "step" => "step",
                    "pivot" => "pivot",
                    "guard" => "guard_pose",
                    "parry" => "parry",
                    "cut" => "cut",
                    "thrust" => "thrust",
                    "brace" => "brace",
                    "bash" => "bash",
                    "hook_bind" => "hook_bind",
                    "grab" => "grab",
                    "shove" => "shove",
                    "kick" => "kick",
                    "recover" => "recover",
                    _ => "guard_pose",
                }
            } else {
                "guard_pose"
            }
        }
        _ => "idle",
    }
}

// Unit-094: Action category lookup for trace-driven UI
fn action_category(action: &str) -> &'static str {
    match action {
        "step" | "pivot" => "movement",
        "guard" | "parry" | "brace" => "defense",
        "cut" | "thrust" | "bash" | "kick" => "attack",
        "hook_bind" => "bind",
        "grab" | "shove" => "grapple",
        "recover" => "recovery",
        _ => "unknown",
    }
}

fn composite_ui_overlay(rgba: &mut [u8], width: u32, height: u32, capture_id: &str, packet: &Value, candidate_assets: &[String]) {
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
            draw_panel(rgba, width, height, 20, 70, 550, 380);
            draw_title_bar(rgba, width, height, 20, 70, 550, "PLAN (INTENT TIMELINE)");
            // Unit-088: YOMI-style intent card display
            draw_text(rgba, width, height, "INTENT: CUT > HIGH > HEAD", 35, 108, 255, 220, 120);
            draw_text(rgba, width, height, "CATEGORY: ATTACK | RANGE: MEASURE", 35, 128, 200, 200, 200);
            draw_text(rgba, width, height, "GOOD VS: STEP, RECOVER", 35, 148, 100, 255, 100);
            draw_text(rgba, width, height, "LOSES TO: GUARD, PARRY, BRACE", 35, 168, 255, 100, 100);
            draw_text(rgba, width, height, "CUE: \\ // (SLASH/ARC)", 35, 188, 255, 200, 80);
            draw_text(rgba, width, height, &format!("BASE COST: 32 FRAMES"), 35, 218, 200, 200, 200);
            draw_text(rgba, width, height, &format!("CURRENT: 38 FRAMES"), 35, 248, 255, 160, 80);
            draw_text(rgba, width, height, "BODY: -60 PERMILLE", 35, 278, 200, 150, 150);
            draw_text(rgba, width, height, "EQUIP: +0", 35, 308, 150, 200, 150);
            draw_text(rgba, width, height, "STATE: +18 PERMILLE", 35, 338, 200, 180, 120);
            draw_text(rgba, width, height, "> COMMIT PLAN (ENTER)", 35, 368, 100, 255, 100);
            // Action key reference — all 13 actions
            draw_text(rgba, width, height, "1=STEP 2=PIVOT 3=GUARD 4=PARRY", 35, 398, 150, 180, 200);
            draw_text(rgba, width, height, "5=CUT 6=THRUST 7=BRACE 8=BASH", 35, 418, 150, 180, 200);
            draw_text(rgba, width, height, "9=HOOK 0=BIND G=GRAB B=SHOVE", 35, 438, 150, 180, 200);
            draw_text(rgba, width, height, "K=KICK R(RECOV)=F6 RECOVER", 35, 458, 150, 180, 200);
        },
        "pre_contact_frame" | "pre_contact_frame_seed" => {
            // Unit-091: Trace-driven simultaneous reveal UI
            let cs = packet.get("combat_summary").unwrap_or(&Value::Null);
            let p_action = cs.get("player_action").and_then(Value::as_str).unwrap_or("cut");
            let o_action = cs.get("opponent_action").and_then(Value::as_str).unwrap_or("guard");
            let matchup = cs.get("matchup_explanation").and_then(Value::as_str).unwrap_or("Combat exchange");
            let p_cat = action_category(p_action);
            let o_cat = action_category(o_action);
            // Unit-104: Dramatic VS reveal — center burst for action reveal
            let vs_burst_cx = (width as i32) / 2;
            let vs_burst_cy = (height as i32) / 3;
            draw_hit_impact(rgba, width, height, vs_burst_cx, vs_burst_cy, true);
            draw_panel(rgba, width, height, 20, 70, 600, 220);
            draw_title_bar(rgba, width, height, 20, 70, 600, "COMMIT / REVEAL");
            draw_text(rgba, width, height, "=== SIMULTANEOUS REVEAL ===", 35, 108, 255, 255, 100);
            let p_line = format!("PLAYER(GOLD): {}", p_action.to_uppercase());
            draw_text(rgba, width, height, &p_line, 35, 138, 255, 220, 60);
            let p_cat_line = format!("  INTENT: {}", p_cat.to_uppercase());
            draw_text(rgba, width, height, &p_cat_line, 35, 158, 200, 180, 100);
            let o_line = format!("OPPONENT(CRIMSON): {}", o_action.to_uppercase());
            draw_text(rgba, width, height, &o_line, 35, 188, 255, 80, 40);
            let o_cat_line = format!("  INTENT: {}", o_cat.to_uppercase());
            draw_text(rgba, width, height, &o_cat_line, 35, 208, 200, 120, 120);
            draw_text(rgba, width, height, "--- MATCHUP ---", 35, 238, 255, 255, 255);
            let matchup_line = format!("{} vs {}: {}", p_action.to_uppercase(), o_action.to_uppercase(), matchup);
            draw_text(rgba, width, height, &matchup_line, 35, 258, 255, 220, 120);
            draw_text(rgba, width, height, "RESULT: see resolve phase", 35, 278, 255, 160, 80);
        },
        "contact_frame" | "contact_frame_seed" => {
            // Unit-091: Trace-driven contact UI
            let cs = packet.get("combat_summary").unwrap_or(&Value::Null);
            let p_action = cs.get("player_action").and_then(Value::as_str).unwrap_or("cut");
            let o_action = cs.get("opponent_action").and_then(Value::as_str).unwrap_or("guard");
            let material = cs.get("material_result").and_then(Value::as_str).unwrap_or("unknown");
            let weapon = cs.get("weapon").and_then(Value::as_str).unwrap_or("longsword");
            let armor = cs.get("armor").and_then(Value::as_str).unwrap_or("mail_hauberk");
            let target = cs.get("target").and_then(Value::as_str).unwrap_or("torso");
            let is_contact = material.contains("hit") || material.contains("cut") || material.contains("deflect") || material == "unknown";
            // Unit-104: Draw combat impact VFX at center of frame
            draw_hit_impact(rgba, width, height, (width as i32) / 2 - 40, (height as i32) / 2 - 60, is_contact);
            draw_panel(rgba, width, height, 20, 70, 550, 180);
            draw_title_bar(rgba, width, height, 20, 70, 550, "RESOLVE (CONTACT)");
            let action_line = format!("PLAYER {} -> OPPONENT {}", p_action.to_uppercase(), o_action.to_uppercase());
            draw_text(rgba, width, height, &action_line, 35, 108, 255, 220, 120);
            let weapon_line = format!("WEAPON: {} vs ARMOR: {}", weapon.to_uppercase(), armor.to_uppercase());
            draw_text(rgba, width, height, &weapon_line, 35, 138, 200, 200, 200);
            let material_line = format!("MATERIAL: {}", material);
            draw_text(rgba, width, height, &material_line, 35, 158, 255, 220, 120);
            let consequence_line = format!("> TARGET: {}", target.to_uppercase());
            draw_text(rgba, width, height, &consequence_line, 35, 188, 255, 160, 80);
            draw_text(rgba, width, height, "> NEXT: Injury/capability result", 35, 208, 200, 200, 100);
            draw_contact_marker(rgba, width, height, (width as i32)/2 - 60, (height as i32)/4 + 20, (width as i32)/2 + 40, (height as i32)/4 + 10);
            let impact_line = format!("IMPACT -> {}", target.to_uppercase());
            draw_text(rgba, width, height, &impact_line, (width as i32)/2 - 40, (height as i32)/4 + 30, 255, 160, 80);
        },
        "injury_capability_consequence_frame" | "material_armor_damage_frame" => {
            draw_panel(rgba, width, height, 20, 70, 580, 230);
            draw_title_bar(rgba, width, height, 20, 70, 580, "CONSEQUENCE");
            draw_text(rgba, width, height, "MATERIAL: PARTIAL DEFLECT", 35, 108, 255, 220, 120);
            draw_text(rgba, width, height, "ANATOMY: SURFACE IMPACT", 35, 138, 255, 220, 120);
            // Unit-065: Add affected region indicator
            draw_text(rgba, width, height, ">> AFFECTED: TORSO REGION", 35, 148, 255, 160, 80);
            // Unit-065: Consequence marker — bright pulse rectangle
            fill_rect(rgba, width, height, 25, 158, 200, 4, 255, 180, 40);
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
            // Unit-091: Trace-driven fight-film review
            let cs = packet.get("combat_summary").unwrap_or(&Value::Null);
            let p_action = cs.get("player_action").and_then(Value::as_str).unwrap_or("cut");
            let o_action = cs.get("opponent_action").and_then(Value::as_str).unwrap_or("guard");
            let matchup = cs.get("matchup_explanation").and_then(Value::as_str).unwrap_or("Combat exchange");
            let material = cs.get("material_result").and_then(Value::as_str).unwrap_or("unknown");
            draw_panel(rgba, width, height, 20, 70, 600, 200);
            draw_title_bar(rgba, width, height, 20, 70, 600, "FIGHT FILM (INTENT REVIEW)");
            draw_text(rgba, width, height, "KEY MOMENT: Simultaneous reveal", 35, 108, 255, 220, 120);
            let pair_line = format!("PLAYER: {} vs OPP: {}", p_action.to_uppercase(), o_action.to_uppercase());
            draw_text(rgba, width, height, &pair_line, 35, 138, 200, 200, 200);
            let why_line = format!("WHY: {}", matchup);
            draw_text(rgba, width, height, &why_line, 35, 158, 200, 180, 100);
            let result_line = format!("RESULT: {}", material);
            draw_text(rgba, width, height, &result_line, 35, 178, 255, 160, 80);
            draw_text(rgba, width, height, "TRACE-LINKED: YES", 35, 198, 150, 255, 150);
            draw_text(rgba, width, height, "CAMERA: VERDICT RING ORBIT", 35, 228, 255, 220, 120);
            draw_text(rgba, width, height, "REPLAY VERIFIED", 35, 258, 150, 255, 150);
        },
        "replay_verification_ui_or_packet_view" => {
            // Unit-091: Trace-driven replay explanation
            let cs = packet.get("combat_summary").unwrap_or(&Value::Null);
            let p_action = cs.get("player_action").and_then(Value::as_str).unwrap_or("cut");
            let o_action = cs.get("opponent_action").and_then(Value::as_str).unwrap_or("guard");
            let matchup = cs.get("matchup_explanation").and_then(Value::as_str).unwrap_or("Combat exchange");
            draw_panel(rgba, width, height, 20, 70, 550, 180);
            draw_title_bar(rgba, width, height, 20, 70, 550, "REPLAY (INTENT VERIFIED)");
            draw_text(rgba, width, height, "VERIFIED: YES", 35, 108, 150, 255, 150);
            draw_text(rgba, width, height, &format!("HASH {}...", &short_hash[..12]), 35, 138, 200, 200, 200);
            draw_text(rgba, width, height, "SCHEMA: OATHYARD.REPLAY.V1", 35, 168, 200, 200, 200);
            draw_text(rgba, width, height, "INTENT TRACE:", 35, 198, 255, 220, 120);
            let trace_line = format!("T0: {} vs {} -> {}", p_action.to_uppercase(), o_action.to_uppercase(), matchup);
            draw_text(rgba, width, height, &trace_line, 35, 218, 200, 180, 100);
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
            // Unit-084: Extract loadout from candidate_assets so the player can
            // identify which fighter has which weapon and armor.
            let mut p_fighter = "";
            let mut o_fighter = "";
            let mut p_weapon = "";
            let mut o_weapon = "";
            let mut p_armor = "";
            let mut o_armor = "";
            for asset in candidate_assets {
                match infer_mesh_asset_class(asset) {
                    "fighter" => {
                        if p_fighter.is_empty() { p_fighter = asset; }
                        else if o_fighter.is_empty() { o_fighter = asset; }
                    }
                    "weapon" => {
                        if p_weapon.is_empty() { p_weapon = asset; }
                        else if o_weapon.is_empty() { o_weapon = asset; }
                    }
                    "armor" => {
                        if p_armor.is_empty() { p_armor = asset; }
                        else if o_armor.is_empty() { o_armor = asset; }
                    }
                    _ => {}
                }
            }

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
                draw_text(rgba, width, height, "PLAYER(GOLD/^) VS OPPONENT(CRIMSON/<>)", 35, 78, 255, 220, 120);  // Unit-065: ^=triangle player, <>=diamond opponent
            // Unit-065: Draw visible non-color identity shapes near fighter positions.
            // These shapes are independent of color tint and visible regardless of lighting.
            let mid_h = (height as i32) / 2;
            let mid_w = (width as i32) / 2;
            draw_marker_shape(rgba, width, height, mid_w - mid_w/3 + 10, mid_h - 20, 28, 255, 220, 60, true);
            draw_text(rgba, width, height, "^=PLAYER", mid_w - mid_w/3 - 30, mid_h + 20, 255, 220, 60);
            draw_marker_shape(rgba, width, height, mid_w + mid_w/3 + 10, mid_h - 20, 28, 255, 80, 40, false);
            draw_text(rgba, width, height, "<>=OPPONENT", mid_w + mid_w/3 - 30, mid_h + 20, 255, 80, 40);
            }

            // Unit-095: Loadout identification panel — wider to prevent text clipping
            if !p_fighter.is_empty() || !o_fighter.is_empty() {
                let ly_x = (width as i32) - 500;
                let ly_y = 70;
                draw_panel(rgba, width, height, ly_x, ly_y, 480, 160);
                draw_title_bar(rgba, width, height, ly_x, ly_y, 480, "LOADOUT");
                let p_line = format!("P: {}", p_fighter.to_uppercase());
                draw_text(rgba, width, height, &p_line, ly_x + 15, ly_y + 38, 255, 220, 60);
                let pw_line = format!("  W:{} A:{}", p_weapon.to_uppercase(), p_armor.to_uppercase());
                draw_text(rgba, width, height, &pw_line, ly_x + 15, ly_y + 58, 255, 220, 120);
                let o_line = format!("O: {}", o_fighter.to_uppercase());
                draw_text(rgba, width, height, &o_line, ly_x + 15, ly_y + 88, 255, 80, 40);
                let ow_line = format!("  W:{} A:{}", o_weapon.to_uppercase(), o_armor.to_uppercase());
                draw_text(rgba, width, height, &ow_line, ly_x + 15, ly_y + 108, 255, 100, 100);
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

// ═══════════════════════════════════════════════════════════════════════════════
// Unit-072: Native window/swapchain playable path
// ═══════════════════════════════════════════════════════════════════════════════

struct WindowedConfig {
    packet_path: PathBuf,
    out_dir: PathBuf,
    mesh_manifest_path: Option<PathBuf>,
    camera_mode: String,
    candidate_assets: Vec<String>,
    smoke_frames: usize,
    auto_exit: bool,
    width: u32,
    height: u32,
    interactive_mode: bool,
    scripted_input_path: Option<PathBuf>,
}

fn parse_windowed_args() -> Result<WindowedConfig, String> {
    let mut packet_path = None;
    let mut out_dir = PathBuf::from("artifacts/windowed/latest");
    let mut mesh_manifest_path = None;
    let mut camera_mode = "oathyard_verdict_ring_establishing".to_string();
    let mut candidate_assets = vec![
        "saltreach_duelist".to_string(),
        "training_yard".to_string(),
    ];
    let mut smoke_frames = 60usize;
    let mut auto_exit = true;
    let mut width = 1280u32;
    let mut height = 720u32;
    let mut interactive_mode = false;
    let mut scripted_input_path: Option<PathBuf> = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--windowed" => {}
            "--packet" => {
                packet_path = Some(PathBuf::from(args.next().ok_or("--packet value")?))
            }
            "--out" => out_dir = PathBuf::from(args.next().ok_or("--out value")?),
            "--mesh-manifest-json" => {
                mesh_manifest_path =
                    Some(PathBuf::from(args.next().ok_or("--mesh-manifest-json value")?))
            }
            "--camera-mode" => camera_mode = args.next().ok_or("--camera-mode value")?,
            "--candidate-assets" => {
                candidate_assets = args
                    .next()
                    .unwrap_or_default()
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect();
            }
            "--smoke-frames" => {
                smoke_frames = args.next().and_then(|s| s.parse().ok()).unwrap_or(60)
            }
            "--no-auto-exit" => auto_exit = false,
            "--width" => width = args.next().and_then(|s| s.parse().ok()).unwrap_or(1280),
            "--height" => height = args.next().and_then(|s| s.parse().ok()).unwrap_or(720),
            "--interactive" => {
                interactive_mode = true;
                auto_exit = false;
            }
            "--scripted-input" => {
                scripted_input_path =
                    Some(PathBuf::from(args.next().ok_or("--scripted-input value")?));
                interactive_mode = true;
            }
            _ => {}
        }
    }

    let packet_path = packet_path.ok_or("--packet is required for windowed mode")?;
    fs::create_dir_all(&out_dir).ok();

    Ok(WindowedConfig {
        packet_path,
        out_dir,
        mesh_manifest_path,
        camera_mode,
        candidate_assets,
        smoke_frames,
        auto_exit,
        width,
        height,
        interactive_mode,
        scripted_input_path,
    })
}

/// Unit-074: Interactive playable state machine for the native window.
/// Mirrors the GameState enum from local_game.rs but lives in presentation layer.
/// All state transitions are presentation-only — they never mutate gameplay truth.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InteractiveState {
    Boot,
    MainMenu,
    FighterSelect,
    LoadoutSelect,
    ArenaSelect,
    MatchIntro,
    Observe,
    Timeline,
    Plan,
    CommitReveal,
    Resolve,
    Consequence,
    Replan,
    MatchResult,
    Replay,
    FightFilm,
    Settings,
    Quit,
}

impl InteractiveState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Boot => "BOOT",
            Self::MainMenu => "MAIN_MENU",
            Self::FighterSelect => "FIGHTER_SELECT",
            Self::LoadoutSelect => "LOADOUT_SELECT",
            Self::ArenaSelect => "ARENA_SELECT",
            Self::MatchIntro => "MATCH_INTRO",
            Self::Observe => "OBSERVE",
            Self::Timeline => "TIMELINE",
            Self::Plan => "PLAN",
            Self::CommitReveal => "COMMIT_REVEAL",
            Self::Resolve => "RESOLVE_CONTACT",
            Self::Consequence => "CONSEQUENCE",
            Self::Replan => "REPLAN",
            Self::MatchResult => "MATCH_RESULT",
            Self::Replay => "REPLAY",
            Self::FightFilm => "FIGHT_FILM",
            Self::Settings => "SETTINGS",
            Self::Quit => "QUIT",
        }
    }

    fn camera_mode(self) -> &'static str {
        match self {
            Self::Boot => "boot_main_menu",
            Self::MainMenu => "boot_main_menu",
            Self::FighterSelect => "fighter_select",
            Self::LoadoutSelect => "loadout_select",
            Self::ArenaSelect => "arena_select",
            Self::MatchIntro => "oathyard_verdict_ring_establishing",
            Self::Observe => "oathyard_verdict_ring_establishing",
            Self::Timeline => "planning_timeline",
            Self::Plan => "planning_timeline",
            Self::CommitReveal => "pre_contact_frame",
            Self::Resolve => "contact_frame",
            Self::Consequence => "injury_capability_consequence_frame",
            Self::Replan => "recovery_replan_frame",
            Self::MatchResult => "oathyard_arena_candidate_01",
            Self::Replay => "fight_film_replay_camera_shot",
            Self::FightFilm => "fight_film_candidate_shot_01",
            Self::Settings => "settings_accessibility",
            Self::Quit => "boot_main_menu",
        }
    }

    /// Advance to the next logical state on Enter/Space
    fn next(self) -> Self {
        match self {
            Self::Boot => Self::MainMenu,
            Self::MainMenu => Self::FighterSelect,
            Self::FighterSelect => Self::LoadoutSelect,
            Self::LoadoutSelect => Self::ArenaSelect,
            Self::ArenaSelect => Self::MatchIntro,
            Self::MatchIntro => Self::Observe,
            Self::Observe => Self::Timeline,
            Self::Timeline => Self::Plan,
            Self::Plan => Self::CommitReveal,
            Self::CommitReveal => Self::Resolve,
            Self::Resolve => Self::Consequence,
            // Unit-104: YOMI-style combat loop — after each exchange, return to
            // Observe for the next action, not Replan. MatchResult goes back to
            // MainMenu so the player can choose to play again.
            Self::Consequence => Self::Observe,
            Self::Replan => Self::MatchResult,
            Self::MatchResult => Self::MainMenu,
            Self::Replay => Self::FightFilm,
            Self::FightFilm => Self::Quit,
            Self::Settings => Self::MainMenu,
            Self::Quit => Self::Quit,
        }
    }
}

/// Structured event log entry for the interactive windowed session
#[derive(Clone, Debug)]
struct InteractiveEvent {
    event_index: u32,
    frame_index: u32,
    event_source: String, // "manual" | "scripted" | "window" | "system"
    raw_event_type: String,
    logical_input: String,
    previous_state: String,
    next_state: String,
    accepted: bool,
    reason_if_ignored: Option<String>,
}

struct WindowedGpuMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    material_bind_group: wgpu::BindGroup,
    // Unit-100: Per-mesh bind group 0 with this mesh's own material uniform buffer.
    // Fixes the queue.write_buffer-inside-render-pass pitfall where all meshes
    // shared the first mesh's material.
    per_mesh_bind_group0: wgpu::BindGroup,
    mesh_material: MeshMaterial,
    _textures: (wgpu::Texture, wgpu::Texture, wgpu::Texture),
}

// Unit-087: Roster arrays for in-game selection cycling
const ROSTER_FIGHTERS_WINDOWED: &[&str] = &[
    "saltreach_duelist",
    "oathyard_writ",
    "bruiser_oath",
    "chainbreaker",
    "gate_shield",
    "reed_sentinel",
];
const ROSTER_WEAPONS_WINDOWED: &[&str] = &[
    "longsword",
    "arming_sword",
    "ash_spear",
    "bearded_axe",
    "billhook",
    "curved_sword",
    "iron_maul",
    "round_shield",
];
const ROSTER_ARMOR_WINDOWED: &[&str] = &[
    "gambeson",
    "mail_hauberk",
    "bruiser_padded_plate",
    "fencer_light",
    "heavy_plate",
    "lamellar",
];
const ROSTER_ARENAS_WINDOWED: &[&str] = &["oathyard_verdict_ring", "training_yard"];

struct WindowedApp {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    sdf_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    mesh_material_buffer: wgpu::Buffer,
    pose_buffer: wgpu::Buffer,
    gpu_meshes: Vec<WindowedGpuMesh>,
    camera_mode: String,
    first_person_default: bool,
    frames_presented: usize,
    redraw_requested_count: usize,
    resize_event_count: usize,
    surface_reconfigure_count: usize,
    input_event_count: usize,
    close_event_handled: bool,
    smoke_frames: usize,
    auto_exit: bool,
    interactive_mode: bool,
    scripted_input_used: bool,
    interactive_state: InteractiveState,
    states_visited: Vec<String>,
    transitions: Vec<String>,
    timeline_slots: Vec<String>,
    opponent_timeline_slots: Vec<String>,
    timeline_cursor: usize,
    timeline_slot_count: usize,
    combat_contacts: Vec<ResolvedContact>,
    match_result: Option<MatchResult>,
    event_log: Vec<InteractiveEvent>,
    camera_buffer: wgpu::Buffer,
    packet_json: Value,
    out_dir: PathBuf,
    surface_format: wgpu::TextureFormat,
    present_mode: wgpu::PresentMode,
    alpha_mode: wgpu::CompositeAlphaMode,
    adapter_info: wgpu::AdapterInfo,
    mesh_asset_count: usize,
    mesh_assets: Vec<String>,
    mesh_vertex_count: usize,
    mesh_triangle_count: usize,
    saltreach_consumed: bool,
    training_yard_consumed: bool,
    // Unit-097: Screenshot capture on state transitions
    last_captured_state: Option<InteractiveState>,
    capture_screenshots: bool,
    // Unit-087: In-game roster selection state
    roster_fighter_idx: usize,
    roster_weapon_idx: usize,
    roster_armor_idx: usize,
    roster_arena_idx: usize,
    // Unit-090: Opponent intent concealment + trace-driven combat state
    opponent_intent_revealed: bool,
    current_slot_display: usize,
    // Unit-104: MotionBricks procedural animation interpolator
    motion_brick: MotionBrick,
    last_clip: String,
    // Unit-104: Cumulative injury tracking across exchanges
    cumulative_player_injury: u32,
    cumulative_opponent_injury: u32,
    cumulative_exchanges: u32,
}

mod wgpu_mesh {
    pub struct GpuMesh {
        pub vertex_buffer: wgpu::Buffer,
        pub index_buffer: wgpu::Buffer,
        pub index_count: u32,
    }
}

fn windowed_main() -> Result<(), String> {
    let config = parse_windowed_args()?;

    // Load and verify presentation packet
    let packet_text = fs::read_to_string(&config.packet_path)
        .map_err(|e| format!("read packet: {e}"))?;
    let packet_json: Value = serde_json::from_str(&packet_text)
        .map_err(|e| format!("parse packet: {e}"))?;
    if packet_json.get("schema").and_then(Value::as_str)
        != Some("oathyard.post_hash_presentation_packet.v1")
    {
        return Err("invalid packet schema".to_string());
    }

    // Load mesh manifests — same path as offscreen renderer
    let mut mesh_specs = Vec::new();
    if let Some(path) = &config.mesh_manifest_path {
        mesh_specs.extend(load_runtime_mesh_manifest(path)?);
    }
    // Unit-096: Auto-load AAA Meshy assets when manifest exists
    let aaa_manifest = std::env::current_dir()
        .map(|p| p.join("assets/manifests/aaa_mesh_manifest.json"))
        .unwrap_or_else(|_| std::path::PathBuf::from("assets/manifests/aaa_mesh_manifest.json"));
    if aaa_manifest.exists() {
        // Unit-096: Replace old fighters/arena with AAA, keep weapons/armor
        mesh_specs.retain(|s| {
            let cls = infer_mesh_asset_class(&s.mesh_asset_id);
            cls == "weapon" || cls == "armor"
        });
        mesh_specs.extend(load_runtime_mesh_manifest(&aaa_manifest)?);
    }
    let runtime_meshes = mesh_specs
        .iter()
        .map(|s| load_runtime_mesh_with_clip(s, "idle"))
        .collect::<Result<Vec<_>, _>>()?;

    let mesh_asset_count = runtime_meshes.len();
    let mesh_assets: Vec<String> = runtime_meshes
        .iter()
        .map(|m| {
            m.summary_json()
                .get("mesh_asset_id")
                .and_then(Value::as_str)
                .unwrap_or("?")
                .to_string()
        })
        .collect();
    let mesh_summary = runtime_meshes
        .first()
        .map(|m| m.summary_json())
        .unwrap_or(Value::Null);
    let mesh_vertex_count = mesh_summary
        .get("vertex_count")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let mesh_triangle_count = mesh_summary
        .get("triangle_count")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let saltreach_consumed = mesh_assets.iter().any(|m| m.contains("saltreach"));
    let training_yard_consumed = mesh_assets.iter().any(|m| m.contains("training_yard"));

    let mut seed = seed_uniforms(&packet_json, "windowed_smoke", &config.candidate_assets);
    // Unit-095: Hide SDF fighters in windowed mode when meshes are loaded
    if !runtime_meshes.is_empty() {
        seed[2] = 1.0;
    }

    // Unit-074: Load scripted input if provided
    let scripted_inputs = if let Some(ref si_path) = config.scripted_input_path {
        parse_scripted_input(si_path)?
    } else {
        Vec::new()
    };

    // Create winit event loop
    let event_loop = winit::event_loop::EventLoop::new()
        .map_err(|e| format!("create event loop: {e}"))?;

    let window_attrs = winit::window::WindowAttributes::default()
        .with_title(format!(
            "OATHYARD — Native Windowed Duel ({})",
            if config.interactive_mode { "Interactive" } else { "Smoke" }
        ))
        .with_inner_size(winit::dpi::PhysicalSize::new(config.width, config.height));

    let mut handler = WindowedAppHandler {
        app: None,
        window: None,
        config,
        packet_json,
        runtime_meshes,
        seed,
        mesh_asset_count,
        mesh_assets,
        mesh_vertex_count,
        mesh_triangle_count,
        saltreach_consumed,
        training_yard_consumed,
        window_attrs,
        scripted_inputs,
        scripted_input_idx: 0,
    };

    event_loop
        .run_app(&mut handler)
        .map_err(|e| format!("event loop error: {e}"))?;

    Ok(())
}

async fn setup_wgpu_surface(
    window: &winit::window::Window,
    width: u32,
    height: u32,
) -> Result<
    (
        wgpu::Device,
        wgpu::Queue,
        wgpu::Surface<'static>,
        wgpu::SurfaceConfiguration,
        wgpu::AdapterInfo,
        wgpu::TextureFormat,
        wgpu::PresentMode,
        wgpu::CompositeAlphaMode,
    ),
    String,
> {
    let mut instance_desc = wgpu::InstanceDescriptor::new_without_display_handle();
    instance_desc.backends = wgpu::Backends::VULKAN;
    let instance = wgpu::Instance::new(instance_desc);

    // Extract raw window + display handles for a 'static surface lifetime.
    let (rwh, rdh) = {
        use winit::raw_window_handle::{HasWindowHandle, HasDisplayHandle};
        let w_handle = window.window_handle().map_err(|e| format!("window handle: {e}"))?;
        let d_handle = window.display_handle().map_err(|e| format!("display handle: {e}"))?;
        (w_handle.as_raw(), Some(d_handle.as_raw()))
    };
    let surface = unsafe {
        instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: rdh,
            raw_window_handle: rwh,
        })
    }
    .map_err(|e| format!("create surface: {e}"))?;

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .map_err(|e| format!("request adapter: {e}"))?;

    let adapter_info = adapter.get_info();

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("oathyard windowed renderer device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            ..Default::default()
        })
        .await
        .map_err(|e| format!("request device: {e}"))?;

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);
    let present_mode = surface_caps
        .present_modes
        .iter()
        .copied()
        .find(|m| *m == wgpu::PresentMode::Mailbox)
        .unwrap_or(wgpu::PresentMode::Fifo);
    let alpha_mode = surface_caps.alpha_modes[0];

    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
        format: surface_format,
        width: width.max(1),
        height: height.max(1),
        present_mode,
        alpha_mode,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &surface_config);

    Ok((device, queue, surface, surface_config, adapter_info, surface_format, present_mode, alpha_mode))
}

/// Unit-075: Resolve camera mode from interactive state + view toggle.
/// Combat states use first-person by default; menus use state-specific cameras.
fn camera_for_state(state: InteractiveState, first_person: bool) -> &'static str {
    if first_person {
        match state {
            InteractiveState::Observe
            | InteractiveState::Timeline
            | InteractiveState::Plan
            | InteractiveState::CommitReveal
            | InteractiveState::Resolve
            | InteractiveState::Consequence
            | InteractiveState::Replan => "first_person_combat_view",
            _ => state.camera_mode(),
        }
    } else {
        match state {
            InteractiveState::Observe
            | InteractiveState::Timeline
            | InteractiveState::Plan
            | InteractiveState::CommitReveal
            | InteractiveState::Resolve
            | InteractiveState::Consequence
            | InteractiveState::Replan => "third_person_combat_view",
            _ => state.camera_mode(),
        }
    }
}

/// Unit-077: Simple opponent AI timeline policy (alternating guard / cut / thrust).
fn opponent_policy_timeline(slot_count: usize) -> Vec<String> {
    // Unit-090: Expanded opponent policy uses more of the 13-action roster
    let actions = ["guard", "cut", "thrust", "step", "brace", "bash", "pivot", "recover"];
    (0..slot_count)
        .map(|i| actions[i % actions.len()].to_string())
        .collect()
}

/// Unit-078/079: Combat resolution + injury tracking + match outcome.
/// Presents deterministic combat results from timeline comparison.
#[derive(Clone, Debug)]
struct ResolvedContact {
    slot: usize,
    player_action: String,
    opponent_action: String,
    contact_type: String,
    injury: String,
    injury_severity: u32,
    outcome: String,
}

#[derive(Clone, Debug)]
struct MatchResult {
    player_injury_score: u32,
    opponent_injury_score: u32,
    winner: String,
    end_condition: String,
    summary: String,
}

fn injury_severity(injury: &str) -> u32 {
    match injury {
        "none" => 0,
        "minor_stamina" => 1,
        "blade_contact_both" => 3,
        "player_exposed" | "opponent_exposed" => 4,
        "thrust_penetrates" | "cut_lands_first" => 5,
        "mutual_impalement" => 6,
        _ => 0,
    }
}

fn resolve_timeline_combat(
    player_slots: &[String],
    opponent_slots: &[String],
) -> (Vec<ResolvedContact>, MatchResult) {
    let len = player_slots.len().min(opponent_slots.len());
    let mut contacts = Vec::new();
    let mut player_injury_score = 0u32;
    let mut opponent_injury_score = 0u32;

    for i in 0..len {
        let pa = &player_slots[i];
        let oa = &opponent_slots[i];
        let (ct, inj, out) = match (pa.as_str(), oa.as_str()) {
            // Guard matchups
            ("guard", "guard") => ("none", "none", "neutral — both guard"),
            ("guard", "cut") | ("guard", "thrust") => ("blocked", "minor_stamina", "player blocked opponent's strike"),
            ("guard", "bash") => ("guard_broken", "player_exposed", "opponent bash broke player's guard"),
            ("guard", "grab") => ("guard_bypassed", "player_exposed", "opponent grab bypassed player's guard"),
            ("guard", "shove") => ("guard_displaced", "player_exposed", "opponent shove displaced player's guard"),
            ("guard", "kick") => ("guard_kicked", "player_exposed", "opponent kick punished static guard"),
            ("guard", "hook_bind") => ("guard_trapped", "minor_stamina", "opponent hook_bind trapped player's weapon"),
            ("guard", "parry") => ("neutral", "none", "both defend — stalemate"),
            ("guard", "step") | ("guard", "pivot") => ("positioning", "none", "opponent repositioned — no contact"),
            ("guard", "brace") => ("neutral", "none", "both defend statically"),
            ("guard", "recover") => ("neutral", "none", "opponent recovering — no pressure"),

            ("cut", "guard") | ("thrust", "guard") => ("strike_blocked", "none", "opponent blocked player's strike"),
            ("cut", "parry") | ("thrust", "parry") => ("strike_deflected", "player_exposed", "opponent parried — player exposed"),
            ("cut", "cut") => ("simultaneous", "blade_contact_both", "simultaneous cut — both exposed"),
            ("cut", "thrust") => ("thrust_vs_cut", "thrust_penetrates", "opponent thrust slips past player's cut"),
            ("thrust", "cut") => ("cut_vs_thrust", "cut_lands_first", "player cut lands before opponent thrust extends"),
            ("thrust", "thrust") => ("double_thrust", "mutual_impalement", "both thrust — mutual contact"),
            ("cut", "brace") | ("thrust", "brace") => ("brace_absorbs", "minor_stamina", "opponent brace absorbed the strike"),
            ("cut", "step") | ("thrust", "step") => ("positioning", "none", "opponent stepped away — no contact"),
            ("cut", "pivot") | ("thrust", "pivot") => ("miss", "none", "opponent pivoted off the line"),
            ("cut", "recover") | ("thrust", "recover") => ("clean_hit", "opponent_exposed", "player strike hit recovering opponent"),
            ("cut", "bash") => ("simultaneous", "blade_contact_both", "cut and bash clash"),
            ("thrust", "bash") => ("thrust_vs_bash", "opponent_exposed", "thrust reaches past bash"),
            ("cut", "hook_bind") | ("thrust", "hook_bind") => ("weapon_trapped", "player_exposed", "opponent hook_bind trapped player's weapon"),
            ("cut", "grab") | ("thrust", "grab") => ("grab_stops_attack", "player_exposed", "opponent grab stopped player's attack"),
            ("cut", "shove") | ("thrust", "shove") => ("shove_disrupts", "player_exposed", "opponent shove disrupted player's attack"),
            ("cut", "kick") | ("thrust", "kick") => ("kick_interrupts", "player_exposed", "opponent kick interrupted player's attack"),

            // Defense vs attack (reversed)
            ("parry", "cut") | ("parry", "thrust") => ("deflected", "opponent_exposed", "player parried — opponent exposed"),
            ("parry", "guard") => ("neutral", "none", "both defend — stalemate"),
            ("parry", "bash") => ("parry_breaks", "player_exposed", "bash overwhelms parry"),

            // Brace matchups
            ("brace", "brace") => ("neutral", "none", "both brace — stalemate"),
            ("brace", "bash") | ("brace", "shove") | ("brace", "kick") => ("brace_holds", "minor_stamina", "player brace absorbed opponent's force"),
            ("brace", "cut") | ("brace", "thrust") => ("brace_overwhelmed", "player_exposed", "opponent strike overwhelmed player's brace"),
            ("brace", "grab") => ("brace_grappled", "player_exposed", "opponent grab controlled bracing player"),
            ("brace", "step") | ("brace", "pivot") => ("positioning", "none", "opponent repositioned"),
            ("brace", "recover") => ("neutral", "none", "opponent recovering"),
            ("brace", "guard") => ("neutral", "none", "both defend"),
            ("brace", "hook_bind") => ("brace_trapped", "minor_stamina", "opponent hook_bind controlled bracing player"),

            // Bash matchups
            ("bash", "guard") => ("guard_broken", "opponent_exposed", "player bash broke opponent's guard"),
            ("bash", "brace") => ("brace_holds", "minor_stamina", "opponent brace absorbed bash"),
            ("bash", "recover") => ("clean_hit", "opponent_exposed", "player bash punished recovering opponent"),
            ("bash", "step") | ("bash", "pivot") => ("miss", "none", "opponent evaded bash"),
            ("bash", "cut") | ("bash", "thrust") => ("simultaneous", "blade_contact_both", "bash and strike clash"),
            ("bash", "bash") => ("simultaneous", "blade_contact_both", "mutual bash — both staggered"),
            ("bash", "grab") => ("bash_vs_grab", "blade_contact_both", "bash and grab clash"),
            ("bash", "shove") => ("bash_vs_shove", "blade_contact_both", "bash and shove trade"),
            ("bash", "kick") => ("bash_vs_kick", "blade_contact_both", "bash and kick trade"),
            ("bash", "hook_bind") => ("bash_vs_bind", "minor_stamina", "bash crashes through bind"),
            ("bash", "parry") => ("bash_overwhelms_parry", "opponent_exposed", "bash overwhelms parry"),

            // Hook_bind matchups
            ("hook_bind", "guard") => ("guard_trapped", "opponent_exposed", "player hook_bind trapped opponent's weapon"),
            ("hook_bind", "cut") | ("hook_bind", "thrust") => ("weapon_controlled", "opponent_exposed", "player hook_bind controlled opponent's weapon"),
            ("hook_bind", "brace") => ("bind_vs_brace", "minor_stamina", "hook_bind vs brace — neutral"),
            ("hook_bind", "step") | ("hook_bind", "pivot") => ("positioning", "none", "opponent disengaged from bind"),
            ("hook_bind", "recover") => ("clean_hit", "opponent_exposed", "hook_bind catches recovering opponent"),
            ("hook_bind", "bash") => ("bind_crashed", "player_exposed", "opponent bash crashed through bind"),
            ("hook_bind", "grab") => ("bind_vs_grab", "blade_contact_both", "hook_bind and grab contest"),
            ("hook_bind", "shove") => ("bind_shoved", "player_exposed", "opponent shove broke the bind"),
            ("hook_bind", "kick") => ("bind_kicked", "player_exposed", "opponent kick punished bind"),
            ("hook_bind", "hook_bind") => ("mutual_bind", "minor_stamina", "mutual bind — neutral"),

            // Grab matchups
            ("grab", "guard") => ("guard_bypassed", "opponent_exposed", "player grab bypassed opponent's guard"),
            ("grab", "brace") => ("brace_grappled", "opponent_exposed", "player grab controlled bracing opponent"),
            ("grab", "recover") => ("clean_hit", "opponent_exposed", "player grab punished recovering opponent"),
            ("grab", "step") | ("grab", "pivot") => ("miss", "none", "opponent evaded grab"),
            ("grab", "cut") | ("grab", "thrust") => ("grab_stopped", "player_exposed", "opponent strike stopped grab"),
            ("grab", "bash") => ("grab_vs_bash", "blade_contact_both", "grab and bash contest"),
            ("grab", "shove") => ("grab_vs_shove", "blade_contact_both", "grab and shove contest"),
            ("grab", "kick") => ("grab_vs_kick", "player_exposed", "opponent kick punished grab attempt"),
            ("grab", "grab") => ("mutual_grab", "blade_contact_both", "mutual grab — clinch"),
            ("grab", "hook_bind") => ("grab_vs_bind", "blade_contact_both", "grab and bind contest"),

            // Shove matchups
            ("shove", "guard") => ("guard_displaced", "opponent_exposed", "player shove displaced opponent's guard"),
            ("shove", "brace") => ("brace_holds", "minor_stamina", "opponent brace absorbed shove"),
            ("shove", "recover") => ("clean_hit", "opponent_exposed", "player shove punished recovering opponent"),
            ("shove", "step") | ("shove", "pivot") => ("miss", "none", "opponent evaded shove"),
            ("shove", "cut") | ("shove", "thrust") => ("shove_stopped", "player_exposed", "opponent strike stopped shove"),
            ("shove", "bash") => ("shove_vs_bash", "blade_contact_both", "shove and bash trade"),
            ("shove", "grab") => ("shove_vs_grab", "blade_contact_both", "shove and grab contest"),
            ("shove", "kick") => ("shove_vs_kick", "blade_contact_both", "shove and kick trade"),
            ("shove", "hook_bind") => ("shove_breaks_bind", "opponent_exposed", "shove breaks opponent's bind"),
            ("shove", "shove") => ("mutual_shove", "blade_contact_both", "mutual shove — both staggered"),

            // Kick matchups
            ("kick", "guard") => ("guard_kicked", "opponent_exposed", "player kick punished opponent's guard"),
            ("kick", "brace") => ("brace_holds", "minor_stamina", "opponent brace absorbed kick"),
            ("kick", "recover") => ("clean_hit", "opponent_exposed", "player kick punished recovering opponent"),
            ("kick", "step") | ("kick", "pivot") => ("miss", "none", "opponent evaded kick"),
            ("kick", "cut") | ("kick", "thrust") => ("kick_stopped", "player_exposed", "opponent strike stopped kick"),
            ("kick", "bash") => ("kick_vs_bash", "blade_contact_both", "kick and bash trade"),
            ("kick", "grab") => ("kick_vs_grab", "opponent_exposed", "kick punishes grab attempt"),
            ("kick", "shove") => ("kick_vs_shove", "blade_contact_both", "kick and shove trade"),
            ("kick", "hook_bind") => ("kick_vs_bind", "opponent_exposed", "kick punishes bind"),
            ("kick", "kick") => ("mutual_kick", "blade_contact_both", "mutual kick — both staggered"),

            // Movement
            ("step", _) | (_, "step") => ("positioning", "none", "footwork — no contact"),
            ("pivot", _) | (_, "pivot") => ("pivot", "none", "angle change — no contact"),

            // Recovery is vulnerable
            ("recover", "cut") | ("recover", "thrust") | ("recover", "bash") | ("recover", "grab") | ("recover", "kick") => ("recovery_punished", "player_exposed", "player recovering — opponent struck"),
            ("recover", _) => ("recovery_safe", "none", "player recovered safely"),
            (_, "recover") => ("opponent_recovery", "opponent_exposed", "opponent recovering — exposed"),

            // Fallback
            _ => ("unknown", "none", "unresolved"),
        };
        let sev = injury_severity(inj);
        // Accumulate: injuries with "player_" prefix affect player, "opponent_" affect opponent
        if inj.contains("player_") || inj == "player_exposed" {
            player_injury_score += sev;
        } else if inj.contains("opponent_") || inj == "opponent_exposed" {
            opponent_injury_score += sev;
        } else if sev > 0 && inj != "none" {
            // Shared injuries (simultaneous, mutual) affect both
            player_injury_score += sev;
            opponent_injury_score += sev;
        }
        contacts.push(ResolvedContact {
            slot: i,
            player_action: pa.clone(),
            opponent_action: oa.clone(),
            contact_type: ct.to_string(),
            injury: inj.to_string(),
            injury_severity: sev,
            outcome: out.to_string(),
        });
    }

    let (winner, end_condition) = if player_injury_score < opponent_injury_score {
        ("player", "player has fewer total injuries")
    } else if opponent_injury_score < player_injury_score {
        ("opponent", "opponent has fewer total injuries")
    } else {
        ("draw", "equal injury score — draw")
    };

    let result = MatchResult {
        player_injury_score,
        opponent_injury_score,
        winner: winner.to_string(),
        end_condition: end_condition.to_string(),
        summary: format!(
            "Player {} ({}) vs Opponent {} ({}) — {}",
            player_injury_score, "injury score",
            opponent_injury_score, "injury score",
            end_condition
        ),
    };

    (contacts, result)
}

/// Unit-074: Scripted input for deterministic interactive windowed smoke
#[derive(Clone, Debug)]
struct ScriptedInput {
    action: String,
    at_frame: u32,
    label: String,
}

fn parse_scripted_input(path: &Path) -> Result<Vec<ScriptedInput>, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("read scripted input: {e}"))?;
    let json: Value = serde_json::from_str(&text).map_err(|e| format!("parse scripted input: {e}"))?;
    let schema = json.get("schema").and_then(Value::as_str).unwrap_or("");
    if schema != "oathyard.windowed_scripted_input.v1" {
        return Err(format!("invalid scripted input schema: {schema}"));
    }
    let inputs = json.get("inputs").and_then(Value::as_array)
        .ok_or("scripted input missing 'inputs' array")?;
    let mut result = Vec::new();
    for item in inputs {
        let action = item.get("action").and_then(Value::as_str).unwrap_or("advance").to_string();
        let at_frame = item.get("at_frame").and_then(Value::as_u64).unwrap_or(0) as u32;
        let label = item.get("label").and_then(Value::as_str).unwrap_or("").to_string();
        result.push(ScriptedInput { action, at_frame, label });
    }
    result.sort_by_key(|i| i.at_frame);
    Ok(result)
}

struct WindowedAppHandler {
    app: Option<WindowedApp>,
    window: Option<winit::window::Window>,
    config: WindowedConfig,
    packet_json: Value,
    runtime_meshes: Vec<RuntimeMesh>,
    seed: [f32; 4],
    mesh_asset_count: usize,
    mesh_assets: Vec<String>,
    mesh_vertex_count: usize,
    mesh_triangle_count: usize,
    saltreach_consumed: bool,
    training_yard_consumed: bool,
    window_attrs: winit::window::WindowAttributes,
    scripted_inputs: Vec<ScriptedInput>,
    scripted_input_idx: usize,
}

impl winit::application::ApplicationHandler for WindowedAppHandler {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_some() {
            return; // Already initialized
        }

        // Create window via ActiveEventLoop
        let window = event_loop
            .create_window(self.window_attrs.clone())
            .expect("create window");

        let window_size = window.inner_size();

        // Setup wgpu surface
        let (device, queue, surface, surface_config, adapter_info, surface_format, present_mode, alpha_mode) =
            pollster::block_on(setup_wgpu_surface(&window, window_size.width, window_size.height))
                .expect("wgpu surface setup");

        // Build full production mesh pipeline — same shader and bindings as offscreen.
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("oathyard windowed shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });

        // Uniform buffers: packet (seed), camera, mesh_material, pose
        let camera_mode_data = camera_for_mode(&self.config.camera_mode);
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
            label: Some("oathyard windowed camera uniform"),
            size: camera_bytes.len() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&camera_buffer, 0, camera_bytes);

        let uniform_bytes: Vec<u8> = self.seed.iter().flat_map(|v| v.to_le_bytes()).collect();
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("oathyard windowed seed uniform"),
            size: uniform_bytes.len() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&uniform_buffer, 0, &uniform_bytes);

        let mesh_material = MeshMaterial {
            material_type: -1.0,
            _pad: [0.0, 0.0, 0.0],
            tint_r: 0.62, tint_g: 0.58, tint_b: 0.54, tint_a: 1.0,
        };
        let mesh_material_bytes = bytemuck::bytes_of(&mesh_material);
        let mesh_material_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("oathyard windowed mesh material uniform"),
            size: mesh_material_bytes.len() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&mesh_material_buffer, 0, mesh_material_bytes);

        let pose = PoseUniform {
            pose_active: 0.0, pose_time: 0.0, _pad: [0.0, 0.0],
            bone_offset_x: [0.0; 4], bone_offset_x2: [0.0; 4],
            bone_offset_y: [0.0; 4], bone_offset_y2: [0.0; 4],
            bone_offset_z: [0.0; 4], bone_offset_z2: [0.0; 4],
            bone_yaw: [0.0; 4], bone_yaw2: [0.0; 4],
        };
        let pose_bytes = bytemuck::bytes_of(&pose);
        let pose_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("oathyard windowed pose uniform"),
            size: pose_bytes.len() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(&pose_buffer, 0, pose_bytes);

        // Group 0 bind group layout: 4 uniform buffers
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("oathyard windowed bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0, visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false, min_binding_size: None,
                    }, count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1, visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false, min_binding_size: None,
                    }, count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false, min_binding_size: None,
                    }, count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3, visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false, min_binding_size: None,
                    }, count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("oathyard windowed bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: camera_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: mesh_material_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: pose_buffer.as_entire_binding() },
            ],
        });

        // Group 1: material textures + sampler
        let material_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("oathyard windowed material texture bind group layout"),
            entries: &[
                texture_layout_entry(0),
                texture_layout_entry(1),
                texture_layout_entry(2),
                wgpu::BindGroupLayoutEntry {
                    binding: 3, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let material_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("oathyard windowed material sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        // Mesh pipeline using the production verdict_ring.wgsl mesh entry points
        let mesh_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("oathyard windowed mesh pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout), Some(&material_bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("oathyard windowed mesh pipeline"),
            layout: Some(&mesh_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("mesh_vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[MeshVertex::layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("mesh_fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    // Unit-100: REPLACE blend for mesh pipeline — mesh fragments
                    // completely overwrite SDF background without alpha blending.
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // Upload mesh geometry + create per-mesh material bind groups
        use wgpu::util::DeviceExt;
        let gpu_meshes: Vec<WindowedGpuMesh> = self.runtime_meshes.iter().map(|mesh| {
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("oathyard windowed mesh vertices"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("oathyard windowed mesh indices"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });
            // Material textures: use dummy 1x1 for meshes without material_validation
            let mat_info = material_for_mesh(&mesh.mesh_asset_id);
            let is_fighter = mat_info.material_type > 3.5 && mat_info.material_type < 4.5;
            let (bt, nt, ot) = if mesh.material.material_texture_binding {
                let mut bi = load_png_rgba(&mesh.material.base_color_texture_path).unwrap_or(RuntimeTextureImage { width: 1, height: 1, rgba: vec![255,255,255,255] });
                // Unit-102: Mild team color tint — 30% team, 70% original texture.
                // Preserves PBR material detail. Team identity comes from the
                // shader fresnel rim band + UI markers, not full-body color.
                if is_fighter && bi.width > 1 {
                    let tr = (mat_info.tint_r * 255.0) as u8;
                    let tg = (mat_info.tint_g * 255.0) as u8;
                    let tb = (mat_info.tint_b * 255.0) as u8;
                    for chunk in bi.rgba.chunks_exact_mut(4) {
                        chunk[0] = ((chunk[0] as f32 * 0.70) + (tr as f32 * 0.30)).min(255.0) as u8;
                        chunk[1] = ((chunk[1] as f32 * 0.70) + (tg as f32 * 0.30)).min(255.0) as u8;
                        chunk[2] = ((chunk[2] as f32 * 0.70) + (tb as f32 * 0.30)).min(255.0) as u8;
                    }
                }
                let ni = load_png_rgba(&mesh.material.normal_texture_path).unwrap_or(RuntimeTextureImage { width: 1, height: 1, rgba: vec![128,128,255,255] });
                let oi = load_png_rgba(&mesh.material.orm_texture_path).unwrap_or(RuntimeTextureImage { width: 1, height: 1, rgba: vec![255,255,255,255] });
                (
                    create_material_texture(&device, &queue, "windowed base", wgpu::TextureFormat::Rgba8UnormSrgb, &bi),
                    create_material_texture(&device, &queue, "windowed normal", wgpu::TextureFormat::Rgba8Unorm, &ni),
                    create_material_texture(&device, &queue, "windowed orm", wgpu::TextureFormat::Rgba8Unorm, &oi),
                )
            } else {
                (
                    create_material_texture(&device, &queue, "windowed dummy", wgpu::TextureFormat::Rgba8UnormSrgb,
                        &RuntimeTextureImage { width: 1, height: 1, rgba: vec![255,255,255,255] }),
                    create_material_texture(&device, &queue, "windowed dummy norm", wgpu::TextureFormat::Rgba8Unorm,
                        &RuntimeTextureImage { width: 1, height: 1, rgba: vec![128,128,255,255] }),
                    create_material_texture(&device, &queue, "windowed dummy orm", wgpu::TextureFormat::Rgba8Unorm,
                        &RuntimeTextureImage { width: 1, height: 1, rgba: vec![255,255,255,255] }),
                )
            };
            let bv = bt.create_view(&wgpu::TextureViewDescriptor::default());
            let nv = nt.create_view(&wgpu::TextureViewDescriptor::default());
            let ov = ot.create_view(&wgpu::TextureViewDescriptor::default());
            let mat_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("oathyard windowed material bind group"),
                layout: &material_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&bv) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&nv) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&ov) },
                    wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&material_sampler) },
                ],
            });
            // Unit-100: Create per-mesh material uniform buffer + bind group 0.
            // This fixes the queue.write_buffer-inside-render-pass pitfall where
            // all meshes shared the first mesh's material uniform.
            let mat = material_for_mesh(&mesh.mesh_asset_id);
            let mat_bytes = bytemuck::bytes_of(&mat);
            let mat_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("oathyard windowed per-mesh material"),
                size: mat_bytes.len() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            queue.write_buffer(&mat_buffer, 0, mat_bytes);
            let per_mesh_bg0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("oathyard windowed per-mesh bind group 0"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: camera_buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: mat_buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 3, resource: pose_buffer.as_entire_binding() },
                ],
            });
            WindowedGpuMesh {
                vertex_buffer, index_buffer,
                index_count: mesh.indices.len() as u32,
                material_bind_group: mat_bg,
                per_mesh_bind_group0: per_mesh_bg0,
                mesh_material: mat,
                _textures: (bt, nt, ot),
            }
        }).collect();

        // Unit-099: SDF pipeline for arena/environment background.
        // The SDF pass (vs_main/fs_main) renders the floor, ring, witness stones,
        // and boundary posts as a fullscreen triangle behind the meshes.
        let sdf_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("oathyard windowed SDF pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let sdf_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("oathyard windowed SDF pipeline"),
            layout: Some(&sdf_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            // Unit-100: SDF is a background layer — write color but NOT depth.
            // depth_compare=Always means SDF always passes; depth_write=false
            // means the depth buffer stays at 1.0 (cleared) so mesh fragments
            // with depth < 1.0 will correctly pass the Less test.
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        self.app = Some(WindowedApp {
            device,
            queue,
            surface,
            surface_config,
            pipeline,
            sdf_pipeline,
            bind_group,
            mesh_material_buffer,
            pose_buffer,
            gpu_meshes,
            camera_mode: self.config.camera_mode.clone(),
            first_person_default: true,
            frames_presented: 0,
            redraw_requested_count: 0,
            resize_event_count: 0,
            surface_reconfigure_count: 0,
            input_event_count: 0,
            close_event_handled: false,
            smoke_frames: self.config.smoke_frames,
            auto_exit: self.config.auto_exit,
            interactive_mode: self.config.interactive_mode,
            scripted_input_used: !self.scripted_inputs.is_empty(),
            interactive_state: InteractiveState::Boot,
            states_visited: vec![InteractiveState::Boot.as_str().to_string()],
            transitions: Vec::new(),
            timeline_slots: vec!["guard".to_string(); 10],
            // Unit-104: YOMI loop — opponent generates ONE action per exchange,
            // not a pre-filled 10-slot timeline. The resolve function only
            // processes up to min(len(player), len(opponent)) exchanges.
            opponent_timeline_slots: vec![opponent_policy_timeline(1)[0].clone()],
            timeline_cursor: 0,
            timeline_slot_count: 10,
            combat_contacts: Vec::new(),
            match_result: None,
            event_log: Vec::new(),
            camera_buffer,
            packet_json: self.packet_json.clone(),
            out_dir: self.config.out_dir.clone(),
            surface_format,
            present_mode,
            alpha_mode,
            adapter_info,
            mesh_asset_count: self.mesh_asset_count,
            mesh_assets: self.mesh_assets.clone(),
            mesh_vertex_count: self.mesh_vertex_count,
            mesh_triangle_count: self.mesh_triangle_count,
            saltreach_consumed: self.saltreach_consumed,
            training_yard_consumed: self.training_yard_consumed,
            last_captured_state: None,
            capture_screenshots: true,
            roster_fighter_idx: 0,
            roster_weapon_idx: 0,
            roster_armor_idx: 0,
            roster_arena_idx: 0,
            opponent_intent_revealed: false,
            current_slot_display: 0,
            motion_brick: MotionBrick::new(),
            last_clip: String::new(),
            cumulative_player_injury: 0,
            cumulative_opponent_injury: 0,
            cumulative_exchanges: 0,
        });

        self.window = Some(window);
        if let Some(ref w) = self.window {
            w.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let app = self.app.as_mut().expect("app state");

        match event {
            winit::event::WindowEvent::CloseRequested => {
                app.close_event_handled = true;
                app.event_log.push(InteractiveEvent {
                    event_index: app.event_log.len() as u32,
                    frame_index: app.frames_presented as u32,
                    event_source: "window".to_string(),
                    raw_event_type: "CloseRequested".to_string(),
                    logical_input: "quit".to_string(),
                    previous_state: app.interactive_state.as_str().to_string(),
                    next_state: "QUIT".to_string(),
                    accepted: true,
                    reason_if_ignored: None,
                });
                write_window_manifest(app);
                event_loop.exit();
            }
            winit::event::WindowEvent::KeyboardInput { event, .. } => {
                app.input_event_count += 1;
                if let winit::event::KeyEvent {
                    state: winit::event::ElementState::Pressed,
                    physical_key: winit::keyboard::PhysicalKey::Code(code),
                    ..
                } = event
                {
                    let prev_state = app.interactive_state;
                    let mut logical = String::new();
                    let mut accepted = true;
                    let mut reason: Option<String> = None;

                    match code {
                        winit::keyboard::KeyCode::Escape => {
                            logical = "quit".to_string();
                            app.close_event_handled = true;
                            app.interactive_state = InteractiveState::Quit;
                            write_window_manifest(app);
                            event_loop.exit();
                        }
                        winit::keyboard::KeyCode::Enter | winit::keyboard::KeyCode::Space => {
                            logical = "advance".to_string();
                            if app.interactive_state != InteractiveState::Quit {
                                let next = app.interactive_state.next();
                                // Unit-078: Resolve combat when transitioning out of TIMELINE
                                if app.interactive_state == InteractiveState::Timeline {
                                    let (contacts, result) = resolve_timeline_combat(
                                        &app.timeline_slots,
                                        &app.opponent_timeline_slots,
                                    );
                                    app.combat_contacts = contacts;
                                    // Unit-104: Accumulate injury across exchanges
                                    app.cumulative_player_injury = app.cumulative_player_injury.saturating_add(result.player_injury_score);
                                    app.cumulative_opponent_injury = app.cumulative_opponent_injury.saturating_add(result.opponent_injury_score);
                                    app.cumulative_exchanges += 1;
                                    app.match_result = Some(result);
                                }
                                // Unit-090: Reveal opponent intent when entering CommitReveal
                                if app.interactive_state == InteractiveState::Timeline {
                                    app.opponent_intent_revealed = true;
                                }
                                // Unit-104: On loop-back to Observe (after Consequence),
                                // reset opponent intent and consume the committed action
                                // so the player can plan the next exchange. If cumulative
                                // injury exceeds threshold, go to MatchResult instead.
                                if app.interactive_state == InteractiveState::Consequence {
                                    app.opponent_intent_revealed = false;
                                    // Check if match is over: one fighter reached injury threshold
                                    const INJURY_THRESHOLD: u32 = 10;
                                    let match_over = app.cumulative_player_injury >= INJURY_THRESHOLD
                                        || app.cumulative_opponent_injury >= INJURY_THRESHOLD;
                                    if match_over {
                                        app.interactive_state = InteractiveState::MatchResult;
                                        let next_str = "MATCH_RESULT".to_string();
                                        if !app.states_visited.contains(&next_str) {
                                            app.states_visited.push(next_str.clone());
                                        }
                                        app.transitions.push(format!(
                                            "{} -> {}",
                                            prev_state.as_str(),
                                            "MATCH_RESULT"
                                        ));
                                    } else {
                                        // Generate a fresh opponent action for the next exchange
                                        app.opponent_timeline_slots = vec![opponent_policy_timeline(1)[0].clone()];
                                        // Clear the first slot (committed action)
                                        if !app.timeline_slots.is_empty() {
                                            app.timeline_slots.remove(0);
                                        }
                                        app.timeline_cursor = 0;
                                        app.interactive_state = next;
                                        let next_str = next.as_str().to_string();
                                        if !app.states_visited.contains(&next_str) {
                                            app.states_visited.push(next_str.clone());
                                        }
                                        app.transitions.push(format!(
                                            "{} -> {}",
                                            prev_state.as_str(),
                                            next.as_str()
                                        ));
                                    }
                                } else {
                                    app.interactive_state = next;
                                    let next_str = next.as_str().to_string();
                                    if !app.states_visited.contains(&next_str) {
                                        app.states_visited.push(next_str.clone());
                                    }
                                    app.transitions.push(format!(
                                        "{} -> {}",
                                        prev_state.as_str(),
                                        next.as_str()
                                    ));
                                }
                            }
                        }
                        winit::keyboard::KeyCode::KeyR => {
                            logical = "replay".to_string();
                            app.interactive_state = InteractiveState::Replay;
                            let s = InteractiveState::Replay.as_str().to_string();
                            if !app.states_visited.contains(&s) {
                                app.states_visited.push(s);
                            }
                            app.transitions.push(format!(
                                "{} -> REPLAY",
                                prev_state.as_str()
                            ));
                        }
                        winit::keyboard::KeyCode::KeyF => {
                            logical = "fight_film".to_string();
                            app.interactive_state = InteractiveState::FightFilm;
                            let s = InteractiveState::FightFilm.as_str().to_string();
                            if !app.states_visited.contains(&s) {
                                app.states_visited.push(s);
                            }
                            app.transitions.push(format!(
                                "{} -> FIGHT_FILM",
                                prev_state.as_str()
                            ));
                        }
                        winit::keyboard::KeyCode::KeyP => {
                            logical = "pause_resume".to_string();
                            // Toggle auto_exit for pause/resume in interactive mode
                            app.auto_exit = !app.auto_exit;
                        }
                        winit::keyboard::KeyCode::KeyH | winit::keyboard::KeyCode::F1 => {
                            logical = "help".to_string();
                            app.interactive_state = InteractiveState::Settings;
                            let s = InteractiveState::Settings.as_str().to_string();
                            if !app.states_visited.contains(&s) {
                                app.states_visited.push(s);
                            }
                            app.transitions.push(format!(
                                "{} -> SETTINGS",
                                prev_state.as_str()
                            ));
                        }
                        winit::keyboard::KeyCode::ArrowLeft | winit::keyboard::KeyCode::KeyA => {
                            logical = "prev".to_string();
                            // Timeline mode: move cursor left
                            if app.interactive_state == InteractiveState::Timeline
                                && app.timeline_cursor > 0
                            {
                                app.timeline_cursor -= 1;
                                logical = format!("timeline_cursor_{}", app.timeline_cursor);
                            }
                            // Unit-087: ArenaSelect cycles arenas
                            if app.interactive_state == InteractiveState::ArenaSelect {
                                if app.roster_arena_idx > 0 {
                                    app.roster_arena_idx -= 1;
                                } else {
                                    app.roster_arena_idx = ROSTER_ARENAS_WINDOWED.len() - 1;
                                }
                                logical = format!(
                                    "arena_prev -> {}",
                                    ROSTER_ARENAS_WINDOWED[app.roster_arena_idx]
                                );
                            }
                            // Unit-087: LoadoutSelect Left cycles armor
                            if app.interactive_state == InteractiveState::LoadoutSelect {
                                if app.roster_armor_idx > 0 {
                                    app.roster_armor_idx -= 1;
                                } else {
                                    app.roster_armor_idx = ROSTER_ARMOR_WINDOWED.len() - 1;
                                }
                                logical = format!(
                                    "armor_prev -> {}",
                                    ROSTER_ARMOR_WINDOWED[app.roster_armor_idx]
                                );
                            }
                        }
                        winit::keyboard::KeyCode::ArrowRight | winit::keyboard::KeyCode::KeyD => {
                            logical = "next".to_string();
                            // Timeline mode: move cursor right
                            if app.interactive_state == InteractiveState::Timeline
                                && app.timeline_cursor + 1 < app.timeline_slot_count
                            {
                                app.timeline_cursor += 1;
                                logical = format!("timeline_cursor_{}", app.timeline_cursor);
                            }
                            // Unit-087: ArenaSelect cycles arenas
                            if app.interactive_state == InteractiveState::ArenaSelect {
                                app.roster_arena_idx += 1;
                                if app.roster_arena_idx >= ROSTER_ARENAS_WINDOWED.len() {
                                    app.roster_arena_idx = 0;
                                }
                                logical = format!(
                                    "arena_next -> {}",
                                    ROSTER_ARENAS_WINDOWED[app.roster_arena_idx]
                                );
                            }
                            // Unit-087: LoadoutSelect Right cycles armor
                            if app.interactive_state == InteractiveState::LoadoutSelect {
                                app.roster_armor_idx += 1;
                                if app.roster_armor_idx >= ROSTER_ARMOR_WINDOWED.len() {
                                    app.roster_armor_idx = 0;
                                }
                                logical = format!(
                                    "armor_next -> {}",
                                    ROSTER_ARMOR_WINDOWED[app.roster_armor_idx]
                                );
                            }
                        }
                        winit::keyboard::KeyCode::Digit1 | winit::keyboard::KeyCode::Numpad1 => {
                            logical = "action_step".to_string();
                            if app.interactive_state == InteractiveState::Timeline {
                                app.timeline_slots[app.timeline_cursor] = "step".to_string();
                            }
                        }
                        winit::keyboard::KeyCode::Digit2 | winit::keyboard::KeyCode::Numpad2 => {
                            logical = "action_pivot".to_string();
                            if app.interactive_state == InteractiveState::Timeline {
                                app.timeline_slots[app.timeline_cursor] = "pivot".to_string();
                            }
                        }
                        winit::keyboard::KeyCode::Digit3 | winit::keyboard::KeyCode::Numpad3 => {
                            logical = "action_guard".to_string();
                            if app.interactive_state == InteractiveState::Timeline {
                                app.timeline_slots[app.timeline_cursor] = "guard".to_string();
                            }
                        }
                        winit::keyboard::KeyCode::Digit4 | winit::keyboard::KeyCode::Numpad4 => {
                            logical = "action_cut".to_string();
                            if app.interactive_state == InteractiveState::Timeline {
                                app.timeline_slots[app.timeline_cursor] = "cut".to_string();
                            }
                        }
                        winit::keyboard::KeyCode::Digit5 | winit::keyboard::KeyCode::Numpad5 => {
                            logical = "action_thrust".to_string();
                            if app.interactive_state == InteractiveState::Timeline {
                                app.timeline_slots[app.timeline_cursor] = "thrust".to_string();
                            }
                        }
                        winit::keyboard::KeyCode::Digit6 | winit::keyboard::KeyCode::Numpad6 => {
                            logical = "action_recover".to_string();
                            if app.interactive_state == InteractiveState::Timeline {
                                app.timeline_slots[app.timeline_cursor] = "recover".to_string();
                            }
                        }
                        // Unit-089: Full 13-action input coverage
                        winit::keyboard::KeyCode::Digit7 | winit::keyboard::KeyCode::Numpad7 => {
                            logical = "action_parry".to_string();
                            if app.interactive_state == InteractiveState::Timeline {
                                app.timeline_slots[app.timeline_cursor] = "parry".to_string();
                            }
                        }
                        winit::keyboard::KeyCode::Digit8 | winit::keyboard::KeyCode::Numpad8 => {
                            logical = "action_brace".to_string();
                            if app.interactive_state == InteractiveState::Timeline {
                                app.timeline_slots[app.timeline_cursor] = "brace".to_string();
                            }
                        }
                        winit::keyboard::KeyCode::Digit9 | winit::keyboard::KeyCode::Numpad9 => {
                            logical = "action_bash".to_string();
                            if app.interactive_state == InteractiveState::Timeline {
                                app.timeline_slots[app.timeline_cursor] = "bash".to_string();
                            }
                        }
                        winit::keyboard::KeyCode::Digit0 | winit::keyboard::KeyCode::Numpad0 => {
                            logical = "action_hook_bind".to_string();
                            if app.interactive_state == InteractiveState::Timeline {
                                app.timeline_slots[app.timeline_cursor] = "hook_bind".to_string();
                            }
                        }
                        winit::keyboard::KeyCode::KeyG => {
                            logical = "action_grab".to_string();
                            if app.interactive_state == InteractiveState::Timeline {
                                app.timeline_slots[app.timeline_cursor] = "grab".to_string();
                            }
                        }
                        winit::keyboard::KeyCode::KeyB => {
                            logical = "action_shove".to_string();
                            if app.interactive_state == InteractiveState::Timeline {
                                app.timeline_slots[app.timeline_cursor] = "shove".to_string();
                            }
                        }
                        winit::keyboard::KeyCode::KeyK => {
                            logical = "action_kick".to_string();
                            if app.interactive_state == InteractiveState::Timeline {
                                app.timeline_slots[app.timeline_cursor] = "kick".to_string();
                            }
                        }
                        winit::keyboard::KeyCode::ArrowUp | winit::keyboard::KeyCode::KeyW => {
                            logical = "up".to_string();
                            // Unit-087: Roster cycling — Up cycles to previous option
                            match app.interactive_state {
                                InteractiveState::FighterSelect => {
                                    if app.roster_fighter_idx > 0 {
                                        app.roster_fighter_idx -= 1;
                                    } else {
                                        app.roster_fighter_idx = ROSTER_FIGHTERS_WINDOWED.len() - 1;
                                    }
                                    logical = format!(
                                        "fighter_prev -> {}",
                                        ROSTER_FIGHTERS_WINDOWED[app.roster_fighter_idx]
                                    );
                                }
                                InteractiveState::LoadoutSelect => {
                                    if app.roster_weapon_idx > 0 {
                                        app.roster_weapon_idx -= 1;
                                    } else {
                                        app.roster_weapon_idx = ROSTER_WEAPONS_WINDOWED.len() - 1;
                                    }
                                    logical = format!(
                                        "weapon_prev -> {}",
                                        ROSTER_WEAPONS_WINDOWED[app.roster_weapon_idx]
                                    );
                                }
                                _ => {}
                            }
                        }
                        winit::keyboard::KeyCode::ArrowDown | winit::keyboard::KeyCode::KeyS => {
                            logical = "down".to_string();
                            // Unit-087: Roster cycling — Down cycles to next option
                            match app.interactive_state {
                                InteractiveState::FighterSelect => {
                                    app.roster_fighter_idx += 1;
                                    if app.roster_fighter_idx >= ROSTER_FIGHTERS_WINDOWED.len() {
                                        app.roster_fighter_idx = 0;
                                    }
                                    logical = format!(
                                        "fighter_next -> {}",
                                        ROSTER_FIGHTERS_WINDOWED[app.roster_fighter_idx]
                                    );
                                }
                                InteractiveState::LoadoutSelect => {
                                    app.roster_weapon_idx += 1;
                                    if app.roster_weapon_idx >= ROSTER_WEAPONS_WINDOWED.len() {
                                        app.roster_weapon_idx = 0;
                                    }
                                    logical = format!(
                                        "weapon_next -> {}",
                                        ROSTER_WEAPONS_WINDOWED[app.roster_weapon_idx]
                                    );
                                }
                                _ => {}
                            }
                        }
                        winit::keyboard::KeyCode::KeyQ => {
                            logical = "quit".to_string();
                            app.close_event_handled = true;
                            app.interactive_state = InteractiveState::Quit;
                            write_window_manifest(app);
                            event_loop.exit();
                        }
                        winit::keyboard::KeyCode::KeyV => {
                            logical = "toggle_camera".to_string();
                            app.first_person_default = !app.first_person_default;
                            app.transitions.push(format!(
                                "{} CAMERA_TOGGLE {}",
                                prev_state.as_str(),
                                if app.first_person_default { "first_person" } else { "third_person" }
                            ));
                        }
                        _ => {
                            accepted = false;
                            reason = Some(format!("unmapped key: {:?}", code));
                            logical = format!("unmapped_{:?}", code);
                        }
                    }

                    // Update camera mode from interactive state
                    let new_cam = camera_for_state(app.interactive_state, app.first_person_default).to_string();
                    app.camera_mode = new_cam.clone();

                    // Update the camera uniform buffer on the GPU
                    let cam_data = camera_for_mode(&new_cam);
                    let cam_uniform = CameraUniform {
                        eye: [cam_data.eye[0], cam_data.eye[1], cam_data.eye[2], cam_data.fov_radians],
                        look_at: [cam_data.look_at[0], cam_data.look_at[1], cam_data.look_at[2], 0.0],
                    };
                    app.queue.write_buffer(&app.camera_buffer, 0, bytemuck::bytes_of(&cam_uniform));

                    // Log the event (except for quit which already wrote manifest)
                    if logical != "quit" {
                        app.event_log.push(InteractiveEvent {
                            event_index: app.event_log.len() as u32,
                            frame_index: app.frames_presented as u32,
                            // Unit-097: Distinguish manual keyboard input from scripted input
                            event_source: if app.interactive_state != prev_state && self.scripted_inputs.len() > self.scripted_input_idx {
                                // Check if this input came from the scripted queue
                                let scheduled = &self.scripted_inputs[self.scripted_input_idx];
                                if scheduled.at_frame <= app.frames_presented as u32 {
                                    "scripted".to_string()
                                } else {
                                    "manual".to_string()
                                }
                            } else {
                                "manual".to_string()
                            },
                            raw_event_type: format!("{:?}", code),
                            logical_input: logical.clone(),
                            previous_state: prev_state.as_str().to_string(),
                            next_state: app.interactive_state.as_str().to_string(),
                            accepted,
                            reason_if_ignored: reason,
                        });
                    }
                }
            }
            winit::event::WindowEvent::Resized(physical_size) => {
                app.resize_event_count += 1;
                app.surface_reconfigure_count += 1;
                let new_width = physical_size.width.max(1);
                let new_height = physical_size.height.max(1);
                app.surface_config.width = new_width;
                app.surface_config.height = new_height;
                let _ = app.surface.configure(&app.device, &app.surface_config);
            }
            winit::event::WindowEvent::RedrawRequested => {
                app.redraw_requested_count += 1;

                let surf_w = app.surface_config.width;
                let surf_h = app.surface_config.height;

                // Unit-092: Render to an offscreen texture, composite CPU UI,
                // then copy to surface for windowed UI overlay support.
                let offscreen_texture = app.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("oathyard windowed offscreen"),
                    size: wgpu::Extent3d { width: surf_w, height: surf_h, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: app.surface_format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                });
                let offscreen_view = offscreen_texture.create_view(&wgpu::TextureViewDescriptor::default());

                // Present a frame
                let surface_texture = app.surface.get_current_texture();
                match surface_texture {
                    wgpu::CurrentSurfaceTexture::Success(surface_tex)
                    | wgpu::CurrentSurfaceTexture::Suboptimal(surface_tex) => {
                        let mut encoder = app.device.create_command_encoder(
                            &wgpu::CommandEncoderDescriptor {
                                label: Some("oathyard windowed render encoder"),
                            },
                        );

                        let depth_texture = app.device.create_texture(&wgpu::TextureDescriptor {
                            label: Some("oathyard windowed depth"),
                            size: wgpu::Extent3d { width: surf_w, height: surf_h, depth_or_array_layers: 1 },
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format: wgpu::TextureFormat::Depth32Float,
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                            view_formats: &[],
                        });
                        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

                        {
                            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("oathyard windowed render pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &offscreen_view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color {
                                            r: 0.02,
                                            g: 0.015,
                                            b: 0.03,
                                            a: 1.0,
                                        }),
                                        store: wgpu::StoreOp::Store,
                                    },
                                    depth_slice: None,
                                })],
                                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                                    view: &depth_view,
                                    depth_ops: Some(wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(1.0),
                                        store: wgpu::StoreOp::Store,
                                    }),
                                    stencil_ops: None,
                                }),
                                occlusion_query_set: None,
                                timestamp_writes: None,
                                multiview_mask: None,
                            });

                            // Unit-099: SDF arena background — fullscreen triangle raymarch
                            render_pass.set_pipeline(&app.sdf_pipeline);
                            render_pass.set_bind_group(0, &app.bind_group, &[]);
                            render_pass.draw(0..3, 0..1);

                            render_pass.set_pipeline(&app.pipeline);
                            render_pass.set_bind_group(0, &app.bind_group, &[]);
                            for mesh in &app.gpu_meshes {
                                // Unit-100: Use per-mesh bind group 0 instead of
                                // queue.write_buffer inside render pass (which
                                // doesn't update per-draw).
                                render_pass.set_bind_group(0, &mesh.per_mesh_bind_group0, &[]);
                                render_pass.set_bind_group(1, &mesh.material_bind_group, &[]);
                                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                                render_pass.set_index_buffer(
                                    mesh.index_buffer.slice(..),
                                    wgpu::IndexFormat::Uint32,
                                );
                                render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
                            }
                        }

                        // Unit-094: Update pose uniform based on current game state/action
                        let action_clip = action_clip_for_state(
                            &app.interactive_state,
                            &app.timeline_slots,
                            &app.combat_contacts,
                        );
                        // Unit-104: MotionBricks — smooth animation interpolation
                        if app.last_clip != action_clip {
                            let target_pose = pose_for_clip(action_clip);
                            let duration = match action_clip {
                                "idle" | "walk" => 15,
                                "guard_pose" => 10,
                                "step" | "pivot" => 8,
                                "cut" | "thrust" | "kick" => 12,
                                "brace" | "bash" => 10,
                                "parry" => 8,
                                "recover" => 15,
                                _ => 12,
                            };
                            app.motion_brick.start_transition(target_pose, action_clip, duration);
                            app.last_clip = action_clip.to_string();
                        }
                        app.motion_brick.advance(1);
                        let interpolated_pose = app.motion_brick.current_pose();
                        app.queue
                            .write_buffer(&app.pose_buffer, 0, bytemuck::bytes_of(&interpolated_pose));

                        // Unit-092: Copy offscreen to buffer for CPU UI compositing
                        let bytes_per_pixel = 4; // RGBA8 or BGRA8
                        let buffer_size = (surf_w as usize * surf_h as usize * bytes_per_pixel);
                        let unpadded_bytes_per_row = surf_w as usize * bytes_per_pixel;
                        // wgpu requires buffer copy alignment of 256 bytes
                        let aligned_bytes_per_row = ((unpadded_bytes_per_row + 255) / 256) * 256;
                        let padded_buffer_size = aligned_bytes_per_row * surf_h as usize;
                        let padded_output_buffer = app.device.create_buffer(&wgpu::BufferDescriptor {
                            label: Some("oathyard windowed readback padded"),
                            size: padded_buffer_size as u64,
                            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                            mapped_at_creation: false,
                        });

                        encoder.copy_texture_to_buffer(
                            wgpu::TexelCopyTextureInfo {
                                texture: &offscreen_texture,
                                mip_level: 0,
                                origin: wgpu::Origin3d::ZERO,
                                aspect: wgpu::TextureAspect::All,
                            },
                            wgpu::TexelCopyBufferInfo {
                                buffer: &padded_output_buffer,
                                layout: wgpu::TexelCopyBufferLayout {
                                    offset: 0,
                                    bytes_per_row: Some(aligned_bytes_per_row as u32),
                                    rows_per_image: Some(surf_h),
                                },
                            },
                            wgpu::Extent3d { width: surf_w, height: surf_h, depth_or_array_layers: 1 },
                        );

                        app.queue.submit(std::iter::once(encoder.finish()));

                        // Read back, composite UI, write to surface
                        let slice = padded_output_buffer.slice(..);
                        let (tx, rx) = std::sync::mpsc::channel();
                        slice.map_async(wgpu::MapMode::Read, move |result| {
                            let _ = tx.send(result);
                        });
                        let _ = app.device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None });
                        let _ = rx.recv();

                        {
                            let data = slice.get_mapped_range();
                            let mut rgba_buf = vec![0u8; buffer_size];
                            // Unpad rows
                            for y in 0..surf_h as usize {
                                let src = &data[y * aligned_bytes_per_row..][..unpadded_bytes_per_row];
                                let dst = &mut rgba_buf[y * unpadded_bytes_per_row..][..unpadded_bytes_per_row];
                                dst.copy_from_slice(src);
                            }
                            drop(data);
                            padded_output_buffer.unmap();

                            // Unit-092: Composite windowed UI overlay
                            composite_windowed_ui(&mut rgba_buf, surf_w, surf_h, app);

                            // Unit-097: Capture screenshot when entering a new state
                            if app.capture_screenshots {
                                let should_capture = match app.last_captured_state {
                                    None => true,
                                    Some(last) => last != app.interactive_state,
                                };
                                if should_capture {
                                    app.last_captured_state = Some(app.interactive_state);
                                    let state_name = app.interactive_state.as_str().to_lowercase();
                                    let cap_dir = app.out_dir.join("captures");
                                    let _ = std::fs::create_dir_all(&cap_dir);
                                    let cap_path = cap_dir.join(format!(
                                        "{}_f{:04}.png",
                                        state_name,
                                        app.frames_presented
                                    ));
                                    let _ = write_png_rgba(&cap_path, surf_w, surf_h, &rgba_buf);
                                }
                            }

                            // Copy composited result to surface texture via staging texture
                            // Repad the buffer for texture upload
                            let mut padded_buf = vec![0u8; padded_buffer_size];
                            for y in 0..surf_h as usize {
                                let src = &rgba_buf[y * unpadded_bytes_per_row..][..unpadded_bytes_per_row];
                                let dst = &mut padded_buf[y * aligned_bytes_per_row..][..unpadded_bytes_per_row];
                                dst.copy_from_slice(src);
                            }

                            let staging_texture = wgpu::util::DeviceExt::create_texture_with_data(
                                &app.device,
                                &app.queue,
                                &wgpu::TextureDescriptor {
                                    label: Some("oathyard windowed composited"),
                                    size: wgpu::Extent3d { width: surf_w, height: surf_h, depth_or_array_layers: 1 },
                                    mip_level_count: 1,
                                    sample_count: 1,
                                    dimension: wgpu::TextureDimension::D2,
                                    format: app.surface_format,
                                    usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::TEXTURE_BINDING,
                                    view_formats: &[],
                                },
                                wgpu_types::TextureDataOrder::default(),
                                &padded_buf,
                            );

                            let mut surface_encoder = app.device.create_command_encoder(
                                &wgpu::CommandEncoderDescriptor {
                                    label: Some("oathyard windowed surface copy"),
                                },
                            );

                            surface_encoder.copy_texture_to_texture(
                                wgpu::TexelCopyTextureInfo {
                                    texture: &staging_texture,
                                    mip_level: 0,
                                    origin: wgpu::Origin3d::ZERO,
                                    aspect: wgpu::TextureAspect::All,
                                },
                                wgpu::TexelCopyTextureInfo {
                                    texture: &surface_tex.texture,
                                    mip_level: 0,
                                    origin: wgpu::Origin3d::ZERO,
                                    aspect: wgpu::TextureAspect::All,
                                },
                                wgpu::Extent3d { width: surf_w, height: surf_h, depth_or_array_layers: 1 },
                            );

                            app.queue.submit(std::iter::once(surface_encoder.finish()));
                        }

                        surface_tex.present();
                        app.frames_presented += 1;

                        // Unit-090: Update the current slot display from timeline cursor
                        app.current_slot_display = app.timeline_cursor;
                    }
                    status => {
                        eprintln!("surface status: {:?}", status);
                    }
                }

                // Inject scripted input if available
                let scripted_inputs = &self.scripted_inputs;
                let si_idx = &mut self.scripted_input_idx;
                while *si_idx < scripted_inputs.len()
                    && scripted_inputs[*si_idx].at_frame <= app.frames_presented as u32
                {
                    let input = &scripted_inputs[*si_idx];
                    let prev_state = app.interactive_state;
                    let prev_str = prev_state.as_str().to_string();
                    let logical_input = input.action.clone();

                    match input.action.as_str() {
                        "advance" => {
                            // Unit-078: Resolve combat when advancing out of TIMELINE
                            if app.interactive_state == InteractiveState::Timeline {
                                let (contacts, result) = resolve_timeline_combat(
                                    &app.timeline_slots,
                                    &app.opponent_timeline_slots,
                                );
                                app.combat_contacts = contacts;
                                app.match_result = Some(result);
                                app.opponent_intent_revealed = true;
                            }
                            let next = app.interactive_state.next();
                            app.interactive_state = next;
                            let next_str = next.as_str().to_string();
                            if !app.states_visited.contains(&next_str) {
                                app.states_visited.push(next_str.clone());
                            }
                            app.transitions.push(format!("{} -> {}", prev_str, next.as_str()));
                            // Unit-097: Log scripted advance to event_log
                            app.event_log.push(InteractiveEvent {
                                event_index: app.event_log.len() as u32,
                                frame_index: app.frames_presented as u32,
                                event_source: "scripted".to_string(),
                                raw_event_type: format!("scripted:advance@{}", input.at_frame),
                                logical_input: input.label.clone(),
                                previous_state: prev_str.clone(),
                                next_state: next_str.clone(),
                                accepted: true,
                                reason_if_ignored: None,
                            });
                        }
                        "replay" => {
                            app.interactive_state = InteractiveState::Replay;
                            let s = "REPLAY".to_string();
                            if !app.states_visited.contains(&s) {
                                app.states_visited.push(s);
                            }
                            app.transitions.push(format!("{} -> REPLAY", prev_str));
                        }
                        "fight_film" => {
                            app.interactive_state = InteractiveState::FightFilm;
                            let s = "FIGHT_FILM".to_string();
                            if !app.states_visited.contains(&s) {
                                app.states_visited.push(s);
                            }
                            app.transitions.push(format!("{} -> FIGHT_FILM", prev_str));
                        }
                        "quit" => {
                            app.close_event_handled = true;
                            app.interactive_state = InteractiveState::Quit;
                            app.event_log.push(InteractiveEvent {
                                event_index: app.event_log.len() as u32,
                                frame_index: app.frames_presented as u32,
                                event_source: "scripted".to_string(),
                                raw_event_type: format!("scripted:{}", input.action),
                                logical_input: logical_input.clone(),
                                previous_state: prev_str,
                                next_state: "QUIT".to_string(),
                                accepted: true,
                                reason_if_ignored: None,
                            });
                            write_window_manifest(app);
                            event_loop.exit();
                            return;
                        }
                        "toggle_camera" => {
                            app.first_person_default = !app.first_person_default;
                            app.transitions.push(format!(
                                "{} CAMERA_TOGGLE {}",
                                prev_str,
                                if app.first_person_default { "first_person" } else { "third_person" }
                            ));
                        }
                        "place_guard" => {
                            if !app.timeline_slots.is_empty() {
                                app.timeline_slots[app.timeline_cursor] = "guard".to_string();
                            }
                        }
                        "place_cut" => {
                            if !app.timeline_slots.is_empty() {
                                app.timeline_slots[app.timeline_cursor] = "cut".to_string();
                            }
                        }
                        "place_thrust" => {
                            if !app.timeline_slots.is_empty() {
                                app.timeline_slots[app.timeline_cursor] = "thrust".to_string();
                            }
                        }
                        "place_step" => {
                            if !app.timeline_slots.is_empty() {
                                app.timeline_slots[app.timeline_cursor] = "step".to_string();
                            }
                        }
                        "place_recover" => {
                            if !app.timeline_slots.is_empty() {
                                app.timeline_slots[app.timeline_cursor] = "recover".to_string();
                            }
                        }
                        // Unit-089: Scripted input for all 13 actions
                        "place_parry" => {
                            if !app.timeline_slots.is_empty() {
                                app.timeline_slots[app.timeline_cursor] = "parry".to_string();
                            }
                        }
                        "place_brace" => {
                            if !app.timeline_slots.is_empty() {
                                app.timeline_slots[app.timeline_cursor] = "brace".to_string();
                            }
                        }
                        "place_bash" => {
                            if !app.timeline_slots.is_empty() {
                                app.timeline_slots[app.timeline_cursor] = "bash".to_string();
                            }
                        }
                        "place_hook_bind" => {
                            if !app.timeline_slots.is_empty() {
                                app.timeline_slots[app.timeline_cursor] = "hook_bind".to_string();
                            }
                        }
                        "place_grab" => {
                            if !app.timeline_slots.is_empty() {
                                app.timeline_slots[app.timeline_cursor] = "grab".to_string();
                            }
                        }
                        "place_shove" => {
                            if !app.timeline_slots.is_empty() {
                                app.timeline_slots[app.timeline_cursor] = "shove".to_string();
                            }
                        }
                        "place_kick" => {
                            if !app.timeline_slots.is_empty() {
                                app.timeline_slots[app.timeline_cursor] = "kick".to_string();
                            }
                        }
                        "place_pivot" => {
                            if !app.timeline_slots.is_empty() {
                                app.timeline_slots[app.timeline_cursor] = "pivot".to_string();
                            }
                        }
                        "timeline_right" => {
                            if app.timeline_cursor + 1 < app.timeline_slot_count {
                                app.timeline_cursor += 1;
                            }
                        }
                        _ => {}
                    }

                    // Update camera from state
                    let new_cam = camera_for_state(app.interactive_state, app.first_person_default).to_string();
                    app.camera_mode = new_cam.clone();
                    let cam_data = camera_for_mode(&new_cam);
                    let cam_uniform = CameraUniform {
                        eye: [cam_data.eye[0], cam_data.eye[1], cam_data.eye[2], cam_data.fov_radians],
                        look_at: [cam_data.look_at[0], cam_data.look_at[1], cam_data.look_at[2], 0.0],
                    };
                    app.queue.write_buffer(&app.camera_buffer, 0, bytemuck::bytes_of(&cam_uniform));

                    app.input_event_count += 1;
                    app.event_log.push(InteractiveEvent {
                        event_index: app.event_log.len() as u32,
                        frame_index: app.frames_presented as u32,
                        event_source: "scripted".to_string(),
                        raw_event_type: format!("scripted:{}", input.action),
                        logical_input: logical_input.clone(),
                        previous_state: prev_str,
                        next_state: app.interactive_state.as_str().to_string(),
                        accepted: true,
                        reason_if_ignored: None,
                    });

                    *si_idx += 1;
                }

                // Auto-exit after smoke frames
                if app.auto_exit && app.frames_presented >= app.smoke_frames {
                    write_window_manifest(app);
                    event_loop.exit();
                    return;
                }

                // Request next redraw for continuous rendering
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

// Unit-092: Composite CPU UI overlay into the windowed render buffer.
// Draws game state, timeline, action labels, and combat results
// using the existing bitmap font system.
fn composite_windowed_ui(rgba: &mut [u8], width: u32, height: u32, app: &WindowedApp) {
    // Unit-100: Skip redundant state label for boot/main menu to prevent ghosting.
    let is_boot_menu = matches!(app.interactive_state, InteractiveState::Boot | InteractiveState::MainMenu);
    if !is_boot_menu {
        // Top-left: state label panel
        let state_label = app.interactive_state.as_str();
        draw_panel(rgba, width, height, 20, 20, 600, 35);
        draw_text(rgba, width, height, state_label, 35, 28, 255, 220, 120);
    }

    // State-specific UI
    match app.interactive_state {
        InteractiveState::Boot | InteractiveState::MainMenu => {
            // Unit-098: OATHYARD branding on boot/main menu
            draw_panel(rgba, width, height, 20, 20, 600, 120);
            draw_text(rgba, width, height, "OATHYARD", 35, 35, 255, 220, 60);
            draw_text(rgba, width, height, "VERDICT-RING COMBAT", 35, 65, 200, 180, 100);
            draw_text(rgba, width, height, "> LOCAL DUEL (ENTER)", 35, 95, 255, 255, 100);
            draw_text(rgba, width, height, "  QUIT (ESC/Q)", 35, 115, 200, 200, 200);
        }
        InteractiveState::FighterSelect => {
            let fighter = ROSTER_FIGHTERS_WINDOWED
                .get(app.roster_fighter_idx)
                .copied()
                .unwrap_or("unknown");
            draw_text(rgba, width, height, "FIGHTER SELECT", 35, 78, 255, 220, 120);
            let line = format!("> {} (UP/DOWN to cycle)", fighter.to_uppercase());
            draw_text(rgba, width, height, &line, 35, 108, 255, 255, 100);
            draw_text(rgba, width, height, "ENTER to confirm", 35, 138, 200, 200, 200);
        }
        InteractiveState::LoadoutSelect => {
            let weapon = ROSTER_WEAPONS_WINDOWED
                .get(app.roster_weapon_idx)
                .copied()
                .unwrap_or("unknown");
            let armor = ROSTER_ARMOR_WINDOWED
                .get(app.roster_armor_idx)
                .copied()
                .unwrap_or("unknown");
            draw_text(rgba, width, height, "LOADOUT SELECT", 35, 78, 255, 220, 120);
            let w_line = format!("WEAPON: {} (UP/DOWN)", weapon.to_uppercase());
            draw_text(rgba, width, height, &w_line, 35, 108, 255, 220, 120);
            let a_line = format!("ARMOR: {} (LEFT/RIGHT)", armor.to_uppercase());
            draw_text(rgba, width, height, &a_line, 35, 138, 255, 220, 120);
            draw_text(rgba, width, height, "ENTER to confirm", 35, 168, 200, 200, 200);
        }
        InteractiveState::ArenaSelect => {
            let arena = ROSTER_ARENAS_WINDOWED
                .get(app.roster_arena_idx)
                .copied()
                .unwrap_or("unknown");
            draw_text(rgba, width, height, "ARENA SELECT", 35, 78, 255, 220, 120);
            let a_line = format!("> {} (LEFT/RIGHT)", arena.to_uppercase());
            draw_text(rgba, width, height, &a_line, 35, 108, 255, 255, 100);
            draw_text(rgba, width, height, "ENTER to start duel", 35, 138, 200, 200, 200);
        }
        InteractiveState::MatchIntro => {
            draw_text(rgba, width, height, "MATCH INTRO", 35, 78, 255, 220, 120);
            draw_text(rgba, width, height, "ENTER to begin combat", 35, 108, 200, 200, 200);
        }
        InteractiveState::Observe => {
            draw_text(rgba, width, height, "OBSERVE", 35, 78, 255, 220, 120);
            draw_text(rgba, width, height, "ENTER to plan your actions", 35, 108, 200, 200, 200);
            // Unit-098: Non-color identity markers — overhead glyphs
            let mid_w = (width as i32) / 2;
            let mid_h = (height as i32) / 2;
            // Player: triangle marker (left side)
            draw_marker_shape(rgba, width, height, mid_w - 200, mid_h - 60, 36, 255, 220, 60, true);
            draw_text(rgba, width, height, "^ PLAYER", mid_w - 235, mid_h - 20, 255, 220, 60);
            // Opponent: diamond marker (right side)
            draw_marker_shape(rgba, width, height, mid_w + 200, mid_h - 60, 36, 255, 80, 40, false);
            draw_text(rgba, width, height, "<> OPPONENT", mid_w + 165, mid_h - 20, 255, 80, 40);
        }
        InteractiveState::Timeline => {
            draw_text(rgba, width, height, "TIMELINE (DECISION PHASE)", 35, 78, 255, 220, 120);
            // Show current slot and action
            let slot_line = format!(
                "SLOT {}/{}: {}",
                app.timeline_cursor + 1,
                app.timeline_slot_count,
                app.timeline_slots
                    .get(app.timeline_cursor)
                    .map(|s| s.as_str())
                    .unwrap_or("empty")
                    .to_uppercase()
            );
            draw_text(rgba, width, height, &slot_line, 35, 108, 255, 255, 100);
            // Unit-098: Show all 10 timeline slots
            for i in 0..10.min(app.timeline_slots.len()) {
                let marker = if i == app.timeline_cursor { ">" } else { " " };
                let s = &app.timeline_slots[i];
                let line = format!("{}[{}] {}", marker, i, s.to_uppercase());
                draw_text(rgba, width, height, &line, 35, 138 + i as i32 * 16, 200, 200, 200);
            }
            // Unit-101: Action legend in two-column panel at bottom for readability.
            draw_panel(rgba, width, height, 20, height as i32 - 80, 700, 60);
            // Left column: keys 1-5
            draw_text(rgba, width, height, "1=STEP  2=PIVOT  3=GUARD", 30, height as i32 - 72, 150, 180, 200);
            draw_text(rgba, width, height, "4=PARRY 5=CUT   6=THRUST", 30, height as i32 - 54, 150, 180, 200);
            // Right column: keys 7-0 + letters
            draw_text(rgba, width, height, "7=BRACE 8=BASH  9=HOOK",  370, height as i32 - 72, 150, 180, 200);
            draw_text(rgba, width, height, "0=BIND  G=GRAB  K=KICK",  370, height as i32 - 54, 150, 180, 200);
            // Commit instruction
            draw_text(rgba, width, height, "B=SHOVE R=RECOVER  ENTER=COMMIT", 30, height as i32 - 36, 255, 220, 120);
        }
        InteractiveState::Plan => {
            draw_text(rgba, width, height, "PLAN - COMMITTING...", 35, 78, 255, 220, 120);
        }
        InteractiveState::CommitReveal => {
            draw_text(rgba, width, height, "=== COMMIT REVEAL ===", 35, 78, 255, 255, 100);
            let p_action = app
                .timeline_slots
                .first()
                .map(|s| s.as_str())
                .unwrap_or("unknown");
            let o_action = if app.opponent_intent_revealed {
                app.opponent_timeline_slots
                    .first()
                    .map(|s| s.as_str())
                    .unwrap_or("unknown")
            } else {
                "???"
            };
            let p_line = format!("PLAYER: {}", p_action.to_uppercase());
            draw_text(rgba, width, height, &p_line, 35, 108, 255, 220, 60);
            let o_line = format!("OPPONENT: {}", o_action.to_uppercase());
            draw_text(rgba, width, height, &o_line, 35, 138, 255, 80, 40);
            if app.opponent_intent_revealed {
                // Show matchup from combat_contacts
                if let Some(ref contact) = app.combat_contacts.first() {
                    let matchup_line = format!("MATCHUP: {}", contact.outcome);
                    draw_text(rgba, width, height, &matchup_line, 35, 168, 255, 220, 120);
                }
            }
            draw_text(rgba, width, height, "ENTER to continue", 35, 198, 200, 200, 200);
        }
        InteractiveState::Resolve => {
            draw_text(rgba, width, height, "RESOLVE (CONTACT)", 35, 78, 255, 220, 120);
            if let Some(ref contact) = app.combat_contacts.first() {
                let action_line = format!("{} vs {}", contact.player_action.to_uppercase(), contact.opponent_action.to_uppercase());
                draw_text(rgba, width, height, &action_line, 35, 108, 255, 220, 120);
                let outcome_line = format!("RESULT: {}", contact.outcome);
                draw_text(rgba, width, height, &outcome_line, 35, 138, 255, 160, 80);
                // Unit-093: Action-specific contact markers
                let mid_w = (width as i32) / 2;
                let mid_h = (height as i32) / 2;
                match contact.player_action.as_str() {
                    "cut" => {
                        // Slash arc marker
                        draw_contact_marker(rgba, width, height, mid_w - 80, mid_h, mid_w + 40, mid_h - 20);
                        draw_text(rgba, width, height, "X", mid_w - 20, mid_h - 30, 255, 200, 50);
                    }
                    "thrust" => {
                        // Forward line marker
                        draw_contact_marker(rgba, width, height, mid_w - 60, mid_h, mid_w + 60, mid_h);
                        draw_text(rgba, width, height, "->", mid_w - 10, mid_h - 30, 255, 200, 50);
                    }
                    "bash" | "shove" | "kick" => {
                        // Impact burst marker
                        fill_rect(rgba, width, height, mid_w - 15, mid_h - 15, 30, 30, 255, 100, 50);
                        draw_text(rgba, width, height, "BAM", mid_w - 15, mid_h - 35, 255, 100, 50);
                    }
                    "guard" | "parry" | "brace" => {
                        // Block/shield marker
                        draw_panel(rgba, width, height, mid_w - 20, mid_h - 20, 40, 40);
                        draw_text(rgba, width, height, "BLK", mid_w - 15, mid_h - 10, 100, 200, 255);
                    }
                    "grab" | "hook_bind" => {
                        // Bind/grapple marker
                        draw_text(rgba, width, height, "><", mid_w - 10, mid_h - 20, 200, 200, 50);
                    }
                    _ => {
                        draw_contact_marker(rgba, width, height, mid_w - 40, mid_h, mid_w + 40, mid_h);
                    }
                }
            }
            draw_text(rgba, width, height, "ENTER to continue", 35, 168, 200, 200, 200);
        }
        InteractiveState::Consequence => {
            draw_text(rgba, width, height, "CONSEQUENCE", 35, 78, 255, 220, 120);
            if let Some(ref contact) = app.combat_contacts.first() {
                let inj_line = format!("INJURY: {} (severity {})", contact.injury, contact.injury_severity);
                draw_text(rgba, width, height, &inj_line, 35, 108, 255, 160, 80);
                // Unit-093: Severity-based consequence VFX
                let mid_w = (width as i32) / 2;
                let mid_h = (height as i32) / 2 + 80;
                match contact.injury_severity {
                    0 => {
                        draw_text(rgba, width, height, "NO DAMAGE", mid_w - 40, mid_h, 100, 255, 100);
                    }
                    1..=2 => {
                        // Minor: small yellow flash
                        fill_rect(rgba, width, height, mid_w - 20, mid_h - 5, 40, 10, 255, 200, 50);
                        draw_text(rgba, width, height, "GRAZE", mid_w - 20, mid_h + 10, 255, 200, 50);
                    }
                    3..=4 => {
                        // Medium: orange burst
                        fill_rect(rgba, width, height, mid_w - 30, mid_h - 10, 60, 20, 255, 140, 30);
                        draw_text(rgba, width, height, "WOUNDED", mid_w - 30, mid_h + 15, 255, 140, 30);
                    }
                    _ => {
                        // Severe: red flash
                        fill_rect(rgba, width, height, mid_w - 40, mid_h - 15, 80, 30, 255, 50, 30);
                        draw_text(rgba, width, height, "CRITICAL", mid_w - 30, mid_h + 20, 255, 50, 30);
                    }
                }
            }
            draw_text(rgba, width, height, "ENTER to continue", 35, 138, 200, 200, 200);
        }
        InteractiveState::Replan => {
            draw_text(rgba, width, height, "REPLAN", 35, 78, 255, 220, 120);
            draw_text(rgba, width, height, "ENTER for match result", 35, 108, 200, 200, 200);
        }
        InteractiveState::MatchResult => {
            draw_text(rgba, width, height, "MATCH RESULT", 35, 78, 255, 220, 120);
            if let Some(ref result) = app.match_result {
                // Unit-093: Impactful match result display
                let winner_color = match result.winner.as_str() {
                    "player" => (255u8, 255u8, 100u8),
                    "opponent" => (255u8, 80u8, 40u8),
                    _ => (200u8, 200u8, 200u8),
                };
                let winner_line = format!("*** WINNER: {} ***", result.winner.to_uppercase());
                draw_text(rgba, width, height, &winner_line, 35, 108, winner_color.0, winner_color.1, winner_color.2);
                let score_line = format!("PLAYER INJURY: {}  |  OPPONENT INJURY: {}", result.player_injury_score, result.opponent_injury_score);
                draw_text(rgba, width, height, &score_line, 35, 138, 200, 200, 200);
                let why_line = format!("WHY: {}", result.end_condition);
                draw_text(rgba, width, height, &why_line, 35, 168, 200, 180, 100);
            }
            draw_text(rgba, width, height, "ENTER for replay  |  R for replay  |  F for fight film", 35, 198, 200, 200, 200);
        }
        InteractiveState::Replay => {
            draw_text(rgba, width, height, "REPLAY", 35, 78, 255, 220, 120);
            if let Some(ref contact) = app.combat_contacts.first() {
                let trace_line = format!("T0: {} vs {} -> {}", contact.player_action.to_uppercase(), contact.opponent_action.to_uppercase(), contact.outcome);
                draw_text(rgba, width, height, &trace_line, 35, 108, 200, 180, 100);
            }
            draw_text(rgba, width, height, "ENTER for fight film", 35, 138, 200, 200, 200);
        }
        InteractiveState::FightFilm => {
            draw_text(rgba, width, height, "FIGHT FILM", 35, 78, 255, 220, 120);
            if let Some(ref contact) = app.combat_contacts.first() {
                let key_line = format!("KEY: {} vs {}", contact.player_action.to_uppercase(), contact.opponent_action.to_uppercase());
                draw_text(rgba, width, height, &key_line, 35, 108, 255, 220, 120);
                let why_line = format!("WHY: {}", contact.outcome);
                draw_text(rgba, width, height, &why_line, 35, 138, 200, 180, 100);
            }
            draw_text(rgba, width, height, "ENTER to quit", 35, 168, 200, 200, 200);
        }
        InteractiveState::Settings => {
            draw_text(rgba, width, height, "SETTINGS/HELP", 35, 78, 255, 220, 120);
            draw_text(rgba, width, height, "INPUT: KEYBOARD", 35, 108, 200, 200, 200);
            draw_text(rgba, width, height, "V: TOGGLE CAMERA", 35, 138, 200, 200, 200);
            draw_text(rgba, width, height, "ENTER to return", 35, 168, 200, 200, 200);
        }
        InteractiveState::Quit => {
            draw_text(rgba, width, height, "QUIT", 35, 78, 255, 100, 100);
        }
    }

    // Unit-098: Remove debug readouts from player-facing UI.
    // TM:F and IN:N are internal verification data, not player-facing.
    // Only show a small OATHYARD watermark in bottom-right.
    draw_text(rgba, width, height, "OATHYARD", (width as i32) - 170, (height as i32) - 25, 80, 80, 100);

    // Unit-097: Bottom-left persistent control hint bar
    let hint = match app.interactive_state {
        InteractiveState::Boot | InteractiveState::MainMenu => "ENTER=Start  ESC=Quit",
        InteractiveState::FighterSelect => "UP/DOWN=Cycle  ENTER=Confirm",
        InteractiveState::LoadoutSelect => "UP/DOWN=Weapon  L/R=Armor  ENTER=Confirm",
        InteractiveState::ArenaSelect => "L/R=Cycle  ENTER=Start Duel",
        InteractiveState::MatchIntro | InteractiveState::Observe => "ENTER=Plan Actions",
        InteractiveState::Timeline => "1-9,0,G,B,K,R=Actions  L/R=Cursor  ENTER=Commit",
        InteractiveState::Plan => "ENTER=Continue",
        InteractiveState::CommitReveal => "ENTER=See Result",
        InteractiveState::Resolve => "ENTER=See Consequence",
        InteractiveState::Consequence => "ENTER=Continue",
        InteractiveState::Replan => "ENTER=Match Result",
        InteractiveState::MatchResult => "ENTER=Replay  R=Replay  F=FightFilm",
        InteractiveState::Replay => "ENTER=Fight Film",
        InteractiveState::FightFilm => "ENTER=Quit  ESC=Quit",
        InteractiveState::Settings => "ENTER=Return  V=Camera  ESC=Return",
        InteractiveState::Quit => "ESC=Close",
    };
    draw_panel(rgba, width, height, 20, (height as i32) - 45, 500, 28);
    draw_text(rgba, width, height, hint, 28, (height as i32) - 38, 200, 200, 230);

    // Unit-093: Runtime audio feedback — deterministic generated tones
    // Plays a short beep for state transitions and combat events
    play_state_audio(app);
}

// Unit-093: Minimal deterministic audio feedback.
// Generates a short WAV tone and plays it via the system audio backend.
// This is placeholder audio — deterministic, no external files, truth_mutation=false.
fn play_state_audio(app: &WindowedApp) {
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};

    static LAST_AUDIO_FRAME: AtomicU64 = AtomicU64::new(0);
    let current_frame = app.frames_presented as u64;

    // Only play audio on state transitions (not every frame)
    if app.transitions.is_empty() {
        return;
    }
    let last_transition = &app.transitions[app.transitions.len() - 1];
    let expected_frame = LAST_AUDIO_FRAME.load(Ordering::Relaxed);
    if current_frame <= expected_frame + 5 {
        return; // Throttle: at most 1 audio per 5 frames
    }
    LAST_AUDIO_FRAME.store(current_frame, Ordering::Relaxed);

    // Determine tone frequency from the transition
    let freq: u32 = if last_transition.contains("COMMIT_REVEAL") || last_transition.contains("REVEAL") {
        660 // Higher pitch for reveal
    } else if last_transition.contains("CONTACT") || last_transition.contains("RESOLVE") {
        220 // Low impact thud
    } else if last_transition.contains("CONSEQUENCE") {
        330 // Mid consequence
    } else if last_transition.contains("MATCH_RESULT") || last_transition.contains("RESULT") {
        880 // Victory chime
    } else if last_transition.contains("QUIT") {
        110 // Low quit
    } else {
        440 // Default UI beep
    };

    // Generate a minimal WAV file (44 bytes header + 1600 bytes data = 0.1s at 8kHz)
    let sample_rate = 8000u32;
    let duration_samples = 800u32; // 0.1 seconds
    let num_channels = 1u16;
    let bits_per_sample = 16u16;
    let data_size = duration_samples * 2; // 16-bit = 2 bytes per sample
    let byte_rate = sample_rate * (bits_per_sample as u32 / 8) * (num_channels as u32);
    let block_align = (bits_per_sample / 8) * num_channels;

    let mut wav = Vec::with_capacity(44 + data_size as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + data_size).to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&num_channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());

    // Generate sine wave with quick decay envelope
    for i in 0..duration_samples {
        let t = i as f64 / sample_rate as f64;
        let envelope = (1.0 - (i as f64 / duration_samples as f64)).powi(2);
        let sample = (freq as f64 * 2.0 * std::f64::consts::PI * t).sin() * envelope * 0.3;
        let s16 = (sample * 32767.0) as i16;
        wav.extend_from_slice(&s16.to_le_bytes());
    }

    // Write to temp file and play
    let wav_path = std::env::temp_dir().join(format!("oathyard_beep_{}.wav", std::process::id()));
    if std::fs::write(&wav_path, &wav).is_ok() {
        // Try multiple audio backends
        let _ = Command::new("paplay").arg(&wav_path).arg("--client-name=oathyard").spawn();
        let _ = Command::new("aplay").arg(&wav_path).arg("-q").spawn();
        // Clean up after a delay (best-effort, non-blocking)
        let path_clone = wav_path.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(500));
            let _ = std::fs::remove_file(&path_clone);
        });
    }
}

fn write_window_manifest(app: &WindowedApp) {
    let manifest = serde_json::json!({
        "schema": "oathyard.native_window_runtime.v1",
        "product": "OATHYARD",
        "unit": "Unit-074",
        "native_windowed_execution": app.frames_presented > 0,
        "windowing_backend": "winit 0.30",
        "renderer_backend": BACKEND_ID,
        "wgpu_backend": format!("{:?}", app.adapter_info.backend),
        "adapter_name": app.adapter_info.name.clone(),
        "adapter_device_type": format!("{:?}", app.adapter_info.device_type),
        "adapter_vendor": app.adapter_info.vendor,
        "surface_format": format!("{:?}", app.surface_format),
        "present_mode": format!("{:?}", app.present_mode),
        "alpha_mode": format!("{:?}", app.alpha_mode),
        "requested_width": app.surface_config.width,
        "requested_height": app.surface_config.height,
        "actual_width": app.surface_config.width,
        "actual_height": app.surface_config.height,
        "frames_requested": app.smoke_frames,
        "frames_presented": app.frames_presented,
        "redraw_requested_count": app.redraw_requested_count,
        "resize_event_count": app.resize_event_count,
        "surface_reconfigure_count": app.surface_reconfigure_count,
        "input_event_count": app.input_event_count,
        "close_event_handled": app.close_event_handled,
        "smoke_mode": !app.interactive_mode,
        "auto_exit": app.auto_exit,
        "interactive_mode": app.interactive_mode,
        "interactive_mode_supported": true,
        "scripted_input_supported": true,
        "scripted_input_used": app.scripted_input_used,
        "current_interactive_state": app.interactive_state.as_str(),
        "states_visited": app.states_visited,
        "transitions": app.transitions,
        "controls_map": {
            "Enter/Space": "advance to next phase",
            "Escape/Q": "quit cleanly",
            "R": "replay view",
            "F": "fight film view",
            "P": "pause/resume",
            "H/F1": "settings/help overlay",
            "Up/Down/W/S": "fighter select: cycle fighter; loadout: cycle weapon",
            "Left/Right/A/D": "timeline: move cursor; arena: cycle arena; loadout: cycle armor",
            "V": "toggle first-person/third-person camera",
            "1-6": "timeline action placement (step/pivot/guard/cut/thrust/recover)",
            "CloseRequested": "window close button",
        },
        "event_log_path": "windowed_interactive_event_log.jsonl",
        "scenario_id": app.packet_json.get("scenario_id").and_then(Value::as_str).unwrap_or("unknown"),
        "final_truth_hash": app.packet_json.get("final_state_hash").and_then(Value::as_str).unwrap_or("unknown"),
        "replay_verified": app.packet_json.get("generated_after_replay_verify").and_then(Value::as_bool).unwrap_or(false),
        "presentation_packet_schema": "oathyard.post_hash_presentation_packet.v1",
        "truth_mutation": false,
        "mesh_geometry_consumed": app.mesh_asset_count > 0,
        "mesh_asset_count": app.mesh_asset_count,
        "mesh_assets": app.mesh_assets,
        "mesh_vertex_count": app.mesh_vertex_count,
        "mesh_triangle_count": app.mesh_triangle_count,
        "saltreach_duelist_consumed": app.saltreach_consumed,
        "training_yard_consumed": app.training_yard_consumed,
        "owner_visual_acceptance": false,
        "public_demo_ready": false,
        "release_candidate_ready": false,
        "production_renderer_complete": false,
        // Unit-087: In-game roster selection evidence
        "roster_selection": {
            "selected_fighter": ROSTER_FIGHTERS_WINDOWED
                .get(app.roster_fighter_idx)
                .copied()
                .unwrap_or("unknown"),
            "selected_weapon": ROSTER_WEAPONS_WINDOWED
                .get(app.roster_weapon_idx)
                .copied()
                .unwrap_or("unknown"),
            "selected_armor": ROSTER_ARMOR_WINDOWED
                .get(app.roster_armor_idx)
                .copied()
                .unwrap_or("unknown"),
            "selected_arena": ROSTER_ARENAS_WINDOWED
                .get(app.roster_arena_idx)
                .copied()
                .unwrap_or("unknown"),
            "fighter_idx": app.roster_fighter_idx,
            "weapon_idx": app.roster_weapon_idx,
            "armor_idx": app.roster_armor_idx,
            "arena_idx": app.roster_arena_idx,
            "available_fighters": ROSTER_FIGHTERS_WINDOWED,
            "available_weapons": ROSTER_WEAPONS_WINDOWED,
            "available_armor": ROSTER_ARMOR_WINDOWED,
            "available_arenas": ROSTER_ARENAS_WINDOWED,
        },
        // Unit-090: Opponent intent concealment + trace-driven evidence
        "opponent_intent_revealed": app.opponent_intent_revealed,
        "player_timeline_slots": app.timeline_slots,
        "opponent_timeline_slots": app.opponent_timeline_slots,
        "combat_contacts_trace_driven": !app.combat_contacts.is_empty(),
        "match_result_trace_driven": app.match_result.is_some(),
    });

    let manifest_path = app.out_dir.join("native_window_runtime_manifest.json");
    let _ = fs::write(&manifest_path, serde_json::to_string_pretty(&manifest).unwrap_or_default());

    // Write interactive manifest (Unit-074)
    let interactive_manifest = serde_json::json!({
        "schema": "oathyard.native_window_interactive.v1",
        "native_windowed_execution": app.frames_presented > 0,
        "interactive_mode_supported": true,
        "scripted_input_supported": true,
        "scripted_input_used": app.scripted_input_used,
        "manual_input_verified": app.input_event_count > 0 && !app.scripted_input_used,
        "frames_presented": app.frames_presented,
        "redraw_requested_count": app.redraw_requested_count,
        "input_event_count": app.input_event_count,
        "close_event_handled": app.close_event_handled,
        "resize_event_count": app.resize_event_count,
        "surface_reconfigure_count": app.surface_reconfigure_count,
        "states_visited": app.states_visited,
        "transitions": app.transitions,
        "timeline_slots": app.timeline_slots,
        "opponent_timeline_slots": app.opponent_timeline_slots,
        "timeline_slot_count": app.timeline_slot_count,
        "combat_contacts": app.combat_contacts.iter().map(|c| serde_json::json!({
            "slot": c.slot,
            "player_action": c.player_action,
            "opponent_action": c.opponent_action,
            "contact_type": c.contact_type,
            "injury": c.injury,
            "injury_severity": c.injury_severity,
            "outcome": c.outcome,
        })).collect::<Vec<_>>(),
        "combat_contact_count": app.combat_contacts.len(),
        "match_result": if let Some(ref r) = app.match_result {
            serde_json::json!({
                "player_injury_score": r.player_injury_score,
                "opponent_injury_score": r.opponent_injury_score,
                "winner": r.winner,
                "end_condition": r.end_condition,
                "summary": r.summary,
            })
        } else {
            Value::Null
        },
        "controls_map": manifest.get("controls_map").cloned().unwrap_or(Value::Null),
        "event_log_path": "windowed_interactive_event_log.jsonl",
        "scenario_id": app.packet_json.get("scenario_id").and_then(Value::as_str).unwrap_or("unknown"),
        "final_truth_hash": app.packet_json.get("final_state_hash").and_then(Value::as_str).unwrap_or("unknown"),
        "replay_verified": app.packet_json.get("generated_after_replay_verify").and_then(Value::as_bool).unwrap_or(false),
        "post_hash_presentation_packet_hash": app.packet_json.get("presentation_packet_sha256").and_then(Value::as_str).unwrap_or(""),
        "mesh_geometry_consumed": app.mesh_asset_count > 0,
        "mesh_asset_count": app.mesh_asset_count,
        "saltreach_duelist_consumed": app.saltreach_consumed,
        "training_yard_consumed": app.training_yard_consumed,
        "truth_mutation": false,
        "owner_visual_acceptance": false,
        "public_demo_ready": false,
        "release_candidate_ready": false,
    });
    let _ = fs::write(
        app.out_dir.join("native_window_interactive_manifest.json"),
        serde_json::to_string_pretty(&interactive_manifest).unwrap_or_default(),
    );

    // Write structured event log as JSONL
    let mut event_log_jsonl = String::new();
    for event in &app.event_log {
        let entry = serde_json::json!({
            "event_index": event.event_index,
            "frame_index": event.frame_index,
            "event_source": event.event_source,
            "raw_event_type": event.raw_event_type,
            "logical_input": event.logical_input,
            "previous_state": event.previous_state,
            "next_state": event.next_state,
            "accepted": event.accepted,
            "reason_if_ignored": event.reason_if_ignored,
            "truth_mutation": false,
        });
        event_log_jsonl.push_str(&entry.to_string());
        event_log_jsonl.push('\n');
    }
    let _ = fs::write(
        app.out_dir.join("windowed_interactive_event_log.jsonl"),
        event_log_jsonl,
    );

    // Legacy TSV for backward compat
    let event_log_tsv = format!(
        "event\tcount\nframes_presented\t{}\nredraw_requested\t{}\nresize_events\t{}\ninput_events\t{}\nclose_handled\t{}\n",
        app.frames_presented, app.redraw_requested_count, app.resize_event_count,
        app.input_event_count, app.close_event_handled
    );
    let _ = fs::write(app.out_dir.join("windowed_event_log.tsv"), event_log_tsv);
}
