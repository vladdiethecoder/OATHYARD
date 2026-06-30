use std::fmt::Write as _;

use crate::{hash_hex, OathError, BOOTSTRAP_VERSION};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeaponProfile {
    pub id: &'static str,
    pub display_name: &'static str,
    pub length_mm: i32,
    pub mass_g: i32,
    pub balance_from_grip_mm: i32,
    pub inertia_g_cm2: i32,
    pub edge_permille: i32,
    pub blunt_permille: i32,
    pub pierce_permille: i32,
    pub hook_permille: i32,
    pub grip_points: i32,
    pub reach_mm: i32,
    pub alignment_permille: i32,
    pub follow_through_permille: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArmorProfile {
    pub id: &'static str,
    pub display_name: &'static str,
    pub material: &'static str,
    pub mass_g: i32,
    pub inertia_g_cm2: i32,
    pub torso_coverage_permille: i32,
    pub head_coverage_permille: i32,
    pub weapon_arm_coverage_permille: i32,
    pub lead_leg_coverage_permille: i32,
    pub gap_permille: i32,
    pub deflection_permille: i32,
    pub absorption_permille: i32,
    pub deformation_permille: i32,
    pub blunt_transfer_permille: i32,
    pub binding_permille: i32,
    pub detachment_risk_permille: i32,
}

pub const WEAPONS: [WeaponProfile; 8] = [
    WeaponProfile {
        id: "curved_sword",
        display_name: "Saltreach curved sword",
        length_mm: 880,
        mass_g: 1040,
        balance_from_grip_mm: 160,
        inertia_g_cm2: 7200,
        edge_permille: 880,
        blunt_permille: 220,
        pierce_permille: 250,
        hook_permille: 180,
        grip_points: 1,
        reach_mm: 800,
        alignment_permille: 840,
        follow_through_permille: 860,
    },
    WeaponProfile {
        id: "longsword",
        display_name: "Longsword",
        length_mm: 1220,
        mass_g: 1510,
        balance_from_grip_mm: 145,
        inertia_g_cm2: 12800,
        edge_permille: 760,
        blunt_permille: 320,
        pierce_permille: 620,
        hook_permille: 140,
        grip_points: 2,
        reach_mm: 1110,
        alignment_permille: 860,
        follow_through_permille: 810,
    },
    WeaponProfile {
        id: "bearded_axe",
        display_name: "Bearded axe",
        length_mm: 760,
        mass_g: 1640,
        balance_from_grip_mm: 310,
        inertia_g_cm2: 16600,
        edge_permille: 720,
        blunt_permille: 520,
        pierce_permille: 120,
        hook_permille: 760,
        grip_points: 1,
        reach_mm: 700,
        alignment_permille: 700,
        follow_through_permille: 920,
    },
    WeaponProfile {
        id: "ash_spear",
        display_name: "Ash spear",
        length_mm: 2120,
        mass_g: 1380,
        balance_from_grip_mm: 620,
        inertia_g_cm2: 18200,
        edge_permille: 160,
        blunt_permille: 260,
        pierce_permille: 900,
        hook_permille: 220,
        grip_points: 2,
        reach_mm: 1960,
        alignment_permille: 930,
        follow_through_permille: 640,
    },
    WeaponProfile {
        id: "round_shield",
        display_name: "Round shield and sidearm",
        length_mm: 620,
        mass_g: 2860,
        balance_from_grip_mm: 80,
        inertia_g_cm2: 23800,
        edge_permille: 140,
        blunt_permille: 780,
        pierce_permille: 120,
        hook_permille: 360,
        grip_points: 2,
        reach_mm: 520,
        alignment_permille: 760,
        follow_through_permille: 560,
    },
    WeaponProfile {
        id: "iron_maul",
        display_name: "Iron maul",
        length_mm: 940,
        mass_g: 3320,
        balance_from_grip_mm: 410,
        inertia_g_cm2: 38600,
        edge_permille: 80,
        blunt_permille: 940,
        pierce_permille: 90,
        hook_permille: 160,
        grip_points: 2,
        reach_mm: 850,
        alignment_permille: 650,
        follow_through_permille: 980,
    },
    WeaponProfile {
        id: "arming_sword",
        display_name: "Arming sword",
        length_mm: 910,
        mass_g: 1180,
        balance_from_grip_mm: 115,
        inertia_g_cm2: 7800,
        edge_permille: 820,
        blunt_permille: 260,
        pierce_permille: 420,
        hook_permille: 80,
        grip_points: 1,
        reach_mm: 830,
        alignment_permille: 900,
        follow_through_permille: 740,
    },
    WeaponProfile {
        id: "billhook",
        display_name: "OATHYARD billhook",
        length_mm: 1680,
        mass_g: 2240,
        balance_from_grip_mm: 520,
        inertia_g_cm2: 28500,
        edge_permille: 520,
        blunt_permille: 340,
        pierce_permille: 380,
        hook_permille: 880,
        grip_points: 2,
        reach_mm: 1510,
        alignment_permille: 720,
        follow_through_permille: 880,
    },
];

pub const ARMORS: [ArmorProfile; 6] = [
    ArmorProfile {
        id: "gambeson",
        display_name: "Layered gambeson",
        material: "quilted_linen",
        mass_g: 4200,
        inertia_g_cm2: 9300,
        torso_coverage_permille: 690,
        head_coverage_permille: 110,
        weapon_arm_coverage_permille: 440,
        lead_leg_coverage_permille: 360,
        gap_permille: 310,
        deflection_permille: 180,
        absorption_permille: 520,
        deformation_permille: 360,
        blunt_transfer_permille: 540,
        binding_permille: 130,
        detachment_risk_permille: 80,
    },
    ArmorProfile {
        id: "mail_hauberk",
        display_name: "Riveted mail hauberk",
        material: "riveted_mail",
        mass_g: 9300,
        inertia_g_cm2: 21300,
        torso_coverage_permille: 930,
        head_coverage_permille: 180,
        weapon_arm_coverage_permille: 780,
        lead_leg_coverage_permille: 620,
        gap_permille: 120,
        deflection_permille: 610,
        absorption_permille: 760,
        deformation_permille: 250,
        blunt_transfer_permille: 670,
        binding_permille: 260,
        detachment_risk_permille: 120,
    },
    ArmorProfile {
        id: "heavy_plate",
        display_name: "Verdict heavy plate",
        material: "tempered_plate",
        mass_g: 22800,
        inertia_g_cm2: 51200,
        torso_coverage_permille: 970,
        head_coverage_permille: 820,
        weapon_arm_coverage_permille: 900,
        lead_leg_coverage_permille: 840,
        gap_permille: 80,
        deflection_permille: 860,
        absorption_permille: 700,
        deformation_permille: 190,
        blunt_transfer_permille: 760,
        binding_permille: 330,
        detachment_risk_permille: 90,
    },
    ArmorProfile {
        id: "lamellar",
        display_name: "Lacquered lamellar",
        material: "lamellar_iron_leather",
        mass_g: 13200,
        inertia_g_cm2: 28800,
        torso_coverage_permille: 880,
        head_coverage_permille: 360,
        weapon_arm_coverage_permille: 720,
        lead_leg_coverage_permille: 610,
        gap_permille: 160,
        deflection_permille: 590,
        absorption_permille: 650,
        deformation_permille: 300,
        blunt_transfer_permille: 620,
        binding_permille: 210,
        detachment_risk_permille: 170,
    },
    ArmorProfile {
        id: "fencer_light",
        display_name: "Fencer light harness",
        material: "buff_leather_textile",
        mass_g: 3100,
        inertia_g_cm2: 6200,
        torso_coverage_permille: 430,
        head_coverage_permille: 90,
        weapon_arm_coverage_permille: 360,
        lead_leg_coverage_permille: 240,
        gap_permille: 540,
        deflection_permille: 210,
        absorption_permille: 360,
        deformation_permille: 520,
        blunt_transfer_permille: 470,
        binding_permille: 80,
        detachment_risk_permille: 130,
    },
    ArmorProfile {
        id: "bruiser_padded_plate",
        display_name: "Bruiser padded plate",
        material: "padded_plate_mix",
        mass_g: 18400,
        inertia_g_cm2: 43200,
        torso_coverage_permille: 920,
        head_coverage_permille: 540,
        weapon_arm_coverage_permille: 760,
        lead_leg_coverage_permille: 700,
        gap_permille: 140,
        deflection_permille: 690,
        absorption_permille: 820,
        deformation_permille: 340,
        blunt_transfer_permille: 520,
        binding_permille: 290,
        detachment_risk_permille: 150,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FighterTradition {
    pub id: &'static str,
    pub display_name: &'static str,
    pub body_mass_g: i32,
    pub reach_bias_mm: i32,
    pub stance_bias_permille: i32,
    pub default_weapon: &'static str,
    pub default_armor: &'static str,
    pub affordance: &'static str,
}

pub const FIGHTER_TRADITIONS: [FighterTradition; 6] = [
    FighterTradition {
        id: "saltreach_duelist",
        display_name: "Saltreach Duelist",
        body_mass_g: 74400,
        reach_bias_mm: 20,
        stance_bias_permille: 1010,
        default_weapon: "curved_sword",
        default_armor: "fencer_light",
        affordance: "curved cuts, evasive pivots, light recovery",
    },
    FighterTradition {
        id: "oathyard_writ",
        display_name: "OATHYARD Writ-Bearer",
        body_mass_g: 81600,
        reach_bias_mm: 50,
        stance_bias_permille: 1030,
        default_weapon: "longsword",
        default_armor: "mail_hauberk",
        affordance: "balanced cuts, thrusts, and guards",
    },
    FighterTradition {
        id: "chainbreaker",
        display_name: "Chainbreaker Axe",
        body_mass_g: 86200,
        reach_bias_mm: -20,
        stance_bias_permille: 960,
        default_weapon: "bearded_axe",
        default_armor: "lamellar",
        affordance: "hooks, binds, and armor shifting blows",
    },
    FighterTradition {
        id: "reed_sentinel",
        display_name: "Reed Sentinel",
        body_mass_g: 79100,
        reach_bias_mm: 240,
        stance_bias_permille: 980,
        default_weapon: "ash_spear",
        default_armor: "gambeson",
        affordance: "long reach thrusts and braced lanes",
    },
    FighterTradition {
        id: "gate_shield",
        display_name: "Gate Shield",
        body_mass_g: 90400,
        reach_bias_mm: -80,
        stance_bias_permille: 1080,
        default_weapon: "round_shield",
        default_armor: "heavy_plate",
        affordance: "bashes, bracing, and shield binds",
    },
    FighterTradition {
        id: "bruiser_oath",
        display_name: "Bruiser Oath",
        body_mass_g: 98200,
        reach_bias_mm: -40,
        stance_bias_permille: 940,
        default_weapon: "iron_maul",
        default_armor: "bruiser_padded_plate",
        affordance: "heavy momentum, blunt trauma, slow recovery",
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArenaProfile {
    pub id: &'static str,
    pub display_name: &'static str,
    pub radius_mm: i32,
    pub surface_material: &'static str,
    pub collision_profile: &'static str,
    pub camera_anchor: &'static str,
}

pub const ARENAS: [ArenaProfile; 2] = [
    ArenaProfile {
        id: "oathyard_verdict_ring",
        display_name: "OATHYARD Verdict Ring",
        radius_mm: 6200,
        surface_material: "chalked_stone",
        collision_profile: "low rim, center oath mark, no hazards",
        camera_anchor: "north_judgment_balcony",
    },
    ArenaProfile {
        id: "training_yard",
        display_name: "Training Yard",
        radius_mm: 4800,
        surface_material: "packed_clay",
        collision_profile: "flat measured yard, debug markers excluded from production",
        camera_anchor: "west_rope_line",
    },
];

pub fn weapon_by_id(id: &str) -> Option<WeaponProfile> {
    WEAPONS.iter().copied().find(|weapon| weapon.id == id)
}

pub fn armor_by_id(id: &str) -> Option<ArmorProfile> {
    ARMORS.iter().copied().find(|armor| armor.id == id)
}

/// R-GAP-1: Freeze gate for content-table asset lookups.
///
/// Enforces the combat-truth freeze gate before resolving any asset ID
/// against the content tables. AI-derived assets (prefix "ai:") must have
/// passed all five freeze conditions. Non-AI compile-time assets pass through
/// without a registry lookup.
///
/// Currently safe because all content tables (WEAPONS, ARMORS,
/// FIGHTER_TRADITIONS, ARENAS) are compile-time constants. But this gate
/// MUST be called by any future runtime content-loading path that accepts
/// non-compiled-in tables derived from AI-generated data.
pub fn enforce_content_freeze_gate(asset_id: &str) -> Result<(), OathError> {
    let repo_root = crate::freeze_status::oathyard_repo_root();
    crate::freeze_status::enforce_combat_truth_freeze_gate(&repo_root, asset_id)
}

/// R-GAP-1: Batch freeze gate for content-table lookups.
///
/// Convenience for checking multiple asset IDs (e.g. weapon_id + armor_id
/// from a fighter spec) in one call.
pub fn enforce_content_freeze_gate_batch<I>(asset_ids: I) -> Result<(), OathError>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let repo_root = crate::freeze_status::oathyard_repo_root();
    crate::freeze_status::enforce_combat_truth_freeze_gate_batch(&repo_root, asset_ids)
}

pub fn content_hash() -> String {
    let mut text = String::new();
    writeln!(&mut text, "version:{BOOTSTRAP_VERSION}").unwrap();
    for weapon in WEAPONS {
        writeln!(
            &mut text,
            "weapon:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
            weapon.id,
            weapon.display_name,
            weapon.length_mm,
            weapon.mass_g,
            weapon.balance_from_grip_mm,
            weapon.inertia_g_cm2,
            weapon.edge_permille,
            weapon.blunt_permille,
            weapon.pierce_permille,
            weapon.hook_permille,
            weapon.grip_points,
            weapon.reach_mm,
            weapon.alignment_permille,
            weapon.follow_through_permille
        )
        .unwrap();
    }
    for armor in ARMORS {
        writeln!(
            &mut text,
            "armor:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
            armor.id,
            armor.display_name,
            armor.material,
            armor.mass_g,
            armor.inertia_g_cm2,
            armor.torso_coverage_permille,
            armor.head_coverage_permille,
            armor.weapon_arm_coverage_permille,
            armor.lead_leg_coverage_permille,
            armor.gap_permille,
            armor.deflection_permille,
            armor.absorption_permille,
            armor.deformation_permille,
            armor.blunt_transfer_permille,
            armor.binding_permille,
            armor.detachment_risk_permille
        )
        .unwrap();
    }
    for fighter in FIGHTER_TRADITIONS {
        writeln!(
            &mut text,
            "fighter:{}:{}:{}:{}:{}:{}:{}:{}",
            fighter.id,
            fighter.display_name,
            fighter.body_mass_g,
            fighter.reach_bias_mm,
            fighter.stance_bias_permille,
            fighter.default_weapon,
            fighter.default_armor,
            fighter.affordance
        )
        .unwrap();
    }
    for arena in ARENAS {
        writeln!(
            &mut text,
            "arena:{}:{}:{}:{}:{}:{}",
            arena.id,
            arena.display_name,
            arena.radius_mm,
            arena.surface_material,
            arena.collision_profile,
            arena.camera_anchor
        )
        .unwrap();
    }
    hash_hex(text.as_bytes())
}
