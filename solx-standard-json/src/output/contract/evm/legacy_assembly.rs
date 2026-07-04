//!
//! The `solc --standard-json` output contract EVM legacy assembly.
//!

///
/// The `solc --standard-json` output contract EVM legacy assembly.
///
/// `solc` emits a deeply nested legacy assembly tree per contract, and deserializing every tree on
/// the single thread that parses the whole `solc` output dominates the serial head of the pipeline.
/// The tree is therefore captured verbatim as [`LegacyAssembly::Raw`] while the output is parsed and
/// materialized into [`LegacyAssembly::Parsed`] in parallel before it is consumed.
///
#[derive(Debug, Clone)]
pub enum LegacyAssembly {
    /// The verbatim JSON captured while the `solc` output is deserialized.
    Raw(Box<serde_json::value::RawValue>),
    /// The parsed assembly, ready for dependency preprocessing, lowering, and JSON output.
    Parsed(solx_evm_assembly::Assembly),
}

impl LegacyAssembly {
    ///
    /// Parses the [`LegacyAssembly::Raw`] variant into [`LegacyAssembly::Parsed`] in place.
    ///
    pub fn materialize(&mut self) -> anyhow::Result<()> {
        if let Self::Raw(raw) = self {
            *self = Self::Parsed(solx_utils::deserialize_from_slice::<
                solx_evm_assembly::Assembly,
            >(raw.get().as_bytes())?);
        }
        Ok(())
    }

    ///
    /// Returns a mutable reference to the parsed assembly.
    ///
    pub fn parsed_mut(&mut self) -> &mut solx_evm_assembly::Assembly {
        match self {
            Self::Parsed(assembly) => assembly,
            Self::Raw(_) => panic!("Legacy assembly is accessed before materialization"),
        }
    }

    ///
    /// Unwraps the parsed assembly.
    ///
    pub fn into_parsed(self) -> solx_evm_assembly::Assembly {
        match self {
            Self::Parsed(assembly) => assembly,
            Self::Raw(_) => panic!("Legacy assembly is accessed before materialization"),
        }
    }
}

impl From<solx_evm_assembly::Assembly> for LegacyAssembly {
    fn from(assembly: solx_evm_assembly::Assembly) -> Self {
        Self::Parsed(assembly)
    }
}

impl<'de> serde::Deserialize<'de> for LegacyAssembly {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Box::<serde_json::value::RawValue>::deserialize(deserializer).map(Self::Raw)
    }
}

impl serde::Serialize for LegacyAssembly {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Raw(raw) => raw.serialize(serializer),
            Self::Parsed(assembly) => assembly.serialize(serializer),
        }
    }
}
