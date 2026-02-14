//!
//! The lexical token integer literal lexeme.
//!

use std::fmt;

///
/// The lexical integer literal.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Integer {
    /// An decimal literal, like `42`.
    Decimal {
        /// The inner without sign.
        inner: String,
        /// The is negative flag.
        negative: bool,
    },
    /// A hexadecimal literal, like `0xffff`.
    Hexadecimal(String),
}

impl Integer {
    /// Characters allowed in the decimal literal.
    pub const CHARACTERS_DECIMAL: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
    /// Characters allowed in the hexadecimal literal.
    pub const CHARACTERS_HEXADECIMAL: [char; 22] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'A', 'B',
        'C', 'D', 'E', 'F',
    ];

    /// The minus character.
    pub const CHARACTER_MINUS: char = '-';
    /// The zero character at the beginning hexadecimal literals.
    pub const CHARACTER_ZERO: char = '0';
    /// The hexadecimal literal second character.
    pub const CHARACTER_INITIAL_HEXADECIMAL: char = 'x';

    ///
    /// Creates a decimal value.
    ///
    pub fn new_decimal(inner: String, negative: bool) -> Self {
        Self::Decimal { inner, negative }
    }

    ///
    /// Creates a hexadecimal value.
    ///
    pub fn new_hexadecimal(inner: String) -> Self {
        Self::Hexadecimal(inner)
    }
}

impl From<Integer> for String {
    fn from(value: Integer) -> String {
        match value {
            Integer::Decimal {
                mut inner,
                negative,
            } => {
                if negative {
                    inner.insert(0, Integer::CHARACTER_MINUS);
                }
                inner
            }
            Integer::Hexadecimal(inner) => inner,
        }
    }
}

impl fmt::Display for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string: String = self.to_owned().into();
        write!(f, "{string}")
    }
}
