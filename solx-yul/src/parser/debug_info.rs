//!
//! Debug info statement.
//!

use std::str::FromStr;

use crate::lexer::token::lexeme::comment::single_line::Comment as SingleLineComment;

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
        let parts = string.splitn(3, ' ').collect::<Vec<&str>>();

        if parts[0] != SingleLineComment::DEBUG_INFO_START {
            anyhow::bail!("Not a debug info comment");
        }

        match parts[1] {
            "@use-src" => {
                let mut sources = Vec::with_capacity(parts.len() - 2);
                for part in parts[2].split(", ") {
                    let src_parts = part.splitn(2, ':').collect::<Vec<&str>>();
                    let id = src_parts[0]
                        .parse::<usize>()
                        .map_err(|_| anyhow::anyhow!("Invalid Yul @use-src source ID"))?;
                    let path = (*src_parts
                        .get(1)
                        .ok_or_else(|| anyhow::anyhow!("Invalid Yul @use-src source path"))?)
                    .trim_matches('"')
                    .to_owned();
                    sources.push((id, path));
                }
                Ok(Self::UseSource(sources))
            }
            "@ast-id" => {
                let id = parts[2]
                    .split_whitespace()
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("Invalid Yul @ast-id tag"))?
                    .parse::<usize>()?;
                Ok(Self::AstId(id))
            }
            "@src" => {
                let location = solx_utils::DebugInfoSolcLocation::parse(
                    parts[2]
                        .split_whitespace()
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("Invalid Yul @src tag"))?,
                    solx_utils::DebugInfoSolcLocationOrdering::Yul,
                )?;
                Ok(Self::SourceLocation(location))
            }
            _ => Ok(Self::Unknown),
        }
    }
}
