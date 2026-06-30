use std::env;
use std::path::PathBuf;

use oathyard::{
    native_combat_render, native_roster_showcase, render_goal_command_output, run_scenario_file,
    verify_replay_export_bundle, verify_replay_file, write_accessibility_artifacts,
    write_ai_duel_artifacts, write_ai_sweep_artifacts, write_animation_state_machine_artifacts,
    write_artifacts, write_audio_device_smoke_artifacts, write_audio_mixer_artifacts,
    write_audio_vfx_artifacts, write_contact_matrix_artifacts, write_gamepad_smoke_artifacts,
    write_input_artifacts, write_match_artifacts, write_negative_input_audit_artifacts,
    write_pbr_material_artifacts, write_performance_summary, write_presentation_bricks_artifacts,
    write_replay_export_bundle, write_runtime_settings_artifacts, write_truth_edge_audit_artifacts,
    write_truth_stress_artifacts, GoalArtifactSpec, OathError,
};

fn main() {
    if let Err(error) = real_main() {
        eprintln!("oathyard: {error}");
        std::process::exit(1);
    }
}

fn real_main() -> Result<(), OathError> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return launch_default_native_flow();
    };

    match command.as_str() {
        "run" => {
            let mut scenario: Option<PathBuf> = None;
            let mut out: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--scenario" => {
                        scenario = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--scenario requires a path".to_string())
                        })?));
                    }
                    "--out" => {
                        out = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--out requires a path".to_string())
                        })?));
                    }
                    other => {
                        return Err(OathError::Parse(format!("unknown run argument '{other}'")));
                    }
                }
            }
            let scenario =
                scenario.ok_or_else(|| OathError::Parse("--scenario is required".to_string()))?;
            let out = out.unwrap_or_else(|| PathBuf::from("artifacts/latest"));
            let result = run_scenario_file(&scenario)?;
            write_artifacts(&result, &out)?;
            println!("OATHYARD duel complete");
            println!("scenario={}", result.scenario_id);
            println!("out={}", out.display());
            println!("final_state_hash={}", result.final_state_hash);
            Ok(())
        }
        "replay" => {
            let mut replay: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--replay" => {
                        replay = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--replay requires a path".to_string())
                        })?));
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown replay argument '{other}'"
                        )));
                    }
                }
            }
            let replay =
                replay.ok_or_else(|| OathError::Parse("--replay is required".to_string()))?;
            let result = verify_replay_file(&replay)?;
            println!("OATHYARD replay verified");
            println!("scenario={}", result.scenario_id);
            println!("final_state_hash={}", result.final_state_hash);
            Ok(())
        }
        "export-bundle" => {
            let mut replay: Option<PathBuf> = None;
            let mut out: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--replay" => {
                        replay = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--replay requires a path".to_string())
                        })?));
                    }
                    "--out" => {
                        out = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--out requires a path".to_string())
                        })?));
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown export-bundle argument '{other}'"
                        )));
                    }
                }
            }
            let replay =
                replay.ok_or_else(|| OathError::Parse("--replay is required".to_string()))?;
            let out = out.unwrap_or_else(|| PathBuf::from("artifacts/export_bundle/latest"));
            let result = write_replay_export_bundle(&replay, &out)?;
            println!("OATHYARD replay export bundle written");
            println!("replay={}", replay.display());
            println!("out={}", out.display());
            println!("final_state_hash={}", result.final_state_hash);
            Ok(())
        }
        "verify-bundle" => {
            let mut bundle: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--bundle" => {
                        bundle = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--bundle requires a directory".to_string())
                        })?));
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown verify-bundle argument '{other}'"
                        )));
                    }
                }
            }
            let bundle =
                bundle.ok_or_else(|| OathError::Parse("--bundle is required".to_string()))?;
            let result = verify_replay_export_bundle(&bundle)?;
            println!("OATHYARD replay export bundle verified");
            println!("bundle={}", bundle.display());
            println!("scenario={}", result.scenario_id);
            println!("final_state_hash={}", result.final_state_hash);
            Ok(())
        }
        "match" => {
            let mut scenario: Option<PathBuf> = None;
            let mut out: Option<PathBuf> = None;
            let mut best_of = 5u32;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--scenario" => {
                        scenario = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--scenario requires a path".to_string())
                        })?));
                    }
                    "--out" => {
                        out = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--out requires a path".to_string())
                        })?));
                    }
                    "--best-of" => {
                        let value = args.next().ok_or_else(|| {
                            OathError::Parse("--best-of requires a number".to_string())
                        })?;
                        best_of = value.parse::<u32>().map_err(|_| {
                            OathError::Parse(format!("invalid --best-of value '{value}'"))
                        })?;
                        if best_of == 0 || best_of % 2 == 0 {
                            return Err(OathError::Parse(
                                "--best-of must be a positive odd number".to_string(),
                            ));
                        }
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown match argument '{other}'"
                        )));
                    }
                }
            }
            let scenario =
                scenario.ok_or_else(|| OathError::Parse("--scenario is required".to_string()))?;
            let out = out.unwrap_or_else(|| PathBuf::from("artifacts/match/latest"));
            write_match_artifacts(&scenario, &out, best_of)?;
            println!("OATHYARD match complete");
            println!("out={}", out.display());
            Ok(())
        }
        "perf-summary" => {
            let mut out: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--out" => {
                        out = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--out requires a path".to_string())
                        })?));
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown perf-summary argument '{other}'"
                        )));
                    }
                }
            }
            let out = out.unwrap_or_else(|| PathBuf::from("artifacts/perf/latest"));
            write_performance_summary(&out)?;
            println!("OATHYARD performance summary written");
            println!("out={}", out.display());
            Ok(())
        }
        "input-map" => {
            let mut out: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--out" => {
                        out = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--out requires a path".to_string())
                        })?));
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown input-map argument '{other}'"
                        )));
                    }
                }
            }
            let out = out.unwrap_or_else(|| PathBuf::from("artifacts/input/latest"));
            write_input_artifacts(&out)?;
            println!("OATHYARD input map written");
            println!("out={}", out.display());
            Ok(())
        }
        "gamepad-smoke" => {
            let mut out: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--out" => {
                        out = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--out requires a path".to_string())
                        })?));
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown gamepad-smoke argument '{other}'"
                        )));
                    }
                }
            }
            let out = out.unwrap_or_else(|| PathBuf::from("artifacts/gamepad/latest"));
            write_gamepad_smoke_artifacts(&out)?;
            println!("OATHYARD gamepad smoke written");
            println!("out={}", out.display());
            Ok(())
        }
        "accessibility" => {
            let mut out: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--out" => {
                        out = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--out requires a path".to_string())
                        })?));
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown accessibility argument '{other}'"
                        )));
                    }
                }
            }
            let out = out.unwrap_or_else(|| PathBuf::from("artifacts/accessibility/latest"));
            write_accessibility_artifacts(&out)?;
            println!("OATHYARD accessibility settings written");
            println!("out={}", out.display());
            Ok(())
        }
        "runtime-settings" => {
            let mut out: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--out" => {
                        out = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--out requires a path".to_string())
                        })?));
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown runtime-settings argument '{other}'"
                        )));
                    }
                }
            }
            let out = out.unwrap_or_else(|| PathBuf::from("artifacts/settings/latest"));
            write_runtime_settings_artifacts(&out)?;
            println!("OATHYARD runtime settings persisted");
            println!("out={}", out.display());
            Ok(())
        }
        "contact-matrix" => {
            let mut out: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--out" => {
                        out = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--out requires a path".to_string())
                        })?));
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown contact-matrix argument '{other}'"
                        )));
                    }
                }
            }
            let out = out.unwrap_or_else(|| PathBuf::from("artifacts/contact_matrix/latest"));
            write_contact_matrix_artifacts(&out)?;
            println!("OATHYARD contact matrix written");
            println!("out={}", out.display());
            Ok(())
        }
        "ai-duel" => {
            let mut out: Option<PathBuf> = None;
            let mut turns = 6u32;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--out" => {
                        out = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--out requires a path".to_string())
                        })?));
                    }
                    "--turns" => {
                        let value = args.next().ok_or_else(|| {
                            OathError::Parse("--turns requires a number".to_string())
                        })?;
                        turns = value.parse::<u32>().map_err(|_| {
                            OathError::Parse(format!("invalid --turns value '{value}'"))
                        })?;
                        if turns == 0 || turns > 12 {
                            return Err(OathError::Parse(
                                "--turns must be between 1 and 12".to_string(),
                            ));
                        }
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown ai-duel argument '{other}'"
                        )));
                    }
                }
            }
            let out = out.unwrap_or_else(|| PathBuf::from("artifacts/ai/latest"));
            let result = write_ai_duel_artifacts(&out, turns)?;
            println!("OATHYARD deterministic AI duel complete");
            println!("scenario={}", result.scenario_id);
            println!("out={}", out.display());
            println!("final_state_hash={}", result.final_state_hash);
            Ok(())
        }
        "ai-sweep" => {
            let mut out: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--out" => {
                        out = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--out requires a path".to_string())
                        })?));
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown ai-sweep argument '{other}'"
                        )));
                    }
                }
            }
            let out = out.unwrap_or_else(|| PathBuf::from("artifacts/ai_sweep/latest"));
            write_ai_sweep_artifacts(&out)?;
            println!("OATHYARD deterministic AI sweep complete");
            println!("out={}", out.display());
            Ok(())
        }
        "truth-stress" => {
            let mut out: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--out" => {
                        out = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--out requires a path".to_string())
                        })?));
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown truth-stress argument '{other}'"
                        )));
                    }
                }
            }
            let out = out.unwrap_or_else(|| PathBuf::from("artifacts/truth_stress/latest"));
            write_truth_stress_artifacts(&out)?;
            println!("OATHYARD truth stress complete");
            println!("out={}", out.display());
            Ok(())
        }
        "truth-edge-audit" => {
            let mut out: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--out" => {
                        out = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--out requires a path".to_string())
                        })?));
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown truth-edge-audit argument '{other}'"
                        )));
                    }
                }
            }
            let out = out.unwrap_or_else(|| PathBuf::from("artifacts/truth_edge/latest"));
            write_truth_edge_audit_artifacts(&out)?;
            println!("OATHYARD truth edge audit complete");
            println!("out={}", out.display());
            Ok(())
        }
        "negative-audit" => {
            let mut out: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--out" => {
                        out = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--out requires a path".to_string())
                        })?));
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown negative-audit argument '{other}'"
                        )));
                    }
                }
            }
            let out = out.unwrap_or_else(|| PathBuf::from("artifacts/negative_audit/latest"));
            write_negative_input_audit_artifacts(&out)?;
            println!("OATHYARD negative input audit complete");
            println!("out={}", out.display());
            Ok(())
        }
        "goal" | "/goal" => {
            let mut repo_root = PathBuf::from(".");
            let mut artifacts = Vec::new();
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--repo-root" => {
                        repo_root = PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--repo-root requires a path".to_string())
                        })?);
                    }
                    "--asset" => {
                        let asset_id = args.next().ok_or_else(|| {
                            OathError::Parse("--asset requires an asset id".to_string())
                        })?;
                        artifacts.push(GoalArtifactSpec::combat_truth_ai_assisted(asset_id));
                    }
                    "--non-ai-asset" => {
                        let asset_id = args.next().ok_or_else(|| {
                            OathError::Parse("--non-ai-asset requires an asset id".to_string())
                        })?;
                        artifacts.push(GoalArtifactSpec::combat_truth_deterministic_never_ai(
                            asset_id,
                        ));
                    }
                    other => {
                        return Err(OathError::Parse(format!("unknown goal argument '{other}'")));
                    }
                }
            }
            print!("{}", render_goal_command_output(&repo_root, &artifacts)?);
            Ok(())
        }
        "freeze" => {
            use oathyard::{
                create_freeze_registry_entry, CrossPlatformEvidence as XPlatEvidence,
                FreezePipelineConfig,
            };
            let mut repo_root = PathBuf::from(".");
            let mut asset_id: Option<String> = None;
            let mut authority_scope = "combat_truth".to_string();
            let mut artifact_path: Option<PathBuf> = None;
            let mut scenario_path: Option<PathBuf> = None;
            let mut xplat_platforms: Vec<String> = Vec::new();
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--repo-root" => {
                        repo_root = PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--repo-root requires a path".to_string())
                        })?);
                    }
                    "--asset-id" => {
                        asset_id = Some(args.next().ok_or_else(|| {
                            OathError::Parse("--asset-id requires an id".to_string())
                        })?);
                    }
                    "--authority-scope" => {
                        authority_scope = args.next().ok_or_else(|| {
                            OathError::Parse("--authority-scope requires a scope".to_string())
                        })?;
                    }
                    "--artifact" => {
                        artifact_path = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--artifact requires a path".to_string())
                        })?));
                    }
                    "--scenario" => {
                        scenario_path = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--scenario requires a path".to_string())
                        })?));
                    }
                    "--xplat-platform" => {
                        xplat_platforms.push(args.next().ok_or_else(|| {
                            OathError::Parse("--xplat-platform requires a name".to_string())
                        })?);
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown freeze argument '{other}'"
                        )));
                    }
                }
            }
            let asset_id =
                asset_id.ok_or_else(|| OathError::Parse("--asset-id is required".to_string()))?;
            let artifact_path = artifact_path
                .ok_or_else(|| OathError::Parse("--artifact is required".to_string()))?;
            let cross_platform_evidence = if xplat_platforms.is_empty() {
                None
            } else {
                Some(XPlatEvidence {
                    platforms: xplat_platforms,
                    all_match: true,
                })
            };
            let config = FreezePipelineConfig {
                authority_scope,
                asset_id: asset_id.clone(),
                artifact_path: artifact_path.clone(),
                scenario_path,
                cross_platform_evidence,
            };
            let output = create_freeze_registry_entry(&repo_root, &config)?;
            println!("OATHYARD freeze pipeline complete");
            println!("asset_id={}", output.asset_id);
            println!("authority_scope={}", output.authority_scope);
            println!("content_hash=sha256:{}", output.content_hash);
            println!("registry_entry={}", output.registry_entry_path.display());
            println!("overall_passed={}", output.overall_passed);
            for step in &output.steps {
                let status = if step.passed { "PASS" } else { "FAIL" };
                println!("  [{status}] {}: {}", step.step, step.detail);
            }
            Ok(())
        }
        "native-roster-showcase" => {
            let mut out: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--out" => {
                        out = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--out requires a path".to_string())
                        })?));
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown native-roster-showcase argument '{other}'"
                        )));
                    }
                }
            }
            let out = out.unwrap_or_else(|| PathBuf::from("artifacts/native_roster/latest"));
            native_roster_showcase(&out)?;
            println!("OATHYARD native roster showcase passed");
            println!("out={}", out.display());
            Ok(())
        }
        "native-combat-render" => {
            let mut scenario: Option<PathBuf> = None;
            let mut out: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--scenario" => {
                        scenario = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--scenario requires a path".to_string())
                        })?));
                    }
                    "--out" => {
                        out = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--out requires a path".to_string())
                        })?));
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown native-combat-render argument '{other}'"
                        )));
                    }
                }
            }
            let scenario =
                scenario.ok_or_else(|| OathError::Parse("--scenario is required".to_string()))?;
            let out = out.unwrap_or_else(|| PathBuf::from("artifacts/native_combat/latest"));
            let result = native_combat_render(&scenario, &out)?;
            println!("OATHYARD native combat render passed");
            println!("scenario={}", result.scenario_id);
            println!("out={}", out.display());
            println!("final_state_hash={}", result.final_state_hash);
            Ok(())
        }
        "pbr-materials" => {
            let mut scenario: Option<PathBuf> = None;
            let mut out: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--scenario" => {
                        scenario = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--scenario requires a path".to_string())
                        })?));
                    }
                    "--out" => {
                        out = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--out requires a path".to_string())
                        })?));
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown pbr-materials argument '{other}'"
                        )));
                    }
                }
            }
            let scenario =
                scenario.ok_or_else(|| OathError::Parse("--scenario is required".to_string()))?;
            let out = out.unwrap_or_else(|| PathBuf::from("artifacts/pbr_materials/latest"));
            let result = write_pbr_material_artifacts(&scenario, &out)?;
            println!("OATHYARD PBR material artifacts passed");
            println!("scenario={}", result.scenario_id);
            println!("out={}", out.display());
            println!("final_state_hash={}", result.final_state_hash);
            Ok(())
        }
        "audio-vfx-render" => {
            let mut scenario: Option<PathBuf> = None;
            let mut out: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--scenario" => {
                        scenario = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--scenario requires a path".to_string())
                        })?));
                    }
                    "--out" => {
                        out = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--out requires a path".to_string())
                        })?));
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown audio-vfx-render argument '{other}'"
                        )));
                    }
                }
            }
            let scenario =
                scenario.ok_or_else(|| OathError::Parse("--scenario is required".to_string()))?;
            let out = out.unwrap_or_else(|| PathBuf::from("artifacts/audio_vfx/latest"));
            let result = write_audio_vfx_artifacts(&scenario, &out)?;
            println!("OATHYARD audio/VFX render passed");
            println!("scenario={}", result.scenario_id);
            println!("out={}", out.display());
            println!("final_state_hash={}", result.final_state_hash);
            Ok(())
        }
        "audio-mixer" => {
            let mut scenario: Option<PathBuf> = None;
            let mut out: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--scenario" => {
                        scenario = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--scenario requires a path".to_string())
                        })?));
                    }
                    "--out" => {
                        out = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--out requires a path".to_string())
                        })?));
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown audio-mixer argument '{other}'"
                        )));
                    }
                }
            }
            let scenario =
                scenario.ok_or_else(|| OathError::Parse("--scenario is required".to_string()))?;
            let out = out.unwrap_or_else(|| PathBuf::from("artifacts/audio_mixer/latest"));
            let result = write_audio_mixer_artifacts(&scenario, &out)?;
            println!("OATHYARD runtime audio mixer passed");
            println!("scenario={}", result.scenario_id);
            println!("out={}", out.display());
            println!("final_state_hash={}", result.final_state_hash);
            Ok(())
        }
        "audio-device-smoke" => {
            let mut scenario: Option<PathBuf> = None;
            let mut out: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--scenario" => {
                        scenario = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--scenario requires a path".to_string())
                        })?));
                    }
                    "--out" => {
                        out = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--out requires a path".to_string())
                        })?));
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown audio-device-smoke argument '{other}'"
                        )));
                    }
                }
            }
            let scenario =
                scenario.ok_or_else(|| OathError::Parse("--scenario is required".to_string()))?;
            let out = out.unwrap_or_else(|| PathBuf::from("artifacts/audio_device/latest"));
            let result = write_audio_device_smoke_artifacts(&scenario, &out)?;
            println!("OATHYARD audio device smoke passed");
            println!("scenario={}", result.scenario_id);
            println!("out={}", out.display());
            println!("final_state_hash={}", result.final_state_hash);
            Ok(())
        }
        "animation-state-machine" => {
            let mut scenario: Option<PathBuf> = None;
            let mut out: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--scenario" => {
                        scenario = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--scenario requires a path".to_string())
                        })?));
                    }
                    "--out" => {
                        out = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--out requires a path".to_string())
                        })?));
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown animation-state-machine argument '{other}'"
                        )));
                    }
                }
            }
            let scenario =
                scenario.ok_or_else(|| OathError::Parse("--scenario is required".to_string()))?;
            let out =
                out.unwrap_or_else(|| PathBuf::from("artifacts/animation_state_machine/latest"));
            let result = write_animation_state_machine_artifacts(&scenario, &out)?;
            println!("OATHYARD animation state machine passed");
            println!("scenario={}", result.scenario_id);
            println!("out={}", out.display());
            println!("final_state_hash={}", result.final_state_hash);
            Ok(())
        }
        "presentation-bricks" => {
            let mut scenario: Option<PathBuf> = None;
            let mut out: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--scenario" => {
                        scenario = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--scenario requires a path".to_string())
                        })?));
                    }
                    "--out" => {
                        out = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--out requires a path".to_string())
                        })?));
                    }
                    other => {
                        return Err(OathError::Parse(format!(
                            "unknown presentation-bricks argument '{other}'"
                        )));
                    }
                }
            }
            let scenario =
                scenario.ok_or_else(|| OathError::Parse("--scenario is required".to_string()))?;
            let out = out.unwrap_or_else(|| PathBuf::from("artifacts/presentation_bricks/latest"));
            let result = write_presentation_bricks_artifacts(&scenario, &out)?;
            println!("OATHYARD PresentationBricks passed");
            println!("scenario={}", result.scenario_id);
            println!("out={}", out.display());
            println!("final_state_hash={}", result.final_state_hash);
            Ok(())
        }
        "--help" | "-h" | "help" => {
            println!("{}", usage());
            Ok(())
        }
        other => Err(OathError::Parse(format!(
            "unknown command '{other}'\n{}",
            usage()
        ))),
    }
}

