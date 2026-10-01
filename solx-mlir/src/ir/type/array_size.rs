//!
//! Length parameter for `sol::ArrayType`.
//!

use ruint::aliases::U256;

/// Length of a `sol::ArrayType`. A storage array may be longer than 2^64 - 1 elements.
pub enum ArraySize {
    /// Dynamic-length array (Solidity `T[]`).
    Dynamic,
    /// Fixed-length array of exactly `n` elements (Solidity `T[n]`).
    Fixed(U256),
}
