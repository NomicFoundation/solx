//!
//! The string literal.
//!

use std::str::FromStr;

use crate::test::function_call::parser::lexical::Location;
use crate::test::function_call::parser::lexical::StringLiteral as LexicalStringLiteral;
use crate::test::function_call::parser::syntax::tree::literal::alignment::Alignment;

///
/// The string literal.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Literal {
    /// The location of the syntax construction.
    pub location: Location,
    /// The inner lexical literal.
    pub inner: LexicalStringLiteral,
    /// The alignment.
    pub alignment: Alignment,
}

impl Literal {
    ///
    /// Creates a new literal value.
    ///
    pub fn new(location: Location, inner: LexicalStringLiteral, alignment: Alignment) -> Self {
        Self {
            location,
            inner,
            alignment,
        }
    }

    ///
    /// Converts literal to bytes.
    ///
    pub fn as_bytes_be(&self) -> anyhow::Result<Vec<u8>> {
        /// The helper state enum for converting a string literal to bytes.
        #[derive(PartialEq)]
        enum State {
            /// The initial state or character has been processed so far.
            Char,
            /// The `\` has been processed so far.
            Backslash,
            /// The `\x` has been processed so far.
            HexFirst,
            /// The first hexadecimal symbol has been processed so far.
            HexSecond,
        }
        let mut result = Vec::new();
        let mut state = State::Char;
        let mut code = String::new();
        for char in self.inner.inner.chars() {
            match state {
                State::Char => match char {
                    '\\' => {
                        state = State::Backslash;
                    }
                    _ => {
                        result.extend(char.to_string().as_bytes());
                    }
                },
                State::Backslash => match char {
                    'x' => {
                        state = State::HexFirst;
                    }
                    '0' => {
                        result.push(0);
                        state = State::Char;
                    }
                    _ => {
                        anyhow::bail!("Invalid escape sequence: expected 'x' or '0' after '\\'");
                    }
                },
                State::HexFirst => {
                    code.push(char);
                    state = State::HexSecond;
                }
                State::HexSecond => {
                    code.push(char);
                    let code_u8 = web3::types::U256::from_str(code.as_str())
                        .map_err(|error| anyhow::anyhow!("Invalid escape sequence: {error}"))?
                        .as_u32() as u8;
                    code.clear();
                    result.push(code_u8);
                    state = State::Char;
                }
            }
        }
        if state != State::Char {
            anyhow::bail!("Unterminated escape sequence");
        }
        let mut pad_len = 0;
        if result.len() % solx_utils::BYTE_LENGTH_FIELD != 0 || result.is_empty() {
            pad_len = solx_utils::BYTE_LENGTH_FIELD - result.len() % solx_utils::BYTE_LENGTH_FIELD;
        }
        if self.alignment == Alignment::Right {
            let mut zeroes = vec![0; pad_len];
            zeroes.extend(result);
            result = zeroes;
        } else {
            result.extend(vec![0; pad_len]);
        }
        Ok(result)
    }
}
