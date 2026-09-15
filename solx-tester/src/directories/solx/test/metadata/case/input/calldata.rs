//!
//! The solx compiler test metadata case input calldata.
//!

use serde::Deserialize;

///
/// The solx compiler test metadata case input calldata.
///
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Calldata {
    /// The single value.
    Value(String),
    /// The list of values.
    List(Vec<String>),
}

impl Default for Calldata {
    fn default() -> Self {
        Self::List(vec![])
    }
}
