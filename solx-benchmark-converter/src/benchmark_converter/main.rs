//!
//! The benchmark analyzer binary.
//!

pub mod arguments;

use clap::Parser;

use solx_benchmark_converter::Benchmark;
use solx_benchmark_converter::Input;
use solx_benchmark_converter::InputReportError;
use solx_benchmark_converter::Output;
use solx_benchmark_converter::ToolchainMatrix;

use self::arguments::Arguments;

///
/// The application entry point.
///
fn main() -> anyhow::Result<()> {
    let arguments = Arguments::try_parse()?;

    let input_paths = Input::resolve_paths(arguments.input_paths)?;
    let mut inputs = Vec::with_capacity(input_paths.len());
    for path in input_paths.into_iter() {
        match Input::try_from(path.as_path()) {
            Ok(input) => inputs.push(input),
            Err(InputReportError::EmptyFile { path }) => {
                if !arguments.quiet {
                    eprintln!("Warning: Input file {path:?} is empty and will be skipped.");
                }
                continue;
            }
            Err(error) => Err(error)?,
        }
    }
    let benchmark = Benchmark::from_inputs(inputs)?;

    let comparisons = ToolchainMatrix::Tester.comparisons(&benchmark.toolchains());
    let output: Output = (benchmark, comparisons, arguments.output_format).try_into()?;
    output.write_to_file(arguments.output_path)?;

    Ok(())
}
