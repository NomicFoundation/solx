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
/// unversioned. Pruning touches only names shaped like entries, so other directories survive.
///
pub struct ProjectCache {
    /// The cache directory.
    directory: PathBuf,
    /// Held while the cache is open, as pruning would delete another process's setup in progress.
    _lock: std::fs::File,
}

impl ProjectCache {
    /// Age after which an entry is set up again.
    const MAX_AGE: Duration = Duration::from_secs(7 * 24 * 60 * 60);

    /// Bytes of the key hash in an entry name.
    const KEY_LENGTH: usize = 8;

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
        let lock_path = directory.join(".lock");
        let lock = std::fs::File::create(lock_path.as_path()).map_err(|error| {
            anyhow::anyhow!(
                "{} project cache lock {lock_path:?}: {error}",
                solx_utils::cargo_status_ok("Creating"),
            )
        })?;
        let locked = match lock.try_lock() {
            Ok(()) => Ok(()),
            Err(std::fs::TryLockError::WouldBlock) => {
                eprintln!(
                    "{} for project cache {directory:?} used by another process",
                    solx_utils::cargo_status_ok("Waiting"),
                );
                lock.lock()
            }
            Err(std::fs::TryLockError::Error(error)) => Err(error),
        };
        locked.map_err(|error| {
            anyhow::anyhow!(
                "{} project cache lock {lock_path:?}: {error}",
                solx_utils::cargo_status_ok("Acquiring"),
            )
        })?;
        let cache = Self {
            directory,
            _lock: lock,
        };
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
        let entry_name = format!(
            "{project_name}-{}",
            hex::encode(&key.as_bytes()[..Self::KEY_LENGTH])
        );
        let entry = self.directory.join(entry_name.as_str());
        let stamp = Self::stamp_path(entry.as_path());

        if stamp.exists() && entry.is_dir() {
            eprintln!(
                "{} cached checkout of project {}",
                solx_utils::cargo_status_ok("Reusing"),
                project_name.bright_white().bold()
            );
            return Ok(entry);
        }

