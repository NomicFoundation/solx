//!
//! CLI tests for the objects each MLIR code segment references.
//!

#[test]
fn per_code_segment() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::TEST_MLIR_CREATION_STANDARD_JSON,
    ];

    let result = crate::cli::execute_solx(args)?;
    let output = solx_utils::deserialize_from_slice::<solx_standard_json::Output>(
        result.success().get_output().stdout.as_slice(),
    )?;

    let mlir = output.contracts["creation.sol"]["C"]
        .mlir
        .as_ref()
        .expect("the MLIR stage is selected");

    assert_eq!(
        mlir.deploy_dependencies.inner,
        ["creation.sol:C_deployed", "creation.sol:A"],
        "the deploy segment's dependencies come from its own module, not the whole source unit"
    );
    assert_eq!(
        mlir.runtime_dependencies.inner,
        ["creation.sol:B"],
        "the runtime segment references only what its own functions create"
    );

    Ok(())
}
