//!
//! Converts `[InputIdentifier]` to the representation used by the benchmark.
//!

use crate::test::case::input::identifier::InputIdentifier;

impl From<InputIdentifier> for solx_benchmark_converter::BenchmarkTestInput {
    ///
    /// Converts `[InputIdentifier]` to the representation used by the benchmark.
    ///
    fn from(val: InputIdentifier) -> Self {
        match val {
            InputIdentifier::Deployer {
                contract_identifier,
            } => solx_benchmark_converter::BenchmarkTestInput::Deployer {
                contract_identifier,
            },
            InputIdentifier::Runtime { input_index, name } => {
                solx_benchmark_converter::BenchmarkTestInput::Runtime { input_index, name }
            }
            InputIdentifier::StorageEmpty { input_index } => {
                solx_benchmark_converter::BenchmarkTestInput::StorageEmpty { input_index }
            }
            InputIdentifier::Balance { input_index } => {
                solx_benchmark_converter::BenchmarkTestInput::Balance { input_index }
            }
            InputIdentifier::Fallback { input_index } => {
                solx_benchmark_converter::BenchmarkTestInput::Fallback { input_index }
            }
        }
    }
}
