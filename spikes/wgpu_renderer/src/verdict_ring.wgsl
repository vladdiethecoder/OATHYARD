struct Packet {
    seed: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> packet: Packet;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

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
        if (t > 8.0) { break; }
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
    if (mat < 1.5) { return vec3<f32>(0.18, 0.16, 0.13) + grit * vec3<f32>(0.06, 0.05, 0.04); }
    if (mat < 2.5) { return vec3<f32>(0.62, 0.50, 0.34) + grit * vec3<f32>(0.08, 0.07, 0.03); }
    if (mat < 3.5) { return vec3<f32>(0.33, 0.30, 0.27) + grit * vec3<f32>(0.07, 0.06, 0.04); }
    if (mat < 4.5) { return vec3<f32>(0.40, 0.34, 0.28) + abs(n.y) * vec3<f32>(0.16, 0.13, 0.09); }
    if (mat < 5.5) { return vec3<f32>(0.24, 0.31, 0.36) + abs(n.x) * vec3<f32>(0.18, 0.16, 0.12); }
    return vec3<f32>(1.0, 0.58, 0.20);
}

fn tone_map(x: vec3<f32>) -> vec3<f32> {
    let y = x * (2.51 * x + vec3<f32>(0.03)) / (x * (2.43 * x + vec3<f32>(0.59)) + vec3<f32>(0.14));
    return pow(clamp(y, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(1.0 / 2.2));
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let uv = input.uv * 2.0 - vec2<f32>(1.0, 1.0);
    let aspect = 1920.0 / 1080.0;
    let camera_shift = (packet.seed.w - 0.5) * 0.18;
    let ro = vec3<f32>(0.0 + camera_shift, 0.55, 3.35);
    let look_at = vec3<f32>(0.0, 0.18, -0.10);
    let forward = normalize(look_at - ro);
    let right = normalize(cross(forward, vec3<f32>(0.0, 1.0, 0.0)));
    let up = cross(right, forward);
    let rd = normalize(forward + right * uv.x * aspect * 0.78 + up * uv.y * 0.78);

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
        let contact_bloom = exp(-32.0 * length(p - vec3<f32>(0.02, 0.42, -0.02)));
        color = color + vec3<f32>(1.0, 0.48, 0.10) * contact_bloom * 1.8;
    }
    let fog = clamp(1.0 - exp(-0.080 * hit.x * hit.x), 0.0, 0.82);
    let fog_color = vec3<f32>(0.18, 0.15, 0.13) + vec3<f32>(0.09, 0.06, 0.03) * input.uv.y;
    color = mix(color, fog_color, fog);
    let vignette = smoothstep(1.32, 0.25, length(uv * vec2<f32>(0.86, 1.0)));
    color = color * (0.48 + 0.52 * vignette);
    color = tone_map(color);
    return vec4<f32>(color, 1.0);
}
