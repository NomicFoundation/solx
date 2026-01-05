//!
//! `solx` compiler.
//!

pub mod mode;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use itertools::Itertools;

use solx_standard_json::CollectableError;

use crate::compilers::mode::Mode;
use crate::compilers::Compiler;
use crate::revm::input::Input as EVMInput;

use self::mode::Mode as SolxMode;

///
/// `solx` compiler.
///
pub struct SolidityCompiler {
    /// Path to the `solx` executable.
    pub path: PathBuf,
    /// `solx` compiler version.
    pub version: semver::Version,
}

impl SolidityCompiler {
    /// The solc's `allow-paths` argument value.
    const SOLC_ALLOW_PATHS: &'static str = "tests";

    ///
    /// A shortcut constructor.
    ///
    pub fn try_from_path(path: PathBuf) -> anyhow::Result<Self> {
        let version = Self::version(path.as_path())?;
        Ok(Self { path, version })
    }

    ///
    /// Runs the `solx` subprocess and returns the output.
    ///
    pub fn standard_json(
        &self,
        mode: &Mode,
        solx_input: solx_standard_json::Input,
        allow_paths: &[&str],
        debug_output_directory: Option<&Path>,
    ) -> anyhow::Result<solx_standard_json::Output> {
        let mut command = std::process::Command::new(self.path.as_path());
        command.stdin(std::process::Stdio::piped());
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());
        command.arg("--standard-json");
        if !allow_paths.is_empty() {
            command.arg("--allow-paths");
            command.args(allow_paths);
        }
        if let Some(debug_output_directory) = debug_output_directory {
            let mut output_directory = debug_output_directory.to_owned();
            output_directory.push(mode.to_string());

            command.arg("--debug-output-dir");
            command.arg(output_directory);
        }

        let mut process = command
            .spawn()
            .map_err(|error| anyhow::anyhow!("{:?} subprocess spawning: {error}", self.path))?;
        let stdin = process
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("{:?} subprocess stdin getting error", self.path))?;
        let stdin_input = serde_json::to_vec(&solx_input).expect("Always valid");
        stdin.write_all(stdin_input.as_slice()).map_err(|error| {
            anyhow::anyhow!("{:?} subprocess stdin writing: {error:?}", self.path)
        })?;

        let result = process.wait_with_output().map_err(|error| {
            anyhow::anyhow!("{:?} subprocess output reading: {error:?}", self.path)
        })?;
        if !result.status.success() {
            anyhow::bail!(
                "{:?} subprocess failed with exit code {:?}:\n{}\n{}",
                self.path,
                result.status.code(),
                String::from_utf8_lossy(result.stdout.as_slice()),
                String::from_utf8_lossy(result.stderr.as_slice()),
            );
        }

        solx_utils::deserialize_from_slice::<solx_standard_json::Output>(result.stdout.as_slice())
            .map_err(|error| {
                anyhow::anyhow!(
                    "{:?} subprocess stdout parsing: {error:?} (stderr: {})",
                    self.path,
                    String::from_utf8_lossy(result.stderr.as_slice()),
                )
            })
    }

    ///
    /// Runs the `solx` subprocess and returns its version.
    ///
    pub fn version(path: &Path) -> anyhow::Result<semver::Version> {
        let mut command = std::process::Command::new(path);
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());
        command.arg("--version");

        let process = command
            .spawn()
            .map_err(|error| anyhow::anyhow!("{path:?} subprocess spawning: {error}"))?;
        let result = process
            .wait_with_output()
            .map_err(|error| anyhow::anyhow!("{path:?} subprocess output reading: {error:?}"))?;
        if !result.status.success() {
            anyhow::bail!(
                "{path:?} subprocess exit code {:?}:\n{}\n{}",
                result.status.code(),
                String::from_utf8_lossy(result.stdout.as_slice()),
                String::from_utf8_lossy(result.stderr.as_slice()),
            );
        }

        let version = String::from_utf8_lossy(result.stdout.as_slice())
            .lines()
            .nth(1)
            .ok_or_else(|| {
                anyhow::anyhow!("{path:?} subprocess version getting: missing 2nd line")
            })?
            .split(' ')
            .nth(1)
            .ok_or_else(|| anyhow::anyhow!("{path:?} subprocess version getting: missing version"))?
            .split('+')
            .next()
            .ok_or_else(|| anyhow::anyhow!("{path:?} subprocess version getting: missing semver"))?
            .parse::<semver::Version>()
            .map_err(|error| anyhow::anyhow!("{path:?} subprocess version parsing: {error}"))?;
        Ok(version)
    }

    ///
    /// Get the method identifiers from the solc output.
    ///
    fn get_method_identifiers(
        solc_output: &solx_standard_json::Output,
    ) -> anyhow::Result<BTreeMap<String, BTreeMap<String, u32>>> {
        let mut selectors = BTreeMap::new();
        for (path, file) in solc_output.contracts.iter() {
            for (name, contract) in file.iter() {
                let mut contract_selectors = BTreeMap::new();
                let contract_method_identifiers = match contract
                    .evm
                    .as_ref()
                    .and_then(|evm| evm.method_identifiers.as_ref())
                {
                    Some(method_identifiers) => method_identifiers,
                    None => {
                        continue;
                    }
                };
                for (entry, selector) in contract_method_identifiers.iter() {
                    let selector =
                        u32::from_str_radix(selector, solx_utils::BASE_HEXADECIMAL)
                            .map_err(|error| {
                                anyhow::anyhow!(
                                    "Invalid selector `{selector}` received from the Solidity compiler: {error}"
                                )
                            })?;
                    contract_selectors.insert(entry.clone(), selector);
                }
                selectors.insert(format!("{path}:{name}"), contract_selectors);
            }
        }
        Ok(selectors)
    }

    ///
    /// Get the last contract from the solc output.
    ///
    fn get_last_contract(
        solc_output: &solx_standard_json::Output,
        sources: &[(String, String)],
    ) -> anyhow::Result<String> {
        for (path, _source) in sources.iter().rev() {
            match Self::last_contract_name(
                solc_output
                    .sources
                    .get(path)
                    .ok_or_else(|| anyhow::anyhow!("The last source not found in the output"))?,
            ) {
                Ok(name) => return Ok(format!("{path}:{name}")),
                Err(_error) => continue,
            }
        }
        anyhow::bail!("The last source not found in the output")
    }

    ///
    /// Returns the last contract name from the sources.
    ///
    fn last_contract_name(source: &solx_standard_json::OutputSource) -> anyhow::Result<String> {
        source
            .ast
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("The AST is empty"))?
            .get("nodes")
            .and_then(|value| value.as_array())
            .ok_or_else(|| {
                anyhow::anyhow!("The last contract cannot be found in an empty list of nodes")
            })?
            .iter()
            .filter_map(
                |node| match node.get("nodeType").and_then(|node| node.as_str()) {
                    Some("ContractDefinition") => Some(node.get("name")?.as_str()?.to_owned()),
                    _ => None,
                },
            )
            .next_back()
            .ok_or_else(|| anyhow::anyhow!("The last contract not found in the AST"))
    }
}

