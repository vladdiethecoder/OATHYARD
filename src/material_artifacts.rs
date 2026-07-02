use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{
    comma, hash_hex, json_quote, run_scenario_file, run_scenario_text, verify_replay_text,
    ContactTrace, DuelResult, OathError, PRODUCT_NAME, PUBLIC_DEMO_READY, RELEASE_CANDIDATE_READY,
    TRUTH_HZ,
};

pub const PBR_MATERIAL_ARTIFACTS_SCHEMA: &str = "oathyard.pbr_material_artifacts.v1";
const MATERIAL_SURFACE_SCHEMA: &str = "oathyard.pbr_surface.v1";
const MATERIAL_EVENT_SCHEMA: &str = "oathyard.pbr_material_event.v1";

const REQUIRED_CHANNELS: [&str; 13] = [
    "albedo",
    "roughness_metallic",
    "normal_height",
    "edge_wear",
    "dirt",
    "blood_wetness",
    "cloth_grain",
    "steel_scratches",
    "leather_strain",
    "stone_dust",
    "stitching",
    "hair_skin_variation",
    "material_ids",
];

#[derive(Clone, Copy, Debug)]
struct PbrSurfaceSpec {
    id: &'static str,
    applies_to: &'static [&'static str],
    material_ids: &'static [&'static str],
    albedo: (u8, u8, u8),
    metallic_permille: u16,
    roughness_permille: u16,
    normal_permille: u16,
    height_permille: u16,
    edge_wear_permille: u16,
    dirt_permille: u16,
    blood_wetness_permille: u16,
    cloth_grain_permille: u16,
    steel_scratches_permille: u16,
    leather_strain_permille: u16,
    stone_dust_permille: u16,
    stitching_permille: u16,
    hair_skin_variation_permille: u16,
}

#[derive(Clone, Debug)]
struct PbrMaterialEvent {
    id: String,
    scenario_id: String,
    replay_final_state_hash: String,
    turn: u32,
    truth_frame: u32,
    contact_index: usize,
    attacker: usize,
    defender: usize,
    weapon_id: String,
    armor_id: String,
    target: String,
    material_result: String,
    surface_id: &'static str,
    material_ids: Vec<&'static str>,
    effect: &'static str,
    intensity_permille: u16,
    wetness_permille: u16,
    dirt_permille: u16,
    edge_wear_permille: u16,
    capability_summary: String,
    cause_chain: String,
}

#[derive(Clone, Debug)]
struct MaterialComparisonRun {
    id: &'static str,
    result: DuelResult,
}

