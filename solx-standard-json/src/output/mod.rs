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
#[derive(Debug, serde::Serialize, serde::Deserialize)]
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
            errors: messages.lock().expect("Sync").drain(..).collect(),
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

        serde_json::to_writer(std::io::stdout(), &self).expect("Stdout writing error");
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
    /// Extracts the debug info from all source code files.
    ///
    pub fn get_debug_info(&self, sources: &BTreeMap<String, InputSource>) -> solx_utils::DebugInfo {
        let mut contract_definitions: HashMap<String, solx_utils::DebugInfoContractDefinition> =
            HashMap::new();
        let mut function_definitions: HashMap<usize, solx_utils::DebugInfoFunctionDefinition> =
            HashMap::new();
        let mut ast_nodes: HashMap<usize, solx_utils::DebugInfoAstNode> = HashMap::new();

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

        solx_utils::DebugInfo::new(contract_definitions, function_definitions, ast_nodes)
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
