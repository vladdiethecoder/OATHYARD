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
    @location(0) color: vec3<f32>,
    @location(1) shade: f32,
    @location(2) material_uv: vec2<f32>,
}

@group(1) @binding(0)
var base_color_texture: texture_2d<f32>;

@group(1) @binding(1)
var normal_texture: texture_2d<f32>;

@group(1) @binding(2)
var orm_texture: texture_2d<f32>;

@group(1) @binding(3)
var material_sampler: sampler;

fn build_camera_vectors() -> vec3<f32> {
    let ro = camera.eye.xyz;
    let look_at = camera.look_at.xyz;
    return ro;
}

fn build_forward(ro: vec3<f32>, look_at: vec3<f32>) -> vec3<f32> {
    return normalize(look_at - ro);
}

fn build_right(forward: vec3<f32>) -> vec3<f32> {
    return normalize(cross(forward, vec3<f32>(0.0, 1.0, 0.0)));
}

fn build_up(forward: vec3<f32>, right: vec3<f32>) -> vec3<f32> {
    return cross(right, forward);
}

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
    let floor = p.y + 0.48;
    let ring = sd_torus_y(p - vec3<f32>(0.0, -0.47, 0.0), 1.48, 0.035);
    let oath_stone = sd_box(rot_y(p - vec3<f32>(0.0, -0.32, -1.22), 0.25), vec3<f32>(0.38, 0.18, 0.12));
    let witness_left = sd_box(rot_y(p - vec3<f32>(-1.38, -0.34, -0.58), 0.10), vec3<f32>(0.12, 0.28, 0.10));
    let witness_right = sd_box(rot_y(p - vec3<f32>(1.38, -0.34, -0.58), -0.10), vec3<f32>(0.12, 0.28, 0.10));
    let guard = 0.35 + 0.40 * packet.seed.x;
    let fighter_a = fighter_sdf(p, -1.0, guard);
    let fighter_b = fighter_sdf(p, 1.0, 1.0 - guard);
    let contact_spark = sd_sphere(p - vec3<f32>(0.02, 0.42, -0.02), 0.08 + packet.seed.y * 0.04);

    var d = floor;
    var mat = 1.0;
    if (ring < d) { d = ring; mat = 2.0; }
    if (oath_stone < d) { d = oath_stone; mat = 3.0; }
    if (witness_left < d) { d = witness_left; mat = 3.0; }
    if (witness_right < d) { d = witness_right; mat = 3.0; }
    if (fighter_a < d) { d = fighter_a; mat = 4.0; }
    if (fighter_b < d) { d = fighter_b; mat = 5.0; }
    if (contact_spark < d) { d = contact_spark; mat = 6.0; }
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

fn material_color(mat: f32, p: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    let grit = 0.5 + 0.5 * sin(37.0 * p.x + 19.0 * p.z + 11.0 * packet.seed.z);
    if (mat < 1.5) { return vec3<f32>(0.16, 0.14, 0.11) + grit * vec3<f32>(0.05, 0.04, 0.03); }
    if (mat < 2.5) { return vec3<f32>(0.55, 0.44, 0.30) + grit * vec3<f32>(0.07, 0.06, 0.03); }
    if (mat < 3.5) { return vec3<f32>(0.28, 0.25, 0.22) + grit * vec3<f32>(0.06, 0.05, 0.04); }
    if (mat < 4.5) { return vec3<f32>(0.38, 0.32, 0.26) + abs(n.y) * vec3<f32>(0.14, 0.11, 0.08); }
    if (mat < 5.5) { return vec3<f32>(0.22, 0.28, 0.33) + abs(n.x) * vec3<f32>(0.16, 0.14, 0.10); }
    return vec3<f32>(1.0, 0.58, 0.20);
}

