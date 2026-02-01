//!
//! The benchmark analyzer library.
//!

#![allow(clippy::large_enum_variant)]
#![allow(clippy::let_and_return)]

pub mod benchmark;
pub mod input;
pub mod output;

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
pub use crate::input::foundry_size::contract::ContractReport as FoundrySizeContractReport;
pub use crate::input::source::Source as InputSource;
pub use crate::input::test_failures::TestFailuresReport;
pub use crate::input::testing_time::TestingTimeReport;
pub use crate::output::Output;
pub use crate::output::format::Format as OutputFormat;
