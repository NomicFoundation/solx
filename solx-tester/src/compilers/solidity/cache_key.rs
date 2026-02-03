//!
//! The Solidity compiler cache key.
//!

///
/// The Solidity compiler cache key.
///
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    /// The test path.
    pub test_path: String,
    /// The Solidity compiler version.
    pub version: semver::Version,
    /// Whether to enable the Yul IR path.
    pub via_ir: bool,
    /// Whether to run the Solidity compiler optimizer.
    pub optimize: bool,
}

impl CacheKey {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(test_path: String, version: semver::Version, via_ir: bool, optimize: bool) -> Self {
        Self {
            test_path,
            version,
            via_ir,
            optimize,
        }
    }
}
