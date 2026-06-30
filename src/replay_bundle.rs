use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::{
    comma, file_hash_hex, hash_hex, json_string_value, verify_replay_file, write_artifacts,
    write_json_field, DuelResult, OathError, PRODUCT_NAME, PUBLIC_DEMO_READY,
    RELEASE_CANDIDATE_READY, REPLAY_EXPORT_BUNDLE_SCHEMA, TRUTH_HZ,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExportBundleFile {
    path: String,
    role: &'static str,
    byte_len: u64,
    canonical_hash: String,
}

pub fn write_replay_export_bundle(
    replay_path: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
) -> Result<DuelResult, OathError> {
    let replay_path = replay_path.as_ref();
    let result = verify_replay_file(replay_path)?;
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)?;

    write_artifacts(&result, out_dir)?;
    let bundle_files = replay_export_bundle_source_files(out_dir)?;
    fs::write(
        out_dir.join("export_bundle_manifest.json"),
        render_replay_export_bundle_manifest(&result, &bundle_files),
    )?;
    fs::write(
        out_dir.join("export_bundle_report.md"),
        render_replay_export_bundle_report(&result, &bundle_files),
    )?;

    let mut hash_files = bundle_files;
    hash_files.push(export_bundle_file_entry(
        out_dir,
        "export_bundle_manifest.json",
        "bundle_manifest",
    )?);
    hash_files.push(export_bundle_file_entry(
        out_dir,
        "export_bundle_report.md",
        "bundle_report",
    )?);
    fs::write(
        out_dir.join("bundle_hashes.txt"),
        render_replay_export_bundle_hashes(&hash_files),
    )?;

    Ok(result)
}

pub fn verify_replay_export_bundle(bundle_dir: impl AsRef<Path>) -> Result<DuelResult, OathError> {
    let bundle_dir = bundle_dir.as_ref();
    let replay_path = bundle_dir.join("replay.json");
    let result = verify_replay_file(&replay_path)?;
    let manifest = fs::read_to_string(bundle_dir.join("export_bundle_manifest.json"))?;
    let schema = json_string_value(&manifest, "schema")
        .ok_or_else(|| OathError::Verify("export bundle manifest missing schema".to_string()))?;
    if schema != REPLAY_EXPORT_BUNDLE_SCHEMA {
        return Err(OathError::Verify(format!(
            "export bundle schema mismatch: expected {REPLAY_EXPORT_BUNDLE_SCHEMA}, got {schema}"
        )));
    }
    let manifest_final_hash =
        json_string_value(&manifest, "final_state_hash").ok_or_else(|| {
            OathError::Verify("export bundle manifest missing final_state_hash".to_string())
        })?;
    if manifest_final_hash != result.final_state_hash {
        return Err(OathError::Verify(format!(
            "export bundle final hash mismatch: expected {}, got {}",
            result.final_state_hash, manifest_final_hash
        )));
    }
    if !manifest.contains("\"replay_verified\": true")
        || !manifest.contains("\"presentation_only\": true")
        || !manifest.contains("\"truth_mutation\": false")
    {
        return Err(OathError::Verify(
            "export bundle manifest missing truth-boundary flags".to_string(),
        ));
    }

    let expected_final_hash = fs::read_to_string(bundle_dir.join("final_state_hash.txt"))?;
    if expected_final_hash.trim() != result.final_state_hash {
        return Err(OathError::Verify(format!(
            "export bundle final_state_hash.txt mismatch: expected {}, got {}",
            result.final_state_hash,
            expected_final_hash.trim()
        )));
    }

    let required = [
        "trace.json",
        "replay.json",
        "final_state_hash.txt",
        "duel_report.md",
        "fight_film_manifest.json",
        "export_bundle_manifest.json",
        "export_bundle_report.md",
    ];
    for rel in required {
        if !bundle_dir.join(rel).is_file() {
            return Err(OathError::Verify(format!(
                "export bundle missing required file {rel}"
            )));
        }
    }

    let hash_text = fs::read_to_string(bundle_dir.join("bundle_hashes.txt"))?;
    let mut seen = Vec::new();
    for (line_index, line) in hash_text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let (expected_hash, rel_path) = line.split_once("  ").ok_or_else(|| {
            OathError::Verify(format!(
                "export bundle hash line {} is malformed",
                line_index + 1
            ))
        })?;
        if rel_path.starts_with('/') || rel_path.contains("..") {
            return Err(OathError::Verify(format!(
                "export bundle hash path is not relative/safe: {rel_path}"
            )));
        }
        let actual_hash = file_hash_hex(bundle_dir.join(rel_path))?;
        if actual_hash != expected_hash {
            return Err(OathError::Verify(format!(
                "export bundle hash mismatch for {rel_path}: expected {expected_hash}, got {actual_hash}"
            )));
        }
        seen.push(rel_path.to_string());
    }
    for rel in required {
        if !seen.iter().any(|path| path == rel) {
            return Err(OathError::Verify(format!(
                "export bundle hash manifest missing required file {rel}"
            )));
        }
    }

    Ok(result)
}

