//!
//! The `solc --standard-json` output.
//!

pub mod contract;
pub mod error;
pub mod source;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;

use crate::input::settings::selection::selector::Selector as InputSettingsSelector;
use crate::input::settings::selection::Selection as InputSettingsSelection;
use crate::input::source::Source as InputSource;

use self::contract::Contract;
use self::error::collectable::Collectable as CollectableError;
use self::error::source_location::SourceLocation as JsonOutputErrorSourceLocation;
use self::error::Error as JsonOutputError;
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
        for (path, file) in self.contracts.iter_mut() {
            for (name, contract) in file.iter_mut() {
                if !output_selection.check_selection(
                    path.as_str(),
                    Some(name.as_str()),
                    InputSettingsSelector::Yul,
                ) {
                    contract.ir = None;
                }
                if let Some(ref mut evm) = contract.evm {
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
    pub fn push_error(&mut self, path: Option<String>, error: anyhow::Error) {
        self.errors.push(JsonOutputError::new_error_with_data(
            None,
            error,
            path.map(JsonOutputErrorSourceLocation::new),
            None,
        ));
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
