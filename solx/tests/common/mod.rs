//!
//! Test common utilities.
//!

#![allow(dead_code)]

use std::path::PathBuf;

use assert_cmd::Command;

/// Returns a path under `tests/data/contracts/` for test contract files.
macro_rules! contract {
    ($relative:literal) => {
        concat!("tests/data/contracts/", $relative)
    };
}
pub(crate) use contract;

/// Returns a path under `tests/data/standard_json_input/` for standard JSON input files.
macro_rules! standard_json {
    ($relative:literal) => {
        concat!("tests/data/standard_json_input/", $relative)
    };
}
pub(crate) use standard_json;

pub const TEST_SOLIDITY_CONTRACT: &str = contract!("solidity/Test.sol");
pub const TEST_YUL_CONTRACT: &str = contract!("yul/Test.yul");
pub const TEST_LLVM_IR_CONTRACT: &str = contract!("llvm_ir/Test.ll");
pub const TEST_SOLIDITY_STANDARD_JSON: &str = standard_json!("solidity.json");

///
/// Setup required test dependencies.
///
pub fn setup() -> anyhow::Result<()> {
    let solx_bin = Command::new(assert_cmd::cargo::cargo_bin!(env!("CARGO_PKG_NAME")));
    let _ = solx_core::process::EXECUTABLE.set(PathBuf::from(solx_bin.get_program()));

    inkwell::support::enable_llvm_pretty_stack_trace();

    Ok(())
}
