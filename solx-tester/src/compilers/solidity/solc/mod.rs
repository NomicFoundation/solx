//!
//! The `solc` Solidity compiler.
//!

pub mod compiler;

use std::path::Path;

use crate::compilers::Compiler;
use crate::compilers::cache::Cache;
use crate::compilers::mode::Mode;
use crate::compilers::solidity::cache_key::CacheKey;
use crate::compilers::solidity::mode::Mode as SolidityMode;
use crate::compilers::yul::mode::Mode as YulMode;
use crate::revm::input::Input as EVMInput;
use crate::toolchain::Toolchain;

use self::compiler::Compiler as SolcUpstreamCompiler;

///
/// The `solc` Solidity compiler.
///
pub struct SolidityCompiler {
    /// The language the compiler will compile.
    language: solx_standard_json::InputLanguage,
    /// The toolchain identifier.
    /// Only `solc` and `solx-mlir` are supported.
    toolchain: Toolchain,
    /// The `solc` process output cache.
    cache: Cache<CacheKey, solx_standard_json::Output>,
}

lazy_static::lazy_static! {
    ///
    /// The Solidity compiler supported modes.
    ///
    /// All compilers must be downloaded before initialization.
    ///
    static ref SOLIDITY_MODES: Vec<Mode> = {
        let mut modes = Vec::new();
        for (via_ir, optimize) in [
            (false, false),
            (false, true),
            (true, false),
            (true, true),
        ] {
            for version in SolidityCompiler::all_versions(via_ir).expect("`solc` versions analysis error") {
                modes.push(SolidityMode::new_solc(version, via_ir, optimize).into());
            }
        }
        modes
    };

    ///
    /// The Yul compiler supported modes.
    ///
    /// All compilers must be downloaded before initialization.
    ///
    static ref YUL_MODES: Vec<Mode> = {
        let mut modes = Vec::new();
        for optimize in [false, true] {
            for version in SolidityCompiler::all_versions(true).expect("`solc` versions analysis error") {
                modes.push(YulMode::new_solc(version, optimize).into());
            }
        }
        modes
    };

    ///
    /// The supported Solidity modes for MLIR codegen.
    ///
    /// All compilers must be downloaded before initialization.
    ///
    static ref SOLIDITY_MLIR_MODES: Vec<Mode> = {
        solx_codegen_evm::OptimizerSettings::combinations()
            .into_iter()
            .map(|llvm_optimizer_settings| {
                SolidityMode::new_solx(
                    SolidityCompiler::CURRENT_MLIR_VERSION,
                    true,  // via_ir always true for MLIR
                    llvm_optimizer_settings,
                ).into()
            })
            .collect::<Vec<Mode>>()
    };

    ///
    /// The supported Yul modes for MLIR codegen.
    ///
    /// All compilers must be downloaded before initialization.
    ///
    static ref YUL_MLIR_MODES: Vec<Mode> = {
        solx_codegen_evm::OptimizerSettings::combinations()
            .into_iter()
            .map(|llvm_optimizer_settings| {
                YulMode::new_solx(llvm_optimizer_settings).into()
            })
            .collect::<Vec<Mode>>()
    };
}

impl SolidityCompiler {
    /// The upstream compiler executables directory.
    const DIRECTORY_UPSTREAM: &'static str = "solc-bin-upstream/";

    /// The LLVM-fork compiler executables directory.
    const DIRECTORY_LLVM: &'static str = "solc-bin-llvm/";

    /// The solc allow paths argument value.
    const SOLC_ALLOW_PATHS: &'static str = "tests";

    /// The current MLIR solc version.
    const CURRENT_MLIR_VERSION: semver::Version = semver::Version::new(0, 8, 30);

    ///
    /// A shortcut constructor.
    ///
    pub fn new(language: solx_standard_json::InputLanguage, toolchain: Toolchain) -> Self {
        Self {
            language,
            toolchain,
            cache: Cache::new(),
        }
    }

    ///
    /// Returns the `solc` executable by its version.
    ///
    pub fn executable(
        toolchain: Toolchain,
        version: &semver::Version,
    ) -> anyhow::Result<SolcUpstreamCompiler> {
        let directory = match toolchain {
            Toolchain::Solc => Self::DIRECTORY_UPSTREAM,
            Toolchain::SolxMlir => Self::DIRECTORY_LLVM,
            toolchain => panic!("Unsupported toolchain: {toolchain}"),
        };
        SolcUpstreamCompiler::new(format!("{directory}/solc-{version}"))
    }

