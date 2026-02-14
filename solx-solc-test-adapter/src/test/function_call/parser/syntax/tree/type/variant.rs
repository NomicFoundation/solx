//!
//! The type variant.
//!

use crate::test::function_call::parser::syntax::tree::r#type::Type;

///
/// The type variant.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Variant {
    /// `bool` in the source code.
    Boolean,
    /// `string` in the source code.
    String,
    /// `address` in the source code.
    Address,
    /// `function` in the source code.
    Function,
    /// `uint{N}` in the source code.
    IntegerUnsigned {
        /// The unsigned integer bit-length.
        bit_length: usize,
    },
    /// `int{N}` in the source code.
    IntegerSigned {
        /// The signed integer bit-length.
        bit_length: usize,
    },
    /// `bytes{N}` in the source code.
    Bytes {
        /// The bytes byte-length.
        byte_length: Option<usize>,
    },
    /// `{type}[{expression}]` in the source code.
    Array {
        /// The array element type.
        inner: Box<Type>,
        /// The array size.
        size: Option<String>,
    },
    /// `({type1}, {type2}, ...)` in the source code.
    Tuple {
        /// The tuple element types.
        inners: Vec<Type>,
    },
}

impl Variant {
    ///
    /// A shortcut constructor.
    ///
    pub fn boolean() -> Self {
        Self::Boolean
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn string() -> Self {
        Self::String
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn address() -> Self {
        Self::Address
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn function() -> Self {
        Self::Function
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn integer_unsigned(bit_length: usize) -> Self {
        Self::IntegerUnsigned { bit_length }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn integer_signed(bit_length: usize) -> Self {
        Self::IntegerSigned { bit_length }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn bytes(byte_length: Option<usize>) -> Self {
        Self::Bytes { byte_length }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn array(inner: Type, size: Option<String>) -> Self {
        Self::Array {
            inner: Box::new(inner),
            size,
        }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn tuple(inners: Vec<Type>) -> Self {
        Self::Tuple { inners }
    }
}