fn launch_default_native_flow() -> Result<(), OathError> {
    let out = env::var("OATHYARD_LAUNCH_OUT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("artifacts/native_combat/launch"));
    let scenario = env::var("OATHYARD_LAUNCH_SCENARIO")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("examples/duels/basic_oathyard.duel"));
    let result = native_combat_render(&scenario, &out)?;
    println!("OATHYARD native 3D combat render complete");
    println!("scenario={}", result.scenario_id);
    println!("out={}", out.display());
    println!("final_state_hash={}", result.final_state_hash);
    Ok(())
}

fn usage() -> &'static str {
    "usage:
  oathyard [no args runs native 3D combat render]
  oathyard run --scenario <path> --out <dir>
  oathyard replay --replay <path>
  oathyard export-bundle --replay <path> --out <dir>
  oathyard verify-bundle --bundle <dir>
  oathyard match --scenario <path> --out <dir> --best-of <odd>
  oathyard perf-summary --out <dir>
  oathyard input-map --out <dir>
  oathyard gamepad-smoke --out <dir>
  oathyard accessibility --out <dir>
  oathyard runtime-settings --out <dir>
  oathyard contact-matrix --out <dir>
  oathyard ai-duel --out <dir> --turns <1-12>
  oathyard ai-sweep --out <dir>
  oathyard truth-stress --out <dir>
  oathyard truth-edge-audit --out <dir>
  oathyard negative-audit --out <dir>
  oathyard goal --repo-root <dir> --asset <asset_id> [--asset <asset_id>...] [--non-ai-asset <asset_id>...]
  oathyard freeze --repo-root <dir> --asset-id <id> --artifact <path> [--authority-scope <scope>] [--scenario <path>] [--xplat-platform <name>...]
  oathyard native-roster-showcase --out <dir>
  oathyard native-combat-render --scenario <path> --out <dir>
  oathyard pbr-materials --scenario <path> --out <dir>
  oathyard audio-vfx-render --scenario <path> --out <dir>
  oathyard audio-mixer --scenario <path> --out <dir>
  oathyard audio-device-smoke --scenario <path> --out <dir>
  oathyard animation-state-machine --scenario <path> --out <dir>
  oathyard presentation-bricks --scenario <path> --out <dir>

launch env:
  OATHYARD_LAUNCH_OUT=<dir> selects no-args artifact path
  OATHYARD_LAUNCH_SCENARIO=<path> selects no-args 3D combat scenario"
}
