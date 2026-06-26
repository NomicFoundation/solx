//!
//! Arithmetic overflow-checking mode for Sol dialect binary operations.
//!

/// Whether an arithmetic operation uses Solidity's checked or `unchecked { }` semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticMode {
    /// Checked arithmetic: the default in Solidity 0.8+. Reverts on overflow.
    Checked,
    /// Unchecked arithmetic: inside `unchecked { }` blocks and for-loop step
    /// expressions. Wraps on overflow.
    Unchecked,
}
