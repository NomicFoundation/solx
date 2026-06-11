//!
//! Output extraction helpers for the test harness.
//!

use std::collections::BTreeMap;
use std::collections::HashMap;

use solx_standard_json::InputLanguage;
use solx_standard_json::Output;
use solx_standard_json::OutputError;

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
            method_identifiers.insert(format!("{path}:{name}"), contract_identifiers);
        }
    }
    Ok(method_identifiers)
}

///
/// Gets the last contract from the output for the given language.
///
pub fn get_last_contract(
    output: &Output,
    language: InputLanguage,
    sources: &[(String, String)],
) -> anyhow::Result<String> {
    match language {
        InputLanguage::Solidity => {
            let output_sources = if output.sources.is_empty() {
                anyhow::bail!("The sources are empty. Found errors: {:?}", output.errors);
            } else {
                &output.sources
            };
            for (path, _source) in sources.iter().rev() {
                let Some(source) = output_sources.get(path) else {
                    continue;
                };
                match last_contract_name(source) {
                    Ok(name) => return Ok(format!("{path}:{name}")),
                    Err(_error) => continue,
                }
            }

            // The Slang frontend produces a CST, not the solc AST, so
            // `last_contract_name` cannot read the contract order off the AST
            // `nodes` array, and the Slang output is a name-sorted map. solc
            // selects the main contract by SOURCE order (the last contract
            // definition), so recover that order from the source text directly:
            // the source-last non-library contract that has an emitted object.
            // Picking the name-sorted last would mis-deploy an alphabetically
            // later contract / library (e.g. `contract X` after the real main
            // `contract B`).
            #[cfg(feature = "slang-ast")]
            {
                let library_names = collect_library_names(sources);
                for (path, source) in sources.iter().rev() {
                    if let Some(contracts) = output.contracts.get(path)
                        && let Some(name) = contract_names_in_source_order(source)
                            .into_iter()
                            .rev()
                            .find(|name| {
                                !library_names.contains(name.as_str())
                                    && contracts.contains_key(name.as_str())
                            })
                    {
                        return Ok(format!("{path}:{name}"));
                    }
                }
                // Library-only sources: fall back to any object.
                for (path, _source) in sources.iter().rev() {
                    if let Some(contracts) = output.contracts.get(path) {
                        if let Some((name, _)) = contracts.last_key_value() {
                            return Ok(format!("{path}:{name}"));
                        }
                    }
                }
            }

            anyhow::bail!("The last contract not found in the output")
        }
        InputLanguage::Yul => {
            if output.contracts.is_empty() {
                anyhow::bail!("The sources are empty. Found errors: {:?}", output.errors);
            }
            output
                .contracts
                .first_key_value()
                .and_then(|(path, contracts)| {
                    contracts
                        .first_key_value()
                        .map(|(name, _contract)| format!("{path}:{name}"))
                })
                .ok_or_else(|| {
                    anyhow::anyhow!("The sources are empty. Found errors: {:?}", output.errors)
                })
        }
        InputLanguage::LLVMIR => {
            anyhow::bail!("LLVM IR language is not supported")
        }
    }
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
            let path = format!("{file}:{name}");
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
fn last_contract_name(
    source: &solx_standard_json::output::source::Source,
) -> anyhow::Result<String> {
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

///
/// Collects the names of all top-level `library` definitions across `sources`.
///
/// The main contract is never a library, but the Slang frontend emits each
/// library as its own object and the Slang output is a name-sorted map — so the
/// fallback main-contract heuristic would otherwise mis-pick an alphabetically
/// later library. This lightweight scan strips `//` line comments and matches
/// `library <identifier>` adjacency, which covers the Solidity semantic-test
/// corpus (the Slang frontend lacks a solc AST to consult instead).
///
///
/// Collects top-level `contract` definition names in SOURCE-declaration order
/// from one source's text, so the main-contract heuristic can pick the
/// source-last contract the way solc does (the Slang output map is name-sorted
/// and carries no order). Matches `contract <identifier>` adjacency — covering
/// `contract C`, `abstract contract C`, and `contract C is B` — after stripping
/// `//` line comments, mirroring [`collect_library_names`]. Libraries and
/// interfaces are not `contract` declarations and are excluded by the keyword.
///
#[cfg(feature = "slang-ast")]
fn contract_names_in_source_order(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in source.lines() {
        let code = line.split("//").next().unwrap_or("");
        let tokens: Vec<&str> = code
            .split(|character: char| {
                !(character.is_alphanumeric() || character == '_' || character == '$')
            })
            .filter(|token| !token.is_empty())
            .collect();
        for window in tokens.windows(2) {
            if window[0] == "contract" {
                names.push(window[1].to_owned());
            }
        }
    }
    names
}

#[cfg(feature = "slang-ast")]
fn collect_library_names(sources: &[(String, String)]) -> std::collections::HashSet<String> {
    let mut names = std::collections::HashSet::new();
    for (_path, source) in sources.iter() {
        for line in source.lines() {
            let code = line.split("//").next().unwrap_or("");
            let tokens: Vec<&str> = code
                .split(|character: char| {
                    !(character.is_alphanumeric() || character == '_' || character == '$')
                })
                .filter(|token| !token.is_empty())
                .collect();
            for window in tokens.windows(2) {
                if window[0] == "library" {
                    names.insert(window[1].to_owned());
                }
            }
        }
    }
    names
}
