//!
//! The lexical token stream.
//!

pub mod comment;
pub mod hex;
pub mod integer;
pub mod string;
pub mod symbol;
pub mod word;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use self::comment::error::Error as CommentParserError;
use self::hex::error::Error as HexParserError;
use self::integer::error::Error as IntegerParserError;
use self::string::error::Error as StringParserError;
use self::symbol::error::Error as SymbolParserError;
use crate::test::function_call::parser::lexical::error::Error;
use crate::test::function_call::parser::lexical::token::Token;
use crate::test::function_call::parser::lexical::token::lexeme::Lexeme;
use crate::test::function_call::parser::lexical::token::lexeme::identifier::Identifier;
use crate::test::function_call::parser::lexical::token::lexeme::literal::Literal;
use crate::test::function_call::parser::lexical::token::location::Location;

///
/// A token stream is initialized for each input file.
///
pub struct TokenStream<'a> {
    /// The input source code string reference
    input: &'a str,
    /// The number of bytes processed so far
    offset_bytes: usize,
    /// The number of characters processed so far
    offset_chars: usize,
    /// The current position in the file
    location: Location,
    /// The queue buffer where the characters acquired with the look-ahead method are stored.
    /// If the queue is not empty, the next character will be taken therefrom.
    look_ahead: VecDeque<Token>,
}

impl<'a> TokenStream<'a> {
    /// The initial capacity of the look-ahead buffer queue.
    const LOOK_AHEAD_INITIAL_CAPACITY: usize = 16;

