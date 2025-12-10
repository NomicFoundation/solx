//!
//! `solx` Hardhat project build system.
//!

///
/// `solx` Hardhat project build system.
///
#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildSystem {
    /// Eponymous build system.
    #[default]
    Npm,
    /// Eponymous build system.
    Yarn,
    /// Eponymous build system.
    Pnpm,
    /// Eponymous build system.
    Bun,
}

impl std::fmt::Display for BuildSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Npm => write!(f, "npm"),
            Self::Yarn => write!(f, "yarn"),
            Self::Pnpm => write!(f, "pnpm"),
            Self::Bun => write!(f, "bun"),
        }
    }
}
