//!
//! Length parameter for `sol::ArrayType`.
//!

/// Length of a `sol::ArrayType`: the dialect encodes this as `int64_t` with `-1` for dynamic length.
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
            Self::Fixed(n) => i64::try_from(n).expect("array size fits in i64"),
        }
    }
}
