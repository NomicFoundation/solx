//!
//! The LLVM compiler.
//!

pub mod mode;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::sync::Arc;

use crate::compilers::Compiler;
use crate::compilers::mode::Mode;
use crate::compilers::solidity::solx::SolidityCompiler as SolxCompiler;
use crate::revm::input::Input as EVMInput;

use self::mode::Mode as LLVMMode;

///
/// The LLVM compiler.
///
pub enum LLVMIRCompiler {
    /// `solx` toolchain.
    Solx(Arc<SolxCompiler>),
}

impl Compiler for LLVMIRCompiler {
    fn compile_for_evm(
        &self,
        _test_path: String,
        sources: Vec<(String, String)>,
        libraries: solx_utils::Libraries,
        mode: &Mode,
        _test_params: Option<&solx_solc_test_adapter::Params>,
        llvm_options: Vec<String>,
        debug_config: Option<solx_codegen_evm::DebugConfig>,
    ) -> anyhow::Result<EVMInput> {
        let llvm_ir_mode = LLVMMode::unwrap(mode);

        let last_contract = sources
            .last()
            .ok_or_else(|| anyhow::anyhow!("LLVM IR sources are empty"))?
            .0
            .clone();

        let builds = match self {
            Self::Solx(solx) => {
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
                selectors.insert(solx_standard_json::InputSelector::Metadata);
                let solx_input = solx_standard_json::Input::from_llvm_ir_sources(
                    sources,
                    libraries.to_owned(),
                    solx_standard_json::InputOptimizer::new(
                        llvm_ir_mode.llvm_optimizer_settings.middle_end_as_char(),
                        llvm_ir_mode
                            .llvm_optimizer_settings
                            .is_fallback_to_size_enabled,
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
                solx_standard_json::CollectableError::check_errors(&solx_output)?;

                let mut builds = HashMap::with_capacity(solx_output.contracts.len());
                for (_file, contracts) in solx_output.contracts.into_iter() {
                    for (name, contract) in contracts.into_iter() {
                        let evm = contract.evm.as_ref().ok_or_else(|| {
                            anyhow::anyhow!("EVM object of the contract `{name}` not found")
                        })?;
                        let deploy_code_string = evm
                            .bytecode
                            .as_ref()
                            .ok_or_else(|| {
                                anyhow::anyhow!("EVM bytecode of the contract `{name}` not found")
                            })?
                            .object
                            .as_ref()
                            .ok_or_else(|| {
                                anyhow::anyhow!(
                                    "EVM bytecode object of the contract `{name}` not found"
                                )
                            })?
                            .as_str();
                        let deploy_code = hex::decode(deploy_code_string).expect("Always valid");
                        let runtime_code_size = evm
                            .deployed_bytecode
                            .as_ref()
                            .ok_or_else(|| {
                                anyhow::anyhow!(
                                    "EVM deployed bytecode of the contract `{name}` not found"
                                )
                            })?
                            .object
                            .as_ref()
                            .ok_or_else(|| {
                                anyhow::anyhow!(
                                    "EVM deployed bytecode object of the contract `{name}` not found"
                                )
                            })?
                            .len();
                        builds.insert(name, (deploy_code, runtime_code_size));
                    }
                }
                builds
            }
        };

        Ok(EVMInput::new(builds, None, last_contract))
    }

    fn all_modes(&self) -> Vec<Mode> {
        solx_codegen_evm::OptimizerSettings::combinations()
            .into_iter()
            .map(|llvm_optimizer_settings| LLVMMode::new(llvm_optimizer_settings).into())
            .collect::<Vec<Mode>>()
    }

    fn allows_multi_contract_files(&self) -> bool {
        false
    }
}
