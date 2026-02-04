//!
//! Boost library download and build utilities for solc.
//!

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

/// Default Boost version.
pub const DEFAULT_BOOST_VERSION: &str = "1.83.0";

/// Boost download URL template.
const BOOST_DOWNLOAD_URL: &str = "https://archives.boost.io/release";

/// Boost libraries required for solc.
const BOOST_LIBRARIES: [&str; 10] = [
    "filesystem",
    "system",
    "program_options",
    "test",
    "thread",
    "date_time",
    "regex",
    "chrono",
    "random",
    "atomic",
];

///
/// Boost configuration.
///
#[derive(Debug, Clone)]
pub struct BoostConfig {
    /// Boost version (e.g., "1.83.0").
    pub version: String,
    /// Base directory where Boost is installed.
    pub base_dir: PathBuf,
}

impl BoostConfig {
    ///
    /// Creates a new Boost configuration.
    ///
    pub fn new(version: String, base_dir: PathBuf) -> Self {
        Self { version, base_dir }
    }

    ///
    /// Returns the Boost CMake root directory.
    ///
    pub fn cmake_root(&self) -> PathBuf {
        self.base_dir
            .join("lib")
            .join("cmake")
            .join(format!("Boost-{}", self.version))
    }

    ///
    /// Returns the Boost library directory.
    ///
    pub fn lib_dir(&self) -> PathBuf {
        self.base_dir.join("lib")
    }

    ///
    /// Returns the Boost include directory.
    ///
    pub fn include_dir(&self) -> PathBuf {
        self.base_dir.join("include")
    }

    ///
    /// Returns the Boost include directory for Windows.
    ///
    /// Windows uses a versioned include directory (e.g., `include/boost-1_83`).
    ///
    pub fn windows_include_dir(&self) -> PathBuf {
        let short_version = self
            .version
            .split('.')
            .take(2)
            .collect::<Vec<_>>()
            .join("_");
        self.base_dir
            .join("include")
            .join(format!("boost-{short_version}"))
    }

    ///
    /// Returns the source directory for the downloaded Boost.
    ///
    pub fn source_dir(&self, working_dir: &Path) -> PathBuf {
        let filename = format!("boost_{}", self.version.replace('.', "_"));
        working_dir.join(filename)
    }

    ///
    /// Returns the archive filename.
    ///
    pub fn archive_filename(&self) -> String {
        format!("boost_{}.tar.gz", self.version.replace('.', "_"))
    }

    ///
    /// Returns the download URL.
    ///
    pub fn download_url(&self) -> String {
        format!(
            "{BOOST_DOWNLOAD_URL}/{}/source/{}",
            self.version,
            self.archive_filename()
        )
    }
}

///
/// Downloads and builds Boost.
///
pub fn download_and_build(working_dir: &Path, boost_config: &BoostConfig) -> anyhow::Result<()> {
    // Skip if already built
    if boost_config.lib_dir().exists() {
        eprintln!(
            "Boost {} already exists at {}",
            boost_config.version,
            boost_config.base_dir.display()
        );
        return Ok(());
    }

    eprintln!("Downloading Boost {}...", boost_config.version);

    // Download
    let archive_path = working_dir.join(boost_config.archive_filename());
    if !archive_path.exists() {
        download(&boost_config.download_url(), &archive_path)?;
    }

    // Extract
    let source_dir = boost_config.source_dir(working_dir);
    if !source_dir.exists() {
        extract(&archive_path, working_dir)?;
    }

    // Bootstrap
    bootstrap(&source_dir, &boost_config.base_dir)?;

    // Build
    build(&source_dir)?;

    eprintln!(
        "Boost {} built successfully at {}",
        boost_config.version,
        boost_config.base_dir.display()
    );

    Ok(())
}

///
/// Downloads a file from URL.
///
fn download(url: &str, output_path: &Path) -> anyhow::Result<()> {
    let mut curl = Command::new("curl");
    curl.arg("-L");
    curl.arg("-o");
    curl.arg(output_path);
    curl.arg(url);

    crate::utils::command(&mut curl, "Downloading Boost")?;
    Ok(())
}

///
/// Extracts a tar.gz archive.
///
fn extract(archive_path: &Path, output_dir: &Path) -> anyhow::Result<()> {
    let mut tar = Command::new("tar");
    tar.arg("xzf");
    tar.arg(archive_path);
    tar.current_dir(output_dir);

    crate::utils::command(&mut tar, "Extracting Boost")?;
    Ok(())
}

///
/// Runs Boost bootstrap script.
///
fn bootstrap(source_dir: &Path, install_prefix: &Path) -> anyhow::Result<()> {
    let bootstrap_script = if cfg!(target_os = "windows") {
        "bootstrap.sh" // MSYS2 uses bash
    } else {
        "./bootstrap.sh"
    };

    let mut bootstrap = Command::new(bootstrap_script);
    bootstrap.current_dir(source_dir);
    bootstrap.arg(format!("--prefix={}", install_prefix.display()));

    // On macOS x86_64, specify Python version for compatibility
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        bootstrap.arg("--with-python-version=2.7");
    }

    crate::utils::command(&mut bootstrap, "Boost bootstrap")?;
    Ok(())
}

///
/// Builds Boost with b2.
///
fn build(source_dir: &Path) -> anyhow::Result<()> {
    let job_count = std::thread::available_parallelism()
        .map(|parallelism| parallelism.get())
        .unwrap_or(1);

    let with_libraries: Vec<String> = BOOST_LIBRARIES
        .iter()
        .map(|lib| format!("--with-{lib}"))
        .collect();

    let mut b2 = Command::new("./b2");
    b2.current_dir(source_dir);
    b2.arg("-d0"); // Suppress output
    b2.arg("link=static");
    b2.arg("runtime-link=static");
    b2.arg("variant=release");
    b2.arg("threading=multi");
    b2.arg("address-model=64");
    b2.args(&with_libraries);
    b2.arg(format!("-j{job_count}"));
    b2.arg("install");

    crate::utils::command(&mut b2, "Boost b2 build")?;
    Ok(())
}
