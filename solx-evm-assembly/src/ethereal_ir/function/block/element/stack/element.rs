//!
//! The Ethereal IR block element stack element.
//!

///
/// The Ethereal IR block element stack element.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Element {
    /// The runtime value.
    Value(String),
    /// The compile-time value.
    Constant(num::BigUint),
    /// The compile-time destination tag.
    Tag(u64),
    /// The compile-time path.
    Path(String),
    /// The compile-time hexadecimal data chunk.
    Data(String),
    /// The defined function return address.
    ReturnAddress(usize),
}

impl Element {
    ///
    /// A shortcut constructor.
    ///
    pub fn value(identifier: String) -> Self {
        Self::Value(identifier)
    }
}

impl std::fmt::Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(identifier) => write!(f, "V_{identifier}"),
            Self::Constant(value) => write!(f, "{value:X}"),
            Self::Tag(tag) => write!(f, "T_{tag}"),
            Self::Path(path) => write!(f, "{path}"),
            Self::Data(data) => write!(f, "{data}"),
            Self::ReturnAddress(_) => write!(f, "RETURN_ADDRESS"),
        }
    }
}
