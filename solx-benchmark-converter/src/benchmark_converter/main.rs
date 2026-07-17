//!
//! The benchmark analyzer binary.
//!

pub mod arguments;

use clap::Parser;

use self::arguments::Arguments;

///
/// The application entry point.
///
fn main() -> anyhow::Result<()> {
    let arguments = Arguments::try_parse()?;

    let input_paths = solx_benchmark_converter::Input::resolve_paths(arguments.input_paths)?;
    let mut inputs = Vec::with_capacity(input_paths.len());
    for path in input_paths.into_iter() {
        match solx_benchmark_converter::Input::try_from(path.as_path()) {
            Ok(input) => inputs.push(input),
            Err(solx_benchmark_converter::InputReportError::EmptyFile { path }) => {
                if !arguments.quiet {
                    eprintln!("Warning: Input file {path:?} is empty and will be skipped.");
                }
                continue;
            }
            Err(error) => Err(error)?,
        }
    }
    let benchmark = solx_benchmark_converter::Benchmark::from_inputs(inputs)?;

    let comparisons = Vec::new();
    let output: solx_benchmark_converter::Output =
        (benchmark, comparisons, arguments.output_format).try_into()?;
    output.write_to_file(arguments.output_path)?;

    Ok(())
}
