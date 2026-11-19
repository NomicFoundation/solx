//!
//! The cache of set-up projects.
//!

use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use colored::Colorize;

///
/// The cache of set-up projects, kept across runs.
///
/// A project is set up in `<project>`, and `<project>.key` next to it holds the key it was set up for.
///
pub struct ProjectCache {
    /// The cache directory.
    directory: PathBuf,
    /// The hash of the `solx-dev` executable, which is a part of every key.
    executable_hash: String,
}

impl ProjectCache {
    ///
    /// Opens the cache and removes the projects that are not in `projects`.
    ///
    pub fn new<P>(directory: &Path, projects: &BTreeMap<String, P>) -> anyhow::Result<Self> {
        std::fs::create_dir_all(directory)
            .with_context(|| format!("Creating project cache {directory:?}"))?;
        let directory = crate::utils::absolute_path(directory)?;

        for entry in std::fs::read_dir(directory.as_path())? {
            let path = entry?.path();
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
            let project_name = file_name.trim_end_matches(".key");
            if projects.contains_key(project_name) {
                continue;
            }
            if path.is_dir() {
                crate::utils::remove(path.as_path(), project_name)?;
            } else {
                std::fs::remove_file(path.as_path())
                    .with_context(|| format!("Removing {path:?}"))?;
            }
        }

        let executable = std::fs::read(std::env::current_exe()?)?;
        Ok(Self {
            directory,
            executable_hash: solx_utils::Keccak256Hash::from_slice(executable.as_slice())
                .to_string(),
        })
    }

    ///
    /// Returns the project checkout, setting it up first if it is not set up for `key`.
    ///
    pub fn checkout(
        &self,
        project_name: &str,
        key: &str,
        set_up: impl FnOnce(&Path) -> anyhow::Result<()>,
    ) -> anyhow::Result<PathBuf> {
        let key = format!("{}{key}", self.executable_hash);
        let checkout = self.directory.join(project_name);
        let stamp = self.directory.join(format!("{project_name}.key"));
        if checkout.exists()
            && std::fs::read_to_string(stamp.as_path()).is_ok_and(|stamped_key| stamped_key == key)
        {
            eprintln!(
                "{} set-up project {}",
                solx_utils::cargo_status_ok("Reusing"),
                project_name.bright_white().bold()
            );
            return Ok(checkout);
        }

        // The stamp is removed before the checkout and written after the setup,
        // so it never marks a partial checkout.
        if stamp.exists() {
            std::fs::remove_file(stamp.as_path()).with_context(|| format!("Removing {stamp:?}"))?;
        }
        crate::utils::remove(checkout.as_path(), project_name)?;
        set_up(checkout.as_path())?;
        std::fs::write(stamp.as_path(), key).with_context(|| format!("Writing {stamp:?}"))?;
        Ok(checkout)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::ProjectCache;

    const PROJECT_NAME: &str = "project";

    fn set_up(checkout: &Path) -> anyhow::Result<()> {
        Ok(std::fs::create_dir(checkout)?)
    }

    fn open(directory: &Path, project_names: &[&str]) -> ProjectCache {
        let projects = project_names
            .iter()
            .map(|project_name| (project_name.to_string(), ()))
            .collect();
        ProjectCache::new(directory, &projects).expect("Always valid")
    }

    #[test]
    fn sets_up_once_per_key() {
        let directory = tempfile::tempdir().expect("Always valid");
        let cache = open(directory.path(), &[PROJECT_NAME]);

        let mut set_up_count = 0;
        for key in ["key", "key", "other key"] {
            cache
                .checkout(PROJECT_NAME, key, |checkout| {
                    set_up_count += 1;
                    set_up(checkout)
                })
                .expect("Always valid");
        }
        assert_eq!(set_up_count, 2);
    }

    #[test]
    fn sets_up_again_after_a_failed_setup() {
        let directory = tempfile::tempdir().expect("Always valid");
        let cache = open(directory.path(), &[PROJECT_NAME]);

        cache
            .checkout(PROJECT_NAME, "key", |checkout| {
                set_up(checkout).and_then(|()| set_up(checkout))
            })
            .expect_err("Always fails");
        let mut is_set_up = false;
        cache
            .checkout(PROJECT_NAME, "key", |checkout| {
                is_set_up = true;
                set_up(checkout)
            })
            .expect("Always valid");
        assert!(is_set_up);
    }

    #[test]
    fn removes_projects_no_longer_configured() {
        let directory = tempfile::tempdir().expect("Always valid");
        open(directory.path(), &[PROJECT_NAME])
            .checkout(PROJECT_NAME, "key", set_up)
            .expect("Always valid");

        open(directory.path(), &[]);
        assert_eq!(
            std::fs::read_dir(directory.path())
                .expect("Always valid")
                .count(),
            0
        );
    }
}
