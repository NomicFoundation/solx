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
    /// Argument passed to `npm install --global` to install this build system,
    /// or `None` to skip the global install entirely. Pinned to exact versions
    /// so behaviour doesn't drift when the registry's `latest` tag advances
    /// under us. `Npm` returns `None` because `npm install -g npm` would
    /// reinstall the very tool we're bootstrapping from — the Node-bundled
    /// npm (managed via `actions/setup-node`) is used directly instead.
    // build-system pin: keep in sync with .github/workflows/integration-tests.yaml ("Install JS package managers" step)
    pub fn to_npm_spec(&self) -> Option<&'static str> {
        match self {
            Self::Npm => None,
            Self::Yarn => Some("yarn@1.22.22"),
            Self::Pnpm => Some("pnpm@10.17.1"),
            Self::Bun => Some("bun@1.3.13"),
        }
    }
}
