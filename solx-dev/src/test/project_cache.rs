//!
//! Project checkouts set up once and copied for every toolchain.
//!

use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;

use colored::Colorize;

///
/// Directory of set-up project checkouts that persists across runs.
///
/// An entry counts only once its `.ready` stamp exists, so an interrupted setup is redone.
/// Entries expire by age, which also refreshes dependencies that the configuration leaves
/// unversioned.
///
pub struct ProjectCache {
    /// Holds one `<project>-<key>` directory per entry, with its `.ready` stamp beside it.
    directory: PathBuf,
}

impl ProjectCache {
    /// Age after which an entry is set up again.
    pub const MAX_AGE: Duration = Duration::from_secs(7 * 24 * 60 * 60);

    ///
    /// Opens the cache, removing expired entries and the remains of interrupted setups.
    ///
    pub fn new(directory: &Path) -> anyhow::Result<Self> {
        std::fs::create_dir_all(directory).map_err(|error| {
            anyhow::anyhow!(
                "{} project cache directory {directory:?}: {error}",
                solx_utils::cargo_status_ok("Creating"),
            )
        })?;
        let directory = crate::utils::absolute_path(directory)?;
        let cache = Self { directory };
        cache.prune(SystemTime::now())?;
        Ok(cache)
    }

    ///
    /// Returns the set-up checkout for `key_material`, running `setup` on a fresh directory
    /// first if the cache has no complete entry.
    ///
    /// `key_material` must cover every input of `setup`, as an entry is never revalidated.
    ///
    pub fn checkout(
        &self,
        project_name: &str,
        key_material: &str,
        setup: impl FnOnce(&Path) -> anyhow::Result<()>,
    ) -> anyhow::Result<PathBuf> {
        let key = solx_utils::Keccak256Hash::from_slice(key_material.as_bytes());
        let entry_name = format!("{project_name}-{}", &key.as_str()[2..18]);
        let entry = self.directory.join(entry_name.as_str());
        let stamp = Self::stamp_path(entry.as_path());

        if stamp.exists() {
            eprintln!(
                "{} setup of project {} {entry:?}. Already exists.",
                solx_utils::cargo_status_ok("Skipping"),
                project_name.bright_white().bold()
            );
            return Ok(entry);
        }

        crate::utils::remove(entry.as_path(), project_name)?;
        setup(entry.as_path())?;
        std::fs::File::create(stamp.as_path()).map_err(|error| {
            anyhow::anyhow!(
                "{} project cache stamp {stamp:?}: {error}",
                solx_utils::cargo_status_ok("Creating"),
            )
        })?;
        Ok(entry)
    }

    ///
    /// Removes entries older than `MAX_AGE` at `now`, and directories without a stamp.
    ///
    fn prune(&self, now: SystemTime) -> anyhow::Result<()> {
        let entries = std::fs::read_dir(self.directory.as_path()).map_err(|error| {
            anyhow::anyhow!(
                "{} project cache directory {:?}: {error}",
                solx_utils::cargo_status_ok("Reading"),
                self.directory
            )
        })?;
        for entry in entries {
            let path = entry?.path();
            if !path.is_dir() {
                continue;
            }
            let stamp = Self::stamp_path(path.as_path());
            let is_expired = match std::fs::metadata(stamp.as_path()) {
                Ok(metadata) => now
                    .duration_since(metadata.modified()?)
                    .is_ok_and(|age| age > Self::MAX_AGE),
                Err(_) => true,
            };
            if !is_expired {
                continue;
            }
            let entry_name = path
                .file_name()
                .expect("Always exists")
                .to_string_lossy()
                .into_owned();
            crate::utils::remove(path.as_path(), entry_name.as_str())?;
            if stamp.exists() {
                std::fs::remove_file(stamp.as_path()).map_err(|error| {
                    anyhow::anyhow!(
                        "{} project cache stamp {stamp:?}: {error}",
                        solx_utils::cargo_status_ok("Removing"),
                    )
                })?;
            }
        }
        Ok(())
    }

    ///
    /// The stamp lives beside the entry, so copies of the entry never carry it.
    ///
    fn stamp_path(entry: &Path) -> PathBuf {
        let mut stamp = entry.as_os_str().to_owned();
        stamp.push(".ready");
        PathBuf::from(stamp)
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::path::Path;
    use std::time::Duration;
    use std::time::SystemTime;

    use super::ProjectCache;

    fn test_directory(name: &str) -> std::path::PathBuf {
        let directory = std::env::temp_dir().join(format!(
            "solx-dev-project-cache-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(directory.as_path());
        directory
    }

    fn write_file(directory: &Path) -> anyhow::Result<()> {
        std::fs::create_dir_all(directory)?;
        std::fs::write(directory.join("file"), "content")?;
        Ok(())
    }

    #[test]
    fn complete_entry_is_reused() {
        let directory = test_directory("reuse");
        let cache = ProjectCache::new(directory.as_path()).expect("Always valid");

        let first = cache
            .checkout("project", "key", write_file)
            .expect("Always valid");
        let second = cache
            .checkout("project", "key", |_| panic!("setup must not rerun"))
            .expect("Always valid");
        assert_eq!(first, second);
        assert!(second.join("file").exists());

        let other = cache
            .checkout("project", "other key", write_file)
            .expect("Always valid");
        assert_ne!(first, other, "a different key must get its own entry");

        std::fs::remove_dir_all(directory).expect("Always valid");
    }

    #[test]
    fn failed_setup_is_redone() {
        let directory = test_directory("failed");
        let cache = ProjectCache::new(directory.as_path()).expect("Always valid");

        let result = cache.checkout("project", "key", |entry| {
            write_file(entry)?;
            anyhow::bail!("setup failed")
        });
        assert!(result.is_err());

        let mut is_setup_rerun = false;
        let entry = cache
            .checkout("project", "key", |entry| {
                is_setup_rerun = true;
                write_file(entry)
            })
            .expect("Always valid");
        assert!(
            is_setup_rerun,
            "a failed setup must not leave a reusable entry"
        );
        assert!(entry.join("file").exists());

        std::fs::remove_dir_all(directory).expect("Always valid");
    }

    #[test]
    fn prune_removes_expired_and_unstamped_entries() {
        let directory = test_directory("prune");
        let cache = ProjectCache::new(directory.as_path()).expect("Always valid");
        let entry = cache
            .checkout("project", "key", write_file)
            .expect("Always valid");
        write_file(directory.join("interrupted").as_path()).expect("Always valid");

        cache.prune(SystemTime::now()).expect("Always valid");
        assert!(entry.exists(), "a fresh entry must survive");
        assert!(
            !directory.join("interrupted").exists(),
            "an entry without a stamp must be removed"
        );

        cache
            .prune(SystemTime::now() + ProjectCache::MAX_AGE + Duration::from_secs(1))
            .expect("Always valid");
        assert!(!entry.exists(), "an expired entry must be removed");
        assert!(
            !ProjectCache::stamp_path(entry.as_path()).exists(),
            "an expired entry's stamp must be removed"
        );

        std::fs::remove_dir_all(directory).expect("Always valid");
    }
}
