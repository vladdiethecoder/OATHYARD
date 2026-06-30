use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::{
    OathError, PRODUCT_NAME, PUBLIC_DEMO_READY, RELEASE_CANDIDATE_READY, RUNTIME_SETTINGS_SCHEMA,
    TRUTH_HZ,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct RuntimeSettings {
    profile_id: String,
    text_scale_permille: i32,
    high_contrast: bool,
    captions_enabled: bool,
    reduced_motion: bool,
    reduced_flash: bool,
    master_gain_permille: i32,
    ui_gain_permille: i32,
    impact_gain_permille: i32,
    capability_gain_permille: i32,
    mute_master: bool,
    hold_to_commit: bool,
    toggle_guard: bool,
    input_profile_id: String,
}

impl RuntimeSettings {
    fn default_profile() -> Self {
        Self {
            profile_id: "default_accessible_local".to_string(),
            text_scale_permille: 1150,
            high_contrast: true,
            captions_enabled: true,
            reduced_motion: true,
            reduced_flash: true,
            master_gain_permille: 860,
            ui_gain_permille: 820,
            impact_gain_permille: 760,
            capability_gain_permille: 900,
            mute_master: false,
            hold_to_commit: false,
            toggle_guard: true,
            input_profile_id: "keyboard_mouse_gamepad_default".to_string(),
        }
    }

    fn deterministic_user_profile() -> Self {
        Self {
            profile_id: "local_player_persisted_smoke".to_string(),
            text_scale_permille: 1400,
            high_contrast: true,
            captions_enabled: true,
            reduced_motion: true,
            reduced_flash: true,
            master_gain_permille: 720,
            ui_gain_permille: 650,
            impact_gain_permille: 680,
            capability_gain_permille: 900,
            mute_master: false,
            hold_to_commit: true,
            toggle_guard: true,
            input_profile_id: "keyboard_mouse_gamepad_default".to_string(),
        }
    }

    fn validate(&self) -> Result<(), OathError> {
        if !(1000..=1600).contains(&self.text_scale_permille) {
            return Err(OathError::Verify(format!(
                "text_scale_permille out of range: {}",
                self.text_scale_permille
            )));
        }
        for (name, value) in [
            ("master_gain_permille", self.master_gain_permille),
            ("ui_gain_permille", self.ui_gain_permille),
            ("impact_gain_permille", self.impact_gain_permille),
            ("capability_gain_permille", self.capability_gain_permille),
        ] {
            if !(0..=1000).contains(&value) {
                return Err(OathError::Verify(format!("{name} out of range: {value}")));
            }
        }
        if self.input_profile_id != "keyboard_mouse_gamepad_default" {
            return Err(OathError::Verify(format!(
                "unknown input profile '{}'",
                self.input_profile_id
            )));
        }
        Ok(())
    }

    fn canonical_text(&self) -> String {
        let mut out = String::new();
        writeln!(&mut out, "schema={RUNTIME_SETTINGS_SCHEMA}").unwrap();
        writeln!(&mut out, "product={PRODUCT_NAME}").unwrap();
        writeln!(&mut out, "profile_id={}", self.profile_id).unwrap();
        writeln!(&mut out, "text_scale_permille={}", self.text_scale_permille).unwrap();
        writeln!(&mut out, "high_contrast={}", self.high_contrast).unwrap();
        writeln!(&mut out, "captions_enabled={}", self.captions_enabled).unwrap();
        writeln!(&mut out, "reduced_motion={}", self.reduced_motion).unwrap();
        writeln!(&mut out, "reduced_flash={}", self.reduced_flash).unwrap();
        writeln!(
            &mut out,
            "master_gain_permille={}",
            self.master_gain_permille
        )
        .unwrap();
        writeln!(&mut out, "ui_gain_permille={}", self.ui_gain_permille).unwrap();
        writeln!(
            &mut out,
            "impact_gain_permille={}",
            self.impact_gain_permille
        )
        .unwrap();
        writeln!(
            &mut out,
            "capability_gain_permille={}",
            self.capability_gain_permille
        )
        .unwrap();
        writeln!(&mut out, "mute_master={}", self.mute_master).unwrap();
        writeln!(&mut out, "hold_to_commit={}", self.hold_to_commit).unwrap();
        writeln!(&mut out, "toggle_guard={}", self.toggle_guard).unwrap();
        writeln!(&mut out, "input_profile_id={}", self.input_profile_id).unwrap();
        out
    }
}

pub fn write_runtime_settings_artifacts(out_dir: impl AsRef<Path>) -> Result<(), OathError> {
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;

    let default_profile = RuntimeSettings::default_profile();
    default_profile.validate()?;
    let saved_profile = RuntimeSettings::deterministic_user_profile();
    saved_profile.validate()?;

    let default_json = render_runtime_settings_json(&default_profile);
    let saved_json = render_runtime_settings_json(&saved_profile);
    let loaded_profile = parse_runtime_settings_json(&saved_json)?;
    loaded_profile.validate()?;
    let loaded_json = render_runtime_settings_json(&loaded_profile);
    let roundtrip_exact = saved_json == loaded_json && saved_profile == loaded_profile;

    if !roundtrip_exact {
        return Err(OathError::Verify(
            "runtime settings roundtrip was not byte-exact".to_string(),
        ));
    }

    fs::write(out_dir.join("runtime_settings.default.json"), default_json)?;
    fs::write(out_dir.join("runtime_settings.saved.json"), &saved_json)?;
    fs::write(out_dir.join("runtime_settings.loaded.json"), &loaded_json)?;
    fs::write(
        out_dir.join("runtime_settings_report.md"),
        render_runtime_settings_report(&saved_profile, &loaded_profile),
    )?;
    Ok(())
}

fn render_runtime_settings_json(settings: &RuntimeSettings) -> String {
    let hash = hash_hex(settings.canonical_text().as_bytes());
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", RUNTIME_SETTINGS_SCHEMA, true);
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    writeln!(&mut out, "  \"truth_hz\": {TRUTH_HZ},").unwrap();
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(&mut out, "  \"replay_hash_affects\": false,").unwrap();
    writeln!(&mut out, "  \"uses_wall_clock\": false,").unwrap();
    writeln!(&mut out, "  \"hidden_rng\": false,").unwrap();
    write_json_field(&mut out, 1, "profile_hash", &hash, true);
    write_json_field(&mut out, 1, "profile_id", &settings.profile_id, true);
    writeln!(
        &mut out,
        "  \"text_scale_permille\": {},",
        settings.text_scale_permille
    )
    .unwrap();
    writeln!(&mut out, "  \"high_contrast\": {},", settings.high_contrast).unwrap();
    writeln!(
        &mut out,
        "  \"captions_enabled\": {},",
        settings.captions_enabled
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"reduced_motion\": {},",
        settings.reduced_motion
    )
    .unwrap();
    writeln!(&mut out, "  \"reduced_flash\": {},", settings.reduced_flash).unwrap();
    writeln!(
        &mut out,
        "  \"master_gain_permille\": {},",
        settings.master_gain_permille
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"ui_gain_permille\": {},",
        settings.ui_gain_permille
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"impact_gain_permille\": {},",
        settings.impact_gain_permille
    )
    .unwrap();
    writeln!(
        &mut out,
        "  \"capability_gain_permille\": {},",
        settings.capability_gain_permille
    )
    .unwrap();
    writeln!(&mut out, "  \"mute_master\": {},", settings.mute_master).unwrap();
    writeln!(
        &mut out,
        "  \"hold_to_commit\": {},",
        settings.hold_to_commit
    )
    .unwrap();
    writeln!(&mut out, "  \"toggle_guard\": {},", settings.toggle_guard).unwrap();
    write_json_field(
        &mut out,
        1,
        "input_profile_id",
        &settings.input_profile_id,
        true,
    );
    writeln!(&mut out, "  \"public_demo_ready\": {PUBLIC_DEMO_READY},").unwrap();
    writeln!(
        &mut out,
        "  \"release_candidate_ready\": {RELEASE_CANDIDATE_READY}"
    )
    .unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

fn parse_runtime_settings_json(text: &str) -> Result<RuntimeSettings, OathError> {
    let schema = json_string_value(text, "schema")
        .ok_or_else(|| OathError::Verify("settings missing schema".to_string()))?;
    if schema != RUNTIME_SETTINGS_SCHEMA {
        return Err(OathError::Verify(format!(
            "settings schema mismatch: expected {RUNTIME_SETTINGS_SCHEMA}, got {schema}"
        )));
    }
    Ok(RuntimeSettings {
        profile_id: json_string_value(text, "profile_id")
            .ok_or_else(|| OathError::Verify("settings missing profile_id".to_string()))?,
        text_scale_permille: json_i32_value(text, "text_scale_permille")?,
        high_contrast: json_bool_value(text, "high_contrast")?,
        captions_enabled: json_bool_value(text, "captions_enabled")?,
        reduced_motion: json_bool_value(text, "reduced_motion")?,
        reduced_flash: json_bool_value(text, "reduced_flash")?,
        master_gain_permille: json_i32_value(text, "master_gain_permille")?,
        ui_gain_permille: json_i32_value(text, "ui_gain_permille")?,
        impact_gain_permille: json_i32_value(text, "impact_gain_permille")?,
        capability_gain_permille: json_i32_value(text, "capability_gain_permille")?,
        mute_master: json_bool_value(text, "mute_master")?,
        hold_to_commit: json_bool_value(text, "hold_to_commit")?,
        toggle_guard: json_bool_value(text, "toggle_guard")?,
        input_profile_id: json_string_value(text, "input_profile_id")
            .ok_or_else(|| OathError::Verify("settings missing input_profile_id".to_string()))?,
    })
}

fn render_runtime_settings_report(saved: &RuntimeSettings, loaded: &RuntimeSettings) -> String {
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD Runtime Settings Persistence Report").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "Status: PASSED").unwrap();
    writeln!(&mut out, "- Presentation only: `true`").unwrap();
    writeln!(&mut out, "- Truth mutation: `none`").unwrap();
    writeln!(&mut out, "- Replay hash affects: `false`").unwrap();
    writeln!(&mut out, "- Hidden RNG: `false`").unwrap();
    writeln!(&mut out, "- Wall clock: `false`").unwrap();
    writeln!(&mut out, "- Roundtrip byte exact: `true`").unwrap();
    writeln!(&mut out, "- Saved profile: `{}`", saved.profile_id).unwrap();
    writeln!(&mut out, "- Loaded profile: `{}`", loaded.profile_id).unwrap();
    writeln!(
        &mut out,
        "- Text scale persisted: `{}` permille",
        loaded.text_scale_permille
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Audio gains persisted: master `{}`, ui `{}`, impact `{}`, capability `{}` permille",
        loaded.master_gain_permille,
        loaded.ui_gain_permille,
        loaded.impact_gain_permille,
        loaded.capability_gain_permille
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Input options persisted: hold_to_commit `{}`, toggle_guard `{}`",
        loaded.hold_to_commit, loaded.toggle_guard
    )
    .unwrap();
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
        "Runtime settings persist presentation, input, accessibility, and audio preferences only. They do not alter authoritative 120 Hz truth, committed action sequences, contact packets, injury/capability state, replay hashes, content hashes, or asset hashes."
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

fn json_string_value(input: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":");
    let start = input.find(&needle)? + needle.len();
    let after_colon = input[start..].trim_start();
    if !after_colon.starts_with('"') {
        return None;
    }
    parse_json_string(after_colon).map(|(value, _)| value)
}

fn json_i32_value(input: &str, key: &str) -> Result<i32, OathError> {
    let value = json_scalar_value(input, key)
        .ok_or_else(|| OathError::Verify(format!("settings missing {key}")))?;
    value
        .parse::<i32>()
        .map_err(|_| OathError::Verify(format!("settings field {key} is not an integer")))
}

fn json_bool_value(input: &str, key: &str) -> Result<bool, OathError> {
    match json_scalar_value(input, key)
        .ok_or_else(|| OathError::Verify(format!("settings missing {key}")))?
        .as_str()
    {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(OathError::Verify(format!(
            "settings field {key} is not a boolean"
        ))),
    }
}

fn json_scalar_value(input: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\":");
    let start = input.find(&needle)? + needle.len();
    let rest = input[start..].trim_start();
    let end = rest
        .find(|ch: char| ch == ',' || ch == '\n' || ch == '}')
        .unwrap_or(rest.len());
    Some(rest[..end].trim().to_string())
}

fn parse_json_string(input: &str) -> Option<(String, usize)> {
    let bytes = input.as_bytes();
    if bytes.first().copied()? != b'"' {
        return None;
    }
    let mut out = String::new();
    let mut index = 1;
    while index < bytes.len() {
        match bytes[index] {
            b'"' => return Some((out, index + 1)),
            b'\\' => {
                index += 1;
                if index >= bytes.len() {
                    return None;
                }
                match bytes[index] {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    _ => return None,
                }
            }
            other => out.push(other as char),
        }
        index += 1;
    }
    None
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

fn hash_hex(bytes: &[u8]) -> String {
    format!("{:016x}", fnv1a64(bytes))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