fn replay_export_bundle_source_files(out_dir: &Path) -> Result<Vec<ExportBundleFile>, OathError> {
    let mut files = Vec::new();
    for (path, role) in [
        ("trace.json", "truth_trace"),
        ("replay.json", "authoritative_replay"),
        ("final_state_hash.txt", "final_state_hash"),
        ("duel_report.md", "human_duel_report"),
        ("fight_film_manifest.json", "trace_highlight_manifest"),
    ] {
        files.push(export_bundle_file_entry(out_dir, path, role)?);
    }

    Ok(files)
}

fn export_bundle_file_entry(
    out_dir: &Path,
    relative_path: &str,
    role: &'static str,
) -> Result<ExportBundleFile, OathError> {
    let path = out_dir.join(relative_path);
    let bytes = fs::read(&path)?;
    Ok(ExportBundleFile {
        path: relative_path.to_string(),
        role,
        byte_len: bytes.len() as u64,
        canonical_hash: hash_hex(&bytes),
    })
}

fn render_replay_export_bundle_manifest(result: &DuelResult, files: &[ExportBundleFile]) -> String {
    let mut out = String::new();
    writeln!(&mut out, "{{").unwrap();
    write_json_field(&mut out, 1, "schema", REPLAY_EXPORT_BUNDLE_SCHEMA, true);
    write_json_field(&mut out, 1, "product", PRODUCT_NAME, true);
    write_json_field(&mut out, 1, "scenario_id", &result.scenario_id, true);
    write_json_field(&mut out, 1, "source", "verified-replay-export", true);
    write_json_field(&mut out, 1, "content_hash", &result.content_hash, true);
    write_json_field(
        &mut out,
        1,
        "initial_state_hash",
        &result.initial_state_hash,
        true,
    );
    write_json_field(
        &mut out,
        1,
        "final_state_hash",
        &result.final_state_hash,
        true,
    );
    writeln!(&mut out, "  \"truth_hz\": {TRUTH_HZ},").unwrap();
    writeln!(&mut out, "  \"replay_verified\": true,").unwrap();
    writeln!(&mut out, "  \"presentation_only\": true,").unwrap();
    writeln!(&mut out, "  \"truth_mutation\": false,").unwrap();
    writeln!(&mut out, "  \"hash_manifest\": \"bundle_hashes.txt\",").unwrap();
    writeln!(&mut out, "  \"public_demo_ready\": {PUBLIC_DEMO_READY},").unwrap();
    writeln!(
        &mut out,
        "  \"release_candidate_ready\": {RELEASE_CANDIDATE_READY},"
    )
    .unwrap();
    writeln!(&mut out, "  \"files\": [").unwrap();
    for (index, file) in files.iter().enumerate() {
        writeln!(&mut out, "    {{").unwrap();
        write_json_field(&mut out, 3, "path", &file.path, true);
        write_json_field(&mut out, 3, "role", file.role, true);
        writeln!(&mut out, "      \"byte_len\": {},", file.byte_len).unwrap();
        write_json_field(&mut out, 3, "canonical_hash", &file.canonical_hash, false);
        writeln!(&mut out, "    }}{}", comma(index + 1, files.len())).unwrap();
    }
    writeln!(&mut out, "  ]").unwrap();
    writeln!(&mut out, "}}").unwrap();
    out
}

fn render_replay_export_bundle_report(result: &DuelResult, files: &[ExportBundleFile]) -> String {
    let total_bytes: u64 = files.iter().map(|file| file.byte_len).sum();
    let mut out = String::new();
    writeln!(&mut out, "# OATHYARD Replay Export Bundle").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "Status: PASSED").unwrap();
    writeln!(&mut out, "- Scenario: `{}`", result.scenario_id).unwrap();
    writeln!(&mut out, "- Content hash: `{}`", result.content_hash).unwrap();
    writeln!(
        &mut out,
        "- Initial state hash: `{}`",
        result.initial_state_hash
    )
    .unwrap();
    writeln!(
        &mut out,
        "- Final state hash: `{}`",
        result.final_state_hash
    )
    .unwrap();
    writeln!(&mut out, "- Replay verified: `true`").unwrap();
    writeln!(&mut out, "- Source: `verified-replay-export`").unwrap();
    writeln!(&mut out, "- Presentation only: `true`").unwrap();
    writeln!(&mut out, "- Truth mutation: `none`").unwrap();
    writeln!(&mut out, "- File count: `{}`", files.len()).unwrap();
    writeln!(&mut out, "- Bundle source bytes: `{total_bytes}`").unwrap();
    writeln!(&mut out, "- Hash manifest: `bundle_hashes.txt`").unwrap();
    writeln!(&mut out, "- Public demo ready: `{PUBLIC_DEMO_READY}`").unwrap();
    writeln!(
        &mut out,
        "- Release candidate ready: `{RELEASE_CANDIDATE_READY}`"
    )
    .unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "## Files").unwrap();
    for file in files {
        writeln!(
            &mut out,
            "- `{}` role `{}` hash `{}` bytes `{}`",
            file.path, file.role, file.canonical_hash, file.byte_len
        )
        .unwrap();
    }
    out
}

fn render_replay_export_bundle_hashes(files: &[ExportBundleFile]) -> String {
    let mut out = String::new();
    for file in files {
        writeln!(&mut out, "{}  {}", file.canonical_hash, file.path).unwrap();
    }
    out
}