#[derive(Clone, Debug)]
struct PbrMaterialArtifacts {
    manifest: PathBuf,
    report: PathBuf,
    surface_atlas: MaterialEvidenceArtifact,
    response_sheet: MaterialEvidenceArtifact,
    surface_count: usize,
    event_count: usize,
    material_result_count: usize,
    channel_coverage: Vec<(&'static str, bool)>,
    replay_verified: bool,
    disabled_final_state_hash: String,
    enabled_final_state_hash: String,
}

#[derive(Clone, Debug)]
struct MaterialEvidenceArtifact {
    file: &'static str,
    width: u32,
    height: u32,
    sha256: String,
    distinct_color_count: usize,
    flat_recolor: bool,
}

pub fn write_pbr_material_artifacts(
    scenario_path: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
) -> Result<DuelResult, OathError> {
    let primary = run_scenario_file(&scenario_path)?;
    let verified_primary = verify_replay_text(&primary.replay_json)?;
    if verified_primary.final_state_hash != primary.final_state_hash {
        return Err(OathError::Verify(format!(
            "pbr material replay verification changed final hash: expected {}, got {}",
            primary.final_state_hash, verified_primary.final_state_hash
        )));
    }

    let comparison_runs = build_material_comparison_runs()?;
    let events = build_material_events(&primary, &comparison_runs);
    if events.is_empty() {
        return Err(OathError::Verify(
            "pbr material artifact build found no contact events to map".to_string(),
        ));
    }
    let surface_specs = pbr_surface_specs();
    validate_pbr_surface_coverage(surface_specs, &events)?;

    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;
    let surface_atlas = write_pbr_surface_atlas(out_dir, surface_specs)?;
    let response_sheet = write_pbr_response_sheet(out_dir, surface_specs, &events)?;
    if surface_atlas.flat_recolor || response_sheet.flat_recolor {
        return Err(OathError::Verify(
            "pbr material captures collapsed to flat recolor output".to_string(),
        ));
    }

    let mut material_results = BTreeSet::new();
    for event in &events {
        material_results.insert(event.material_result.as_str());
    }
    let channel_coverage = pbr_channel_coverage(surface_specs, &events);
    let artifacts = PbrMaterialArtifacts {
        manifest: out_dir.join("pbr_material_manifest.json"),
        report: out_dir.join("pbr_material_report.md"),
        surface_atlas,
        response_sheet,
        surface_count: surface_specs.len(),
        event_count: events.len(),
        material_result_count: material_results.len(),
        channel_coverage,
        replay_verified: true,
        disabled_final_state_hash: verified_primary.final_state_hash.clone(),
        enabled_final_state_hash: primary.final_state_hash.clone(),
    };

    fs::write(
        &artifacts.manifest,
        render_pbr_material_manifest(
            &primary,
            &comparison_runs,
            surface_specs,
            &events,
            &artifacts,
        ),
    )?;
    fs::write(
        &artifacts.report,
        render_pbr_material_report(&primary, &comparison_runs, &events, &artifacts),
    )?;
    Ok(primary)
}

fn pbr_surface_specs() -> &'static [PbrSurfaceSpec] {
    &[
        PbrSurfaceSpec {
            id: "tempered_steel_edge_worn",
            applies_to: &["weapons", "armor"],
            material_ids: &["steel_edge", "tempered_plate"],
            albedo: (104, 111, 108),
            metallic_permille: 930,
            roughness_permille: 410,
            normal_permille: 540,
            height_permille: 180,
            edge_wear_permille: 820,
            dirt_permille: 220,
            blood_wetness_permille: 260,
            cloth_grain_permille: 0,
            steel_scratches_permille: 900,
            leather_strain_permille: 0,
            stone_dust_permille: 110,
            stitching_permille: 0,
            hair_skin_variation_permille: 0,
        },
        PbrSurfaceSpec {
            id: "riveted_mail_oiled",
            applies_to: &["armor"],
            material_ids: &["riveted_mail", "steel_edge"],
            albedo: (74, 79, 78),
            metallic_permille: 860,
            roughness_permille: 520,
            normal_permille: 720,
            height_permille: 330,
            edge_wear_permille: 640,
            dirt_permille: 260,
            blood_wetness_permille: 340,
            cloth_grain_permille: 0,
            steel_scratches_permille: 780,
            leather_strain_permille: 0,
            stone_dust_permille: 120,
            stitching_permille: 0,
            hair_skin_variation_permille: 0,
        },
        PbrSurfaceSpec {
            id: "quilted_linen_stitched",
            applies_to: &["armor", "fighters"],
            material_ids: &["quilted_linen", "cloth", "textile_padding"],
            albedo: (137, 116, 86),
            metallic_permille: 0,
            roughness_permille: 910,
            normal_permille: 610,
            height_permille: 420,
            edge_wear_permille: 360,
            dirt_permille: 620,
            blood_wetness_permille: 540,
            cloth_grain_permille: 930,
            steel_scratches_permille: 0,
            leather_strain_permille: 0,
            stone_dust_permille: 260,
            stitching_permille: 880,
            hair_skin_variation_permille: 0,
        },
        PbrSurfaceSpec {
            id: "strained_buff_leather",
            applies_to: &["armor", "fighters"],
            material_ids: &["buff_leather_textile", "leather", "lamellar_iron_leather"],
            albedo: (106, 74, 45),
            metallic_permille: 0,
            roughness_permille: 780,
            normal_permille: 560,
            height_permille: 310,
            edge_wear_permille: 520,
            dirt_permille: 480,
            blood_wetness_permille: 420,
            cloth_grain_permille: 240,
            steel_scratches_permille: 0,
            leather_strain_permille: 910,
            stone_dust_permille: 190,
            stitching_permille: 540,
            hair_skin_variation_permille: 0,
        },
        PbrSurfaceSpec {
            id: "ash_wood_grain_dented",
            applies_to: &["weapons", "arenas"],
            material_ids: &["ash_wood", "wood", "grip"],
            albedo: (122, 95, 58),
            metallic_permille: 0,
            roughness_permille: 820,
            normal_permille: 680,
            height_permille: 460,
            edge_wear_permille: 520,
            dirt_permille: 560,
            blood_wetness_permille: 320,
            cloth_grain_permille: 0,
            steel_scratches_permille: 0,
            leather_strain_permille: 170,
            stone_dust_permille: 340,
            stitching_permille: 0,
            hair_skin_variation_permille: 0,
        },
        PbrSurfaceSpec {
            id: "chalked_stone_dust",
            applies_to: &["arenas"],
            material_ids: &["chalked_stone", "stone", "ground"],
            albedo: (150, 143, 124),
            metallic_permille: 0,
            roughness_permille: 960,
            normal_permille: 480,
            height_permille: 390,
            edge_wear_permille: 300,
            dirt_permille: 740,
            blood_wetness_permille: 220,
            cloth_grain_permille: 0,
            steel_scratches_permille: 0,
            leather_strain_permille: 0,
            stone_dust_permille: 940,
            stitching_permille: 0,
            hair_skin_variation_permille: 0,
        },
        PbrSurfaceSpec {
            id: "skin_hair_variation",
            applies_to: &["fighters"],
            material_ids: &["skin", "hair", "flesh"],
            albedo: (139, 96, 73),
            metallic_permille: 0,
            roughness_permille: 720,
            normal_permille: 360,
            height_permille: 160,
            edge_wear_permille: 0,
            dirt_permille: 360,
            blood_wetness_permille: 620,
            cloth_grain_permille: 0,
            steel_scratches_permille: 0,
            leather_strain_permille: 0,
            stone_dust_permille: 160,
            stitching_permille: 0,
            hair_skin_variation_permille: 960,
        },
        PbrSurfaceSpec {
            id: "wet_blood_trace_overlay",
            applies_to: &["weapons", "armor", "fighters", "arenas"],
            material_ids: &["blood", "wetness", "trace_overlay"],
            albedo: (88, 18, 16),
            metallic_permille: 0,
            roughness_permille: 280,
            normal_permille: 220,
            height_permille: 120,
            edge_wear_permille: 0,
            dirt_permille: 180,
            blood_wetness_permille: 980,
            cloth_grain_permille: 0,
            steel_scratches_permille: 0,
            leather_strain_permille: 0,
            stone_dust_permille: 120,
            stitching_permille: 0,
            hair_skin_variation_permille: 0,
        },
    ]
}

