//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_SOLIDITY_CONTRACT, "--evmla", "--bin"];

    let result = crate::cli::execute_solx(args)?;

    #[cfg(feature = "solc")]
    result
        .success()
        .stdout(predicate::str::contains("Deploy EVM legacy assembly:"))
        .stdout(predicate::str::contains("PUSH"));
    // The bytecode still comes out; only the solc-pipeline dump is missing.
    #[cfg(not(feature = "solc"))]
    result
        .success()
        .stdout(predicate::str::contains("Binary:"))
        .stderr(predicate::str::contains("--evmla is ignored"));

    Ok(())
}
