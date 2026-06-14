//!
//! Data-location policy for slang→MLIR type resolution.
//!

use solx_utils::DataLocation;

/// How [`TypeConversion::resolve_slang_type`] picks the data location of each
/// reference type it resolves.
///
/// [`TypeConversion::resolve_slang_type`]: super::TypeConversion::resolve_slang_type
#[derive(Clone, Copy)]
pub enum LocationPolicy {
    /// Use each reference type's declared data location, substituting the carried
    /// location wherever slang reports `Inherited` (struct-field-relative). Top
    /// level passes `None`; struct-member resolution carries the parent struct's
    /// resolved location.
    Declared(Option<DataLocation>),
    /// Force every reference type to `Memory` — the external (ABI) representation,
    /// where `calldata` cannot cross the call boundary.
    ForceMemory,
}

impl LocationPolicy {
    /// The dialect data location for a reference type whose slang location is
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
