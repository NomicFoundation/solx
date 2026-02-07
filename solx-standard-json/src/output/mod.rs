//!
//! The `solc --standard-json` output.
//!

pub mod contract;
pub mod error;
pub mod source;

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use solx_utils::SyncLock;

use crate::input::language::Language as InputLanguage;
use crate::input::settings::selection::Selection as InputSettingsSelection;
use crate::input::settings::selection::selector::Selector as InputSettingsSelector;
use crate::input::source::Source as InputSource;

use self::contract::Contract;
use self::error::Error as JsonOutputError;
use self::error::collectable::Collectable as CollectableError;
use self::source::Source;

///
/// The `solc --standard-json` output.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Output {
    /// File-contract mapping.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub contracts: BTreeMap<String, BTreeMap<String, Contract>>,
    /// Source code mapping data.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sources: BTreeMap<String, Source>,
    /// Compilation errors and warnings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<JsonOutputError>,
    /// Compilation pipeline benchmarks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub benchmarks: Vec<(String, u64)>,
}

impl Output {
    ///
    /// Initializes a standard JSON output.
    ///
    /// Is used for projects compiled without `solc`.
    ///
    pub fn new(sources: &BTreeMap<String, InputSource>) -> Self {
        let sources = sources
            .keys()
            .enumerate()
            .map(|(index, path)| (path.to_owned(), Source::new(index)))
            .collect::<BTreeMap<String, Source>>();

        Self {
            contracts: BTreeMap::new(),
            sources,
            errors: Vec::new(),
            benchmarks: Vec::new(),
        }
    }

    ///
    /// Initializes a standard JSON output with messages.
    ///
    /// Is used to emit errors in standard JSON mode.
    ///
    pub fn new_with_messages(messages: Arc<Mutex<Vec<JsonOutputError>>>) -> Self {
        Self {
            contracts: BTreeMap::new(),
            sources: BTreeMap::new(),
            errors: messages.lock_sync().drain(..).collect(),
            benchmarks: Vec::new(),
        }
    }

    ///
    /// Prunes the output JSON and prints it to stdout.
    ///
    pub fn write_and_exit(mut self, output_selection: &InputSettingsSelection) -> ! {
        for (path, source) in self.sources.iter_mut() {
            if !output_selection.check_selection(path.as_str(), None, InputSettingsSelector::AST) {
                source.ast = None;
            }
        }
        for (path, file) in self.contracts.iter_mut() {
            for (name, contract) in file.iter_mut() {
                if !output_selection.check_selection(
                    path.as_str(),
                    Some(name.as_str()),
                    InputSettingsSelector::Yul,
                ) {
                    contract.ir = None;
                }
                if let Some(evm) = contract.evm.as_mut() {
                    if !output_selection.check_selection(
                        path.as_str(),
                        Some(name.as_str()),
                        InputSettingsSelector::EVMLegacyAssembly,
                    ) {
                        evm.legacy_assembly = None;
                    }
                    if evm
                        .bytecode
                        .as_ref()
                        .map(|bytecode| bytecode.is_empty())
                        .unwrap_or(true)
                    {
                        evm.bytecode = None;
                    }
                    if evm
                        .deployed_bytecode
                        .as_ref()
                        .map(|bytecode| bytecode.is_empty())
                        .unwrap_or(true)
                    {
                        evm.deployed_bytecode = None;
                    }
                }
                if contract
                    .evm
                    .as_ref()
                    .map(|evm| evm.is_empty())
                    .unwrap_or(true)
                {
                    contract.evm = None;
                }
            }
        }

        self.contracts.retain(|_, contracts| {
            contracts.retain(|_, contract| !contract.is_empty());
            !contracts.is_empty()
        });

        let mut stdout = std::io::stdout().lock();
        serde_json::to_writer(&mut stdout, &self).expect("Stdout writing error");
        std::io::Write::flush(&mut stdout).expect("Stdout flush error");
        std::process::exit(solx_utils::EXIT_CODE_SUCCESS);
    }

    ///
    /// Pushes an arbitrary error with path.
    ///
    /// Please do not push project-general errors without paths here.
    ///
    pub fn push_error(&mut self, path: &str, error: anyhow::Error) {
        self.errors
            .push(JsonOutputError::new_error_contract(Some(path), error));
    }

    ///
    /// Returns the contracts as Option, returning None if empty.
    ///
    /// This is a convenience method for code that checks for the presence of contracts.
    ///
    pub fn contracts_opt(&self) -> Option<&BTreeMap<String, BTreeMap<String, Contract>>> {
        if self.contracts.is_empty() {
            None
        } else {
            Some(&self.contracts)
        }
    }

