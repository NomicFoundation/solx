//!
//! Slang Solidity frontend implementation.
//!

/// Compilation builder configuration for the Slang frontend.
pub mod compilation_config;

use std::collections::BTreeMap;
use std::path::PathBuf;

use slang_solidity_v2::compilation::CompilationBuilder;
use slang_solidity_v2::compilation::CompilationUnit;
use slang_solidity_v2::diagnostics::DiagnosticExtensions;
use slang_solidity_v2::utils::LanguageVersion;

use solx_core::Frontend;
use solx_standard_json::CollectableError;
use solx_standard_json::output::error::source_location::SourceLocation;

use crate::ast::AstEmitter;

use self::compilation_config::CompilationConfig;

/// The Slang frontend implementation.
#[derive(Debug)]
pub struct Slang {
    /// The Slang compiler latest supported version.
    version: solx_standard_json::Version,
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

    /// Builds a Slang compilation unit from the given source files.
    ///
    /// Uses the `CompilationBuilder` to parse all sources and resolve imports.
    ///
    /// # Errors
    ///
    /// Returns an error if the compilation builder fails to initialize or
    /// if import resolution fails.
    pub fn compile(&self, sources: BTreeMap<String, String>) -> anyhow::Result<CompilationUnit> {
        let paths: Vec<String> = sources.keys().cloned().collect();
        let configuration = CompilationConfig::new(sources);
        let version: LanguageVersion =
            self.version.default.clone().try_into().map_err(|error| {
                anyhow::anyhow!(
                    "failed to convert Solidity version '{}' to a Slang language version: {error}",
                    self.version.default
                )
            })?;
        let mut builder = CompilationBuilder::create(version, configuration);

        for path in paths.iter() {
            builder.add_file(path.clone());
        }

        Ok(builder.build())
    }

    /// Records the compilation unit's diagnostics as standard-JSON errors,
    /// mapping each diagnostic's file id and text range to a source location.
    fn record_diagnostics(
        unit: &CompilationUnit,
        input_json: &solx_standard_json::Input,
        output: &mut solx_standard_json::Output,
    ) {
        output
            .errors
            .extend(unit.diagnostics().iter().map(|diagnostic| {
                let file_identifier = diagnostic.file_id();
                let text_range = diagnostic.text_range();
                let source_location =
                    solx_standard_json::output::error::source_location::SourceLocation::new(
                        file_identifier,
                        text_range.start as isize,
                        text_range.end as isize,
                    );
                solx_standard_json::OutputError::new_error_with_data(
                    Some(file_identifier),
                    None,
                    diagnostic.message(),
                    Some(source_location),
                    Some(&input_json.sources),
                )
            }));
    }

