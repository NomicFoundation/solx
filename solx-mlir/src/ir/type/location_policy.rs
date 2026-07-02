//!
//! Data-location policy for Slang-to-MLIR type resolution.
//!

use slang_solidity_v2::ast::DataLocation;

/// How Slang-to-MLIR type resolution picks the data location of each reference type.
#[derive(Clone, Copy)]
pub enum LocationPolicy {
    /// Use each reference type's declared location, substituting the carried location for `Inherited` members.
    Declared(Option<solx_utils::DataLocation>),
    /// Force every reference type to `Memory`: the external ABI representation,
    /// where `calldata` cannot cross the call boundary.
    ForceMemory,
}

impl LocationPolicy {
    /// The dialect data location for a reference type whose Slang location is
    /// `slang_location`.
    pub fn data_location(self, slang_location: DataLocation) -> solx_utils::DataLocation {
        match self {
            Self::Declared(inherited) => {
                solx_utils::DataLocation::from_slang(slang_location, inherited)
            }
            Self::ForceMemory => solx_utils::DataLocation::Memory,
        }
    }

    /// The policy for a struct's members, given the struct's own resolved `location`.
    pub fn within_struct(self, location: solx_utils::DataLocation) -> Self {
        match self {
            Self::Declared(_) => Self::Declared(Some(location)),
            Self::ForceMemory => Self::ForceMemory,
        }
    }
}
