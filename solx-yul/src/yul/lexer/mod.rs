//!
//! The compiler lexer.
//!

pub mod error;
pub mod token;

#[cfg(test)]
mod tests;

use self::error::Error;
use self::token::Token;
use self::token::lexeme::Lexeme;
use self::token::lexeme::comment::Comment;
use self::token::lexeme::identifier::Identifier;
use self::token::lexeme::literal::integer::Integer as IntegerLiteral;
use self::token::lexeme::literal::string::String as StringLiteral;
use self::token::lexeme::symbol::Symbol;
use self::token::location::Location;

///
/// The compiler lexer.
///
pub struct Lexer<'a> {
    /// The input source code.
    input: &'a str,
    /// The number of characters processed so far.
    offset: usize,
    /// The current location.
    location: Location,
    /// The peeked lexeme, waiting to be fetched.
    peeked: Option<Token>,
}

impl<'a> Lexer<'a> {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            offset: 0,
            location: Location::default(),
            peeked: None,
        }
    }

    ///
    /// Advances the lexer, returning the next lexeme.
    ///
    pub fn next(&mut self) -> Result<Token, Error> {
        if let Some(peeked) = self.peeked.take() {
            return Ok(peeked);
        }

        while self.offset < self.input.len() {
            let input = &self.input[self.offset..];

            if input.starts_with(|character| char::is_ascii_whitespace(&character)) {
                if input.starts_with('\n') {
                    self.location.line += 1;
                    self.location.column = 1;
                } else if !input.starts_with('\r') {
                    self.location.column += 1;
                }
                self.offset += 1;
                continue;
            }

            if let Some(token) = Comment::parse(input) {
                self.offset += token.length;
                self.location
                    .shift_down(token.location.line, token.location.column);
                continue;
            }

            if let Some(mut token) = StringLiteral::parse(input) {
                token.location = self.location;

                self.offset += token.length;
                self.location.shift_right(token.length);
                return Ok(token);
            }

            if let Some(mut token) = IntegerLiteral::parse(input) {
                token.location = self.location;

                self.offset += token.length;
                self.location.shift_right(token.length);
                return Ok(token);
            }

            if let Some(mut token) = Identifier::parse(input) {
                token.location = self.location;

                self.offset += token.length;
                self.location.shift_right(token.length);
                return Ok(token);
            }

            if let Some(mut token) = Symbol::parse(input) {
                token.location = self.location;

                self.offset += token.length;
                self.location.shift_right(token.length);
                return Ok(token);
            }

            let end = self.input[self.offset..]
                .find(char::is_whitespace)
                .unwrap_or(self.input.len());
            return Err(Error::InvalidLexeme {
                location: self.location,
                sequence: self.input[self.offset..self.offset + end].to_owned(),
            });
        }

        Ok(Token::new(self.location, Lexeme::EndOfFile, 0))
    }

    ///
    /// Peeks the next lexeme without advancing the iterator.
    ///
    pub fn peek(&mut self) -> Result<Token, Error> {
        match self.peeked.as_ref() {
            Some(peeked) => Ok(peeked.clone()),
            None => {
                let peeked = self.next()?;
                self.peeked = Some(peeked.clone());
                Ok(peeked)
            }
        }
    }
}
