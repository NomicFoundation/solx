//!
//! The project representation.
//!

pub mod contract;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

use crate::build::contract::Contract as EVMContractBuild;
use crate::build::Build as EVMBuild;
use crate::error::Error;
use crate::process::input::Input as EVMProcessInput;
use crate::process::output::Output as EVMProcessOutput;

use self::contract::ir::evmla::EVMLegacyAssembly as ContractEVMLegacyAssembly;
use self::contract::ir::llvm_ir::LLVMIR as ContractLLVMIR;
use self::contract::ir::yul::Yul as ContractYul;
use self::contract::ir::IR as ContractIR;
use self::contract::metadata::Metadata as ContractMetadata;
use self::contract::Contract;

///
/// The project representation.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Project {
    /// The project language.
    pub language: solx_standard_json::InputLanguage,
    /// The `solc` compiler version.
    /// Used only for Solidity and Yul input languages.
    pub solc_version: Option<solx_standard_json::Version>,
    /// The project build results.
    pub contracts: BTreeMap<String, Contract>,
    /// The Solidity AST JSONs of the source files.
    pub ast_jsons: Option<BTreeMap<String, Option<serde_json::Value>>>,
    /// The mapping of auxiliary identifiers, e.g. Yul object names, to full contract paths.
    pub identifier_paths: BTreeMap<String, String>,
    /// The library addresses.
    pub libraries: solx_utils::Libraries,
}

