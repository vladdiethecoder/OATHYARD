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
