//!
//! The keyword lexeme.
//!

use crate::yul::lexer::token::Token;
use crate::yul::lexer::token::lexeme::Lexeme;
use crate::yul::lexer::token::lexeme::literal::Literal;
use crate::yul::lexer::token::lexeme::literal::boolean::Boolean as BooleanLiteral;
use crate::yul::lexer::token::location::Location;

///
/// The keyword lexeme.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keyword {
    /// The `function` keyword.
    Function,
    /// The `let` keyword.
    Let,
    /// The `if` keyword.
    If,
    /// The `switch` keyword.
    Switch,
    /// The `case` keyword.
    Case,
    /// The `default` keyword.
    Default,
    /// The `for` keyword.
    For,
    /// The `break` keyword.
    Break,
    /// The `continue` keyword.
    Continue,
    /// The `leave` keyword.
    Leave,
    /// The `true` keyword.
    True,
    /// The `false` keyword.
    False,
    /// The `bool` keyword.
    Bool,
    /// The `int{N}` keyword.
    Int(usize),
    /// The `uint{N}` keyword.
    Uint(usize),
}

impl Keyword {
    ///
    /// Parses the keyword, returning it as a token.
    ///
    pub fn parse(input: &str) -> Option<Token> {
        let keyword = Self::parse_keyword(input)?;
        let lexeme = match BooleanLiteral::try_from(keyword) {
            Ok(literal) => Lexeme::Literal(Literal::Boolean(literal)),
            Err(keyword) => Lexeme::Keyword(keyword),
        };

        let length = lexeme.to_string().len();
        if length != input.len() {
            return None;
        }

        Some(Token::new(Location::new(0, length), lexeme, length))
    }

    ///
    /// Parses the keyword itself.
    ///
    fn parse_keyword(input: &str) -> Option<Self> {
        if !input.starts_with(Self::can_begin) {
            return None;
        }
        let end = input.find(Self::cannot_continue).unwrap_or(input.len());
        let input = &input[..end];

        if let Some(input) = input.strip_prefix("int")
            && let Ok(bitlength) = input.parse::<usize>()
        {
            return Some(Self::Int(bitlength));
        }

        if let Some(input) = input.strip_prefix("uint")
            && let Ok(bitlength) = input.parse::<usize>()
        {
            return Some(Self::Uint(bitlength));
        }

        Some(match input {
            "function" => Self::Function,
            "let" => Self::Let,
            "if" => Self::If,
            "switch" => Self::Switch,
            "case" => Self::Case,
            "default" => Self::Default,
            "for" => Self::For,
            "break" => Self::Break,
            "continue" => Self::Continue,
            "leave" => Self::Leave,
            "true" => Self::True,
            "false" => Self::False,
            "bool" => Self::Bool,

            _ => return None,
        })
    }

    ///
    /// Checks whether the character can begin a keyword.
    ///
    pub fn can_begin(character: char) -> bool {
        character.is_alphabetic()
    }

    ///
    /// Checks whether the character can continue a keyword.
    ///
    pub fn can_continue(character: char) -> bool {
        Self::can_begin(character) || character.is_numeric()
    }

    ///
    /// Checks whether the character cannot continue a keyword.
    ///
    pub fn cannot_continue(character: char) -> bool {
        !Self::can_continue(character)
    }
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Function => write!(f, "function"),
            Self::Let => write!(f, "let"),
            Self::If => write!(f, "if"),
            Self::Switch => write!(f, "switch"),
            Self::Case => write!(f, "case"),
            Self::Default => write!(f, "default"),
            Self::For => write!(f, "for"),
            Self::Break => write!(f, "break"),
            Self::Continue => write!(f, "continue"),
            Self::Leave => write!(f, "leave"),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::Bool => write!(f, "bool"),
            Self::Int(bitlength) => write!(f, "int{bitlength}"),
            Self::Uint(bitlength) => write!(f, "uint{bitlength}"),
        }
    }
}
