//!
//! CLI tests for the eponymous option.
//!

use predicates::prelude::*;

#[test]
fn default() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_SOLIDITY_CONTRACT, "--ethir", "--bin"];

    let result = crate::cli::execute_solx(args)?;

    // Tier 1: predecessor-invariant structure -- green on both main and the PR.
    // Both segments dump with recovered functions and blocks, and the per-element
    // stacks render, exercising the `capture_stacks` reconstruction path. The two
    // segments carry only their own blocks (`dt_` vs `rt_`), and `stack_usage` /
    // ` + [` cover the `finalize` size and the reconstructed stack output.
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

/// Tier 2: locks the predecessor-attachment fix (PR commit `c306640f`). When a block
/// key has multiple instances, the incoming predecessor must attach to the instance
/// whose initial stack matches the visited hash -- not to the last-created instance.
/// This assertion is expected to be red on `main` (pre-fix) and green on the PR.
///
/// Stub: the exact `block_<seg>_<tag>/<instance>: (predecessors: <key>/<inst>)` line
/// the fix corrects depends on instance numbering that can only be read from the
/// generated dump. Fill from the first CI `--ethir` artifact, then drop `#[ignore]`.
#[test]
#[ignore = "stub: fill the predecessor assertion from the first CI --ethir dump artifact"]
fn predecessor_attaches_to_matching_instance() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[crate::common::TEST_SOLIDITY_CONTRACT, "--ethir"];

    let _result = crate::cli::execute_solx(args)?;
    // TODO: pin the corrected predecessor line, e.g.
    // .stdout(predicate::str::contains(
    //     "block_rt_<tag>/1: (predecessors: rt_<key>/<inst>)",
    // ));

    Ok(())
}
