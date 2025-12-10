//!
//! `solx` Foundry test tools.
//!

pub mod config;
pub mod output;

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use colored::Colorize;
use itertools::Itertools;

use self::config::Config;
use self::output::build::Build as BuildOutput;
use self::output::test::Test as TestOutput;

///
/// Tests and runs benchmark on Foundry projects.
///
pub fn test(
    config: Config,
    projects_directory: std::path::PathBuf,
    output_directory: std::path::PathBuf,
    solidity_version: String,
    project_filter: Vec<String>,
) -> anyhow::Result<()> {
    crate::utils::exists("forge")?;

    std::fs::create_dir_all(projects_directory.as_path()).map_err(|error| {
        anyhow::anyhow!(
            "{} Foundry projects directory {projects_directory:?}: {error}",
            "Creating".bright_green().bold(),
        )
    })?;

    let mut benchmark_inputs = Vec::with_capacity(config.projects.len() * 4);
    let mut build_correctness_table = BTreeMap::new();
    let mut test_correctness_table = BTreeMap::new();

    for (project_name, project) in config
        .projects
        .into_iter()
        .filter(|(project_name, project)| {
            !project.disabled
                && (project_filter.is_empty()
                    || project_filter
                        .iter()
                        .any(|element| project_name.contains(element)))
        })
    {
        let mut project_directory = crate::utils::absolute_path(projects_directory.as_path())?;
        project_directory.push(project_name.as_str());
        if !project_directory.exists() {
            crate::utils::exists("git")?;

            let mut clone_command = Command::new("git");
            clone_command.arg("clone");
            clone_command.args(["--depth", "1"]);
            clone_command.arg("--recurse-submodules");
            clone_command.arg("--shallow-submodules");
            clone_command.arg(project.url.as_str());
            clone_command.arg(project_directory.to_string_lossy().as_ref());
            crate::utils::command(
                &mut clone_command,
                format!(
                    "{} Foundry project {}",
                    solx_utils::cargo_status_ok("Cloning"),
                    project_name.bright_white().bold()
                )
                .as_str(),
            )?;
        } else {
            clean(project_directory.as_path(), project_name.as_str())?;
        }

        eprintln!(
            "{} pragmas in Foundry project {}",
            solx_utils::cargo_status_ok("Fixing"),
            project_name.bright_white().bold()
        );
        for solidity_file in
            glob::glob(format!("{}/**/*.sol", project_directory.to_string_lossy()).as_str())
                .expect("Always valid")
                .filter_map(Result::ok)
        {
            if !solidity_file.is_file() {
                continue;
            }
            crate::utils::sed_file(
                solidity_file.as_path(),
                &[
                    format!(r#"s/pragma solidity.*/pragma solidity ={solidity_version};/g"#)
                        .as_str(),
                ],
            )?;
        }

        if project.requires_yarn {
            crate::utils::exists("npm")?;

            let build_system = "yarn";
            let mut npm_install_yarn = Command::new("npm");
            npm_install_yarn.current_dir(project_directory.as_path());
            npm_install_yarn.arg("install");
            npm_install_yarn.args(["--loglevel", "error"]);
            npm_install_yarn.arg("--force");
            npm_install_yarn.arg("--yes");
            npm_install_yarn.arg("--global");
            npm_install_yarn.arg(build_system);
            crate::utils::command(
                &mut npm_install_yarn,
                format!(
                    "{} build system {} for Foundry project {project_name}",
                    solx_utils::cargo_status_ok("Installing"),
                    build_system.bright_yellow().bold()
                )
                .as_str(),
            )?;
            let mut yarn_install_command = Command::new(build_system);
            yarn_install_command.args(["--cwd", project_directory.to_string_lossy().as_ref()]);
            yarn_install_command.arg("install");
            crate::utils::command(
                &mut yarn_install_command,
                format!(
                    "{} dependencies for Foundry project {project_name}",
                    solx_utils::cargo_status_ok("Installing")
                )
                .as_str(),
            )?;
        }

        let mut forge_config_fix_command = Command::new("forge");
        forge_config_fix_command.current_dir(project_directory.as_path());
        forge_config_fix_command.arg("config");
        forge_config_fix_command.arg("--fix");
        crate::utils::command(
            &mut forge_config_fix_command,
            format!(
                "{} Foundry project {}",
                solx_utils::cargo_status_ok("Fixing"),
                project_name.bright_white().bold()
            )
            .as_str(),
        )?;
        crate::utils::sed_file(
            project_directory.join("foundry.toml").as_path(),
            &[
                r#"s/deny_warnings\s*=.*\n//g"#,
                r#"s/via_ir\s*=.*\n//g"#,
                r#"s/evm_version\s*=.*\n//g"#,
                format!(r#"s/solc_version\s*=\s*["'].*["']/solc_version = '{solidity_version}'/g"#)
                    .as_str(),
                format!(r#"s/solc\s*=\s*["'].*["']/solc_version = '{solidity_version}'/g"#)
                    .as_str(),
            ],
        )?;

        for ((_identifier, compiler), codegen) in config
            .compilers
            .iter()
            .cartesian_product(["legacy", "viaIR"])
        {
            let compiler_path = crate::utils::absolute_path(compiler.path.as_str())?;
            let toolchain_name = format!("{}-{codegen}", compiler.name);

            let mut forge_build_command = Command::new("forge");
            forge_build_command.arg("build");
            forge_build_command.args(["--root", project_directory.to_string_lossy().as_ref()]);
            forge_build_command.args(["--use", compiler_path.to_string_lossy().as_ref()]);
            if codegen == "viaIR" {
                forge_build_command.arg("--via-ir");
            }
            forge_build_command.args([
                "--evm-version",
                solx_utils::EVMVersion::Prague.to_string().as_str(),
            ]);
            forge_build_command.arg("--optimize");
            forge_build_command.arg("--no-metadata");
            forge_build_command.arg("--force");
            forge_build_command.arg("--json");
            for (key, value) in project.env.iter() {
                forge_build_command.env(key, value);
            }
            let build_timestamp_start = Instant::now();
            let build_output = match crate::utils::command_with_json_output::<BuildOutput>(
                &mut forge_build_command,
                format!(
                    "{} Foundry project {} with {}",
                    solx_utils::cargo_status_ok("Building"),
                    project_name.bright_white().bold(),
                    toolchain_name.bright_white().bold()
                )
                .as_str(),
                false,
            ) {
                Ok(build_output) => build_output,
                Err(_) => {
                    eprintln!(
                        "{} Foundry project {} with {} failed",
                        solx_utils::cargo_status_error("Building"),
                        project_name.bright_white().bold(),
                        toolchain_name.bright_white().bold()
                    );
                    clean(project_directory.as_path(), project_name.as_str())?;
                    continue;
                }
            };
            let compilation_time = build_timestamp_start.elapsed().as_millis() as u64;
            for error in build_output.errors.iter() {
                eprintln!(
                    "{}",
                    error["formattedMessage"]
                        .as_str()
                        .expect("formattedMessage is always a string")
                );
            }
            let built_contracts_count = build_output
                .contracts
                .values()
                .map(|contracts| contracts.len())
                .sum::<usize>();
            let build_errors = build_output
                .errors
                .iter()
                .filter(|error| {
                    error
                        .as_object()
                        .expect("Always valid")
                        .get("severity")
                        .unwrap_or(&serde_json::Value::String("".to_string()))
                        == "error"
                })
                .count();
            build_correctness_table
                .entry(project_name.clone())
                .or_insert_with(BTreeMap::new)
                .insert(toolchain_name.clone(), build_errors);
            if build_errors > 0 || built_contracts_count == 0 {
                benchmark_inputs.push(solx_benchmark_converter::Input::new(
                    solx_benchmark_converter::BuildFailuresReport(build_errors),
                    project_name.clone(),
                    toolchain_name.clone(),
                ));
                eprintln!("{} Building Foundry project {} with {} failed with {build_errors} errors and {built_contracts_count} built contracts", solx_utils::cargo_status_error("Error"), project_name.bright_white().bold(), toolchain_name.bright_white().bold());
                clean(project_directory.as_path(), project_name.as_str())?;
                continue;
            }

            let mut forge_build_sizes_command = Command::new("forge");
            forge_build_sizes_command.arg("build");
            forge_build_sizes_command
                .args(["--root", project_directory.to_string_lossy().as_ref()]);
            forge_build_sizes_command.args(["--use", compiler_path.to_string_lossy().as_ref()]);
            if codegen == "viaIR" {
                forge_build_sizes_command.arg("--via-ir");
            }
            forge_build_sizes_command.args([
                "--evm-version",
                solx_utils::EVMVersion::Prague.to_string().as_str(),
            ]);
            forge_build_sizes_command.arg("--optimize");
            forge_build_sizes_command.arg("--no-metadata");
            forge_build_sizes_command.arg("--sizes");
            forge_build_sizes_command.arg("--json");
            for (key, value) in project.env.iter() {
                forge_build_sizes_command.env(key, value);
            }
            let build_sizes_output = crate::utils::command_with_json_output::<
                solx_benchmark_converter::FoundrySizeReport,
            >(
                &mut forge_build_sizes_command,
                format!(
                    "{} Foundry project {} for {} with {}",
                    solx_utils::cargo_status_ok("Benchmarking"),
                    project_name.bright_white().bold(),
                    "[SIZE]".bright_white().bold(),
                    toolchain_name.bright_white().bold()
                )
                .as_str(),
                true,
            )?;
            benchmark_inputs.push(solx_benchmark_converter::Input::new(
                solx_benchmark_converter::InputReport::FoundrySize(build_sizes_output),
                project_name.clone(),
                toolchain_name.clone(),
            ));

            let mut forge_test_command = Command::new("forge");
            forge_test_command.arg("test");
            forge_test_command.args(["--root", project_directory.to_string_lossy().as_ref()]);
            forge_test_command.args(["--use", compiler_path.to_string_lossy().as_ref()]);
            if codegen == "viaIR" {
                forge_test_command.arg("--via-ir");
            }
            forge_test_command.args(["--fuzz-runs", "0"]);
            forge_test_command.args(["--fuzz-seed", "0xdeadbeef"]);
            forge_test_command.args([
                "--evm-version",
                solx_utils::EVMVersion::Prague.to_string().as_str(),
            ]);
            forge_test_command.arg("--optimize");
            forge_test_command.arg("--no-metadata");
            forge_test_command.arg("--json");
            forge_test_command.arg("--allow-failure");
            for (key, value) in project.env.iter() {
                forge_test_command.env(key, value);
            }
            let test_timestamp_start = Instant::now();
            let test_output = crate::utils::command_with_json_output::<TestOutput>(
                &mut forge_test_command,
                format!(
                    "{} Foundry project {} with {}",
                    solx_utils::cargo_status_ok("Testing"),
                    project_name.bright_white().bold(),
                    toolchain_name.bright_white().bold()
                )
                .as_str(),
                false,
            )?;
            let testing_time = test_timestamp_start.elapsed().as_millis() as u64;
            let test_failures_count =
                Iterator::flatten(test_output.0.iter().map(|(full_path, file)| {
                    file.test_results.iter().filter(move |(test_name, result)| {
                        if result.status == "Failure" {
                            eprintln!(
                                "{} {}\n{}{}",
                                solx_utils::cargo_status_error("Failure"),
                                format!("{full_path}.{test_name}").bright_white().bold(),
                                " ".repeat(13),
                                result
                                    .reason
                                    .as_ref()
                                    .map_or("<Unknown reason>", |v| v)
                                    .bright_black()
                                    .bold()
                            );
                            return true;
                        }
                        false
                    })
                }))
                .count();
            test_correctness_table
                .entry(project_name.clone())
                .or_insert_with(BTreeMap::new)
                .insert(toolchain_name.clone(), test_failures_count);
            benchmark_inputs.push(solx_benchmark_converter::Input::new(
                solx_benchmark_converter::TestFailuresReport(test_failures_count),
                project_name.clone(),
                toolchain_name.clone(),
            ));

            let mut forge_test_gas_command = Command::new("forge");
            forge_test_gas_command.arg("test");
            forge_test_gas_command.args(["--root", project_directory.to_string_lossy().as_ref()]);
            forge_test_gas_command.args(["--use", compiler_path.to_string_lossy().as_ref()]);
            if codegen == "viaIR" {
                forge_test_gas_command.arg("--via-ir");
            }
            forge_test_gas_command.args(["--fuzz-runs", "0"]);
            forge_test_gas_command.args(["--fuzz-seed", "0xdeadbeef"]);
            forge_test_gas_command.args([
                "--evm-version",
                solx_utils::EVMVersion::Prague.to_string().as_str(),
            ]);
            forge_test_gas_command.arg("--optimize");
            forge_test_gas_command.arg("--no-metadata");
            forge_test_gas_command.arg("--gas-report");
            forge_test_gas_command.arg("--json");
            forge_test_gas_command.arg("--allow-failure");
            for (key, value) in project.env.iter() {
                forge_test_gas_command.env(key, value);
            }
            let test_gas_output = crate::utils::command_with_json_output::<
                solx_benchmark_converter::FoundryGasReport,
            >(
                &mut forge_test_gas_command,
                format!(
                    "{} Foundry project {} for {} with {}",
                    solx_utils::cargo_status_ok("Benchmarking"),
                    project_name.bright_white().bold(),
                    "[GAS]".bright_white().bold(),
                    toolchain_name.bright_white().bold()
                )
                .as_str(),
                false,
            )?;

            benchmark_inputs.push(solx_benchmark_converter::Input::new(
                solx_benchmark_converter::InputReport::FoundryGas(test_gas_output),
                project_name.clone(),
                toolchain_name.clone(),
            ));
            benchmark_inputs.push(solx_benchmark_converter::Input::new(
                solx_benchmark_converter::CompilationTimeReport(compilation_time),
                project_name.clone(),
                toolchain_name.clone(),
            ));
            benchmark_inputs.push(solx_benchmark_converter::Input::new(
                solx_benchmark_converter::TestingTimeReport(testing_time),
                project_name.clone(),
                toolchain_name.clone(),
            ));

            clean(project_directory.as_path(), project_name.as_str())?;
        }
    }

    let benchmark = solx_benchmark_converter::Benchmark::from_inputs(benchmark_inputs.into_iter())?;
    let output: solx_benchmark_converter::Output = (
        benchmark,
        solx_benchmark_converter::InputSource::Tooling,
        solx_benchmark_converter::OutputFormat::Xlsx,
    )
        .try_into()?;

    std::fs::create_dir_all(output_directory.as_path()).map_err(|error| {
        anyhow::anyhow!(
            "{} Foundry output reports directory {output_directory:?}: {error}",
            "Creating".bright_green().bold(),
        )
    })?;
    let mut output_path = crate::utils::absolute_path(output_directory)?;
    output_path.push("foundry-report.xlsx");
    eprintln!(
        "{} the spreadsheet report to {}",
        solx_utils::cargo_status_ok("Writing"),
        output_path.to_string_lossy().bright_white().bold()
    );
    output.write_to_file(output_path)?;

    let correctness_reference_compiler = config
        .compilers
        .values()
        .find(|compiler| compiler.is_correctness_reference)
        .ok_or_else(|| {
            anyhow::anyhow!("No reference compiler specified in the Foundry test configuration")
        })?
        .name
        .clone();
    let correctness_candidate_compiler = config
        .compilers
        .values()
        .find(|compiler| compiler.is_correctness_candidate)
        .ok_or_else(|| {
            anyhow::anyhow!("No candidate compiler specified in the Foundry test configuration")
        })?
        .name
        .clone();
    let mut errors = Vec::new();
    for project in build_correctness_table.keys() {
        for codegen in ["legacy", "viaIR"] {
            let reference_toolchain = format!("{}-{}", correctness_reference_compiler, codegen);
            let candidate_toolchain = format!("{}-{}", correctness_candidate_compiler, codegen);
            let reference_build_errors = build_correctness_table
                .get(project)
                .and_then(|toolchains| toolchains.get(&reference_toolchain))
                .copied()
                .unwrap_or_default();
            let candidate_build_errors = build_correctness_table
                .get(project)
                .and_then(|toolchains| toolchains.get(&candidate_toolchain))
                .copied()
                .unwrap_or_default();
            if candidate_build_errors > reference_build_errors {
                errors.push(format!(
                    "{} Building correctness mismatch for project {} between reference toolchain {} ({} errors) and candidate toolchain {} ({} errors)",
                    solx_utils::cargo_status_error("Error"),
                    project.bright_white().bold(),
                    reference_toolchain.bright_white().bold(),
                    reference_build_errors,
                    candidate_toolchain.bright_white().bold(),
                    candidate_build_errors
                ));
            }

            let reference_test_failures = test_correctness_table
                .get(project)
                .and_then(|toolchains| toolchains.get(&reference_toolchain))
                .copied()
                .unwrap_or_default();
            let candidate_test_failures = test_correctness_table
                .get(project)
                .and_then(|toolchains| toolchains.get(&candidate_toolchain))
                .copied()
                .unwrap_or_default();
            if candidate_test_failures > reference_test_failures {
                errors.push(format!(
                    "{} Testing correctness mismatch for project {} between reference toolchain {} ({} failures) and candidate toolchain {} ({} failures)",
                    solx_utils::cargo_status_error("Error"),
                    project.bright_white().bold(),
                    reference_toolchain.bright_white().bold(),
                    reference_test_failures,
                    candidate_toolchain.bright_white().bold(),
                    candidate_test_failures
                ));
            }
        }
    }
    if !errors.is_empty() {
        anyhow::bail!(errors.join("\n"));
    }

    Ok(())
}

///
/// Cleans the project after building and testing.
///
pub fn clean(project_directory: &Path, project_name: &str) -> anyhow::Result<()> {
    let mut forge_clean_command = Command::new("forge");
    forge_clean_command.arg("clean");
    forge_clean_command.args(["--root", project_directory.to_string_lossy().as_ref()]);
    crate::utils::command(
        &mut forge_clean_command,
        format!(
            "{} Foundry project {}",
            solx_utils::cargo_status_ok("Cleaning"),
            project_name.bright_white().bold()
        )
        .as_str(),
    )?;
    Ok(())
}
