//!
//! Solidity frontend interface trait.
//!

use std::collections::BTreeSet;
use std::path::PathBuf;

///
/// What a frontend can be asked for.
///
/// Requests outside of it are rejected before compilation instead of being dropped, so that a
/// missing artifact is never the first sign that a flag or setting did nothing.
///
#[derive(Debug)]
pub struct Capabilities {
    /// Output selectors the frontend does not produce yet, which are errors: the artifact is
    /// meant to exist and whatever reads it would find the key missing.
    pub unsupported_selectors: BTreeSet<solx_standard_json::InputSelector>,
    /// Output selectors that are by-products of solc's codegen pipelines, which are warnings: a
    /// frontend without those pipelines has nothing to emit, and never will.
    pub pipeline_selectors: BTreeSet<solx_standard_json::InputSelector>,
    /// Whether the frontend reads imports from disk, that is `--base-path` and its family.
    pub disk_imports: bool,
    /// Whether the frontend produces metadata, which `--metadata-literal` and ipfs hashing need.
    pub metadata: bool,
    /// Whether the frontend has solc's codegen pipelines, that is `--via-ir` and legacy.
    pub solc_pipelines: bool,
    /// Whether the frontend honors solc's optimizer settings.
    pub solc_optimizer: bool,
}

impl Capabilities {
    /// Command line options whose support is not expressed by an output selector.
    const DISK_IMPORT_OPTIONS: &'static [&'static str] =
        &["--base-path", "--include-path", "--allow-paths"];

    ///
    /// Returns a diagnostic for every command line option the frontend cannot honor.
    ///
    /// Options that ask for an output the frontend does not produce are errors, since the artifact
    /// would be missing. Options that only pick how an output is produced are warnings: the output
    /// is there, it is just not built the way the option asks for.
    ///
    pub fn arguments_diagnostics(
        &self,
        arguments: &crate::Arguments,
        name: &str,
    ) -> Vec<solx_standard_json::OutputError> {
        let mut diagnostics = Vec::new();

        if arguments.via_ir && !self.solc_pipelines {
            diagnostics.push(solx_standard_json::OutputError::new_warning(
                Self::describe_pipeline(name),
            ));
        }
        if !self.disk_imports {
            let requested = [
                arguments.base_path.is_some(),
                !arguments.include_path.is_empty(),
                arguments.allow_paths.is_some(),
            ];
            for option in Self::DISK_IMPORT_OPTIONS
                .iter()
                .zip(requested)
                .filter_map(|(option, is_requested)| is_requested.then_some(option))
            {
                diagnostics.push(solx_standard_json::OutputError::new_error(format!(
                    "Command line option {option} is not supported in {name}. Imports are not read from disk; pass every source explicitly."
                )));
            }
        }
        if !self.metadata {
            if arguments.metadata_literal {
                diagnostics.push(solx_standard_json::OutputError::new_error(format!(
                    "Command line option --metadata-literal is not supported in {name}."
                )));
            }
            if arguments.metadata_hash == Some(solx_utils::MetadataHashType::IPFS) {
                diagnostics.push(solx_standard_json::OutputError::new_error(format!(
                    "Command line option --metadata-hash ipfs is not supported in {name}."
                )));
            }
        }

        for (option, is_requested, selector) in [
            (
                "--abi",
                arguments.output_abi,
                solx_standard_json::InputSelector::ABI,
            ),
            (
                "--hashes",
                arguments.output_hashes,
                solx_standard_json::InputSelector::MethodIdentifiers,
            ),
            (
                "--metadata",
                arguments.output_metadata,
                solx_standard_json::InputSelector::Metadata,
            ),
            (
                "--userdoc",
                arguments.output_userdoc,
                solx_standard_json::InputSelector::UserDocumentation,
            ),
            (
                "--devdoc",
                arguments.output_devdoc,
                solx_standard_json::InputSelector::DeveloperDocumentation,
            ),
            (
                "--storage-layout",
                arguments.output_storage_layout,
                solx_standard_json::InputSelector::StorageLayout,
            ),
            (
                "--transient-storage-layout",
                arguments.output_transient_storage_layout,
                solx_standard_json::InputSelector::TransientStorageLayout,
            ),
            (
                "--asm-solc-json",
                arguments.output_asm_solc_json,
                solx_standard_json::InputSelector::EVMLegacyAssembly,
            ),
            (
                "--ir",
                arguments.output_ir,
                solx_standard_json::InputSelector::Yul,
            ),
            (
                "--evmla",
                arguments.output_evmla,
                solx_standard_json::InputSelector::BytecodeEVMLA,
            ),
            (
                "--ethir",
                arguments.output_ethir,
                solx_standard_json::InputSelector::BytecodeEthIR,
            ),
            (
                "--debug-info",
                arguments.output_debug_info,
                solx_standard_json::InputSelector::BytecodeDebugInfo,
            ),
            (
                "--debug-info-runtime",
                arguments.output_debug_info_runtime,
                solx_standard_json::InputSelector::RuntimeBytecodeDebugInfo,
            ),
        ] {
            if !is_requested {
                continue;
            }
            if self.unsupported_selectors.contains(&selector) {
                diagnostics.push(solx_standard_json::OutputError::new_error(format!(
                    "Command line option {option} is not supported in {name}."
                )));
            } else if self.pipeline_selectors.contains(&selector) {
                diagnostics.push(solx_standard_json::OutputError::new_warning(
                    Self::describe_pipeline_output(option, name),
                ));
            }
        }

        self.push_empty_output_diagnostic(
            &arguments.output_selection().selectors(),
            name,
            &mut diagnostics,
        );

        diagnostics
    }

    ///
    /// Returns a diagnostic for every standard JSON setting and output selection the frontend
    /// cannot honor, with the same error and warning split as the command line.
    ///
    /// Only selections spelled out in the input are checked: an umbrella selection such as `evm`
    /// asks for whatever the compiler has, not for each of its members.
    ///
    pub fn standard_json_diagnostics(
        &self,
        input: &solx_standard_json::Input,
        name: &str,
    ) -> Vec<solx_standard_json::OutputError> {
        let mut diagnostics = Vec::new();

        if input.settings.via_ir && !self.solc_pipelines {
            diagnostics.push(solx_standard_json::OutputError::new_warning(
                Self::describe_pipeline(name),
            ));
        }
        if !self.solc_optimizer {
            let optimizer = &input.settings.optimizer;
            for option in [
                optimizer.enabled.map(|_| "enabled"),
                optimizer.runs.map(|_| "runs"),
                optimizer.details.as_ref().map(|_| "details"),
            ]
            .into_iter()
            .flatten()
            {
                diagnostics.push(solx_standard_json::OutputError::new_warning(format!(
                    r#"Standard JSON option "optimizer.{option}" is not honored in {name}, which optimizes through LLVM: use "optimizer.mode" instead."#
                )));
            }
        }
        if input.settings.metadata.use_literal_content && !self.metadata {
            diagnostics.push(solx_standard_json::OutputError::new_error(format!(
                r#"Standard JSON option "useLiteralContent" is not supported in {name}."#
            )));
        }
        let selectors = input.settings.output_selection.selectors();
        for selector in selectors.iter() {
            if self.unsupported_selectors.contains(selector) {
                diagnostics.push(solx_standard_json::OutputError::new_error(format!(
                    r#"Standard JSON output selection "{selector}" is not supported in {name}."#
                )));
            } else if self.pipeline_selectors.contains(selector) {
                diagnostics.push(solx_standard_json::OutputError::new_warning(
                    Self::describe_pipeline_output(
                        format!(r#"Standard JSON output selection "{selector}""#).as_str(),
                        name,
                    ),
                ));
            }
        }

        self.push_empty_output_diagnostic(&selectors, name, &mut diagnostics);

        diagnostics
    }

    ///
    /// Errors when every requested output is unavailable, which per-request warnings alone would
    /// leave as a successful run that produces nothing.
    ///
    fn push_empty_output_diagnostic(
        &self,
        selectors: &BTreeSet<solx_standard_json::InputSelector>,
        name: &str,
        diagnostics: &mut Vec<solx_standard_json::OutputError>,
    ) {
        if selectors.is_empty()
            || diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == "error")
        {
            return;
        }
        if selectors.iter().all(|selector| {
            self.unsupported_selectors.contains(selector)
                || self.pipeline_selectors.contains(selector)
        }) {
            diagnostics.push(solx_standard_json::OutputError::new_error(format!(
                "Nothing would be produced: every requested output is unavailable in {name}."
            )));
        }
    }

    ///
    /// The warning for an output that only solc's codegen pipelines produce.
    ///
    fn describe_pipeline_output(request: &str, name: &str) -> String {
        format!(
            "{request} is not honored in {name}: it names an artifact of solc's codegen pipelines, which {name} does not have."
        )
    }

    ///
    /// The pipeline warning, which is the same on both input paths.
    ///
    fn describe_pipeline(name: &str) -> String {
        format!(
            "Via IR codegen is not honored in {name}, which has a single pipeline that is neither solc's legacy codegen nor its IR codegen. Bytecode differs from both."
        )
    }
}