fn tone_map(x: vec3<f32>) -> vec3<f32> {
    // Unit-048: improved ACES-approx tone map with better contrast
    let y = x * (2.51 * x + vec3<f32>(0.03)) / (x * (2.43 * x + vec3<f32>(0.59)) + vec3<f32>(0.14));
    return pow(clamp(y, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(1.0 / 2.2));
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let uv = input.uv * 2.0 - vec2<f32>(1.0, 1.0);
    let aspect = 1920.0 / 1080.0;

    // Unit-048: use camera uniform for view transform
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
        let base = material_color(hit.y, p, n);
        color = base * (0.18 + diffuse * 1.55 + fill_light) * ao + vec3<f32>(0.65, 0.48, 0.28) * rim_light;
        // contact bloom
        let contact_bloom = exp(-32.0 * length(p - vec3<f32>(0.02, 0.42, -0.02)));
        color = color + vec3<f32>(0.85, 0.48, 0.10) * contact_bloom * 1.8;
    }
    // Unit-048: reduced fog opacity for better visibility
    let fog = clamp(1.0 - exp(-0.060 * hit.x * hit.x), 0.0, 0.65);
    let fog_color = vec3<f32>(0.14, 0.12, 0.10) + vec3<f32>(0.06, 0.04, 0.02) * input.uv.y;
    color = mix(color, fog_color, fog);
    // Unit-048: increased vignette for better focus
    let vignette = smoothstep(1.42, 0.20, length(uv * vec2<f32>(0.82, 1.0)));
    color = color * (0.52 + 0.48 * vignette);
    color = tone_map(color);
    return vec4<f32>(color, 1.0);
}

@vertex
fn mesh_vs_main(input: MeshVertexIn) -> MeshVertexOut {
    // Unit-048: proper camera-based projection
    let ro = camera.eye.xyz;
    let look_at = camera.look_at.xyz;
    let fwd = normalize(look_at - ro);
    let rgt = normalize(cross(fwd, vec3<f32>(0.0, 1.0, 0.0)));
    let up = cross(rgt, fwd);
    let aspect = 1920.0 / 1080.0;
    let fov = camera.eye.w;

    // Transform mesh vertex to world space relative to camera
    let angle = -0.42 + packet.seed.x * 0.26;
    let c = cos(angle);
    let s = sin(angle);
    let world_pos = vec3<f32>(
        c * input.position.x + s * input.position.z,
        input.position.y,
        -s * input.position.x + c * input.position.z,
    );

    // View transform: convert world position to camera-relative
    let rel = world_pos - ro;
    let view_x = dot(rel, rgt);
    let view_y = dot(rel, up);
    let view_z = dot(rel, fwd);

    // Simple perspective projection
    let near = 0.1;
    let proj_scale = 1.0 / (fov * max(view_z, near));
    let clip_x = view_x * proj_scale / aspect;
    let clip_y = view_y * proj_scale;

    var out: MeshVertexOut;
    out.position = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    out.color = input.color;
    out.material_uv = input.material_uv;
    out.shade = clamp(0.54 + view_z * 0.25 + abs(world_pos.y) * 0.12, 0.22, 1.35);
    return out;
}

@fragment
fn mesh_fs_main(input: MeshVertexOut) -> @location(0) vec4<f32> {
    let edge_warmth = 0.18 + 0.10 * packet.seed.y;
    let material_uv = fract(input.material_uv);
    let base_color = textureSample(base_color_texture, material_sampler, material_uv).rgb;
    let normal_sample = textureSample(normal_texture, material_sampler, material_uv).rgb;
    let orm_sample = textureSample(orm_texture, material_sampler, material_uv).rgb;
    let normal_relief = 0.75 + 0.25 * length(normal_sample * 2.0 - vec3<f32>(1.0));
    let roughness = clamp(orm_sample.g, 0.18, 0.95);
    let metallic = clamp(orm_sample.b, 0.0, 1.0);
    let material_color = mix(input.color, base_color, 0.64);
    let spec_edge = vec3<f32>(0.18 + 0.40 * metallic) * (1.0 - roughness) * 0.32;
    let color = tone_map(material_color * input.shade * normal_relief + spec_edge + vec3<f32>(edge_warmth, edge_warmth * 0.62, 0.05));
    return vec4<f32>(color, 0.92);
}