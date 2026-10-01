//!
//! CLI tests for the objects each MLIR code segment may embed.
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
        mlir.deploy_dependencies.runtime.as_deref(),
        Some("creation.sol:C_deployed"),
        "the deploy segment returns its own runtime object"
    );
    assert_eq!(
        mlir.deploy_dependencies.objects,
        ["creation.sol:A", "creation.sol:A_deployed"],
        "the deploy segment may embed what its creation code creates"
    );
    assert_eq!(
        mlir.runtime_dependencies.runtime, None,
        "a runtime segment has no runtime child"
    );
    assert_eq!(
        mlir.runtime_dependencies.objects,
        ["creation.sol:B", "creation.sol:B_deployed"],
        "the runtime segment may embed only what its own functions create"
    );

    Ok(())
}
