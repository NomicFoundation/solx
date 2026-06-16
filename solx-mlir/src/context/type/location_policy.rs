//!
//! Data-location policy for Slang→MLIR type resolution.
//!

use solx_utils::DataLocation;

/// How Slang→MLIR type resolution (the `ResolveType` projection) picks the data
/// location of each reference type it resolves.
#[derive(Clone, Copy)]
pub enum LocationPolicy {
    /// Use each reference type's declared data location, substituting the carried
    /// location wherever Slang reports `Inherited` (struct-field-relative). Top
    /// level passes `None`; struct-member resolution carries the parent struct's
    /// resolved location.
    Declared(Option<DataLocation>),
    /// Force every reference type to `Memory` — the external (ABI) representation,
    /// where `calldata` cannot cross the call boundary.
    ForceMemory,
}

impl LocationPolicy {
    /// The dialect data location for a reference type whose Slang location is
    /// `slang_location`.
    pub fn data_location(
        self,
        slang_location: slang_solidity_v2::ast::DataLocation,
    ) -> DataLocation {
        match self {
            Self::Declared(inherited) => DataLocation::from_slang(slang_location, inherited),
            Self::ForceMemory => DataLocation::Memory,
        }
    }

    /// The policy for a struct's members given the struct's own resolved
    /// `location`: declared resolution inherits it for `Inherited` members;
    /// forced-memory stays forced.
    pub fn within_struct(self, location: DataLocation) -> Self {
        match self {
            Self::Declared(_) => Self::Declared(Some(location)),
            Self::ForceMemory => Self::ForceMemory,
        }
    }
}
