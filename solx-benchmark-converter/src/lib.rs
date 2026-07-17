//!
//! The benchmark analyzer library.
//!

#![allow(clippy::large_enum_variant)]
#![allow(clippy::let_and_return)]

pub(crate) mod benchmark;
pub(crate) mod input;
pub(crate) mod output;
pub(crate) mod utils;

pub use crate::benchmark::Benchmark;
pub use crate::benchmark::test::Test as BenchmarkTest;
pub use crate::benchmark::test::input::Input as BenchmarkTestInput;
pub use crate::benchmark::test::metadata::Metadata as BenchmarkTestMetadata;
pub use crate::benchmark::test::selector::Selector as BenchmarkTestSelector;
pub use crate::input::Input;
pub use crate::input::Report as InputReport;
pub use crate::input::build_failures::BuildFailuresReport;
pub use crate::input::compilation_time::CompilationTimeReport;
pub use crate::input::error::Error as InputReportError;
pub use crate::input::foundry_gas::FoundryGasReport;
pub use crate::input::foundry_size::FoundrySizeReport;
pub use crate::input::resolve_paths as resolve_input_paths;
pub use crate::input::test_failures::TestFailuresReport;
pub use crate::input::testing_time::TestingTimeReport;
pub use crate::output::Output;
pub use crate::output::comparison::Comparison as OutputComparison;
pub use crate::output::format::Format as OutputFormat;
pub use crate::output::json::Json as OutputJson;
pub use crate::output::summary::SuiteKind;
pub use crate::output::summary::SuiteOutcome;
pub use crate::output::summary::Summary;
pub use crate::output::summary::SummarySuite;
pub use crate::output::summary::ToolchainMatrix;
