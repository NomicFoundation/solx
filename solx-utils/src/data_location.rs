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
