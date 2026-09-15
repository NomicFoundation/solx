//!
//! The solx compiler tests directory.
//!

pub mod test;

use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use crate::directories::Collection;
use crate::filters::Filters;
use crate::summary::Summary;

use self::test::SolxTest;

///
/// The solx compiler tests directory.
///
pub struct SolxDirectory;

impl Collection for SolxDirectory {
    type Test = SolxTest;

    fn read_all(
        directory_path: &Path,
        extension: &'static str,
        summary: Arc<Mutex<Summary>>,
        filters: &Filters,
    ) -> anyhow::Result<Vec<Self::Test>> {
        let mut tests = Vec::new();

        for entry in std::fs::read_dir(directory_path)? {
            let entry = entry?;
            let path = entry.path();
            let entry_type = entry.file_type().map_err(|error| {
                anyhow::anyhow!("Failed to get the type of file {path:?}: {error}")
            })?;

            if entry_type.is_dir() {
                tests.extend(Self::read_all(
                    path.as_path(),
                    extension,
                    summary.clone(),
                    filters,
                )?);
                continue;
            } else if !entry_type.is_file() {
                anyhow::bail!("Invalid type of file {path:?}");
            }

            if entry.file_name().to_string_lossy().starts_with('.') {
                continue;
            }

            let file_extension = path
                .extension()
                .ok_or_else(|| anyhow::anyhow!("Failed to get the extension of file {path:?}"))?;
            if file_extension != extension {
                continue;
            }

            if let Some(test) = SolxTest::new(path, summary.clone(), filters) {
                tests.push(test);
            }
        }

        Ok(tests)
    }
}
