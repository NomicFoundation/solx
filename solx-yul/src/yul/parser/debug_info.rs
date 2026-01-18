//!
//! Debug info statement.
//!

use std::str::FromStr;

use crate::yul::lexer::token::lexeme::comment::single_line::Comment as SingleLineComment;

///
/// Debug info statement.
///
#[derive(Debug)]
pub enum DebugInfo {
    /// Source code identifier.
    UseSource {
        /// Source code ID.
        id: usize,
        /// Source code path.
        path: String,
    },
    /// AST node identifier.
    AstId(usize),
    /// Source code location.
    SourceLocation(solx_utils::DebugInfoSolcLocation),
    /// Unknown debug info statement.
    Unknown,
}

impl FromStr for DebugInfo {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let parts = string.split_whitespace().collect::<Vec<&str>>();

        if parts[0] != SingleLineComment::DEBUG_INFO_START {
            anyhow::bail!("Not a debug info comment");
        }

        match parts[1] {
            "@use-src" => {
                let src_parts = parts[2].splitn(2, ':').collect::<Vec<&str>>();
                let id = src_parts[0].parse::<usize>()?;
                let path = src_parts[1].trim_matches('"').to_string();
                Ok(Self::UseSource { id, path })
            }
            "@ast-id" => {
                let id = parts[2].parse::<usize>()?;
                Ok(Self::AstId(id))
            }
            "@src" => {
                let location = solx_utils::DebugInfoSolcLocation::parse(
                    parts[2],
                    solx_utils::DebugInfoSolcLocationOrdering::Yul,
                )?;
                Ok(Self::SourceLocation(location))
            }
            _ => Ok(Self::Unknown),
        }
    }
}
