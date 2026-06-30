//!
//! Sol dialect data location for pointer types.
//!

use crate::AddressSpace;

/// Sol dialect data location for MLIR pointer types.
///
/// Mirrors `mlir::sol::DataLocation` from the LLVM Sol dialect.
/// Use [`From<AddressSpace>`] to convert from the EVM address space model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum DataLocation {
    /// Persistent storage (SLOAD/SSTORE).
    Storage = 0,
    /// Calldata (read-only input).
    CallData = 1,
    /// Heap memory.
    Memory = 2,
    /// Stack memory (local variables, allocas).
    Stack = 3,
    /// Immutable storage.
    Immutable = 4,
    /// Transient storage (TLOAD/TSTORE).
    Transient = 5,
}

impl From<AddressSpace> for DataLocation {
    fn from(space: AddressSpace) -> Self {
        match space {
            AddressSpace::Stack => Self::Stack,
            AddressSpace::Heap => Self::Memory,
            AddressSpace::Calldata => Self::CallData,
            AddressSpace::Storage => Self::Storage,
            AddressSpace::TransientStorage => Self::Transient,
            // TODO: map ReturnData and Code once the Sol dialect supports them.
            _ => unimplemented!("no DataLocation equivalent for {space:?}"),
        }
    }
}

impl TryFrom<u32> for DataLocation {
    type Error = u32;

    /// Recovers a data location from its dialect ordinal, the inverse of the
    /// `#[repr(u32)]` discriminant. An ordinal outside the six known locations is
    /// returned verbatim as the error.
    fn try_from(ordinal: u32) -> Result<Self, Self::Error> {
        match ordinal {
            0 => Ok(Self::Storage),
            1 => Ok(Self::CallData),
            2 => Ok(Self::Memory),
            3 => Ok(Self::Stack),
            4 => Ok(Self::Immutable),
            5 => Ok(Self::Transient),
            other => Err(other),
        }
    }
}

#[cfg(feature = "slang")]
impl DataLocation {
    /// Converts a Slang semantic data location into the dialect's data location.
    ///
    /// `inherited_fallback` is substituted when the Slang location is `Inherited`
    /// (struct-field-relative). Top-level callers pass `None`; recursive struct
    /// member resolution passes the parent struct's resolved location.
    ///
    /// # Panics
    ///
    /// Panics if `location` is `Inherited` and `inherited_fallback` is `None`,
    /// because semantic analysis guarantees `Inherited` only appears inside a
    /// struct member context where a fallback is available.
    pub fn from_slang(
        location: slang_solidity_v2::ast::DataLocation,
        inherited_fallback: Option<Self>,
    ) -> Self {
        use slang_solidity_v2::ast::DataLocation as Slang;
        match location {
            Slang::Storage => Self::Storage,
            Slang::Calldata => Self::CallData,
            Slang::Memory => Self::Memory,
            Slang::Inherited => inherited_fallback
                .expect("data location 'Inherited' encountered without a parent struct location"),
        }
    }
}
