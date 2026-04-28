//!
//! MLIR dialects exposed via `--emit-mlir`.
//!

use std::fmt;
use std::str::FromStr;

/// MLIR dialects produced by the Sol → LLVM pass pipeline and selectable
/// via `--emit-mlir`.
///
/// Variants are listed in pipeline order: [`Dialect::Sol`] is captured
/// before lowering, [`Dialect::Llvm`] after the pass pipeline.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, clap::ValueEnum,
)]
#[serde(rename_all = "lowercase")]
pub enum Dialect {
    /// Sol dialect, captured before lowering passes run.
    Sol,
    /// LLVM dialect, captured after the pass pipeline.
    Llvm,
}

impl Dialect {
    /// Returns the canonical lowercase identifier of the dialect.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sol => "sol",
            Self::Llvm => "llvm",
        }
    }
}

impl FromStr for Dialect {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> anyhow::Result<Self> {
        match value {
            "sol" => Ok(Self::Sol),
            "llvm" => Ok(Self::Llvm),
            other => anyhow::bail!("unknown MLIR dialect: {other}"),
        }
    }
}

impl fmt::Display for Dialect {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
