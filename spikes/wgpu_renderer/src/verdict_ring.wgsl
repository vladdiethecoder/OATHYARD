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
}

struct MeshVertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) shade: f32,
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
    let fighter_a = fighter_sdf(p, -1.0, guard);
    let fighter_b = fighter_sdf(p, 1.0, 1.0 - guard);
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

// Unit-049: Procedural PBR material function
// material_type: 0=blade/metal, 1=leather, 2=textile/armor, 3=stone, 4=skin
fn procedural_pbr(world_pos: vec3<f32>, n: vec3<f32>, material_type: f32, tint: vec3<f32>) -> vec3<f32> {
    var base = tint;
    var roughness = 0.5;
    var metallic = 0.0;

    if (material_type < 0.5) {
        // Blade/metal: brushed steel pattern
        let blade_uv = fract(vec2<f32>(world_pos.x * 12.0 + world_pos.z * 3.0, world_pos.y * 4.0));
        let brushing = noise2(blade_uv * 8.0) * 0.15 - 0.075;
        base = tint + vec3<f32>(brushing * 0.3, brushing * 0.3, brushing * 0.4);
        roughness = 0.25 + noise2(blade_uv * 4.0) * 0.1;
        metallic = 0.92;
    } else if (material_type < 1.5) {
        // Leather: crosshatch, darker with seam lines
        let leather_uv = vec2<f32>(world_pos.x * 6.0 + world_pos.z * 6.0, world_pos.y * 6.0);
        let cross = max(abs(sin(leather_uv.x * 18.0)), abs(sin(leather_uv.y * 18.0)));
        let grain = noise2(leather_uv * 2.0) * 0.35;
        base = tint * (0.65 + grain + cross * 0.4);
        roughness = 0.72 + cross * 0.18;
        metallic = 0.02;
    } else if (material_type < 2.5) {
        // Textile/armor: woven pattern with quilting
        let weave_uv = fract(vec2<f32>(world_pos.x * 10.0, world_pos.y * 10.0));
        let weave = 0.5 + 0.5 * sin((weave_uv.x + weave_uv.y) * 12.566);
        let quilt = max(0.0, 1.0 - length(fract(vec2<f32>(world_pos.x * 3.0, world_pos.y * 3.0)) - vec2<f32>(0.5)) * 3.5);
        base = tint * (0.82 + weave * 0.12 + quilt * 0.25);
        roughness = 0.78 + weave * 0.1;
        metallic = 0.02;
    } else if (material_type < 3.5) {
        // Stone: rough with cracks, slate/blue-gray tones
        let stone_uv = vec2<f32>(world_pos.x * 2.0 + world_pos.z * 2.0, world_pos.y * 4.0 + world_pos.x * 3.0);
        let chip = noise2(stone_uv * 4.0) * 0.45;
        let crack = smoothstep(0.03, 0.0, abs(noise2(stone_uv * 8.0) - 0.5)) * 0.5;
        base = tint * (0.55 + chip - crack);
        roughness = 0.88;
        metallic = 0.01;
    } else {
        // Skin: subtle color variation, low roughness
        let skin_uv = vec2<f32>(world_pos.x * 3.0, world_pos.y * 5.0);
        let freckle = noise2(skin_uv * 6.0) * 0.25 + noise2(skin_uv * 14.0) * 0.12;
        base = tint * (0.88 + freckle);
        roughness = 0.55 + freckle * 0.12;
        metallic = 0.01;
    }

    return base * (1.0 - metallic * 0.0) + vec3<f32>(metallic * 0.4, metallic * 0.42, metallic * 0.45);
}

