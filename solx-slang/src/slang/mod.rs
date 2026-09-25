//!
//! Slang Solidity frontend implementation.
//!

mod import_resolver;

use std::collections::BTreeMap;

use slang_solidity_v2::compilation::CompilationUnit;
use slang_solidity_v2::compilation::Configuration;
use slang_solidity_v2::compilation::FileId;
use slang_solidity_v2::diagnostics::DiagnosticExtensions;
use slang_solidity_v2::diagnostics::DiagnosticSeverity;
use slang_solidity_v2::utils::EvmTarget;
use slang_solidity_v2::utils::LanguageVersion;

use solx_standard_json::CollectableError;
use solx_standard_json::OutputError;
use solx_standard_json::output::error::source_location::SourceLocation;
use solx_utils::Profiler;
use solx_utils::Remapping;
use solx_utils::RevertStrings;

use crate::scope::source_unit::SourceUnitScope;

use self::import_resolver::SourceImportResolver;

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
    /// The frontend name the compiler reports and prefixes its pipeline benchmarks with.
    pub const NAME: &'static str = "Slang";

    /// Builds a Slang compilation unit from the given source files, parsing every source and
    /// resolving imports with the given remappings, following solc's semantics.
    ///
    /// Every EVM built-in is admitted (`EvmTarget::LATEST`): Slang gates built-in availability on
    /// the target, whereas solx handles EVM-version targeting downstream.
    ///
    /// # Errors
    ///
    /// Returns an error if Slang does not support the Solidity version.
    fn compile(
        &self,
        sources: &BTreeMap<FileId, &str>,
        remappings: &[Remapping],
    ) -> anyhow::Result<CompilationUnit> {
        let language_version: LanguageVersion =
            self.version.default.clone().try_into().map_err(|error| {
                anyhow::anyhow!(
                    "failed to convert Solidity version '{}' to a Slang language version: {error}",
                    self.version.default
                )
            })?;

        Ok(CompilationUnit::create(Configuration {
            language_version,
            evm_target: EvmTarget::LATEST,
            sources: sources
                .iter()
                .map(|(file_id, content)| (file_id.clone(), *content)),
            resolver: SourceImportResolver { remappings },
        }))
    }
}

impl Slang {
    ///
    /// The Solidity `--standard-json` mirror.
    ///
    /// Metadata is always requested in order to calculate the metadata hash even if not requested in the `output_selection`.
    ///
    pub fn standard_json(
        &self,
        input_json: &mut solx_standard_json::Input,
    ) -> anyhow::Result<solx_standard_json::Output> {
        let mut profiler = Profiler::default();
        let mut output = solx_standard_json::Output::new(&input_json.sources);

        if input_json.language != solx_standard_json::InputLanguage::Solidity {
            output.errors.push(OutputError::new_error(
                "Slang frontend only supports Solidity sources.",
            ));
            return Ok(output);
        }

        if input_json.settings.via_ir {
            output.errors.push(OutputError::new_error(
                "Slang frontend does not support viaIR.",
            ));
            return Ok(output);
        }

        if let Err(error) = input_json.resolve_sources() {
            output.errors.push(OutputError::new_error(error));
            return Ok(output);
        }

        let mut sources = BTreeMap::new();
        for (path, source) in input_json.sources.iter() {
            let Some(source_code) = source.content() else {
                output.errors.push(OutputError::new_error_with_data(
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
            sources.insert(path.as_str().into(), source_code);
        }

        let run_analysis =
            profiler.start_pipeline_element(format!("{}_ParseAndBind", Self::NAME).as_str());
        let unit = self.compile(&sources, &input_json.settings.remappings)?;
        run_analysis.borrow_mut().finish();

        output
            .errors
            .extend(unit.diagnostics().iter().map(|diagnostic| {
                let file_id = diagnostic.file_id();
                let text_range = diagnostic.text_range();
                let new_with_data = match diagnostic.severity() {
                    DiagnosticSeverity::Error => OutputError::new_error_with_data,
                    DiagnosticSeverity::Warning => OutputError::new_warning_with_data,
                };
                new_with_data(
                    Some(file_id.as_str()),
                    Some(diagnostic.code()),
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
                let run_ast_serialization = profiler.start_pipeline_element(
                    format!("{}_SerializeAST:{file_id}", Self::NAME).as_str(),
                );
                output_source.ast = Some(
                    serde_json::to_value(file.ast())
                        .map_err(|error| anyhow::anyhow!("AST serialization: {error}"))?,
                );
                run_ast_serialization.borrow_mut().finish();
            }
        }

        if output.has_errors() {
            return Ok(output);
        }

        let evm_version = input_json.settings.evm_version.unwrap_or_default();
        let revert_strings = input_json
            .settings
            .debug
            .as_ref()
            .and_then(|debug| debug.revert_strings)
            .unwrap_or(RevertStrings::Default);
        for file in unit.files() {
            let file_id = file.id();
            let contracts = SourceUnitScope::source_unit(
                &file.ast(),
                evm_version,
                revert_strings,
                |contract_name| {
                    input_json.settings.output_selection.check_selection(
                        file_id.as_str(),
                        Some(contract_name),
                        solx_standard_json::InputSelector::MLIR,
                    )
                },
                &mut profiler,
            )?;
            output
                .contracts
                .entry(file_id.to_string())
                .or_default()
                .extend(contracts);
        }

        if input_json.settings.output_selection.check_selection(
            solx_standard_json::InputSelection::WILDCARD,
            Some(solx_standard_json::InputSelection::ANY_CONTRACT),
            solx_standard_json::InputSelector::Benchmarks,
        ) {
            output.benchmarks = profiler.to_vec();
        }

        Ok(output)
    }
}
