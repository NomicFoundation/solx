//!
//! The Solidity project build.
//!

pub mod contract;

use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use normpath::PathExt;
use solx_utils::SyncLock;

use solx_standard_json::CollectableError;

use crate::error::Error;

use self::contract::Contract;
use self::contract::object::Object as ContractObject;

///
/// The Solidity project build.
///
#[derive(Debug, Default)]
pub struct Build {
    /// The contract builds,
    pub contracts: BTreeMap<String, Contract>,
    /// The Solidity AST JSONs of the source files.
    pub ast_jsons: Option<BTreeMap<String, Option<serde_json::Value>>>,
    /// The additional message to output.
    pub messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
    /// Compilation pipeline benchmarks.
    pub benchmarks: Vec<(String, u64)>,
}

impl Build {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        contracts: BTreeMap<String, Contract>,
        ast_jsons: Option<BTreeMap<String, Option<serde_json::Value>>>,
        messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
    ) -> Self {
        Self {
            contracts,
            ast_jsons,
            messages,
            benchmarks: Vec::new(),
        }
    }

    ///
    /// Links the EVM build.
    ///
    pub fn link(
        mut self,
        linker_symbols: BTreeMap<String, [u8; solx_utils::BYTE_LENGTH_ETH_ADDRESS]>,
    ) -> Self {
        let ast_jsons = self.ast_jsons.take();

        loop {
            let assembled_objects_data = {
                let all_objects = self
                    .contracts
                    .values()
                    .flat_map(|contract| contract.objects_ref())
                    .collect::<Vec<&ContractObject>>();

                let assembleable_objects = all_objects
                    .iter()
                    .filter(|object| {
                        !object.is_assembled
                            && object.dependencies.inner.iter().all(|dependency| {
                                all_objects
                                    .iter()
                                    .find(|object| {
                                        object.identifier.as_str() == dependency.as_str()
                                    })
                                    .map(|object| object.is_assembled)
                                    .unwrap_or_default()
                            })
                    })
                    .copied()
                    .collect::<Vec<_>>();
                if assembleable_objects.is_empty() {
                    break;
                }

                let mut assembled_objects_data = Vec::with_capacity(assembleable_objects.len());
                for object in assembleable_objects.into_iter() {
                    let assembled_object = match object.assemble(all_objects.as_slice()) {
                        Ok(assembled_object) => assembled_object,
                        Err(error) => {
                            self.messages.lock_sync().push(
                                solx_standard_json::OutputError::new_error_contract(
                                    Some(object.contract_name.path.as_str()),
                                    &error,
                                ),
                            );
                            return Self::new(BTreeMap::new(), ast_jsons, self.messages);
                        }
                    };
                    assembled_objects_data.push((
                        object.contract_name.full_path.to_owned(),
                        object.code_segment,
                        assembled_object,
                    ));
                }
                assembled_objects_data
            };

            for (full_path, code_segment, assembled_object) in assembled_objects_data.into_iter() {
                let contract = self
                    .contracts
                    .get_mut(full_path.as_str())
                    .expect("Always exists");
                let object = match contract.object_mut_by_code_segment(code_segment) {
                    Some(object) => object,
                    None => continue,
                };
                object.bytecode = Some(assembled_object.as_slice().to_owned());
                for undefined_reference in assembled_object
                    .get_undefined_references_evm()
                    .into_iter()
                    .filter(|reference| !linker_symbols.contains_key(reference))
                {
                    let symbol_offsets =
                        assembled_object.get_symbol_offsets_evm(undefined_reference.as_str());
                    object
                        .unlinked_symbols
                        .insert(undefined_reference, symbol_offsets);
                }
                object.is_assembled = true;
            }
        }

        for contract in self.contracts.values_mut() {
            for object in contract.objects_mut().into_iter() {
                if let Err(error) = object.link(&linker_symbols) {
                    self.messages.lock_sync().push(
                        solx_standard_json::OutputError::new_error_contract(
                            Some(object.contract_name.path.as_str()),
                            &error,
                        ),
                    );
                    return Self::new(BTreeMap::new(), ast_jsons, self.messages);
                }
            }
        }

        Self::new(self.contracts, ast_jsons, self.messages)
    }

    ///
    /// Writes all contracts to the terminal.
    ///
    pub fn write_to_terminal(
        mut self,
        output_selection: &solx_standard_json::InputSelection,
    ) -> anyhow::Result<()> {
        self.take_and_write_warnings();
        self.exit_on_error();

        for (path, ast) in self.ast_jsons.unwrap_or_default().into_iter() {
            if output_selection.check_selection(
                path.as_str(),
                None,
                solx_standard_json::InputSelector::AST,
            ) {
                writeln!(std::io::stdout(), "\n======= {path} =======",)?;
                let ast = ast.ok_or_else(|| anyhow::anyhow!("AST JSON is missing for `{path}`"))?;
                writeln!(std::io::stdout(), "JSON AST:\n{}", ast)?;
            }
        }
        if output_selection.check_selection(
            solx_standard_json::InputSelection::WILDCARD,
            Some(solx_standard_json::InputSelection::ANY_CONTRACT),
            solx_standard_json::InputSelector::Benchmarks,
        ) {
            writeln!(std::io::stdout(), "Benchmarks:")?;
            for (name, value) in self.benchmarks.iter() {
                writeln!(std::io::stdout(), "{name}: {value}ms")?;
            }
        }

        for contract in self.contracts.into_values() {
            contract.write_to_terminal(output_selection)?;
        }

        Ok(())
    }

    ///
    /// Writes all contracts to the specified directory.
    ///
    pub fn write_to_directory(
        mut self,
        output_directory: &Path,
        output_selection: &solx_standard_json::InputSelection,
        overwrite: bool,
    ) -> anyhow::Result<()> {
        self.take_and_write_warnings();
        self.exit_on_error();

        std::fs::create_dir_all(output_directory)?;

        for (path, ast_json) in self.ast_jsons.into_iter().flatten() {
            if output_selection.check_selection(
                path.as_str(),
                None,
                solx_standard_json::InputSelector::AST,
            ) {
                let path = PathBuf::from(path).normalize()?;
                let path = if path.starts_with(std::env::current_dir()?) {
                    path.as_path().strip_prefix(std::env::current_dir()?)?
                } else {
                    path.as_path()
                }
                .to_string_lossy()
                .replace(['\\', '/'], "_");

                let output_name = format!(
                    "{path}_{}.{}",
                    solx_utils::EXTENSION_JSON,
                    solx_utils::EXTENSION_SOLIDITY_AST
                );
                let mut output_path = output_directory.to_owned();
                output_path.push(output_name.as_str());

                let ast_json = ast_json
                    .ok_or_else(|| anyhow::anyhow!("AST JSON is missing for `{path}`"))?
                    .to_string();
                Contract::write_to_file(output_path.as_path(), ast_json, overwrite)?;
            }
        }

        if output_selection.check_selection(
            solx_standard_json::InputSelection::WILDCARD,
            Some(solx_standard_json::InputSelection::ANY_CONTRACT),
            solx_standard_json::InputSelector::Benchmarks,
        ) {
            let mut output_path = output_directory.to_owned();
            output_path.push("benchmarks.txt");

            let mut output = String::with_capacity(self.benchmarks.len() * 256);
            output.push_str("Benchmarks:\n");
            for (name, value) in self.benchmarks.iter() {
                output.push_str(format!("{name}: {value}ms\n").as_str());
            }
            Contract::write_to_file(output_path.as_path(), output, overwrite)?;
        }

        for contract in self.contracts.into_values() {
            contract.write_to_directory(output_directory, output_selection, overwrite)?;
        }

        writeln!(
            std::io::stderr(),
            "Compiler run successful. Artifact(s) can be found in directory {output_directory:?}."
        )?;
        Ok(())
    }

    ///
    /// Writes all contracts assembly and bytecode to the standard JSON.
    ///
    pub fn write_to_standard_json(
        mut self,
        standard_json: &mut solx_standard_json::Output,
        output_selection: &solx_standard_json::InputSelection,
        is_bytecode_linked: bool,
        benchmarks: Vec<(String, u64)>,
    ) -> anyhow::Result<()> {
        for (path, ast_json) in self.ast_jsons.iter_mut().flatten() {
            if let Some(source) = standard_json.sources.get_mut(path.as_str())
                && let Some(ast_json) = ast_json.take().filter(|_| {
                    output_selection.check_selection(
                        path.as_str(),
                        None,
                        solx_standard_json::InputSelector::AST,
                    )
                })
            {
                source.ast = Some(ast_json);
            }
        }

        for mut contract in self.contracts.into_values() {
            if let (Some(deploy_object_result), Some(runtime_object_result)) = (
                contract.deploy_object_result.as_mut(),
                contract.runtime_object_result.as_mut(),
            ) {
                standard_json.errors.extend(
                    deploy_object_result
                        .as_mut()
                        .map(|object| object.take_warnings_standard_json())
                        .unwrap_or_default(),
                );
                standard_json.errors.extend(
                    runtime_object_result
                        .as_mut()
                        .map(|object| object.take_warnings_standard_json())
                        .unwrap_or_default(),
                );
                if deploy_object_result.is_err() || runtime_object_result.is_err() {
                    if let Some(Err(Error::StandardJson(error))) =
                        contract.deploy_object_result.take()
                    {
                        standard_json.errors.push(error);
                    }
                    if let Some(Err(Error::StandardJson(error))) =
                        contract.runtime_object_result.take()
                    {
                        standard_json.errors.push(error);
                    }
                    continue;
                }
            };

            let name = contract.name.clone();

            match standard_json
                .contracts
                .get_mut(name.path.as_str())
                .and_then(|contracts| {
                    contracts.get_mut(name.name.as_deref().unwrap_or(name.path.as_str()))
                }) {
                Some(standard_json_contract) => {
                    contract.write_to_standard_json(
                        standard_json_contract,
                        output_selection,
                        is_bytecode_linked,
                    );
                }
                None => {
                    let contracts = standard_json
                        .contracts
                        .entry(name.path.clone())
                        .or_default();
                    let mut standard_json_contract = solx_standard_json::OutputContract::default();
                    contract.write_to_standard_json(
                        &mut standard_json_contract,
                        output_selection,
                        is_bytecode_linked,
                    );
                    contracts.insert(name.name.unwrap_or(name.path), standard_json_contract);
                }
            }
        }
        standard_json
            .errors
            .extend(self.messages.lock_sync().drain(..));
        if standard_json.has_errors() {
            standard_json.contracts.clear();
        }

        if output_selection.check_selection(
            solx_standard_json::InputSelection::WILDCARD,
            Some(solx_standard_json::InputSelection::ANY_CONTRACT),
            solx_standard_json::InputSelector::Benchmarks,
        ) {
            standard_json.benchmarks.extend(benchmarks);
        }
        Ok(())
    }
}

