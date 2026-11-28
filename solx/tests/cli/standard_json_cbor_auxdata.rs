//!
//! CLI tests for the `cborAuxdata` standard JSON output.
//!

#[test]
fn lists_every_trailer() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_evm_bytecode_cbor_auxdata.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    let output: serde_json::Value =
        serde_json::from_slice(result.success().get_output().stdout.as_slice())?;

    // The entries of every (contract, segment) pair the fixture produces, as
    // (owner, entry count): `Factory`'s constructor and `create` each create
    // `Child`, and `spawn` creates `Other`.
    let expected = [
        ("Child", "bytecode", vec![("A:Child", 1)]),
        ("Child", "deployedBytecode", vec![("A:Child", 1)]),
        ("Other", "bytecode", vec![("A:Other", 1)]),
        ("Other", "deployedBytecode", vec![("A:Other", 1)]),
        (
            "Factory",
            "bytecode",
            vec![("A:Child", 2), ("A:Factory", 1), ("A:Other", 1)],
        ),
        (
            "Factory",
            "deployedBytecode",
            vec![("A:Child", 1), ("A:Factory", 1), ("A:Other", 1)],
        ),
    ];
    for (name, segment, owners) in expected.iter() {
        let auxdata = output["contracts"]["A"][name]["evm"][segment]["cborAuxdata"]
            .as_object()
            .expect("the CBOR auxdata is an object");
        assert_eq!(auxdata.len(), owners.len(), "{name} in {segment}");
        for (owner, count) in owners.iter() {
            let entries = auxdata[*owner]
                .as_array()
                .expect("the entries are an array");
            assert_eq!(entries.len(), *count, "{owner} in {name} {segment}");
            let offsets: Vec<u64> = entries
                .iter()
                .map(|entry| entry["offset"].as_u64().expect("the offset is a number"))
                .collect();
            assert!(
                offsets.is_sorted_by(|first, second| first < second),
                "{owner} offsets ascend in {name} {segment}"
            );
        }
    }

    Ok(())
}

#[test]
fn own_trailer_ends_the_runtime_code() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_evm_bytecode_cbor_auxdata.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    let output: serde_json::Value =
        serde_json::from_slice(result.success().get_output().stdout.as_slice())?;

    for name in ["Child", "Other", "Factory"] {
        let deployed_bytecode = &output["contracts"]["A"][name]["evm"]["deployedBytecode"];
        let code = hex::decode(
            deployed_bytecode["object"]
                .as_str()
                .expect("the object is a string"),
        )?;
        let entry = deployed_bytecode["cborAuxdata"][format!("A:{name}")][0]
            .as_object()
            .expect("the own trailer is reported");
        let offset = entry["offset"].as_u64().expect("the offset is a number") as usize;
        let value = hex::decode(
            entry["value"]
                .as_str()
                .expect("the value is a string")
                .trim_start_matches("0x"),
        )?;
        assert_eq!(offset + value.len(), code.len(), "{name}");
    }

    Ok(())
}

#[test]
fn bytes_at_offset_match() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_evm_bytecode_cbor_auxdata.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    let output: serde_json::Value =
        serde_json::from_slice(result.success().get_output().stdout.as_slice())?;

    let mut checked = 0;
    for contract in output["contracts"]["A"]
        .as_object()
        .expect("the contracts are an object")
        .values()
    {
        for segment in ["bytecode", "deployedBytecode"] {
            let bytecode = &contract["evm"][segment];
            let code = hex::decode(bytecode["object"].as_str().expect("the object is a string"))?;
            let auxdata = bytecode["cborAuxdata"]
                .as_object()
                .expect("the CBOR auxdata is an object");
            for (owner, entries) in auxdata {
                for entry in entries.as_array().expect("the entries are an array") {
                    let offset = entry["offset"].as_u64().expect("the offset is a number") as usize;
                    let value = hex::decode(
                        entry["value"]
                            .as_str()
                            .expect("the value is a string")
                            .trim_start_matches("0x"),
                    )?;
                    assert_eq!(
                        &code[offset..offset + value.len()],
                        value.as_slice(),
                        "{owner} at {offset} in {segment}"
                    );
                    let length_suffix =
                        u16::from_be_bytes([value[value.len() - 2], value[value.len() - 1]]);
                    assert_eq!(length_suffix as usize, value.len() - 2, "{owner}");
                    checked += 1;
                }
            }
        }
    }
    assert_eq!(checked, 11);

    Ok(())
}

#[test]
fn selected_alone() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_evm_bytecode_cbor_auxdata.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    let output: serde_json::Value =
        serde_json::from_slice(result.success().get_output().stdout.as_slice())?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_evm_deployed_bytecode_cbor_auxdata_only.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    let output_alone: serde_json::Value =
        serde_json::from_slice(result.success().get_output().stdout.as_slice())?;

    for name in ["Child", "Other", "Factory"] {
        let deployed_bytecode = &output_alone["contracts"]["A"][name]["evm"]["deployedBytecode"];
        assert_eq!(deployed_bytecode["object"], serde_json::Value::Null);
        assert_eq!(
            deployed_bytecode["cborAuxdata"],
            output["contracts"]["A"][name]["evm"]["deployedBytecode"]["cborAuxdata"],
            "{name}"
        );
    }

    Ok(())
}

#[test]
fn kept_with_bytecode_hash_none() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_evm_deployed_bytecode_cbor_auxdata_no_hash.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    let output: serde_json::Value =
        serde_json::from_slice(result.success().get_output().stdout.as_slice())?;

    let auxdata = output["contracts"]["A"]["Factory"]["evm"]["deployedBytecode"]["cborAuxdata"]
        .as_object()
        .expect("the CBOR auxdata is an object");
    assert_eq!(auxdata.len(), 3);
    let mut offsets = Vec::with_capacity(3);
    let mut values = Vec::with_capacity(3);
    for owner in ["A:Child", "A:Factory", "A:Other"] {
        let entries = auxdata[owner].as_array().expect("the entries are an array");
        assert_eq!(entries.len(), 1, "{owner}");
        offsets.push(
            entries[0]["offset"]
                .as_u64()
                .expect("the offset is a number"),
        );
        values.push(entries[0]["value"].as_str().expect("the value is a string"));
    }
    offsets.sort_unstable();
    offsets.dedup();
    assert_eq!(offsets.len(), 3, "every trailer has its own offset");
    assert!(
        values.windows(2).all(|window| window[0] == window[1]),
        "hashless trailers are byte-identical, told apart by owner and offset"
    );

    Ok(())
}

#[test]
fn empty_with_append_cbor_false() -> anyhow::Result<()> {
    crate::common::setup()?;

    let args = &[
        "--standard-json",
        crate::common::standard_json!("select_evm_deployed_bytecode_cbor_auxdata_no_cbor.json"),
    ];

    let result = crate::cli::execute_solx(args)?;
    let output: serde_json::Value =
        serde_json::from_slice(result.success().get_output().stdout.as_slice())?;

    for name in ["Child", "Other", "Factory"] {
        assert_eq!(
            output["contracts"]["A"][name]["evm"]["deployedBytecode"]["cborAuxdata"],
            serde_json::json!({}),
            "{name}"
        );
    }

    Ok(())
}
