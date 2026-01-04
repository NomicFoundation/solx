//!
//! Defines available output formats.
//!

use crate::benchmark::Benchmark;

///
/// Native JSON format that corresponds to the inner benchmark analyzer data model.
///
#[derive(Default)]
pub struct Json {
    /// Serialized JSON.
    pub content: String,
}

impl From<Benchmark> for Json {
    fn from(benchmark: Benchmark) -> Self {
        let content = serde_json::to_string_pretty(&benchmark).expect("Always valid");
        Self { content }
    }
}
