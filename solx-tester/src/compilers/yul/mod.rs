//!
//! The Yul compiler.
//!

pub mod mode;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;

use crate::compilers::Compiler;
use crate::compilers::mode::Mode;
use crate::compilers::solidity::solc::SolidityCompiler as SolcCompiler;
use crate::compilers::solidity::solx::SolidityCompiler as SolxCompiler;
use crate::revm::input::Input as EVMInput;
use crate::toolchain::Toolchain;

use self::mode::Mode as YulMode;

///
/// The Yul compiler.
///
pub enum YulCompiler {
    /// `solx` toolchain.
    Solx(Arc<SolxCompiler>),
    /// `solc` toolchain.
    Solc,
    /// `solx-mlir` toolchain.
    SolxMlir,
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
                let llvm_settings = yul_mode
                    .llvm_optimizer_settings
                    .as_ref()
                    .expect("solx Yul mode must have LLVM settings");

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
                        llvm_settings.middle_end_as_char(),
                        llvm_settings.is_fallback_to_size_enabled,
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

                let last_contract =
                    solx_output.get_last_contract(solx_standard_json::InputLanguage::Yul, &[])?;
                let last_contract = last_contract
                    .rsplit_once(':')
                    .map(|(path, _name)| path.to_owned())
                    .unwrap_or(last_contract);
                let builds = solx_output
                    .extract_bytecode_builds()?
                    .into_iter()
                    .map(|(key, value)| {
                        let key = key
                            .rsplit_once(':')
                            .map(|(path, _name)| path.to_owned())
                            .unwrap_or(key);
                        (key, value)
                    })
                    .collect();

                Ok(EVMInput::new(builds, None, last_contract))
            }
            Self::Solc | Self::SolxMlir => {
                let language = solx_standard_json::InputLanguage::Yul;

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

                let last_contract = solc_output.get_last_contract(language, &sources)?;
                let last_contract = last_contract
                    .rsplit_once(':')
                    .map(|(path, _name)| path.to_owned())
                    .unwrap_or(last_contract);
                let builds = solc_output
                    .extract_bytecode_builds()?
                    .into_iter()
                    .map(|(key, value)| {
                        let key = key
                            .rsplit_once(':')
                            .map(|(path, _name)| path.to_owned())
                            .unwrap_or(key);
                        (key, value)
                    })
                    .collect();

                Ok(EVMInput::new(builds, None, last_contract))
            }
        }
    }

    fn all_modes(&self) -> Vec<Mode> {
        match self {
            Self::Solx(_) => solx_codegen_evm::OptimizerSettings::combinations()
                .into_iter()
                .map(|llvm_optimizer_settings| YulMode::new_solx(llvm_optimizer_settings).into())
                .collect::<Vec<Mode>>(),
            Self::Solc | Self::SolxMlir => {
                // For solc toolchain, delegate to SolcCompiler which generates proper modes
                let language = solx_standard_json::InputLanguage::Yul;
                let solc_compiler = SolcCompiler::new(language, Toolchain::from(self));
                solc_compiler.all_modes()
            }
        }
    }

    fn allows_multi_contract_files(&self) -> bool {
        false
    }
}

impl From<&YulCompiler> for Toolchain {
    fn from(value: &YulCompiler) -> Self {
        match value {
            YulCompiler::Solc => Self::Solc,
            YulCompiler::Solx(_) => Self::Solx,
            YulCompiler::SolxMlir => Self::SolxMlir,
        }
    }
}
