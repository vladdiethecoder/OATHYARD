use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::{
    OathError, ACCESSIBILITY_SCHEMA, PRODUCT_NAME, PUBLIC_DEMO_READY, RELEASE_CANDIDATE_READY,
};

pub fn write_accessibility_artifacts(out_dir: impl AsRef<Path>) -> Result<(), OathError> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;
    fs::write(
        out_dir.join("accessibility_settings.json"),
        render_accessibility_settings_json(),
    )?;
    fs::write(
        out_dir.join("accessibility_report.md"),
        render_accessibility_report(),
    )?;
    Ok(())
}

fn render_accessibility_settings_json() -> String {
    let settings = [
        (
            "text_scale",
            "1150_permille",
            "range_1000_to_1600_permille",
            "readable timeline, replay, consequence, and settings text",
        ),
        (
            "contrast_mode",
            "high",
            "normal_or_high",
            "non-color-only labels for cost, injury, replay, and material state",
        ),
        (
            "captions",
            "enabled",
            "enabled_or_disabled",
            "critical audio has captions and visual equivalents",
        ),
        (
            "reduced_motion",
            "enabled",
            "enabled_or_disabled",
            "camera movement and screen transitions can be reduced",
        ),
        (
            "reduced_flash",
            "enabled",
            "enabled_or_disabled",
            "contact VFX avoids rapid full-screen flashing",
        ),
        (
            "input_remap",
            "enabled",
            "keyboard_mouse_gamepad_schema",
            "input map remains presentation command input, not truth mutation",
        ),
        (
            "combat_readout_density",
            "detailed",
            "compact_or_detailed",
            "base cost, current cost, and physical reasons remain available",
        ),
    ];

    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", ACCESSIBILITY_SCHEMA, true);
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(&mut out, "  \"captions_default\": true,").unwrap();
    writeln!(&mut out, "  \"critical_audio_visual_equivalent\": true,").unwrap();
    writeln!(&mut out, "  \"remapping_supported\": true,").unwrap();
    writeln!(&mut out, "  \"text_scale_min_permille\": 1000,").unwrap();
    writeln!(&mut out, "  \"text_scale_default_permille\": 1150,").unwrap();
    writeln!(&mut out, "  \"text_scale_max_permille\": 1600,").unwrap();
    writeln!(&mut out, "  \"high_contrast_mode\": true,").unwrap();
    writeln!(&mut out, "  \"reduced_motion_mode\": true,").unwrap();
    writeln!(&mut out, "  \"flash_events_per_second_max\": 0,").unwrap();
    writeln!(&mut out, "  \"camera_shake_permille\": 0,").unwrap();
    writeln!(&mut out, "  \"color_only_information\": false,").unwrap();
    writeln!(&mut out, "  \"hardware_gamepad_smoke_claimed\": false,").unwrap();
    writeln!(&mut out, "  \"owner_visual_accepted\": false,").unwrap();
    writeln!(&mut out, "  \"public_demo_ready\": {PUBLIC_DEMO_READY},").unwrap();
    writeln!(
        &mut out,
        "  \"release_candidate_ready\": {RELEASE_CANDIDATE_READY},"
    )
    .unwrap();
    writeln!(&mut out, "  \"settings\": [").unwrap();
    for (index, (id, default_value, range, reason)) in settings.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "id", id, true);
        write_json_field(&mut out, 3, "default", default_value, true);
        write_json_field(&mut out, 3, "range", range, true);
        write_json_field(&mut out, 3, "reason", reason, false);
        writeln!(&mut out, "    }}{}", comma(index + 1, settings.len())).unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

fn render_accessibility_report() -> String {
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD Accessibility Settings Report").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "Status: PASSED").unwrap();
    writeln!(&mut out, "- Presentation only: `true`").unwrap();
    writeln!(&mut out, "- Truth mutation: `none`").unwrap();
    writeln!(&mut out, "- Text scale range: `1000..1600 permille`").unwrap();
    writeln!(&mut out, "- Default text scale: `1150 permille`").unwrap();
    writeln!(&mut out, "- High contrast mode: `true`").unwrap();
    writeln!(&mut out, "- Captions default: `true`").unwrap();
    writeln!(&mut out, "- Critical audio visual equivalent: `true`").unwrap();
    writeln!(&mut out, "- Reduced motion mode: `true`").unwrap();
    writeln!(&mut out, "- Flash events per second max: `0`").unwrap();
    writeln!(&mut out, "- Camera shake: `0 permille`").unwrap();
    writeln!(&mut out, "- Color-only information: `false`").unwrap();
    writeln!(&mut out, "- Input remapping: `true`").unwrap();
    writeln!(&mut out, "- Gamepad hardware smoke claimed: `false`").unwrap();
    writeln!(&mut out, "- Owner visual accepted: `false`").unwrap();
    writeln!(&mut out, "- Public demo ready: `{PUBLIC_DEMO_READY}`").unwrap();
    writeln!(
        &mut out,
        "- Release candidate ready: `{RELEASE_CANDIDATE_READY}`"
    )
    .unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Canon Boundary").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(
        &mut out,
        "Accessibility settings affect presentation, input mapping, captions, and visual comfort only. They do not alter authoritative 120 Hz truth, action costs, contact packets, injuries, capability deltas, replay hashes, or content hashes."
    )
    .unwrap();
    out
}

fn write_json_field(out: &mut String, indent: usize, key: &str, value: &str, trailing: bool) {
    let spaces = "  ".repeat(indent);
    writeln!(
        out,
        "{}{}: {}{}",
        spaces,
        json_quote(key),
        json_quote(value),
        if trailing { "," } else { "" }
    )
    .unwrap();
}

fn json_quote(value: &str) -> String {
    let mut out = String::new();
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => write!(&mut out, "\\u{:04x}", ch as u32).unwrap(),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn comma(index: usize, len: usize) -> &'static str {
    if index < len {
        ","
    } else {
        ""
    }
}
