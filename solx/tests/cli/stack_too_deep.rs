//!
//! CLI tests for stack-too-deep handling.
//!

use predicates::prelude::*;

#[cfg(feature = "solc")]
#[test]
fn stack_too_deep_solc() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/StackTooDeepSolc.sol"),
        "--bin",
    ];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains("Binary").count(1));

    Ok(())
}

#[cfg(feature = "solc")]
#[test]
fn stack_too_deep_llvm() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/StackTooDeepLLVM.sol"),
        "--bin",
        "-O1",
    ];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stderr(predicate::str::contains("Warning: Performance of this contract can be compromised due to the presence of this memory-unsafe assembly block."));

    Ok(())
}

#[cfg(feature = "solc")]
#[test]
fn stack_too_deep_llvm_suppressed() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/StackTooDeepLLVM.sol"),
        "--bin",
        "-O1",
    ];
    let env_vars = vec![("EVM_DISABLE_MEMORY_SAFE_ASM_CHECK", "1".to_owned())];

    let result = crate::cli::execute_solx_with_env_vars(args, env_vars)?;

    result
        .success()
        .stdout(predicate::str::contains("Binary").count(2));

    Ok(())
}

// The reported spill area is underestimated under the pinned LLVM backend, so the
// fixture compiles only through stack-too-deep retries in both the initial settings
// and the size fallback.
#[cfg(feature = "solc")]
#[test]
fn stack_too_deep_size_fallback() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("stack_too_deep_size_fallback.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains("bytecode"))
        .stdout(predicate::str::contains("object"))
        .stdout(predicate::str::contains("Stack-too-deep error").not());

    Ok(())
}
