// Unit-081: Native gameplay renderer evidence contract tests.
//
// These tests intentionally stay static/fast: the expensive runtime evidence is
// exercised by shell gates. The tests prevent the evidence generators from
// regressing back to meshless, untextured, or body-only native captures.

use std::fs;

const REQUIRED_MESH_IDS: [&str; 7] = [
    "player_saltreach_duelist",
    "opponent_saltreach_duelist",
    "player_gambeson",
    "opponent_gambeson",
    "player_longsword",
    "opponent_longsword",
    "training_yard",
];

const REQUIRED_TEXTURE_NAMES: [&str; 4] = [
    "saltreach_duelist",
    "longsword",
    "gambeson",
    "training_yard",
];

fn assert_contains_all(haystack: &str, path: &str, needles: &[&str]) {
    for needle in needles {
        assert!(
            haystack.contains(needle),
            "{path} missing Unit-081 contract literal: {needle}"
        );
    }
}

#[test]
fn unit081_native_combat_manifest_stages_seven_textured_meshes() {
    let path = "src/lib.rs";
    let text = fs::read_to_string(path).expect("read src/lib.rs");

    assert_contains_all(&text, path, &REQUIRED_MESH_IDS);
    assert_contains_all(&text, path, &REQUIRED_TEXTURE_NAMES);
    assert_contains_all(
        &text,
        path,
        &[
            "material_separation_classes",
            "fighter_body",
            "armor_clothing",
            "weapon_metal",
            "arena_stone_ground",
            "source-approved runtime texture paths",
            "base_color_texture_path",
            "normal_texture_path",
            "orm_texture_path",
            "saltreach_duelist,longsword,gambeson,training_yard",
            "assets/presentation_runtime/longsword.mesh.json",
            "assets/presentation_runtime/gambeson.mesh.json",
            "\"mesh_asset_class\":\"armor\"",
            "\"mesh_asset_class\":\"weapon\"",
            "\"truth_mutation\":false",
        ],
    );

    assert!(
        !text.contains("assets/runtime/longsword.mesh.json")
            && !text.contains("assets/runtime/gambeson.mesh.json"),
        "native combat path must use renderable presentation_runtime meshes, not runtime metadata shells"
    );
}

#[test]
fn unit081_windowed_and_exchange_scripts_stage_loadout_meshes_and_textures() {
    for path in [
        "tools/run_native_windowed_game.sh",
        "tools/exchange_capture_matrix.sh",
    ] {
        let text = fs::read_to_string(path).expect("read native renderer script");
        assert_contains_all(&text, path, &REQUIRED_MESH_IDS);
        assert_contains_all(&text, path, &REQUIRED_TEXTURE_NAMES);
        assert_contains_all(
            &text,
            path,
            &[
                "material_separation_classes",
                "fighter_body",
                "armor_clothing",
                "weapon_metal",
                "arena_stone_ground",
                "base_color_texture_path",
                "normal_texture_path",
                "orm_texture_path",
                "saltreach_duelist,longsword,gambeson,training_yard",
                "assets/presentation_runtime/longsword.mesh.json",
                "assets/presentation_runtime/gambeson.mesh.json",
                "mesh_asset_class",
                "armor",
                "weapon",
                "truth_mutation",
            ],
        );
        assert!(
            !text.contains("assets/runtime/longsword.mesh.json")
                && !text.contains("assets/runtime/gambeson.mesh.json"),
            "{path} must use renderable presentation_runtime meshes, not runtime metadata shells"
        );
    }
}

#[test]
fn unit081_renderer_binds_explicit_texture_paths_before_material_validation_fallback() {
    let path = "crates/oathyard_renderer/src/main.rs";
    let text = fs::read_to_string(path).expect("read native renderer main");

    let explicit_index = text
        .find("spec.base_color_texture_path.clone()")
        .expect("explicit texture path branch missing");
    let material_validation_index = text
        .find("let material_validation = data")
        .expect("material_validation fallback missing");
    assert!(
        explicit_index < material_validation_index,
        "explicit runtime mesh manifest texture paths must be honored before material_validation fallback"
    );

    assert_contains_all(
        &text,
        path,
        &[
            "material_texture_binding: true",
            "explicit_runtime_mesh_manifest_paths",
            "id == \"player_saltreach\" || id == \"player_saltreach_duelist\"",
            "id == \"opponent_saltreach\" || id == \"opponent_saltreach_duelist\"",
            "id if id == \"player_gambeson\"",
            "id if id == \"opponent_gambeson\"",
            "id if id.contains(\"training_yard\")",
        ],
    );
}

