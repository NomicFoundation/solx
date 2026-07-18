//!
//! The compilation pipeline a run used: EVM legacy assembly or Yul via-IR.
//!

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::str::FromStr;

///
/// The compilation pipeline a run used. The project suites name it by compiler
/// flag, `legacy` or `viaIR`; the tester names the same two pipelines by their
/// codegen token, `E` or `Y`.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Pipeline {
    /// EVM legacy assembly: the `legacy` flag, the `E` codegen token.
    Legacy,
    /// Yul via-IR: the `viaIR` flag, the `Y` codegen token.
    ViaIr,
}

impl FromStr for Pipeline {
    type Err = ();

    /// # Errors
    /// The token names no known pipeline flag or codegen.
    fn from_str(token: &str) -> Result<Self, Self::Err> {
        match token {
            "legacy" | "E" => Ok(Self::Legacy),
            "viaIR" | "Y" => Ok(Self::ViaIr),
            _ => Err(()),
        }
    }
}

impl Display for Pipeline {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Legacy => write!(formatter, "legacy"),
            Self::ViaIr => write!(formatter, "viaIR"),
        }
    }
}
