//!
//! The LLVM IR compiler.
//!

pub mod mode;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use crate::compilers::Compiler;
use crate::compilers::mode::Mode;
use crate::revm::input::Input as EVMInput;
use crate::toolchain::Toolchain;

use self::mode::Mode as LLVMMode;

///
/// The LLVM IR compiler.
///
/// Only solx toolchain supports LLVM IR compilation.
///
pub struct LLVMIRCompiler {
    /// The toolchain.
    toolchain: Toolchain,
    /// Path to the solx executable (for solx toolchain).
    executable_path: Option<PathBuf>,
}

impl LLVMIRCompiler {
    ///
    /// Creates a new LLVM IR compiler.
    /// Only solx toolchain supports LLVM IR compilation.
    ///
    pub fn new(executable_path: PathBuf) -> Self {
        Self {
            toolchain: Toolchain::Solx,
            executable_path: Some(executable_path),
        }
    }

    ///
    /// Creates a placeholder LLVM IR compiler for solc toolchain.
    /// LLVM IR is not supported by solc, so this returns empty modes.
    ///
    pub fn new_solc() -> Self {
        Self {
            toolchain: Toolchain::Solc,
            executable_path: None,
        }
    }

    ///
    /// Runs the solx subprocess for LLVM IR compilation.
    ///
    fn run_solx(
        &self,
        mode: &Mode,
        input: solx_standard_json::Input,
        debug_output_directory: Option<&Path>,
    ) -> anyhow::Result<solx_standard_json::Output> {
        let path = self
            .executable_path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("LLVM IR compilation requires solx toolchain"))?;

        let mut command = std::process::Command::new(path);
        command.stdin(std::process::Stdio::piped());
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());
        command.arg("--standard-json");

        if let Some(debug_output_directory) = debug_output_directory {
            let mut output_directory = debug_output_directory.to_owned();
            output_directory.push(mode.to_string());

            command.arg("--debug-output-dir");
            command.arg(output_directory);
        }

        let mut process = command
            .spawn()
            .map_err(|error| anyhow::anyhow!("{:?} subprocess spawning: {error}", path))?;
        let stdin = process
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("{:?} subprocess stdin getting error", path))?;
        let stdin_input = serde_json::to_vec(&input).expect("Always valid");
        stdin
            .write_all(stdin_input.as_slice())
            .map_err(|error| anyhow::anyhow!("{:?} subprocess stdin writing: {error:?}", path))?;

        let result = process
            .wait_with_output()
            .map_err(|error| anyhow::anyhow!("{:?} subprocess output reading: {error:?}", path))?;
        if !result.status.success() {
            anyhow::bail!(
                "{:?} subprocess failed with exit code {:?}:\n{}\n{}",
                path,
                result.status.code(),
                String::from_utf8_lossy(result.stdout.as_slice()),
                String::from_utf8_lossy(result.stderr.as_slice()),
            );
        }

        solx_utils::deserialize_from_slice::<solx_standard_json::Output>(result.stdout.as_slice())
            .map_err(|error| {
                anyhow::anyhow!(
                    "{:?} subprocess stdout parsing: {error:?} (stderr: {})",
                    path,
                    String::from_utf8_lossy(result.stderr.as_slice()),
                )
            })
    }
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
        debug_config: Option<solx_codegen_evm::OutputConfig>,
    ) -> anyhow::Result<EVMInput> {
        if self.toolchain != Toolchain::Solx {
            anyhow::bail!("LLVM IR compilation is only supported by solx toolchain");
        }

        let llvm_ir_mode = LLVMMode::unwrap(mode);

        let last_contract = sources
            .last()
            .ok_or_else(|| anyhow::anyhow!("LLVM IR sources are empty"))?
            .0
            .clone();

        let sources_json: BTreeMap<String, solx_standard_json::InputSource> = sources
            .iter()
            .map(|(path, source)| {
                (
                    path.to_owned(),
                    solx_standard_json::InputSource {
                        content: Some(source.to_owned()),
                        urls: None,
                    },
                )
            })
            .collect();

        let mut selectors = BTreeSet::new();
        selectors.insert(solx_standard_json::InputSelector::Bytecode);
        selectors.insert(solx_standard_json::InputSelector::RuntimeBytecode);
        selectors.insert(solx_standard_json::InputSelector::Metadata);

        let input = crate::compilers::input_ext::new_input_from_llvm_ir_sources(
            sources_json,
            libraries,
            solx_standard_json::InputOptimizer {
                enabled: None,
                mode: Some(llvm_ir_mode.llvm_optimizer_settings.middle_end_as_char()),
                size_fallback: Some(
                    llvm_ir_mode
                        .llvm_optimizer_settings
                        .is_fallback_to_size_enabled,
                ),
            },
            &solx_standard_json::InputSelection::new(selectors),
            solx_standard_json::InputMetadata::default(),
            llvm_options,
        );

        let output = self.run_solx(
            mode,
            input,
            debug_config
                .as_ref()
                .map(|config| config.output_directory.as_path()),
        )?;
        solx_standard_json::CollectableError::check_errors(&output)?;

        let mut builds = HashMap::with_capacity(output.contracts.len());
        for (_file, contracts) in output.contracts.into_iter() {
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
                        anyhow::anyhow!("EVM bytecode object of the contract `{name}` not found")
                    })?
                    .as_str();
                let deploy_code = hex::decode(deploy_code_string).expect("Always valid");
                let runtime_code_size = evm
                    .deployed_bytecode
                    .as_ref()
                    .ok_or_else(|| {
                        anyhow::anyhow!("EVM deployed bytecode of the contract `{name}` not found")
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

        Ok(EVMInput::new(builds, None, last_contract))
    }

    fn all_modes(&self) -> Vec<Mode> {
        // Only solx supports LLVM IR
        if self.toolchain != Toolchain::Solx {
            return Vec::new();
        }

        super::optimizer_combinations()
            .into_iter()
            .map(|llvm_optimizer_settings| LLVMMode::new(llvm_optimizer_settings).into())
            .collect::<Vec<Mode>>()
    }

    fn allows_multi_contract_files(&self) -> bool {
        false
    }
}
