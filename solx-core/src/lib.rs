//!
//! Solidity compiler library.
//!

#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::result_large_err)]
#![allow(clippy::large_enum_variant)]

pub mod arguments;
pub mod build;
pub mod r#const;
pub mod error;
pub mod process;
pub mod project;
pub mod solc;

pub use self::arguments::Arguments;
pub use self::build::Build as EVMBuild;
pub use self::build::contract::Contract as EVMContractBuild;
pub use self::r#const::*;
pub use self::error::Error;
pub use self::error::stack_too_deep::StackTooDeep as StackTooDeepError;
pub use self::process::EXECUTABLE;
pub use self::process::input::Input as EVMProcessInput;
pub use self::process::output::Output as EVMProcessOutput;
pub use self::process::run as run_recursive;
pub use self::project::Project;
pub use self::project::contract::Contract as ProjectContract;
pub use self::solc::Solc;

use std::collections::BTreeSet;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use solx_standard_json::CollectableError;

/// The default error compatible with `solc` standard JSON output.
pub type Result<T> = std::result::Result<T, Error>;

///
/// The `main` function that implements the core CLI application logic.
///
pub fn main(
    arguments: Arguments,
    solc: impl Solc,
    messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
) -> anyhow::Result<()> {
    if arguments.version {
        let version = solc.version();
        writeln!(
            std::io::stdout(),
            "{DEFAULT_EXECUTABLE_NAME}, {DEFAULT_PACKAGE_DESCRIPTION} v{}, LLVM build: {}",
            env!("CARGO_PKG_VERSION"),
            inkwell::support::get_commit_id().to_string(),
        )?;
        writeln!(std::io::stdout(), "Version: {}", version.long)?;
        return Ok(());
    }

    let mut thread_pool_builder = rayon::ThreadPoolBuilder::new();
    if let Some(threads) = arguments.threads {
        thread_pool_builder = thread_pool_builder.num_threads(threads);
    }
    thread_pool_builder
        .stack_size(WORKER_THREAD_STACK_SIZE)
        .build_global()
        .expect("Thread pool configuration failure");

    inkwell::support::enable_llvm_pretty_stack_trace();
    solx_codegen_evm::initialize_target();

    if arguments.recursive_process {
        return self::run_recursive();
    }

    let (input_files, remappings) = arguments.split_input_files_and_remappings()?;

    let mut optimizer_settings = match arguments.optimization {
        Some(mode) => solx_codegen_evm::OptimizerSettings::try_from_cli(mode)?,
        None if arguments.standard_json.is_none() => {
            if let Ok(optimization) = std::env::var(SOLX_OPTIMIZATION_ENV) {
                if !solx_codegen_evm::OptimizerSettings::MIDDLE_END_LEVELS
                    .contains(&optimization.as_str())
                {
                    anyhow::bail!(
                        "Invalid value `{optimization}` for environment variable '{SOLX_OPTIMIZATION_ENV}': only values {} are supported.",
                        solx_codegen_evm::OptimizerSettings::MIDDLE_END_LEVELS.join(", ")
                    );
                }
                solx_codegen_evm::OptimizerSettings::try_from_cli(
                    optimization.chars().next().expect("Always exists"),
                )?
            } else {
                solx_codegen_evm::OptimizerSettings::cycles()
            }
        }
        None => solx_codegen_evm::OptimizerSettings::cycles(),
    };
    if arguments.size_fallback || std::env::var(SOLX_OPTIMIZATION_SIZE_FALLBACK_ENV).is_ok() {
        optimizer_settings.enable_fallback_to_size();
    }
    optimizer_settings.is_verify_each_enabled = arguments.llvm_verify_each;
    optimizer_settings.is_debug_logging_enabled = arguments.llvm_debug_logging;

    let mut selectors = BTreeSet::new();
    if arguments.output_evmla {
        selectors.insert(solx_standard_json::InputSelector::BytecodeEVMLA);
        selectors.insert(solx_standard_json::InputSelector::RuntimeBytecodeEVMLA);
    }
    if arguments.output_ethir {
        selectors.insert(solx_standard_json::InputSelector::BytecodeEthIR);
        selectors.insert(solx_standard_json::InputSelector::RuntimeBytecodeEthIR);
    }
    if arguments.output_llvm_ir {
        selectors.insert(solx_standard_json::InputSelector::BytecodeLLVMIRUnoptimized);
        selectors.insert(solx_standard_json::InputSelector::BytecodeLLVMIROptimized);
        selectors.insert(solx_standard_json::InputSelector::RuntimeBytecodeLLVMIRUnoptimized);
        selectors.insert(solx_standard_json::InputSelector::RuntimeBytecodeLLVMIROptimized);
    }
    if arguments.output_bytecode {
        selectors.insert(solx_standard_json::InputSelector::BytecodeObject);
    }
    if arguments.output_bytecode_runtime {
        selectors.insert(solx_standard_json::InputSelector::RuntimeBytecodeObject);
    }
    if arguments.output_assembly {
        selectors.insert(solx_standard_json::InputSelector::BytecodeLLVMAssembly);
        selectors.insert(solx_standard_json::InputSelector::RuntimeBytecodeLLVMAssembly);
    }
    if arguments.output_debug_info {
        selectors.insert(solx_standard_json::InputSelector::BytecodeDebugInfo);
    }
    if arguments.output_debug_info_runtime {
        selectors.insert(solx_standard_json::InputSelector::RuntimeBytecodeDebugInfo);
    }
    if arguments.output_metadata {
        selectors.insert(solx_standard_json::InputSelector::Metadata);
    }
    if arguments.output_abi {
        selectors.insert(solx_standard_json::InputSelector::ABI);
    }
    if arguments.output_hashes {
        selectors.insert(solx_standard_json::InputSelector::MethodIdentifiers);
    }
    if arguments.output_userdoc {
        selectors.insert(solx_standard_json::InputSelector::UserDocumentation);
    }
    if arguments.output_devdoc {
        selectors.insert(solx_standard_json::InputSelector::DeveloperDocumentation);
    }
    if arguments.output_storage_layout {
        selectors.insert(solx_standard_json::InputSelector::StorageLayout);
    }
    if arguments.output_transient_storage_layout {
        selectors.insert(solx_standard_json::InputSelector::TransientStorageLayout);
    }
    if arguments.output_ast_json {
        selectors.insert(solx_standard_json::InputSelector::AST);
    }
    if arguments.output_asm_solc_json {
        selectors.insert(solx_standard_json::InputSelector::EVMLegacyAssembly);
    }
    if arguments.output_ir {
        selectors.insert(solx_standard_json::InputSelector::Yul);
    }
    if arguments.output_benchmarks {
        selectors.insert(solx_standard_json::InputSelector::Benchmarks);
    }
    let output_selection = solx_standard_json::InputSelection::new(selectors);

    let llvm_options: Vec<String> = arguments
        .llvm_options
        .as_ref()
        .map(|options| {
            options
                .split_whitespace()
                .map(|option| option.to_owned())
                .collect()
        })
        .unwrap_or_default();

    // Determine the output configuration for IR artifacts.
    // Priority: SOLX_OUTPUT_DIR env var (debug mode) > --output-dir with IR flags
    let output_config = if let Some(debug_output_directory) =
        std::env::var("SOLX_OUTPUT_DIR").ok().map(PathBuf::from)
    {
        // Debug mode via environment variable: output all IRs, always overwrite
        std::fs::create_dir_all(debug_output_directory.as_path())?;
        Some(solx_codegen_evm::OutputConfig::new_debug(
            debug_output_directory,
        ))
    } else if let Some(ref output_directory) = arguments.output_dir {
        // New mode: output selected IRs based on CLI flags
        let has_ir_flags = arguments.output_ir
            || arguments.output_evmla
            || arguments.output_ethir
            || arguments.output_llvm_ir
            || arguments.output_assembly;
        if has_ir_flags {
            std::fs::create_dir_all(output_directory.as_path())?;
            Some(solx_codegen_evm::OutputConfig::new(
                output_directory.to_owned(),
                arguments.overwrite,
                arguments.output_ir,
                arguments.output_evmla,
                arguments.output_ethir,
                arguments.output_llvm_ir,
                arguments.output_assembly,
            ))
        } else {
            None
        }
    } else {
        None
    };

    let metadata_hash_type = arguments
        .metadata_hash
        .unwrap_or(solx_utils::MetadataHashType::IPFS);
    let append_cbor = !arguments.no_cbor_metadata;
    let use_import_callback = !arguments.no_import_callback;

    let build = if arguments.yul {
        self::yul_to_evm(
            solc,
            input_files.as_slice(),
            arguments.libraries.as_slice(),
            &output_selection,
            messages,
            arguments.evm_version,
            metadata_hash_type,
            append_cbor,
            optimizer_settings,
            llvm_options,
            output_config,
        )
    } else if arguments.llvm_ir {
        self::llvm_ir_to_evm(
            input_files.as_slice(),
            arguments.libraries.as_slice(),
            &output_selection,
            messages,
            arguments.evm_version,
            metadata_hash_type,
            append_cbor,
            optimizer_settings,
            llvm_options,
            output_config,
        )
    } else if let Some(standard_json) = arguments.standard_json {
        return self::standard_json_evm(
            solc,
            standard_json.map(PathBuf::from),
            messages,
            arguments.base_path,
            arguments.include_path,
            arguments.allow_paths,
            use_import_callback,
            output_config,
        );
    } else {
        self::standard_output_evm(
            solc,
            input_files.as_slice(),
            arguments.libraries.as_slice(),
            &output_selection,
            messages,
            arguments.evm_version,
            arguments.via_ir,
            metadata_hash_type,
            arguments.metadata_literal,
            append_cbor,
            arguments.base_path,
            arguments.include_path,
            arguments.allow_paths,
            use_import_callback,
            remappings,
            optimizer_settings,
            llvm_options,
            output_config,
        )
    }?;

    if output_selection.is_empty() {
        writeln!(
            std::io::stdout(),
            "Compiler run successful. No output generated."
        )?;
        return Ok(());
    }

    if let Some(output_directory) = arguments.output_dir {
        build.write_to_directory(&output_directory, &output_selection, arguments.overwrite)?;
    } else {
        build.write_to_terminal(&output_selection)?;
    }

    Ok(())
}

