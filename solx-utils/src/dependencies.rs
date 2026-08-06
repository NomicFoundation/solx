//!
//! Collection of dependencies.
//!

use std::iter::Chain;
use std::option::Iter as OptionIter;
use std::slice::Iter as SliceIter;

///
/// The objects a code segment references. The assembler reads the first object yielded by
/// iteration as the segment's runtime object.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Dependencies {
    /// Top-level object identifier.
    pub identifier: String,
    /// The runtime object the deploy code returns. `None` in a runtime segment.
    pub runtime: Option<String>,
    /// List of EVM dependencies in the order they are encountered in IR.
    pub inner: Vec<String>,
}

impl Dependencies {
    /// The deployed object identifier suffix used by the Yul AST and the Sol-to-LLVM pass output.
    pub const DEPLOYED_OBJECT_SUFFIX: &'static str = "_deployed";

    ///
    /// Create a new instance of dependencies.
    ///
    pub fn new(identifier: &str, runtime: Option<String>) -> Self {
        Self {
            identifier: identifier.to_owned(),
            runtime,
            inner: Vec::new(),
        }
    }

    ///
    /// Push a single dependency.
    ///
    pub fn push(&mut self, dependency: String) {
        if dependency == self.identifier
            || Some(&dependency) == self.runtime.as_ref()
            || self.inner.contains(&dependency)
        {
            return;
        }

        self.inner.push(dependency);
    }
}

impl<'a> IntoIterator for &'a Dependencies {
    type Item = &'a String;
    type IntoIter = Chain<OptionIter<'a, String>, SliceIter<'a, String>>;

    fn into_iter(self) -> Self::IntoIter {
        self.runtime.iter().chain(self.inner.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::Dependencies;

    /// The assembler reads the first dependency as the runtime object, so a contract the
    /// constructor creates must not displace the runtime child when it is encountered first.
    #[test]
    fn the_runtime_child_leads_whatever_the_encounter_order() {
        let mut encountered_first = Dependencies::new("C", Some("C_deployed".to_owned()));
        encountered_first.push("C_deployed".to_owned());
        encountered_first.push("A".to_owned());

        let mut encountered_last = Dependencies::new("C", Some("C_deployed".to_owned()));
        encountered_last.push("A".to_owned());
        encountered_last.push("C_deployed".to_owned());

        assert_eq!(Vec::from_iter(&encountered_first), ["C_deployed", "A"]);
        assert_eq!(Vec::from_iter(&encountered_last), ["C_deployed", "A"]);
    }

    /// An object never depends on itself, and a repeated reference adds nothing.
    #[test]
    fn the_object_itself_and_repeats_are_dropped() {
        let mut dependencies = Dependencies::new("C", None);
        dependencies.push("C".to_owned());
        dependencies.push("A".to_owned());
        dependencies.push("A".to_owned());

        assert_eq!(Vec::from_iter(&dependencies), ["A"]);
    }
}
