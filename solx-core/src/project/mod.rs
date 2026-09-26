//!
//! The project representation.
//!

pub mod contract;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;

use crate::build::Build as EVMBuild;
use crate::build::contract::Contract as EVMContractBuild;
use crate::error::Error;
use crate::process::job::Job as EVMProcessJob;
use crate::process::output::Output as EVMProcessOutput;
use crate::process::pool::Pool as EVMProcessPool;
use crate::process::session::Session as EVMProcessSession;

use self::contract::Contract;
use self::contract::ir::IR as ContractIR;
use self::contract::ir::llvm_ir::LLVMIR as ContractLLVMIR;
use self::contract::ir::mlir::MLIR as ContractMLIR;
use self::contract::metadata::Metadata as ContractMetadata;

///
/// The project representation.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Project {
    /// The `solc` compiler version, absent for LLVM IR projects.
    pub solc_version: Option<solx_standard_json::Version>,
    /// The project build results.
    pub contracts: BTreeMap<String, Contract>,
    /// The Solidity AST JSONs of the source files.
    pub ast_jsons: Option<BTreeMap<String, Option<serde_json::Value>>>,
    /// The library addresses.
    pub libraries: solx_utils::Libraries,
}

impl Project {
    /// The number of stack-too-deep retries allowed with the initial optimizer settings and, separately, with the size fallback.
    const STACK_TOO_DEEP_RETRY_LIMIT: usize = 4;

    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        solc_version: Option<solx_standard_json::Version>,
        contracts: BTreeMap<String, Contract>,
        ast_jsons: Option<BTreeMap<String, Option<serde_json::Value>>>,
        libraries: solx_utils::Libraries,
    ) -> Self {
        Self {
            solc_version,
            contracts,
            ast_jsons,
            libraries,
        }
    }

    ///
    /// Parses the Solidity `sources` and returns a Solidity project.
    ///
    pub fn try_from_solidity_output(
        solc_version: &solx_standard_json::Version,
        libraries: solx_utils::Libraries,
        output: &mut solx_standard_json::Output,
    ) -> anyhow::Result<Self> {
        let ast_jsons = output
            .sources
            .iter_mut()
            .map(|(path, source)| (path.to_owned(), source.ast.take()))
            .collect::<BTreeMap<String, Option<serde_json::Value>>>();

        let mut input_contracts = Vec::with_capacity(output.contracts.len());
        for path in output
            .contracts
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
        {
            let file = output
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
                let result = contract.mlir.as_ref().map(|output| {
                    let runtime_code = ContractMLIR {
                        source: output.runtime_source.clone(),
                        dependencies: output.runtime_dependencies.clone(),
                        runtime_code: None,
                    };
                    let deploy_code = ContractMLIR {
                        source: output.deploy_source.clone(),
                        dependencies: output.deploy_dependencies.clone(),
                        runtime_code: Some(Box::new(runtime_code)),
                    };
                    Ok::<_, anyhow::Error>(Some(ContractIR::from(deploy_code)))
                });
                let ir = match result {
                    Some(Ok(Some(ir))) => Some(ir),
                    Some(Err(error)) => return (name, Err(error)),
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
                    contract.mlir.take(),
                );
                (name, Ok(contract))
            })
            .collect::<Vec<(solx_utils::ContractName, anyhow::Result<Contract>)>>();

        let mut contracts = BTreeMap::new();
        for (contract_name, result) in results.into_iter() {
            match result {
                Ok(contract) => {
                    contracts.insert(contract_name.full_path, contract);
                }
                Err(error) => output.push_error(contract_name.path.as_str(), error),
            }
        }
        Ok(Project::new(
            Some(solc_version.to_owned()),
            contracts,
            Some(ast_jsons),
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
        output: Option<&mut solx_standard_json::Output>,
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

        Self::try_from_llvm_ir_sources(sources, libraries, output_selection, output)
    }

    ///
    /// Parses the LLVM IR `sources` and returns an LLVM IR project.
    ///
    pub fn try_from_llvm_ir_sources(
        sources: BTreeMap<String, solx_standard_json::InputSource>,
        libraries: solx_utils::Libraries,
        output_selection: &solx_standard_json::InputSelection,
        mut output: Option<&mut solx_standard_json::Output>,
    ) -> anyhow::Result<Self> {
        let results = sources
            .into_par_iter()
            .map(|(path, mut source)| {
                let contract_name = solx_utils::ContractName::new(path.clone(), None);

                let source_code = match source.try_resolve() {
                    Ok(()) => match source.take_content() {
                        Some(content) => content,
                        None => {
                            return (
                                contract_name,
                                Err(anyhow::anyhow!("Source content is missing for `{path}`")),
                            );
                        }
                    },
                    Err(error) => return (contract_name, Err(error)),
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
                    contract_name.clone(),
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
                );

                (contract_name, Ok(contract))
            })
            .collect::<Vec<(solx_utils::ContractName, anyhow::Result<Contract>)>>();

        let mut contracts = BTreeMap::new();
        for (contract_name, result) in results.into_iter() {
            match result {
                Ok(contract) => {
                    contracts.insert(contract_name.full_path, contract);
                }
                Err(error) => match output.as_mut() {
                    Some(output) => output.push_error(contract_name.path.as_str(), error),
                    None => anyhow::bail!(error),
                },
            }
        }
        Ok(Self::new(None, contracts, None, libraries))
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
        output_config: Option<solx_codegen_evm::OutputConfig>,
    ) -> anyhow::Result<EVMBuild> {
        let Self {
            solc_version,
            contracts,
            ast_jsons,
            libraries: _,
        } = self;
        let pool = EVMProcessPool::new(EVMProcessSession::new(
            evm_version,
            output_selection.clone(),
            llvm_options.clone(),
            output_config,
        ))?;

        let mut contracts: Vec<(String, Contract)> = contracts.into_iter().collect();
        contracts.sort_unstable_by(|(_, left), (_, right)| {
            right
                .estimated_compilation_cost()
                .cmp(&left.estimated_compilation_cost())
        });
        let results = contracts
            .into_iter()
            .par_bridge()
            .map(|(path, mut contract)| {
                let contract_name = contract.name.clone();

                let metadata = contract.metadata.take().map(|metadata| {
                    ContractMetadata::new(
                        solc_version.as_ref(),
                        optimizer_settings.clone(),
                        llvm_options.as_slice(),
                    )
                    .insert_into(metadata.as_str())
                });
                let abi = contract.abi.take();
                let method_identifiers = contract.method_identifiers.take();
                let userdoc = contract.userdoc.take();
                let devdoc = contract.devdoc.take();
                let storage_layout = contract.storage_layout.take();
                let transient_storage_layout = contract.transient_storage_layout.take();
                let mlir = contract.mlir.take();

                let (deploy_code_ir, runtime_code_ir): (ContractIR, ContractIR) = match contract.ir
                {
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
                    Some(ContractIR::MLIR(mut deploy_code)) => {
                        let runtime_code: ContractMLIR =
                            *deploy_code.runtime_code.take().expect("Always exists");
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
                            mlir,
                        );
                        return (path, build);
                    }
                };

                let (runtime_object_result, metadata) = {
                    let metadata_bytes = Self::cbor_metadata(
                        metadata.as_deref(),
                        solc_version.as_ref(),
                        metadata_hash_type,
                        append_cbor,
                    );

                    let mut job = EVMProcessJob::new(
                        contract_name.clone(),
                        runtime_code_ir,
                        solx_utils::CodeSegment::Runtime,
                        None,
                        metadata_bytes,
                        optimizer_settings.clone(),
                    );

                    let result = Self::run_multi_pass_pipeline(&pool, &mut job);
                    (result, metadata)
                };

                let immutables = runtime_object_result
                    .as_ref()
                    .ok()
                    .and_then(|output| output.object.immutables.to_owned());
                let deploy_object_result: crate::Result<EVMProcessOutput> = {
                    let mut job = EVMProcessJob::new(
                        contract_name.clone(),
                        deploy_code_ir,
                        solx_utils::CodeSegment::Deploy,
                        immutables,
                        None,
                        optimizer_settings.clone(),
                    );

                    Self::run_multi_pass_pipeline(&pool, &mut job)
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
                    mlir,
                );
                (path, build)
            })
            .collect::<BTreeMap<String, EVMContractBuild>>();

        Ok(EVMBuild::new(results, ast_jsons, messages))
    }

    ///
    /// Returns the CBOR metadata, based on the current settings.
    ///
    fn cbor_metadata(
        metadata: Option<&str>,
        solc_version: Option<&solx_standard_json::Version>,
        metadata_hash_type: solx_utils::MetadataHashType,
        append_cbor: bool,
    ) -> Option<Vec<u8>> {
        if !append_cbor {
            return None;
        }

        let metadata_hash = metadata.and_then(|metadata| match metadata_hash_type {
            solx_utils::MetadataHashType::None => None,
            solx_utils::MetadataHashType::IPFS => {
                Some(solx_utils::IPFSHash::from_slice(metadata.as_bytes()).to_vec())
            }
        });

        let mut cbor_version_parts = Vec::with_capacity(2);
        cbor_version_parts.push((
            crate::r#const::DEFAULT_EXECUTABLE_NAME.to_owned(),
            crate::Compiler::version()
                .parse()
                .expect("version string is valid semver"),
        ));
        if let Some(solc_version) = solc_version {
            cbor_version_parts.push((
                crate::r#const::SOLC_METADATA_TAG.to_owned(),
                solc_version.default.to_owned(),
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
    /// Stack-too-deep errors are retried with the reported spill area size, up to
    /// `STACK_TOO_DEEP_RETRY_LIMIT` times with the initial optimizer settings and as many
    /// times again after switching to the size fallback to overcome the EVM bytecode size limit.
    ///
    fn run_multi_pass_pipeline(
        pool: &EVMProcessPool,
        job: &mut EVMProcessJob,
    ) -> crate::Result<EVMProcessOutput> {
        let mut profiler = solx_utils::Profiler::default();
        let mut stack_too_deep_retries = 0;
        let mut attempt = 0;
        let mut result = loop {
            let run_roundtrip = profiler.start_evm_translation_unit(
                job.contract_name.full_path.as_str(),
                Some(job.code_segment),
                format!("WorkerRoundtrip({attempt})").as_str(),
                job.optimizer_settings.to_string().as_str(),
                job.optimizer_settings.spill_area_size(),
            );
            let attempt_result = pool.execute(job);
            run_roundtrip.borrow_mut().finish();
            attempt += 1;

            match attempt_result {
                Err(Error::StackTooDeep(stack_too_deep)) => {
                    if stack_too_deep.is_size_fallback
                        && !job.optimizer_settings.is_fallback_to_size_active()
                    {
                        job.optimizer_settings.switch_to_size_fallback();
                        stack_too_deep_retries = 0;
                    } else if stack_too_deep_retries == Self::STACK_TOO_DEEP_RETRY_LIMIT {
                        break Err(solx_standard_json::OutputError::new_error_contract(
                            Some(job.contract_name.path.as_str()),
                            stack_too_deep,
                        )
                        .into());
                    } else {
                        stack_too_deep_retries += 1;
                    }
                    job.optimizer_settings
                        .set_spill_area_size(stack_too_deep.spill_area_size);
                }
                result => break result,
            }
        };
        if let Ok(output) = result.as_mut() {
            output.object.benchmarks.extend(profiler.to_vec());
        }
        result
    }
}
