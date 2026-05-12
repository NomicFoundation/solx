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

impl BuildSystem {
    /// Argument passed to `npm install --global` to install this build system.
    /// Pinned to exact versions so behaviour doesn't drift when registry
    /// `latest` advances under us. `npm` itself is left unpinned because it
    /// ships with Node and we manage Node via `actions/setup-node`.
    // build-system pin: keep in sync with .github/workflows/integration-tests.yaml ("Install Yarn" step)
    pub fn to_npm_spec(&self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Yarn => "yarn@1.22.22",
            Self::Pnpm => "pnpm@10.17.1",
            Self::Bun => "bun@1.3.13",
        }
    }
}
