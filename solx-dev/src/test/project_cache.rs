//!
//! Set-up project checkouts kept across runs.
//!

use std::path::Path;
use std::path::PathBuf;

use colored::Colorize;

///
/// Beside each `<project>` checkout, a `<project>.key` stamp holds the key it was set up for,
/// so copies of the checkout never carry it.
///
pub struct ProjectCache {
    /// Holds the checkouts and their stamps.
    directory: PathBuf,
}

impl ProjectCache {
    ///
    /// Opens the cache in `directory`.
    ///
    pub fn new(directory: &Path) -> anyhow::Result<Self> {
        std::fs::create_dir_all(directory).map_err(|error| {
            anyhow::anyhow!(
                "{} project cache directory {directory:?}: {error}",
                solx_utils::cargo_status_ok("Creating"),
            )
        })?;
        Ok(Self {
            directory: crate::utils::absolute_path(directory)?,
        })
    }

    ///
    /// Returns the checkout of `project_name`, running `set_up` on a fresh directory first
    /// unless the checkout was set up for `key`.
    ///
    /// `key` must change with every input of `set_up`, as a checkout is never revalidated.
    ///
    pub fn checkout(
        &self,
        project_name: &str,
        key: &str,
        set_up: impl FnOnce(&Path) -> anyhow::Result<()>,
    ) -> anyhow::Result<PathBuf> {
        let checkout = self.directory.join(project_name);
        let stamp = self.directory.join(format!("{project_name}.key"));
        if std::fs::read_to_string(stamp.as_path()).is_ok_and(|stamped_key| stamped_key == key) {
            eprintln!(
                "{} set-up project {}",
                solx_utils::cargo_status_ok("Reusing"),
                project_name.bright_white().bold()
            );
            return Ok(checkout);
        }

        // The stamp goes before the checkout and comes after the setup, so it never marks a
        // partial checkout.
        if stamp.exists() {
            std::fs::remove_file(stamp.as_path()).map_err(|error| {
                anyhow::anyhow!(
                    "{} project cache stamp {stamp:?}: {error}",
                    solx_utils::cargo_status_ok("Removing"),
                )
            })?;
        }
        crate::utils::remove(checkout.as_path(), project_name)?;
        set_up(checkout.as_path())?;
        std::fs::write(stamp.as_path(), key).map_err(|error| {
            anyhow::anyhow!(
                "{} project cache stamp {stamp:?}: {error}",
                solx_utils::cargo_status_ok("Writing"),
            )
        })?;
        Ok(checkout)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::ProjectCache;

    const PROJECT_NAME: &str = "project";

    const KEY: &str = "key";

    const OTHER_KEY: &str = "other key";

    fn set_up(checkout: &Path) -> anyhow::Result<()> {
        Ok(std::fs::create_dir(checkout)?)
    }

    #[test]
    fn checkout_sets_up_once_per_key() {
        let directory = tempfile::tempdir().expect("Always valid");
        let cache = ProjectCache::new(directory.path()).expect("Always valid");

        let mut set_up_count = 0;
        for key in [KEY, KEY, OTHER_KEY] {
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
    fn checkout_sets_up_again_after_a_failed_set_up() {
        let directory = tempfile::tempdir().expect("Always valid");
        let cache = ProjectCache::new(directory.path()).expect("Always valid");

        cache
            .checkout(PROJECT_NAME, KEY, |checkout| {
                set_up(checkout).and_then(|()| set_up(checkout))
            })
            .expect_err("Always fails");
        let mut is_set_up = false;
        cache
            .checkout(PROJECT_NAME, KEY, |checkout| {
                is_set_up = true;
                set_up(checkout)
            })
            .expect("Always valid");
        assert!(is_set_up);
    }
}
