//!
//! `solx` tester executable.
//!

pub(crate) mod arguments;

use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;

use clap::Parser;
use colored::Colorize;

use self::arguments::Arguments;

/// The rayon worker stack size.
const RAYON_WORKER_STACK_SIZE: usize = 16 * 1024 * 1024;

///
/// The application entry point.
///
fn main() {
    let exit_code = match Arguments::try_parse()
        .map_err(|error| anyhow::anyhow!(error))
        .and_then(main_inner)
    {
        Ok(()) => solx_utils::EXIT_CODE_SUCCESS,
        Err(error) => {
            eprintln!("{error:?}");
            solx_utils::EXIT_CODE_FAILURE
        }
    };
    std::process::exit(exit_code);
}

///
/// The entry point wrapper used for proper error handling.
///
fn main_inner(arguments: Arguments) -> anyhow::Result<()> {
    println!(
        "    {} {} v{}",
        "Starting".bright_green().bold(),
        env!("CARGO_PKG_DESCRIPTION"),
        env!("CARGO_PKG_VERSION"),
    );

    solx_codegen_evm::initialize_target();
    solx_tester::LLVMOptions::initialize(arguments.llvm_verify_each, arguments.llvm_debug_logging)?;

    let debug_config = if arguments.debug {
        std::fs::create_dir_all(solx_tester::DEBUG_DIRECTORY)?;
        Some(solx_codegen_evm::DebugConfig::new(PathBuf::from_str(
            solx_tester::DEBUG_DIRECTORY,
        )?))
    } else {
        None
    };

    let mut thread_pool_builder = rayon::ThreadPoolBuilder::new();
    if let Some(threads) = arguments.threads {
        thread_pool_builder = thread_pool_builder.num_threads(threads);
    }
    thread_pool_builder
        .stack_size(RAYON_WORKER_STACK_SIZE)
        .build_global()
        .expect("Thread pool configuration failure");

    let toolchain = arguments
        .toolchain
        .unwrap_or(solx_tester::Toolchain::Solx);

    let mut executable_download_config_paths = Vec::with_capacity(1);
    if let Some(path) = match toolchain {
        solx_tester::Toolchain::Solx => None,
        solx_tester::Toolchain::Solc => Some("./solx-compiler-downloader/solc-upstream.json"),
        solx_tester::Toolchain::SolxMlir => Some("./solx-compiler-downloader/solx-mlir.json"),
    }
    .map(PathBuf::from)
    {
        executable_download_config_paths.push(path);
    }

    let summary = solx_tester::Summary::new(arguments.verbose, arguments.quiet).wrap();

    let path_filters = arguments
        .path
        .iter()
        .map(|path| path.strip_prefix("./").unwrap_or(path.as_str()))
        .collect();
    let filters = solx_tester::Filters::new(path_filters, arguments.mode, arguments.group);

    let compiler_tester = solx_tester::SolxTester::new(
        summary.clone(),
        filters,
        debug_config.clone(),
        arguments.workflow,
    )?;

    let run_time_start = Instant::now();
    println!(
        "     {} tests with {} worker threads",
        "Running".bright_green().bold(),
        rayon::current_num_threads(),
    );

    solx_tester::REVM::download(executable_download_config_paths)?;
    compiler_tester.run_revm(toolchain, arguments.solx, arguments.trace)?;

    let summary = solx_tester::Summary::unwrap_arc(summary);
    print!("{summary}");
    println!(
        "    {} running tests in {}m{:02}s",
        "Finished".bright_green().bold(),
        run_time_start.elapsed().as_secs() / 60,
        run_time_start.elapsed().as_secs() % 60,
    );

    if let Some(path) = arguments.benchmark {
        let benchmark = summary.benchmark(toolchain)?;
        let comparisons = Vec::new();
        let output: solx_benchmark_converter::Output =
            (benchmark, comparisons, arguments.benchmark_format).try_into()?;
        output.write_to_file(path)?;
    }

    if !summary.is_successful() {
        anyhow::bail!("");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::arguments::Arguments;

    #[test]
    fn test_manually() {
        std::env::set_current_dir("..").expect("Change directory failed");

        let arguments = Arguments {
            verbose: false,
            quiet: false,
            debug: false,
            trace: false,
            mode: vec!["Y+M3B3 0.8.33".to_owned()],
            path: vec!["tests/solidity/simple/default.sol".to_owned()],
            group: vec![],
            benchmark: None,
            benchmark_format: solx_benchmark_converter::OutputFormat::Xlsx,
            threads: Some(1),
            solx: Some(assert_cmd::cargo::cargo_bin!("SOLX").to_path_buf()),
            toolchain: Some(solx_tester::Toolchain::Solx),
            workflow: solx_tester::Workflow::BuildAndRun,
            solc_bin_config_path: Some(PathBuf::from(
                "solx-compiler-downloader/solc-bin-default.json",
            )),
            llvm_verify_each: false,
            llvm_debug_logging: false,
        };

        crate::main_inner(arguments).expect("Manual testing failed");
    }
}
