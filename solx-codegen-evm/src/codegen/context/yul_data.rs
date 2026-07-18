//!
//! The LLVM IR generator Yul data.
//!

use std::collections::BTreeMap;

use crate::context::traits::yul_data::IYulData;

///
/// The LLVM IR generator Yul data.
///
/// Describes some data that is only relevant to Yul.
///
#[derive(Debug, Default)]
pub struct YulData {
    /// Mapping from Yul object identifiers to full contract paths.
    identifier_paths: BTreeMap<String, String>,
}

impl YulData {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(identifier_paths: BTreeMap<String, String>) -> Self {
        Self { identifier_paths }
    }
}

impl IYulData for YulData {
    fn resolve_path(&self, identifier: &str) -> Option<&str> {
        self.identifier_paths
            .get(identifier)
            .map(|path| path.as_str())
    }
}
