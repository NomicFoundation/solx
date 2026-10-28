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
mod ast_json;
mod base_path;
mod benchmarks;
mod bin;
mod bin_runtime;
mod debug_info;
mod debug_info_runtime;
mod debug_symbols;
mod devdoc;
mod docs_examples;
mod emit_llvm_ir;
mod emit_mlir;
mod empty;
mod evm_version;
mod hashes;
mod help;
mod include_path;
mod libraries;
mod llvm_ir;
mod llvm_options;
mod metadata;
mod metadata_hash;
mod metadata_literal;
mod mlir_dependencies;
mod no_cbor_metadata;
mod no_import_callback;
mod optimization;
mod optimization_size_fallback;
mod output_dir;
mod overwrite;
mod recursive_process;
mod remappings;
mod solidity_version;
mod stack_too_deep;
mod standard_json;
mod standard_json_immutable_references;
mod standard_json_optimizer;
mod standard_json_output;
mod storage_layout;
mod threads;
mod transient_storage_layout;
mod userdoc;
mod version;
mod via_ir;
mod yul;
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

///
/// Execute `solx` with the given arguments into a fresh `--output-dir`, and return the directory.
///
pub fn execute_solx_to_directory(args: &[&str]) -> anyhow::Result<tempfile::TempDir> {
    let directory = tempfile::TempDir::with_prefix("solx_output")?;
    let mut args = args.to_vec();
    args.extend([
        "--output-dir",
        directory.path().to_str().expect("Always valid"),
    ]);
    execute_solx(args.as_slice())?.success();
    Ok(directory)
}

///
/// The file with `suffix` that `--output-dir` wrote to `directory` for contract `name` in `path`.
///
pub fn output_file(
    directory: &tempfile::TempDir,
    path: &str,
    name: &str,
    suffix: &str,
) -> anyhow::Result<String> {
    let file_name = format!(
        "{}_{name}{suffix}",
        solx_utils::ContractName::sanitize_path(path)
    );
    std::fs::read_to_string(directory.path().join(file_name.as_str())).map_err(|error| {
        let written = std::fs::read_dir(directory.path())
            .into_iter()
            .flatten()
            .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
            .collect::<Vec<_>>();
        anyhow::anyhow!("reading `{file_name}`: {error}; written: {written:?}")
    })
}

///
/// The metadata that `--output-dir` wrote to `directory` for contract `name` in `path`.
///
pub fn output_metadata(
    directory: &tempfile::TempDir,
    path: &str,
    name: &str,
) -> anyhow::Result<serde_json::Value> {
    Ok(serde_json::from_str(
        output_file(directory, path, name, "_meta.json")?.as_str(),
    )?)
}
