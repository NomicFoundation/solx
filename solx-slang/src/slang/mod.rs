//!
//! Slang Solidity frontend implementation.
//!

/// Compilation builder configuration for the Slang frontend.
pub mod compilation_config;

use std::collections::BTreeMap;
use std::path::PathBuf;

use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::SourceUnitMember;
use slang_solidity_v2::compilation::CompilationBuilder;
use slang_solidity_v2::compilation::CompilationUnit;
use slang_solidity_v2::diagnostics::DiagnosticExtensions;
use slang_solidity_v2::utils::LanguageVersion;
use slang_solidity_v2_common::evm_targets::EvmTarget;

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
        // The Slang frontend gates EVM built-in availability on the target; solx
        // handles EVM-version targeting downstream, so admit every built-in here.
        let mut builder = CompilationBuilder::create(version, EvmTarget::LATEST, configuration);

        for path in paths.iter() {
            builder.add_file(path.clone());
        }

        Ok(builder.build())
    }

    /// Gathers every file-level (free) function across the compilation unit.
    ///
    /// Free functions are not part of any contract's linearised function set,
    /// so they are collected once per unit and handed to each contract emitter,
    /// which pre-registers and emits the ones that contract reaches.
    fn gather_free_functions(unit: &CompilationUnit) -> Vec<FunctionDefinition> {
        unit.file_ids()
            .iter()
            .filter_map(|file_identifier| unit.file(file_identifier))
            .flat_map(|file| {
                file.ast()
                    .members()
                    .iter()
                    .filter_map(|member| {
                        if let SourceUnitMember::FunctionDefinition(function) = member {
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
        let runtime_code_identifier = format!("{name}{}", solx_codegen_evm::DEPLOYED_OBJECT_SUFFIX);
        let capture_sol_dialect = input_json.settings.output_selection.check_selection(
            file_identifier,
            Some(name.as_str()),
            solx_standard_json::InputSelector::MLIR,
        );
        let mlir_stages = context.finalize_module(&runtime_code_identifier, capture_sol_dialect)?;
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

        let unit = self.compile(sources)?;

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
        let operator_bindings = crate::ast::operator_binding::OperatorBindings::gather(&unit);

        for file_identifier in &file_identifiers {
            let Some(file) = unit.file(file_identifier) else {
                continue;
            };
            let source_unit = file.ast();

            // Emit every contract in the file as its own deployable object. A
            // file commonly defines several contracts (bases plus the deployed,
            // most-derived one); the test harness deploys any of them by name, so
            // each must be present — not just the first. Each contract gets its
            // own MLIR context/module, with its C3-linearised bases pulled in by
            // the contract emitter. A contract the recut cannot yet lower is
            // simply absent from the artifacts — exactly as when only the first
            // contract was emitted — so a sibling's gap (an `Err` or a `panic!`
            // in emission) never aborts the file or the other contracts; the
            // missing object instead surfaces as that contract's failing tests.
            for contract in source_unit.contracts().iter() {
                // An `abstract contract` cannot be instantiated and is never
                // deployed by the harness; skip it (its bodyless functions have
                // no code to lower anyway).
                if contract.abstract_keyword().is_some() {
                    continue;
                }
                let emitted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                    || -> anyhow::Result<()> {
                        let melior_context = solx_mlir::Context::create_mlir_context();
                        let evm_version = input_json.settings.evm_version.unwrap_or_default();
                        let mut context = solx_mlir::Context::new(&melior_context, evm_version);
                        let mut emitter = AstEmitter::new(&mut context);
                        let (contract_name, method_identifiers) =
                            emitter.emit_contract(contract, &free_functions, &operator_bindings);
                        Self::record_object(
                            context,
                            contract_name,
                            method_identifiers,
                            input_json,
                            file_identifier,
                            &mut output,
                        )
                    },
                ));
                let _ = emitted;
            }

            // Emit every library as its own deployable object, including
            // internal-only ones. solc emits a (call-protected stub) object for
            // every library regardless of visibility, and the test harness's
            // `// library:` directive deploys and links it by name — so the
            // object must exist in the build artifacts even when all functions
            // are `internal`. Each library gets its own MLIR context / module,
            // emitted like a contract: an unsupported construct propagates the
            // same way any contract's would.
            for member in source_unit.members().iter() {
                let SourceUnitMember::LibraryDefinition(library) = member else {
                    continue;
                };

                let melior_context = solx_mlir::Context::create_mlir_context();
                let evm_version = input_json.settings.evm_version.unwrap_or_default();
                let mut context = solx_mlir::Context::new(&melior_context, evm_version);
                let (library_name, method_identifiers) =
                    crate::ast::contract::ContractEmitter::new(&mut context).emit_library(&library);
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
