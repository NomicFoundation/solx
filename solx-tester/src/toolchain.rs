//!
//! The compiler toolchain to compile tests with.
//!

///
/// The compiler toolchain to compile tests with.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize)]
pub enum Toolchain {
    /// The upstream `solc` compiler.
    Solc,
    /// The default LLVM-based compiler: `solx` for EVM.
    Solx,
    /// The forked `solc` compiler with MLIR.
    SolxMlir,
}

impl std::str::FromStr for Toolchain {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "solc" => Ok(Self::Solc),
            "solx" => Ok(Self::Solx),
            "solx-mlir" => Ok(Self::SolxMlir),
            string => anyhow::bail!(
                "Unknown toolchain `{}`. Supported toolchains: {}",
                string,
                vec![Self::Solx, Self::Solc, Self::SolxMlir]
                    .into_iter()
                    .map(|element| element.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

impl std::fmt::Display for Toolchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Solc => write!(f, "solc"),
            Self::Solx => write!(f, "solx"),
            Self::SolxMlir => write!(f, "solx-mlir"),
        }
    }
}