    ///
    /// Returns the sources as Option, returning None if empty.
    ///
    /// This is a convenience method for code that checks for the presence of sources.
    ///
    pub fn sources_opt(&self) -> Option<&BTreeMap<String, Source>> {
        if self.sources.is_empty() {
            None
        } else {
            Some(&self.sources)
        }
    }

    ///
    /// Returns the errors as Option slice, returning None if empty.
    ///
    /// This is a convenience method for code that checks for the presence of errors.
    ///
    pub fn errors_opt(&self) -> Option<&[JsonOutputError]> {
        if self.errors.is_empty() {
            None
        } else {
            Some(&self.errors)
        }
    }

    ///
    /// Extracts the debug info from all source code files.
    ///
    pub fn get_debug_info(&self, sources: &BTreeMap<String, InputSource>) -> solx_utils::DebugInfo {
        let mut contract_definitions: HashMap<String, solx_utils::DebugInfoContractDefinition> =
            HashMap::new();
        let mut function_definitions: HashMap<usize, solx_utils::DebugInfoFunctionDefinition> =
            HashMap::new();
        let mut ast_nodes: HashMap<usize, solx_utils::DebugInfoAstNode> = HashMap::new();

        // Build source_id -> path mapping
        let source_ids: BTreeMap<usize, String> = self
            .sources
            .iter()
            .map(|(path, source)| (source.id, path.clone()))
            .collect();

        for (path, source) in self.sources.iter() {
            if let Some(ref ast_json) = source.ast {
                contract_definitions.extend(Source::get_ast_nodes(
                    &Source::contract_definition,
                    path.as_str(),
                    ast_json,
                    sources,
                ));

                function_definitions.extend(Source::get_ast_nodes(
                    &Source::function_definition,
                    path.as_str(),
                    ast_json,
                    sources,
                ));

                ast_nodes.extend(Source::get_ast_nodes(
                    &Source::ast_node,
                    path.as_str(),
                    ast_json,
                    sources,
                ));
            }
        }

        solx_utils::DebugInfo::new(
            contract_definitions,
            function_definitions,
            ast_nodes,
            source_ids,
        )
    }

    ///
    /// Extracts method identifiers from all contracts.
    ///
    /// Returns a map of `path:name` to a map of method signatures to selectors.
    ///
    pub fn get_method_identifiers(
        &self,
    ) -> anyhow::Result<BTreeMap<String, BTreeMap<String, u32>>> {
        let mut method_identifiers = BTreeMap::new();
        for (path, contracts) in self.contracts.iter() {
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
    /// For Solidity, finds the last contract definition in the AST of the last source file.
    /// For Yul, returns the first contract in the output.
    ///
    pub fn get_last_contract(
        &self,
        language: InputLanguage,
        sources: &[(String, String)],
    ) -> anyhow::Result<String> {
        match language {
            InputLanguage::Solidity => {
                let output_sources = self.sources_opt().ok_or_else(|| {
                    anyhow::anyhow!("The sources are empty. Found errors: {:?}", self.errors)
                })?;
                for (path, _source) in sources.iter().rev() {
                    let Some(source) = output_sources.get(path) else {
                        continue;
                    };
                    match source.last_contract_name() {
                        Ok(name) => return Ok(format!("{path}:{name}")),
                        Err(_error) => continue,
                    }
                }
                anyhow::bail!("The last contract not found in the output")
            }
            InputLanguage::Yul => self
                .contracts_opt()
                .and_then(|contracts| contracts.first_key_value())
                .and_then(|(path, contracts)| {
                    contracts
                        .first_key_value()
                        .map(|(name, _contract)| format!("{path}:{name}"))
                })
                .ok_or_else(|| {
                    anyhow::anyhow!("The sources are empty. Found errors: {:?}", self.errors)
                }),
            InputLanguage::LLVMIR => {
                anyhow::bail!("LLVM IR language is not supported")
            }
        }
    }

    ///
    /// Extracts bytecode builds from all contracts.
    ///
    /// Returns a HashMap mapping `path:name` to `(deploy_code, runtime_code_size)`.
    ///
    pub fn extract_bytecode_builds(&self) -> anyhow::Result<HashMap<String, (Vec<u8>, usize)>> {
        let contracts = self
            .contracts_opt()
            .ok_or_else(|| anyhow::anyhow!("Contracts not found in the output"))?;

        let mut builds = HashMap::with_capacity(contracts.len());
        for (file, source) in contracts.iter() {
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
}

impl CollectableError for Output {
    fn error_strings(&self) -> Vec<String> {
        self.errors
            .iter()
            .filter_map(|error| {
                if error.severity == "error" {
                    Some(error.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    fn take_warnings(&mut self) -> Vec<JsonOutputError> {
        self.errors
            .extract_if(.., |message| message.severity == "warning")
            .collect()
    }

    fn has_errors(&self) -> bool {
        self.errors.iter().any(|error| error.severity == "error")
    }
}
