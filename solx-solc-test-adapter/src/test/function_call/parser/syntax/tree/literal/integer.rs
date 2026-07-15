//!
//! The integer literal.
//!

use alloy_primitives::U256;

use crate::test::function_call::parser::lexical::IntegerLiteral as LexicalIntegerLiteral;
use crate::test::function_call::parser::lexical::Location;
use crate::test::function_call::parser::syntax::tree::literal::alignment::Alignment;

///
/// The integer literal.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Literal {
    /// The location of the syntax construction.
    pub location: Location,
    /// The inner lexical literal.
    pub inner: LexicalIntegerLiteral,
    /// The alignment.
    pub alignment: Alignment,
}

impl Literal {
    ///
    /// Creates a new literal value.
    ///
    pub fn new(location: Location, inner: LexicalIntegerLiteral, alignment: Alignment) -> Self {
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
        match &self.inner {
            LexicalIntegerLiteral::Decimal { inner, negative } => {
                let mut number = U256::from_str_radix(inner.as_str(), 10)
                    .expect("validated by parser before semantic conversion");
                if *negative {
                    number = !number;
                    number += U256::from(1);
                }
                result.copy_from_slice(number.to_be_bytes_vec().as_slice());
                let first = result
                    .iter()
                    .position(|byte| *byte != 0)
                    .unwrap_or(result.len() - 1);
                result = result[first..].to_owned();
            }
            LexicalIntegerLiteral::Hexadecimal(inner) => {
                let number = crate::u256_from_hex_str(inner)
                    .expect("validated by parser before semantic conversion");
                result.copy_from_slice(number.to_be_bytes_vec().as_slice());
                result = result[result.len() - inner.len().div_ceil(2)..].to_owned();
            }
        }
        if self.alignment == Alignment::Left {
            result.extend(vec![0; solx_utils::BYTE_LENGTH_FIELD - result.len()]);
        } else {
            let mut zeroes = vec![0; solx_utils::BYTE_LENGTH_FIELD - result.len()];
            zeroes.extend(result);
            result = zeroes;
        }
        result
    }
}
