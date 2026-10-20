//!
//! CLI tests for standard JSON debug settings.
//!

use predicates::prelude::*;

/// The first word of `Ether sent to non-payable function`, the callvalue guard's message under
/// debug revert strings.
const CALLVALUE_GUARD_MESSAGE: &str =
    "45746865722073656e7420746f206e6f6e2d70617961626c652066756e637469";

#[test]
fn revert_strings_name_the_callvalue_guard() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("debug_revert_strings.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    result
        .success()
        .stdout(predicate::str::contains(CALLVALUE_GUARD_MESSAGE));

    Ok(())
}
