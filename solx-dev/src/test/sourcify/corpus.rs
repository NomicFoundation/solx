//!
//! `solx` Sourcify sweep corpus.
//!

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use super::config::corpus::Corpus;

///
/// A round-robin slice of the sorted corpus, so CI shards split it evenly.
///
#[derive(Debug, Clone, Copy)]
pub struct Slice {
    /// The number of shards.
    pub shard_count: usize,
    /// The shard to keep.
    pub shard_index: usize,
    /// The maximum number of contracts to keep, if any.
    pub limit: Option<usize>,
}

impl Slice {
    ///
    /// Sorts the corpus file names and keeps this slice of them.
    ///
    pub fn select(&self, mut names: Vec<String>) -> Vec<String> {
        names.sort();
        names
            .into_iter()
            .enumerate()
            .filter(|(index, _name)| index % self.shard_count == self.shard_index)
            .map(|(_index, name)| name)
            .take(self.limit.unwrap_or(usize::MAX))
            .collect()
    }
}

///
/// Lists the slice of a local corpus, laid out as the pinned release with a `contracts/` directory.
///
pub fn local(directory: &Path, slice: Slice) -> anyhow::Result<Vec<PathBuf>> {
    let contracts_directory = directory.join("contracts");
    let names = std::fs::read_dir(contracts_directory.as_path())
        .and_then(|entries| {
            entries
                .map(|entry| entry.map(|entry| entry.file_name().to_string_lossy().into_owned()))
                .collect::<std::io::Result<Vec<String>>>()
        })
        .map_err(|error| {
            anyhow::anyhow!("Corpus directory {contracts_directory:?} reading: {error}")
        })?
        .into_iter()
        .filter(|name| name.ends_with(".json"))
        .collect();
    let names = slice.select(names);
    if names.is_empty() {
        anyhow::bail!("No corpus records under {contracts_directory:?}");
    }
    Ok(names
        .into_iter()
        .map(|name| contracts_directory.join(name))
        .collect())
}

///
/// Downloads the pinned corpus release into `directory`, verifies it, and extracts only the
/// slice, which is a small part of the full corpus.
///
pub fn pinned(corpus: &Corpus, directory: &Path, slice: Slice) -> anyhow::Result<Vec<PathBuf>> {
    crate::utils::exists("tar")?;
    crate::utils::exists("sha256sum")?;

    let file_name = corpus
        .url
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .ok_or_else(|| anyhow::anyhow!("Corpus URL `{}` has no file name", corpus.url))?;
    let archive = directory.join(file_name);
    if archive.exists() {
        eprintln!(
            "{} corpus {archive:?}. Already exists.",
            solx_utils::cargo_status_ok("Skipping"),
        );
    } else {
        eprintln!(
            "{} corpus {} => {archive:?}",
            solx_utils::cargo_status_ok("Downloading"),
            corpus.url,
        );
        solx_compiler_downloader::Downloader::default()
            .download_file(corpus.url.as_str(), archive.as_path())?;
    }

    let mut sha256sum = Command::new("sha256sum");
    sha256sum.arg(archive.as_path());
    let checksum = output(&mut sha256sum)?;
    let checksum = checksum.split_whitespace().next().unwrap_or_default();
    if checksum != corpus.sha256 {
        anyhow::bail!(
            "Corpus {archive:?} has SHA-256 {checksum}, but the configuration pins {}",
            corpus.sha256
        );
    }

    let mut list = Command::new("tar");
    list.arg("-tJf").arg(archive.as_path());
    let names: Vec<String> = output(&mut list)?
        .lines()
        .filter(|name| name.starts_with("./contracts/") && name.ends_with(".json"))
        .map(str::to_owned)
        .collect();
    let names = slice.select(names);
    if names.is_empty() {
        anyhow::bail!("No corpus records in {archive:?}");
    }

    let extracted = directory.join("extracted");
    if extracted.exists() {
        std::fs::remove_dir_all(extracted.as_path())
            .map_err(|error| anyhow::anyhow!("Removing {extracted:?}: {error}"))?;
    }
    std::fs::create_dir_all(extracted.as_path())
        .map_err(|error| anyhow::anyhow!("Creating {extracted:?}: {error}"))?;
    let slice_list = directory.join("slice.txt");
    std::fs::write(slice_list.as_path(), names.join("\n") + "\n")
        .map_err(|error| anyhow::anyhow!("Writing {slice_list:?}: {error}"))?;
    let mut extract = Command::new("tar");
    extract
        .arg("-xJf")
        .arg(archive.as_path())
        .arg("-C")
        .arg(extracted.as_path())
        .arg("-T")
        .arg(slice_list.as_path());
    crate::utils::command(
        &mut extract,
        format!(
            "{} {} corpus records",
            solx_utils::cargo_status_ok("Extracting"),
            names.len()
        )
        .as_str(),
    )?;

    Ok(names.into_iter().map(|name| extracted.join(name)).collect())
}

///
/// Runs a command and returns its standard output.
///
fn output(command: &mut Command) -> anyhow::Result<String> {
    let output = command
        .output()
        .map_err(|error| anyhow::anyhow!("{command:?} process spawning error: {error}"))?;
    if !output.status.success() {
        anyhow::bail!(
            "{command:?} subprocess failed: {}",
            String::from_utf8_lossy(output.stderr.as_slice())
        );
    }
    String::from_utf8(output.stdout)
        .map_err(|error| anyhow::anyhow!("{command:?} output is not UTF-8: {error}"))
}
