//!
//! A run of a test with fixed compiler options (mode).
//!

use serde::Deserialize;
use serde::Serialize;

///
/// Run of a test with specific compiler options.
///
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Run {
    /// Contract deploy code size.
    #[serde(default)]
    pub size: Vec<u64>,
    /// Contract runtime code size.
    #[serde(default)]
    pub runtime_size: Vec<u64>,
    /// Amount of EVM gas.
    #[serde(default)]
    pub gas: Vec<u64>,
    /// Compilation time in milliseconds.
    #[serde(default)]
    pub compilation_time: Vec<u64>,
    /// Testing time in milliseconds.
    #[serde(default)]
    pub testing_time: Vec<u64>,
}

impl Run {
    ///
    /// Extends the run with another run, averaging the values.
    ///
    pub fn extend(&mut self, other: &Self) {
        self.size.extend_from_slice(other.size.as_slice());
        self.runtime_size
            .extend_from_slice(other.runtime_size.as_slice());
        self.gas
            .extend(other.gas.iter().filter(|value| value < &&(u32::MAX as u64)));
        self.compilation_time
            .extend_from_slice(other.compilation_time.as_slice());
        self.testing_time
            .extend_from_slice(other.testing_time.as_slice());
    }

    ///
    /// Average contract size.
    ///
    pub fn average_size(&self) -> u64 {
        if self.size.is_empty() {
            return 0;
        }

        self.size.iter().sum::<u64>() / (self.size.len() as u64)
    }

    ///
    /// Average runtime code size.
    ///
    pub fn average_runtime_size(&self) -> u64 {
        if self.runtime_size.is_empty() {
            return 0;
        }

        self.runtime_size.iter().sum::<u64>() / (self.runtime_size.len() as u64)
    }

    ///
    /// Average amount of EVM gas.
    ///
    pub fn average_gas(&self) -> u64 {
        if self.gas.is_empty() {
            return 0;
        }

        self.gas.iter().sum::<u64>() / (self.gas.len() as u64)
    }

    ///
    /// Average compilation time in milliseconds.
    ///
    pub fn average_compilation_time(&self) -> u64 {
        if self.compilation_time.is_empty() {
            return 0;
        }

        self.compilation_time.iter().sum::<u64>() / (self.compilation_time.len() as u64)
    }

    ///
    /// Average testing time in milliseconds.
    ///
    pub fn average_testing_time(&self) -> u64 {
        if self.testing_time.is_empty() {
            return 0;
        }

        self.testing_time.iter().sum::<u64>() / (self.testing_time.len() as u64)
    }
}
