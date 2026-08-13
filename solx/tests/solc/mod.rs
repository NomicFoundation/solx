//!
//! The `solc` frontend tests.
//!

use solx_core::Frontend;

///
/// `standard_json` narrows `output_selection` and clears the optimizer mode for `solc`,
/// then restores both once `solc` returns. An early return between the two leaves the
/// caller's input carrying solc's selection instead of its own. An interior NUL byte in
/// the base path fails after the mutation without entering `solc`, so the restore is
/// pinned without linking `solc` behavior into the assertion.
///
#[test]
fn standard_json_restores_input_settings_on_error() -> anyhow::Result<()> {
    crate::common::setup()?;

    let mut input = solx_standard_json::Input::try_from(Some(std::path::Path::new(
        crate::common::standard_json!("solidity.json"),
    )))?;
    let original = serde_json::to_value(&input)?;

    let error = solx::Solc::default()
        .standard_json(&mut input, true, Some("abc\0def"), &[], None)
        .expect_err("a base path with an interior NUL byte is rejected");

    assert!(error.to_string().contains("solc base path CString"));
    assert_eq!(serde_json::to_value(&input)?, original);

    Ok(())
}
