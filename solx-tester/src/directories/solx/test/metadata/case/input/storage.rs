//!
//! The solx compiler test metadata case input contract storage.
//!

use std::collections::HashMap;

///
/// The solx compiler test metadata case input contract storage.
///
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged)]
pub enum Storage {
    /// The list, where the key starts from 0.
    List(Vec<String>),
    /// The map with actual explicit keys.
    Map(HashMap<String, String>),
}

impl Default for Storage {
    fn default() -> Self {
        Self::Map(HashMap::new())
    }
}
