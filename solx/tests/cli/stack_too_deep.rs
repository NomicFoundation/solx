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

// Recursive functions cannot use the memory spill, so their stack-too-deep is
// unrecoverable: the worker relays the LLVM fatal error, and the failure must name
// the contract instead of dropping it from the output.
#[cfg(feature = "solc")]
#[test]
fn stack_too_deep_recursive() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        crate::common::contract!("solidity/RecursiveStackTooDeep.sol"),
        "--bin",
        "-O1",
    ];

    let result = crate::cli::execute_solx(args)?;

    result.failure().stderr(predicate::str::contains(
        "It is recursive and has stack too deep errors.",
    ));

    Ok(())
}

#[cfg(feature = "solc")]
#[test]
fn stack_too_deep_recursive_standard_json() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("recursive_stack_too_deep.json"),
    ];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains(
            "It is recursive and has stack too deep errors.",
        ))
        .stdout(predicate::str::contains("RecursiveStackTooDeep.sol"));

    Ok(())
}

// Chain 97 0xe8426cf4a7fb9f73eba08edc6dd7f20bb2f8aff5; rejected at modes 1 to 3, compiles at s and z.
#[cfg(feature = "solc")]
#[test]
fn recursive_stack_too_deep_slot_placement() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("recursive_stack_too_deep_slot_placement.json"),
    ];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains(
            "Stackification failed for '_createAndPlaceSlot",
        ))
        .stdout(predicate::str::contains("xqore/XQoreBTC.sol"));

    Ok(())
}

// Chain 97 0x5ff2c60b164c928038cbeb51464c2f36a009b586; rejected only at mode 1. The named wrapper
// has its callee inlined; the cycle is _autoFlushQueue -> _closeIfNeeded -> _autoFlushQueue.
#[cfg(feature = "solc")]
#[test]
fn recursive_stack_too_deep_queue_flush() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("recursive_stack_too_deep_queue_flush.json"),
    ];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains(
            "Stackification failed for '_autoFlushQueue",
        ))
        .stdout(predicate::str::contains("contracts/VaultExCore.sol"));

    Ok(())
}

// Chain 97 0x1a9ea54ad1cf25ee0489c0c9996d5d8db166f524; rejected from mode 2 up. Mode 1 was rejected
// before recursive spills and compiles since.
#[cfg(feature = "solc")]
#[test]
fn recursive_stack_too_deep_level_matrix() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("recursive_stack_too_deep_level_matrix.json"),
    ];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains(
            "Stackification failed for '_placeInLevelMatrix",
        ))
        .stdout(predicate::str::contains("contracts/DualMatrixSystem.sol"));

    Ok(())
}

// Chain 56 0x699a152689c193ac9c6297afdd66785fff9f7aef; rejected only at mode 1.
#[cfg(feature = "solc")]
#[test]
fn recursive_stack_too_deep_pair_tax() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("recursive_stack_too_deep_pair_tax.json"),
    ];

    let result = crate::cli::execute_solx(args)?;

    result
        .success()
        .stdout(predicate::str::contains(
            "Stackification failed for '_takePairTax",
        ))
        .stdout(predicate::str::contains("2026/BscLpHoldingTaxToken.sol"));

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
