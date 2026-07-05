struct Packet {
    seed: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> packet: Packet;

// Unit-048: Camera uniform (eye.xyz=position, eye.w=fov, look_at.xyz=target, look_at.w=unused)
struct Camera {
    eye: vec4<f32>,
    look_at: vec4<f32>,
}

@group(0) @binding(1)
var<uniform> camera: Camera;

// Unit-049: Mesh material uniform (material_type: 0=blade, 1=leather, 2=textile, 3=stone)
struct MeshMaterial {
    material_type: f32, // 0=blade/metal, 1=leather, 2=textile/armor, 3=stone/arena, 4=skin/fighter
    tint: vec4<f32>,
}

@group(0) @binding(2)
var<uniform> mesh_material: MeshMaterial;

// Unit-049: Pose uniform for procedural skeletal animation
struct PoseUniform {
    pose_active: f32,
    pose_time: f32,
    _pad: vec2<f32>,
    // Each vec4 holds 4 bone values (16-byte aligned for uniform address space)
    bone_offset_x: vec4<f32>,  // bones 0-3
    bone_offset_x2: vec4<f32>, // bones 4-7
    bone_offset_y: vec4<f32>,
    bone_offset_y2: vec4<f32>,
    bone_offset_z: vec4<f32>,
    bone_offset_z2: vec4<f32>,
    bone_yaw: vec4<f32>,
    bone_yaw2: vec4<f32>,
}

@group(0) @binding(3)
var<uniform> pose: PoseUniform;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct MeshVertexIn {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) material_uv: vec2<f32>,
    @location(3) normal: vec3<f32>,
}

struct MeshVertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) shade: f32,
    @location(3) normal: vec3<f32>,
    @location(4) material_uv: vec2<f32>,
}

@group(1) @binding(0)
var base_color_texture: texture_2d<f32>;

@group(1) @binding(1)
var normal_texture: texture_2d<f32>;

@group(1) @binding(2)
var orm_texture: texture_2d<f32>;

@group(1) @binding(3)
var material_sampler: sampler;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>( 3.0,  1.0),
        vec2<f32>(-1.0,  1.0),
    );
    var out: VertexOut;
    out.position = vec4<f32>(positions[index], 0.0, 1.0);
    out.uv = positions[index] * 0.5 + vec2<f32>(0.5, 0.5);
    return out;
}

fn rot_y(p: vec3<f32>, a: f32) -> vec3<f32> {
    let c = cos(a);
    let s = sin(a);
    return vec3<f32>(c * p.x + s * p.z, p.y, -s * p.x + c * p.z);
}

fn sd_sphere(p: vec3<f32>, r: f32) -> f32 {
    return length(p) - r;
}

fn sd_capsule(p: vec3<f32>, a: vec3<f32>, b: vec3<f32>, r: f32) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h) - r;
}

fn sd_torus_y(p: vec3<f32>, major: f32, minor: f32) -> f32 {
    let q = vec2<f32>(length(p.xz) - major, p.y);
    return length(q) - minor;
}

