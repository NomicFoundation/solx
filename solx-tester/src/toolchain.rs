//!
//! The compiler toolchain to compile tests with.
//!

use std::path::Path;

///
/// The compiler toolchain to compile tests with.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize)]
pub enum Toolchain {
    /// The upstream `solc` compiler.
    Solc,
    /// The default LLVM-based compiler: `solx` for EVM.
    Solx,
}

impl Toolchain {
    ///
    /// Auto-detects the toolchain from the compiler's version output.
    ///
    /// Returns `Solx` if the version output starts with "solx,",
    /// otherwise returns `Solc`.
    ///
    pub fn detect(path: &Path) -> anyhow::Result<Self> {
        let mut command = std::process::Command::new(path);
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());
        command.arg("--version");

        let process = command
            .spawn()
            .map_err(|error| anyhow::anyhow!("{path:?} subprocess spawning: {error}"))?;
        let result = process
            .wait_with_output()
            .map_err(|error| anyhow::anyhow!("{path:?} subprocess output reading: {error:?}"))?;
        if !result.status.success() {
            anyhow::bail!(
                "{path:?} subprocess exit code {:?}:\n{}\n{}",
                result.status.code(),
                String::from_utf8_lossy(result.stdout.as_slice()),
                String::from_utf8_lossy(result.stderr.as_slice()),
            );
        }

        let stdout = String::from_utf8_lossy(result.stdout.as_slice());
        let first_line = stdout.lines().next().unwrap_or_default();

        if first_line.starts_with("solx") {
            Ok(Self::Solx)
        } else {
            Ok(Self::Solc)
        }
    }
}

impl std::str::FromStr for Toolchain {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match string {
            "solc" => Ok(Self::Solc),
            "solx" => Ok(Self::Solx),
            string => anyhow::bail!(
                "Unknown toolchain `{}`. Supported toolchains: {}",
                string,
                vec![Self::Solx, Self::Solc]
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
        }
    }
}