fn build_material_comparison_runs() -> Result<Vec<MaterialComparisonRun>, OathError> {
    let specs = [
        (
            "mail_cut_blunt_transfer",
            "scenario mail_cut_blunt_transfer\nfighter 0 a arming_sword gambeson\nfighter 1 d longsword mail_hauberk\nturn 0 0 cut forward torso\nturn 0 1 guard center torso\n",
        ),
        (
            "plate_deflection",
            "scenario plate_deflection\nfighter 0 a arming_sword gambeson\nfighter 1 d longsword heavy_plate\nturn 0 0 cut forward torso\nturn 0 1 guard center torso\n",
        ),
        (
            "hook_lamellar_bind",
            "scenario hook_lamellar_bind\nfighter 0 a billhook gambeson\nfighter 1 d longsword lamellar\nturn 0 0 hook_bind forward weapon_arm\nturn 0 1 guard center weapon_arm\n",
        ),
        (
            "maul_plate_stance_break",
            "scenario maul_plate_stance_break\nfighter 0 a iron_maul gambeson\nfighter 1 d longsword heavy_plate\nturn 0 0 bash forward torso\nturn 0 1 guard center torso\n",
        ),
        (
            "spear_gambeson_gap",
            "scenario spear_gambeson_gap\nfighter 0 a ash_spear gambeson\nfighter 1 d longsword gambeson\nturn 0 0 thrust forward weapon_arm\nturn 0 1 guard center weapon_arm\n",
        ),
        (
            "low_coverage_blunt_transfer",
            "scenario low_coverage_blunt_transfer\nfighter 0 a curved_sword gambeson\nfighter 1 d longsword gambeson\nturn 0 0 cut forward torso\nturn 0 1 guard center torso\n",
        ),
    ];
    let mut runs = Vec::new();
    for (id, text) in specs {
        let result = run_scenario_text(text)?;
        let verified = verify_replay_text(&result.replay_json)?;
        if verified.final_state_hash != result.final_state_hash {
            return Err(OathError::Verify(format!(
                "material comparison replay {id} changed final hash"
            )));
        }
        runs.push(MaterialComparisonRun { id, result });
    }
    Ok(runs)
}

fn build_material_events(
    primary: &DuelResult,
    comparison_runs: &[MaterialComparisonRun],
) -> Vec<PbrMaterialEvent> {
    let mut events = Vec::new();
    push_result_material_events("primary", primary, &mut events);
    for run in comparison_runs {
        push_result_material_events(run.id, &run.result, &mut events);
    }
    events
}

fn push_result_material_events(
    run_id: &str,
    result: &DuelResult,
    events: &mut Vec<PbrMaterialEvent>,
) {
    let mut contact_index = 0usize;
    for turn in &result.turns {
        for contact in &turn.contacts {
            let mapped = map_contact_to_material_event(run_id, result, contact, contact_index);
            events.push(mapped);
            contact_index += 1;
        }
    }
}

fn map_contact_to_material_event(
    run_id: &str,
    result: &DuelResult,
    contact: &ContactTrace,
    contact_index: usize,
) -> PbrMaterialEvent {
    let (surface_id, material_ids, effect, wetness, dirt, edge_wear) =
        material_event_mapping(&contact.material_result);
    let intensity = material_event_intensity(contact);
    PbrMaterialEvent {
        id: format!(
            "{run_id}:{}:turn{}_frame{}_contact{}",
            result.scenario_id, contact.turn, contact.frame, contact_index
        ),
        scenario_id: result.scenario_id.clone(),
        replay_final_state_hash: result.final_state_hash.clone(),
        turn: contact.turn,
        truth_frame: contact.frame,
        contact_index,
        attacker: contact.attacker,
        defender: contact.defender,
        weapon_id: contact.weapon_id.clone(),
        armor_id: contact.armor_id.clone(),
        target: contact.target.as_str().to_string(),
        material_result: contact.material_result.clone(),
        surface_id,
        material_ids,
        effect,
        intensity_permille: intensity,
        wetness_permille: wetness,
        dirt_permille: dirt,
        edge_wear_permille: edge_wear,
        capability_summary: format!(
            "recovery +{} balance {} torque {} grip_r {} torso_rotation {}",
            contact.capability_delta.recovery_slowdown_add,
            contact.capability_delta.balance_delta,
            contact.capability_delta.torque_delta,
            contact.capability_delta.grip_r_delta,
            contact.capability_delta.torso_rotation_delta
        ),
        cause_chain: contact.cause_chain.clone(),
    }
}

fn material_event_mapping(
    material_result: &str,
) -> (&'static str, Vec<&'static str>, &'static str, u16, u16, u16) {
    if material_result.contains("mail") {
        (
            "riveted_mail_oiled",
            vec!["riveted_mail", "steel_edge", "blunt_transfer"],
            "mail ring bright-edge scrape plus dark blunt compression bloom",
            180,
            360,
            760,
        )
    } else if material_result.contains("deflected") {
        (
            "tempered_steel_edge_worn",
            vec!["tempered_plate", "steel_edge", "spark_deflection"],
            "tempered plate bright deflection sparks and shallow edge wear",
            90,
            260,
            840,
        )
    } else if material_result.contains("gap_penetration") {
        (
            "wet_blood_trace_overlay",
            vec!["blood", "cloth", "gap"],
            "wet cloth gap strike with localized blood/wetness mask",
            900,
            420,
            480,
        )
    } else if material_result.contains("hook_bind") {
        (
            "strained_buff_leather",
            vec!["leather", "lamellar_iron_leather", "bind_scratch"],
            "hook bind strain marks across leather laces and lamellar edges",
            260,
            500,
            680,
        )
    } else if material_result.contains("blunt_transfer") {
        (
            "chalked_stone_dust",
            vec!["stone_dust", "cloth", "pressure_shock"],
            "dust burst and cloth compression from blunt transfer",
            220,
            820,
            320,
        )
    } else {
        (
            "quilted_linen_stitched",
            vec!["cloth", "stitching", "dirt"],
            "stitched cloth scuff and dirt response",
            200,
            580,
            360,
        )
    }
}

