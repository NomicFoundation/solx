//!
//! Unified Solidity/Yul compiler for all toolchains.
//!

pub mod cache_key;
pub mod mode;
pub mod subprocess;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use itertools::Itertools;

use crate::compilers::Compiler;
use crate::compilers::cache::Cache;
use crate::compilers::mode::Mode;
use crate::compilers::yul::mode::Mode as YulMode;
use crate::revm::input::Input as EVMInput;
use crate::toolchain::Toolchain;

use self::cache_key::CacheKey;
use self::mode::Mode as SolidityMode;
use self::subprocess::Subprocess;

///
/// Unified Solidity/Yul compiler for all toolchains.
///
pub struct SolidityCompiler {
    /// The toolchain (Solx, Solc, or SolxMlir).
    toolchain: Toolchain,
    /// The language (Solidity or Yul).
    language: solx_standard_json::InputLanguage,
    /// Path to the executable (for Solx toolchain).
    executable_path: Option<PathBuf>,
    /// Compiler version.
    version: semver::Version,
    /// Cache for compiler outputs.
    cache: Cache<CacheKey, solx_standard_json::Output>,
}

lazy_static::lazy_static! {
    ///
    /// The Solidity compiler supported modes for solc toolchain.
    /// Only optimized modes (E and Y) are generated.
    ///
    static ref SOLIDITY_SOLC_MODES: Vec<Mode> = {
        let mut modes = Vec::new();
        for via_ir in [false, true] {
            for version in SolidityCompiler::all_solc_versions(via_ir).unwrap_or_default() {
                modes.push(SolidityMode::new_solc(version, via_ir, true).into());
            }
        }
        modes
    };

    ///
    /// The Yul compiler supported modes for solc toolchain.
    /// Only optimized mode (Y) is generated.
    ///
    static ref YUL_SOLC_MODES: Vec<Mode> = {
        let mut modes = Vec::new();
        for version in SolidityCompiler::all_solc_versions(true).unwrap_or_default() {
            modes.push(YulMode::new_solc(version, true).into());
        }
        modes
    };

}

impl SolidityCompiler {
    /// The upstream solc executables directory.
    const DIRECTORY_SOLC: &'static str = "solc-bin-upstream/";

    /// The LLVM-fork solc executables directory.
    const DIRECTORY_SOLC_LLVM: &'static str = "solc-bin-llvm/";

    /// The solc allow paths argument value.
    const ALLOW_PATHS: &'static str = "tests";

    /// The first version of `solc` where `--via-ir` is supported.
    const FIRST_VIA_IR_VERSION: semver::Version = semver::Version::new(0, 8, 13);

    /// The current MLIR solc version.
    const MLIR_VERSION: semver::Version = semver::Version::new(0, 8, 30);

    ///
    /// Creates a new compiler for the solx toolchain.
    ///
    pub fn new_solx(
        executable_path: PathBuf,
        language: solx_standard_json::InputLanguage,
    ) -> anyhow::Result<Self> {
        let version = Self::get_solx_version(executable_path.as_path())?;
        Ok(Self {
            toolchain: Toolchain::Solx,
            language,
            executable_path: Some(executable_path),
            version,
            cache: Cache::new(),
        })
    }

    ///
    /// Creates a new compiler for the solc toolchain.
    ///
    pub fn new_solc(language: solx_standard_json::InputLanguage) -> Self {
        Self {
            toolchain: Toolchain::Solc,
            language,
            executable_path: None,
            version: semver::Version::new(0, 0, 0), // Determined per-mode
            cache: Cache::new(),
        }
    }

    ///
    /// Creates a new compiler for the solx-mlir toolchain.
    ///
    pub fn new_solx_mlir(language: solx_standard_json::InputLanguage) -> Self {
        Self {
            toolchain: Toolchain::SolxMlir,
            language,
            executable_path: None,
            version: Self::MLIR_VERSION,
            cache: Cache::new(),
        }
    }

    ///
    /// Returns the solx compiler version.
    ///
    pub fn version(&self) -> &semver::Version {
        &self.version
    }

