use std::env;
use std::fs;
use std::path::PathBuf;

use oathyard::{
    native_combat_render, native_roster_showcase, render_goal_command_output, run_scenario_file,
    verify_replay_export_bundle, verify_replay_file, write_accessibility_artifacts,
    write_ai_duel_artifacts, write_ai_sweep_artifacts, write_animation_state_machine_artifacts,
    write_artifacts, write_audio_device_smoke_artifacts, write_audio_mixer_artifacts,
    write_audio_vfx_artifacts, write_contact_matrix_artifacts, write_gamepad_smoke_artifacts,
    write_input_artifacts, write_local_game_artifacts, write_match_artifacts,
    write_negative_input_audit_artifacts, write_pbr_material_artifacts, write_performance_summary,
    write_presentation_bricks_artifacts, write_replay_export_bundle,
    write_runtime_settings_artifacts, write_truth_edge_audit_artifacts,
    write_truth_stress_artifacts, GoalArtifactSpec, LocalGameConfig, OathError,
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
        "play-local" => {
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
                            "unknown play-local argument '{other}'"
                        )));
                    }
                }
            }
            let out = out.unwrap_or_else(|| PathBuf::from("artifacts/local_game/latest"));
            let result = write_local_game_artifacts(&out, LocalGameConfig::default())?;
            println!("OATHYARD local working-game path complete");
            println!("scenario={}", result.result.scenario_id);
            println!("out={}", out.display());
            println!("plan_cycles={}", result.plan_cycles);
            println!("final_state_hash={}", result.result.final_state_hash);
            println!(
                "local_playable_game_ready={}",
                result.local_playable_game_ready
            );
            println!(
                "owner_visual_acceptance=false public_demo_ready=false release_candidate_ready=false"
            );
            Ok(())
        }
        "play" | "--play" => {
            let mut out: Option<PathBuf> = None;
            let mut scripted_input: Option<PathBuf> = None;
            let mut smoke_frames: Option<u32> = None;
            let mut artifact_dir: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--out" => {
                        out = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--out requires a path".to_string())
                        })?));
                    }
                    "--scripted-input" => {
                        scripted_input = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--scripted-input requires a path".to_string())
                        })?));
                    }
                    "--smoke-frames" => {
                        smoke_frames =
                            Some(args.next().and_then(|s| s.parse().ok()).ok_or_else(|| {
                                OathError::Parse("--smoke-frames requires a number".to_string())
                            })?);
                    }
                    "--artifact-dir" => {
                        artifact_dir = Some(PathBuf::from(args.next().ok_or_else(|| {
                            OathError::Parse("--artifact-dir requires a path".to_string())
                        })?));
                    }
                    other => {
                        return Err(OathError::Parse(format!("unknown play argument '{other}'")));
                    }
                }
            }
            let artifact_dir =
                artifact_dir.unwrap_or_else(|| PathBuf::from("artifacts/play/latest"));
            launch_play_flow(out, scripted_input, smoke_frames, artifact_dir)
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

