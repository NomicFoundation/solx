//!
//! The benchmark test representation.
//!

pub mod input;
pub mod metadata;
pub mod run;
pub mod selector;

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use self::metadata::Metadata;
use self::run::Run;

///
/// The benchmark test representation.
///
/// Each test can have multiple runs with different compiler modes.
///
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Test {
    /// Metadata for this test.
    #[serde(default)]
    pub metadata: Metadata,
    /// Runs keyed by mode string (e.g., "solx-Y+M3B3-0.8.28" or "solc").
    #[serde(default)]
    pub runs: BTreeMap<String, Run>,

    /// The number of non-zero gas values across all runs.
    #[serde(skip)]
    pub non_zero_gas_values: usize,
}

impl Test {
    ///
    /// Creates a new test with provided metadata.
    ///
    pub fn new(metadata: Metadata) -> Self {
        Self {
            metadata,
            runs: Default::default(),
            non_zero_gas_values: 0,
        }
    }

    ///
    /// Whether the test is for a deploy transaction.
    ///
    pub fn is_deploy(&self) -> bool {
        self.metadata
            .selector
            .input
            .as_ref()
            .map(|input| input.is_deploy())
            .unwrap_or_default()
    }
}
