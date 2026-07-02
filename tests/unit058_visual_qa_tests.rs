// Unit-058: Automated Visual QA regression tests.
//
// Tests the visual QA system: baseline schema validation, fail on missing
// capture, fail on low resolution, fail on truth mutation, report-only mode,
// report completeness, readiness flags, and truth isolation.

use std::fs;
use std::path::Path;
use std::process::Command;

const VISUAL_QA_SCRIPT: &str = "./tools/visual_qa.sh";

fn run_visual_qa(out_dir: &str, extra_args: &[&str]) -> (bool, String, String) {
    let result = Command::new("bash")
        .arg(VISUAL_QA_SCRIPT)
        .arg(out_dir)
        .args(extra_args)
        .output();
    match result {
        Ok(output) => (
            output.status.success(),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ),
        Err(e) => (false, String::new(), format!("failed to run: {e}")),
    }
}

fn make_test_png(path: &Path, width: u32, height: u32) {
    // Generate a valid PNG file using Python (guaranteed available).
    let code = format!(
        r#"
from PIL import Image
img = Image.new("RGB", ({w}, {h}))
for y in range({h}):
    for x in range({w}):
        r = (x * 255) // max({w}, 1)
        g = (x % 100) * 2
        b = (x * 50) // max({w}, 1)
        img.putpixel((x, y), (r, g, b))
img.save("{p}")
"#,
        w = width,
        h = height,
        p = path.display()
    );
    Command::new("python3")
        .arg("-c")
        .arg(&code)
        .status()
        .expect("python3 failed to create test PNG");
}

fn json_parse(text: &str) -> std::collections::HashMap<String, String> {
    // Simple key-value extraction for basic JSON fields.
    let mut map = std::collections::HashMap::new();
    for line in text.lines() {
        let trimmed = line.trim().trim_end_matches(',');
        if let Some(colon_pos) = trimmed.find(':') {
            let key = trimmed[..colon_pos].trim().trim_matches('"').to_string();
            let value = trimmed[colon_pos + 1..]
                .trim()
                .trim_matches('"')
                .to_string();
            if !key.is_empty() {
                map.insert(key, value);
            }
        }
    }
    map
}

#[test]
fn unit058_visual_qa_baseline_manifest_schema_validates() {
    let baseline_path = Path::new("content/visual_qa/working_game_baseline.json");
    assert!(baseline_path.exists(), "baseline manifest missing");
    let text = fs::read_to_string(baseline_path).unwrap();

    // Validate schema via python3 json module
    let result = Command::new("python3")
        .arg("-c")
        .arg(format!(
            r#"
import json, sys
d = json.load(open("{}"))
assert d["schema"] == "oathyard.visual_qa_baseline.v1", f"schema: {{d['schema']}}"
assert d["candidate_baseline_pending_review"] == True, "baseline must be pending review"
assert d["owner_visual_acceptance"] == False, "owner_visual_acceptance must be false"
assert d["public_demo_ready"] == False
assert d["release_candidate_ready"] == False
assert d["truth_mutation"] == False
print("baseline schema OK")
"#,
            baseline_path.display()
        ))
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "baseline schema validation failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn unit058_visual_qa_fails_on_missing_capture() {
    let tmp = Path::new("target/tmp/unit058_missing_capture");
    let _ = fs::remove_dir_all(tmp);
    fs::create_dir_all(tmp).unwrap();

    let current_dir = tmp.join("current");
    fs::create_dir_all(&current_dir).unwrap();

    let (passed, stdout, _stderr) = run_visual_qa(
        &tmp.join("qa_out").to_string_lossy(),
        &["--current", &current_dir.to_string_lossy()],
    );

    assert!(!passed, "visual QA should fail when captures are missing");
    assert!(
        stdout.contains("FAILED") || _stderr.contains("FAILED"),
        "should contain FAILED in output"
    );

    let _ = fs::remove_dir_all(tmp);
}

#[test]
fn unit058_visual_qa_fails_on_low_resolution() {
    let tmp = Path::new("target/tmp/unit058_low_res");
    let _ = fs::remove_dir_all(tmp);
    fs::create_dir_all(tmp).unwrap();

    let current_dir = tmp.join("current");
    fs::create_dir_all(&current_dir).unwrap();
    let png_path = current_dir.join("production_renderer_native_low.png");
    make_test_png(&png_path, 100, 100);

    let (passed, _stdout, stderr) = run_visual_qa(
        &tmp.join("qa_out").to_string_lossy(),
        &["--current", &current_dir.to_string_lossy()],
    );

    assert!(!passed, "visual QA should fail on low resolution");
    assert!(
        stderr.contains("resolution") || stderr.contains("FAILED"),
        "should report resolution failure: {}",
        stderr
    );

    let _ = fs::remove_dir_all(tmp);
}

#[test]
fn unit058_visual_qa_fails_on_truth_mutation_true() {
    let tmp = Path::new("target/tmp/unit058_truth_mut");
    let _ = fs::remove_dir_all(tmp);
    fs::create_dir_all(tmp).unwrap();

    let current_dir = tmp.join("current");
    fs::create_dir_all(&current_dir).unwrap();

    let png_path = current_dir.join("observe.png");
    make_test_png(&png_path, 1920, 1080);

    let captures_json = r#"{
        "captures": [{
            "capture_id": "observe",
            "file": "observe.png",
            "truth_mutation": true
        }]
    }"#;
    fs::write(current_dir.join("captures.json"), captures_json).unwrap();

    let (passed, _stdout, stderr) = run_visual_qa(
        &tmp.join("qa_out").to_string_lossy(),
        &["--current", &current_dir.to_string_lossy()],
    );

    assert!(!passed, "visual QA should fail on truth_mutation=true");
    assert!(
        stderr.contains("truth_mutation") || stderr.contains("FAILED"),
        "should report truth_mutation failure: {}",
        stderr
    );

    let _ = fs::remove_dir_all(tmp);
}

