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
    UseSource(Vec<(usize, String)>),
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
                let mut sources = Vec::with_capacity(parts.len() - 2);
                for part in parts[2..].iter() {
                    let src_parts = part.splitn(2, ':').collect::<Vec<&str>>();
                    let id = match src_parts[0].parse::<usize>() {
                        Ok(id) => id,
                        Err(_) => break,
                    };
                    let path = match src_parts.get(1) {
                        Some(path) => path.trim_matches('"').to_string(),
                        None => break,
                    };
                    sources.push((id, path));
                }
                Ok(Self::UseSource(sources))
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
