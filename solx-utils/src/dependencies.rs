//!
//! Collection of dependencies.
//!

///
/// The objects a code segment references, the runtime child first and the rest in the order they
/// are encountered in IR. The assembler reads index 0 as the segment's runtime object.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Dependencies {
    /// Top-level object identifier.
    pub identifier: String,
    /// List of EVM dependencies.
    pub inner: Vec<String>,
}

impl Dependencies {
    /// The deployed object identifier suffix used by the Yul AST and the Sol-to-LLVM pass output.
    pub const DEPLOYED_OBJECT_SUFFIX: &'static str = "_deployed";

    /// The runtime object identifier suffix used by EVM legacy assembly and LLVM IR.
    pub const RUNTIME_SEGMENT_SUFFIX: &'static str = ".runtime";

    ///
    /// Create a new instance of dependencies.
    ///
    pub fn new(identifier: &str) -> Self {
        Self {
            identifier: identifier.to_owned(),
            inner: Vec::new(),
        }
    }

    ///
    /// Push a single dependency, the runtime child taking the lead.
    ///
    pub fn push(&mut self, dependency: String) {
        if dependency == self.identifier || self.inner.contains(&dependency) {
            return;
        }

        if self.is_runtime_child(dependency.as_str()) {
            self.inner.insert(0, dependency);
        } else {
            self.inner.push(dependency);
        }
    }

    ///
    /// Whether `dependency` is this object's own runtime segment, named by either the Yul and
    /// Sol-to-LLVM suffix or the EVM legacy assembly one.
    ///
    fn is_runtime_child(&self, dependency: &str) -> bool {
        dependency
            .strip_suffix(Self::DEPLOYED_OBJECT_SUFFIX)
            .or_else(|| dependency.strip_suffix(Self::RUNTIME_SEGMENT_SUFFIX))
            == Some(self.identifier.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::Dependencies;

    /// The assembler reads index 0 as the runtime object, so a contract the constructor creates
    /// must not displace the runtime child when it is encountered first.
    #[test]
    fn runtime_child_leads_whatever_the_encounter_order() {
        let mut encountered_first = Dependencies::new("C");
        encountered_first.push("C_deployed".to_owned());
        encountered_first.push("A".to_owned());

        let mut encountered_last = Dependencies::new("C");
        encountered_last.push("A".to_owned());
        encountered_last.push("C_deployed".to_owned());

        assert_eq!(encountered_first.inner, ["C_deployed", "A"]);
        assert_eq!(encountered_last.inner, ["C_deployed", "A"]);
    }

    /// EVM legacy assembly qualifies its runtime segment with `.runtime` rather than `_deployed`.
    #[test]
    fn runtime_child_leads_under_the_legacy_assembly_suffix() {
        let mut dependencies = Dependencies::new("C.sol:C");
        dependencies.push("C.sol:A".to_owned());
        dependencies.push("C.sol:C.runtime".to_owned());

        assert_eq!(dependencies.inner, ["C.sol:C.runtime", "C.sol:A"]);
    }

    /// A contract whose name ends in the suffix is not thereby its referrer's runtime child.
    #[test]
    fn a_foreign_object_named_like_a_runtime_child_does_not_lead() {
        let mut dependencies = Dependencies::new("C");
        dependencies.push("A".to_owned());
        dependencies.push("D_deployed".to_owned());

        assert_eq!(dependencies.inner, ["A", "D_deployed"]);
    }

    /// An object never depends on itself, and a repeated reference adds nothing.
    #[test]
    fn the_object_itself_and_repeats_are_dropped() {
        let mut dependencies = Dependencies::new("C");
        dependencies.push("C".to_owned());
        dependencies.push("A".to_owned());
        dependencies.push("A".to_owned());

        assert_eq!(dependencies.inner, ["A"]);
    }
}
