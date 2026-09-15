//!
//! The solx compiler test metadata expected event.
//!

use serde::Deserialize;

///
/// The solx compiler test metadata expected event.
///
#[derive(Debug, Clone, Deserialize)]
pub struct Event {
    /// The emitter contract address.
    pub address: Option<String>,
    /// The indexed topics.
    pub topics: Vec<String>,
    /// The ordinary values.
    pub values: Vec<String>,
}