fn tone_map(x: vec3<f32>) -> vec3<f32> {
    let y = x * (2.51 * x + vec3<f32>(0.03)) / (x * (2.43 * x + vec3<f32>(0.59)) + vec3<f32>(0.14));
    return pow(clamp(y, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(1.0 / 2.2));
}

// Unit-049: SDF material palette
fn sdf_material_color(mat: f32, p: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    if (mat < 1.5) { return procedural_pbr(p, n, 3.0, vec3<f32>(0.14, 0.12, 0.09)); } // floor
    if (mat < 2.5) { return procedural_pbr(p, n, 3.0, vec3<f32>(0.50, 0.42, 0.28)); } // ring
    if (mat < 3.5) { return procedural_pbr(p, n, 3.0, vec3<f32>(0.32, 0.28, 0.24)); } // stone
    if (mat < 4.5) { return procedural_pbr(p, n, 4.0, vec3<f32>(0.72, 0.40, 0.28)); } // skin/fighter
    if (mat < 5.5) { return procedural_pbr(p, n, 0.0, vec3<f32>(0.78, 0.76, 0.82)); } // blade
    if (mat < 6.5) { return vec3<f32>(1.0, 0.48, 0.12); } // accent
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
        let fill_light = max(dot(n, fill), 0.0) * 0.24;
        let rim_light = pow(max(dot(reflect(-rim, n), -rd), 0.0), 2.5) * 0.42;
        let ao = clamp(0.38 + 0.62 * n.y, 0.16, 1.0);
        let base = sdf_material_color(hit.y, p, n);
        color = base * (0.18 + diffuse * 1.55 + fill_light) * ao + vec3<f32>(0.65, 0.48, 0.28) * rim_light;
        let contact_bloom = exp(-32.0 * length(p - vec3<f32>(0.02, 0.42, -0.02)));
        color = color + vec3<f32>(0.85, 0.48, 0.10) * contact_bloom * 1.8;
    }
    let fog = clamp(1.0 - exp(-0.060 * hit.x * hit.x), 0.0, 0.65);
    let fog_color = vec3<f32>(0.14, 0.12, 0.10) + vec3<f32>(0.06, 0.04, 0.02) * input.uv.y;
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
    out.position = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    out.world_pos = world_pos;
    out.color = input.color;
    out.shade = clamp(0.54 + view_z * 0.25 + abs(world_pos.y) * 0.12, 0.22, 1.35);
    return out;
}

// Unit-049: Triplanar procedural PBR fragment shader
@fragment
fn mesh_fs_main(input: MeshVertexOut) -> @location(0) vec4<f32> {
    let mat_type = mesh_material.material_type;
    let tint = mesh_material.tint.rgb;

    // Compute world-space flat normal via screen-space derivatives
    let dp = dpdx(input.world_pos);
    let dp2 = dpdy(input.world_pos);
    let flat_n = normalize(cross(dp, dp2));
    let n = normalize(mix(flat_n, vec3<f32>(0.0, 1.0, 0.0), 0.15));

    // Triplanar blend weights based on normal
    let blend_weights = max(abs(n), vec3<f32>(0.00001));
    let blend_total = blend_weights.x + blend_weights.y + blend_weights.z;
    let w = blend_weights / vec3<f32>(blend_total);

    // Procedural PBR base
    let base = procedural_pbr(input.world_pos, n, mat_type, tint);

    // Lighting
    let key = normalize(vec3<f32>(-0.48, 0.88, 0.30));
    let fill = normalize(vec3<f32>(0.42, 0.36, 0.82));
    let diffuse = max(dot(n, key), 0.0);
    let fill_light = max(dot(n, fill), 0.0) * 0.25;

    // Simple shadow approximation
    let ao = clamp(0.42 + 0.58 * n.y, 0.18, 1.0);
    let color = base * (0.22 + diffuse * 1.65 + fill_light) * ao * input.shade;

    // Subtle specular for metallic materials
    let spec_power = mix(6.0, 32.0, 1.0 - abs(mat_type - 0.5) * 2.0);
    let rim_vec = normalize(vec3<f32>(0.72, 0.54, -0.72));
    let rim_light = pow(max(dot(reflect(-rim_vec, n), vec3<f32>(0.0, 0.0, 1.0)), 0.0), spec_power) * mix(0.08, 0.38, step(0.5, abs(mat_type - 0.25)));

    let final_color = tone_map(color + vec3<f32>(0.55, 0.40, 0.20) * rim_light);
    return vec4<f32>(final_color, 0.95);
}
