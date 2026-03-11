//!
//! EVM address space identifiers.
//!

/// EVM target address space identifiers.
///
/// Each variant maps to a distinct LLVM address space used by the EVM target
/// backend. Used by `solx-mlir` for MLIR pointer types and by
/// `solx-codegen-evm` for LLVM IR pointer types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum AddressSpace {
    /// Stack memory (local variables, allocas).
    Stack = 0,
    /// Heap memory (dynamic allocations).
    Heap = 1,
    /// Calldata (read-only input).
    Calldata = 2,
    /// Return data.
    ReturnData = 3,
    /// Code memory.
    Code = 4,
    /// Persistent storage (SLOAD/SSTORE).
    Storage = 5,
    /// Transient storage (TLOAD/TSTORE).
    TransientStorage = 6,
}
