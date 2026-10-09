//!
//! Slang Solidity frontend implementation.
//!

mod import_resolver;

use std::cell::OnceCell;
use std::cmp::Reverse;
use std::collections::BTreeMap;

use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;
use slang_solidity_v2::ast::SourceUnitMember;
use slang_solidity_v2::compilation::CompilationUnit;
use slang_solidity_v2::compilation::Configuration;
use slang_solidity_v2::compilation::FileId;
use slang_solidity_v2::diagnostics::DiagnosticExtensions;
use slang_solidity_v2::diagnostics::DiagnosticSeverity;
use slang_solidity_v2::utils::EvmTarget;
use slang_solidity_v2::utils::LanguageVersion;

use solx_mlir::Context;
use solx_standard_json::CollectableError;
use solx_standard_json::OutputError;
use solx_standard_json::output::contract::Contract;
use solx_standard_json::output::error::source_location::SourceLocation;
use solx_utils::EVMVersion;
use solx_utils::Profiler;
use solx_utils::Remapping;
use solx_utils::RevertStrings;

use crate::abi::AbiDefinition;
use crate::debug_locations::resolver::Resolver;
use crate::debug_locations::sources::Sources;
use crate::scope::source_unit::SourceUnitScope;

use self::import_resolver::SourceImportResolver;