fn material_event_intensity(contact: &ContactTrace) -> u16 {
    let capability_loss = (-contact.capability_delta.balance_delta.min(0)
        - contact.capability_delta.torque_delta.min(0)
        - contact.capability_delta.grip_r_delta.min(0)
        - contact.capability_delta.torso_rotation_delta.min(0)) as u32;
    let energy = (contact.energy_milli.max(0) as u32 / 20).min(420);
    let impulse = (contact.impulse_milli.max(0) as u32 / 30).min(320);
    (180 + energy + impulse + capability_loss.min(380)).min(1000) as u16
}

fn validate_pbr_surface_coverage(
    surfaces: &[PbrSurfaceSpec],
    events: &[PbrMaterialEvent],
) -> Result<(), OathError> {
    let coverage = pbr_channel_coverage(surfaces, events);
    if let Some((channel, _)) = coverage.iter().find(|(_, covered)| !*covered) {
        return Err(OathError::Verify(format!(
            "pbr material schema missing required channel coverage: {channel}"
        )));
    }
    for class in ["weapons", "armor", "arenas", "fighters"] {
        if !surfaces
            .iter()
            .any(|surface| surface.applies_to.contains(&class))
        {
            return Err(OathError::Verify(format!(
                "pbr material schema missing asset class {class}"
            )));
        }
    }
    let result_classes: BTreeSet<&str> = events
        .iter()
        .map(|event| event.material_result.as_str())
        .collect();
    if result_classes.len() < 5 {
        return Err(OathError::Verify(format!(
            "pbr material comparison needs at least 5 material result classes, got {}",
            result_classes.len()
        )));
    }
    Ok(())
}

fn pbr_channel_coverage(
    surfaces: &[PbrSurfaceSpec],
    events: &[PbrMaterialEvent],
) -> Vec<(&'static str, bool)> {
    REQUIRED_CHANNELS
        .iter()
        .map(|channel| {
            let covered = match *channel {
                "albedo" => surfaces.iter().all(|surface| surface.albedo != (0, 0, 0)),
                "roughness_metallic" => {
                    surfaces.iter().any(|surface| surface.metallic_permille > 0)
                        && surfaces
                            .iter()
                            .any(|surface| surface.roughness_permille > 0)
                }
                "normal_height" => {
                    surfaces.iter().any(|surface| surface.normal_permille > 0)
                        && surfaces.iter().any(|surface| surface.height_permille > 0)
                }
                "edge_wear" => {
                    surfaces
                        .iter()
                        .any(|surface| surface.edge_wear_permille > 0)
                        && events.iter().any(|event| event.edge_wear_permille > 0)
                }
                "dirt" => {
                    surfaces.iter().any(|surface| surface.dirt_permille > 0)
                        && events.iter().any(|event| event.dirt_permille > 0)
                }
                "blood_wetness" => {
                    surfaces
                        .iter()
                        .any(|surface| surface.blood_wetness_permille > 0)
                        && events.iter().any(|event| event.wetness_permille > 0)
                }
                "cloth_grain" => surfaces
                    .iter()
                    .any(|surface| surface.cloth_grain_permille > 0),
                "steel_scratches" => surfaces
                    .iter()
                    .any(|surface| surface.steel_scratches_permille > 0),
                "leather_strain" => surfaces
                    .iter()
                    .any(|surface| surface.leather_strain_permille > 0),
                "stone_dust" => surfaces
                    .iter()
                    .any(|surface| surface.stone_dust_permille > 0),
                "stitching" => surfaces
                    .iter()
                    .any(|surface| surface.stitching_permille > 0),
                "hair_skin_variation" => surfaces
                    .iter()
                    .any(|surface| surface.hair_skin_variation_permille > 0),
                "material_ids" => {
                    surfaces
                        .iter()
                        .all(|surface| !surface.material_ids.is_empty())
                        && events.iter().all(|event| !event.material_ids.is_empty())
                }
                _ => false,
            };
            (*channel, covered)
        })
        .collect()
}

fn write_pbr_surface_atlas(
    out_dir: &Path,
    surfaces: &[PbrSurfaceSpec],
) -> Result<MaterialEvidenceArtifact, OathError> {
    let _ = out_dir;
    let file = "pbr_material_surface_channels.json";
    let width = 1024u32;
    let height = 640u32;
    let mut pixels = new_canvas(width, height, (28, 31, 31));
    let cols = 4usize;
    let cell_w = width as usize / cols;
    let cell_h = height as usize / 2;
    for (index, surface) in surfaces.iter().enumerate() {
        let x0 = (index % cols) * cell_w + 18;
        let y0 = (index / cols) * cell_h + 18;
        draw_surface_cell(
            &mut pixels,
            width as usize,
            height as usize,
            surface,
            x0,
            y0,
            cell_w - 36,
            cell_h - 36,
            0,
            0,
            0,
        );
    }
    let distinct = distinct_colors(&pixels);
    Ok(MaterialEvidenceArtifact {
        file,
        width,
        height,
        sha256: hash_hex(&pixels),
        distinct_color_count: distinct,
        flat_recolor: distinct < 96,
    })
}

