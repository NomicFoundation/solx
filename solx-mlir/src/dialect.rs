//!
//! MLIR dialects exposed via `--emit-mlir`.
//!

/// MLIR dialects produced by the Sol-to-LLVM pass pipeline, selectable via `--emit-mlir`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, clap::ValueEnum,
)]
#[serde(rename_all = "lowercase")]
pub enum Dialect {
    /// Sol dialect, captured before the conversion passes run.
    Sol,
    /// LLVM dialect, captured after the pass pipeline.
    Llvm,
}