        Self::remove_stamp(stamp.as_path())?;
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
    /// Removes entries older than `MAX_AGE` at `now`, and entries without a stamp.
    ///
    /// An entry that cannot be removed only warns, as no project of this run needs it gone.
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
            let entry_name = path
                .file_name()
                .expect("Always exists")
                .to_string_lossy()
                .into_owned();
            if !path.is_dir() || !Self::is_entry_name(entry_name.as_str()) {
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
            // The stamp goes first, so an interrupted removal leaves an unstamped entry to prune.
            if let Err(error) = Self::remove_stamp(stamp.as_path())
                .and_then(|()| crate::utils::remove(path.as_path(), entry_name.as_str()))
            {
                eprintln!(
                    "{} project cache entry {path:?}: {error}",
                    solx_utils::cargo_status_error("Pruning"),
                );
            }
        }
        Ok(())
    }

    ///
    /// Removes `stamp` if it exists.
    ///
    fn remove_stamp(stamp: &Path) -> anyhow::Result<()> {
        if !stamp.exists() {
            return Ok(());
        }
        std::fs::remove_file(stamp).map_err(|error| {
            anyhow::anyhow!(
                "{} project cache stamp {stamp:?}: {error}",
                solx_utils::cargo_status_ok("Removing"),
            )
        })
    }

    ///
    /// Whether `name` has the `<project>-<key>` shape that `checkout` gives entries.
    ///
    fn is_entry_name(name: &str) -> bool {
        name.rsplit_once('-').is_some_and(|(project_name, key)| {
            !project_name.is_empty()
                && key.len() == Self::KEY_LENGTH * 2
                && key.bytes().all(|byte| byte.is_ascii_hexdigit())
        })
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
                std::fs::create_dir(entry)?;
                std::fs::write(entry.join("file"), "content")?;
                Ok(())
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
    fn expired_entry_is_set_up_again() {
        let directory = test_directory("expired");
        let cache = ProjectCache::new(directory.as_path()).expect("Always valid");
        let entry = cache
            .checkout("project", "key", write_file)
            .expect("Always valid");
        std::fs::File::options()
            .write(true)
            .open(ProjectCache::stamp_path(entry.as_path()))
            .expect("Always valid")
            .set_modified(SystemTime::now() - ProjectCache::MAX_AGE - Duration::from_secs(60))
            .expect("Always valid");

        drop(cache);
        let cache = ProjectCache::new(directory.as_path()).expect("Always valid");
        let mut is_setup_rerun = false;
        cache
            .checkout("project", "key", |entry| {
                is_setup_rerun = true;
                write_file(entry)
            })
            .expect("Always valid");
        assert!(is_setup_rerun, "opening the cache must expire old entries");

        std::fs::remove_dir_all(directory).expect("Always valid");
    }

    #[test]
    fn stamp_without_entry_is_set_up_again() {
        let directory = test_directory("orphan");
        let cache = ProjectCache::new(directory.as_path()).expect("Always valid");
        let entry = cache
            .checkout("project", "key", write_file)
            .expect("Always valid");
        std::fs::remove_dir_all(entry.as_path()).expect("Always valid");

        drop(cache);
        let cache = ProjectCache::new(directory.as_path()).expect("Always valid");
        let result = cache.checkout("project", "key", |entry| {
            std::fs::create_dir(entry)?;
            anyhow::bail!("setup failed")
        });
        assert!(result.is_err());
        let entry = cache
            .checkout("project", "key", write_file)
            .expect("Always valid");
        assert!(entry.join("file").exists());

        std::fs::remove_dir_all(directory).expect("Always valid");
    }

    #[test]
    fn prune_keeps_directories_it_did_not_create() {
        let directory = test_directory("foreign");
        write_file(directory.join("target").as_path()).expect("Always valid");

        ProjectCache::new(directory.as_path()).expect("Always valid");
        assert!(directory.join("target").join("file").exists());

        std::fs::remove_dir_all(directory).expect("Always valid");
    }

    #[test]
    fn prune_removes_expired_and_unstamped_entries() {
        let directory = test_directory("prune");
        let cache = ProjectCache::new(directory.as_path()).expect("Always valid");
        let entry = cache
            .checkout("project", "key", write_file)
            .expect("Always valid");
        let interrupted = directory.join("project-0123456789abcdef");
        write_file(interrupted.as_path()).expect("Always valid");

        cache.prune(SystemTime::now()).expect("Always valid");
        assert!(entry.exists(), "a fresh entry must survive");
        assert!(
            !interrupted.exists(),
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

    #[test]
    fn prune_continues_past_unremovable_entry() {
        let directory = test_directory("unremovable");
        let cache = ProjectCache::new(directory.as_path()).expect("Always valid");
        let unremovable = directory.join("project-0123456789abcdef");
        write_file(unremovable.as_path()).expect("Always valid");
        // A directory where the stamp file should be makes removing the stamp fail, even as root.
        std::fs::create_dir(ProjectCache::stamp_path(unremovable.as_path())).expect("Always valid");
        let entry = cache
            .checkout("project", "key", write_file)
            .expect("Always valid");

        cache
            .prune(SystemTime::now() + ProjectCache::MAX_AGE + Duration::from_secs(1))
            .expect("an entry that cannot be removed must not fail the run");
        assert!(unremovable.exists());
        assert!(
            !entry.exists(),
            "the other expired entries must still be removed"
        );

        std::fs::remove_dir_all(directory).expect("Always valid");
    }

    #[test]
    fn second_process_waits_for_open_cache() {
        let directory = test_directory("lock");
        let cache = ProjectCache::new(directory.as_path()).expect("Always valid");

        let (sender, receiver) = std::sync::mpsc::channel();
        let waiter = {
            let directory = directory.clone();
            std::thread::spawn(move || {
                let cache = ProjectCache::new(directory.as_path()).expect("Always valid");
                sender.send(()).expect("Always valid");
                cache
            })
        };
        assert!(
            receiver.recv_timeout(Duration::from_millis(200)).is_err(),
            "a second open must wait while the cache is open"
        );
        drop(cache);
        receiver
            .recv_timeout(Duration::from_secs(10))
            .expect("closing the cache must let the second open through");
        drop(waiter.join().expect("Always valid"));

        std::fs::remove_dir_all(directory).expect("Always valid");
    }
}