    ///
    /// Initializes a stream with a file identifier.
    /// The file identifier can be used to get its path from the global type index.
    ///
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            offset_bytes: 0,
            offset_chars: 0,
            location: Location::new(),
            look_ahead: VecDeque::with_capacity(Self::LOOK_AHEAD_INITIAL_CAPACITY),
        }
    }

    ///
    /// Wraps the stream into `Rc<RefCell<_>>` simplifying most of initializations.
    ///
    pub fn wrap(self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self))
    }

    ///
    /// Picks a character from the look-ahead queue.
    /// If the queue is empty, advances the stream iterator.
    ///
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Token, Error> {
        let token = match self.look_ahead.pop_front() {
            Some(token) => token,
            None => self.advance()?,
        };
        Ok(token)
    }

    ///
    /// Initializes a stream with an auto-generated file identifier.
    /// The file identifier can be used to get its path from the global type index.
    /// Used for testing purposes.
    ///
    #[allow(dead_code)]
    pub fn test(input: &'a str) -> Self {
        Self {
            input,
            offset_bytes: 0,
            offset_chars: 0,
            location: Location::new(),
            look_ahead: VecDeque::with_capacity(Self::LOOK_AHEAD_INITIAL_CAPACITY),
        }
    }

    ///
    /// The function checks if a character:
    /// 1. Is a whitespace -> skip
    /// 2. Starts a comment -> start the comment subparser
    /// 3. Starts a string literal -> start the string subparser
    /// 4. Starts a number -> start the number subparser
    /// 5. Starts a word -> start the word subparser (also tries to parse boolean or hex literal)
    /// 6. Starts a symbol -> start the operand subparser
    /// 7. Is unknown -> yield an 'invalid character' error
    ///
    /// If the end of input has been reached, an 'EOF' token is returned for consequent calls.
    ///
    fn advance(&mut self) -> Result<Token, Error> {
        while let Some(character) = self.input.chars().nth(self.offset_chars) {
            if character.is_ascii_whitespace() {
                if character == '\n' {
                    self.location.line += 1;
                    self.location.column = 1;
                } else if character != '\r' {
                    self.location.column += 1;
                }
                self.offset_bytes += 1;
                self.offset_chars += 1;
                continue;
            }

            if character == '#' {
                match self::comment::parse(&self.input[self.offset_bytes..]) {
                    Ok(output) => {
                        self.location =
                            self.location
                                .shifted_down(output.lines, output.column, output.column);
                        self.offset_bytes += output.length_bytes;
                        self.offset_chars += output.length_chars;
                        continue;
                    }
                    Err(CommentParserError::NotAComment) => {}
                }
            }

            if character == '\"' {
                match self::string::parse(&self.input[self.offset_bytes..]) {
                    Ok(output) => {
                        let location = self.location;
                        self.location =
                            self.location
                                .shifted_down(output.lines, output.column, output.column);
                        self.offset_bytes += output.length_bytes;
                        self.offset_chars += output.length_chars;
                        return Ok(Token::new(
                            Lexeme::Literal(Literal::String(output.string)),
                            location,
                        ));
                    }
                    Err(StringParserError::NotAString) => {}
                    Err(StringParserError::UnterminatedDoubleQuote { lines, column }) => {
                        return Err(Error::unterminated_double_quote_string(
                            self.location,
                            self.location.shifted_down(lines, column, column - 1),
                        ));
                    }
                }
            }

            if character.is_ascii_digit() || character == '-' {
                match self::integer::parse(&self.input[self.offset_bytes..]) {
                    Ok(output) => {
                        let location = self.location;
                        self.location.column += output.size;
                        self.offset_bytes += output.size;
                        self.offset_chars += output.size;
                        return Ok(Token::new(
                            Lexeme::Literal(Literal::Integer(output.integer)),
                            location,
                        ));
                    }
                    Err(IntegerParserError::NotAnInteger) => {}
                    Err(IntegerParserError::EmptyHexadecimalBody { offset }) => {
                        return Err(Error::unexpected_end(self.location.shifted_right(offset)));
                    }
                    Err(IntegerParserError::ExpectedOneOfDecimalOrX { found, offset }) => {
                        return Err(Error::expected_one_of_decimal_or_x_integer(
                            self.location.shifted_right(offset),
                            found,
                        ));
                    }
                    Err(IntegerParserError::ExpectedOneOfDecimal { found, offset }) => {
                        return Err(Error::expected_one_of_decimal_integer(
                            self.location.shifted_right(offset),
                            found,
                        ));
                    }
                    Err(IntegerParserError::ExpectedOneOfHexadecimal { found, offset }) => {
                        return Err(Error::expected_one_of_hexadecimal_integer(
                            self.location.shifted_right(offset),
                            found,
                        ));
                    }
                }
            }

            if Identifier::can_start_with(character) {
                let output = self::word::parse(&self.input[self.offset_bytes..]);
                if let Lexeme::Identifier(Identifier { inner }) = output.word.clone()
                    && inner == "hex"
                {
                    match self::hex::parse(&self.input[self.offset_bytes..]) {
                        Ok(output) => {
                            let location = self.location;
                            self.location.column += output.size;
                            self.offset_bytes += output.size;
                            self.offset_chars += output.size;
                            return Ok(Token::new(
                                Lexeme::Literal(Literal::Hex(output.hex)),
                                location,
                            ));
                        }
                        Err(HexParserError::NotAHex) => {}
                        Err(HexParserError::UnterminatedDoubleQuote { offset }) => {
                            return Err(Error::unterminated_double_quote_hex(
                                self.location,
                                self.location.shifted_right(offset),
                            ));
                        }
                        Err(HexParserError::ExpectedOneOfHexadecimal { found, offset }) => {
                            return Err(Error::expected_one_of_hexadecimal_hex(
                                self.location.shifted_right(offset),
                                found,
                            ));
                        }
                    }
                }
                let location = self.location;
                self.location.column += output.size;
                self.offset_bytes += output.size;
                self.offset_chars += output.size;
                return Ok(Token::new(output.word, location));
            }

            return match self::symbol::parse(&self.input[self.offset_bytes..]) {
                Ok(output) => {
                    let location = self.location;
                    self.location.column += output.size;
                    self.offset_bytes += output.size;
                    self.offset_chars += output.size;
                    Ok(Token::new(Lexeme::Symbol(output.symbol), location))
                }
                Err(SymbolParserError::InvalidCharacter { found, offset }) => Err(
                    Error::invalid_character(self.location.shifted_right(offset), found),
                ),
                Err(SymbolParserError::UnexpectedEnd) => {
                    Err(Error::unexpected_end(self.location.shifted_right(1)))
                }
            };
        }

        Ok(Token::new(Lexeme::Eof, self.location))
    }
}
