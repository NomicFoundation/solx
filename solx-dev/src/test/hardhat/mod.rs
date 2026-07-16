//!
//! `solx` Hardhat test tools.
//!

pub mod config;
pub mod output;

use std::collections::BTreeMap;
use std::process::Command;
use std::time::Instant;

use colored::Colorize;
use itertools::Itertools;

use self::config::Config;
use self::config::project::build_system::BuildSystem;
use self::output::test::Test as TestOutput;

///
/// Tests and runs benchmark on Hardhat projects.
///
pub fn test(
    config: Config,
    projects_directory: std::path::PathBuf,
    output_directory: std::path::PathBuf,
    solidity_version: String,
    project_filter: Vec<String>,
) -> anyhow::Result<()> {
    crate::utils::exists("git")?;
    crate::utils::exists("npm")?;

    std::fs::create_dir_all(projects_directory.as_path()).map_err(|error| {
        anyhow::anyhow!(
            "{} Hardhat projects directory {projects_directory:?}: {error}",
            "Creating".bright_green().bold(),
        )
    })?;

    let mut benchmark_inputs = Vec::with_capacity(config.projects.len() * 4);
    let mut attempted_projects = Vec::with_capacity(config.projects.len());
    let mut build_correctness_table = BTreeMap::new();
    let mut test_correctness_table = BTreeMap::new();
    let correctness_reference_compiler = config
        .compilers
        .values()
        .find(|compiler| !compiler.disabled && compiler.is_correctness_reference)
        .ok_or_else(|| {
            anyhow::anyhow!("No reference compiler specified in the Hardhat test configuration")
        })?
        .name
        .clone();
    let correctness_candidate_compiler = config
        .compilers
        .values()
        .find(|compiler| !compiler.disabled && compiler.is_correctness_candidate)
        .ok_or_else(|| {
            anyhow::anyhow!("No candidate compiler specified in the Hardhat test configuration")
        })?
        .name
        .clone();

    let mut compiler_shims = BTreeMap::new();
    for (identifier, compiler) in config
        .compilers
        .iter()
        .filter(|(_identifier, compiler)| !compiler.disabled)
    {
        let shim_directory =
            crate::utils::absolute_path(format!("./temp-compiler-shims/hardhat/{identifier}"))?;
        let compiler_path = crate::utils::absolute_path(compiler.path.as_str())?;
        compiler_shims.insert(
            identifier.clone(),
            crate::shim::CompilerInvocationShim::new(compiler_path, shim_directory.as_path())?,
        );
    }

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
        attempted_projects.push(project_name.clone());
        let mut project_directory = crate::utils::absolute_path(projects_directory.as_path())?;
        project_directory.push(project_name.as_str());

        for ((identifier, compiler), codegen) in config
            .compilers
            .iter()
            .filter(|(_identifier, compiler)| !compiler.disabled)
            .cartesian_product(crate::test::CODEGENS)
        {
            crate::utils::remove(project_directory.as_path(), project_name.as_str())?;

            let solidity_version = compiler
                .solidity_version
                .as_deref()
                .unwrap_or(solidity_version.as_str());

            let project_directory_str = project_directory.to_string_lossy();
            crate::utils::clone_repository(
                project.url.as_str(),
                &project_directory_str,
                project.commit.as_deref(),
                &format!(
                    "{} Hardhat project {}",
                    solx_utils::cargo_status_ok("Cloning"),
                    project_name.bright_white().bold()
                ),
            )?;

            eprintln!(
                "{} pragmas in Hardhat project {}",
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
                        format!(r#"s/pragma solidity.*/pragma solidity >={solidity_version};/g"#)
                            .as_str(),
                    ],
                )?;
            }

            let build_system = project.build_system.to_string();
            if let Some(version) = config.build_systems.get(&project.build_system) {
                let npm_spec = format!("{build_system}@{version}");
                let mut npm_install_build_system = Command::new("npm");
                npm_install_build_system.current_dir(project_directory.as_path());
                npm_install_build_system.args(["--loglevel", "error"]);
                npm_install_build_system.arg("--force");
                npm_install_build_system.arg("--yes");
                npm_install_build_system.arg("install");
                npm_install_build_system.arg("--global");
                npm_install_build_system.arg(&npm_spec);
                crate::utils::command_with_retries(
                    &mut npm_install_build_system,
                    format!(
                        "{} build system {} for Hardhat project {project_name}",
                        solx_utils::cargo_status_ok("Installing"),
                        build_system.bright_yellow().bold()
                    )
                    .as_str(),
                    16,
                )?;
            } else if project.build_system != BuildSystem::Npm {
                anyhow::bail!("Hardhat test configuration missing `build_systems.{build_system}`");
            }
            let mut build_system_install_command = Command::new(build_system.as_str());
            build_system_install_command.current_dir(project_directory.as_path());
            match project.build_system {
                BuildSystem::Npm => {
                    build_system_install_command.args(["--loglevel", "error"]);
                    build_system_install_command.arg("--force");
                    build_system_install_command.arg("--yes");
                }
                BuildSystem::Pnpm => {
                    build_system_install_command.arg("--ignore-scripts");
                }
                _ => {}
            }
            build_system_install_command.arg("install");
            crate::utils::command_with_retries(
                &mut build_system_install_command,
                format!(
                    "{} dependencies for Hardhat project {project_name}",
                    solx_utils::cargo_status_ok("Installing")
                )
                .as_str(),
                16,
            )?;

            let mut dependency_override_command = Command::new(build_system.as_str());
            dependency_override_command.current_dir(project_directory.as_path());
            match project.build_system {
                BuildSystem::Npm => {
                    dependency_override_command.args(["--loglevel", "error"]);
                    dependency_override_command.arg("--force");
                    dependency_override_command.arg("--yes");
                }
                BuildSystem::Yarn => {
                    dependency_override_command.arg("--silent");
                }
                BuildSystem::Pnpm => {
                    dependency_override_command.arg("--ignore-scripts");
                }
                _ => {}
            }
            dependency_override_command.arg("install");
            dependency_override_command.args(project.dependencies.as_slice());
            dependency_override_command.arg("--save-dev");
            crate::utils::command_with_retries(
                &mut dependency_override_command,
                format!(
                    "{} dependences with {} for Hardhat project {project_name}",
                    solx_utils::cargo_status_ok("Overriding"),
                    project
                        .dependencies
                        .iter()
                        .map(|dependency| dependency.bright_yellow().bold())
                        .join(", ")
                )
                .as_str(),
                16,
            )?;

            let config_file_name = if project_directory.join("hardhat.config.ts").exists() {
                Some("hardhat.config.ts")
            } else if project_directory.join("hardhat.config.js").exists() {
                Some("hardhat.config.js")
            } else {
                None
            };
            if let Some(config_file_name) = config_file_name {
                eprintln!(
                    "{} the configuration file {} of Hardhat project {}",
                    solx_utils::cargo_status_ok("Fixing"),
                    config_file_name.bright_white().bold(),
                    project_name.bright_white().bold(),
                );
                crate::utils::sed_file(
                    project_directory.join(config_file_name).as_path(),
                    &[
                        format!(r#"s/version:\s*["']0.8.30["']/version: "{solidity_version}"/g"#)
                            .as_str(),
                        format!(r#"s/default:\s*["']0.8.30["']/default: "{solidity_version}"/g"#)
                            .as_str(),
                    ],
                )?;
            }

            let compiler_shim = compiler_shims
                .get(identifier.as_str())
                .expect("Always exists");
            let compiler_path_str = compiler_shim.compiler_path.to_string_lossy();
            let toolchain_name = crate::test::toolchain_name(compiler.name.as_str(), codegen);
            compiler_shim.reset()?;

            let mut npm_compile_command = Command::new("npm");
            npm_compile_command.current_dir(&*project_directory_str);
            npm_compile_command.arg("run");
            npm_compile_command.arg("compile");
            for (key, value) in project.env.iter() {
                npm_compile_command.env(key, value);
            }
            if toolchain_name.contains("solx") {
                npm_compile_command.env("USE_SOLX", "true");
                npm_compile_command.env("SOLX", &*compiler_path_str);
            }
            npm_compile_command.env("VIA_IR", (codegen == "viaIR").to_string());
            let build_timestamp_start = Instant::now();
            let build_status = crate::utils::command(
                &mut npm_compile_command,
                format!(
                    "{} Hardhat project {} with {}",
                    solx_utils::cargo_status_ok("Building"),
                    project_name.bright_white().bold(),
                    toolchain_name.bright_white().bold()
                )
                .as_str(),
            );
            if let Err(error) = build_status {
                build_correctness_table
                    .entry(project_name.clone())
                    .or_insert_with(BTreeMap::new)
                    .insert(toolchain_name.clone(), 1);
                benchmark_inputs.push(solx_benchmark_converter::Input::new(
                    solx_benchmark_converter::BuildFailuresReport(1),
                    project_name.clone(),
                    toolchain_name.clone(),
                ));
                eprintln!(
                    "{} Hardhat project {} with {} failed: {error}",
                    solx_utils::cargo_status_error("Building"),
                    project_name.bright_white().bold(),
                    toolchain_name.bright_white().bold()
                );
                continue;
            }
            let compilation_time = build_timestamp_start.elapsed().as_millis() as u64;
            // solc toolchains compile with Hardhat's own downloaded compiler —
            // the configured path is never passed to the project, so only solx
            // toolchains can be identity-checked.
            if toolchain_name.contains("solx") {
                compiler_shim.verify(toolchain_name.as_str(), project_name.as_str())?;
            }

            let mut npm_test_command = Command::new("npm");
            npm_test_command.current_dir(&*project_directory_str);
            npm_test_command.arg("run");
            npm_test_command.arg("test");
            for (key, value) in project.env.iter() {
                npm_test_command.env(key, value);
            }
            let npm_test_report_path = project_directory.join("junit-report.json");
            let npm_test_report_path_str = npm_test_report_path.to_string_lossy();
            npm_test_command.env("JUNIT_REPORT", &*npm_test_report_path_str);
            if toolchain_name.contains("solx") {
                npm_test_command.env("USE_SOLX", "true");
                npm_test_command.env("SOLX", &*compiler_path_str);
            }
            npm_test_command.env("VIA_IR", (codegen == "viaIR").to_string());
            let test_timestamp_start = Instant::now();
            let _ = crate::utils::command(
                &mut npm_test_command,
                format!(
                    "{} Hardhat project {} with {}",
                    solx_utils::cargo_status_ok("Testing"),
                    project_name.bright_white().bold(),
                    toolchain_name.bright_white().bold()
                )
                .as_str(),
            );
            let test_output = TestOutput::try_from(npm_test_report_path)?;
            for failure in test_output.failures.iter() {
                eprintln!(
                    "{} {}:{}\n{}{}",
                    solx_utils::cargo_status_error("Failure"),
                    failure.file.bright_white().bold(),
                    failure.title.bright_white().bold(),
                    " ".repeat(13),
                    failure.err.to_string().bright_black().bold(),
                );
            }
            let testing_time = test_timestamp_start.elapsed().as_millis() as u64;
            test_correctness_table
                .entry(project_name.clone())
                .or_insert_with(BTreeMap::new)
                .insert(toolchain_name.clone(), test_output.stats.failures);
            benchmark_inputs.push(solx_benchmark_converter::Input::new(
                solx_benchmark_converter::TestFailuresReport(test_output.stats.failures),
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
        }
    }

    let enabled_compilers: Vec<&str> = config
        .compilers
        .values()
        .filter(|compiler| !compiler.disabled)
        .map(|compiler| compiler.name.as_str())
        .collect();
    crate::test::verify_benchmark_coverage(
        benchmark_inputs.as_slice(),
        attempted_projects.as_slice(),
        enabled_compilers.as_slice(),
    )?;

    let benchmark = solx_benchmark_converter::Benchmark::from_inputs(benchmark_inputs)?;
    let enabled_compiler_names: std::collections::HashSet<&str> = config
        .compilers
        .values()
        .filter(|compiler| !compiler.disabled)
        .map(|compiler| compiler.name.as_str())
        .collect();
    let comparisons: Vec<solx_benchmark_converter::OutputComparison> = config
        .comparisons
        .iter()
        .filter(|comparison| {
            if comparison.disabled {
                return false;
            }
            let left_compiler = comparison
                .left
                .trim_end_matches("-legacy")
                .trim_end_matches("-viaIR");
            let right_compiler = comparison
                .right
                .trim_end_matches("-legacy")
                .trim_end_matches("-viaIR");
            enabled_compiler_names.contains(left_compiler)
                && enabled_compiler_names.contains(right_compiler)
        })
        .map(|comparison| {
            solx_benchmark_converter::OutputComparison::new(
                comparison.left.clone(),
                comparison.right.clone(),
            )
        })
        .collect();
    std::fs::create_dir_all(output_directory.as_path()).map_err(|error| {
        anyhow::anyhow!(
            "{} Hardhat output reports directory {output_directory:?}: {error}",
            "Creating".bright_green().bold(),
        )
    })?;
    let base_path = crate::utils::absolute_path(output_directory)?;

    crate::test::write_benchmark_json(
        &benchmark,
        base_path.as_path(),
        solx_benchmark_converter::SuiteKind::Hardhat.benchmark_file(),
    )?;

    let output: solx_benchmark_converter::Output = (
        benchmark,
        comparisons,
        solx_benchmark_converter::OutputFormat::Xlsx,
    )
        .try_into()?;
    let mut output_path = base_path;
    output_path.push("hardhat-report.xlsx");
    eprintln!(
        "{} the spreadsheet report to {}",
        solx_utils::cargo_status_ok("Writing"),
        output_path.to_string_lossy().bright_white().bold()
    );
    output.write_to_file(output_path)?;

    let mut errors = Vec::new();
    for project in attempted_projects.iter() {
        for codegen in crate::test::CODEGENS {
            let reference_toolchain =
                crate::test::toolchain_name(correctness_reference_compiler.as_str(), codegen);
            let candidate_toolchain =
                crate::test::toolchain_name(correctness_candidate_compiler.as_str(), codegen);
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
                continue;
            }

            // Only comparable when both toolchains actually ran the tests;
            // a toolchain that failed to build has no test entry.
            let reference_test_failures = test_correctness_table
                .get(project)
                .and_then(|toolchains| toolchains.get(&reference_toolchain))
                .copied();
            let candidate_test_failures = test_correctness_table
                .get(project)
                .and_then(|toolchains| toolchains.get(&candidate_toolchain))
                .copied();
            if let (Some(reference_test_failures), Some(candidate_test_failures)) =
                (reference_test_failures, candidate_test_failures)
                && candidate_test_failures > reference_test_failures
            {
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
