//!
//! Output extraction helpers for the test harness.
//!

use std::collections::BTreeMap;
use std::collections::HashMap;

use solx_standard_json::Output;
use solx_standard_json::OutputError;
use solx_standard_json::output::source::Source;
use solx_utils::ContractName;

use crate::compilers::solidity::slang_ast::SlangAst;

///
/// Extracts method identifiers from all contracts in the output.
///
pub fn get_method_identifiers(
    output: &Output,
) -> anyhow::Result<BTreeMap<String, BTreeMap<String, u32>>> {
    let mut method_identifiers = BTreeMap::new();
    for (path, contracts) in output.contracts.iter() {
        for (name, contract) in contracts.iter() {
            let contract_method_identifiers = match contract
                .evm
                .as_ref()
                .and_then(|evm| evm.method_identifiers.as_ref())
            {
                Some(method_identifiers) => method_identifiers,
                None => continue,
            };
            let mut contract_identifiers = BTreeMap::new();
            for (entry, selector) in contract_method_identifiers.iter() {
                let selector = u32::from_str_radix(selector, solx_utils::BASE_HEXADECIMAL)
                    .map_err(|error| {
                        anyhow::anyhow!(
                            "Invalid selector `{selector}` from the Solidity compiler: {error}"
                        )
                    })?;
                contract_identifiers.insert(entry.clone(), selector);
            }
            method_identifiers.insert(ContractName::full_path(path, name), contract_identifiers);
        }
    }
    Ok(method_identifiers)
}

///
/// Gets the last contract from the output for the given language.
///
pub fn get_last_contract(output: &Output, sources: &[(String, String)]) -> anyhow::Result<String> {
    if output.sources.is_empty() {
        anyhow::bail!("The sources are empty. Found errors: {:?}", output.errors);
    }
    for (path, _source) in sources.iter().rev() {
        let Some(source) = output.sources.get(path) else {
            continue;
        };
        if let Ok(name) = last_contract_name(source) {
            return Ok(ContractName::full_path(path, name.as_str()));
        }
    }

    if let Some(full_path) = SlangAst::parse(sources).last_deployable(&output.contracts) {
        return Ok(full_path);
    }

    anyhow::bail!("The last contract not found in the output")
}

///
/// Extracts bytecode builds (deploy code + runtime size) from all contracts.
///
pub fn extract_bytecode_builds(
    output: &Output,
) -> anyhow::Result<HashMap<String, (Vec<u8>, usize)>> {
    if output.contracts.is_empty() {
        anyhow::bail!("Contracts not found in the output");
    }

    let mut builds = HashMap::with_capacity(output.contracts.len());
    for (file, source) in output.contracts.iter() {
        for (name, contract) in source.iter() {
            let path = ContractName::full_path(file, name);
            let deploy_code = match contract
                .evm
                .as_ref()
                .and_then(|evm| evm.bytecode.as_ref())
                .and_then(|bytecode| bytecode.object.as_ref())
            {
                Some(bytecode) => hex::decode(bytecode.as_str()).map_err(|error| {
                    anyhow::anyhow!("EVM bytecode of the contract `{path}` is invalid: {error}")
                })?,
                None => continue,
            };
            let runtime_code_size = contract
                .evm
                .as_ref()
                .and_then(|evm| evm.deployed_bytecode.as_ref())
                .and_then(|deployed_bytecode| deployed_bytecode.object.as_ref())
                .map(|object| object.len() / 2)
                .unwrap_or(0);
            builds.insert(path, (deploy_code, runtime_code_size));
        }
    }
    Ok(builds)
}

///
/// Returns errors as Option slice, None if empty.
///
pub fn errors_opt(output: &Output) -> Option<&[OutputError]> {
    if output.errors.is_empty() {
        None
    } else {
        Some(&output.errors)
    }
}

///
/// Returns the name of the last contract in the AST.
///
fn last_contract_name(source: &Source) -> anyhow::Result<String> {
    source
        .ast
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("The AST is empty"))?
        .get("nodes")
        .and_then(|value| value.as_array())
        .ok_or_else(|| {
            anyhow::anyhow!("The last contract cannot be found in an empty list of nodes")
        })?
        .iter()
        .filter_map(
            |node| match node.get("nodeType").and_then(|node| node.as_str()) {
                Some("ContractDefinition") => Some(node.get("name")?.as_str()?.to_owned()),
                _ => None,
            },
        )
        .next_back()
        .ok_or_else(|| anyhow::anyhow!("The last contract not found in the AST"))
}
