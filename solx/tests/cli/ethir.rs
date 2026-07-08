//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_SOLIDITY_CONTRACT, "--ethir", "--bin"];

    let result = crate::cli::execute_solx(args)?;

    // Guard the structure of the Ethereal IR dump: both segments, entry-function
    // and per-segment block recovery, the stack-usage line, and rendered stacks.
    result
        .success()
        .stdout(predicate::str::contains("Deploy Ethereal IR:"))
        .stdout(predicate::str::contains("Runtime Ethereal IR:"))
        .stdout(predicate::str::contains("function __entry {"))
        .stdout(predicate::str::contains("stack_usage:"))
        .stdout(predicate::str::contains("block_dt_"))
        .stdout(predicate::str::contains("block_rt_"))
        .stdout(predicate::str::contains(" - ["))
        .stdout(predicate::str::contains(" + ["));

    Ok(())
}
