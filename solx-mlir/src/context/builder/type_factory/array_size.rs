//!
//! Length parameter for `sol::ArrayType`.
//!

/// Length of a `sol::ArrayType`.
///
/// The dialect's TableGen definition encodes the length as `int64_t`, with
/// `-1` reserved as the sentinel for dynamic length. This enum keeps that
/// encoding inside `solx-mlir` so call sites name the kind they want instead
/// of passing a magic number.
pub enum ArraySize {
    /// Dynamic-length array (Solidity `T[]`).
    Dynamic,
    /// Fixed-length array of exactly `n` elements (Solidity `T[n]`).
    Fixed(u64),
}

impl ArraySize {
    /// Encodes the size into the dialect's wire format.
    pub fn as_dialect_i64(self) -> i64 {
        match self {
            Self::Dynamic => -1,
            Self::Fixed(n) => i64::try_from(n).expect("array length exceeds i64::MAX"),
        }
    }
}
