//!
//! The Yul compiler.
//!

pub mod mode;
pub mod mode_upstream;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::sync::Arc;

use solx_standard_json::CollectableError as SolxCollectableError;

use crate::compilers::Compiler;
use crate::compilers::mode::Mode;
use crate::compilers::solidity::solc::SolidityCompiler as SolcCompiler;
use crate::compilers::solidity::solx::SolidityCompiler as SolxCompiler;
use crate::revm::input::Input as EVMInput;
use crate::toolchain::Toolchain;
use solx_standard_json::InputLanguage as SolcStandardJsonInputLanguage;

use self::mode::Mode as YulMode;

///
/// The Yul compiler.
///
pub enum YulCompiler {
    /// `solx` toolchain.
    Solx(Arc<SolxCompiler>),
    /// `solc` toolchain.
    Solc,
    /// `solc-llvm` toolchain.
    SolcLLVM,
}

impl Compiler for YulCompiler {
    fn compile_for_evm(
        &self,
        test_path: String,
        sources: Vec<(String, String)>,
        libraries: solx_utils::Libraries,
        mode: &Mode,
        test_params: Option<&solx_solc_test_adapter::Params>,
        llvm_options: Vec<String>,
        debug_config: Option<solx_codegen_evm::DebugConfig>,
    ) -> anyhow::Result<EVMInput> {
        match self {
            Self::Solx(solx) => {
                let yul_mode = YulMode::unwrap(mode);

                let last_contract = sources
                    .last()
                    .ok_or_else(|| anyhow::anyhow!("Yul sources are empty"))?
                    .0
                    .clone();

                let sources: BTreeMap<String, solx_standard_json::InputSource> = sources
                    .iter()
                    .map(|(path, source)| {
                        (
                            path.to_owned(),
                            solx_standard_json::InputSource::from(source.to_owned()),
                        )
                    })
                    .collect();

                let libraries = solx_utils::Libraries {
                    inner: libraries.inner,
                };

                let mut selectors = BTreeSet::new();
                selectors.insert(solx_standard_json::InputSelector::Bytecode);
                selectors.insert(solx_standard_json::InputSelector::RuntimeBytecode);
                selectors.insert(solx_standard_json::InputSelector::AST);
                selectors.insert(solx_standard_json::InputSelector::MethodIdentifiers);
                selectors.insert(solx_standard_json::InputSelector::Metadata);
                selectors.insert(solx_standard_json::InputSelector::Yul);
                let solx_input = solx_standard_json::Input::from_yul_sources(
                    sources,
                    libraries.to_owned(),
                    solx_standard_json::InputOptimizer::new(
                        yul_mode.llvm_optimizer_settings.middle_end_as_char(),
                        yul_mode.llvm_optimizer_settings.is_fallback_to_size_enabled,
                    ),
                    &solx_standard_json::InputSelection::new(selectors),
                    solx_standard_json::InputMetadata::default(),
                    llvm_options,
                );

                let solx_output = solx.standard_json(
                    mode,
                    solx_input,
                    &[],
                    debug_config
                        .as_ref()
                        .map(|debug_config| debug_config.output_directory.as_path()),
                )?;
                solx_output.check_errors()?;

                let mut builds = HashMap::with_capacity(solx_output.contracts.len());
                for (file, contracts) in solx_output.contracts.into_iter() {
                    for (_name, contract) in contracts.into_iter() {
                        let evm = contract.evm.as_ref().ok_or_else(|| {
                            anyhow::anyhow!("EVM object of the contract `{file}` not found")
                        })?;
                        let deploy_code_string = evm
                            .bytecode
                            .as_ref()
                            .ok_or_else(|| {
                                anyhow::anyhow!("EVM bytecode of the contract `{file}` not found")
                            })?
                            .object
                            .as_ref()
                            .ok_or_else(|| {
                                anyhow::anyhow!(
                                    "EVM bytecode object of the contract `{file}` not found"
                                )
                            })?
                            .as_str();
                        let deploy_code = hex::decode(deploy_code_string).expect("Always valid");
                        let runtime_code_size = evm
                            .deployed_bytecode
                            .as_ref()
                            .ok_or_else(|| {
                                anyhow::anyhow!("EVM bytecode of the contract `{file}` not found")
                            })?
                            .object
                            .as_ref()
                            .ok_or_else(|| {
                                anyhow::anyhow!(
                                    "EVM bytecode object of the contract `{file}` not found"
                                )
                            })?
                            .len()
                            / 2;
                        builds.insert(file.clone(), (deploy_code, runtime_code_size));
                    }
                }

                Ok(EVMInput::new(builds, None, last_contract))
            }
            Self::Solc | Self::SolcLLVM => {
                let language = SolcStandardJsonInputLanguage::Yul;

                let solc_compiler = SolcCompiler::new(language, Toolchain::from(self));

                let solc_output = solc_compiler.standard_json_output_cached(
                    test_path,
                    language,
                    &sources,
                    &libraries,
                    mode,
                    test_params,
                )?;

                if let Some(errors) = solc_output.errors_opt() {
                    let mut has_errors = false;
                    let mut error_messages = Vec::with_capacity(errors.len());

                    for error in errors.iter() {
                        if error.severity.as_str() == "error" {
                            has_errors = true;
                            error_messages.push(error.formatted_message.to_owned());
                        }
                    }

                    if has_errors {
                        anyhow::bail!("`solc` errors found: {error_messages:?}");
                    }
                }

                let last_contract = sources
                    .last()
                    .ok_or_else(|| anyhow::anyhow!("Yul sources are empty"))?
                    .0
                    .clone();

                let contracts = solc_output
                    .contracts_opt()
                    .ok_or_else(|| anyhow::anyhow!("Solidity contracts not found in the output"))?;

                let mut builds = HashMap::with_capacity(contracts.len());
                for (file, contracts) in contracts.iter() {
                    for (name, contract) in contracts.iter() {
                        let path = format!("{file}:{name}");
                        let evm = contract.evm.as_ref().ok_or_else(|| {
                            anyhow::anyhow!("EVM object of the contract `{path}` not found")
                        })?;
                        let deploy_code_string = evm
                            .bytecode
                            .as_ref()
                            .ok_or_else(|| {
                                anyhow::anyhow!("EVM bytecode of the contract `{path}` not found")
                            })?
                            .object
                            .as_ref()
                            .ok_or_else(|| {
                                anyhow::anyhow!(
                                    "EVM bytecode object of the contract `{path}` not found"
                                )
                            })?
                            .as_str();
                        let deploy_code = hex::decode(deploy_code_string).expect("Always valid");
                        let runtime_code_size = evm
                            .deployed_bytecode
                            .as_ref()
                            .ok_or_else(|| {
                                anyhow::anyhow!("EVM bytecode of the contract `{path}` not found")
                            })?
                            .object
                            .as_ref()
                            .ok_or_else(|| {
                                anyhow::anyhow!("EVM deployed bytecode object of the contract `{path}` not found")
                            })?
                            .len()
                            / 2;
                        builds.insert(path, (deploy_code, runtime_code_size));
                    }
                }

                Ok(EVMInput::new(builds, None, last_contract))
            }
        }
    }

    fn all_modes(&self) -> Vec<Mode> {
        solx_codegen_evm::OptimizerSettings::combinations()
            .into_iter()
            .map(|llvm_optimizer_settings| YulMode::new(llvm_optimizer_settings).into())
            .collect::<Vec<Mode>>()
    }

    fn allows_multi_contract_files(&self) -> bool {
        false
    }
}

impl From<&YulCompiler> for Toolchain {
    fn from(value: &YulCompiler) -> Self {
        match value {
            YulCompiler::Solc => Self::Solc,
            YulCompiler::Solx { .. } => Self::IrLLVM,
            YulCompiler::SolcLLVM => Self::SolcLLVM,
        }
    }
}