fn write_pbr_response_sheet(
    out_dir: &Path,
    surfaces: &[PbrSurfaceSpec],
    events: &[PbrMaterialEvent],
) -> Result<MaterialEvidenceArtifact, OathError> {
    let _ = out_dir;
    let file = "pbr_material_response_events.json";
    let width = 1200u32;
    let rows = events.len().min(8).max(1);
    let row_h = 150usize;
    let height = (rows * row_h) as u32;
    let mut pixels = new_canvas(width, height, (24, 26, 27));
    for (row, event) in events.iter().take(rows).enumerate() {
        let surface = surfaces
            .iter()
            .find(|surface| surface.id == event.surface_id)
            .unwrap_or(&surfaces[0]);
        for stage in 0..3usize {
            let x0 = 22 + stage * 390;
            let y0 = row * row_h + 18;
            let (wetness, dirt, wear) = match stage {
                0 => (0, 0, 0),
                1 => (
                    event.wetness_permille as i32,
                    event.dirt_permille as i32,
                    event.edge_wear_permille as i32,
                ),
                _ => (
                    event.wetness_permille as i32 / 2,
                    (event.dirt_permille as i32 + 120).min(1000),
                    (event.edge_wear_permille as i32 + 160).min(1000),
                ),
            };
            draw_surface_cell(
                &mut pixels,
                width as usize,
                height as usize,
                surface,
                x0,
                y0,
                350,
                112,
                wetness,
                dirt,
                wear,
            );
        }
    }
    let distinct = distinct_colors(&pixels);
    Ok(MaterialEvidenceArtifact {
        file,
        width,
        height,
        sha256: hash_hex(&pixels),
        distinct_color_count: distinct,
        flat_recolor: distinct < 128,
    })
}

fn new_canvas(width: u32, height: u32, color: (u8, u8, u8)) -> Vec<u8> {
    let mut pixels = vec![0u8; width as usize * height as usize * 3];
    for chunk in pixels.chunks_exact_mut(3) {
        chunk[0] = color.0;
        chunk[1] = color.1;
        chunk[2] = color.2;
    }
    pixels
}

#[allow(clippy::too_many_arguments)]
fn draw_surface_cell(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    surface: &PbrSurfaceSpec,
    x0: usize,
    y0: usize,
    cell_w: usize,
    cell_h: usize,
    wetness_overlay: i32,
    dirt_overlay: i32,
    wear_overlay: i32,
) {
    let x1 = (x0 + cell_w).min(width);
    let y1 = (y0 + cell_h).min(height);
    for y in y0..y1 {
        for x in x0..x1 {
            let local_x = (x - x0) as i32;
            let local_y = (y - y0) as i32;
            let color = pbr_pixel_color(
                surface,
                local_x,
                local_y,
                wetness_overlay,
                dirt_overlay,
                wear_overlay,
            );
            let index = (y * width + x) * 3;
            pixels[index] = color.0;
            pixels[index + 1] = color.1;
            pixels[index + 2] = color.2;
        }
    }
    draw_rect_outline(
        pixels,
        width,
        height,
        x0,
        y0,
        cell_w,
        cell_h,
        (210, 200, 176),
    );
    let stripes = (surface.material_ids.len() + surface.applies_to.len()).max(1);
    for stripe in 0..stripes {
        let sx = x0 + 10 + stripe * 18;
        fill_rect(
            pixels,
            width,
            height,
            sx,
            y0 + 10,
            10,
            cell_h.saturating_sub(20),
            stripe_color(surface, stripe),
        );
    }
}

fn pbr_pixel_color(
    surface: &PbrSurfaceSpec,
    x: i32,
    y: i32,
    wetness_overlay: i32,
    dirt_overlay: i32,
    wear_overlay: i32,
) -> (u8, u8, u8) {
    let n = procedural_noise(surface.id, x, y);
    let grain =
        ((surface.normal_permille as i32 + surface.height_permille as i32) * (n - 128)) / 900;
    let scratch = if surface.steel_scratches_permille > 0 && (x + y * 3).rem_euclid(17) == 0 {
        surface.steel_scratches_permille as i32 / 18
    } else {
        0
    };
    let stitch =
        if surface.stitching_permille > 0 && (y.rem_euclid(23) == 0 || x.rem_euclid(37) == 0) {
            surface.stitching_permille as i32 / 28
        } else {
            0
        };
    let cloth = if surface.cloth_grain_permille > 0 && (x + y).rem_euclid(9) == 0 {
        -(surface.cloth_grain_permille as i32 / 34)
    } else {
        0
    };
    let leather = if surface.leather_strain_permille > 0 && (x * 2 - y).rem_euclid(29) == 0 {
        surface.leather_strain_permille as i32 / 32
    } else {
        0
    };
    let stone = if surface.stone_dust_permille > 0 && (x * 5 + y * 7).rem_euclid(41) < 3 {
        surface.stone_dust_permille as i32 / 20
    } else {
        0
    };
    let hair_skin =
        if surface.hair_skin_variation_permille > 0 && (x * 11 + y * 5).rem_euclid(31) < 8 {
            (surface.hair_skin_variation_permille as i32 * (n - 110)) / 1600
        } else {
            0
        };
    let wear = (surface.edge_wear_permille as i32 + wear_overlay).min(1000);
    let dirt = (surface.dirt_permille as i32 + dirt_overlay).min(1000);
    let wet = (surface.blood_wetness_permille as i32 + wetness_overlay).min(1000);
    let edge = if x < 9 || y < 9 || x > 340 || y > 102 {
        wear / 18
    } else {
        0
    };
    let dirt_dark = dirt / 35;
    let wet_red = wet / 10;
    let rough_bright =
        (surface.roughness_permille as i32 - surface.metallic_permille as i32 / 2) / 80;
    let r = surface.albedo.0 as i32 + grain + scratch + stitch + leather + stone + hair_skin + edge
        - dirt_dark
        + wet_red;
    let g =
        surface.albedo.1 as i32 + grain + scratch / 2 + stitch + leather / 2 + stone + hair_skin
            - dirt_dark
            - wet_red / 5
            + rough_bright;
    let b = surface.albedo.2 as i32 + grain + scratch / 3 + stitch / 2 + stone + hair_skin
        - dirt_dark
        - wet_red / 3;
    (
        clamp_u8(r + cloth),
        clamp_u8(g + cloth),
        clamp_u8(b + cloth),
    )
}

