//!
//! The CLI/e2e tests entry module.
//!

use std::io::Write;
use std::process::Command;

use assert_cmd::assert::Assert;
use assert_cmd::assert::OutputAssertExt;

mod abi;
mod allow_paths;
mod asm;
mod asm_solc_json;
mod ast_json;
mod base_path;
mod benchmarks;
mod bin;
mod bin_runtime;
mod debug_info;
mod debug_info_runtime;
mod devdoc;
mod empty;
mod evm_version;
mod hashes;
mod help;
mod include_path;
mod ir;
mod ir_output;
mod libraries;
mod llvm_ir;
mod llvm_options;
mod metadata;
mod metadata_hash;
mod metadata_literal;
mod no_cbor_metadata;
mod no_import_callback;
mod optimization;
mod optimization_size_fallback;
mod output_dir;
mod overwrite;
mod recursive_process;
mod remappings;
mod standard_json;
mod storage_layout;
mod threads;
mod transient_storage_layout;
mod userdoc;
mod version;
mod via_ir;
mod yul;

///
/// Execute `solx` with the given arguments and assert the result.
///
pub fn execute_solx(args: &[&str]) -> anyhow::Result<assert_cmd::assert::Assert> {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!(env!("CARGO_PKG_NAME")));
    Ok(command.args(args).assert())
}

///
/// Execute `solx` with the given arguments and environment variables, and assert the result.
///
pub fn execute_solx_with_env_vars(
    args: &[&str],
    env_vars: Vec<(&str, String)>,
) -> anyhow::Result<assert_cmd::assert::Assert> {
    let mut command = Command::new(assert_cmd::cargo::cargo_bin!(env!("CARGO_PKG_NAME")));
    for (key, value) in env_vars.into_iter() {
        command.env(key, value);
    }
    Ok(command.args(args).assert())
}

///
/// Execute `solx` with the given arguments and stdin input, and assert the result.
///
pub fn execute_solx_with_stdin(
    args: &[&str],
    path: &str,
) -> anyhow::Result<assert_cmd::assert::Assert> {
    let content = std::fs::read_to_string(path)
        .map_err(|error| anyhow::anyhow!("Failed to read file {path}: {error}"))?;

    let mut command = Command::new(assert_cmd::cargo::cargo_bin!(env!("CARGO_PKG_NAME")));
    command.stdin(std::process::Stdio::piped());
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());
    command.args(args);

    let mut process = command
        .spawn()
        .map_err(|error| anyhow::anyhow!("Subprocess spawning: {error:?}"))?;
    let stdin = process
        .stdin
        .as_mut()
        .ok_or_else(|| anyhow::anyhow!("Subprocess stdin getting error"))?;
    stdin
        .write_all(content.as_bytes())
        .map_err(|error| anyhow::anyhow!("Subprocess stdin writing: {error:?}"))?;

    let output = process
        .wait_with_output()
        .map_err(|error| anyhow::anyhow!("Subprocess output reading: {error:?}"))?;
    Ok(Assert::new(output).append_context("command", format!("{command:?}")))
}
