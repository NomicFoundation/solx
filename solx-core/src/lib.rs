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
pub mod frontend;
pub mod process;
pub mod project;

pub use self::arguments::Arguments;
pub use self::build::Build as EVMBuild;
pub use self::build::contract::Contract as EVMContractBuild;
pub use self::r#const::*;
pub use self::error::Error;
pub use self::error::stack_too_deep::StackTooDeep as StackTooDeepError;
pub use self::frontend::Frontend;
pub use self::process::EXECUTABLE;
pub use self::process::input::Input as EVMProcessInput;
pub use self::process::output::Output as EVMProcessOutput;
pub use self::process::run as run_subprocess;
pub use self::project::Project;
pub use self::project::contract::Contract as ProjectContract;

use std::collections::BTreeSet;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use solx_standard_json::CollectableError;

/// The default error compatible with `solc` standard JSON output.
pub type Result<T> = std::result::Result<T, Error>;

///
/// Initialize the compiler runtime: rayon thread pool, LLVM stack trace, and
/// EVM target.
///
/// If `arguments.recursive_process` is set, runs the subprocess handler and
/// returns `Ok(true)` — the caller should return immediately.
/// Otherwise returns `Ok(false)`.
///
pub fn initialize(arguments: &Arguments) -> anyhow::Result<bool> {
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
        self::run_subprocess()?;
        return Ok(true);
    }

    Ok(false)
}