    ///
    /// Returns the solc versions available for the specified mode.
    ///
    pub fn all_solc_versions(via_ir: bool) -> anyhow::Result<Vec<semver::Version>> {
        let mut versions = Vec::new();
        for entry in std::fs::read_dir(Self::DIRECTORY_SOLC)? {
            let entry = entry?;
            let path = entry.path();
            let entry_type = entry.file_type().map_err(|error| {
                anyhow::anyhow!(
                    "File `{}` type getting error: {}",
                    path.to_string_lossy(),
                    error
                )
            })?;
            if !entry_type.is_file() {
                anyhow::bail!(
                    "Invalid `solc` executable file type: {}",
                    path.to_string_lossy()
                );
            }

            let file_name = entry.file_name().to_string_lossy().to_string();
            let version_str = match file_name.strip_prefix("solc-") {
                Some(version_str) => version_str,
                None => continue,
            };
            let version: semver::Version = match version_str.parse() {
                Ok(version) => version,
                Err(_) => continue,
            };
            if via_ir && version < Self::FIRST_VIA_IR_VERSION {
                continue;
            }

            versions.push(version);
        }
        Ok(versions)
    }

    ///
    /// Gets the solx version from its executable.
    ///
    fn get_solx_version(path: &Path) -> anyhow::Result<semver::Version> {
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
    /// Runs the solx subprocess and returns the output.
    ///
    fn run_solx(
        &self,
        mode: &Mode,
        input: solx_standard_json::Input,
        allow_paths: &[&str],
        debug_output_directory: Option<&Path>,
    ) -> anyhow::Result<solx_standard_json::Output> {
        let path = self
            .executable_path
            .as_ref()
            .expect("solx toolchain must have executable path");

        let mut command = std::process::Command::new(path);
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
            .map_err(|error| anyhow::anyhow!("{:?} subprocess spawning: {error}", path))?;
        let stdin = process
            .stdin
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("{:?} subprocess stdin getting error", path))?;
        let stdin_input = serde_json::to_vec(&input).expect("Always valid");
        stdin.write_all(stdin_input.as_slice()).map_err(|error| {
            anyhow::anyhow!("{:?} subprocess stdin writing: {error:?}", path)
        })?;

        let result = process.wait_with_output().map_err(|error| {
            anyhow::anyhow!("{:?} subprocess output reading: {error:?}", path)
        })?;
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

    ///
    /// Runs the solc subprocess and returns the output.
    ///
    fn run_solc(
        toolchain: Toolchain,
        version: &semver::Version,
        input: solx_standard_json::Input,
        allow_paths: Option<String>,
    ) -> anyhow::Result<solx_standard_json::Output> {
        let directory = match toolchain {
            Toolchain::Solc => Self::DIRECTORY_SOLC,
            Toolchain::SolxMlir => Self::DIRECTORY_SOLC_LLVM,
            toolchain => panic!("Unsupported toolchain for run_solc: {toolchain}"),
        };
        let mut subprocess = Subprocess::new(format!("{directory}/solc-{version}"))?;
        subprocess.standard_json(input, None, vec![], allow_paths)
    }

    ///
    /// Creates input for solx toolchain (Solidity).
    ///
    fn create_solx_solidity_input(
        sources: &[(String, String)],
        libraries: &solx_utils::Libraries,
        mode: &SolidityMode,
        test_params: Option<&solx_solc_test_adapter::Params>,
        llvm_options: Vec<String>,
    ) -> anyhow::Result<solx_standard_json::Input> {
        let llvm_settings = mode
            .llvm_optimizer_settings
            .as_ref()
            .expect("solx mode must have LLVM settings");

        let sources_json: BTreeMap<String, solx_standard_json::InputSource> = sources
            .iter()
            .map(|(path, source)| {
                (
                    path.to_owned(),
                    solx_standard_json::InputSource::from(source.to_owned()),
                )
            })
            .collect();

        let evm_version = test_params.map(|params| params.evm_version.newest_matching());

        let mut selectors = BTreeSet::new();
        selectors.insert(solx_standard_json::InputSelector::Bytecode);
        selectors.insert(solx_standard_json::InputSelector::RuntimeBytecode);
        selectors.insert(solx_standard_json::InputSelector::AST);
        selectors.insert(solx_standard_json::InputSelector::MethodIdentifiers);
        selectors.insert(solx_standard_json::InputSelector::Metadata);
        selectors.insert(if mode.via_ir {
            solx_standard_json::InputSelector::Yul
        } else {
            solx_standard_json::InputSelector::EVMLegacyAssembly
        });

        solx_standard_json::Input::try_from_solidity_sources(
            sources_json,
            libraries.clone(),
            BTreeSet::new(),
            solx_standard_json::InputOptimizer::new(
                llvm_settings.middle_end_as_char(),
                llvm_settings.is_fallback_to_size_enabled,
            ),
            evm_version,
            mode.via_ir,
            &solx_standard_json::InputSelection::new(selectors),
            solx_standard_json::InputMetadata::default(),
            llvm_options,
        )
        .map_err(|error| anyhow::anyhow!("Solidity standard JSON I/O error: {error}"))
    }

    ///
    /// Creates input for solx toolchain (Yul).
    ///
    fn create_solx_yul_input(
        sources: &[(String, String)],
        libraries: &solx_utils::Libraries,
        mode: &YulMode,
        llvm_options: Vec<String>,
    ) -> solx_standard_json::Input {
        let llvm_settings = mode
            .llvm_optimizer_settings
            .as_ref()
            .expect("solx Yul mode must have LLVM settings");

        let sources_json: BTreeMap<String, solx_standard_json::InputSource> = sources
            .iter()
            .map(|(path, source)| {
                (
                    path.to_owned(),
                    solx_standard_json::InputSource::from(source.to_owned()),
                )
            })
            .collect();

        let mut selectors = BTreeSet::new();
        selectors.insert(solx_standard_json::InputSelector::Bytecode);
        selectors.insert(solx_standard_json::InputSelector::RuntimeBytecode);
        selectors.insert(solx_standard_json::InputSelector::AST);
        selectors.insert(solx_standard_json::InputSelector::MethodIdentifiers);
        selectors.insert(solx_standard_json::InputSelector::Metadata);
        selectors.insert(solx_standard_json::InputSelector::Yul);

        solx_standard_json::Input::from_yul_sources(
            sources_json,
            libraries.clone(),
            solx_standard_json::InputOptimizer::new(
                llvm_settings.middle_end_as_char(),
                llvm_settings.is_fallback_to_size_enabled,
            ),
            &solx_standard_json::InputSelection::new(selectors),
            solx_standard_json::InputMetadata::default(),
            llvm_options,
        )
    }

    ///
    /// Creates input for solc toolchain.
    ///
    fn create_solc_input(
        language: solx_standard_json::InputLanguage,
        sources: &[(String, String)],
        libraries: &solx_utils::Libraries,
        mode: &Mode,
        test_params: Option<&solx_solc_test_adapter::Params>,
    ) -> solx_standard_json::Input {
        let (solc_version, via_ir, optimizer_enabled) = match mode {
            Mode::Solidity(mode) => (
                &mode.solc_version,
                mode.via_ir,
                mode.solc_optimize.unwrap_or(false),
            ),
            Mode::Yul(mode) => {
                let version = mode
                    .solc_version
                    .as_ref()
                    .expect("Yul mode requires solc_version for solc toolchain");
                (version, true, mode.solc_optimize.unwrap_or(false))
            }
            mode => panic!("Unsupported mode for solc input: {mode}"),
        };

        let output_selection =
            solx_standard_json::InputSelection::new_required_for_testing(via_ir);

        let evm_version = match mode {
            Mode::Solidity(_) => test_params.map(|params| params.evm_version.newest_matching()),
            Mode::Yul(_) => Some(solx_utils::EVMVersion::Cancun),
            _ => None,
        };

        let debug = if solc_version >= &semver::Version::new(0, 6, 3) {
            test_params.map(|test_params| {
                solx_standard_json::InputDebug::new(Some(test_params.revert_strings.to_string()))
            })
        } else {
            None
        };

        solx_standard_json::Input::new_for_solc(
            language,
            sources.iter().cloned().collect(),
            libraries.clone(),
            None,
            evm_version,
            via_ir,
            output_selection,
            optimizer_enabled,
            debug,
        )
    }

    ///
    /// Compiles using solc toolchain with caching.
    ///
    fn compile_solc_cached(
        &self,
        test_path: String,
        sources: &[(String, String)],
        libraries: &solx_utils::Libraries,
        mode: &Mode,
        test_params: Option<&solx_solc_test_adapter::Params>,
    ) -> anyhow::Result<solx_standard_json::Output> {
        let cache_key = match mode {
            Mode::Solidity(mode) => CacheKey::new(
                test_path,
                mode.solc_version.to_owned(),
                mode.via_ir,
                mode.solc_optimize.unwrap_or(false),
            ),
            Mode::Yul(mode) => {
                let version = mode
                    .solc_version
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Yul mode requires solc_version for caching"))?
                    .to_owned();
                CacheKey::new(
                    test_path,
                    version,
                    true, // Yul is always via_ir
                    mode.solc_optimize.unwrap_or(false),
                )
            }
            mode => anyhow::bail!("Unsupported mode for caching: {mode}"),
        };

        if !self.cache.contains(&cache_key) {
            let solc_version = match mode {
                Mode::Solidity(mode) => &mode.solc_version,
                Mode::Yul(mode) => mode.solc_version.as_ref().unwrap(),
                _ => unreachable!(),
            };

            let input =
                Self::create_solc_input(self.language, sources, libraries, mode, test_params);

            let allow_paths = Path::new(Self::ALLOW_PATHS)
                .canonicalize()
                .expect("Always valid")
                .to_string_lossy()
                .to_string();

            self.cache.evaluate(cache_key.clone(), || {
                Self::run_solc(self.toolchain, solc_version, input, Some(allow_paths))
            });
        }

        self.cache.get_cloned(&cache_key)
    }

    ///
    /// Compiles for EVM using solx toolchain.
    ///
    fn compile_solx_for_evm(
        &self,
        sources: Vec<(String, String)>,
        libraries: solx_utils::Libraries,
        mode: &Mode,
        test_params: Option<&solx_solc_test_adapter::Params>,
        llvm_options: Vec<String>,
        debug_config: Option<solx_codegen_evm::DebugConfig>,
    ) -> anyhow::Result<EVMInput> {
        let allow_path = Path::new(Self::ALLOW_PATHS)
            .canonicalize()
            .expect("Always valid")
            .to_string_lossy()
            .to_string();

        let output = match self.language {
            solx_standard_json::InputLanguage::Solidity => {
                let solidity_mode = SolidityMode::unwrap(mode);
                let input = Self::create_solx_solidity_input(
                    &sources,
                    &libraries,
                    solidity_mode,
                    test_params,
                    llvm_options,
                )?;

                self.run_solx(
                    mode,
                    input,
                    &[allow_path.as_str()],
                    debug_config
                        .as_ref()
                        .map(|config| config.output_directory.as_path()),
                )?
            }
            solx_standard_json::InputLanguage::Yul => {
                let yul_mode = YulMode::unwrap(mode);
                let input =
                    Self::create_solx_yul_input(&sources, &libraries, yul_mode, llvm_options);

                self.run_solx(
                    mode,
                    input,
                    &[],
                    debug_config
                        .as_ref()
                        .map(|config| config.output_directory.as_path()),
                )?
            }
            solx_standard_json::InputLanguage::LLVMIR => {
                anyhow::bail!("LLVM IR language should use the LLVM compiler")
            }
        };

        solx_standard_json::CollectableError::check_errors(&output)?;

        let method_identifiers = match self.language {
            solx_standard_json::InputLanguage::Solidity => Some(output.get_method_identifiers()?),
            _ => None,
        };

        let last_contract = output.get_last_contract(self.language, &sources)?;
        let builds = output.extract_bytecode_builds()?;

        // For Yul, strip the contract name suffix
        if self.language == solx_standard_json::InputLanguage::Yul {
            let last_contract = last_contract
                .rsplit_once(':')
                .map(|(path, _name)| path.to_owned())
                .unwrap_or(last_contract);
            let builds = builds
                .into_iter()
                .map(|(key, value)| {
                    let key = key
                        .rsplit_once(':')
                        .map(|(path, _name)| path.to_owned())
                        .unwrap_or(key);
                    (key, value)
                })
                .collect();
            return Ok(EVMInput::new(builds, method_identifiers, last_contract));
        }

        Ok(EVMInput::new(builds, method_identifiers, last_contract))
    }

    ///
    /// Compiles for EVM using solc/solx-mlir toolchain.
    ///
    fn compile_solc_for_evm(
        &self,
        test_path: String,
        sources: Vec<(String, String)>,
        libraries: solx_utils::Libraries,
        mode: &Mode,
        test_params: Option<&solx_solc_test_adapter::Params>,
    ) -> anyhow::Result<EVMInput> {
        let output =
            self.compile_solc_cached(test_path, &sources, &libraries, mode, test_params)?;

        if let Some(errors) = output.errors_opt() {
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

        let method_identifiers = match self.language {
            solx_standard_json::InputLanguage::Solidity => Some(output.get_method_identifiers()?),
            solx_standard_json::InputLanguage::Yul => None,
            solx_standard_json::InputLanguage::LLVMIR => {
                anyhow::bail!("LLVM IR language is not supported by solc")
            }
        };

        let last_contract = output.get_last_contract(self.language, &sources)?;
        let builds = output.extract_bytecode_builds()?;

        // For Yul, strip the contract name suffix
        if self.language == solx_standard_json::InputLanguage::Yul {
            let last_contract = last_contract
                .rsplit_once(':')
                .map(|(path, _name)| path.to_owned())
                .unwrap_or(last_contract);
            let builds = builds
                .into_iter()
                .map(|(key, value)| {
                    let key = key
                        .rsplit_once(':')
                        .map(|(path, _name)| path.to_owned())
                        .unwrap_or(key);
                    (key, value)
                })
                .collect();
            return Ok(EVMInput::new(builds, method_identifiers, last_contract));
        }

        Ok(EVMInput::new(builds, method_identifiers, last_contract))
    }
}

impl Compiler for SolidityCompiler {
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
        match self.toolchain {
            Toolchain::Solx => self.compile_solx_for_evm(
                sources,
                libraries,
                mode,
                test_params,
                llvm_options,
                debug_config,
            ),
            Toolchain::Solc | Toolchain::SolxMlir => {
                self.compile_solc_for_evm(test_path, sources, libraries, mode, test_params)
            }
        }
    }

    fn all_modes(&self) -> Vec<Mode> {
        match (self.language, self.toolchain) {
            (solx_standard_json::InputLanguage::Solidity, Toolchain::Solx) => {
                let mut codegen_versions = Vec::new();
                for via_ir in [false, true] {
                    codegen_versions.push((via_ir, self.version.to_owned()));
                }

                solx_codegen_evm::OptimizerSettings::combinations()
                    .into_iter()
                    .cartesian_product(codegen_versions)
                    .map(|(llvm_optimizer_settings, (via_ir, version))| {
                        SolidityMode::new_solx(version, via_ir, llvm_optimizer_settings).into()
                    })
                    .collect::<Vec<Mode>>()
            }
            (solx_standard_json::InputLanguage::Solidity, Toolchain::SolxMlir) => {
                // SolxMlir uses LLVM optimizer settings with fixed version
                solx_codegen_evm::OptimizerSettings::combinations()
                    .into_iter()
                    .map(|llvm_optimizer_settings| {
                        SolidityMode::new_solx(Self::MLIR_VERSION, true, llvm_optimizer_settings)
                            .into()
                    })
                    .collect()
            }
            (solx_standard_json::InputLanguage::Solidity, Toolchain::Solc) => {
                SOLIDITY_SOLC_MODES.clone()
            }
            (solx_standard_json::InputLanguage::Yul, Toolchain::Solx) => {
                solx_codegen_evm::OptimizerSettings::combinations()
                    .into_iter()
                    .map(|llvm_optimizer_settings| {
                        YulMode::new_solx(llvm_optimizer_settings).into()
                    })
                    .collect::<Vec<Mode>>()
            }
            (solx_standard_json::InputLanguage::Yul, Toolchain::SolxMlir) => {
                // SolxMlir uses LLVM optimizer settings
                solx_codegen_evm::OptimizerSettings::combinations()
                    .into_iter()
                    .map(|llvm_optimizer_settings| {
                        YulMode::new_solx(llvm_optimizer_settings).into()
                    })
                    .collect()
            }
            (solx_standard_json::InputLanguage::Yul, Toolchain::Solc) => YUL_SOLC_MODES.clone(),
            (solx_standard_json::InputLanguage::LLVMIR, _) => Vec::new(),
        }
    }

    fn allows_multi_contract_files(&self) -> bool {
        self.language != solx_standard_json::InputLanguage::Yul
    }
}