///
/// Runs the Yul mode for the EVM target.
///
pub fn yul_to_evm(
    solc: impl Solc,
    paths: &[PathBuf],
    libraries: &[String],
    output_selection: &solx_standard_json::InputSelection,
    messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
    evm_version: Option<solx_utils::EVMVersion>,
    metadata_hash_type: solx_utils::MetadataHashType,
    append_cbor: bool,
    optimizer_settings: solx_codegen_evm::OptimizerSettings,
    llvm_options: Vec<String>,
    output_config: Option<solx_codegen_evm::OutputConfig>,
) -> anyhow::Result<EVMBuild> {
    if output_selection.is_debug_info_set_for_any() {
        anyhow::bail!(solx_standard_json::OutputError::new_error(
            "Debug info is only supported for Solidity source code input."
        ));
    }

    let libraries = solx_utils::Libraries::try_from(libraries)?;
    let linker_symbols = libraries.as_linker_symbols()?;

    solc.validate_yul_paths(paths, libraries.clone())?;

    let project = Project::try_from_yul_paths(
        solc.version(),
        paths,
        libraries,
        output_selection,
        None,
        output_config.as_ref(),
    )?;

    let mut build = project.compile_to_evm(
        messages,
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
/// Runs the LLVM IR mode for the EVM target.
///
pub fn llvm_ir_to_evm(
    paths: &[PathBuf],
    libraries: &[String],
    output_selection: &solx_standard_json::InputSelection,
    messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
    evm_version: Option<solx_utils::EVMVersion>,
    metadata_hash_type: solx_utils::MetadataHashType,
    append_cbor: bool,
    optimizer_settings: solx_codegen_evm::OptimizerSettings,
    llvm_options: Vec<String>,
    output_config: Option<solx_codegen_evm::OutputConfig>,
) -> anyhow::Result<EVMBuild> {
    if output_selection.is_debug_info_set_for_any() {
        anyhow::bail!(solx_standard_json::OutputError::new_error(
            "Debug info is only supported for Solidity source code input."
        ));
    }

    let libraries = solx_utils::Libraries::try_from(libraries)?;
    let linker_symbols = libraries.as_linker_symbols()?;

    let project = Project::try_from_llvm_ir_paths(paths, libraries, output_selection, None)?;

    let mut build = project.compile_to_evm(
        messages,
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
/// Runs the standard output mode for the EVM target.
///
pub fn standard_output_evm(
    solc: impl Solc,
    paths: &[PathBuf],
    libraries: &[String],
    output_selection: &solx_standard_json::InputSelection,
    messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
    evm_version: Option<solx_utils::EVMVersion>,
    via_ir: bool,
    metadata_hash_type: solx_utils::MetadataHashType,
    metadata_literal: bool,
    append_cbor: bool,
    base_path: Option<String>,
    include_paths: Vec<String>,
    allow_paths: Option<String>,
    use_import_callback: bool,
    remappings: BTreeSet<String>,
    optimizer_settings: solx_codegen_evm::OptimizerSettings,
    llvm_options: Vec<String>,
    output_config: Option<solx_codegen_evm::OutputConfig>,
) -> anyhow::Result<EVMBuild> {
    let mut profiler = solx_codegen_evm::Profiler::default();

    let mut solc_input = solx_standard_json::Input::try_from_solidity_paths(
        paths,
        libraries,
        remappings,
        solx_standard_json::InputOptimizer::default(),
        evm_version,
        via_ir,
        output_selection,
        solx_standard_json::InputMetadata::new(metadata_literal, append_cbor, metadata_hash_type),
        llvm_options.clone(),
    )?;

    let run_solc_standard_json = profiler.start_pipeline_element("solc_Solidity_Standard_JSON");
    let mut solc_output = solc.standard_json(
        &mut solc_input,
        use_import_callback,
        base_path.as_deref(),
        include_paths.as_slice(),
        allow_paths,
    )?;
    run_solc_standard_json.borrow_mut().finish();
    solc_output.take_and_write_warnings();
    solc_output.check_errors()?;

    let linker_symbols = solc_input.settings.libraries.as_linker_symbols()?;
    solc_input.resolve_sources()?;
    let debug_info = solc_output.get_debug_info(&solc_input.sources);

    let run_solx_project = profiler.start_pipeline_element("solx_Solidity_IR_Analysis");
    let project = Project::try_from_solc_output(
        solc.version(),
        solc_input.settings.libraries.clone(),
        via_ir,
        &mut solc_output,
        Some(debug_info),
        output_config.as_ref(),
    )?;
    run_solx_project.borrow_mut().finish();
    solc_output.take_and_write_warnings();
    solc_output.check_errors()?;

    let run_solx_compile = profiler.start_pipeline_element("solx_Compilation");
    let mut build = project.compile_to_evm(
        messages,
        &solc_input.settings.output_selection,
        evm_version,
        metadata_hash_type,
        append_cbor,
        optimizer_settings.clone(),
        llvm_options,
        output_config.clone(),
    )?;
    run_solx_compile.borrow_mut().finish();
    build.take_and_write_warnings();
    build.check_errors()?;

    let mut build = if solc_input
        .settings
        .output_selection
        .is_bytecode_set_for_any()
    {
        let run_solx_link = profiler.start_pipeline_element("solx_Linking");
        let mut build = build.link(linker_symbols);
        run_solx_link.borrow_mut().finish();
        build.take_and_write_warnings();
        build.check_errors()?;
        build
    } else {
        build
    };
    build.benchmarks = profiler.to_vec();
    Ok(build)
}

///
/// Runs the standard JSON mode for the EVM target.
///
pub fn standard_json_evm(
    solc: impl Solc,
    json_path: Option<PathBuf>,
    messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
    base_path: Option<String>,
    include_paths: Vec<String>,
    allow_paths: Option<String>,
    use_import_callback: bool,
    output_config: Option<solx_codegen_evm::OutputConfig>,
) -> anyhow::Result<()> {
    let mut solc_input = solx_standard_json::Input::try_from(json_path.as_deref())?;
    let language = solc_input.language;
    let via_ir = solc_input.settings.via_ir;
    let linker_symbols = solc_input.settings.libraries.as_linker_symbols()?;

    let optimization_mode = if let Ok(optimization) = std::env::var(SOLX_OPTIMIZATION_ENV) {
        if !solx_codegen_evm::OptimizerSettings::MIDDLE_END_LEVELS.contains(&optimization.as_str())
        {
            anyhow::bail!(
                "Invalid value `{optimization}` for environment variable '{SOLX_OPTIMIZATION_ENV}': only values {} are supported.",
                solx_codegen_evm::OptimizerSettings::MIDDLE_END_LEVELS.join(", ")
            );
        }
        optimization.chars().next().expect("Always exists")
    } else {
        solc_input
            .settings
            .optimizer
            .mode
            .unwrap_or(solx_standard_json::InputOptimizer::default_mode().expect("Always exists"))
    };
    let mut optimizer_settings =
        solx_codegen_evm::OptimizerSettings::try_from_cli(optimization_mode)?;
    if solc_input
        .settings
        .optimizer
        .size_fallback
        .unwrap_or_default()
        || std::env::var(SOLX_OPTIMIZATION_SIZE_FALLBACK_ENV).is_ok()
    {
        optimizer_settings.enable_fallback_to_size();
    }
    let llvm_options = solc_input.settings.llvm_options.clone();

    let metadata_hash_type = solc_input.settings.metadata.bytecode_hash;
    let append_cbor = solc_input.settings.metadata.append_cbor;

    let mut profiler = solx_codegen_evm::Profiler::default();
    let (mut solc_output, project) = match language {
        solx_standard_json::InputLanguage::Solidity => {
            let run_solc_standard_json =
                profiler.start_pipeline_element("solc_Solidity_Standard_JSON");
            let mut solc_output = solc.standard_json(
                &mut solc_input,
                use_import_callback,
                base_path.as_deref(),
                include_paths.as_slice(),
                allow_paths,
            )?;
            run_solc_standard_json.borrow_mut().finish();

            solc_input.resolve_sources()?;
            let function_definitions = solc_output.get_debug_info(&solc_input.sources);

            if solc_output.has_errors() {
                solc_output.write_and_exit(&solc_input.settings.output_selection);
            }
            messages
                .lock()
                .expect("Sync")
                .extend(solc_output.errors.drain(..));

            let run_solx_project = profiler.start_pipeline_element("solx_Solidity_IR_Analysis");
            let project = Project::try_from_solc_output(
                solc.version(),
                solc_input.settings.libraries.clone(),
                via_ir,
                &mut solc_output,
                Some(function_definitions),
                output_config.as_ref(),
            )?;
            run_solx_project.borrow_mut().finish();
            if solc_output.has_errors() {
                solc_output.write_and_exit(&solc_input.settings.output_selection);
            }

            (solc_output, project)
        }
        solx_standard_json::InputLanguage::Yul => {
            if solc_input
                .settings
                .output_selection
                .is_debug_info_set_for_any()
            {
                anyhow::bail!(solx_standard_json::OutputError::new_error(
                    "Debug info is only supported for Solidity source code input."
                ));
            }

            let run_solc_validate_yul = profiler.start_pipeline_element("solc_Yul_Validation");
            let mut solc_output = solc.validate_yul_standard_json(&mut solc_input)?;
            run_solc_validate_yul.borrow_mut().finish();
            if solc_output.has_errors() {
                solc_output.write_and_exit(&solc_input.settings.output_selection);
            }

            let run_solx_yul_project = profiler.start_pipeline_element("solx_Yul_IR_Analysis");
            let project = Project::try_from_yul_sources(
                solc.version(),
                solc_input.sources,
                solc_input.settings.libraries.clone(),
                &solc_input.settings.output_selection,
                Some(&mut solc_output),
                output_config.as_ref(),
            )?;
            run_solx_yul_project.borrow_mut().finish();
            if solc_output.has_errors() {
                solc_output.write_and_exit(&solc_input.settings.output_selection);
            }

            (solc_output, project)
        }
        solx_standard_json::InputLanguage::LLVMIR => {
            if solc_input
                .settings
                .output_selection
                .is_debug_info_set_for_any()
            {
                anyhow::bail!(solx_standard_json::OutputError::new_error(
                    "Debug info is only supported for Solidity source code input."
                ));
            }

            let mut solc_output = solx_standard_json::Output::new(&solc_input.sources);

            let run_solx_llvm_ir_project = profiler.start_pipeline_element("solx_LLVM_IR_Analysis");
            let project = Project::try_from_llvm_ir_sources(
                solc_input.sources,
                solc_input.settings.libraries.clone(),
                &solc_input.settings.output_selection,
                Some(&mut solc_output),
            )?;
            run_solx_llvm_ir_project.borrow_mut().finish();
            if solc_output.has_errors() {
                solc_output.write_and_exit(&solc_input.settings.output_selection);
            }

            (solc_output, project)
        }
    };

    let run_solx_compile = profiler.start_pipeline_element("solx_Compilation");
    let build = project.compile_to_evm(
        messages,
        &solc_input.settings.output_selection,
        solc_input.settings.evm_version,
        metadata_hash_type,
        append_cbor,
        optimizer_settings.clone(),
        llvm_options,
        output_config.clone(),
    )?;
    run_solx_compile.borrow_mut().finish();
    let output_selection = solc_input.settings.output_selection.clone();
    if build.has_errors() {
        build.write_to_standard_json(
            &mut solc_output,
            &solc_input.settings.output_selection,
            false,
            profiler.to_vec(),
        )?;
        solc_output.write_and_exit(&solc_input.settings.output_selection);
    }
    let build = if output_selection.is_bytecode_set_for_any() {
        let run_solx_link = profiler.start_pipeline_element("solx_Linking");
        let build = build.link(linker_symbols);
        run_solx_link.borrow_mut().finish();
        build
    } else {
        build
    };
    build.write_to_standard_json(&mut solc_output, &output_selection, true, profiler.to_vec())?;
    solc_output.write_and_exit(&output_selection);
}
