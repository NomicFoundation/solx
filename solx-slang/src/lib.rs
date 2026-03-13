//!
//! Slang Solidity frontend for solx.
//!

#![allow(clippy::too_many_arguments)]

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use slang_solidity::compilation::CompilationUnit;

use solx_core::Frontend;
use solx_standard_json::CollectableError;

/// Slang AST construction from parsed compilation units.
pub mod ast;
/// MLIR code generation from the Slang SemanticAst.
pub mod codegen;
/// Compilation builder configuration for the Slang frontend.
pub mod compilation_config;
/// Slang Solidity frontend implementation.
pub mod slang;

pub use self::ast::SemanticAst;
pub use self::compilation_config::SlangCompilationConfig;
pub use self::slang::SlangFrontend;

impl SlangFrontend {
    /// Collects resolved source contents into a map, reporting unavailable sources as errors.
    pub(crate) fn collect_sources(
        input_sources: &BTreeMap<String, solx_standard_json::InputSource>,
        output: &mut solx_standard_json::Output,
    ) -> BTreeMap<String, String> {
        let mut sources = BTreeMap::new();
        for (path, source) in input_sources {
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
                        Some(input_sources),
                    ));
                continue;
            };
            sources.insert(path.clone(), source_code.to_owned());
        }
        sources
    }

    /// Reports compilation errors and serializes ASTs from a compilation unit.
    pub(crate) fn report_compilation_results(
        unit: &CompilationUnit,
        input_sources: &BTreeMap<String, solx_standard_json::InputSource>,
        output: &mut solx_standard_json::Output,
    ) -> anyhow::Result<()> {
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
                    Some(input_sources),
                )
            }));

            if let Some(output_source) = output.sources.get_mut(file_identifier) {
                output_source.ast = Some(
                    serde_json::to_value(file.tree().as_ref())
                        .map_err(|error| anyhow::anyhow!("CST serialization: {error}"))?,
                );
            }
        }
        Ok(())
    }

    /// The Slang + MLIR compilation pipeline entry point.
    pub fn main(
        arguments: solx_core::Arguments,
        slang: SlangFrontend,
        messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
    ) -> anyhow::Result<()> {
        arguments.warn_unsupported_outputs(&messages);

        if solx_core::initialize(&arguments)? {
            return Ok(());
        }

        let output_config = arguments.output_config()?;

        if let Some(standard_json) = arguments.standard_json {
            return Self::mlir_standard_json(
                slang,
                standard_json.map(PathBuf::from),
                messages,
                output_config,
            );
        }

        let (input_files, remappings) = arguments.split_input_files_and_remappings()?;
        let output_selection = arguments.output_selection();

        let optimizer_settings = arguments.optimizer_settings()?;
        let llvm_options = arguments.llvm_options();
        let metadata_hash_type = arguments
            .metadata_hash
            .unwrap_or(solx_utils::MetadataHashType::IPFS);
        let append_cbor = !arguments.no_cbor_metadata;

        if let Some(ref output_directory) = arguments.output_dir {
            Self::mlir_directory(
                input_files.as_slice(),
                arguments.libraries.as_slice(),
                remappings,
                &output_selection,
                output_directory,
                arguments.overwrite,
                &slang,
                messages,
                arguments.evm_version,
                metadata_hash_type,
                append_cbor,
                optimizer_settings,
                llvm_options,
                output_config,
            )
        } else {
            Self::mlir_terminal(
                input_files.as_slice(),
                arguments.libraries.as_slice(),
                remappings,
                &output_selection,
                &slang,
                messages,
                arguments.evm_version,
                metadata_hash_type,
                append_cbor,
                optimizer_settings,
                llvm_options,
                output_config,
            )
        }
    }

    /// Core MLIR -> EVM compilation.
    ///
    /// Generates MLIR from the `SemanticAst` using the melior API, compiles through
    /// the `Project` pipeline, and links. Returns the linked `EVMBuild`.
    ///
    /// Only the first contract per source file is lowered. Function bodies are
    /// limited to constant return expressions for now.
    fn mlir_to_evm(
        contract_paths: &[String],
        libraries: solx_utils::Libraries,
        output_selection: &solx_standard_json::InputSelection,
        slang_version: &solx_standard_json::Version,
        semantic_ast: &SemanticAst,
        messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
        evm_version: Option<solx_utils::EVMVersion>,
        metadata_hash_type: solx_utils::MetadataHashType,
        append_cbor: bool,
        optimizer_settings: solx_codegen_evm::OptimizerSettings,
        llvm_options: Vec<String>,
        output_config: Option<solx_codegen_evm::OutputConfig>,
    ) -> anyhow::Result<solx_core::EVMBuild> {
        let linker_symbols = libraries.as_linker_symbols()?;

        // TODO: A new MLIR context is created per compilation. Consider threading
        // a long-lived context through callers to amortize initialization cost.
        let mlir_context = solx_mlir::Context::new();

        let mut contracts = BTreeMap::new();
        for path in contract_paths {
            let source_unit = semantic_ast
                .file_ast(path)
                .ok_or_else(|| anyhow::anyhow!("no AST for source file '{path}'"))?;

            let mut state = codegen::MlirContext::new(mlir_context.mlir());
            let has_contract =
                codegen::source_unit::SourceUnitEmitter::new(&mut state).emit(&source_unit)?;
            if !has_contract {
                continue;
            }
            let mlir_source = state.into_mlir_source()?;

            let contract_name = solx_utils::ContractName::new(path.clone(), None);
            let ir = solx_core::project::contract::ir::mlir::MLIR {
                source: mlir_source,
            };
            let contract = solx_core::ProjectContract::new(
                contract_name,
                Some(ir.into()),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            );
            contracts.insert(path.clone(), contract);
        }

        let project = solx_core::Project::new(
            solx_standard_json::InputLanguage::Solidity,
            Some(slang_version.to_owned()),
            contracts,
            None,
            libraries,
            None,
        );

        let mut build = project.compile_to_evm(
            messages.clone(),
            output_selection,
            evm_version,
            metadata_hash_type,
            append_cbor,
            optimizer_settings,
            llvm_options,
            output_config,
        )?;
        build.take_and_write_warnings();
        build.check_errors()?;

        Ok(if output_selection.is_bytecode_set_for_any() {
            let mut build = build.link(linker_symbols);
            build.take_and_write_warnings();
            build.check_errors()?;
            build
        } else {
            build
        })
    }

    /// Compiles Solidity sources from filesystem paths to an EVM build.
    fn compile_paths_to_evm(
        input_files: &[PathBuf],
        libraries: &[String],
        remappings: BTreeSet<String>,
        output_selection: &solx_standard_json::InputSelection,
        slang: &SlangFrontend,
        messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
        evm_version: Option<solx_utils::EVMVersion>,
        metadata_hash_type: solx_utils::MetadataHashType,
        append_cbor: bool,
        optimizer_settings: solx_codegen_evm::OptimizerSettings,
        llvm_options: Vec<String>,
        output_config: Option<solx_codegen_evm::OutputConfig>,
    ) -> anyhow::Result<solx_core::EVMBuild> {
        let unit = slang.compile_from_paths(input_files, libraries, remappings)?;

        // Surface parse errors before attempting AST lowering.
        let parse_errors: Vec<String> = unit
            .files()
            .iter()
            .flat_map(|f| {
                f.errors()
                    .iter()
                    .map(move |e| format!("{}: {}", f.id(), e.message()))
            })
            .collect();
        if !parse_errors.is_empty() {
            anyhow::bail!("parse errors:\n{}", parse_errors.join("\n"));
        }

        let semantic_ast = SemanticAst::build(&unit);

        let paths: Vec<String> = input_files
            .iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect();
        let libraries = solx_utils::Libraries::try_from(libraries)?;
        let mut build = Self::mlir_to_evm(
            &paths,
            libraries,
            output_selection,
            slang.version(),
            &semantic_ast,
            messages,
            evm_version,
            metadata_hash_type,
            append_cbor,
            optimizer_settings,
            llvm_options,
            output_config,
        )?;
        build.ast_jsons = Some(semantic_ast.stub_ast_jsons());
        Ok(build)
    }

    /// Terminal output: compile sources and write bytecode to stdout.
    fn mlir_terminal(
        input_files: &[PathBuf],
        libraries: &[String],
        remappings: BTreeSet<String>,
        output_selection: &solx_standard_json::InputSelection,
        slang: &SlangFrontend,
        messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
        evm_version: Option<solx_utils::EVMVersion>,
        metadata_hash_type: solx_utils::MetadataHashType,
        append_cbor: bool,
        optimizer_settings: solx_codegen_evm::OptimizerSettings,
        llvm_options: Vec<String>,
        output_config: Option<solx_codegen_evm::OutputConfig>,
    ) -> anyhow::Result<()> {
        let build = Self::compile_paths_to_evm(
            input_files,
            libraries,
            remappings,
            output_selection,
            slang,
            messages,
            evm_version,
            metadata_hash_type,
            append_cbor,
            optimizer_settings,
            llvm_options,
            output_config,
        )?;

        if output_selection.is_empty() {
            writeln!(
                std::io::stdout(),
                "Compiler run successful. No output generated."
            )?;
            return Ok(());
        }

        build.write_to_terminal(output_selection)?;
        Ok(())
    }

    /// Directory output: compile sources and write bytecode to files.
    fn mlir_directory(
        input_files: &[PathBuf],
        libraries: &[String],
        remappings: BTreeSet<String>,
        output_selection: &solx_standard_json::InputSelection,
        output_directory: &std::path::Path,
        overwrite: bool,
        slang: &SlangFrontend,
        messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
        evm_version: Option<solx_utils::EVMVersion>,
        metadata_hash_type: solx_utils::MetadataHashType,
        append_cbor: bool,
        optimizer_settings: solx_codegen_evm::OptimizerSettings,
        llvm_options: Vec<String>,
        output_config: Option<solx_codegen_evm::OutputConfig>,
    ) -> anyhow::Result<()> {
        let build = Self::compile_paths_to_evm(
            input_files,
            libraries,
            remappings,
            output_selection,
            slang,
            messages,
            evm_version,
            metadata_hash_type,
            append_cbor,
            optimizer_settings,
            llvm_options,
            output_config,
        )?;

        if output_selection.is_empty() {
            writeln!(
                std::io::stdout(),
                "Compiler run successful. No output generated."
            )?;
            return Ok(());
        }

        build.write_to_directory(output_directory, output_selection, overwrite)?;
        Ok(())
    }

    /// Standard JSON output: read JSON input, compile with Slang, and write JSON output.
    fn mlir_standard_json(
        slang: SlangFrontend,
        json_path: Option<PathBuf>,
        messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
        output_config: Option<solx_codegen_evm::OutputConfig>,
    ) -> anyhow::Result<()> {
        let mut input = solx_standard_json::Input::try_from(json_path.as_deref())?;

        if input.language != solx_standard_json::InputLanguage::Solidity {
            return solx_core::standard_json_evm(
                slang,
                json_path,
                messages,
                None,
                vec![],
                None,
                false,
                output_config,
            );
        }

        let output_selection = input.settings.output_selection.clone();

        let optimizer_settings = solx_codegen_evm::OptimizerSettings::try_from_standard_json(
            input.settings.optimizer.mode,
            input.settings.optimizer.size_fallback,
        )?;
        let llvm_options = input.settings.llvm_options.clone();
        let evm_version = input.settings.evm_version;
        let metadata_hash_type = input.settings.metadata.bytecode_hash;
        let append_cbor = input.settings.metadata.append_cbor;

        let mut output = solx_standard_json::Output::new(&input.sources);

        if let Err(error) = input.resolve_sources() {
            output
                .errors
                .push(solx_standard_json::OutputError::new_error(error));
            output.write_and_exit(&output_selection);
        }

        let sources = Self::collect_sources(&input.sources, &mut output);
        let unit = slang.compile(sources)?;
        Self::report_compilation_results(&unit, &input.sources, &mut output)?;

        if output.has_errors() {
            output.write_and_exit(&output_selection);
        }
        messages
            .lock()
            .expect("lock is never poisoned because worker threads do not panic")
            .extend(output.errors.drain(..));

        let semantic_ast = SemanticAst::build(&unit);

        let contract_paths: Vec<String> = input.sources.keys().cloned().collect();
        let mut build = Self::mlir_to_evm(
            &contract_paths,
            input.settings.libraries,
            &output_selection,
            slang.version(),
            &semantic_ast,
            messages,
            evm_version,
            metadata_hash_type,
            append_cbor,
            optimizer_settings,
            llvm_options,
            output_config,
        )?;
        build.ast_jsons = Some(semantic_ast.stub_ast_jsons());

        build.write_to_standard_json(&mut output, &output_selection, true, vec![])?;
        output.write_and_exit(&output_selection);
    }
}
