//!
//! `solx` Sourcify sweep corpus.
//!

pub mod slice;

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use super::config::corpus::Corpus;

use self::slice::Slice;

///
/// Lists the slice of a local corpus, laid out as the pinned release with a `contracts/` directory.
///
pub fn local(directory: &Path, slice: &Slice) -> anyhow::Result<Vec<PathBuf>> {
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
pub fn pinned(corpus: &Corpus, directory: &Path, slice: &Slice) -> anyhow::Result<Vec<PathBuf>> {
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
    let checksum = crate::utils::command_with_output(
        &mut sha256sum,
        format!(
            "{} corpus checksum",
            solx_utils::cargo_status_ok("Verifying")
        )
        .as_str(),
    )?;
    let checksum = checksum.split_whitespace().next().unwrap_or_default();
    if checksum != corpus.sha256 {
        anyhow::bail!(
            "Corpus {archive:?} has SHA-256 {checksum}, but the configuration pins {}",
            corpus.sha256
        );
    }

    let mut list = Command::new("tar");
    list.arg("-tJf").arg(archive.as_path());
    let names: Vec<String> = crate::utils::command_with_output(
        &mut list,
        format!("{} corpus records", solx_utils::cargo_status_ok("Listing")).as_str(),
    )?
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