    ///
    /// Returns the compiler versions downloaded for the specified compilation mode.
    ///
    pub fn all_versions(via_ir: bool) -> anyhow::Result<Vec<semver::Version>> {
        let mut versions = Vec::new();
        for entry in std::fs::read_dir(Self::DIRECTORY_UPSTREAM)? {
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
            if via_ir && version < SolcUpstreamCompiler::FIRST_VIA_IR_VERSION {
                continue;
            }

            versions.push(version);
        }
        Ok(versions)
    }

    ///
    /// Runs the solc subprocess and returns the output.
    ///
    pub fn standard_json_output(
        language: solx_standard_json::InputLanguage,
        toolchain: Toolchain,
        sources: &[(String, String)],
        libraries: &solx_utils::Libraries,
        mode: &Mode,
        test_params: Option<&solx_solc_test_adapter::Params>,
    ) -> anyhow::Result<solx_standard_json::Output> {
        let (solc_version, via_ir, optimizer_enabled) = match mode {
            Mode::Solidity(mode) => (
                &mode.solc_version,
                mode.via_ir,
                mode.solc_optimize.unwrap_or(false),
            ),
            Mode::Yul(mode) => {
                let version = mode.solc_version.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("Yul mode requires solc_version for solc toolchain")
                })?;
                (version, true, mode.solc_optimize.unwrap_or(false))
            }
            mode => anyhow::bail!("Unsupported mode: {mode}"),
        };

        let mut solc = Self::executable(toolchain, solc_version)?;

        let output_selection = solx_standard_json::InputSelection::new_required_for_testing(via_ir);

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

        let solc_input = solx_standard_json::Input::new_for_solc(
            language,
            sources.iter().cloned().collect(),
            libraries.clone(),
            None,
            evm_version,
            via_ir,
            output_selection,
            optimizer_enabled,
            debug,
        );

        let allow_paths = Path::new(Self::SOLC_ALLOW_PATHS)
            .canonicalize()
            .expect("Always valid")
            .to_string_lossy()
            .to_string();

        solc.standard_json(solc_input, None, vec![], Some(allow_paths))
    }

    ///
    /// Evaluates the standard JSON output or loads it from the cache.
    ///
    pub fn standard_json_output_cached(
        &self,
        test_path: String,
        language: solx_standard_json::InputLanguage,
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
            mode => anyhow::bail!("Unsupported mode: {mode}"),
        };

        if !self.cache.contains(&cache_key) {
            self.cache.evaluate(cache_key.clone(), || {
                Self::standard_json_output(
                    language,
                    self.toolchain,
                    sources,
                    libraries,
                    mode,
                    test_params,
                )
            });
        }

        self.cache.get_cloned(&cache_key)
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
        _llvm_options: Vec<String>,
        _debug_config: Option<solx_codegen_evm::DebugConfig>,
    ) -> anyhow::Result<EVMInput> {
        let solc_output = self.standard_json_output_cached(
            test_path,
            self.language,
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

        let method_identifiers = match self.language {
            solx_standard_json::InputLanguage::Solidity => {
                Some(solc_output.get_method_identifiers()?)
            }
            solx_standard_json::InputLanguage::Yul => None,
            solx_standard_json::InputLanguage::LLVMIR => {
                anyhow::bail!("LLVM IR language is not supported by solc")
            }
        };

        let last_contract = solc_output.get_last_contract(self.language, &sources)?;
        let builds = solc_output.extract_bytecode_builds()?;

        Ok(EVMInput::new(builds, method_identifiers, last_contract))
    }

    fn all_modes(&self) -> Vec<Mode> {
        match (self.language, self.toolchain) {
            (solx_standard_json::InputLanguage::Solidity, Toolchain::SolxMlir) => {
                SOLIDITY_MLIR_MODES.clone()
            }
            (solx_standard_json::InputLanguage::Solidity, _) => SOLIDITY_MODES.clone(),
            (solx_standard_json::InputLanguage::Yul, Toolchain::SolxMlir) => YUL_MLIR_MODES.clone(),
            (solx_standard_json::InputLanguage::Yul, _) => YUL_MODES.clone(),
            (solx_standard_json::InputLanguage::LLVMIR, _) => Vec::new(),
        }
    }

    fn allows_multi_contract_files(&self) -> bool {
        true
    }
}
