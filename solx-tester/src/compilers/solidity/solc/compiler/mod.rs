//!
//! The Solidity compiler.
//!

use std::io::Write;

use solx_standard_json::Input as StandardJsonInput;
use solx_standard_json::Output as StandardJsonOutput;

///
/// The Solidity compiler.
///
pub struct Compiler {
    /// The executable name.
    pub executable: String,
}

impl Compiler {
    /// The first version of `solc`, where Yul codegen is considered robust enough.
    pub const FIRST_YUL_VERSION: semver::Version = semver::Version::new(0, 8, 0);

    /// The first version of `solc`, where `--via-ir` codegen mode is supported.
    pub const FIRST_VIA_IR_VERSION: semver::Version = semver::Version::new(0, 8, 13);

    ///
    /// A shortcut constructor.
    ///
    /// Different tools may use different `executable` names. For example, the integration tester
    /// uses `solc-<version>` format.
    ///
    pub fn new(executable: String) -> anyhow::Result<Self> {
        if let Err(error) = which::which(executable.as_str()) {
            anyhow::bail!("The `{executable}` executable not found in ${{PATH}}: {error}");
        }
        Ok(Self { executable })
    }

    ///
    /// Compiles the Solidity `--standard-json` input into Yul IR.
    ///
    pub fn standard_json(
        &mut self,
        input: StandardJsonInput,
        base_path: Option<String>,
        include_paths: Vec<String>,
        allow_paths: Option<String>,
    ) -> anyhow::Result<StandardJsonOutput> {
        let mut command = std::process::Command::new(self.executable.as_str());
        command.stdin(std::process::Stdio::piped());
        command.stdout(std::process::Stdio::piped());
        command.arg("--standard-json");

        if let Some(base_path) = base_path {
            command.arg("--base-path");
            command.arg(base_path);
        }
        for include_path in include_paths.into_iter() {
            command.arg("--include-path");
            command.arg(include_path);
        }
        if let Some(allow_paths) = allow_paths {
            command.arg("--allow-paths");
            command.arg(allow_paths);
        }

        let input_json = serde_json::to_vec(&input).expect("Always valid");

        let process = command.spawn().map_err(|error| {
            anyhow::anyhow!("{} subprocess spawning error: {:?}", self.executable, error)
        })?;
        process
            .stdin
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("{} stdin getting error", self.executable))?
            .write_all(input_json.as_slice())
            .map_err(|error| {
                anyhow::anyhow!("{} stdin writing error: {:?}", self.executable, error)
            })?;

        let output = process.wait_with_output().map_err(|error| {
            anyhow::anyhow!("{} subprocess output error: {:?}", self.executable, error)
        })?;
        if !output.status.success() {
            anyhow::bail!(
                "{} error: {}",
                self.executable,
                String::from_utf8_lossy(output.stderr.as_slice())
            );
        }

        let output: StandardJsonOutput =
            solx_utils::deserialize_from_slice(output.stdout.as_slice()).map_err(|error| {
                anyhow::anyhow!(
                    "{} subprocess output parsing error: {}\n{}",
                    self.executable,
                    error,
                    solx_utils::deserialize_from_slice::<serde_json::Value>(
                        output.stdout.as_slice()
                    )
                    .map(|json| serde_json::to_string_pretty(&json).expect("Always valid"))
                    .unwrap_or_else(
                        |_| String::from_utf8_lossy(output.stdout.as_slice()).to_string()
                    ),
                )
            })?;

        Ok(output)
    }
}
