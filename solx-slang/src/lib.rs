//!
//! Slang Solidity frontend for solx.
//!

#![allow(clippy::too_many_arguments)]

pub(crate) mod slang;

pub use self::slang::SlangFrontend;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use solx_standard_json::CollectableError;

///
/// The Slang + MLIR compilation pipeline entry point.
///
pub fn main(
    arguments: solx_core::Arguments,
    slang: impl solx_core::Frontend,
    messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
) -> anyhow::Result<()> {
    arguments.warn_unsupported_outputs(&messages);

    if solx_core::initialize(&arguments)? {
        return Ok(());
    }

    let output_config = arguments.output_config()?;

    if let Some(standard_json) = arguments.standard_json {
        return mlir_standard_json(
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
        mlir_directory(
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
        mlir_terminal(
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

///
/// Parses Solidity sources with the Slang frontend and returns AST JSONs.
///
fn get_ast_jsons(
    input_files: &[PathBuf],
    libraries: &[String],
    remappings: BTreeSet<String>,
    slang: &impl solx_core::Frontend,
) -> anyhow::Result<BTreeMap<String, Option<serde_json::Value>>> {
    let mut input = solx_standard_json::Input::try_from_solidity_paths(
        input_files,
        libraries,
        remappings,
        solx_standard_json::InputOptimizer::default(),
        None,
        false,
        &solx_standard_json::InputSelection::default(),
        solx_standard_json::InputMetadata::default(),
        vec![],
    )?;

    let mut output = slang.standard_json(&mut input, false, None, &[], None)?;
    output.take_and_write_warnings();
    output.check_errors()?;

    Ok(output
        .sources
        .iter_mut()
        .map(|(path, source)| (path.to_owned(), source.ast.take()))
        .collect())
}

///
/// Core MLIR → EVM compilation.
///
/// Creates MLIR contracts from the given paths, compiles through the `Project`
/// pipeline, and links. Returns the linked `EVMBuild`.
///
/// This is a temporary stub — the MLIR source is a fixed module that stores
/// the value 42 and returns it. Will be replaced by actual Solidity → MLIR
/// lowering.
///
fn mlir_to_evm(
    contract_paths: &[String],
    libraries: solx_utils::Libraries,
    output_selection: &solx_standard_json::InputSelection,
    slang_version: &solx_standard_json::Version,
    ast_jsons: BTreeMap<String, Option<serde_json::Value>>,
    messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
    evm_version: Option<solx_utils::EVMVersion>,
    metadata_hash_type: solx_utils::MetadataHashType,
    append_cbor: bool,
    optimizer_settings: solx_codegen_evm::OptimizerSettings,
    llvm_options: Vec<String>,
    output_config: Option<solx_codegen_evm::OutputConfig>,
) -> anyhow::Result<solx_core::EVMBuild> {
    let linker_symbols = libraries.as_linker_symbols()?;
    messages
        .lock()
        .expect("Sync")
        .push(solx_standard_json::OutputError::new_warning_with_data(
            None,
            None,
            "EVM bytecode is currently a stub. Slang frontend is under construction.",
            None,
            None,
        ));

    let mlir_source = r#"
    module {
      llvm.func @llvm.evm.return(!llvm.ptr<1>, i256)

      llvm.func @__entry() {
        %c42 = llvm.mlir.constant(42 : i256) : i256
        %c0 = llvm.mlir.constant(0 : i256) : i256
        %ptr = llvm.inttoptr %c0 : i256 to !llvm.ptr<1>
        llvm.store %c42, %ptr : i256, !llvm.ptr<1>
        %c32 = llvm.mlir.constant(32 : i256) : i256
        llvm.call @llvm.evm.return(%ptr, %c32) : (!llvm.ptr<1>, i256) -> ()
        llvm.unreachable
      }
    }
    "#;

    let mut contracts = BTreeMap::new();
    for path in contract_paths {
        let contract_name = solx_utils::ContractName::new(path.clone(), None);
        let ir = solx_core::project::contract::ir::mlir::MLIR {
            source: mlir_source.to_owned(),
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
        Some(ast_jsons),
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

///
/// Terminal output: validate, compile MLIR, and write bytecode to stdout.
///
fn mlir_terminal(
    input_files: &[PathBuf],
    libraries: &[String],
    remappings: BTreeSet<String>,
    output_selection: &solx_standard_json::InputSelection,
    slang: &impl solx_core::Frontend,
    messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
    evm_version: Option<solx_utils::EVMVersion>,
    metadata_hash_type: solx_utils::MetadataHashType,
    append_cbor: bool,
    optimizer_settings: solx_codegen_evm::OptimizerSettings,
    llvm_options: Vec<String>,
    output_config: Option<solx_codegen_evm::OutputConfig>,
) -> anyhow::Result<()> {
    let ast_jsons = get_ast_jsons(input_files, libraries, remappings, slang)?;

    let paths: Vec<String> = input_files
        .iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect();
    let libraries = solx_utils::Libraries::try_from(libraries)?;
    let build = mlir_to_evm(
        &paths,
        libraries,
        output_selection,
        slang.version(),
        ast_jsons,
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

///
/// Directory output: validate, compile MLIR, and write bytecode to files.
///
fn mlir_directory(
    input_files: &[PathBuf],
    libraries: &[String],
    remappings: BTreeSet<String>,
    output_selection: &solx_standard_json::InputSelection,
    output_directory: &std::path::Path,
    overwrite: bool,
    slang: &impl solx_core::Frontend,
    messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
    evm_version: Option<solx_utils::EVMVersion>,
    metadata_hash_type: solx_utils::MetadataHashType,
    append_cbor: bool,
    optimizer_settings: solx_codegen_evm::OptimizerSettings,
    llvm_options: Vec<String>,
    output_config: Option<solx_codegen_evm::OutputConfig>,
) -> anyhow::Result<()> {
    let ast_jsons = get_ast_jsons(input_files, libraries, remappings, slang)?;

    let paths: Vec<String> = input_files
        .iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect();
    let libraries = solx_utils::Libraries::try_from(libraries)?;
    let build = mlir_to_evm(
        &paths,
        libraries,
        output_selection,
        slang.version(),
        ast_jsons,
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

///
/// Standard JSON output: read JSON input, validate with Slang, compile MLIR,
/// and write JSON output.
///
fn mlir_standard_json(
    slang: impl solx_core::Frontend,
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

    let optimization_mode = if let Ok(optimization) =
        std::env::var(solx_core::SOLX_OPTIMIZATION_ENV)
    {
        if !solx_codegen_evm::OptimizerSettings::MIDDLE_END_LEVELS.contains(&optimization.as_str())
        {
            anyhow::bail!(
                "Invalid value `{optimization}` for environment variable '{}': only values {} are supported.",
                solx_core::SOLX_OPTIMIZATION_ENV,
                solx_codegen_evm::OptimizerSettings::MIDDLE_END_LEVELS.join(", ")
            );
        }
        optimization.chars().next().expect("Always exists")
    } else {
        input
            .settings
            .optimizer
            .mode
            .unwrap_or(solx_standard_json::InputOptimizer::default_mode().expect("Always exists"))
    };
    let mut optimizer_settings =
        solx_codegen_evm::OptimizerSettings::try_from_cli(optimization_mode)?;
    if input.settings.optimizer.size_fallback.unwrap_or_default()
        || std::env::var(solx_core::SOLX_OPTIMIZATION_SIZE_FALLBACK_ENV).is_ok()
    {
        optimizer_settings.enable_fallback_to_size();
    }
    let llvm_options = input.settings.llvm_options.clone();
    let evm_version = input.settings.evm_version;
    let metadata_hash_type = input.settings.metadata.bytecode_hash;
    let append_cbor = input.settings.metadata.append_cbor;

    let mut output = slang.standard_json(&mut input, false, None, &[], None)?;
    if output.has_errors() {
        output.write_and_exit(&output_selection);
    }
    messages
        .lock()
        .expect("Sync")
        .extend(output.errors.drain(..));

    let ast_jsons = output
        .sources
        .iter_mut()
        .map(|(path, source)| (path.to_owned(), source.ast.take()))
        .collect();

    let contract_paths: Vec<String> = input.sources.keys().cloned().collect();
    let build = mlir_to_evm(
        &contract_paths,
        input.settings.libraries,
        &output_selection,
        slang.version(),
        ast_jsons,
        messages,
        evm_version,
        metadata_hash_type,
        append_cbor,
        optimizer_settings,
        llvm_options,
        output_config,
    )?;

    build.write_to_standard_json(&mut output, &output_selection, true, vec![])?;
    output.write_and_exit(&output_selection);
}
