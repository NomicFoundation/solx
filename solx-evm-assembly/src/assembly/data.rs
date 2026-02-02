//!
//! The inner JSON legacy assembly code element.
//!

use std::collections::BTreeSet;

use crate::assembly::Assembly;

///
/// The inner JSON legacy assembly code element.
///
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum Data {
    /// The assembly code wrapper.
    Assembly(Assembly),
    /// The hash.
    Hash(String),
    /// The full contract path after the factory dependencies replacing pass.
    Path(String),
}

impl Data {
    ///
    /// Returns the inner assembly reference if it is present.
    ///
    pub fn get_assembly(&self) -> Option<&Assembly> {
        match self {
            Self::Assembly(assembly) => Some(assembly),
            Self::Hash(_) => None,
            Self::Path(_) => None,
        }
    }
    ///
    /// Returns the inner assembly mutable reference if it is present.
    ///
    pub fn get_assembly_mut(&mut self) -> Option<&mut Assembly> {
        match self {
            Self::Assembly(assembly) => Some(assembly),
            Self::Hash(_) => None,
            Self::Path(_) => None,
        }
    }

    ///
    /// Get the list of unlinked deployable libraries.
    ///
    pub fn get_unlinked_libraries(&self) -> BTreeSet<String> {
        match self {
            Self::Assembly(assembly) => assembly.get_unlinked_libraries(),
            Self::Hash(_) => BTreeSet::new(),
            Self::Path(_) => BTreeSet::new(),
        }
    }
}

impl std::fmt::Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assembly(inner) => writeln!(f, "{inner}"),
            Self::Hash(inner) => writeln!(f, "Hash `{inner}`"),
            Self::Path(inner) => writeln!(f, "Path `{inner}`"),
        }
    }
}