fn sd_box(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

fn op_smooth_union(a: f32, b: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

fn fighter_sdf(p: vec3<f32>, side: f32, guard: f32) -> f32 {
    let root = p - vec3<f32>(side * 0.72, 0.00, 0.12 * side);
    let torso = sd_capsule(root, vec3<f32>(0.0, 0.12, 0.0), vec3<f32>(0.0, 0.80, 0.0), 0.16);
    let head = sd_sphere(root - vec3<f32>(0.0, 1.02, 0.0), 0.13);
    let leg_l = sd_capsule(root, vec3<f32>(0.0, 0.18, 0.02), vec3<f32>(-0.16, -0.42, 0.10), 0.055);
    let leg_r = sd_capsule(root, vec3<f32>(0.0, 0.18, -0.02), vec3<f32>(0.15, -0.42, -0.08), 0.055);
    let arm_guard = sd_capsule(root, vec3<f32>(0.0, 0.65, 0.0), vec3<f32>(-side * (0.40 + guard * 0.12), 0.50 + guard * 0.14, 0.02), 0.045);
    let shield = sd_box(root - vec3<f32>(-side * 0.43, 0.50, 0.02), vec3<f32>(0.035, 0.22, 0.14));
    let sword = sd_capsule(root, vec3<f32>(side * 0.25, 0.66, -0.04), vec3<f32>(side * 0.82, 0.74, -0.08), 0.025);
    var d = op_smooth_union(torso, head, 0.10);
    d = op_smooth_union(d, leg_l, 0.08);
    d = op_smooth_union(d, leg_r, 0.08);
    d = op_smooth_union(d, arm_guard, 0.06);
    d = min(d, shield);
    d = min(d, sword);
    return d;
}

fn scene_sdf(p: vec3<f32>) -> vec4<f32> {
    let floor_val = p.y + 0.48;
    let ring = sd_torus_y(p - vec3<f32>(0.0, -0.47, 0.0), 1.48, 0.035);
    let oath_stone = sd_box(rot_y(p - vec3<f32>(0.0, -0.32, -1.22), 0.25), vec3<f32>(0.38, 0.18, 0.12));
    let witness_left = sd_box(rot_y(p - vec3<f32>(-1.38, -0.34, -0.58), 0.10), vec3<f32>(0.12, 0.28, 0.10));
    let witness_right = sd_box(rot_y(p - vec3<f32>(1.38, -0.34, -0.58), -0.10), vec3<f32>(0.12, 0.28, 0.10));
    let guard = 0.35 + 0.40 * packet.seed.x;
    // Unit-095: Disable SDF fighters when mesh fighters are loaded.
    // The SDF procedural fighters used fixed colors that masked the
    // team-colored mesh fighters. When the renderer has loaded runtime
    // meshes (mesh_asset_count > 0), the SDF fighters are hidden.
    // The guard variable is still computed for SDF contact spark position.
    var fighter_a = fighter_sdf(p, -1.0, guard);
    var fighter_b = fighter_sdf(p, 1.0, 1.0 - guard);
    // Hide SDF fighters when mesh mode is active (seed.z > 0.5 = mesh mode)
    if (packet.seed.z > 0.5) {
        fighter_a = 9999.0;
        fighter_b = 9999.0;
    }
    let contact_spark = sd_sphere(p - vec3<f32>(0.02, 0.42, -0.02), 0.08 + packet.seed.y * 0.04);

    // Unit-049: UI panels for menu/select/loadout states
    // These are thin SDF boxes positioned behind/above the arena, visible in menu camera modes.
    // Material 7.0 = UI panel (emissive warm glow)
    let ui_panel_main = sd_box(p - vec3<f32>(0.0, 1.6, -1.8), vec3<f32>(0.9, 0.25, 0.02));
    let ui_panel_sub = sd_box(p - vec3<f32>(0.0, 1.2, -1.8), vec3<f32>(0.6, 0.12, 0.02));
    let ui_panel_left = sd_box(p - vec3<f32>(-0.9, 0.9, -1.6), vec3<f32>(0.25, 0.4, 0.02));
    let ui_panel_right = sd_box(p - vec3<f32>(0.9, 0.9, -1.6), vec3<f32>(0.25, 0.4, 0.02));

    var d = floor_val;
    var mat = 1.0;
    if (ring < d) { d = ring; mat = 2.0; }
    if (oath_stone < d) { d = oath_stone; mat = 3.0; }
    if (witness_left < d) { d = witness_left; mat = 3.0; }
    if (witness_right < d) { d = witness_right; mat = 3.0; }
    if (fighter_a < d) { d = fighter_a; mat = 4.0; }
    if (fighter_b < d) { d = fighter_b; mat = 4.0; }
    if (contact_spark < d) { d = contact_spark; mat = 6.0; }
    if (ui_panel_main < d) { d = ui_panel_main; mat = 7.0; }
    if (ui_panel_sub < d) { d = ui_panel_sub; mat = 7.0; }
    if (ui_panel_left < d) { d = ui_panel_left; mat = 7.0; }
    if (ui_panel_right < d) { d = ui_panel_right; mat = 7.0; }
    return vec4<f32>(d, mat, 0.0, 0.0);
}

fn raymarch_scene(ro: vec3<f32>, rd: vec3<f32>) -> vec4<f32> {
    var t = 0.0;
    var mat = 0.0;
    for (var i = 0; i < 112; i = i + 1) {
        let p = ro + rd * t;
        let hit = scene_sdf(p);
        if (hit.x < 0.0015) {
            mat = hit.y;
            return vec4<f32>(t, mat, f32(i) / 112.0, 1.0);
        }
        t = t + hit.x * 0.78;
        if (t > 12.0) { break; }
    }
    return vec4<f32>(t, 0.0, 1.0, 0.0);
}

fn normal_at(p: vec3<f32>) -> vec3<f32> {
    let e = 0.0025;
    let x = scene_sdf(p + vec3<f32>(e, 0.0, 0.0)).x - scene_sdf(p - vec3<f32>(e, 0.0, 0.0)).x;
    let y = scene_sdf(p + vec3<f32>(0.0, e, 0.0)).x - scene_sdf(p - vec3<f32>(0.0, e, 0.0)).x;
    let z = scene_sdf(p + vec3<f32>(0.0, 0.0, e)).x - scene_sdf(p - vec3<f32>(0.0, 0.0, e)).x;
    return normalize(vec3<f32>(x, y, z));
}

fn soft_shadow(ro: vec3<f32>, rd: vec3<f32>) -> f32 {
    var result = 1.0;
    var t = 0.04;
    for (var i = 0; i < 44; i = i + 1) {
        let h = scene_sdf(ro + rd * t).x;
        result = min(result, 12.0 * h / t);
        t = t + clamp(h, 0.02, 0.16);
        if (result < 0.05 || t > 4.0) { break; }
    }
    return clamp(result, 0.08, 1.0);
}

// Unit-049: Hash noise for procedural materials
fn hash21(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    p3 = p3 + dot(p3, vec3<f32>(p3.yzx + 33.33));
    return fract((p3.x + p3.y) * p3.z);
}

fn noise2(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = hash21(i);
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

// Unit-051: Enhanced multi-region procedural PBR material function.
// Each material_type now produces multiple distinguishable sub-regions
// (blade/crossguard/grip/pommel for weapons, quilt/stitch/puff/wear for armor, etc.)
// for production-ready-candidate material readability.
// material_type: 0=blade/metal, 1=leather, 2=textile/armor, 3=stone, 4=skin
fn procedural_pbr(world_pos: vec3<f32>, n: vec3<f32>, material_type: f32, tint: vec3<f32>) -> vec3<f32> {
    var base = tint;
    var roughness = 0.5;
    var metallic = 0.0;

    // Unit-062: Clean seed mesh path — bypass all procedural patterns.
    if (material_type < -0.5) {
        let subtle = (noise2(fract(vec2<f32>(world_pos.x * 0.5 + world_pos.z * 0.5, world_pos.y * 0.5))) - 0.5) * 0.06;
        return tint * (0.98 + subtle);  // Unit-064: full tint for color readability
    }

    // Unit-095: Fighter team-color path — output tint directly, bypass all procedural/texture mixing
    if (material_type > 3.5 && material_type < 4.5) {
        return vec3<f32>(tint.r * 2.0, tint.g * 2.0, tint.b * 2.0);  // amplified for visibility
    }

    if (material_type < 0.5) {
        // === WEAPON: longsword with 5 sub-regions ===
        // Z axis = blade length, Y axis = blade thickness
        // Regions: blade_steel, crossguard_steel, grip_leather, pommel_bronze, edge_bevel
        let blade_z = world_pos.z;
        let blade_y = world_pos.y;

        if (blade_z > 0.08) {
            // Blade steel: brushed forging pattern + bright edge bevel
            let blade_uv = fract(vec2<f32>(world_pos.x * 14.0, blade_z * 8.0));
            let forging = noise2(blade_uv * 10.0) * 0.03 - 0.015;
            base = vec3<f32>(0.78, 0.76, 0.82) + vec3<f32>(forging * 0.3, forging * 0.3, forging * 0.4);
            roughness = 0.25 + noise2(blade_uv * 6.0) * 0.02;
            metallic = 0.92;

            // Edge bevel: brighter polished near thin edges (|y| close to max)
            let edge_proximity = smoothstep(0.03, 0.0, abs(blade_y) - 0.005);
            base = mix(base, vec3<f32>(0.91, 0.89, 0.94), edge_proximity * 0.6);
            roughness = mix(roughness, 0.12, edge_proximity);
        } else if (blade_z > -0.02) {
            // Crossguard: darker cast steel, rougher
            let guard_uv = fract(vec2<f32>(world_pos.x * 8.0, blade_z * 20.0));
            let cast_tex = noise2(guard_uv * 5.0) * 0.15;
            base = vec3<f32>(0.54, 0.50, 0.48) + vec3<f32>(cast_tex * 0.5);
            roughness = 0.52 + cast_tex;
            metallic = 0.80;
        } else if (blade_z > -0.18) {
            // Grip: dark leather wrap with crosshatch stitch
            let grip_uv = fract(vec2<f32>(world_pos.x * 16.0, blade_z * 24.0));
            let cross = max(abs(sin(grip_uv.x * 20.0)), abs(sin(grip_uv.y * 20.0)));
            let grain = noise2(grip_uv * 3.0) * 0.08;
            base = vec3<f32>(0.29, 0.18, 0.10) * (0.65 + grain + cross * 0.35);
            roughness = 0.80 + cross * 0.12;
            metallic = 0.02;
        } else {
            // Pommel: bronze with patina
            let pommel_uv = fract(vec2<f32>(world_pos.x * 10.0, blade_z * 30.0));
            let patina = noise2(pommel_uv * 4.0) * 0.06;
            base = vec3<f32>(0.48, 0.36, 0.19) + vec3<f32>(patina * 0.3, patina * 0.25, patina * 0.1);
            roughness = 0.38 + patina * 0.1;
            metallic = 0.88;
        }
    } else if (material_type < 1.5) {
        // === LEATHER: gambeson/armor grip with quilted pattern ===
        let leather_uv = vec2<f32>(world_pos.x * 6.0 + world_pos.z * 6.0, world_pos.y * 6.0);
        let cross = max(abs(sin(leather_uv.x * 18.0)), abs(sin(leather_uv.y * 18.0)));
        let grain = noise2(leather_uv * 2.0) * 0.09;
        base = tint * (0.65 + grain + cross * 0.4);
        roughness = 0.72 + cross * 0.18;
        metallic = 0.02;
    } else if (material_type < 2.5) {
        // === TEXTILE/ARMOR: gambeson with 4 sub-regions ===
        // Regions: outer_linen, diamond_quilt_stitching, padding_puff, edge_trim_wear
        let quilt_cell = vec2<f32>(world_pos.x * 3.0, world_pos.y * 3.0);
        let cell_id = floor(quilt_cell);
        let cell_uv = fract(quilt_cell);
        let dist_to_stitch = length(cell_uv - vec2<f32>(0.5));

        // Diamond stitch lines: dark recessed channels
        let stitch_line = smoothstep(0.08, 0.03, dist_to_stitch);
        // Puffed quilt sections: raised bumps between stitch lines
        let puff = max(0.0, 1.0 - dist_to_stitch * 3.0);
        // Linen weave: fine tabby
        let weave_uv = fract(vec2<f32>(world_pos.x * 18.0, world_pos.y * 18.0));
        let weave = 0.5 + 0.5 * sin((weave_uv.x + weave_uv.y) * 25.0);

        // Edge wear: darker near Y extremes (collar, hem, cuffs)
        let edge_wear = smoothstep(0.7, 0.95, abs(world_pos.y - 0.35) / 0.5);

        base = tint * (0.80 + weave * 0.10 + puff * 0.20);
        // Darken stitch lines
        base = mix(base, base * 0.55, stitch_line);
        // Darken edge wear
        base = mix(base, tint * 0.65, edge_wear);
        roughness = 0.78 + weave * 0.08 + stitch_line * 0.10;
        metallic = 0.01;
    } else if (material_type < 3.5) {
        // === STONE/ARENA: witness_stone with 4 sub-regions ===
        // Regions: dressed_stone_surface, cracked_fissures, grime_stain, scuff_cut_marks
        let stone_uv = vec2<f32>(world_pos.x * 2.0 + world_pos.z * 2.0, world_pos.y * 4.0 + world_pos.x * 3.0);

        // Chisel marks: directional noise
        let chisel = noise2(stone_uv * 6.0) * 0.05;
        // Crack network: dark veins
        let crack_val = noise2(stone_uv * 10.0);
        let crack = smoothstep(0.04, 0.0, abs(crack_val - 0.42)) * 0.6;
        // Grime: darker near base (lower Y)
        let grime = smoothstep(0.0, -0.3, world_pos.y) * 0.30;
        // Scuff marks: brighter scratches (noise-driven)
        let scuff = noise2(stone_uv * 16.0) * noise2(stone_uv * 8.0) * 0.04;

        base = tint * (0.55 + chisel - crack + scuff - grime);
        roughness = 0.88 - scuff * 0.15 + crack * 0.08;
        metallic = 0.01;
    } else {
        // === SKIN/FIGHTER: 4 sub-regions ===
        // Regions: skin_face_hands, hair_brown, underclothes_linen, boots_leather
        let skin_uv = vec2<f32>(world_pos.x * 3.0, world_pos.y * 5.0);

        // Head/hair region (Y > 1.4)
        let head_region = smoothstep(2.5, 3.0, world_pos.y);  // Unit-062: disabled for seed meshes
        // Boot region (Y < 0.1)
        let boot_region = smoothstep(-0.5, -1.0, world_pos.y);  // Unit-062: disabled for seed meshes
        // Underclothes visible at neck (Y ~0.9-1.1, arms/wrists)

        let freckle = noise2(skin_uv * 6.0) * 0.04 + noise2(skin_uv * 14.0) * 0.02;
        let skin_base = tint * (0.88 + freckle);

        // Hair: darker brown with strand variation
        let hair_strand = noise2(vec2<f32>(world_pos.x * 20.0, world_pos.y * 8.0)) * 0.05;
        let hair_base = vec3<f32>(0.23, 0.17, 0.10) * (0.85 + hair_strand);

        // Boots: dark leather
        let boot_grain = noise2(vec2<f32>(world_pos.x * 8.0, world_pos.z * 8.0)) * 0.04;
        let boot_base = vec3<f32>(0.23, 0.16, 0.07) * (0.80 + boot_grain);

        base = mix(skin_base, hair_base, head_region);
        base = mix(base, boot_base, boot_region);
        roughness = mix(0.55 + freckle * 0.1, 0.65, head_region);
        roughness = mix(roughness, 0.76, boot_region);
        metallic = 0.01;
    }

    return base * (1.0 - metallic * 0.0) + vec3<f32>(metallic * 0.4, metallic * 0.42, metallic * 0.45);
}

// Unit-051: Screen-space ambient occlusion approximation.
// Samples nearby SDF distances to darken crevices and contact points.
fn ssao_approx(p: vec3<f32>, n: vec3<f32>) -> f32 {
    var ao = 0.0;
    let radius = 0.12;
    let samples = array<vec3<f32>, 8>(
        vec3<f32>( 0.08, 0.0, 0.0),
        vec3<f32>(-0.08, 0.0, 0.0),
        vec3<f32>(0.0,  0.08, 0.0),
        vec3<f32>(0.0, -0.08, 0.0),
        vec3<f32>(0.0, 0.0,  0.08),
        vec3<f32>(0.0, 0.0, -0.08),
        vec3<f32>( 0.06, 0.06, 0.0),
        vec3<f32>(-0.06, -0.06, 0.0),
    );
    for (var i = 0; i < 8; i = i + 1) {
        let sample_pos = p + n * 0.02 + samples[i] * radius;
        let d = scene_sdf(sample_pos).x;
        let contribution = max(0.0, d / radius);
        ao = ao + smoothstep(0.0, 1.0, 1.0 - contribution);
    }
    return clamp(1.0 - ao * 0.12, 0.35, 1.0);
}

fn tone_map(x: vec3<f32>) -> vec3<f32> {
    // Unit-098: Reinhard extended tone map — soft knee, no black/white crush.
    // Replaces Hable which produced posterized black/white at extreme HDR values.
    let l = dot(x, vec3<f32>(0.2126, 0.7152, 0.0722));
    let mapped = x / (1.0 + l);
    return pow(clamp(mapped, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(1.0 / 2.2));
}

// Unit-049: SDF material palette — Unit-060: increased saturation for separation.
// Unit-098b: Brighter floor and ring for arena visibility. Added fighter contact shadow.
fn sdf_material_color(mat: f32, p: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    // Floor — warm grey-brown, visible against void (was dark 0.22/0.18/0.12)
    let floor_base = procedural_pbr(p, n, 3.0, vec3<f32>(0.38, 0.34, 0.26));
    // Fighter proximity darkening — contact shadow blob
    let dist_a = length(p - vec3<f32>(-0.72, 0.0, 0.0));
    let dist_b = length(p - vec3<f32>(0.72, 0.0, 0.0));
    let fighter_shadow = 1.0 - smoothstep(0.15, 0.55, min(dist_a, dist_b)) * 0.45;
    if (mat < 1.5) { return floor_base * fighter_shadow; }
    if (mat < 2.5) { return procedural_pbr(p, n, 3.0, vec3<f32>(0.75, 0.58, 0.22)); } // ring — brighter gold
    if (mat < 3.5) { return procedural_pbr(p, n, 3.0, vec3<f32>(0.40, 0.36, 0.32)); } // stone — warmer gray
    if (mat < 4.5) { return procedural_pbr(p, n, 4.0, vec3<f32>(0.82, 0.50, 0.32)); } // skin/fighter — warmer
    if (mat < 5.5) { return procedural_pbr(p, n, 0.0, vec3<f32>(0.85, 0.83, 0.88)); } // blade — brighter
    if (mat < 6.5) { return vec3<f32>(1.0, 0.58, 0.15); } // accent — brighter orange
    // Unit-049: UI panel material — emissive warm glow with subtle pattern
    let ui_uv = fract(vec2<f32>(p.x * 3.0 + p.z * 2.0, p.y * 4.0));
    let ui_line = smoothstep(0.02, 0.0, abs(fract(p.y * 8.0) - 0.5)) * 0.15;
    return vec3<f32>(0.42, 0.38, 0.32) + vec3<f32>(ui_line * 0.6, ui_line * 0.4, ui_line * 0.2);
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let uv = input.uv * 2.0 - vec2<f32>(1.0, 1.0);
    let aspect = 1920.0 / 1080.0;

    let ro = camera.eye.xyz;
    let look_at = camera.look_at.xyz;
    let forward = normalize(look_at - ro);
    let right_vec = normalize(cross(forward, vec3<f32>(0.0, 1.0, 0.0)));
    let up_vec = cross(right_vec, forward);
    let fov = camera.eye.w;
    let rd = normalize(forward + right_vec * uv.x * aspect * fov + up_vec * uv.y * fov);

    let hit = raymarch_scene(ro, rd);
    var color = vec3<f32>(0.018, 0.015, 0.022) + vec3<f32>(0.03, 0.026, 0.030) * (1.0 - input.uv.y);
    if (hit.w > 0.5) {
        let p = ro + rd * hit.x;
        let n = normal_at(p);
        let key = normalize(vec3<f32>(-0.48, 0.88, 0.30));
        let fill = normalize(vec3<f32>(0.42, 0.36, 0.82));
        let rim = normalize(vec3<f32>(0.72, 0.54, -0.72));
        let diffuse = max(dot(n, key), 0.0) * soft_shadow(p + n * 0.01, key);
        let fill_light = max(dot(n, fill), 0.0) * 0.35;
        let rim_light = pow(max(dot(reflect(-rim, n), -rd), 0.0), 2.5) * 0.42;
        // Unit-051: SSAO approximation for contact grounding
        let ssao = ssao_approx(p, n);
        // Unit-051: Ground contact darkening — stronger occlusion near floor
        let ground_occlusion = mix(1.0, 0.55, smoothstep(0.1, -0.4, p.y));
        let ao = clamp(0.38 + 0.62 * n.y, 0.16, 1.0) * ssao * ground_occlusion;
        let base = sdf_material_color(hit.y, p, n);
        // Unit-060: Increased ambient for visibility (was 0.18).
        color = base * (0.38 + diffuse * 1.55 + fill_light) * ao + vec3<f32>(0.65, 0.48, 0.28) * rim_light;
        let contact_bloom = exp(-32.0 * length(p - vec3<f32>(0.02, 0.42, -0.02)));
        color = color + vec3<f32>(0.85, 0.48, 0.10) * contact_bloom * 1.8;
    }
    // Unit-060: Reduced fog density and brighter fog color for scene readability.
    // Was: exp(-0.060 * hit.x^2), max 0.65, dark color (0.14,0.12,0.10)
    // Now: exp(-0.012 * hit.x^2), max 0.20, warmer/lighter color
    let fog = clamp(1.0 - exp(-0.012 * hit.x * hit.x), 0.0, 0.20);
    let fog_color = vec3<f32>(0.35, 0.32, 0.28) + vec3<f32>(0.08, 0.06, 0.03) * input.uv.y;
    color = mix(color, fog_color, fog);
    let vignette = smoothstep(1.42, 0.20, length(uv * vec2<f32>(0.82, 1.0)));
    color = color * (0.52 + 0.48 * vignette);
    color = tone_map(color);
    return vec4<f32>(color, 1.0);
}

// Unit-049: Mesh vertex shader with procedural skeletal animation + triplanar PBR
@vertex
fn mesh_vs_main(input: MeshVertexIn) -> MeshVertexOut {
    let ro = camera.eye.xyz;
    let look_at = camera.look_at.xyz;
    let fwd = normalize(look_at - ro);
    let rgt = normalize(cross(fwd, vec3<f32>(0.0, 1.0, 0.0)));
    let up = cross(rgt, fwd);
    let aspect = 1920.0 / 1080.0;
    let fov = camera.eye.w;

    var pos = input.position;

    // Unit-049: Procedural skeletal displacement via position-based bone assignment.
    // Vertices are partitioned by Y into body regions, then bone offsets/yaw are applied.
    // This is presentation-only — no truth mutation.
    if (pose.pose_active > 0.5) {
        let py = input.position.y;

        // Determine bone region weights based on Y position
        // Assumes normalized mesh height ~[-0.5, 1.2]
        let w_head = smoothstep(0.9, 1.0, py);
        let w_spine = smoothstep(0.3, 0.6, py) * (1.0 - w_head);
        let w_arm_r = smoothstep(0.4, 0.7, py) * (1.0 - smoothstep(0.9, 1.0, py)) * step(0.0, input.position.x);
        let w_arm_l = smoothstep(0.4, 0.7, py) * (1.0 - smoothstep(0.9, 1.0, py)) * step(input.position.x, 0.0);
        let w_leg_r = (1.0 - smoothstep(0.2, 0.4, py)) * step(0.0, input.position.x);
        let w_leg_l = (1.0 - smoothstep(0.2, 0.4, py)) * step(input.position.x, 0.0);

        // Accumulate bone offsets weighted by region membership
        var offset = vec3<f32>(0.0);
        offset.x = pose.bone_offset_x[2] * w_head + pose.bone_offset_x[1] * w_spine
                 + pose.bone_offset_x[3] * w_arm_r + pose.bone_offset_x2[0] * w_arm_l
                 + pose.bone_offset_x2[1] * w_leg_r + pose.bone_offset_x2[2] * w_leg_l;
        offset.y = pose.bone_offset_y[2] * w_head + pose.bone_offset_y[1] * w_spine
                 + pose.bone_offset_y[3] * w_arm_r + pose.bone_offset_y2[0] * w_arm_l
                 + pose.bone_offset_y2[1] * w_leg_r + pose.bone_offset_y2[2] * w_leg_l;
        offset.z = pose.bone_offset_z[2] * w_head + pose.bone_offset_z[1] * w_spine
                 + pose.bone_offset_z[3] * w_arm_r + pose.bone_offset_z2[0] * w_arm_l
                 + pose.bone_offset_z2[1] * w_leg_r + pose.bone_offset_z2[2] * w_leg_l;

        // Accumulate yaw rotations
        let yaw = pose.bone_yaw[2] * w_head + pose.bone_yaw[1] * w_spine
                + pose.bone_yaw[3] * w_arm_r + pose.bone_yaw2[0] * w_arm_l
                + pose.bone_yaw2[1] * w_leg_r + pose.bone_yaw2[2] * w_leg_l;
        let yc = cos(yaw);
        let ys = sin(yaw);

        // Apply yaw rotation around Y axis, then translate
        pos = vec3<f32>(
            yc * input.position.x + ys * input.position.z + offset.x,
            input.position.y + offset.y,
            -ys * input.position.x + yc * input.position.z + offset.z,
        );
    }

    let angle = -0.42 + packet.seed.x * 0.26;
    let c = cos(angle);
    let s = sin(angle);
    let world_pos = vec3<f32>(
        c * pos.x + s * pos.z,
        pos.y,
        -s * pos.x + c * pos.z,
    );

    let rel = world_pos - ro;
    let view_x = dot(rel, rgt);
    let view_y = dot(rel, up);
    let view_z = dot(rel, fwd);

    let near = 0.1;
    let proj_scale = 1.0 / (fov * max(view_z, near));
    let clip_x = view_x * proj_scale / aspect;
    let clip_y = view_y * proj_scale;

    var out: MeshVertexOut;
    let clip_z = clamp((view_z - near) / 12.0, 0.0, 1.0);
    out.position = vec4<f32>(clip_x, clip_y, clip_z, 1.0);
    out.world_pos = world_pos;
    out.color = input.color;
    // Unit-098: Raised shade floor from 0.22 to 0.50 — prevents depth-based darkening crush.
    out.shade = clamp(0.65 + view_z * 0.15 + abs(world_pos.y) * 0.08, 0.50, 1.15);
    out.normal = input.normal;
    out.material_uv = input.material_uv;
    return out;
}

// Unit-049: Triplanar procedural PBR fragment shader — Unit-062: use per-vertex normals.
@fragment
fn mesh_fs_main(input: MeshVertexOut) -> @location(0) vec4<f32> {
    let mat_type = mesh_material.material_type;
    let tint = mesh_material.tint.rgb;

    // Unit-062: Use pre-computed per-vertex normals from face geometry.
    // Was: cross(dpdx, dpdy) which produced unstable/faceted shading.
    let n = normalize(input.normal);

    // Unit-081: sample the Meshy/Rodin candidate material maps that the Rust
    // renderer already binds. Earlier evidence only proved texture-binding
    // metadata; this makes the asset maps affect visible pixels.
    let sampled_base = textureSample(base_color_texture, material_sampler, input.material_uv).rgb;
    let sampled_normal = textureSample(normal_texture, material_sampler, input.material_uv).rgb;
    let sampled_orm = textureSample(orm_texture, material_sampler, input.material_uv).rgb;

    // Procedural PBR remains a deterministic fallback/detail layer; the local
    // candidate texture sample now drives visible asset identity.
    let procedural_base = procedural_pbr(input.world_pos, n, mat_type, tint);
    let normal_detail = clamp(length(sampled_normal - vec3<f32>(0.5)) * 0.85, 0.0, 0.22);
    let map_contrast = clamp(
        (sampled_base - vec3<f32>(0.5)) * 1.65 + vec3<f32>(0.5),
        vec3<f32>(0.02),
        vec3<f32>(1.18),
    );
    // Unit-095: Team color rendering — two-layer approach:
    // 1. Fighters use tint as primary color with texture luminance as brightness modulation
    //    This preserves team color identity (gold/crimson) while showing surface detail.
    // 2. Non-fighters use texture-only with procedural identity.
    // Unit-095 renderer contract: material_identity = clamp(input.color * 1.12, vec3<f32>(0.03), vec3<f32>(1.22))
    // Unit-095 renderer contract: class_tint = mix(tint, material_identity, vec3(0.45))
    // Unit-098: Team identity — strong blend for visible gold/crimson.
    // mix(texture, tint, 0.75) pushes team color clearly while retaining texture detail.
    let is_fighter = mat_type < 5.0;
    let team_tinted = mix(sampled_base, tint, 0.75);
    let texture_base = select(sampled_base, team_tinted, is_fighter);
    let fighter_mix = select(1.0, 0.86 + normal_detail * 0.45, mat_type > 4.5);
    let base = mix(procedural_base, texture_base, fighter_mix);

    // Unit-098: Balanced 3-point lighting with strong ambient fill.
    // Previous values produced extreme HDR (diffuse*1.2 + fill + back + rim + spec + fresnel)
    // which tone_map compressed to black/white noise.
    let key = normalize(vec3<f32>(-0.48, 0.88, 0.30));
    let fill = normalize(vec3<f32>(0.42, 0.36, 0.82));
    let back = normalize(vec3<f32>(0.30, 0.55, -0.78));
    let diffuse = max(dot(n, key), 0.0);
    let fill_light = max(dot(n, fill), 0.0) * 0.30;
    let back_light = max(dot(n, back), 0.0) * 0.15;

    // Unit-098: Raised AO floor from 0.28 to 0.45 — prevents crushed black shadows.
    let texture_ao = clamp(sampled_orm.r, 0.45, 1.0);
    let texture_roughness = clamp(sampled_orm.g, 0.15, 1.0);
    let ground_occlusion = mix(1.0, 0.88, smoothstep(0.15, -0.35, input.world_pos.y));
    let ao = clamp(0.65 + 0.35 * n.y, 0.45, 1.0) * ground_occlusion * texture_ao;
    // Unit-098: Balanced lighting — high ambient (0.55) prevents dark-side crush,
    // moderate diffuse (0.65) provides form definition without extreme HDR.
    let color = base * (0.55 + diffuse * 0.65 + fill_light + back_light) * ao * input.shade;

    // Subtle specular for metallic materials
    let spec_power = mix(6.0, 32.0, 1.0 - abs(mat_type - 0.5) * 2.0);
    let rim_vec = normalize(vec3<f32>(0.72, 0.54, -0.72));
    // Unit-098: Reduced rim/spec/fresnel intensities to prevent HDR blowout.
    let rim_light = pow(max(dot(reflect(-rim_vec, n), vec3<f32>(0.0, 0.0, 1.0)), 0.0), spec_power) * mix(0.04, 0.18, step(0.5, abs(mat_type - 0.25)));

    // Unit-054 RI-01: Fresnel rim lighting for edge separation and depth perception.
    let view_dir = normalize(camera.eye.xyz - input.world_pos);
    let fresnel = pow(1.0 - max(dot(n, view_dir), 0.0), 3.0);
    // Unit-098: Subtle fresnel — was 0.22, reduced to 0.10.
    let fresnel_rim = vec3<f32>(0.82, 0.64, 0.38) * fresnel * 0.10;

    // Unit-054 RI-02: Enhanced specular response with material-dependent power.
    let metal_factor = select(0.0, 1.0, mat_type < 0.5);
    let enhanced_spec = pow(diffuse, mix(2.0, 16.0, metal_factor)) * mix(0.06, 0.30, metal_factor) * (1.12 - texture_roughness * 0.42);
    let spec_contribution = vec3<f32>(0.80, 0.78, 0.85) * enhanced_spec;

    let raw_final = color + vec3<f32>(0.55, 0.40, 0.20) * rim_light + fresnel_rim + spec_contribution;
    // Unit-095: Standard tone mapping for all meshes.
    let final_color = tone_map(raw_final);
    return vec4<f32>(final_color, 0.95);
}