#[test]
fn unit081_mesh_shader_samples_bound_material_textures_and_depths_meshes() {
    let shader_path = "crates/oathyard_renderer/src/verdict_ring.wgsl";
    let shader = fs::read_to_string(shader_path).expect("read renderer shader");

    assert_contains_all(
        &shader,
        shader_path,
        &[
            "@location(4) material_uv: vec2<f32>",
            "out.material_uv = input.material_uv",
            "textureSample(base_color_texture, material_sampler, input.material_uv)",
            "textureSample(normal_texture, material_sampler, input.material_uv)",
            "textureSample(orm_texture, material_sampler, input.material_uv)",
            "candidate texture sample now drives visible asset identity",
            "material_identity = clamp(input.color * 1.12",
            "class_tint = mix(tint, material_identity",
            "let clip_z = clamp((view_z - near) / 12.0, 0.0, 1.0)",
        ],
    );

    let renderer_path = "crates/oathyard_renderer/src/main.rs";
    let renderer = fs::read_to_string(renderer_path).expect("read native renderer main");
    assert_contains_all(
        &renderer,
        renderer_path,
        &[
            "fn depth_stencil_state(depth_write_enabled: bool)",
            "wgpu::TextureFormat::Depth32Float",
            "fn create_depth_texture(",
            "oathyard production render depth target",
            "depth_stencil: Some(depth_stencil_state(true))",
            "RenderPassDepthStencilAttachment",
            ".get(\"texcoords\")",
            "wrap01(mesh_texcoords[vi][0])",
            "material_colors",
            "mesh_material_colors",
        ],
    );
}

#[test]
fn unit081_runtime_asset_sets_stage_coherent_local_meshy_rodin_bundles() {
    let generator_path = "tools/generate_runtime_asset_sets.py";
    let generator = fs::read_to_string(generator_path).expect("read asset-set generator");
    assert_contains_all(
        &generator,
        generator_path,
        &[
            "saltreach_writ_judgement",
            "chainbreaker_gate_clash",
            "reed_bruiser_trial",
            "oathyard_writ",
            "chainbreaker",
            "gate_shield",
            "reed_sentinel",
            "bruiser_oath",
            "mail_hauberk",
            "heavy_plate",
            "lamellar",
            "bruiser_padded_plate",
            "bearded_axe",
            "round_shield",
            "ash_spear",
            "billhook",
            "source-approved runtime texture paths; no mesh may omit base/normal/ORM",
            "TEXCOORD_0",
            "NORMAL",
            "baseColorFactor",
            "positions",
            "normals",
            "texcoords",
            "material_colors",
            "\"truth_mutation\": False",
        ],
    );

    let wrapper_path = "tools/render_runtime_asset_sets.sh";
    let wrapper = fs::read_to_string(wrapper_path).expect("read asset-set render wrapper");
    assert_contains_all(
        &wrapper,
        wrapper_path,
        &[
            "tools/generate_runtime_asset_sets.py",
            "--mesh-manifest-json",
            "--camera-mode \"pre_contact_frame\"",
            "production_renderer_asset_set_${set_id}_1920x1080",
            "runtime_asset_sets_render_manifest.json",
            "production_renderer_manifest.json",
            "runtime_asset_set_candidate_native_3d_capture",
            "mesh_geometry_consumed",
            "mesh_asset_ids",
            "truth_mutation",
        ],
    );
}

#[test]
fn unit081_material_classifier_covers_full_local_asset_family() {
    let renderer_path = "crates/oathyard_renderer/src/main.rs";
    let renderer = fs::read_to_string(renderer_path).expect("read native renderer main");
    assert_contains_all(
        &renderer,
        renderer_path,
        &[
            "full local Meshy/Rodin candidate family",
            "saltreach_duelist",
            "oathyard_writ",
            "chainbreaker",
            "reed_sentinel",
            "gate_shield",
            "bruiser_oath",
            "mail_hauberk",
            "heavy_plate",
            "lamellar",
            "fencer_light",
            "bruiser_padded_plate",
            "curved_sword",
            "bearded_axe",
            "ash_spear",
            "round_shield",
            "iron_maul",
            "arming_sword",
            "billhook",
            "oathyard_verdict_ring",
            "training_yard",
        ],
    );
}
