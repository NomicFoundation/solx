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

impl From<ArraySize> for i64 {
    fn from(size: ArraySize) -> Self {
        match size {
            ArraySize::Dynamic => -1,
            ArraySize::Fixed(size) => Self::try_from(size).expect("array size fits in i64"),
        }
    }
}