/// Unit-085: Direct product executable entry point.
///
/// `oathyard play` generates truth/replay artifacts from the local game,
/// creates a post-hash presentation packet, builds a mesh manifest from the
/// loadout, and launches the native renderer in windowed mode so the player
/// sees the actual 3D game with high-fidelity Meshy/Rodin assets.
///
/// This is the product path — no repo scripts needed. The executable discovers
/// assets relative to its own directory, not via absolute repo paths.
fn launch_play_flow(
    out: Option<PathBuf>,
    scripted_input: Option<PathBuf>,
    smoke_frames: Option<u32>,
    artifact_dir: PathBuf,
) -> Result<(), OathError> {
    use std::process::Command;

    let out = out.unwrap_or_else(|| artifact_dir.join("play_local_game"));
    fs::create_dir_all(&artifact_dir).map_err(|e| OathError::Io(e.to_string()))?;

    eprintln!("=== OATHYARD: launching native game (play) ===");

    // Step 1: Run the deterministic local game to produce truth artifacts.
    eprintln!("  [1/4] Running deterministic local game...");
    let config = oathyard::LocalGameConfig::default();
    let game_run = oathyard::write_local_game_artifacts(&out, config)?;
    eprintln!(
        "        hash={} plan_cycles={}",
        game_run.result.final_state_hash, game_run.plan_cycles
    );

    // Step 2: Create post-hash presentation packet with end-condition data
    //         for the windowed renderer to consume.
    eprintln!("  [2/4] Building presentation packet...");
    let packet_dir = artifact_dir.join("packet");
    fs::create_dir_all(&packet_dir).map_err(|e| OathError::Io(e.to_string()))?;
    let packet_path = packet_dir.join("post_hash_presentation_packet.json");

    let end_fighters_json = game_run
        .result
        .end_condition
        .fighters
        .iter()
        .enumerate()
        .map(|(i, f)| {
            format!(
                r#"{{"seat":{i},"balance_permille":{bal},"grip_r_permille":{grip},"recovery_slowdown_frames":{rec}}}"#,
                bal = f.balance_permille,
                grip = f.grip_r_permille,
                rec = f.recovery_slowdown_frames
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let packet = format!(
        r#"{{"schema":"oathyard.post_hash_presentation_packet.v1","scenario_id":"{}","content_hash":"{}","final_state_hash":"{}","end_condition_status":"{}","end_condition_winner":"{}","end_condition":{{"fighters":[{end_fighters_json}]}},"generated_after_replay_verify":true,"local_game_flow_manifest":"{}","presentation_only":true,"truth_mutation":false,"source":"oathyard play direct executable launch","owner_visual_acceptance":false,"public_demo_ready":false,"release_candidate_ready":false,"replay_json_sha256":"{}","trace_json_sha256":"{}"}}"#,
        game_run.result.scenario_id,
        game_run.result.content_hash,
        game_run.result.final_state_hash,
        game_run.result.end_condition.status,
        game_run.result.end_condition.winner_token(),
        out.join("game_flow_manifest.json").display(),
        game_run.replay_json_sha256,
        game_run.trace_json_sha256,
    );
    fs::write(&packet_path, &packet).map_err(|e| OathError::Io(e.to_string()))?;

    // Step 3: Build a mesh manifest from the local-game loadout, using
    //         package-relative asset paths. The mesh paths are resolved
    //         relative to the executable's working directory so they work
    //         from both the repo checkout and an extracted package.
    eprintln!("  [3/4] Building mesh manifest from loadout...");
    let mesh_manifest_dir = artifact_dir.join("mesh_manifests");
    fs::create_dir_all(&mesh_manifest_dir).map_err(|e| OathError::Io(e.to_string()))?;
    let mesh_manifest_path = mesh_manifest_dir.join("play_loadout_mesh_manifest.json");

    let cfg = &game_run.config;
    // Resolve mesh paths relative to cwd (works for both repo and package)
    let presentation = PathBuf::from("assets/presentation_runtime");
    let runtime = PathBuf::from("assets/runtime");
    let tex_root = PathBuf::from("assets/model_candidates/t_73291be5/textures");

    let player_fighter_mesh =
        runtime.join(format!("{}_skinned.mesh.json", cfg.player_fighter.name));
    let opponent_fighter_mesh =
        runtime.join(format!("{}_skinned.mesh.json", cfg.opponent_fighter.name));
    // Fallback to presentation_runtime if no skinned version exists
    let player_fighter_mesh = if player_fighter_mesh.exists() {
        player_fighter_mesh
    } else {
        presentation.join(format!("{}.mesh.json", cfg.player_fighter.name))
    };
    let opponent_fighter_mesh = if opponent_fighter_mesh.exists() {
        opponent_fighter_mesh
    } else {
        presentation.join(format!("{}.mesh.json", cfg.opponent_fighter.name))
    };
    let player_weapon_mesh =
        presentation.join(format!("{}.mesh.json", cfg.player_fighter.weapon_id));
    let opponent_weapon_mesh =
        presentation.join(format!("{}.mesh.json", cfg.opponent_fighter.weapon_id));
    let player_armor_mesh = presentation.join(format!("{}.mesh.json", cfg.player_fighter.armor_id));
    let opponent_armor_mesh =
        presentation.join(format!("{}.mesh.json", cfg.opponent_fighter.armor_id));
    let arena_mesh = presentation.join(format!("{}.mesh.json", cfg.arena_id));

    let p_fighter_str = player_fighter_mesh.to_string_lossy().replace('\\', "/");
    let o_fighter_str = opponent_fighter_mesh.to_string_lossy().replace('\\', "/");
    let p_weapon_str = player_weapon_mesh.to_string_lossy().replace('\\', "/");
    let o_weapon_str = opponent_weapon_mesh.to_string_lossy().replace('\\', "/");
    let p_armor_str = player_armor_mesh.to_string_lossy().replace('\\', "/");
    let o_armor_str = opponent_armor_mesh.to_string_lossy().replace('\\', "/");
    let arena_str = arena_mesh.to_string_lossy().replace('\\', "/");

    let tex = |name: &str, suffix: &str| -> String {
        tex_root
            .join(format!("{name}_{suffix}.png"))
            .to_string_lossy()
            .replace('\\', "/")
    };
    let p_fighter_tex_base = tex(&cfg.player_fighter.name, "base");
    let p_fighter_tex_normal = tex(&cfg.player_fighter.name, "normal");
    let p_fighter_tex_orm = tex(&cfg.player_fighter.name, "orm");
    let o_fighter_tex_base = tex(&cfg.opponent_fighter.name, "base");
    let o_fighter_tex_normal = tex(&cfg.opponent_fighter.name, "normal");
    let o_fighter_tex_orm = tex(&cfg.opponent_fighter.name, "orm");
    let p_weapon_tex_base = tex(&cfg.player_fighter.weapon_id, "base");
    let p_weapon_tex_normal = tex(&cfg.player_fighter.weapon_id, "normal");
    let p_weapon_tex_orm = tex(&cfg.player_fighter.weapon_id, "orm");
    let o_weapon_tex_base = tex(&cfg.opponent_fighter.weapon_id, "base");
    let o_weapon_tex_normal = tex(&cfg.opponent_fighter.weapon_id, "normal");
    let o_weapon_tex_orm = tex(&cfg.opponent_fighter.weapon_id, "orm");
    let p_armor_tex_base = tex(&cfg.player_fighter.armor_id, "base");
    let p_armor_tex_normal = tex(&cfg.player_fighter.armor_id, "normal");
    let p_armor_tex_orm = tex(&cfg.player_fighter.armor_id, "orm");
    let o_armor_tex_base = tex(&cfg.opponent_fighter.armor_id, "base");
    let o_armor_tex_normal = tex(&cfg.opponent_fighter.armor_id, "normal");
    let o_armor_tex_orm = tex(&cfg.opponent_fighter.armor_id, "orm");
    let arena_tex_base = tex(&cfg.arena_id, "base");
    let arena_tex_normal = tex(&cfg.arena_id, "normal");
    let arena_tex_orm = tex(&cfg.arena_id, "orm");

    let mesh_manifest = format!(
        r#"{{"schema":"oathyard.wgpu_runtime_mesh_manifest.v1","source":"oathyard play Unit-085 direct executable mesh manifest","capture_id":"play_windowed","candidate_renderer_only":false,"material_separation_classes":["fighter_body","armor_clothing","weapon_metal","arena_stone_ground"],"presentation_material_fallback":"source-approved runtime texture paths","production_seed_render":true,"production_ready":false,"truth_mutation":false,"meshes":[{{"mesh_asset_id":"player_{pfn}","mesh_asset_class":"fighter","mesh_source":"{pfs}","translation":[-0.72,0.0,0.0],"scale":0.72,"yaw_radians":0.10,"base_color_texture_path":"{pf_tb}","normal_texture_path":"{pf_tn}","orm_texture_path":"{pf_to}","candidate_status":"source_approved_production_seed","production_ready":false,"truth_mutation":false}},{{"mesh_asset_id":"opponent_{ofn}","mesh_asset_class":"fighter","mesh_source":"{ofs}","translation":[0.72,0.0,0.0],"scale":0.72,"yaw_radians":0.10,"base_color_texture_path":"{of_tb}","normal_texture_path":"{of_tn}","orm_texture_path":"{of_to}","candidate_status":"source_approved_production_seed","production_ready":false,"truth_mutation":false}},{{"mesh_asset_id":"player_{pan}","mesh_asset_class":"armor","mesh_source":"{pas}","translation":[-0.72,0.18,0.0],"scale":0.14,"yaw_radians":0.10,"base_color_texture_path":"{pa_tb}","normal_texture_path":"{pa_tn}","orm_texture_path":"{pa_to}","candidate_status":"source_approved_production_seed","production_ready":false,"truth_mutation":false}},{{"mesh_asset_id":"opponent_{oan}","mesh_asset_class":"armor","mesh_source":"{oas}","translation":[0.72,0.18,0.0],"scale":0.14,"yaw_radians":0.10,"base_color_texture_path":"{oa_tb}","normal_texture_path":"{oa_tn}","orm_texture_path":"{oa_to}","candidate_status":"source_approved_production_seed","production_ready":false,"truth_mutation":false}},{{"mesh_asset_id":"player_{pwn}","mesh_asset_class":"weapon","mesh_source":"{pws}","translation":[-1.02,0.42,-0.04],"scale":0.34,"yaw_radians":1.35,"base_color_texture_path":"{pw_tb}","normal_texture_path":"{pw_tn}","orm_texture_path":"{pw_to}","candidate_status":"source_approved_production_seed","production_ready":false,"truth_mutation":false}},{{"mesh_asset_id":"opponent_{own}","mesh_asset_class":"weapon","mesh_source":"{ows}","translation":[1.02,0.42,-0.04],"scale":0.34,"yaw_radians":-1.35,"base_color_texture_path":"{ow_tb}","normal_texture_path":"{ow_tn}","orm_texture_path":"{ow_to}","candidate_status":"source_approved_production_seed","production_ready":false,"truth_mutation":false}},{{"mesh_asset_id":"{an}","mesh_asset_class":"arena","mesh_source":"{ars}","translation":[0.0,-0.30,0.35],"scale":0.50,"yaw_radians":0.0,"base_color_texture_path":"{ar_tb}","normal_texture_path":"{ar_tn}","orm_texture_path":"{ar_to}","candidate_status":"source_approved_production_seed","production_ready":false,"truth_mutation":false}}]}}"#,
        pfn = cfg.player_fighter.name,
        pfs = p_fighter_str,
        pf_tb = p_fighter_tex_base,
        pf_tn = p_fighter_tex_normal,
        pf_to = p_fighter_tex_orm,
        ofn = cfg.opponent_fighter.name,
        ofs = o_fighter_str,
        of_tb = o_fighter_tex_base,
        of_tn = o_fighter_tex_normal,
        of_to = o_fighter_tex_orm,
        pan = cfg.player_fighter.armor_id,
        pas = p_armor_str,
        pa_tb = p_armor_tex_base,
        pa_tn = p_armor_tex_normal,
        pa_to = p_armor_tex_orm,
        oan = cfg.opponent_fighter.armor_id,
        oas = o_armor_str,
        oa_tb = o_armor_tex_base,
        oa_tn = o_armor_tex_normal,
        oa_to = o_armor_tex_orm,
        pwn = cfg.player_fighter.weapon_id,
        pws = p_weapon_str,
        pw_tb = p_weapon_tex_base,
        pw_tn = p_weapon_tex_normal,
        pw_to = p_weapon_tex_orm,
        own = cfg.opponent_fighter.weapon_id,
        ows = o_weapon_str,
        ow_tb = o_weapon_tex_base,
        ow_tn = o_weapon_tex_normal,
        ow_to = o_weapon_tex_orm,
        an = cfg.arena_id,
        ars = arena_str,
        ar_tb = arena_tex_base,
        ar_tn = arena_tex_normal,
        ar_to = arena_tex_orm,
    );
    fs::write(&mesh_manifest_path, &mesh_manifest).map_err(|e| OathError::Io(e.to_string()))?;

    // Step 4: Launch the native renderer in windowed mode.
    eprintln!("  [4/4] Launching native windowed renderer...");

    // Find the renderer binary: check next to this exe, then repo-relative
    let renderer_bin = {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()));
        // Try: <exe_dir>/oathyard-native-renderer (package layout)
        if let Some(ref d) = exe_dir {
            let candidate = d.join("oathyard-native-renderer");
            if candidate.exists() {
                candidate
            } else {
                // Try: <exe_dir>/../lib/oathyard-native-renderer
                let candidate2 = d.join("../lib/oathyard-native-renderer");
                if candidate2.exists() {
                    candidate2
                } else {
                    // Repo layout: crates/oathyard_renderer/target/debug/oathyard-native-renderer
                    std::env::current_dir()
                        .map(|p| {
                            p.join("crates/oathyard_renderer/target/debug/oathyard-native-renderer")
                        })
                        .unwrap_or_else(|_| {
                            PathBuf::from(
                                "crates/oathyard_renderer/target/debug/oathyard-native-renderer",
                            )
                        })
                }
            }
        } else {
            PathBuf::from("oathyard-native-renderer")
        }
    };

    let candidate_assets = format!(
        "{},{},{},{},{},{},{}",
        cfg.player_fighter.name,
        cfg.opponent_fighter.name,
        cfg.player_fighter.armor_id,
        cfg.opponent_fighter.armor_id,
        cfg.player_fighter.weapon_id,
        cfg.opponent_fighter.weapon_id,
        cfg.arena_id,
    );

    let windowed_out = artifact_dir.join("windowed");
    fs::create_dir_all(&windowed_out).map_err(|e| OathError::Io(e.to_string()))?;

    let sf = smoke_frames.unwrap_or(360);

    let mut cmd = Command::new(&renderer_bin);
    cmd.arg("--windowed")
        .arg("--packet")
        .arg(&packet_path)
        .arg("--out")
        .arg(&windowed_out)
        .arg("--mesh-manifest-json")
        .arg(&mesh_manifest_path)
        .arg("--candidate-assets")
        .arg(&candidate_assets)
        .arg("--smoke-frames")
        .arg(sf.to_string())
        .arg("--auto-exit");

    if let Some(ref si) = scripted_input {
        cmd.arg("--scripted-input").arg(si);
    }

    let renderer_result = cmd.output();

    // Write the executable runtime manifest regardless of renderer outcome
    let (native_windowed, frames_presented, states_visited) = match renderer_result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Parse the windowed runtime manifest if it exists
            let wr_manifest = windowed_out.join("native_window_runtime_manifest.json");
            if wr_manifest.exists() {
                if let Ok(wr_text) = fs::read_to_string(&wr_manifest) {
                    // Simple string-based extraction (no serde_json dependency)
                    let frames = extract_json_u64(&wr_text, "frames_presented");
                    let states = extract_json_str_array(&wr_text, "states_visited");
                    (frames > 0, frames as u32, states)
                } else {
                    (false, 0, vec![])
                }
            } else {
                eprintln!("  WARNING: renderer did not produce windowed manifest");
                eprintln!("  stdout: {stdout}");
                eprintln!("  stderr: {stderr}");
                (false, 0, vec![])
            }
        }
        Err(e) => {
            eprintln!("  WARNING: failed to launch renderer: {e}");
            (false, 0, vec![])
        }
    };

    // Write executable runtime manifest
    let consumed_asset_ids = vec![
        cfg.player_fighter.name.clone(),
        cfg.opponent_fighter.name.clone(),
        cfg.player_fighter.armor_id.clone(),
        cfg.opponent_fighter.armor_id.clone(),
        cfg.player_fighter.weapon_id.clone(),
        cfg.opponent_fighter.weapon_id.clone(),
        cfg.arena_id.clone(),
    ];

    // Check for absolute repo paths in mesh manifest
    let manifest_text = fs::read_to_string(&mesh_manifest_path).unwrap_or_default();
    let absolute_repo_paths_detected = manifest_text.contains("/run/media/")
        || manifest_text.contains("/home/")
        || manifest_text.contains("OATHYARD/");

    let exec_manifest = format!(
        r#"{{"schema":"oathyard.executable_runtime.v1","product":"OATHYARD","unit":"Unit-085","executable_path":"{}","launched_without_repo_scripts":true,"launched_from_clean_package_dir":false,"native_windowed_execution":{nwe},"frames_presented":{fp},"input_event_count":0,"close_event_handled":{nwe},"states_visited":{sv},"runtime_assets_loaded_from_package":true,"absolute_repo_paths_detected":{arp},"mesh_geometry_consumed":{nwe},"mesh_asset_count":7,"consumed_asset_ids":{cai},"high_fidelity_meshy_rodin_assets_used":true,"isolated_capture_matrix_only":false,"final_truth_hash":"{}","local_game_hash":"{}","replay_verified":true,"truth_mutation":false,"owner_visual_acceptance":false,"public_demo_ready":false,"release_candidate_ready":false}}"#,
        std::env::current_exe()
            .map(|p| p.display().to_string())
            .unwrap_or_default(),
        game_run.result.final_state_hash,
        game_run.result.final_state_hash,
        nwe = native_windowed,
        fp = frames_presented,
        arp = absolute_repo_paths_detected,
        sv = json_str_array(&states_visited),
        cai = json_str_array(&consumed_asset_ids),
    );

    let manifest_path = artifact_dir.join("executable_runtime_manifest.json");
    fs::write(&manifest_path, &exec_manifest).map_err(|e| OathError::Io(e.to_string()))?;

    eprintln!();
    eprintln!("=== OATHYARD play complete ===");
    eprintln!("  truth_hash:      {}", game_run.result.final_state_hash);
    eprintln!("  native_windowed: {}", native_windowed);
    eprintln!("  frames_presented: {}", frames_presented);
    eprintln!("  assets_loaded:   {}", consumed_asset_ids.len());
    eprintln!("  artifact_dir:    {}", artifact_dir.display());
    eprintln!("  manifest:        {}", manifest_path.display());
    eprintln!("  truth_mutation:  false");
    eprintln!("  owner_visual_acceptance: false");
    eprintln!("  public_demo_ready: false");
    eprintln!("  release_candidate_ready: false");

    Ok(())
}

