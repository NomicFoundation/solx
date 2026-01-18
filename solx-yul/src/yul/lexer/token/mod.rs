//!
//! The token.
//!

pub mod lexeme;
pub mod location;

use std::str::FromStr;

use crate::yul::parser::debug_info::DebugInfo;

use self::lexeme::Lexeme;
use self::location::Location;

///
/// The token.
///
/// Contains a lexeme and its location.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// The token location.
    pub location: Location,
    /// The lexeme.
    pub lexeme: Lexeme,
    /// The token length, including whitespaces.
    pub length: usize,
    /// Comments associated with the token.
    /// Some of the comments contain debug info.
    pub comments: Vec<String>,
}

impl Token {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(location: Location, lexeme: Lexeme, length: usize) -> Self {
        Self {
            location,
            lexeme,
            length,
            comments: Vec::new(),
        }
    }

    ///
    /// Sets the comments associated with the token.
    ///
    pub fn set_comments(&mut self, comments: Vec<String>) {
        self.comments = comments;
    }

    ///
    /// Takes the source code ID from the comments, if any.
    ///
    pub fn take_source_id(&mut self) -> anyhow::Result<Option<usize>> {
        Ok(self
            .comments
            .drain(..)
            .map(|comment| DebugInfo::from_str(comment.as_str()))
            .collect::<Result<Vec<DebugInfo>, _>>()?
            .into_iter()
            .find_map(|debug_info| match debug_info {
                DebugInfo::UseSource { id, .. } => Some(id),
                _ => None,
            }))
    }

    ///
    /// Takes the AST ID from the comments, if any.
    ///
    pub fn take_ast_id(&mut self) -> anyhow::Result<Option<usize>> {
        Ok(self
            .comments
            .drain(..)
            .map(|comment| DebugInfo::from_str(comment.as_str()))
            .collect::<Result<Vec<DebugInfo>, _>>()?
            .into_iter()
            .find_map(|debug_info| match debug_info {
                DebugInfo::AstId(id) => Some(id),
                _ => None,
            }))
    }

    ///
    /// Takes the Solidity location from the comments, if any.
    ///
    pub fn take_solidity_location(
        &mut self,
    ) -> anyhow::Result<Option<solx_utils::DebugInfoSolcLocation>> {
        Ok(self
            .comments
            .drain(..)
            .map(|comment| DebugInfo::from_str(comment.as_str()))
            .collect::<Result<Vec<DebugInfo>, _>>()?
            .into_iter()
            .find_map(|debug_info| match debug_info {
                DebugInfo::SourceLocation(source_location) => Some(source_location),
                _ => None,
            }))
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.location, self.lexeme)
    }
}
