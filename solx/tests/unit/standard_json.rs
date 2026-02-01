//!
//! Unit tests for standard JSON for all supported languages.
//!

use std::path::PathBuf;

#[test]
fn standard_json_yul_solc() {
    let solc_input = solx_standard_json::Input::try_from(Some(
        PathBuf::from("tests/data/standard_json_input/yul.json").as_path(),
    ))
    .expect("Standard JSON reading error");
    let solc_output = crate::common::build_yul_standard_json(solc_input).expect("Test failure");

    assert!(
        !solc_output
            .contracts
            .get("Test")
            .expect("The `Test` contract is missing")
            .get("Return")
            .expect("The `Return` contract is missing")
            .evm
            .as_ref()
            .expect("The `evm` field is missing")
            .bytecode
            .as_ref()
            .expect("The `bytecode` field is missing")
            .object
            .as_ref()
            .expect("The `object` field is missing")
            .is_empty()
    )
}

#[test]
fn standard_json_yul_solc_validated() {
    let solc_input = solx_standard_json::Input::try_from(Some(
        PathBuf::from("tests/data/standard_json_input/yul.json").as_path(),
    ))
    .expect("Standard JSON reading error");

    let solc_output = crate::common::build_yul_standard_json(solc_input).expect("Test failure");

    assert!(
        !solc_output
            .contracts
            .get("Test")
            .expect("The `Test` contract is missing")
            .get("Return")
            .expect("The `Return` contract is missing")
            .evm
            .as_ref()
            .expect("The `evm` field is missing")
            .bytecode
            .as_ref()
            .expect("The `bytecode` field is missing")
            .object
            .as_ref()
            .expect("The `object` field is missing")
            .is_empty()
    )
}

#[test]
fn standard_json_yul_solc_urls() {
    let solc_input = solx_standard_json::Input::try_from(Some(
        PathBuf::from("tests/data/standard_json_input/yul_urls.json").as_path(),
    ))
    .expect("Standard JSON reading error");
    let solc_output = crate::common::build_yul_standard_json(solc_input).expect("Test failure");

    assert!(
        !solc_output
            .contracts
            .get("Test")
            .expect("The `Test` contract is missing")
            .get("Return")
            .expect("The `Return` contract is missing")
            .evm
            .as_ref()
            .expect("The `evm` field is missing")
            .bytecode
            .as_ref()
            .expect("The `bytecode` field is missing")
            .object
            .as_ref()
            .expect("The `object` field is missing")
            .is_empty()
    )
}

#[test]
fn standard_json_yul_solc_urls_validated() {
    let solc_input = solx_standard_json::Input::try_from(Some(
        PathBuf::from("tests/data/standard_json_input/yul_urls.json").as_path(),
    ))
    .expect("Standard JSON reading error");

    let solc_output = crate::common::build_yul_standard_json(solc_input).expect("Test failure");

    assert!(
        !solc_output
            .contracts
            .get("Test")
            .expect("The `Test` contract is missing")
            .get("Return")
            .expect("The `Return` contract is missing")
            .evm
            .as_ref()
            .expect("The `evm` field is missing")
            .bytecode
            .as_ref()
            .expect("The `bytecode` field is missing")
            .object
            .as_ref()
            .expect("The `object` field is missing")
            .is_empty()
    )
}

#[test]
fn standard_json_llvm_ir_urls() {
    let solc_input = solx_standard_json::Input::try_from(Some(
        PathBuf::from("tests/data/standard_json_input/llvm_ir_urls.json").as_path(),
    ))
    .expect("Standard JSON reading error");
    let solc_output = crate::common::build_llvm_ir_standard_json(solc_input).expect("Test failure");

    assert!(
        !solc_output
            .contracts
            .get("Test")
            .expect("The `Test` contract is missing")
            .get("Test")
            .expect("The `Test` contract is missing")
            .evm
            .as_ref()
            .expect("The `evm` field is missing")
            .bytecode
            .as_ref()
            .expect("The `bytecode` field is missing")
            .object
            .as_ref()
            .expect("The `object` field is missing")
            .is_empty()
    )
}
