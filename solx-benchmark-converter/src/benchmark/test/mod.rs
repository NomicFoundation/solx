//!
//! The benchmark group representation.
//!

pub mod input;
pub mod metadata;
pub mod selector;
pub mod toolchain;

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use self::metadata::Metadata;
use self::toolchain::ToolchainGroup;

///
/// The codegen associated with a test definition.
///
pub type Toolchain = String;

///
/// The benchmark group representation.
///
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Test {
    /// Metadata for this test.
    #[serde(default)]
    pub metadata: Metadata,
    /// Toolchain groups.
    pub toolchain_groups: BTreeMap<Toolchain, ToolchainGroup>,

    /// The number of non-zero gas values across all toolchains.
    #[serde(skip)]
    pub non_zero_gas_values: usize,
}

impl Test {
    ///
    /// Creates a new test with provided metadata.
    ///
    pub fn new(metadata: Metadata) -> Self {
        Self {
            toolchain_groups: Default::default(),
            metadata,

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
