//!
//! Slang Solidity frontend implementation.
//!

/// Compilation builder configuration for the Slang frontend.
pub mod compilation_config;

use std::collections::BTreeMap;
use std::path::PathBuf;

use slang_solidity::compilation::CompilationBuilder;
use slang_solidity::compilation::CompilationUnit;
use slang_solidity::utils::LanguageFacts;

use solx_core::Frontend;
use solx_standard_json::CollectableError;

use self::compilation_config::CompilationConfig;

/// The Slang frontend implementation.
#[derive(Debug)]
pub struct SlangFrontend {
    /// The Slang compiler version.
    version: solx_standard_json::Version,
}

impl Default for SlangFrontend {
    fn default() -> Self {
        let default = LanguageFacts::LATEST_VERSION;

        Self {
            version: solx_standard_json::Version::new(default.to_string(), default),
        }
    }
}

impl SlangFrontend {
    /// Builds a Slang compilation unit from the given source files.
    ///
    /// Uses the `CompilationBuilder` to parse all sources and resolve imports.
    ///
    /// # Errors
    ///
    /// Returns an error if the compilation builder fails to initialize or
    /// if import resolution fails.
    pub fn compile(&self, sources: BTreeMap<String, String>) -> anyhow::Result<CompilationUnit> {
        let keys: Vec<String> = sources.keys().cloned().collect();
        let configuration = CompilationConfig::new(sources);
        let mut builder = CompilationBuilder::create(self.version.default.clone(), configuration)
            .map_err(|error| anyhow::anyhow!("slang compilation builder: {error}"))?;

        for path in &keys {
            builder.add_file(path)?;
        }

        Ok(builder.build())
    }
}

impl Frontend for SlangFrontend {
    fn name(&self) -> &str {
        "Slang"
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
        for (path, source) in &input_json.sources {
            let Some(source_code) = source.content() else {
                output
                    .errors
                    .push(solx_standard_json::OutputError::new_error_with_data(
                        Some(path.as_str()),
                        None,
                        "Source content is unavailable.",
                        Some(
                            solx_standard_json::output::error::source_location::SourceLocation::new(
                                path.to_owned(),
                                None,
                                None,
                            ),
                        ),
                        Some(&input_json.sources),
                    ));
                continue;
            };
            sources.insert(path.clone(), source_code.to_owned());
        }

        let unit = self.compile(sources)?;

        for file in unit.files() {
            let file_identifier = file.id();
            output.errors.extend(file.errors().iter().map(|error| {
                let text_range = error.text_range();
                let source_location =
                    solx_standard_json::output::error::source_location::SourceLocation::new(
                        file_identifier.to_owned(),
                        Some(text_range.start.utf8 as isize),
                        Some(text_range.end.utf8 as isize),
                    );

                solx_standard_json::OutputError::new_error_with_data(
                    Some(file_identifier),
                    None,
                    error.message(),
                    Some(source_location),
                    Some(&input_json.sources),
                )
            }));

            if let Some(output_source) = output.sources.get_mut(file_identifier) {
                output_source.ast = Some(
                    serde_json::to_value(file.tree().as_ref())
                        .map_err(|error| anyhow::anyhow!("CST serialization: {error}"))?,
                );
            }
        }

        if output.has_errors() {
            return Ok(output);
        }

        let semantic_ast = crate::SemanticAst::build(&unit);
        let mlir_context = solx_mlir::Context::new();

        for file_identifier in semantic_ast.file_identifiers() {
            let Some(source_unit) = semantic_ast.file_ast(file_identifier) else {
                continue;
            };

            let mut builder =
                solx_mlir::MlirContext::new(mlir_context.mlir(), solx_mlir::EvmVersion::Cancun);
            let mut emitter = crate::ast::source_unit::SourceUnitEmitter::new(&mut builder);
            let Some((contract_name, method_identifiers)) = emitter.emit(&source_unit)? else {
                continue;
            };

            let runtime_code_id = format!("{contract_name}_deployed");
            let mlir_source = mlir_context.finalize_module(builder, &runtime_code_id)?;

            let evm = if method_identifiers.is_empty() {
                None
            } else {
                Some(solx_standard_json::output::contract::evm::EVM {
                    method_identifiers: Some(method_identifiers),
                    ..Default::default()
                })
            };

            let contract = solx_standard_json::output::contract::Contract {
                mlir: Some(mlir_source),
                evm,
                ..Default::default()
            };

            output
                .contracts
                .entry(file_identifier.clone())
                .or_default()
                .insert(contract_name, contract);
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