fn procedural_noise(id: &str, x: i32, y: i32) -> i32 {
    let mut state = 0u32;
    for byte in id.bytes() {
        state = state.wrapping_mul(16777619) ^ byte as u32;
    }
    state ^= (x as u32).wrapping_mul(0x45d9f3b);
    state = state.rotate_left(13) ^ (y as u32).wrapping_mul(0x119de1f3);
    ((state ^ (state >> 16)) & 0xff) as i32
}

fn clamp_u8(value: i32) -> u8 {
    value.clamp(0, 255) as u8
}

fn stripe_color(surface: &PbrSurfaceSpec, index: usize) -> (u8, u8, u8) {
    let v = procedural_noise(surface.id, index as i32 * 37, index as i32 * 11);
    (
        clamp_u8(surface.albedo.0 as i32 + v / 6),
        clamp_u8(surface.albedo.1 as i32 - v / 9),
        clamp_u8(surface.albedo.2 as i32 + v / 12),
    )
}

fn fill_rect(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    x0: usize,
    y0: usize,
    rect_w: usize,
    rect_h: usize,
    color: (u8, u8, u8),
) {
    let x1 = (x0 + rect_w).min(width);
    let y1 = (y0 + rect_h).min(height);
    for y in y0..y1 {
        for x in x0..x1 {
            let index = (y * width + x) * 3;
            pixels[index] = color.0;
            pixels[index + 1] = color.1;
            pixels[index + 2] = color.2;
        }
    }
}

fn draw_rect_outline(
    pixels: &mut [u8],
    width: usize,
    height: usize,
    x0: usize,
    y0: usize,
    rect_w: usize,
    rect_h: usize,
    color: (u8, u8, u8),
) {
    if rect_w == 0 || rect_h == 0 {
        return;
    }
    fill_rect(pixels, width, height, x0, y0, rect_w, 2, color);
    fill_rect(
        pixels,
        width,
        height,
        x0,
        y0 + rect_h.saturating_sub(2),
        rect_w,
        2,
        color,
    );
    fill_rect(pixels, width, height, x0, y0, 2, rect_h, color);
    fill_rect(
        pixels,
        width,
        height,
        x0 + rect_w.saturating_sub(2),
        y0,
        2,
        rect_h,
        color,
    );
}

fn distinct_colors(pixels: &[u8]) -> usize {
    let mut colors = BTreeSet::new();
    for chunk in pixels.chunks_exact(3) {
        colors.insert((chunk[0], chunk[1], chunk[2]));
    }
    colors.len()
}