impl Project {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        language: solx_standard_json::InputLanguage,
        solc_version: Option<solx_standard_json::Version>,
        contracts: BTreeMap<String, Contract>,
        ast_jsons: Option<BTreeMap<String, Option<serde_json::Value>>>,
        libraries: solx_utils::Libraries,
    ) -> Self {
        let mut identifier_paths = BTreeMap::new();
        for (path, contract) in contracts.iter() {
            identifier_paths.insert(contract.identifier().to_owned(), path.to_owned());
        }

        let solc_version = match language {
            solx_standard_json::InputLanguage::Solidity
            | solx_standard_json::InputLanguage::Yul => Some(
                solc_version.expect("`solc` version is mandatory for Solidity and Yul projects"),
            ),
            solx_standard_json::InputLanguage::LLVMIR => None,
        };

        Self {
            language,
            solc_version,
            contracts,
            ast_jsons,
            identifier_paths,
            libraries,
        }
    }

    ///
    /// Parses the Solidity `sources` and returns a Solidity project.
    ///
    pub fn try_from_solc_output(
        solc_version: &solx_standard_json::Version,
        libraries: solx_utils::Libraries,
        via_ir: bool,
        solc_output: &mut solx_standard_json::Output,
        debug_config: Option<&solx_codegen_evm::DebugConfig>,
    ) -> anyhow::Result<Self> {
        if !via_ir {
            let legacy_assemblies: BTreeMap<
                String,
                BTreeMap<String, &mut solx_evm_assembly::Assembly>,
            > = solc_output
                .contracts
                .iter_mut()
                .map(|(path, file)| {
                    let legacy_assemblies: BTreeMap<String, &mut solx_evm_assembly::Assembly> =
                        file.iter_mut()
                            .filter_map(|(name, contract)| {
                                Some((
                                    name.to_owned(),
                                    contract
                                        .evm
                                        .as_mut()
                                        .and_then(|evm| evm.legacy_assembly.as_mut())?,
                                ))
                            })
                            .collect();
                    (path.to_owned(), legacy_assemblies)
                })
                .collect();
            solx_evm_assembly::Assembly::preprocess_dependencies(legacy_assemblies)?;
        }

        let ast_jsons = solc_output
            .sources
            .iter_mut()
            .map(|(path, source)| (path.to_owned(), source.ast.take()))
            .collect::<BTreeMap<String, Option<serde_json::Value>>>();

        let mut input_contracts = Vec::with_capacity(solc_output.contracts.len());
        for path in solc_output
            .contracts
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
        {
            let file = solc_output
                .contracts
                .remove(path.as_str())
                .expect("Always exists");
            for (name, contract) in file.into_iter() {
                let name = solx_utils::ContractName::new(path.clone(), Some(name));
                input_contracts.push((name, contract));
            }
        }

        let results = input_contracts
            .into_par_iter()
            .map(|(name, mut contract)| {
                let method_identifiers = contract
                    .evm
                    .as_mut()
                    .and_then(|evm| evm.method_identifiers.take());
                let legacy_assembly = contract
                    .evm
                    .as_mut()
                    .and_then(|evm| evm.legacy_assembly.take());
                let extra_metadata = contract
                    .evm
                    .as_mut()
                    .and_then(|evm| evm.extra_metadata.take());

                let result = if via_ir {
                    contract.ir.as_deref().map(|ir| {
                        ContractYul::try_from_source(name.full_path.as_str(), ir, debug_config)
                            .map(|yul| yul.map(ContractIR::from))
                    })
                } else {
                    legacy_assembly.as_ref().map(|legacy_assembly| {
                        Ok(Some(ContractIR::from(
                            ContractEVMLegacyAssembly::from_contract(
                                legacy_assembly.to_owned(),
                                extra_metadata,
                            ),
                        )))
                    })
                };
                let ir = match result {
                    Some(Ok(Some(ir))) => Some(ir),
                    Some(Err(error)) => return (name.full_path, Err(error)),
                    Some(Ok(None)) | None => None,
                };
                let contract = Contract::new(
                    name.clone(),
                    ir,
                    contract.metadata,
                    contract.abi,
                    method_identifiers,
                    contract.userdoc,
                    contract.devdoc,
                    contract.storage_layout,
                    contract.transient_storage_layout,
                    legacy_assembly,
                    contract.ir,
                );
                (name.full_path, Ok(contract))
            })
            .collect::<BTreeMap<String, anyhow::Result<Contract>>>();

        let mut contracts = BTreeMap::new();
        for (path, result) in results.into_iter() {
            match result {
                Ok(contract) => {
                    contracts.insert(path, contract);
                }
                Err(error) => solc_output.push_error(Some(path), error),
            }
        }
        Ok(Project::new(
            solx_standard_json::InputLanguage::Solidity,
            Some(solc_version.to_owned()),
            contracts,
            Some(ast_jsons),
            libraries,
        ))
    }

    ///
    /// Reads the Yul source code `paths` and returns a Yul project.
    ///
    pub fn try_from_yul_paths(
        solc_version: &solx_standard_json::Version,
        paths: &[PathBuf],
        libraries: solx_utils::Libraries,
        output_selection: &solx_standard_json::InputSelection,
        solc_output: Option<&mut solx_standard_json::Output>,
        debug_config: Option<&solx_codegen_evm::DebugConfig>,
    ) -> anyhow::Result<Self> {
        let sources = paths
            .iter()
            .map(|path| {
                let source = solx_standard_json::InputSource::try_from_path(path.as_path())?;
                let path = if path.to_string_lossy()
                    == solx_standard_json::InputSource::STDIN_INPUT_IDENTIFIER
                {
                    solx_standard_json::InputSource::STDIN_OUTPUT_IDENTIFIER.to_owned()
                } else {
                    path.to_string_lossy().to_string()
                };
                Ok((path, source))
            })
            .collect::<anyhow::Result<BTreeMap<String, solx_standard_json::InputSource>>>()?;

        Self::try_from_yul_sources(
            solc_version,
            sources,
            libraries,
            output_selection,
            solc_output,
            debug_config,
        )
    }

    ///
    /// Parses the Yul `sources` and returns a Yul project.
    ///
    pub fn try_from_yul_sources(
        solc_version: &solx_standard_json::Version,
        sources: BTreeMap<String, solx_standard_json::InputSource>,
        libraries: solx_utils::Libraries,
        output_selection: &solx_standard_json::InputSelection,
        mut solc_output: Option<&mut solx_standard_json::Output>,
        debug_config: Option<&solx_codegen_evm::DebugConfig>,
    ) -> anyhow::Result<Self> {
        let results = sources
            .into_par_iter()
            .map(|(path, mut source)| {
                let source_code = match source.try_resolve() {
                    Ok(()) => source.take_content().expect("Always exists"),
                    Err(error) => return (path, Err(error)),
                };

                let metadata = if output_selection.check_selection(
                    path.as_str(),
                    None,
                    solx_standard_json::InputSelector::Metadata,
                ) {
                    let source_hash = solx_utils::Keccak256Hash::from_slice(source_code.as_bytes());
                    let metadata_json = serde_json::json!({
                        "source_hash": source_hash.to_string(),
                        "solc_version": solc_version,
                    });
                    Some(serde_json::to_string(&metadata_json).expect("Always valid"))
                } else {
                    None
                };

                let ir = match ContractYul::try_from_source(
                    path.as_str(),
                    source_code.as_str(),
                    debug_config,
                ) {
                    Ok(ir) => ir,
                    Err(error) => return (path, Err(error)),
                };

                let name = solx_utils::ContractName::new(
                    path.clone(),
                    ir.as_ref().map(|ir| ir.object.0.identifier.to_owned()),
                );
                let full_path = name.full_path.clone();
                let contract = Contract::new(
                    name,
                    ir.map(ContractIR::from),
                    metadata,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                );
                (full_path, Ok(contract))
            })
            .collect::<BTreeMap<String, anyhow::Result<Contract>>>();

        let mut contracts = BTreeMap::new();
        for (path, result) in results.into_iter() {
            match result {
                Ok(contract) => {
                    contracts.insert(path, contract);
                }
                Err(error) => match solc_output {
                    Some(ref mut solc_output) => solc_output.push_error(Some(path), error),
                    None => anyhow::bail!(error),
                },
            }
        }
        Ok(Self::new(
            solx_standard_json::InputLanguage::Yul,
            Some(solc_version.to_owned()),
            contracts,
            None,
            libraries,
        ))
    }

    ///
    /// Reads the LLVM IR source code `paths` and returns an LLVM IR project.
    ///
    pub fn try_from_llvm_ir_paths(
        paths: &[PathBuf],
        libraries: solx_utils::Libraries,
        output_selection: &solx_standard_json::InputSelection,
        solc_output: Option<&mut solx_standard_json::Output>,
    ) -> anyhow::Result<Self> {
        let sources = paths
            .iter()
            .map(|path| {
                let source = solx_standard_json::InputSource::try_from_path(path.as_path())?;
                let path = if path.to_string_lossy()
                    == solx_standard_json::InputSource::STDIN_INPUT_IDENTIFIER
                {
                    solx_standard_json::InputSource::STDIN_OUTPUT_IDENTIFIER.to_owned()
                } else {
                    path.to_string_lossy().to_string()
                };
                Ok((path, source))
            })
            .collect::<anyhow::Result<BTreeMap<String, solx_standard_json::InputSource>>>()?;

        Self::try_from_llvm_ir_sources(sources, libraries, output_selection, solc_output)
    }

    ///
    /// Parses the LLVM IR `sources` and returns an LLVM IR project.
    ///
    pub fn try_from_llvm_ir_sources(
        sources: BTreeMap<String, solx_standard_json::InputSource>,
        libraries: solx_utils::Libraries,
        output_selection: &solx_standard_json::InputSelection,
        mut solc_output: Option<&mut solx_standard_json::Output>,
    ) -> anyhow::Result<Self> {
        let results = sources
            .into_par_iter()
            .map(|(path, mut source)| {
                let source_code = match source.try_resolve() {
                    Ok(()) => source.take_content().expect("Always exists"),
                    Err(error) => return (path, Err(error)),
                };

                let metadata = if output_selection.check_selection(
                    path.as_str(),
                    None,
                    solx_standard_json::InputSelector::Metadata,
                ) {
                    let source_hash = solx_utils::Keccak256Hash::from_slice(source_code.as_bytes());
                    let metadata_json = serde_json::json!({
                        "source_hash": source_hash.to_string(),
                        "llvm_version": solx_codegen_evm::LLVM_VERSION,
                    });
                    Some(serde_json::to_string(&metadata_json).expect("Always valid"))
                } else {
                    None
                };

                let contract = Contract::new(
                    solx_utils::ContractName::new(path.clone(), None),
                    Some(
                        ContractLLVMIR::new(
                            path.clone(),
                            solx_utils::CodeSegment::Runtime,
                            source_code,
                        )
                        .into(),
                    ),
                    metadata,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                );

                (path, Ok(contract))
            })
            .collect::<BTreeMap<String, anyhow::Result<Contract>>>();

        let mut contracts = BTreeMap::new();
        for (path, result) in results.into_iter() {
            match result {
                Ok(contract) => {
                    contracts.insert(path, contract);
                }
                Err(error) => match solc_output {
                    Some(ref mut solc_output) => solc_output.push_error(Some(path), error),
                    None => anyhow::bail!(error),
                },
            }
        }
        Ok(Self::new(
            solx_standard_json::InputLanguage::LLVMIR,
            None,
            contracts,
            None,
            libraries,
        ))
    }

    ///
    /// Compiles all contracts to EVM, returning their build artifacts.
    ///
    pub fn compile_to_evm(
        self,
        messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
        output_selection: &solx_standard_json::InputSelection,
        evm_version: Option<solx_utils::EVMVersion>,
        metadata_hash_type: solx_utils::MetadataHashType,
        append_cbor: bool,
        optimizer_settings: solx_codegen_evm::OptimizerSettings,
        llvm_options: Vec<String>,
        debug_config: Option<solx_codegen_evm::DebugConfig>,
    ) -> anyhow::Result<EVMBuild> {
        let results = self
            .contracts
            .into_par_iter()
            .map(|(path, mut contract)| {
                let contract_name = contract.name.clone();

                let metadata = contract.metadata.take();
                let abi = contract.abi.take();
                let method_identifiers = contract.method_identifiers.take();
                let userdoc = contract.userdoc.take();
                let devdoc = contract.devdoc.take();
                let storage_layout = contract.storage_layout.take();
                let transient_storage_layout = contract.transient_storage_layout.take();
                let legacy_assembly = contract.legacy_assembly.take();
                let yul = contract.yul.take();

                let (deploy_code_ir, runtime_code_ir): (ContractIR, ContractIR) = match contract.ir
                {
                    Some(ContractIR::Yul(mut deploy_code)) => {
                        let runtime_code: ContractYul =
                            *deploy_code.runtime_code.take().expect("Always exists");
                        (deploy_code.into(), runtime_code.into())
                    }
                    Some(ContractIR::EVMLegacyAssembly(mut deploy_code)) => {
                        let runtime_code: ContractEVMLegacyAssembly =
                            *deploy_code.runtime_code.take().expect("Always exists");
                        (deploy_code.into(), runtime_code.into())
                    }
                    Some(ContractIR::LLVMIR(runtime_code)) => {
                        let deploy_code_identifier = contract.name.full_path.to_owned();
                        let runtime_code_identifier = format!(
                            "{deploy_code_identifier}.{}",
                            solx_utils::CodeSegment::Runtime
                        );

                        let deploy_code = ContractLLVMIR::new(
                            deploy_code_identifier.clone(),
                            solx_utils::CodeSegment::Deploy,
                            solx_codegen_evm::minimal_deploy_code(
                                deploy_code_identifier.as_str(),
                                runtime_code_identifier.as_str(),
                            ),
                        );
                        (deploy_code.into(), runtime_code.into())
                    }
                    None => {
                        let build = EVMContractBuild::new(
                            contract_name,
                            None,
                            None,
                            metadata,
                            abi,
                            method_identifiers,
                            userdoc,
                            devdoc,
                            storage_layout,
                            transient_storage_layout,
                            legacy_assembly,
                            yul,
                        );
                        return (path, build);
                    }
                };

                let (runtime_object_result, metadata) = {
                    let metadata_bytes = Self::cbor_metadata(
                        metadata.as_deref(),
                        self.solc_version.as_ref(),
                        &optimizer_settings,
                        llvm_options.as_slice(),
                        metadata_hash_type,
                        append_cbor,
                    );

                    let mut input = EVMProcessInput::new(
                        self.solc_version.clone(),
                        contract_name.clone(),
                        runtime_code_ir,
                        solx_utils::CodeSegment::Runtime,
                        evm_version,
                        self.identifier_paths.clone(),
                        output_selection.to_owned(),
                        None,
                        metadata_bytes,
                        optimizer_settings.clone(),
                        llvm_options.clone(),
                        debug_config.clone(),
                    );

                    let result = Self::run_multi_pass_pipeline(path.as_str(), &mut input);
                    (result, metadata)
                };

                let immutables = runtime_object_result
                    .as_ref()
                    .ok()
                    .and_then(|output| output.object.immutables.to_owned());
                let deploy_object_result: crate::Result<EVMProcessOutput> = {
                    let mut input = EVMProcessInput::new(
                        self.solc_version.clone(),
                        contract_name.clone(),
                        deploy_code_ir,
                        solx_utils::CodeSegment::Deploy,
                        evm_version,
                        self.identifier_paths.clone(),
                        output_selection.to_owned(),
                        immutables,
                        None,
                        optimizer_settings.clone(),
                        llvm_options.clone(),
                        debug_config.clone(),
                    );

                    Self::run_multi_pass_pipeline(path.as_str(), &mut input)
                };

                let build = EVMContractBuild::new(
                    contract_name,
                    Some(deploy_object_result.map(|deploy_code_output| deploy_code_output.object)),
                    Some(
                        runtime_object_result.map(|runtime_code_output| runtime_code_output.object),
                    ),
                    metadata,
                    abi,
                    method_identifiers,
                    userdoc,
                    devdoc,
                    storage_layout,
                    transient_storage_layout,
                    legacy_assembly,
                    yul,
                );
                (path, build)
            })
            .collect::<BTreeMap<String, EVMContractBuild>>();

        Ok(EVMBuild::new(results, self.ast_jsons, messages))
    }

    ///
    /// Returns the CBOR metadata, based on the current settings.
    ///
    fn cbor_metadata(
        metadata: Option<&str>,
        solc_version: Option<&solx_standard_json::Version>,
        optimizer_settings: &solx_codegen_evm::OptimizerSettings,
        llvm_options: &[String],
        metadata_hash_type: solx_utils::MetadataHashType,
        append_cbor: bool,
    ) -> Option<Vec<u8>> {
        if !append_cbor {
            return None;
        }

        let metadata = metadata.map(|metadata| {
            ContractMetadata::new(solc_version, optimizer_settings.clone(), llvm_options)
                .insert_into(metadata)
        });
        let metadata_hash = metadata
            .as_ref()
            .and_then(|metadata| match metadata_hash_type {
                solx_utils::MetadataHashType::None => None,
                solx_utils::MetadataHashType::IPFS => {
                    Some(solx_utils::IPFSHash::from_slice(metadata.as_bytes()).to_vec())
                }
            });

        let mut cbor_version_parts = Vec::with_capacity(3);
        cbor_version_parts.push((
            crate::r#const::DEFAULT_EXECUTABLE_NAME.to_owned(),
            crate::r#const::version().parse().expect("Always valid"),
        ));
        if let Some(solc_version) = solc_version {
            cbor_version_parts.push((
                crate::r#const::SOLC_METADATA_TAG.to_owned(),
                solc_version.default.to_owned(),
            ));
            cbor_version_parts.push((
                crate::r#const::SOLC_LLVM_REVISION_METADATA_TAG.to_owned(),
                solc_version.llvm_revision.to_owned(),
            ));
        }
        let cbor_data = (
            crate::r#const::SOLC_METADATA_TAG.to_owned(),
            cbor_version_parts,
        );

        match metadata_hash {
            Some(hash) => {
                let cbor = solx_utils::CBOR::new(
                    Some((solx_utils::MetadataHashType::IPFS, hash.as_slice())),
                    cbor_data.0,
                    cbor_data.1,
                );
                Some(cbor.to_vec())
            }
            None => {
                let cbor = solx_utils::CBOR::<'_, String>::new(None, cbor_data.0, cbor_data.1);
                Some(cbor.to_vec())
            }
        }
    }

    ///
    /// Runs the multi-pass compilation pipeline.
    ///
    /// It is expected to run up to 4 passes in the process of handling stack too deep errors
    /// and turning on the size fallback to overcome the EVM bytecode size limit.
    ///
    fn run_multi_pass_pipeline(
        path: &str,
        input: &mut EVMProcessInput,
    ) -> crate::Result<EVMProcessOutput> {
        let mut result: crate::Result<EVMProcessOutput>;
        let mut pass_count = 0;
        loop {
            result = crate::process::call(path, input);
            pass_count += 1;
            match result {
                Err(Error::StackTooDeep(ref stack_too_deep)) => {
                    assert!(pass_count <= 2, "Stack too deep error is not resolved after {pass_count} passes: {stack_too_deep}");

                    if stack_too_deep.is_size_fallback {
                        input.optimizer_settings.switch_to_size_fallback();
                    }
                    input
                        .optimizer_settings
                        .set_spill_area_size(stack_too_deep.spill_area_size);

                    continue;
                }
                _ => break,
            }
        }
        result
    }
}