    /// Collects the resolved source contents into a path→code map, recording a
    /// standard-JSON error for any source whose content is unavailable.
    fn collect_sources(
        input_json: &solx_standard_json::Input,
        output: &mut solx_standard_json::Output,
    ) -> BTreeMap<String, String> {
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
            sources.insert(path.clone(), source_code.to_owned());
        }
        sources
    }

    /// Gathers every file-level free function in the unit. Free functions are
    /// callable across imports, so each contract emitter is handed the full set
    /// and collects only the subset it actually reaches (resolved by node id).
    fn gather_free_functions(
        unit: &CompilationUnit,
    ) -> Vec<slang_solidity_v2::ast::FunctionDefinition> {
        unit.file_ids()
            .iter()
            .filter_map(|file_identifier| unit.file(file_identifier))
            .flat_map(|file| {
                file.ast()
                    .members()
                    .iter()
                    .filter_map(|member| {
                        if let slang_solidity_v2::ast::SourceUnitMember::FunctionDefinition(
                            function,
                        ) = member
                        {
                            Some(function)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// Finalises a freshly-emitted object's module and records its MLIR stages
    /// and method identifiers under `(file_identifier, name)` in the output.
    /// Shared by the contract and deployable-library emission paths.
    fn record_object(
        context: solx_mlir::Context<'_>,
        name: String,
        method_identifiers: BTreeMap<String, String>,
        input_json: &solx_standard_json::Input,
        file_identifier: &str,
        output: &mut solx_standard_json::Output,
    ) -> anyhow::Result<()> {
        let runtime_code_identifier =
            format!("{name}{}", solx_codegen_evm::DEPLOYED_OBJECT_SUFFIX);
        let capture_sol_dialect = input_json.settings.output_selection.check_selection(
            file_identifier,
            Some(name.as_str()),
            solx_standard_json::InputSelector::MLIR,
        );
        let mlir_stages =
            context.finalize_module(&runtime_code_identifier, capture_sol_dialect)?;
        output
            .contracts
            .entry(file_identifier.to_string())
            .or_default()
            .insert(
                name,
                solx_standard_json::output::contract::Contract {
                    mlir: Some(mlir_stages),
                    evm: Some(solx_standard_json::output::contract::evm::EVM {
                        method_identifiers: Some(method_identifiers),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            );
        Ok(())
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

        let sources = Self::collect_sources(input_json, &mut output);
        let unit = self.compile(sources)?;

        Self::record_diagnostics(&unit, input_json, &mut output);

        for file_identifier in &unit.file_ids() {
            if let Some(output_source) = output.sources.get_mut(file_identifier) {
                output_source.ast = Some(
                    serde_json::to_value(unit.file(file_identifier).map(|file| file.ast()))
                        .map_err(|error| anyhow::anyhow!("AST serialization: {error}"))?,
                );
            }
        }

        if output.has_errors() {
            return Ok(output);
        }

        let file_identifiers = unit.file_ids();
        let free_functions = Self::gather_free_functions(&unit);

        for file_identifier in &file_identifiers {
            let Some(file) = unit.file(file_identifier) else {
                continue;
            };
            let source_unit = file.ast();

            for contract in source_unit.contracts() {
                if contract.abstract_keyword().is_some() {
                    continue;
                }

                let melior_context = solx_mlir::Context::create_mlir_context();

                let evm_version = input_json.settings.evm_version.unwrap_or_default();
                let mut context = solx_mlir::Context::new(&melior_context, evm_version);
                let mut emitter = AstEmitter::new(&mut context);
                let (contract_name, method_identifiers) =
                    emitter.emit(&contract, &free_functions)?;

                Self::record_object(
                    context,
                    contract_name,
                    method_identifiers,
                    input_json,
                    file_identifier,
                    &mut output,
                )?;
            }

            // Libraries with `external`/`public` functions are deployed and
            // reached by `delegatecall` (`L.f(...)`), so emit each as its own
            // object — mirroring the contract emission above.
            for member in source_unit.members().iter() {
                let slang_solidity_v2::ast::SourceUnitMember::LibraryDefinition(library) = member
                else {
                    continue;
                };

                // Only libraries with `external`/`public` functions are deployed
                // and `delegatecall`ed. Internal-only libraries are fully inlined
                // into their callers (like solc, which emits no object for them);
                // emitting one would make the tester try to deploy/link it.
                let has_deployable_function = library.members().iter().any(|member| {
                    matches!(&member,
                        slang_solidity_v2::ast::ContractMember::FunctionDefinition(function)
                            if matches!(
                                function.visibility(),
                                slang_solidity_v2::ast::FunctionVisibility::External
                                    | slang_solidity_v2::ast::FunctionVisibility::Public
                            ))
                });
                if !has_deployable_function {
                    continue;
                }

                let melior_context = solx_mlir::Context::create_mlir_context();
                let evm_version = input_json.settings.evm_version.unwrap_or_default();
                let mut context = solx_mlir::Context::new(&melior_context, evm_version);

                // A deployable library is emitted like a contract: its own
                // module, errors propagated with `?`. An unsupported construct
                // is an `unimplemented!` panic (not an error) and propagates the
                // same way any contract's would — no special recovery.
                let (library_name, method_identifiers) =
                    crate::ast::contract::ContractEmitter::new(&mut context).emit_library(&library)?;
                Self::record_object(
                    context,
                    library_name,
                    method_identifiers,
                    input_json,
                    file_identifier,
                    &mut output,
                )?;
            }
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