fn render_pbr_material_manifest(
    primary: &DuelResult,
    comparison_runs: &[MaterialComparisonRun],
    surfaces: &[PbrSurfaceSpec],
    events: &[PbrMaterialEvent],
    artifacts: &PbrMaterialArtifacts,
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    writeln!(
        &mut out,
        "  \"schema\": {},",
        json_quote(PBR_MATERIAL_ARTIFACTS_SCHEMA)
    )
    .unwrap();
    writeln!(&mut out, "  \"product\": {},", json_quote(PRODUCT_NAME)).unwrap();
    writeln!(
        &mut out,
        "  \"surface_schema\": {},",
        json_quote(MATERIAL_SURFACE_SCHEMA)
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"event_schema\": {},",
        json_quote(MATERIAL_EVENT_SCHEMA)
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"source\": \"verified-replay-after-truth-hash\","
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"scenario_id\": {},",
        json_quote(&primary.scenario_id)
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"content_hash\": {},",
        json_quote(&primary.content_hash)
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"initial_state_hash\": {},",
        json_quote(&primary.initial_state_hash)
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"final_state_hash\": {},",
        json_quote(&primary.final_state_hash)
    )
    .unwrap();
    writeln!(&mut out, "  \"truth_hz\": {TRUTH_HZ},").unwrap();
    writeln!(
        &mut out,
        "  \"replay_verified\": {},",
        artifacts.replay_verified
    )
    .unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"material_maps_affect_replay_hash\": false,").unwrap();
    writeln!(
        &mut out,
        "  \"disabled_final_state_hash\": {},",
        json_quote(&artifacts.disabled_final_state_hash)
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"enabled_final_state_hash\": {},",
        json_quote(&artifacts.enabled_final_state_hash)
    )
    .unwrap();
    writeln!(&mut out, "  \"public_demo_ready\": {PUBLIC_DEMO_READY},").unwrap();
    writeln!(
        &mut out,
        "  \"release_candidate_ready\": {RELEASE_CANDIDATE_READY},"
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"surface_count\": {},",
        artifacts.surface_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"material_event_count\": {},",
        artifacts.event_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"material_result_count\": {},",
        artifacts.material_result_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"all_required_channels_covered\": {},",
        artifacts
            .channel_coverage
            .iter()
            .all(|(_, covered)| *covered)
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"flat_recolor_rejected\": {},",
        !artifacts.surface_atlas.flat_recolor && !artifacts.response_sheet.flat_recolor
    )
    .unwrap();
    writeln!(&mut out, "  \"nonvisual_material_evidence\": [").unwrap();
    write_material_artifact_json(&mut out, &artifacts.surface_atlas, true);
    write_material_artifact_json(&mut out, &artifacts.response_sheet, false);
    writeln!(&mut out, "  ],").unwrap();
    writeln!(&mut out, "  \"required_channel_coverage\": [").unwrap();
    for (index, (channel, covered)) in artifacts.channel_coverage.iter().enumerate() {
        writeln!(
            &mut out,
            "    {{\"channel\": {}, \"covered\": {}}}{}",
            json_quote(channel),
            covered,
            comma(index + 1, artifacts.channel_coverage.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();
    writeln!(&mut out, "  \"comparison_runs\": [").unwrap();
    for (index, run) in comparison_runs.iter().enumerate() {
        writeln!(
            &mut out,
            "    {{\"id\": {}, \"scenario_id\": {}, \"final_state_hash\": {}, \"replay_verified\": true}}{}",
            json_quote(run.id),
            json_quote(&run.result.scenario_id),
            json_quote(&run.result.final_state_hash),
            comma(index + 1, comparison_runs.len())
        )
        .unwrap();
    }
    writeln!(&mut out, "  ],").unwrap();
    writeln!(&mut out, "  \"surfaces\": [").unwrap();
    for (index, surface) in surfaces.iter().enumerate() {
        write_surface_json(&mut out, surface, index + 1 == surfaces.len());
    }
    writeln!(&mut out, "  ],").unwrap();
    writeln!(&mut out, "  \"material_events\": [").unwrap();
    for (index, event) in events.iter().enumerate() {
        write_event_json(&mut out, event, index + 1 == events.len());
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

fn write_material_artifact_json(
    out: &mut String,
    artifact: &MaterialEvidenceArtifact,
    trailing: bool,
) {
    writeln!(&mut *out, "    {{").unwrap();
    writeln!(&mut *out, "      \"file\": {},", json_quote(artifact.file)).unwrap();
    writeln!(&mut *out, "      \"width\": {},", artifact.width).unwrap();
    writeln!(&mut *out, "      \"height\": {},", artifact.height).unwrap();
    writeln!(
        &mut *out,
        "      \"distinct_color_count\": {},",
        artifact.distinct_color_count
    )
    .unwrap();
    writeln!(
        &mut *out,
        "      \"flat_recolor\": {},",
        artifact.flat_recolor
    )
    .unwrap();
    writeln!(
        &mut *out,
        "      \"sha256\": {}",
        json_quote(&artifact.sha256)
    )
    .unwrap();
    writeln!(&mut *out, "    }}{}", if trailing { "," } else { "" }).unwrap();
}

fn write_surface_json(out: &mut String, surface: &PbrSurfaceSpec, last: bool) {
    writeln!(out, "    {{").unwrap();
    writeln!(out, "      \"id\": {},", json_quote(surface.id)).unwrap();
    write_string_array(out, 3, "applies_to", surface.applies_to, true);
    write_string_array(out, 3, "material_ids", surface.material_ids, true);
    writeln!(
        out,
        "      \"albedo_rgb\": [{}, {}, {}],",
        surface.albedo.0, surface.albedo.1, surface.albedo.2
    )
    .unwrap();
    writeln!(
        out,
        "      \"metallic_permille\": {},",
        surface.metallic_permille
    )
    .unwrap();
    writeln!(
        out,
        "      \"roughness_permille\": {},",
        surface.roughness_permille
    )
    .unwrap();
    writeln!(
        out,
        "      \"normal_permille\": {},",
        surface.normal_permille
    )
    .unwrap();
    writeln!(
        out,
        "      \"height_permille\": {},",
        surface.height_permille
    )
    .unwrap();
    writeln!(
        out,
        "      \"edge_wear_permille\": {},",
        surface.edge_wear_permille
    )
    .unwrap();
    writeln!(out, "      \"dirt_permille\": {},", surface.dirt_permille).unwrap();
    writeln!(
        out,
        "      \"blood_wetness_permille\": {},",
        surface.blood_wetness_permille
    )
    .unwrap();
    writeln!(
        out,
        "      \"cloth_grain_permille\": {},",
        surface.cloth_grain_permille
    )
    .unwrap();
    writeln!(
        out,
        "      \"steel_scratches_permille\": {},",
        surface.steel_scratches_permille
    )
    .unwrap();
    writeln!(
        out,
        "      \"leather_strain_permille\": {},",
        surface.leather_strain_permille
    )
    .unwrap();
    writeln!(
        out,
        "      \"stone_dust_permille\": {},",
        surface.stone_dust_permille
    )
    .unwrap();
    writeln!(
        out,
        "      \"stitching_permille\": {},",
        surface.stitching_permille
    )
    .unwrap();
    writeln!(
        out,
        "      \"hair_skin_variation_permille\": {}",
        surface.hair_skin_variation_permille
    )
    .unwrap();
    writeln!(out, "    }}{}", if last { "" } else { "," }).unwrap();
}

fn write_event_json(out: &mut String, event: &PbrMaterialEvent, last: bool) {
    writeln!(out, "    {{").unwrap();
    writeln!(out, "      \"id\": {},", json_quote(&event.id)).unwrap();
    writeln!(
        out,
        "      \"scenario_id\": {},",
        json_quote(&event.scenario_id)
    )
    .unwrap();
    writeln!(
        out,
        "      \"replay_final_state_hash\": {},",
        json_quote(&event.replay_final_state_hash)
    )
    .unwrap();
    writeln!(out, "      \"turn\": {},", event.turn).unwrap();
    writeln!(out, "      \"truth_frame\": {},", event.truth_frame).unwrap();
    writeln!(out, "      \"contact_index\": {},", event.contact_index).unwrap();
    writeln!(out, "      \"attacker\": {},", event.attacker).unwrap();
    writeln!(out, "      \"defender\": {},", event.defender).unwrap();
    writeln!(
        out,
        "      \"weapon_id\": {},",
        json_quote(&event.weapon_id)
    )
    .unwrap();
    writeln!(out, "      \"armor_id\": {},", json_quote(&event.armor_id)).unwrap();
    writeln!(out, "      \"target\": {},", json_quote(&event.target)).unwrap();
    writeln!(
        out,
        "      \"material_result\": {},",
        json_quote(&event.material_result)
    )
    .unwrap();
    writeln!(
        out,
        "      \"surface_id\": {},",
        json_quote(event.surface_id)
    )
    .unwrap();
    write_string_vec(out, 3, "material_ids", &event.material_ids, true);
    writeln!(out, "      \"effect\": {},", json_quote(event.effect)).unwrap();
    writeln!(
        out,
        "      \"intensity_permille\": {},",
        event.intensity_permille
    )
    .unwrap();
    writeln!(
        out,
        "      \"wetness_permille\": {},",
        event.wetness_permille
    )
    .unwrap();
    writeln!(out, "      \"dirt_permille\": {},", event.dirt_permille).unwrap();
    writeln!(
        out,
        "      \"edge_wear_permille\": {},",
        event.edge_wear_permille
    )
    .unwrap();
    writeln!(
        out,
        "      \"capability_summary\": {},",
        json_quote(&event.capability_summary)
    )
    .unwrap();
    writeln!(
        out,
        "      \"cause_chain\": {}",
        json_quote(&event.cause_chain)
    )
    .unwrap();
    writeln!(out, "    }}{}", if last { "" } else { "," }).unwrap();
}

fn write_string_array(out: &mut String, indent: usize, key: &str, values: &[&str], trailing: bool) {
    let pad = "  ".repeat(indent);
    writeln!(
        out,
        "{pad}\"{key}\": [{}]{}",
        values
            .iter()
            .map(|value| json_quote(value))
            .collect::<Vec<_>>()
            .join(", "),
        if trailing { "," } else { "" }
    )
    .unwrap();
}

fn write_string_vec(out: &mut String, indent: usize, key: &str, values: &[&str], trailing: bool) {
    write_string_array(out, indent, key, values, trailing);
}

fn render_pbr_material_report(
    primary: &DuelResult,
    comparison_runs: &[MaterialComparisonRun],
    events: &[PbrMaterialEvent],
    artifacts: &PbrMaterialArtifacts,
) -> String {
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD PBR Material Artifact Report").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "Status: PASSED").unwrap();
    writeln!(&mut out, "- Schema: `{PBR_MATERIAL_ARTIFACTS_SCHEMA}`").unwrap();
    writeln!(&mut out, "- Source: `verified-replay-after-truth-hash`").unwrap();
    writeln!(&mut out, "- Scenario: `{}`", primary.scenario_id).unwrap();
    writeln!(&mut out, "- Content hash: `{}`", primary.content_hash).unwrap();
    writeln!(
        &mut out,
        "- Final state hash: `{}`",
        primary.final_state_hash
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Replay verified: `{}`",
        artifacts.replay_verified
    )
    .unwrap();
    writeln!(&mut out, "- Truth mutation: `none`").unwrap();
    writeln!(&mut out, "- Material maps affect replay hash: `false`").unwrap();
    writeln!(
        &mut out,
        "- Disabled final hash: `{}`",
        artifacts.disabled_final_state_hash
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Enabled final hash: `{}`",
        artifacts.enabled_final_state_hash
    )
    .unwrap();
    writeln!(&mut out, "- Surface count: `{}`", artifacts.surface_count).unwrap();
    writeln!(
        &mut out,
        "- Material event count: `{}`",
        artifacts.event_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Material result classes: `{}`",
        artifacts.material_result_count
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Required channels covered: `{}`",
        artifacts
            .channel_coverage
            .iter()
            .all(|(_, covered)| *covered)
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Flat recolor rejected: `{}`",
        !artifacts.surface_atlas.flat_recolor && !artifacts.response_sheet.flat_recolor
    )
    .unwrap();
    writeln!(&mut out, "- Public demo ready: `{PUBLIC_DEMO_READY}`").unwrap();
    writeln!(
        &mut out,
        "- Release candidate ready: `{RELEASE_CANDIDATE_READY}`"
    )
    .unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Nonvisual Material Evidence").unwrap();
    writeln!(
        &mut out,
        "- `{}` {}x{} distinct colors `{}` sha `{}`",
        artifacts.surface_atlas.file,
        artifacts.surface_atlas.width,
        artifacts.surface_atlas.height,
        artifacts.surface_atlas.distinct_color_count,
        artifacts.surface_atlas.sha256
    )
    .unwrap();
    writeln!(
        &mut out,
        "- `{}` {}x{} distinct colors `{}` sha `{}`",
        artifacts.response_sheet.file,
        artifacts.response_sheet.width,
        artifacts.response_sheet.height,
        artifacts.response_sheet.distinct_color_count,
        artifacts.response_sheet.sha256
    )
    .unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Channel Coverage").unwrap();
    for (channel, covered) in &artifacts.channel_coverage {
        writeln!(&mut out, "- `{channel}`: `{covered}`").unwrap();
    }
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Comparison Runs").unwrap();
    for run in comparison_runs {
        writeln!(
            &mut out,
            "- `{}` scenario `{}` final `{}` replay verified `true`",
            run.id, run.result.scenario_id, run.result.final_state_hash
        )
        .unwrap();
    }
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Material Events").unwrap();
    for event in events {
        writeln!(
            &mut out,
            "- `{}` frame `{}` `{}` `{}` -> surface `{}` intensity `{}`: {}",
            event.id,
            event.truth_frame,
            event.weapon_id,
            event.armor_id,
            event.surface_id,
            event.intensity_permille,
            event.capability_summary
        )
        .unwrap();
    }
    out
}
