//!
//! Slang Solidity frontend implementation.
//!

mod compilation_config;

use std::collections::BTreeMap;
use std::path::PathBuf;

use slang_solidity_v2::compilation::CompilationBuilder;
use slang_solidity_v2::compilation::CompilationUnit;
use slang_solidity_v2::compilation::FileId;
use slang_solidity_v2::diagnostics::DiagnosticExtensions;
use slang_solidity_v2::utils::EvmTarget;
use slang_solidity_v2::utils::LanguageVersion;

use solx_core::Frontend;
use solx_standard_json::CollectableError;
use solx_standard_json::output::error::source_location::SourceLocation;

use crate::scope::source_unit::SourceUnitScope;

use self::compilation_config::CompilationConfig;

/// The Slang frontend implementation.
#[derive(Debug)]
pub struct Slang {
    /// The Slang compiler latest supported version.
    pub version: solx_standard_json::Version,
}

impl Default for Slang {
    fn default() -> Self {
        let default: semver::Version = LanguageVersion::LATEST.into();

        Self {
            version: solx_standard_json::Version::new(default.to_string(), default),
        }
    }
}

impl Slang {
    /// The name of the Slang frontend, used in error messages and output metadata.
    pub const NAME: &'static str = "Slang";

    /// Builds a Slang compilation unit from the given source files, parsing every source and
    /// resolving imports.
    ///
    /// Every EVM built-in is admitted (`EvmTarget::LATEST`): Slang gates built-in availability on
    /// the target, whereas solx handles EVM-version targeting downstream.
    ///
    /// # Errors
    ///
    /// Returns an error if the compilation builder fails to initialize or if import resolution
    /// fails.
    fn compile(&self, sources: BTreeMap<FileId, String>) -> anyhow::Result<CompilationUnit> {
        let file_ids: Vec<FileId> = sources.keys().cloned().collect();
        let configuration = CompilationConfig::new(sources);
        let version: LanguageVersion =
            self.version.default.clone().try_into().map_err(|error| {
                anyhow::anyhow!(
                    "failed to convert Solidity version '{}' to a Slang language version: {error}",
                    self.version.default
                )
            })?;
        let mut builder = CompilationBuilder::create(version, EvmTarget::LATEST, configuration);

        for file_id in file_ids {
            builder.add_file(file_id);
        }

        Ok(builder.build())
    }
}

impl Frontend for Slang {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn standard_json(
        &self,
        input_json: &mut solx_standard_json::Input,
        _use_import_callback: bool,
        _base_path: Option<&str>,
        _include_paths: &[String],
        _allow_paths: Option<String>,
    ) -> anyhow::Result<solx_standard_json::Output> {
        let mut output = solx_standard_json::Output::new(&input_json.sources);

        if input_json.language != solx_standard_json::InputLanguage::Solidity {
            output
                .errors
                .push(solx_standard_json::OutputError::new_error(
                    "Slang frontend only supports Solidity sources.",
                ));
            return Ok(output);
        }

        if let Err(error) = input_json.resolve_sources() {
            output
                .errors
                .push(solx_standard_json::OutputError::new_error(error));
            return Ok(output);
        }

        let mut sources = BTreeMap::new();
        for (path, source) in input_json.sources.iter() {
            let Some(source_code) = source.content() else {
                output
                    .errors
                    .push(solx_standard_json::OutputError::new_error_with_data(
                        Some(path.as_str()),
                        None,
                        "Source content is unavailable.",
                        Some(SourceLocation::new(
                            path.to_owned(),
                            SourceLocation::UNKNOWN_OFFSET,
                            SourceLocation::UNKNOWN_OFFSET,
                        )),
                        Some(&input_json.sources),
                    ));
                continue;
            };
            sources.insert(path.as_str().into(), source_code.to_owned());
        }

        let unit = self.compile(sources)?;

        output
            .errors
            .extend(unit.diagnostics().iter().map(|diagnostic| {
                let file_id = diagnostic.file_id();
                let text_range = diagnostic.text_range();
                solx_standard_json::OutputError::new_error_with_data(
                    Some(file_id.as_str()),
                    None,
                    diagnostic.message(),
                    Some(SourceLocation::new(
                        file_id.to_string(),
                        text_range.start as isize,
                        text_range.end as isize,
                    )),
                    Some(&input_json.sources),
                )
            }));

        for file in unit.files() {
            let file_id = file.id();
            if let Some(output_source) = output.sources.get_mut(file_id.as_str()) {
                output_source.ast = Some(
                    serde_json::to_value(file.ast())
                        .map_err(|error| anyhow::anyhow!("AST serialization: {error}"))?,
                );
            }
        }

        if output.has_errors() {
            return Ok(output);
        }

        let evm_version = input_json.settings.evm_version.unwrap_or_default();
        for file in unit.files() {
            let file_id = file.id();
            let contracts =
                SourceUnitScope::source_unit(&file.ast(), evm_version, |contract_name| {
                    input_json.settings.output_selection.check_selection(
                        file_id.as_str(),
                        Some(contract_name),
                        solx_standard_json::InputSelector::MLIR,
                    )
                })?;
            output
                .contracts
                .entry(file_id.to_string())
                .or_default()
                .extend(contracts);
        }

        Ok(output)
    }

    fn validate_yul_paths(
        &self,
        paths: &[PathBuf],
        libraries: solx_utils::Libraries,
    ) -> anyhow::Result<solx_standard_json::Output> {
        let mut solc_input = solx_standard_json::Input::from_yul_paths(
            paths,
            libraries,
            solx_standard_json::InputOptimizer::default(),
            &solx_standard_json::InputSelection::default(),
            solx_standard_json::InputMetadata::default(),
            vec![],
        );

        self.validate_yul_standard_json(&mut solc_input)
    }

    fn validate_yul_standard_json(
        &self,
        solc_input: &mut solx_standard_json::Input,
    ) -> anyhow::Result<solx_standard_json::Output> {
        let mut output = solx_standard_json::Output::new(&solc_input.sources);
        output
            .errors
            .push(solx_standard_json::OutputError::new_error(
                "Yul validation is not supported by the Slang frontend.",
            ));
        Ok(output)
    }

    fn version(&self) -> &solx_standard_json::Version {
        &self.version
    }
}
