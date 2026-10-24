//!
//! CLI tests for the `immutableReferences` standard JSON output.
//!

use std::collections::BTreeMap;

#[test]
fn keyed_by_node_id() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_evm_deployed_bytecode_immutable_references.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    let result = result.success();
    let output: serde_json::Value = serde_json::from_slice(result.get_output().stdout.as_slice())?;
    let identifiers = state_variable_identifiers(&output["sources"]["A"]["ast"]);
    let expected: BTreeMap<String, usize> = [("a", 2), ("owner_address", 1)]
        .into_iter()
        .map(|(name, count)| (identifiers[name].to_string(), count))
        .collect();
    let counts: BTreeMap<String, usize> = output["contracts"]["A"]["C"]["evm"]["deployedBytecode"]
        ["immutableReferences"]
        .as_object()
        .expect("the immutable references are an object")
        .iter()
        .map(|(key, references)| {
            let references = references.as_array().expect("the references are an array");
            (key.to_owned(), references.len())
        })
        .collect();
    assert_eq!(counts, expected);

    Ok(())
}

#[test]
fn points_at_push32_operands() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_evm_deployed_bytecode_immutable_references.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    let result = result.success();
    let output: serde_json::Value = serde_json::from_slice(result.get_output().stdout.as_slice())?;
    let deployed_bytecode = &output["contracts"]["A"]["C"]["evm"]["deployedBytecode"];
    let code = hex::decode(
        deployed_bytecode["object"]
            .as_str()
            .expect("the object is a string"),
    )?;
    let immutable_references = deployed_bytecode["immutableReferences"]
        .as_object()
        .expect("the immutable references are an object");
    assert!(!immutable_references.is_empty());
    for (key, references) in immutable_references {
        for reference in references.as_array().expect("the references are an array") {
            let start = reference["start"].as_u64().expect("the start is a number") as usize;
            assert_eq!(reference["length"], 32, "{key} at {start}");
            assert_eq!(code[start - 1], 0x7f, "{key} at {start}");
            assert_eq!(code[start..start + 32], [0; 32], "{key} at {start}");
        }
    }

    Ok(())
}

#[test]
fn library_address() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_evm_deployed_bytecode_immutable_references.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    let result = result.success();
    let output: serde_json::Value = serde_json::from_slice(result.get_output().stdout.as_slice())?;
    let immutable_references =
        output["contracts"]["A"]["L"]["evm"]["deployedBytecode"]["immutableReferences"]
            .as_object()
            .expect("the immutable references are an object");
    assert_eq!(immutable_references.len(), 1);
    assert_eq!(
        immutable_references["library_deploy_address"]
            .as_array()
            .expect("the references are an array")
            .len(),
        1
    );

    Ok(())
}

#[test]
fn empty_without_code() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_evm_deployed_bytecode_immutable_references.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    let result = result.success();
    let output: serde_json::Value = serde_json::from_slice(result.get_output().stdout.as_slice())?;
    assert_eq!(
        output["contracts"]["A"]["I"]["evm"]["deployedBytecode"]["immutableReferences"],
        serde_json::json!({})
    );

    Ok(())
}

#[test]
fn selected_alone() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_evm_deployed_bytecode_immutable_references.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    let result = result.success();
    let output: serde_json::Value = serde_json::from_slice(result.get_output().stdout.as_slice())?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!(
            "select_evm_deployed_bytecode_immutable_references_only.json"
        ),
    ];

    let result = crate::cli::execute_solx(args)?;
    let result = result.success();
    let output_alone: serde_json::Value =
        serde_json::from_slice(result.get_output().stdout.as_slice())?;
    let deployed_bytecode = &output_alone["contracts"]["A"]["C"]["evm"]["deployedBytecode"];
    assert_eq!(deployed_bytecode["object"], serde_json::Value::Null);
    assert_eq!(
        deployed_bytecode["immutableReferences"],
        output["contracts"]["A"]["C"]["evm"]["deployedBytecode"]["immutableReferences"]
    );

    Ok(())
}

/// The node id of each state variable the `ast` declares, by name.
fn state_variable_identifiers(ast: &serde_json::Value) -> BTreeMap<String, u64> {
    let mut identifiers = BTreeMap::new();
    let mut nodes = vec![ast];
    while let Some(node) = nodes.pop() {
        match node {
            serde_json::Value::Object(object) => {
                if object
                    .get("type")
                    .is_some_and(|kind| kind == "StateVariableDefinition")
                {
                    identifiers.insert(
                        object["name"]["text"]
                            .as_str()
                            .expect("the name is a string")
                            .to_owned(),
                        object["id"].as_u64().expect("the id is a number"),
                    );
                }
                nodes.extend(object.values());
            }
            serde_json::Value::Array(array) => nodes.extend(array),
            _ => {}
        }
    }
    identifiers
}
