//!
//! Information associated with a specific test in benchmark.
//!

use serde::Deserialize;
use serde::Serialize;

use crate::benchmark::test::selector::Selector;

///
/// Information associated with a specific test in benchmark.
///
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(default)]
    /// Tests may be tagged with one or many groups.
    pub tags: Vec<String>,
    /// Test selector.
    pub selector: Selector,
}

impl Metadata {
    ///
    /// Creates a new instance of test metadata provided with the test selector and tags.
    ///
    pub fn new(selector: Selector, tags: Vec<String>) -> Self {
        Self { selector, tags }
    }
}
