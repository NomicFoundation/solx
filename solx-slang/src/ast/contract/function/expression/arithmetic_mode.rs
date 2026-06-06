//!
//! Arithmetic overflow-checking mode for Sol dialect binary operations.
//!

/// Whether an arithmetic operation uses Solidity's checked (overflow-reverting)
/// semantics or the `unchecked { }` wrapping semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticMode {
    /// Checked arithmetic (`sol.cadd`, `sol.csub`, …) — the default in
    /// Solidity 0.8+. Reverts on overflow.
    Checked,
    /// Unchecked arithmetic (`sol.add`, `sol.sub`, …) — inside `unchecked { }`
    /// blocks and for-loop step expressions. Wraps on overflow.
    Unchecked,
}
