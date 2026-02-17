//!
//! `solx` developer tool.
//!

#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::too_many_arguments)]

use std::collections::HashSet;
use std::str::FromStr;

use clap::Parser;

use solx_dev::Arguments;
use solx_dev::arguments::llvm::LLVM as LLVMArguments;
use solx_dev::arguments::solc::Solc as SolcArguments;
use solx_dev::arguments::test::Test as TestArguments;

///
/// The entry.
///
fn main() {
    match main_inner() {
        Ok(()) => std::process::exit(0),
        Err(error) => {
            eprintln!("{error:?}");
            std::process::exit(1)
        }
    }
}

///
/// The entry result wrapper.
///
fn main_inner() -> anyhow::Result<()> {
    let arguments = Arguments::parse();

    match arguments {
        Arguments::LLVM(LLVMArguments::Build(arguments)) => {
            let extra_args_unescaped: Vec<String> = arguments
                .extra_args
                .iter()
                .map(|argument| {
                    argument
                        .strip_prefix('\\')
                        .unwrap_or(argument.as_str())
                        .to_owned()
                })
                .collect();
            if arguments.verbose {
                println!("\nextra_args: {:#?}", arguments.extra_args);
                println!("\nextra_args_unescaped: {extra_args_unescaped:#?}");
            }

            if let Some(ccache_variant) = arguments.ccache_variant {
                solx_dev::exists(ccache_variant.to_string().as_str())?;
            }

            let mut projects = arguments
                .llvm_projects
                .into_iter()
                .map(|project| solx_dev::LLVMProject::from_str(project.to_string().as_str()))
                .collect::<Result<HashSet<solx_dev::LLVMProject>, String>>()
                .map_err(|project| anyhow::anyhow!("Unknown LLVM project `{project}`"))?;
            projects.insert(solx_dev::LLVMProject::LLD);

            solx_dev::llvm_build(
                arguments.build_type,
                projects,
                arguments.enable_rtti,
                arguments.enable_tests,
                arguments.enable_coverage,
                extra_args_unescaped,
                arguments.ccache_variant,
                arguments.enable_assertions,
                arguments.sanitizer,
                arguments.enable_valgrind,
                arguments.valgrind_options,
                arguments.clean,
            )?;
        }
        Arguments::Solc(SolcArguments::Build(arguments)) => {
            if let Some(ccache_variant) = arguments.ccache_variant {
                solx_dev::exists(ccache_variant.to_string().as_str())?;
            }

            solx_dev::solc_build(
                arguments.build_type,
                arguments.pedantic,
                arguments.tests,
                arguments.extra_args,
                arguments.clean,
                arguments.boost_version,
                arguments.enable_mlir,
                arguments.use_gcc,
                arguments.build_boost,
                arguments.ccache_variant,
            )?;
        }
        Arguments::Test(TestArguments::Hardhat(arguments)) => {
            let test_config_path = solx_dev::absolute_path(arguments.test_config_path)?;
            let test_config = solx_dev::HardhatTestConfig::try_from(test_config_path)?;

            let downloader_config_path = solx_dev::absolute_path(arguments.downloader_config_path)?;
            let downloader = solx_compiler_downloader::Downloader::default();
            downloader.download(downloader_config_path.as_path())?;

            solx_dev::test_hardhat(
                test_config,
                arguments.projects_dir,
                arguments.output_dir,
                arguments.solidity_version,
                arguments.project_filter,
            )?;
        }
        Arguments::Test(TestArguments::Foundry(arguments)) => {
            let test_config_path = solx_dev::absolute_path(arguments.test_config_path)?;
            let test_config = solx_dev::FoundryTestConfig::try_from(test_config_path)?;

            let downloader_config_path = solx_dev::absolute_path(arguments.downloader_config_path)?;
            let downloader = solx_compiler_downloader::Downloader::default();
            downloader.download(downloader_config_path.as_path())?;

            solx_dev::test_foundry(
                test_config,
                arguments.projects_dir,
                arguments.output_dir,
                arguments.solidity_version,
                arguments.project_filter,
            )?;
        }
        Arguments::Test(TestArguments::SolxTester(arguments)) => {
            solx_dev::solx_tester(arguments)?;
        }
    }

    Ok(())
}
