//!
//! The lexical parser error.
//!

use crate::test::function_call::parser::lexical::token::lexeme::literal::hex::Hex;
use crate::test::function_call::parser::lexical::token::lexeme::literal::integer::Integer;
use crate::test::function_call::parser::lexical::token::location::Location;

///
/// The lexical parser error.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The string has not been terminated.
    UnterminatedDoubleQuoteString {
        /// The location where the unterminated string starts.
        start: Location,
        /// The location where the unterminated string ends.
        end: Location,
    },

    /// A non-decimal or `x` character is found after a single `0` in an integer literal.
    ExpectedOneOfDecimalOrXInteger {
        /// The location of the invalid character.
        location: Location,
        /// The allowed characters.
        expected: String,
        /// The invalid character.
        found: char,
    },
    /// A non-decimal character is found in a decimal literal.
    ExpectedOneOfDecimalInteger {
        /// The location of the invalid character.
        location: Location,
        /// The allowed characters.
        expected: String,
        /// The invalid character.
        found: char,
    },
    /// A non-hexadecimal character is found in a hexadecimal literal.
    ExpectedOneOfHexadecimalInteger {
        /// The location of the invalid character.
        location: Location,
        /// The allowed characters.
        expected: String,
        /// The invalid character.
        found: char,
    },

    /// The hex has not been terminated.
    UnterminatedDoubleQuoteHex {
        /// The location where the unterminated hex starts.
        start: Location,
        /// The location where the unterminated hex ends.
        end: Location,
    },
    /// A non-hexadecimal character is found in a hex literal.
    ExpectedOneOfHexadecimalHex {
        /// The location of the invalid character.
        location: Location,
        /// The allowed characters.
        expected: String,
        /// The invalid character.
        found: char,
    },

    /// An unexpected character forbidden in the current state.
    InvalidCharacter {
        /// The location of the invalid character.
        location: Location,
        /// The invalid character.
        found: char,
    },
    /// Unable to finish a lexeme.
    UnexpectedEnd {
        /// The location of the end of the unfinished lexeme.
        location: Location,
    },
}

impl Error {
    ///
    /// A shortcut constructor.
    ///
    pub fn unterminated_double_quote_string(start: Location, end: Location) -> Self {
        Self::UnterminatedDoubleQuoteString { start, end }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn expected_one_of_decimal_or_x_integer(location: Location, found: char) -> Self {
        let mut expected = Integer::CHARACTERS_DECIMAL.to_vec();
        expected.push(Integer::CHARACTER_INITIAL_HEXADECIMAL);
        Self::ExpectedOneOfDecimalOrXInteger {
            location,
            expected: Self::join_expected(expected),
            found,
        }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn expected_one_of_decimal_integer(location: Location, found: char) -> Self {
        Self::ExpectedOneOfDecimalInteger {
            location,
            expected: Self::join_expected(Integer::CHARACTERS_DECIMAL.to_vec()),
            found,
        }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn expected_one_of_hexadecimal_integer(location: Location, found: char) -> Self {
        Self::ExpectedOneOfHexadecimalInteger {
            location,
            expected: Self::join_expected(Integer::CHARACTERS_HEXADECIMAL.to_vec()),
            found,
        }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn unterminated_double_quote_hex(start: Location, end: Location) -> Self {
        Self::UnterminatedDoubleQuoteHex { start, end }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn expected_one_of_hexadecimal_hex(location: Location, found: char) -> Self {
        Self::ExpectedOneOfHexadecimalHex {
            location,
            expected: Self::join_expected(Hex::CHARACTERS.to_vec()),
            found,
        }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn invalid_character(location: Location, found: char) -> Self {
        Self::InvalidCharacter { location, found }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn unexpected_end(location: Location) -> Self {
        Self::UnexpectedEnd { location }
    }

    ///
    /// Converts a group of characters into a comma-separated list.
    ///
    /// E.g. ['a', 'b', 'c'] turns into `a`, `b`, `c`.
    ///
    fn join_expected(chars: Vec<char>) -> String {
        chars
            .into_iter()
            .map(|symbol| format!("`{symbol}`"))
            .collect::<Vec<String>>()
            .join(", ")
    }
}
