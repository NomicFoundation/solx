//!
//! `solx-tester` subprocess runner.
//!

use std::path::PathBuf;
use std::process::Command;

use crate::arguments::test::SolxTester as Arguments;

///
/// Returns the default path to the solx-tester binary.
/// Honors CARGO_TARGET_DIR if set, otherwise uses ./target.
///
fn default_binary_path() -> PathBuf {
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "./target".to_owned());
    PathBuf::from(target_dir)
        .join("release")
        .join("solx-tester")
}

///
/// Runs solx-tester as a subprocess with the provided arguments.
///
pub fn run(arguments: Arguments) -> anyhow::Result<()> {
    let binary = arguments.binary.unwrap_or_else(default_binary_path);

    if !binary.exists() {
        anyhow::bail!(
            "solx-tester binary not found at {binary:?}. Build it with: cargo build --release --bin solx-tester"
        );
    }

    let mut command = Command::new(&binary);

    if arguments.verbose {
        command.arg("--verbose");
    }
    if arguments.quiet {
        command.arg("--quiet");
    }
    if arguments.debug {
        command.arg("--debug");
    }
    if arguments.trace {
        command.arg("--trace");
    }
    if arguments.via_ir {
        command.arg("--via-ir");
    }
    if let Some(optimizer) = &arguments.optimizer {
        command.arg("--optimizer").arg(optimizer);
    }
    for path in &arguments.path {
        command.arg("--path").arg(path);
    }
    for group in &arguments.group {
        command.arg("--group").arg(group);
    }
    if let Some(benchmark) = &arguments.benchmark {
        command.arg("--benchmark").arg(benchmark);
    }
    if let Some(benchmark_format) = &arguments.benchmark_format {
        command.arg("--benchmark-format").arg(benchmark_format);
    }
    if let Some(threads) = arguments.threads {
        command.arg("--threads").arg(threads.to_string());
    }
    if let Some(solidity_compiler) = &arguments.solidity_compiler {
        command.arg("--solidity-compiler").arg(solidity_compiler);
    }
    if let Some(workflow) = &arguments.workflow {
        command.arg("--workflow").arg(workflow);
    }
    if let Some(solc_bin_config_path) = &arguments.solc_bin_config_path {
        command
            .arg("--solc-bin-config-path")
            .arg(solc_bin_config_path);
    }
    if arguments.llvm_verify_each {
        command.arg("--llvm-verify-each");
    }
    if arguments.llvm_debug_logging {
        command.arg("--llvm-debug-logging");
    }

    crate::utils::command(&mut command, "Running solx-tester")
}