impl solx_standard_json::CollectableError for Build {
    fn error_strings(&self) -> Vec<String> {
        let mut errors: Vec<String> = self
            .contracts
            .values()
            .flat_map(|contract| {
                let mut errors = Vec::with_capacity(2);
                if let Some(Err(error)) = contract.deploy_object_result.as_ref() {
                    errors.push(error);
                }
                if let Some(Err(error)) = contract.runtime_object_result.as_ref() {
                    errors.push(error);
                }
                errors
            })
            .map(|error| error.unwrap_standard_json_ref().to_string())
            .collect();
        errors.extend(
            self.messages
                .lock()
                .expect("Sync")
                .iter()
                .filter_map(|message| {
                    if message.severity == "error" {
                        Some(message.to_string())
                    } else {
                        None
                    }
                }),
        );
        errors
    }

    fn take_warnings(&mut self) -> Vec<solx_standard_json::OutputError> {
        let mut warnings: Vec<solx_standard_json::OutputError> = self
            .messages
            .lock()
            .expect("Sync")
            .extract_if(.., |message| message.severity == "warning")
            .collect();
        for contract in self.contracts.values_mut() {
            let (mut deploy_object_result, mut runtime_object_result) = match (
                contract.deploy_object_result.as_mut(),
                contract.runtime_object_result.as_mut(),
            ) {
                (Some(deploy_object_result), Some(runtime_object_result)) => (
                    deploy_object_result.as_mut(),
                    runtime_object_result.as_mut(),
                ),
                _ => continue,
            };

            warnings.extend(
                deploy_object_result
                    .as_mut()
                    .map(|object| object.take_warnings_standard_json())
                    .unwrap_or_default(),
            );
            warnings.extend(
                runtime_object_result
                    .as_mut()
                    .map(|object| object.take_warnings_standard_json())
                    .unwrap_or_default(),
            );
        }
        warnings
    }

    fn has_errors(&self) -> bool {
        self.contracts.values().any(|contract| {
            contract
                .deploy_object_result
                .as_ref()
                .map(|result| result.is_err())
                .unwrap_or_default()
                || contract
                    .runtime_object_result
                    .as_ref()
                    .map(|result| result.is_err())
                    .unwrap_or_default()
        }) || self
            .messages
            .lock()
            .expect("Sync")
            .iter()
            .any(|message| message.severity == "error")
    }
}
