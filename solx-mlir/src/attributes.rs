//!
//! Sol dialect attribute enums for MLIR code generation.
//!

/// Sol dialect state mutability.
///
/// Maps to the `StateMutabilityAttr` values in the C++ Sol dialect.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateMutability {
    /// Pure — no reads or writes.
    Pure = 0,
    /// View — reads state, no writes.
    View = 1,
    /// NonPayable — reads/writes state, no ether.
    NonPayable = 2,
    /// Payable — can receive ether.
    Payable = 3,
}

/// Sol dialect contract kind.
///
/// Maps to the `ContractKindAttr` values in the C++ Sol dialect.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractKind {
    /// Interface contract.
    Interface = 0,
    /// Regular contract.
    Contract = 1,
    /// Library contract.
    Library = 2,
}

/// Sol dialect EVM version.
///
/// Maps to the `EvmVersionAttr` values in the C++ Sol dialect.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvmVersion {
    /// Homestead release.
    Homestead = 0,
    /// Tangerine Whistle release.
    TangerineWhistle = 1,
    /// Spurious Dragon release.
    SpuriousDragon = 2,
    /// Byzantium release.
    Byzantium = 3,
    /// Constantinople release.
    Constantinople = 4,
    /// Petersburg release.
    Petersburg = 5,
    /// Istanbul release.
    Istanbul = 6,
    /// Berlin release.
    Berlin = 7,
    /// London release.
    London = 8,
    /// Paris / The Merge release.
    Paris = 9,
    /// Shanghai release.
    Shanghai = 10,
    /// Cancun release.
    Cancun = 11,
}

/// MLIR LLVM dialect `llvm.icmp` predicate values.
///
/// Matches the LLVM `ICmpPredicate` encoding used by the MLIR LLVM dialect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i64)]
pub enum ICmpPredicate {
    /// Equal.
    Eq = 0,
    /// Not equal.
    Ne = 1,
    /// Signed less than.
    Slt = 2,
    /// Signed less than or equal.
    Sle = 3,
    /// Signed greater than.
    Sgt = 4,
    /// Signed greater than or equal.
    Sge = 5,
    /// Unsigned less than.
    Ult = 6,
    /// Unsigned less than or equal.
    Ule = 7,
    /// Unsigned greater than.
    Ugt = 8,
    /// Unsigned greater than or equal.
    Uge = 9,
}
