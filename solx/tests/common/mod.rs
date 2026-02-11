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

///
/// Setup required test dependencies.
///
pub fn setup() -> anyhow::Result<()> {
    let solx_bin = Command::new(assert_cmd::cargo::cargo_bin!(env!("CARGO_PKG_NAME")));
    let _ = solx_core::process::EXECUTABLE.set(PathBuf::from(solx_bin.get_program()));

    inkwell::support::enable_llvm_pretty_stack_trace();

    Ok(())
}
