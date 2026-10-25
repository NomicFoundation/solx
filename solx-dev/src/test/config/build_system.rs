//!
//! `solx` project build system.
//!

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

use colored::Colorize;
use itertools::Itertools;

///
/// `solx` project build system.
///
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, serde::Deserialize)]
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

impl BuildSystem {
    ///
    /// Installs the pinned version of each of `build_systems` globally.
    ///
    /// `npm` comes with Node and needs no pin.
    ///
    pub fn install_globally(
        build_systems: BTreeSet<Self>,
        versions: &BTreeMap<Self, String>,
        suite: &str,
    ) -> anyhow::Result<()> {
        for build_system in build_systems {
            let Some(version) = versions.get(&build_system) else {
                if build_system != Self::Npm {
                    anyhow::bail!(
                        "{suite} test configuration missing `build_systems.{build_system}`"
                    );
                }
                continue;
            };
            let npm_spec = format!("{build_system}@{version}");
            let mut npm_install_build_system = Command::new("npm");
            npm_install_build_system.args(["--loglevel", "error"]);
            npm_install_build_system.arg("--force");
            npm_install_build_system.arg("--yes");
            npm_install_build_system.arg("install");
            npm_install_build_system.arg("--global");
            npm_install_build_system.arg(&npm_spec);
            crate::utils::command_with_retries(
                &mut npm_install_build_system,
                format!(
                    "{} build system {} for {suite} projects",
                    solx_utils::cargo_status_ok("Installing"),
                    build_system.to_string().bright_yellow().bold()
                )
                .as_str(),
                16,
            )?;
        }
        Ok(())
    }

    ///
    /// Installs the dependencies of the project at `directory`.
    ///
    /// A project with a lockfile gets exactly the versions it locks, so a cached setup does not
    /// depend on when or where it was made.
    ///
    pub fn install_dependencies(
        self,
        directory: &Path,
        project_name: &str,
        suite: &str,
    ) -> anyhow::Result<()> {
        crate::utils::command_with_retries(
            &mut self.install_command(directory),
            format!(
                "{} dependencies for {suite} project {project_name}",
                solx_utils::cargo_status_ok("Installing")
            )
            .as_str(),
            16,
        )
    }

    ///
    /// Adds `dependencies` to the project at `directory` as development dependencies, replacing
    /// any version the project pins.
    ///
    pub fn add_dev_dependencies(
        self,
        directory: &Path,
        dependencies: &[String],
        project_name: &str,
        suite: &str,
    ) -> anyhow::Result<()> {
        if dependencies.is_empty() {
            return Ok(());
        }
        crate::utils::command_with_retries(
            &mut self.add_dev_dependencies_command(directory, dependencies),
            format!(
                "{} dependencies with {} for {suite} project {project_name}",
                solx_utils::cargo_status_ok("Overriding"),
                dependencies
                    .iter()
                    .map(|dependency| dependency.bright_yellow().bold())
                    .join(", ")
            )
            .as_str(),
            16,
        )
    }

    ///
    /// Returns the install command for the project at `directory`.
    ///
    fn install_command(self, directory: &Path) -> Command {
        let mut command = self.command(directory);
        let is_locked = self
            .lockfiles()
            .iter()
            .any(|lockfile| directory.join(lockfile).exists());
        match (self, is_locked) {
            (Self::Npm, true) => {
                command.arg("ci");
            }
            (_, true) => {
                command.arg("install");
                command.arg("--frozen-lockfile");
            }
            (_, false) => {
                command.arg("install");
            }
        }
        command
    }

    ///
    /// Returns the command adding `dependencies` to the project at `directory`.
    ///
    fn add_dev_dependencies_command(self, directory: &Path, dependencies: &[String]) -> Command {
        let mut command = self.command(directory);
        match self {
            Self::Npm => {
                command.arg("install");
                command.arg("--save-dev");
            }
            Self::Yarn | Self::Pnpm | Self::Bun => {
                command.arg("add");
                command.arg("--dev");
            }
        }
        command.args(dependencies);
        command
    }

    ///
    /// Returns the build system command run in `directory`, with its output and script flags.
    ///
    fn command(self, directory: &Path) -> Command {
        let mut command = Command::new(self.to_string());
        command.current_dir(directory);
        match self {
            Self::Npm => {
                command.args(["--loglevel", "error"]);
                command.arg("--force");
                command.arg("--yes");
            }
            Self::Yarn => {
                command.arg("--silent");
            }
            Self::Pnpm => {
                command.arg("--ignore-scripts");
            }
            Self::Bun => {}
        }
        command
    }

    ///
    /// The lockfiles this build system installs from.
    ///
    fn lockfiles(self) -> &'static [&'static str] {
        match self {
            Self::Npm => &["package-lock.json", "npm-shrinkwrap.json"],
            Self::Yarn => &["yarn.lock"],
            Self::Pnpm => &["pnpm-lock.yaml"],
            Self::Bun => &["bun.lock", "bun.lockb"],
        }
    }
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

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use super::BuildSystem;

    fn arguments(command: &Command) -> Vec<String> {
        command
            .get_args()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn install_follows_lockfile() {
        let directory = std::env::temp_dir().join(format!(
            "solx-dev-build-system-lockfile-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(directory.as_path());
        std::fs::create_dir_all(directory.as_path()).expect("Always valid");
        for build_system in [
            BuildSystem::Npm,
            BuildSystem::Yarn,
            BuildSystem::Pnpm,
            BuildSystem::Bun,
        ] {
            assert_eq!(
                arguments(&build_system.install_command(directory.as_path()))
                    .last()
                    .map(String::as_str),
                Some("install"),
                "{build_system} without a lockfile must resolve the dependencies"
            );
        }

        for lockfile in [
            "package-lock.json",
            "yarn.lock",
            "pnpm-lock.yaml",
            "bun.lockb",
        ] {
            std::fs::write(directory.join(lockfile), "").expect("Always valid");
        }
        assert_eq!(
            arguments(&BuildSystem::Npm.install_command(directory.as_path()))
                .last()
                .map(String::as_str),
            Some("ci"),
            "npm with a lockfile must install exactly what it locks"
        );
        for build_system in [BuildSystem::Yarn, BuildSystem::Pnpm, BuildSystem::Bun] {
            assert!(
                arguments(&build_system.install_command(directory.as_path()))
                    .ends_with(&["install".to_owned(), "--frozen-lockfile".to_owned()]),
                "{build_system} with a lockfile must install exactly what it locks"
            );
        }

        std::fs::remove_dir_all(directory).expect("Always valid");
    }

    #[test]
    fn dev_dependencies_are_added() {
        let dependencies = ["hardhat@3.0.0".to_owned()];
        assert!(
            arguments(
                &BuildSystem::Npm.add_dev_dependencies_command(Path::new("."), &dependencies)
            )
            .ends_with(&[
                "install".to_owned(),
                "--save-dev".to_owned(),
                "hardhat@3.0.0".to_owned()
            ])
        );
        for build_system in [BuildSystem::Yarn, BuildSystem::Pnpm, BuildSystem::Bun] {
            assert!(
                arguments(
                    &build_system.add_dev_dependencies_command(Path::new("."), &dependencies)
                )
                .ends_with(&[
                    "add".to_owned(),
                    "--dev".to_owned(),
                    "hardhat@3.0.0".to_owned()
                ]),
                "{build_system} takes new packages through `add`"
            );
        }
    }
}