#[test]
fn unit058_visual_qa_report_only_mode_exits_zero() {
    let tmp = Path::new("target/tmp/unit058_report_only");
    let _ = fs::remove_dir_all(tmp);
    fs::create_dir_all(tmp).unwrap();

    let current_dir = tmp.join("current");
    fs::create_dir_all(&current_dir).unwrap();

    let (passed, stdout, _stderr) = run_visual_qa(
        &tmp.join("qa_out").to_string_lossy(),
        &["--current", &current_dir.to_string_lossy(), "--report-only"],
    );

    assert!(passed, "report-only mode should exit 0 even with failures");
    assert!(
        stdout.contains("report-only"),
        "should indicate report-only mode"
    );

    let _ = fs::remove_dir_all(tmp);
}

#[test]
fn unit058_visual_qa_report_includes_all_required_roles() {
    let tmp = Path::new("target/tmp/unit058_all_roles");
    let _ = fs::remove_dir_all(tmp);
    fs::create_dir_all(tmp).unwrap();

    let current_dir = tmp.join("current");
    fs::create_dir_all(&current_dir).unwrap();

    run_visual_qa(
        &tmp.join("qa_out").to_string_lossy(),
        &["--current", &current_dir.to_string_lossy(), "--report-only"],
    );

    let report_path = tmp.join("qa_out").join("visual_qa_report.json");

    // Use python3 to validate all roles present
    let result = Command::new("python3")
        .arg("-c")
        .arg(format!(
            r#"
import json
d = json.load(open("{}"))
required = [
    "boot_main_menu", "mode_select", "fighter_select", "loadout_select",
    "arena_select", "observe", "plan", "commit_reveal", "resolve_contact",
    "consequence_cause_chain", "replan", "match_result", "replay_view",
    "fight_film_view", "settings", "quit_or_return_to_menu",
]
reported = [r["role"] for r in d["role_results"]]
for role in required:
    assert role in reported, f"missing role: {{role}}"
print("all roles present")
"#,
            report_path.display()
        ))
        .output()
        .unwrap();

    assert!(
        result.status.success(),
        "not all required roles present: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    let _ = fs::remove_dir_all(tmp);
}

#[test]
fn unit058_visual_qa_report_keeps_readiness_flags_false() {
    let tmp = Path::new("target/tmp/unit058_flags");
    let _ = fs::remove_dir_all(tmp);
    fs::create_dir_all(tmp).unwrap();

    let current_dir = tmp.join("current");
    fs::create_dir_all(&current_dir).unwrap();

    run_visual_qa(
        &tmp.join("qa_out").to_string_lossy(),
        &["--current", &current_dir.to_string_lossy(), "--report-only"],
    );

    let report_path = tmp.join("qa_out").join("visual_qa_report.json");

    let result = Command::new("python3")
        .arg("-c")
        .arg(format!(
            r#"
import json
d = json.load(open("{}"))
assert d["owner_visual_acceptance"] == False, "owner_visual_acceptance must be false"
assert d["public_demo_ready"] == False, "public_demo_ready must be false"
assert d["release_candidate_ready"] == False, "release_candidate_ready must be false"
assert d["truth_mutation"] == False, "truth_mutation must be false"
print("readiness flags OK")
"#,
            report_path.display()
        ))
        .output()
        .unwrap();

    assert!(
        result.status.success(),
        "readiness flags not false: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    let _ = fs::remove_dir_all(tmp);
}

#[test]
fn unit058_visual_qa_does_not_mutate_truth() {
    let tmp = Path::new("target/tmp/unit058_no_truth_mut");
    let _ = fs::remove_dir_all(tmp);
    fs::create_dir_all(tmp).unwrap();

    let before_result = Command::new("./target/debug/oathyard")
        .args([
            "run",
            "--scenario",
            "examples/duels/basic_oathyard.duel",
            "--out",
        ])
        .arg(tmp.join("truth_before"))
        .output()
        .unwrap();
    assert!(before_result.status.success());

    let before_hash = fs::read_to_string(tmp.join("truth_before/final_state_hash.txt"))
        .unwrap()
        .trim()
        .to_string();

    run_visual_qa(&tmp.join("qa_out").to_string_lossy(), &["--report-only"]);

    let after_result = Command::new("./target/debug/oathyard")
        .args([
            "run",
            "--scenario",
            "examples/duels/basic_oathyard.duel",
            "--out",
        ])
        .arg(tmp.join("truth_after"))
        .output()
        .unwrap();
    assert!(after_result.status.success());

    let after_hash = fs::read_to_string(tmp.join("truth_after/final_state_hash.txt"))
        .unwrap()
        .trim()
        .to_string();

    assert_eq!(
        before_hash, after_hash,
        "canonical truth hash must be unchanged after visual QA"
    );
    assert_eq!(
        before_hash, "f17c8f76b9dfae86",
        "canonical truth hash must match expected value"
    );

    let _ = fs::remove_dir_all(tmp);
}
