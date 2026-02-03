//!
//! The hex literal.
//!

use std::str::FromStr;

use crate::test::function_call::parser::lexical::HexLiteral as LexicalHexLiteral;
use crate::test::function_call::parser::lexical::Location;
use crate::test::function_call::parser::syntax::tree::literal::alignment::Alignment;

///
/// The hex literal.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Literal {
    /// The location of the syntax construction.
    pub location: Location,
    /// The inner lexical literal.
    pub inner: LexicalHexLiteral,
    /// The alignment.
    pub alignment: Alignment,
}

impl Literal {
    ///
    /// Creates a new literal value.
    ///
    pub fn new(location: Location, inner: LexicalHexLiteral, alignment: Alignment) -> Self {
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
        web3::types::U256::from_str(self.inner.inner.as_str())
            .expect("Always valid")
            .to_big_endian(&mut result);
        result = result[result.len() - self.inner.inner.len().div_ceil(2)..].to_owned();
        result
    }
}
