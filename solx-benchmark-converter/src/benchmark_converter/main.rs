//!
//! The benchmark analyzer binary.
//!

pub(crate) mod arguments;

use clap::Parser;

use self::arguments::Arguments;

///
/// The application entry point.
///
fn main() -> anyhow::Result<()> {
    let arguments = Arguments::try_parse()?;

    let mut benchmark = solx_benchmark_converter::Benchmark::default();
    let input_paths = if arguments.input_paths.len() == 1 {
        if !arguments.input_paths[0].is_dir() {
            anyhow::bail!(
                "Expected a directory with JSON files, but got a file: {:?}",
                arguments.input_paths[0]
            );
        }
        let resolution_pattern =
            format!("{}/**/*.json", arguments.input_paths[0].to_string_lossy());
        glob::glob(resolution_pattern.as_str())?
            .filter_map(Result::ok)
            .collect()
    } else if arguments.input_paths.is_empty() {
        anyhow::bail!("No input files provided. Use `--input-paths` to specify input files.");
    } else {
        arguments.input_paths
    };
    for path in input_paths.into_iter() {
        match solx_benchmark_converter::Input::try_from(path.as_path()) {
            Ok(input) => benchmark.extend(input)?,
            Err(solx_benchmark_converter::InputReportError::EmptyFile { path }) => {
                if !arguments.quiet {
                    eprintln!("Warning: Input file {path:?} is empty and will be skipped.");
                }
                continue;
            }
            Err(error) => Err(error)?,
        }
    }
    benchmark.remove_zero_deploy_gas();

    let output: solx_benchmark_converter::Output =
        (benchmark, arguments.input_source, arguments.output_format).try_into()?;
    output.write_to_file(arguments.output_path)?;

    Ok(())
}
