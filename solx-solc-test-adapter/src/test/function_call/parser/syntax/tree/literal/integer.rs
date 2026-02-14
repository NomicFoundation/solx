//!
//! The integer literal.
//!

use std::ops::Add;
use std::ops::BitXor;
use std::str::FromStr;

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
                let mut number = web3::types::U256::from_dec_str(inner.as_str())
                    .expect("validated by parser before semantic conversion");
                if *negative {
                    number = number.bitxor(web3::types::U256::max_value());
                    number = number.add(web3::types::U256::one());
                }
                number.to_big_endian(&mut result);
                let first = result
                    .iter()
                    .position(|byte| *byte != 0)
                    .unwrap_or(result.len() - 1);
                result = result[first..].to_owned();
            }
            LexicalIntegerLiteral::Hexadecimal(inner) => {
                web3::types::U256::from_str(inner)
                    .expect("validated by parser before semantic conversion")
                    .to_big_endian(&mut result);
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
