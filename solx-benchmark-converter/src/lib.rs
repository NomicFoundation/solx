//!
//! The benchmark analyzer library.
//!

#![allow(clippy::large_enum_variant)]

pub(crate) mod benchmark;
pub(crate) mod comparison;
pub(crate) mod input;
pub(crate) mod output;
pub(crate) mod role;
pub(crate) mod suite_kind;
pub(crate) mod suite_outcome;
pub(crate) mod summary_suite;
pub(crate) mod toolchain_matrix;
pub(crate) mod utils;

pub use crate::benchmark::Benchmark;
pub use crate::benchmark::test::Test as BenchmarkTest;
pub use crate::benchmark::test::input::Input as BenchmarkTestInput;
pub use crate::benchmark::test::metadata::Metadata as BenchmarkTestMetadata;
pub use crate::benchmark::test::selector::Selector as BenchmarkTestSelector;
pub use crate::comparison::Comparison as OutputComparison;
pub use crate::input::Input;
pub use crate::input::build_failures::BuildFailuresReport;
pub use crate::input::compilation_time::CompilationTimeReport;
pub use crate::input::error::Error as InputReportError;
pub use crate::input::foundry_gas::FoundryGasReport;
pub use crate::input::foundry_size::FoundrySizeReport;
pub use crate::input::report::Report as InputReport;
pub use crate::input::test_failures::TestFailuresReport;
pub use crate::input::testing_time::TestingTimeReport;
pub use crate::output::Output;
pub use crate::output::format::Format as OutputFormat;
pub use crate::output::json::Json as OutputJson;
pub use crate::output::summary::Summary;
pub use crate::suite_kind::SuiteKind;
pub use crate::suite_outcome::SuiteOutcome;
pub use crate::summary_suite::SummarySuite;
pub use crate::toolchain_matrix::ToolchainMatrix;