///
/// The `main` function that implements the core CLI application logic.
///
pub fn main(
    arguments: Arguments,
    frontend: impl Frontend,
    messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
) -> anyhow::Result<()> {
    if self::initialize(&arguments)? {
        return Ok(());
    }

    #[cfg(all(feature = "solc", feature = "mlir"))]
    {
        if arguments.yul {
            anyhow::bail!("--yul is not supported in MLIR passthrough mode.");
        }
        if arguments.llvm_ir {
            anyhow::bail!("--llvm-ir is not supported in MLIR passthrough mode.");
        }
        if arguments.output_assembly {
            anyhow::bail!(
                "--asm is not supported in MLIR passthrough mode. Use --asm-solc-json for EVM assembly."
            );
        }
        if arguments.output_debug_info {
            anyhow::bail!("--debug-info is not supported in MLIR passthrough mode.");
        }
        if arguments.output_debug_info_runtime {
            anyhow::bail!("--debug-info-runtime is not supported in MLIR passthrough mode.");
        }
        if arguments.output_llvm_ir {
            anyhow::bail!("--emit-llvm-ir is not supported in MLIR passthrough mode.");
        }
        if arguments.output_evmla {
            anyhow::bail!("--evmla is not supported in MLIR passthrough mode.");
        }
        if arguments.output_ethir {
            anyhow::bail!("--ethir is not supported in MLIR passthrough mode.");
        }
    }

    let (input_files, remappings) = arguments.split_input_files_and_remappings()?;

    let optimizer_settings = arguments.optimizer_settings()?;
    let output_selection = arguments.output_selection();
    let llvm_options = arguments.llvm_options();

    let output_config = arguments.output_config()?;

    let metadata_hash_type = arguments
        .metadata_hash
        .unwrap_or(solx_utils::MetadataHashType::IPFS);
    let append_cbor = !arguments.no_cbor_metadata;
    let use_import_callback = !arguments.no_import_callback;

    let build = if arguments.yul {
        self::yul_to_evm(
            frontend,
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
            frontend,
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
            frontend,
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
    frontend: impl Frontend,
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

    frontend.validate_yul_paths(paths, libraries.clone())?;

    let project = Project::try_from_yul_paths(
        frontend.version(),
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
    frontend: impl Frontend,
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
    // In passthrough mode, wire the CLI -O flag to solc's optimizer.mode.
    #[cfg(all(feature = "solc", feature = "mlir"))]
    {
        solc_input.settings.optimizer.mode = Some(optimizer_settings.middle_end_as_char());
    }

    let run_solc_standard_json = profiler.start_pipeline_element("solc_Solidity_Standard_JSON");
    let mut solc_output = frontend.standard_json(
        &mut solc_input,
        use_import_callback,
        base_path.as_deref(),
        include_paths.as_slice(),
        allow_paths,
    )?;
    run_solc_standard_json.borrow_mut().finish();
    solc_output.take_and_write_warnings();
    solc_output.check_errors()?;

    // MLIR passthrough: solc produced final bytecode, extract it directly.
    #[cfg(all(feature = "solc", feature = "mlir"))]
    {
        let _ = &optimizer_settings;
        let _ = &output_config;
        Ok(EVMBuild::from_solc_output(solc_output, messages))
    }

    #[cfg(not(all(feature = "solc", feature = "mlir")))]
    {
        let linker_symbols = solc_input.settings.libraries.as_linker_symbols()?;
        solc_input.resolve_sources()?;
        let debug_info = solc_output.get_debug_info(&solc_input.sources);

        let run_solx_project = profiler.start_pipeline_element("solx_Solidity_IR_Analysis");
        let project = Project::try_from_solc_output(
            frontend.version(),
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
}

///
/// Runs the standard JSON mode for the EVM target.
///
pub fn standard_json_evm(
    frontend: impl Frontend,
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

    let mut profiler = solx_codegen_evm::Profiler::default();

    // MLIR passthrough: solc produces final output. Call solc and write the result directly.
    #[cfg(all(feature = "solc", feature = "mlir"))]
    {
        let _ = &output_config;
        match language {
            solx_standard_json::InputLanguage::Solidity => {
                let run_solc_standard_json =
                    profiler.start_pipeline_element("solc_Solidity_Standard_JSON");
                let mut solc_output = frontend.standard_json(
                    &mut solc_input,
                    use_import_callback,
                    base_path.as_deref(),
                    include_paths.as_slice(),
                    allow_paths,
                )?;
                run_solc_standard_json.borrow_mut().finish();

                let merged: Vec<_> = messages.lock().expect("Sync").drain(..).collect();
                solc_output.errors.extend(merged);
                solc_output.write_and_exit(&solc_input.settings.output_selection);
            }
            solx_standard_json::InputLanguage::Yul => {
                anyhow::bail!(solx_standard_json::OutputError::new_error(
                    "Yul language is not supported in MLIR passthrough mode. Use Solidity input."
                ));
            }
            solx_standard_json::InputLanguage::LLVMIR => {
                anyhow::bail!(solx_standard_json::OutputError::new_error(
                    "LLVM IR language is not supported in MLIR passthrough mode. Use Solidity input."
                ));
            }
        }
    }

    // Normal compilation path.
    #[cfg(not(all(feature = "solc", feature = "mlir")))]
    {
        let via_ir = solc_input.settings.via_ir;
        let linker_symbols = solc_input.settings.libraries.as_linker_symbols()?;

        let optimization_mode = if let Ok(optimization) = std::env::var(SOLX_OPTIMIZATION_ENV) {
            if !solx_codegen_evm::OptimizerSettings::MIDDLE_END_LEVELS
                .contains(&optimization.as_str())
            {
                anyhow::bail!(
                    "Invalid value `{optimization}` for environment variable '{SOLX_OPTIMIZATION_ENV}': only values {} are supported.",
                    solx_codegen_evm::OptimizerSettings::MIDDLE_END_LEVELS.join(", ")
                );
            }
            optimization.chars().next().expect("Always exists")
        } else {
            solc_input.settings.optimizer.mode.unwrap_or(
                solx_standard_json::InputOptimizer::default_mode().expect("Always exists"),
            )
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

        let (mut solc_output, project) = match language {
            solx_standard_json::InputLanguage::Solidity => {
                let run_solc_standard_json =
                    profiler.start_pipeline_element("solc_Solidity_Standard_JSON");
                let mut solc_output = frontend.standard_json(
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
                    frontend.version(),
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
                let mut solc_output = frontend.validate_yul_standard_json(&mut solc_input)?;
                run_solc_validate_yul.borrow_mut().finish();
                if solc_output.has_errors() {
                    solc_output.write_and_exit(&solc_input.settings.output_selection);
                }

                let run_solx_yul_project = profiler.start_pipeline_element("solx_Yul_IR_Analysis");
                let project = Project::try_from_yul_sources(
                    frontend.version(),
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

                let run_solx_llvm_ir_project =
                    profiler.start_pipeline_element("solx_LLVM_IR_Analysis");
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
        build.write_to_standard_json(
            &mut solc_output,
            &output_selection,
            true,
            profiler.to_vec(),
        )?;
        solc_output.write_and_exit(&output_selection);
    }
}

///
/// Prints the compiler version information to stdout.
///
pub fn print_version(frontend: &impl Frontend) -> anyhow::Result<()> {
    writeln!(
        std::io::stdout(),
        "{DEFAULT_EXECUTABLE_NAME} v{}, {DEFAULT_PACKAGE_DESCRIPTION}, Front end: {}, LLVM build: {}",
        env!("CARGO_PKG_VERSION"),
        frontend.name(),
        inkwell::support::get_commit_id().to_string(),
    )?;
    writeln!(std::io::stdout(), "Version: {}", frontend.version().long)?;
    #[cfg(all(feature = "solc", feature = "mlir"))]
    writeln!(std::io::stdout(), "Backend: MLIR (passthrough)")?;
    Ok(())
}
