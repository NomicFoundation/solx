//!
//! The CLI/e2e tests entry module.
//!

use std::io::Write;
use std::process::Command;

use assert_cmd::assert::Assert;
use assert_cmd::assert::OutputAssertExt;

#[cfg(feature = "solc")]
mod abi;
#[cfg(feature = "solc")]
mod allow_paths;
mod asm;
#[cfg(feature = "solc")]
mod asm_solc_json;
mod ast_json;
#[cfg(feature = "solc")]
mod base_path;
mod benchmarks;
mod bin;
mod bin_runtime;
#[cfg(feature = "solc")]
mod debug_info;
#[cfg(feature = "solc")]
mod debug_info_runtime;
#[cfg(feature = "solc")]
mod devdoc;
mod empty;
mod evm_version;
#[cfg(feature = "solc")]
mod hashes;
mod help;
#[cfg(feature = "solc")]
mod include_path;
#[cfg(feature = "solc")]
mod ir;
mod ir_output;
#[cfg(feature = "solc")]
mod libraries;
mod llvm_ir;
mod llvm_options;
#[cfg(feature = "solc")]
mod metadata;
mod metadata_hash;
#[cfg(feature = "solc")]
mod metadata_literal;
mod no_cbor_metadata;
#[cfg(feature = "solc")]
mod no_import_callback;
mod optimization;
mod optimization_size_fallback;
mod output_dir;
mod overwrite;
mod recursive_process;
mod remappings;
mod standard_json;
mod standard_json_optimizer;
mod standard_json_output;
#[cfg(feature = "solc")]
mod storage_layout;
mod threads;
#[cfg(feature = "solc")]
mod transient_storage_layout;
#[cfg(feature = "solc")]
mod userdoc;
mod version;
#[cfg(feature = "solc")]
mod via_ir;
#[cfg(feature = "solc")]
mod yul;
#[cfg(feature = "solc")]
mod yul_parser;

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
