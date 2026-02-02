//!
//! Excel report comparison configuration.
//!

///
/// A comparison between two compilers for Excel diff columns.
///
#[derive(Debug, Clone)]
pub struct Comparison {
    /// The left compiler name (e.g., "03.solx-legacy").
    pub left: String,
    /// The right compiler name (e.g., "00.solc-0.8.33-legacy").
    pub right: String,
}

impl Comparison {
    ///
    /// Creates a new comparison.
    ///
    pub fn new(left: String, right: String) -> Self {
        Self { left, right }
    }
}