///
/// Solidity frontend interface trait.
///
pub trait Frontend {
    ///
    /// Returns the frontend compiler name.
    ///
    fn name(&self) -> &str;

    ///
    /// Returns what the frontend can be asked for.
    ///
    fn capabilities(&self) -> &Capabilities;

    ///
    /// The Solidity `--standard-json` mirror.
    ///
    /// Metadata is always requested in order to calculate the metadata hash even if not requested in the `output_selection`.
    /// EVM assembly or Yul is always selected in order to compile the Solidity code.
    ///
    fn standard_json(
        &self,
        input_json: &mut solx_standard_json::Input,
        use_import_callback: bool,
        base_path: Option<&str>,
        include_paths: &[String],
        allow_paths: Option<String>,
    ) -> anyhow::Result<solx_standard_json::Output>;

    ///
    /// Validates the Yul project as paths and libraries.
    ///
    fn validate_yul_paths(
        &self,
        paths: &[PathBuf],
        libraries: solx_utils::Libraries,
    ) -> anyhow::Result<solx_standard_json::Output>;

    ///
    /// Validates the Yul project as standard JSON input.
    ///
    fn validate_yul_standard_json(
        &self,
        solc_input: &mut solx_standard_json::Input,
    ) -> anyhow::Result<solx_standard_json::Output>;

    ///
    /// Returns the frontend compiler version.
    ///
    fn version(&self) -> &solx_standard_json::Version;
}