/// Parses and binds Solidity with Slang and lowers it to Sol-dialect MLIR.
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

    ///
    /// The Solidity `--standard-json` mirror.
    ///
    /// Metadata is always requested in order to calculate the metadata hash even if not requested in the `output_selection`.
    ///
    /// Sets `settings.solidity_version` to the version the sources are compiled as, which goes into
    /// the metadata.
    ///
    pub fn standard_json(
        &self,
        input_json: &mut solx_standard_json::Input,
    ) -> anyhow::Result<solx_standard_json::Output> {
        let mut profiler = Profiler::default();
        let mut output = solx_standard_json::Output::new(&input_json.sources);

        if input_json.language != solx_standard_json::InputLanguage::Solidity {
            output
                .errors
                .push(OutputError::new_error("Yul is not supported yet."));
            return Ok(output);
        }

        if input_json.settings.via_ir {
            output.errors.push(OutputError::new_warning(
                "viaIR is ignored: Slang has a single compilation pipeline.",
            ));
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

        let language_version = match Self::language_version(
            input_json.settings.solidity_version.as_ref(),
            &mut output.errors,
        ) {
            Ok(language_version) => language_version,
            Err(error) => {
                output.errors.push(OutputError::new_error(error));
                return Ok(output);
            }
        };
        input_json.settings.solidity_version = Some(language_version.into());
        let evm_version = Self::evm_version(
            input_json.settings.evm_version,
            language_version,
            &mut output.errors,
        );

        let run_analysis =
            profiler.start_pipeline_element(format!("{}_ParseAndBind", Self::NAME).as_str());
        let unit = Self::compile(
            language_version,
            evm_version,
            &sources,
            &input_json.settings.remappings,
        );
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
            if !input_json.settings.output_selection.check_selection(
                file_id.as_str(),
                None,
                solx_standard_json::InputSelector::AST,
            ) {
                continue;
            }
            if let Some(output_source) = output.sources.get_mut(file_id.as_str()) {
                let run_ast_serialization = profiler.start_pipeline_element(
                    format!("{}_SerializeAST:{file_id}", Self::NAME).as_str(),
                );
                output_source.ast = Some(
                    serde_json::value::to_raw_value(&file.ast())
                        .map_err(|error| anyhow::anyhow!("AST serialization: {error}"))?,
                );
                run_ast_serialization.borrow_mut().finish();
            }
        }

        if output.has_errors() {
            return Ok(output);
        }

        let revert_strings = input_json
            .settings
            .debug
            .as_ref()
            .and_then(|debug| debug.revert_strings)
            .unwrap_or(RevertStrings::Default);
        let sources = Sources::new(&sources);
        let benchmarks = input_json.settings.output_selection.check_selection(
            solx_standard_json::InputSelection::WILDCARD,
            Some(solx_standard_json::InputSelection::ANY_CONTRACT),
            solx_standard_json::InputSelector::Benchmarks,
        );
        let output_selection = &input_json.settings.output_selection;
        let mut members = unit
            .files()
            .flat_map(|file| {
                let file_id = file.id().to_string();
                file.ast()
                    .members()
                    .iter()
                    .map(|member| (file_id.clone(), member))
                    .collect::<Vec<_>>()
            })
            .enumerate()
            .collect::<Vec<_>>();
        members.sort_by_cached_key(|(_, (_, member))| {
            Reverse(
                AbiDefinition::from_member(member)
                    .and_then(AbiDefinition::into_object)
                    .map_or(0, |object| object.estimated_lowering_cost()),
            )
        });
        let lower = |melior: &_, (index, (file_id, member)): (_, (String, SourceUnitMember))| {
            let definition = AbiDefinition::from_member(&member)?;
            let name = definition.name().name().to_owned();
            let Some(abi) = definition.abi() else {
                return Some((index, (file_id, name, Err(member))));
            };
            let storage_layout = abi.storage_layout();
            let abi_value = abi.into_value();
            let method_identifiers = definition.method_identifiers();
            let Some(object) = definition.into_object() else {
                let contract = Contract::new_abi(abi_value, method_identifiers);
                return Some((index, (file_id, name, Ok(Ok((contract, Vec::new()))))));
            };
            let selected = |selector| {
                output_selection.check_selection(file_id.as_str(), Some(name.as_str()), selector)
            };
            let mut object_profiler = Profiler::default();
            let contract = SourceUnitScope::object(
                melior,
                &object,
                &storage_layout,
                evm_version,
                revert_strings,
                selected(solx_standard_json::InputSelector::BytecodeDebugInfo),
                selected(solx_standard_json::InputSelector::RuntimeBytecodeDebugInfo),
                selected(solx_standard_json::InputSelector::MLIR),
                &sources,
                benchmarks,
                &mut object_profiler,
            )
            .map(|mlir| {
                // `convert-sol-to-yul` builds the entry-point dispatcher from the same selectors.
                let contract = Contract::new_mlir(mlir, abi_value, method_identifiers);
                (contract, object_profiler.to_vec())
            });
            Some((index, (file_id, name, Ok(contract))))
        };
        let outputs: BTreeMap<_, _> = if Context::is_multithreaded() {
            // A melior context serves one thread, so each worker lowers off a context of its own.
            members
                .into_iter()
                .par_bridge()
                .map_init(OnceCell::new, |melior, member| lower(melior, member))
                .flatten()
                .collect()
        } else {
            let melior = OnceCell::new();
            members
                .into_iter()
                .filter_map(|member| lower(&melior, member))
                .collect()
        };

        let mut pipeline_benchmarks = profiler.to_vec();
        for (file_id, name, contract) in outputs.into_values() {
            match contract {
                Ok(contract) => {
                    let (contract, object_benchmarks) = contract?;
                    output
                        .contracts
                        .entry(file_id)
                        .or_default()
                        .insert(name, contract);
                    pipeline_benchmarks.extend(object_benchmarks);
                }
                Err(member) => {
                    let (file_id, text_range) = Resolver::source_range(&member);
                    output.errors.push(OutputError::new_error_with_data(
                        Some(file_id.as_str()),
                        None,
                        format!("Slang cannot compute the ABI of `{name}`."),
                        Some(SourceLocation::new(
                            file_id.to_string(),
                            text_range.start as isize,
                            text_range.end as isize,
                        )),
                        Some(&input_json.sources),
                    ));
                }
            }
        }

        if benchmarks {
            output.benchmarks = pipeline_benchmarks;
        }

        Ok(output)
    }

    /// Picks the language version to compile as: the requested one, or the latest Slang supports.
    ///
    /// A prerelease or build suffix is dropped with a warning pushed to `messages`.
    ///
    /// # Errors
    ///
    /// Returns an error if Slang does not support the requested version.
    fn language_version(
        requested: Option<&semver::Version>,
        messages: &mut Vec<OutputError>,
    ) -> Result<LanguageVersion, String> {
        let Some(requested) = requested else {
            return Ok(LanguageVersion::LATEST);
        };
        let mut version = requested.clone();
        if !version.pre.is_empty() || !version.build.is_empty() {
            version.pre = semver::Prerelease::EMPTY;
            version.build = semver::BuildMetadata::EMPTY;
            messages.push(OutputError::new_warning(format!(
                "Solidity version {requested} is compiled as {version}: the suffix names a solc build, which does not change the language version."
            )));
        }
        version.clone().try_into().map_err(|_| {
            format!(
                "Solidity version {version} is not supported. Supported versions are {} to {}.",
                semver::Version::from(LanguageVersion::EARLIEST),
                semver::Version::from(LanguageVersion::LATEST),
            )
        })
    }

    /// Picks the EVM version to analyze at: the requested one, or solc's default for the language
    /// version.
    ///
    /// A default older than Cancun falls back to Cancun with a warning pushed to `messages`.
    fn evm_version(
        requested: Option<EVMVersion>,
        language_version: LanguageVersion,
        messages: &mut Vec<OutputError>,
    ) -> EVMVersion {
        if let Some(requested) = requested {
            return requested;
        }
        let default = language_version.default_evm_target();
        if default < EvmTarget::Cancun {
            // TODO: target the EVM versions older than Cancun that solc defaults to.
            messages.push(OutputError::new_warning(format!(
                "Solidity version {} defaults to EVM version {}, which the compiler does not support yet. Compiling for {}, the oldest EVM version the compiler supports.",
                semver::Version::from(language_version),
                default.to_string().to_lowercase(),
                EVMVersion::Cancun,
            )));
            return EVMVersion::Cancun;
        }
        EVMVersion::try_from(default)
            .expect("the compiler supports every EVM version from Cancun up")
    }

    /// Builds a Slang compilation unit from the given source files, parsing every source and
    /// resolving imports with the given remappings, following solc's semantics.
    fn compile(
        language_version: LanguageVersion,
        evm_version: EVMVersion,
        sources: &BTreeMap<FileId, &str>,
        remappings: &[Remapping],
    ) -> CompilationUnit {
        CompilationUnit::create(Configuration {
            language_version,
            evm_target: evm_version.into(),
            sources: sources
                .iter()
                .map(|(file_id, content)| (file_id.clone(), *content)),
            resolver: SourceImportResolver { remappings },
        })
    }
}