impl Compiler for SolidityCompiler {
    fn compile_for_evm(
        &self,
        _test_path: String,
        sources: Vec<(String, String)>,
        libraries: solx_utils::Libraries,
        mode: &Mode,
        test_params: Option<&solx_solc_test_adapter::Params>,
        llvm_options: Vec<String>,
        debug_config: Option<solx_codegen_evm::DebugConfig>,
    ) -> anyhow::Result<EVMInput> {
        let solx_mode = SolxMode::unwrap(mode);

        let sources_json: BTreeMap<String, solx_standard_json::InputSource> = sources
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

        let evm_version = test_params.map(|params| params.evm_version.newest_matching());

        let mut selectors = BTreeSet::new();
        selectors.insert(solx_standard_json::InputSelector::Bytecode);
        selectors.insert(solx_standard_json::InputSelector::RuntimeBytecode);
        selectors.insert(solx_standard_json::InputSelector::AST);
        selectors.insert(solx_standard_json::InputSelector::MethodIdentifiers);
        selectors.insert(solx_standard_json::InputSelector::Metadata);
        selectors.insert(if solx_mode.via_ir {
            solx_standard_json::InputSelector::Yul
        } else {
            solx_standard_json::InputSelector::EVMLegacyAssembly
        });
        let solx_input = solx_standard_json::Input::try_from_solidity_sources(
            sources_json,
            libraries,
            BTreeSet::new(),
            solx_standard_json::InputOptimizer::new(
                solx_mode.llvm_optimizer_settings.middle_end_as_char(),
                solx_mode
                    .llvm_optimizer_settings
                    .is_fallback_to_size_enabled,
            ),
            evm_version,
            solx_mode.via_ir,
            &solx_standard_json::InputSelection::new(selectors),
            solx_standard_json::InputMetadata::default(),
            llvm_options,
        )
        .map_err(|error| anyhow::anyhow!("Solidity standard JSON I/O error: {error}"))?;

        let allow_path = Path::new(Self::SOLC_ALLOW_PATHS)
            .canonicalize()
            .expect("Always valid")
            .to_string_lossy()
            .to_string();

        let solx_output = self.standard_json(
            mode,
            solx_input,
            &[allow_path.as_str()],
            debug_config
                .as_ref()
                .map(|debug_config| debug_config.output_directory.as_path()),
        )?;
        solx_output.check_errors()?;

        let method_identifiers = Self::get_method_identifiers(&solx_output)?;

        let last_contract = Self::get_last_contract(&solx_output, &sources)?;

        let mut builds = HashMap::with_capacity(solx_output.contracts.len());
        for (file, source) in solx_output.contracts.iter() {
            for (name, contract) in source.iter() {
                let deploy_code = match contract
                    .evm
                    .as_ref()
                    .and_then(|evm| evm.bytecode.as_ref())
                    .and_then(|bytecode| bytecode.object.as_ref())
                {
                    Some(bytecode) => hex::decode(bytecode.as_str())?,
                    None => continue,
                };
                let runtime_code_size = match contract
                    .evm
                    .as_ref()
                    .and_then(|evm| evm.deployed_bytecode.as_ref())
                    .and_then(|deployed_bytecode| deployed_bytecode.object.as_ref())
                {
                    Some(deployed_bytecode) => deployed_bytecode.len() / 2,
                    None => 0,
                };
                builds.insert(format!("{file}:{name}"), (deploy_code, runtime_code_size));
            }
        }

        Ok(EVMInput::new(
            builds,
            Some(method_identifiers),
            last_contract,
        ))
    }

    fn all_modes(&self) -> Vec<Mode> {
        let mut solc_codegen_versions = Vec::new();
        for via_ir in [false, true] {
            solc_codegen_versions.push((via_ir, self.version.to_owned()));
        }

        solx_codegen_evm::OptimizerSettings::combinations()
            .into_iter()
            .cartesian_product(solc_codegen_versions)
            .map(|(llvm_optimizer_settings, (via_ir, version))| {
                SolxMode::new(version, via_ir, false, llvm_optimizer_settings).into()
            })
            .collect::<Vec<Mode>>()
    }

    fn allows_multi_contract_files(&self) -> bool {
        true
    }
}