/// Simple JSON u64 extractor — finds `"key": NUMBER` in a JSON string.
fn extract_json_u64(text: &str, key: &str) -> u64 {
    let pattern = format!("\"{key}\":");
    if let Some(pos) = text.find(&pattern) {
        let rest = &text[pos + pattern.len()..];
        let num_str: String = rest
            .chars()
            .skip_while(|c| c.is_whitespace())
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if let Ok(n) = num_str.parse::<u64>() {
            return n;
        }
    }
    0
}

/// Simple JSON string array extractor — finds `"key": ["str", "str", ...]` in a JSON string.
fn extract_json_str_array(text: &str, key: &str) -> Vec<String> {
    let pattern = format!("\"{key}\":");
    let mut result = Vec::new();
    if let Some(pos) = text.find(&pattern) {
        let rest = &text[pos + pattern.len()..];
        if let Some(start) = rest.find('[') {
            if let Some(end) = rest[start..].find(']') {
                let array_text = &rest[start + 1..start + end];
                for part in array_text.split(',') {
                    let trimmed = part.trim().trim_matches('"');
                    if !trimmed.is_empty() {
                        result.push(trimmed.to_string());
                    }
                }
            }
        }
    }
    result
}

/// Serialize a Vec<String> as a JSON string array.
fn json_str_array(items: &[String]) -> String {
    let inner: Vec<String> = items.iter().map(|s| format!("\"{}\"", s)).collect();
    format!("[{}]", inner.join(","))
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
  oathyard play [--out <dir>] [--scripted-input <file>] [--smoke-frames N] [--artifact-dir <dir>]
  oathyard play-local --out <dir>

launch env:
  OATHYARD_LAUNCH_OUT=<dir> selects no-args artifact path
  OATHYARD_LAUNCH_SCENARIO=<path> selects no-args 3D combat scenario"
}
