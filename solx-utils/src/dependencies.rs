//!
//! Collection of dependencies.
//!

use std::iter::Chain;
use std::option::Iter as OptionIter;
use std::slice::Iter as SliceIter;

///
/// The objects a code segment may embed. The assembler reads the first object yielded by
/// iteration as the segment's runtime object.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Dependencies {
    /// Top-level object identifier.
    pub identifier: String,
    /// The runtime object the deploy code returns. `None` in a runtime segment.
    pub runtime: Option<String>,
    /// The objects of every dependency.
    pub objects: Vec<String>,
}

impl Dependencies {
    /// The deployed object identifier suffix the Sol-to-LLVM pass output names runtime objects with.
    const DEPLOYED_OBJECT_SUFFIX: &'static str = "_deployed";

    ///
    /// Create a new instance of dependencies. Each dependency brings both its objects, since the
    /// code may name either and the assembler links only those it names.
    ///
    pub fn new(
        identifier: &str,
        runtime: Option<String>,
        dependencies: impl IntoIterator<Item = String>,
    ) -> Self {
        let objects = dependencies
            .into_iter()
            .flat_map(|dependency| {
                let runtime = Self::runtime_identifier(dependency.as_str());
                [dependency, runtime]
            })
            .collect();

        Self {
            identifier: identifier.to_owned(),
            runtime,
            objects,
        }
    }

    ///
    /// The identifier of the runtime object the deploy code of `identifier` returns.
    ///
    pub fn runtime_identifier(identifier: &str) -> String {
        format!("{identifier}{}", Self::DEPLOYED_OBJECT_SUFFIX)
    }
}

impl<'a> IntoIterator for &'a Dependencies {
    type Item = &'a String;
    type IntoIter = Chain<OptionIter<'a, String>, SliceIter<'a, String>>;

    fn into_iter(self) -> Self::IntoIter {
        self.runtime.iter().chain(self.objects.iter())
    }
}
