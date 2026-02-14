//!
//! The boolean literal.
//!

use crate::test::function_call::parser::lexical::Location;
use crate::test::function_call::parser::syntax::tree::literal::alignment::Alignment;

///
/// The boolean literal.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Literal {
    /// The location of the syntax construction.
    pub location: Location,
    /// The inner lexical literal.
    pub inner: bool,
    /// The alignment.
    pub alignment: Alignment,
}

impl Literal {
    ///
    /// Creates a new literal value.
    ///
    pub fn new(location: Location, inner: bool, alignment: Alignment) -> Self {
        Self {
            location,
            inner,
            alignment,
        }
    }

    ///
    /// Converts literal to bytes.
    ///
    pub fn as_bytes_be(&self) -> Vec<u8> {
        let mut result = vec![0u8; solx_utils::BYTE_LENGTH_FIELD];
        if self.inner {
            if self.alignment == Alignment::Left {
                result[0] = 1;
            } else {
                *result
                    .last_mut()
                    .expect("vector initialized with fixed non-zero length") = 1;
            }
        }
        result
    }
}
